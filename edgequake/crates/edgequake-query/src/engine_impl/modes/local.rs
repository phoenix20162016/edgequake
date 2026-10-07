//! Local query mode — entity-centric retrieval with graph context.

use std::collections::HashMap;
use std::sync::Arc;

use crate::context::QueryContext;
use crate::error::Result;
use crate::helpers::{build_entity_from_node, build_relationship_from_edge};
use crate::keywords::ExtractedKeywords;
use crate::vector_filter::{filter_by_type, VectorType};

use edgequake_storage::traits::VectorStorage;

use super::super::{QueryEmbeddings, QueryEngine};
use super::chunk_retrieval::append_score_ranked_chunks;
use super::make_scope_metadata_filter;

impl QueryEngine {
    /// Local mode with workspace-specific vector storage.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine_impl) async fn query_local_with_vector_storage(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        // Document IDs to restrict vector search to (SPEC-031 Tier 1 pre-filter).
        allowed_document_ids: Option<&[String]>,
        vector_storage: &Arc<dyn VectorStorage>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        let retrieval_config = self.config_with_max_chunks(max_chunks);
        let mut context = QueryContext::new();
        // SPEC-031: push document scope filter to SQL layer (Tier 1 pre-filter)
        // SPEC-058: push vector_type=entity to SQL (Naive already pushes chunk).
        let mf = make_scope_metadata_filter(
            tenant_id.clone(),
            workspace_id.clone(),
            allowed_document_ids,
            Some("entity"),
            Some(embeddings.model.as_str()).filter(|s| !s.is_empty()),
        );

        let vector_results = vector_storage
            .query_filtered(
                &embeddings.low_level,
                self.config.max_entities * 3,
                None,
                mf.as_ref(),
            )
            .await?;

        let entity_vectors = filter_by_type(vector_results, VectorType::Entity);

        let entity_scores: HashMap<String, f32> = entity_vectors
            .iter()
            .filter(|r| r.score >= self.config.min_score)
            .filter_map(|r| {
                let bare = crate::helpers::decode_entity_name_from_result(&r.id, &r.metadata);
                let bare = if bare.is_empty() { r.id.clone() } else { bare };
                let graph_id =
                    crate::helpers::graph_entity_id_for_workspace(&bare, workspace_id.as_deref());
                if graph_id.is_empty() {
                    None
                } else {
                    Some((graph_id, r.score))
                }
            })
            .collect();

        let entity_ids: Vec<String> = entity_vectors
            .iter()
            .filter(|r| r.score >= self.config.min_score)
            .filter_map(|r| {
                let bare = crate::helpers::decode_entity_name_from_result(&r.id, &r.metadata);
                let bare = if bare.is_empty() { r.id.clone() } else { bare };
                let graph_id =
                    crate::helpers::graph_entity_id_for_workspace(&bare, workspace_id.as_deref());
                if graph_id.is_empty() {
                    None
                } else {
                    Some(graph_id)
                }
            })
            .take(self.config.max_entities)
            .collect();

        if !entity_ids.is_empty() {
            let graph = self.graph_read();
            let (nodes_map, degrees) = tokio::join!(
                graph.get_nodes_batch(&entity_ids),
                graph.node_degrees_batch(&entity_ids),
            );

            let nodes_map = nodes_map?;
            let degrees: HashMap<String, usize> = degrees?.into_iter().collect();

            for id in &entity_ids {
                if let Some(node) = nodes_map.get(id) {
                    let degree = degrees.get(id).copied().unwrap_or(0);
                    let entity_score = entity_scores.get(id).copied().unwrap_or(0.0);
                    let entity = build_entity_from_node(id, &node.properties, degree, entity_score);
                    context.add_entity(entity);
                }
            }

            // Hydration hole: ANN returned ids that load zero graph nodes.
            // Fall through to lexical/topic admit — do not treat as a successful
            // vector hit that skips fallback, and do not use popular hubs.
            if context.entities.is_empty() {
                tracing::debug!(
                    ann_ids = entity_ids.len(),
                    workspace_id = ?workspace_id,
                    "local: ANN entity ids hydrated to zero nodes — lexical admit"
                );
            } else {
                if crate::keyword_boost::keyword_lexical_boost_enabled() {
                    let kw = keywords.all_keywords();
                    crate::keyword_boost::boost_entities_by_keywords(&mut context.entities, &kw);
                }

                let edges = crate::graph_expand::expand_neighborhood_edges(
                    &graph,
                    &entity_ids,
                    self.config.graph_depth,
                    self.config.max_relationships,
                    self.config.graph_walk,
                    tenant_id.as_deref(),
                    workspace_id.as_deref(),
                )
                .await?;

                for edge in edges {
                    let rel =
                        build_relationship_from_edge(&edge.source, &edge.target, &edge.properties);
                    context.add_relationship(rel);
                }
            }
        }

        // Topic admit (env-gated) then exact-label graph admit before popular hubs.
        crate::topic_entity_admit::admit_topic_entities(
            self.graph_read(),
            &mut context,
            query_text,
            keywords,
            workspace_id.as_deref(),
        )
        .await?;

        let label_candidates =
            crate::graph_seed_admit::exact_label_candidates(query_text, keywords);
        if context.entities.is_empty() {
            let _ = crate::graph_seed_admit::admit_into_context(
                self.graph_read(),
                self.kv_storage.as_deref(),
                &mut context,
                &[],
                query_text,
                keywords,
                tenant_id.as_deref(),
                workspace_id.as_deref(),
            )
            .await?;
        }

        // Popular hubs only when ANN empty, no label candidates, and admit found nothing.
        // Named-entity questions must not get unrelated high-degree nodes.
        if context.entities.is_empty() && entity_ids.is_empty() && label_candidates.is_empty() {
            if crate::keyword_boost::popular_node_fallback_enabled() {
                tracing::debug!(
                    workspace_id = ?workspace_id,
                    "OODA-231: No entity vectors found, falling back to popular entities from graph"
                );
                crate::retrieval_telemetry::mark_popular_node_fallback(&mut context, "local");
                let graph = self.graph_read();
                let popular = graph
                    .get_popular_nodes_with_degree(
                        self.config.max_entities,
                        None,
                        None,
                        tenant_id.as_deref(),
                        workspace_id.as_deref(),
                    )
                    .await?;

                let fallback_entity_ids: Vec<String> =
                    popular.iter().map(|(n, _)| n.id.clone()).collect();

                for (node, degree) in popular {
                    let entity = build_entity_from_node(&node.id, &node.properties, degree, 0.0);
                    context.add_entity(entity);
                }

                if !fallback_entity_ids.is_empty() {
                    let edges = crate::graph_expand::expand_neighborhood_edges(
                        &graph,
                        &fallback_entity_ids,
                        self.config.graph_depth,
                        self.config.max_relationships,
                        self.config.graph_walk,
                        tenant_id.as_deref(),
                        workspace_id.as_deref(),
                    )
                    .await?;
                    for edge in edges {
                        let rel = build_relationship_from_edge(
                            &edge.source,
                            &edge.target,
                            &edge.properties,
                        );
                        context.add_relationship(rel);
                    }
                }
            } else {
                tracing::debug!(
                    workspace_id = ?workspace_id,
                    "No entity vectors; popular-node fallback disabled (EDGEQUAKE_POPULAR_NODE_FALLBACK=0)"
                );
            }
        } else if !label_candidates.is_empty() && context.entities.is_empty() {
            tracing::debug!(
                candidates = label_candidates.len(),
                "skip popular-node fallback: exact label candidates present"
            );
        }

        // Chunk fetch SSOT uses vector_type=chunk (not the entity ANN `mf` above).
        // 078 R3: Mix post_truncate skips per-arm pick (re-pick after E/R truncate).
        if !crate::kg_chunk_pick::arm_kg_chunks_skipped() {
            let (chunks, sparse_outcome) = append_score_ranked_chunks(
                self,
                &context,
                query_text,
                &embeddings.low_level,
                tenant_id,
                workspace_id,
                vector_storage,
                &retrieval_config,
                allowed_document_ids,
                Some(embeddings.model.as_str()).filter(|s| !s.is_empty()),
                "local",
            )
            .await?;

            crate::retrieval_telemetry::mark_sparse_outcome(
                &mut context,
                sparse_outcome.as_str(),
                sparse_outcome.is_fts_fallback(),
            );
            for chunk in chunks {
                context.add_chunk(chunk);
            }
        }

        // 073: soft-label relationship endpoints for Connections / LLM context.
        if let Err(e) = crate::helpers::resolve_relationship_endpoint_labels(
            &self.graph_read(),
            &mut context.relationships,
        )
        .await
        {
            tracing::warn!(error = %e, "failed to resolve relationship endpoint labels");
        }

        Ok(context)
    }
}
