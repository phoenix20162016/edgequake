//! Shared chunk retrieval for local/global query modes.
//!
//! SPEC-046 P0.3: supports LightRAG-style `related_chunk_number` and
//! `kg_chunk_pick_method` (vector | weight).
//!
//! SPEC-047 / 021 L-A3: candidate chunk ids are intersected with allowed
//! documents before vector fetch (fail-closed under document_scope).
//!
//! **065 / First Principles:** chunk ANN/id fetch always uses `vector_type=chunk`.
//! Callers must not pass entity/relationship filters into this path — local/global
//! keep typed filters only for their KG ANN steps.

use std::sync::Arc;

use edgequake_storage::traits::{MetadataFilter, VectorStorage};

use crate::context::{QueryContext, RetrievedChunk};
use crate::engine_impl::{QueryEngine, QueryEngineConfig};
use crate::error::Result;
use crate::graph_ppr::GraphWalkMode;
use crate::helpers::build_chunk_from_result;
use crate::kg_chunk_pick::{
    collect_kg_chunk_ids_scoped, lr_vector_budget_enabled, lr_vector_chunk_budget,
    pick_chunks_by_bipartite_ppr, pick_chunks_by_weight, KgChunkPickMethod,
};
use crate::lineage_scope::filter_chunk_ids_by_allowed_docs;
use edgequake_storage::traits::GraphEdge;

/// SSOT MetadataFilter for chunk vector fetch (tenant/workspace/docs + type=chunk).
///
/// Never reuse entity/relationship ANN filters here — wrong `vector_type` returns
/// zero chunk rows and empties Broad/Focused page evidence.
pub(super) fn chunk_fetch_metadata_filter(
    tenant_id: Option<String>,
    workspace_id: Option<String>,
    allowed_document_ids: Option<&[String]>,
    embedding_model: Option<&str>,
) -> Option<MetadataFilter> {
    super::make_scope_metadata_filter(
        tenant_id,
        workspace_id,
        allowed_document_ids,
        Some("chunk"),
        embedding_model,
    )
}

#[allow(clippy::too_many_arguments)] // retrieval pipeline mirrors QueryEngine workspace arity
pub(super) async fn append_score_ranked_chunks(
    engine: &QueryEngine,
    context: &QueryContext,
    query_text: &str,
    query_embedding: &[f32],
    tenant_id: Option<String>,
    workspace_id: Option<String>,
    vector_storage: &Arc<dyn VectorStorage>,
    retrieval_config: &QueryEngineConfig,
    allowed_document_ids: Option<&[String]>,
    embedding_model: Option<&str>,
    log_label: &str,
) -> Result<(
    Vec<RetrievedChunk>,
    crate::sparse_retrieval::SparseRetrievalOutcome,
)> {
    // Owned filter — do not borrow caller's entity/relationship mf.
    let mf_chunk_owned = chunk_fetch_metadata_filter(
        tenant_id.clone(),
        workspace_id.clone(),
        allowed_document_ids,
        embedding_model,
    );
    let mf_chunk = mf_chunk_owned.as_ref();

    let related_n = retrieval_config.related_chunk_number;
    let topic_ids = crate::topic_entity_admit::topic_chunk_ids_from_context(context);

    // Dual-node (EQ-046-17): bipartite PPR over entity relations ∪ mentions.
    // Falls back to lite entity-score projection when no relations are present.
    let mut raw_ids = if retrieval_config.graph_walk == GraphWalkMode::Ppr {
        let entity_edges: Vec<GraphEdge> = context
            .relationships
            .iter()
            .map(|r| GraphEdge {
                source: r.source.clone(),
                target: r.target.clone(),
                properties: std::collections::HashMap::new(),
            })
            .collect();
        let ranked =
            pick_chunks_by_bipartite_ppr(context, &entity_edges, retrieval_config.max_chunks);
        if ranked.is_empty() {
            collect_kg_chunk_ids_scoped(context, related_n, allowed_document_ids)
        } else {
            filter_chunk_ids_by_allowed_docs(&ranked, allowed_document_ids)
        }
    } else {
        match retrieval_config.kg_chunk_pick_method {
            KgChunkPickMethod::Weight => {
                let weighted = pick_chunks_by_weight(context, retrieval_config.max_chunks);
                if weighted.is_empty() {
                    collect_kg_chunk_ids_scoped(context, related_n, allowed_document_ids)
                } else {
                    filter_chunk_ids_by_allowed_docs(&weighted, allowed_document_ids)
                }
            }
            KgChunkPickMethod::Vector => {
                // 024 Q2: LightRAG VECTOR uses the full entity-linked pool, then
                // cosine-takes `related_chunk_number * n_entities / 2`.
                let per_entity_cap = if lr_vector_budget_enabled() {
                    0
                } else {
                    related_n
                };
                collect_kg_chunk_ids_scoped(context, per_entity_cap, allowed_document_ids)
            }
        }
    };

    // 038 SELECT: Acc default graph_walk=Ppr — PPR shortlist can omit topic
    // entity chunks even after admit. Union topic source_chunk_ids into the
    // candidate pool (document-scoped), then pin them after fetch.
    if !topic_ids.is_empty() {
        let scoped_topic = filter_chunk_ids_by_allowed_docs(&topic_ids, allowed_document_ids);
        let have: std::collections::HashSet<&str> = raw_ids.iter().map(|s| s.as_str()).collect();
        let mut prepend: Vec<String> = scoped_topic
            .into_iter()
            .filter(|id| !have.contains(id.as_str()))
            .collect();
        if !prepend.is_empty() {
            prepend.append(&mut raw_ids);
            raw_ids = prepend;
        }
    }

    let chunk_ids_vec = raw_ids;
    let lr_budget = lr_vector_budget_enabled()
        && retrieval_config.kg_chunk_pick_method == KgChunkPickMethod::Vector
        && retrieval_config.graph_walk != GraphWalkMode::Ppr;
    let vector_take = if lr_budget {
        // LightRAG: do not pre-clamp to max_chunks before cosine pick.
        lr_vector_chunk_budget(related_n, context.entities.len()).max(1)
    } else {
        retrieval_config.max_chunks
    };

    tracing::info!(
        total_chunk_ids = chunk_ids_vec.len(),
        entity_count = context.entities.len(),
        relationship_count = context.relationships.len(),
        pick_method = retrieval_config.kg_chunk_pick_method.as_str(),
        graph_walk = ?retrieval_config.graph_walk,
        related_chunk_number = related_n,
        lr_vector_budget = lr_budget,
        vector_take,
        scoped = allowed_document_ids.is_some(),
        log_label,
        "OODA-230: chunk collection (workspace)"
    );

    if chunk_ids_vec.is_empty() {
        return Ok((
            Vec::new(),
            crate::sparse_retrieval::SparseRetrievalOutcome::VectorOnly,
        ));
    }

    // Preserve ranked order for Weight and PPR dual-node picks; Vector uses score filter.
    let preserve_order = retrieval_config.kg_chunk_pick_method == KgChunkPickMethod::Weight
        || retrieval_config.graph_walk == GraphWalkMode::Ppr;

    let results = if preserve_order {
        let unordered = vector_storage
            .query_filtered(
                query_embedding,
                chunk_ids_vec.len().max(retrieval_config.max_chunks),
                Some(&chunk_ids_vec),
                mf_chunk,
            )
            .await?;
        let mut by_id: std::collections::HashMap<String, _> =
            unordered.into_iter().map(|r| (r.id.clone(), r)).collect();
        chunk_ids_vec
            .iter()
            .filter_map(|id| by_id.remove(id))
            .collect::<Vec<_>>()
    } else {
        vector_storage
            .query_filtered(query_embedding, vector_take, Some(&chunk_ids_vec), mf_chunk)
            .await?
    };

    // 038: pin topic-entity chunks to the front of the fetched shortlist
    // (works for VECTOR and PPR — Acc fairness uses graph_walk=Ppr).
    let mut results = results;
    if !topic_ids.is_empty() {
        crate::topic_entity_admit::pin_topic_chunks_in_results(&mut results, &topic_ids, |r| {
            r.id.as_str()
        });
        let have: std::collections::HashSet<&str> = results.iter().map(|r| r.id.as_str()).collect();
        let missing: Vec<String> = topic_ids
            .iter()
            .filter(|id| chunk_ids_vec.iter().any(|c| c == *id) && !have.contains(id.as_str()))
            .cloned()
            .collect();
        if !missing.is_empty() {
            if let Ok(extra) = vector_storage
                .query_filtered(query_embedding, missing.len(), Some(&missing), mf_chunk)
                .await
            {
                let mut merged = extra;
                merged.append(&mut results);
                results = merged;
                crate::topic_entity_admit::pin_topic_chunks_in_results(
                    &mut results,
                    &topic_ids,
                    |r| r.id.as_str(),
                );
            }
        }
    }

    tracing::debug!(
        candidates = chunk_ids_vec.len(),
        returned = results.len(),
        topic_pinned = topic_ids.len(),
        log_label,
        "OODA-231: Chunk retrieval result"
    );

    let (mut chunks, outcome) = if crate::sparse_retrieval::bm25_retrieval_enabled(retrieval_config)
    {
        crate::sparse_retrieval::fuse_vector_and_bm25_chunks(
            query_text,
            &results,
            vector_storage,
            mf_chunk,
            engine.reranker.as_deref(),
            engine.kv_storage.as_deref(),
            retrieval_config,
        )
        .await
    } else {
        // D-39: always enforce min_score (preserve_order must not bypass threshold).
        (
            results
                .iter()
                .filter(|r| r.score >= retrieval_config.min_score)
                .take(vector_take)
                .map(build_chunk_from_result)
                .collect(),
            crate::sparse_retrieval::SparseRetrievalOutcome::VectorOnly,
        )
    };

    crate::chunk_hydration::hydrate_retrieved_chunks(engine.kv_storage.as_deref(), &mut chunks)
        .await;

    Ok((chunks, outcome))
}

#[cfg(test)]
mod tests {
    use super::chunk_fetch_metadata_filter;

    #[test]
    fn chunk_fetch_filter_always_typed_chunk() {
        let mf = chunk_fetch_metadata_filter(
            Some("t1".into()),
            Some("ws1".into()),
            Some(&["doc-a".to_string(), "doc-b".to_string()]),
            Some("mistral-embed"),
        )
        .expect("filter must be Some when type=chunk is set");
        assert_eq!(mf.vector_type.as_deref(), Some("chunk"));
        assert_eq!(mf.tenant_id.as_deref(), Some("t1"));
        assert_eq!(mf.workspace_id.as_deref(), Some("ws1"));
        assert_eq!(mf.embedding_model.as_deref(), Some("mistral-embed"));
        assert_eq!(
            mf.document_ids.as_deref(),
            Some(["doc-a".to_string(), "doc-b".to_string()].as_slice())
        );
    }

    #[test]
    fn chunk_fetch_filter_never_entity_or_relationship() {
        let mf =
            chunk_fetch_metadata_filter(None, None, None, None).expect("type alone yields Some");
        assert_eq!(mf.vector_type.as_deref(), Some("chunk"));
        assert_ne!(mf.vector_type.as_deref(), Some("entity"));
        assert_ne!(mf.vector_type.as_deref(), Some("relationship"));
    }
}
