//! Mix query mode — intent-gated max-after-minmax / RRF blend of local, global, and naive arms.
//!
//! SPEC-046 OPS-P1: skip zero-weight / intent-gated arms (DRY via [`resolve_arm_plan`]).

use std::collections::HashMap;
use std::sync::Arc;

use crate::context::{QueryContext, RetrievedChunk};
use crate::error::Result;
use crate::keywords::ExtractedKeywords;
use crate::mix_weights::{
    mix_arm_gate_enabled, resolve_arm_plan, ArmPlan, META_ARMS_GATED, META_ARMS_RUN,
    META_ARM_GLOBAL_CHUNKS, META_ARM_GLOBAL_MS, META_ARM_LOCAL_CHUNKS, META_ARM_LOCAL_MS,
    META_ARM_NAIVE_CHUNKS, META_ARM_NAIVE_MS,
};

use edgequake_storage::traits::VectorStorage;

use super::super::{QueryEmbeddings, QueryEngine};
use super::arm_timed::run_arm_timed;
use super::chunk_retrieval::append_score_ranked_chunks;

impl QueryEngine {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine_impl) async fn query_mix_with_vector_storage(
        &self,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        allowed_document_ids: Option<&[String]>,
        vector_storage: &Arc<dyn VectorStorage>,
        mix_weights: Option<&crate::mix_weights::MixWeightOverride>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        let plan = resolve_arm_plan(
            &self.config,
            mix_weights,
            keywords.query_intent,
            mix_arm_gate_enabled(),
        );

        // 078 R3: LightRAG truncate-E/R-then-VECTOR — skip per-arm KG chunks during arms.
        let post_truncate = crate::kg_chunk_pick::KgChunkPickTiming::from_env().is_post_truncate();

        // Box each arm so join! holds three pointers, not three full retrieval FSMs
        // (same stack-overflow class as Hybrid — SPEC-047).
        let (local_res, global_res, naive_res) = {
            let _skip_arm_chunks =
                post_truncate.then(crate::kg_chunk_pick::SkipArmKgChunksGuard::enter);
            let local_fut = Box::pin(run_arm_timed(
                plan.run_local,
                "local",
                "mix",
                query_text,
                max_chunks,
                || {
                    self.query_local_with_vector_storage(
                        query_text,
                        keywords,
                        embeddings,
                        tenant_id.clone(),
                        workspace_id.clone(),
                        allowed_document_ids,
                        vector_storage,
                        max_chunks,
                    )
                },
            ));
            let global_fut = Box::pin(run_arm_timed(
                plan.run_global,
                "global",
                "mix",
                query_text,
                max_chunks,
                || {
                    self.query_global_with_vector_storage(
                        query_text,
                        keywords,
                        embeddings,
                        tenant_id.clone(),
                        workspace_id.clone(),
                        allowed_document_ids,
                        vector_storage,
                        max_chunks,
                    )
                },
            ));
            let naive_fut = Box::pin(run_arm_timed(
                plan.run_naive,
                "naive",
                "mix",
                query_text,
                max_chunks,
                || {
                    self.query_naive_with_vector_storage(
                        query_text,
                        embeddings,
                        tenant_id.clone(),
                        workspace_id.clone(),
                        allowed_document_ids,
                        vector_storage,
                        max_chunks,
                    )
                },
            ));
            tokio::join!(local_fut, global_fut, naive_fut)
        };

        let (local_context, local_ms) = local_res?;
        let (global_context, global_ms) = global_res?;
        let (naive_context, naive_ms) = naive_res?;

        let local_chunks = local_context.chunks.len();
        let global_chunks = global_context.chunks.len();
        let naive_chunks = naive_context.chunks.len();

        let mut merged = fuse_mix_contexts(
            &local_context,
            &global_context,
            &naive_context,
            plan,
            max_chunks,
        );

        if post_truncate {
            merged = self
                .mix_post_truncate_kg_chunks(
                    merged,
                    query_text,
                    keywords,
                    embeddings,
                    tenant_id.clone(),
                    workspace_id.clone(),
                    allowed_document_ids,
                    vector_storage,
                    max_chunks,
                )
                .await?;
        }

        attach_arm_metadata(
            &mut merged,
            plan,
            local_ms,
            global_ms,
            naive_ms,
            local_chunks,
            global_chunks,
            naive_chunks,
        );

        tracing::debug!(
            merged_chunks = merged.chunks.len(),
            merged_entities = merged.entities.len(),
            merged_relationships = merged.relationships.len(),
            run_local = plan.run_local,
            run_global = plan.run_global,
            run_naive = plan.run_naive,
            w_local = plan.w_local,
            w_global = plan.w_global,
            w_naive = plan.w_naive,
            post_truncate,
            local_ms,
            global_ms,
            naive_ms,
            "Mix merge complete (intent-gated)"
        );

        Ok(merged)
    }

    /// 078 R3: truncate fused E/R by token budget → one VECTOR(+LR budget) pick →
    /// RR-merge with naive chunks (LightRAG `_build_query_context` stage order).
    #[allow(clippy::too_many_arguments)]
    async fn mix_post_truncate_kg_chunks(
        &self,
        mut merged: QueryContext,
        query_text: &str,
        keywords: &ExtractedKeywords,
        embeddings: &QueryEmbeddings,
        tenant_id: Option<String>,
        workspace_id: Option<String>,
        allowed_document_ids: Option<&[String]>,
        vector_storage: &Arc<dyn VectorStorage>,
        max_chunks: usize,
    ) -> Result<QueryContext> {
        let trunc_cfg = crate::truncation::truncation_config_for_intent(
            &self.config.truncation,
            keywords.query_intent,
        );
        let pre_e = merged.entities.len();
        let pre_r = merged.relationships.len();
        merged.entities = crate::truncation::truncate_entities(
            std::mem::take(&mut merged.entities),
            trunc_cfg.max_entity_tokens,
            self.tokenizer.as_ref(),
        );
        merged.relationships = crate::truncation::truncate_relationships(
            std::mem::take(&mut merged.relationships),
            trunc_cfg.max_relation_tokens,
            self.tokenizer.as_ref(),
        );

        let naive_chunks = std::mem::take(&mut merged.chunks);
        let retrieval_config = self.config_with_max_chunks(max_chunks);
        // Prefer low-level embed for VECTOR pick (local/entity path); fall back high.
        let query_embedding = if !embeddings.low_level.is_empty() {
            embeddings.low_level.as_slice()
        } else {
            embeddings.high_level.as_slice()
        };
        let (kg_chunks, sparse_outcome) = append_score_ranked_chunks(
            self,
            &merged,
            query_text,
            query_embedding,
            tenant_id,
            workspace_id,
            vector_storage,
            &retrieval_config,
            allowed_document_ids,
            Some(embeddings.model.as_str()).filter(|s| !s.is_empty()),
            "mix_post_truncate",
        )
        .await?;
        crate::retrieval_telemetry::mark_sparse_outcome(
            &mut merged,
            sparse_outcome.as_str(),
            sparse_outcome.is_fts_fallback(),
        );

        // Empty global list: one KG pool (entities∪relations) + naive, via RR arm order.
        let empty: Vec<RetrievedChunk> = Vec::new();
        for chunk in crate::hybrid_merge::round_robin_merge_chunks(
            &kg_chunks,
            &empty,
            &naive_chunks,
            max_chunks,
        ) {
            merged.add_chunk(chunk);
        }

        tracing::info!(
            pre_entities = pre_e,
            post_entities = merged.entities.len(),
            pre_relationships = pre_r,
            post_relationships = merged.relationships.len(),
            kg_chunks = kg_chunks.len(),
            naive_chunks = naive_chunks.len(),
            merged_chunks = merged.chunks.len(),
            max_entity_tokens = trunc_cfg.max_entity_tokens,
            max_relation_tokens = trunc_cfg.max_relation_tokens,
            "078 Mix post_truncate: truncate E/R → VECTOR pick → RR with naive"
        );

        Ok(merged)
    }
}

/// Shared Mix fusion (max-after-minmax or RRF) — pure relative to arm execution.
fn fuse_mix_contexts(
    local_context: &QueryContext,
    global_context: &QueryContext,
    naive_context: &QueryContext,
    plan: ArmPlan,
    max_chunks: usize,
) -> QueryContext {
    let _stage = edgequake_observability::enter_pipeline_stage("query.fuse");
    let fusion = crate::fusion::mix_fusion_mode_label(crate::fusion::mix_fusion_mode_from_env());
    edgequake_observability::record_observation_meta("fusion", fusion);
    // Match Hybrid: drop orphan KG payloads when an arm returned no chunks.
    let local_context = crate::hybrid_merge::prune_empty_arm_graph(local_context.clone());
    let global_context = crate::hybrid_merge::prune_empty_arm_graph(global_context.clone());

    let w_local = plan.w_local;
    let w_global = plan.w_global;
    let w_naive = plan.w_naive;

    let mut merged = QueryContext::new();

    match crate::fusion::mix_fusion_mode_from_env() {
        crate::fusion::MixFusionMode::Rrf => {
            let mut chunk_lookup: HashMap<String, RetrievedChunk> = HashMap::new();
            for ctx in [&local_context, &global_context, naive_context] {
                for chunk in &ctx.chunks {
                    chunk_lookup
                        .entry(chunk.id.clone())
                        .or_insert_with(|| chunk.clone());
                }
            }

            let ranked_lists = [
                local_context.chunks.iter().map(|c| c.id.clone()).collect(),
                global_context.chunks.iter().map(|c| c.id.clone()).collect(),
                naive_context.chunks.iter().map(|c| c.id.clone()).collect(),
            ];
            let weights = [w_local, w_global, w_naive];
            let fused = crate::fusion::reciprocal_rank_fusion(
                &ranked_lists,
                &weights,
                crate::fusion::RRF_K,
            );
            for chunk in crate::fusion::chunks_from_rrf_ranking(&fused, &chunk_lookup, max_chunks) {
                merged.add_chunk(chunk);
            }
        }
        crate::fusion::MixFusionMode::RoundRobin => {
            for chunk in crate::hybrid_merge::round_robin_merge_chunks(
                &local_context.chunks,
                &global_context.chunks,
                &naive_context.chunks,
                max_chunks,
            ) {
                merged.add_chunk(chunk);
            }
        }
        // D-35: max of weighted min-max contributions (not a weighted sum).
        crate::fusion::MixFusionMode::MaxAfterMinMax => {
            let mut blended: HashMap<String, (RetrievedChunk, f32)> = HashMap::new();
            for (ctx, weight) in [
                (&local_context, w_local),
                (&global_context, w_global),
                (naive_context, w_naive),
            ] {
                if weight <= 0.0 {
                    continue;
                }
                // D-37: convert raw arm scores → MinMax only at fusion boundary.
                let raw: Vec<f32> = ctx.chunks.iter().map(|c| c.score).collect();
                let norm = crate::score_scale::min_max_normalize_to_fusion_scale(&raw);
                for (chunk, scaled) in ctx.chunks.iter().zip(norm.iter()) {
                    let contribution =
                        crate::score_scale::weighted_minmax_contribution(*scaled, weight)
                            .map(|s| s.value())
                            .unwrap_or(0.0);
                    blended
                        .entry(chunk.id.clone())
                        .and_modify(|(_, score)| {
                            if contribution > *score {
                                *score = contribution;
                            }
                        })
                        .or_insert_with(|| (chunk.clone(), contribution));
                }
            }

            let mut chunks: Vec<(RetrievedChunk, f32)> = blended.into_values().collect();
            chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for (mut chunk, score) in chunks.into_iter().take(max_chunks) {
                chunk.score = score.max(0.0);
                merged.add_chunk(chunk);
            }
        }
    }

    let mut seen_entities = std::collections::HashSet::new();
    for e in local_context
        .entities
        .iter()
        .chain(global_context.entities.iter())
    {
        if seen_entities.insert(e.name.clone()) {
            merged.add_entity(e.clone());
        }
    }
    let mut seen_rels = std::collections::HashSet::new();
    for rel in local_context
        .relationships
        .iter()
        .chain(global_context.relationships.iter())
    {
        let key = format!("{}-{}-{}", rel.source, rel.relation_type, rel.target);
        if seen_rels.insert(key) {
            merged.add_relationship(rel.clone());
        }
    }

    // 039: propagate topic_admit_* from arms (fuse previously dropped metadata).
    crate::topic_entity_admit::merge_topic_admit_metadata(
        &mut merged,
        &[&local_context, &global_context, naive_context],
    );
    let topic_ids = crate::topic_entity_admit::topic_chunk_ids_from_context(&merged);
    if crate::topic_entity_admit::topic_survival_enabled() && !topic_ids.is_empty() {
        let mut lookup: HashMap<String, RetrievedChunk> = HashMap::new();
        for ctx in [&local_context, &global_context, naive_context] {
            for chunk in &ctx.chunks {
                lookup
                    .entry(chunk.id.clone())
                    .or_insert_with(|| chunk.clone());
            }
        }
        crate::topic_entity_admit::fuse_protect_topic_chunks(
            &mut merged.chunks,
            &lookup,
            &topic_ids,
            max_chunks,
        );
    }

    // Phase 1 (SPEC-001): sync RRF-score prune only. Cosine mode runs in postprocess
    // (needs embedding provider) so Mix must not pre-truncate the candidate pool.
    let prune_cfg = crate::relevancy_prune::RelevancyPruneConfig::from_env();
    if prune_cfg.applies_in_mix_fuse() {
        crate::relevancy_prune::apply_relevancy_prune(merged, &prune_cfg)
    } else {
        merged
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn attach_arm_metadata(
    ctx: &mut QueryContext,
    plan: ArmPlan,
    local_ms: u64,
    global_ms: u64,
    naive_ms: u64,
    local_chunks: usize,
    global_chunks: usize,
    naive_chunks: usize,
) {
    if plan.run_local {
        ctx.metadata
            .insert(META_ARM_LOCAL_MS.into(), serde_json::json!(local_ms));
        ctx.metadata.insert(
            META_ARM_LOCAL_CHUNKS.into(),
            serde_json::json!(local_chunks as u64),
        );
    }
    if plan.run_global {
        ctx.metadata
            .insert(META_ARM_GLOBAL_MS.into(), serde_json::json!(global_ms));
        ctx.metadata.insert(
            META_ARM_GLOBAL_CHUNKS.into(),
            serde_json::json!(global_chunks as u64),
        );
    }
    if plan.run_naive {
        ctx.metadata
            .insert(META_ARM_NAIVE_MS.into(), serde_json::json!(naive_ms));
        ctx.metadata.insert(
            META_ARM_NAIVE_CHUNKS.into(),
            serde_json::json!(naive_chunks as u64),
        );
    }
    let mut run = Vec::new();
    if plan.run_local {
        run.push("local");
    }
    if plan.run_global {
        run.push("global");
    }
    if plan.run_naive {
        run.push("naive");
    }
    ctx.metadata
        .insert(META_ARMS_RUN.into(), serde_json::json!(run.join(",")));
    let gated = !(plan.run_local && plan.run_global && plan.run_naive);
    ctx.metadata
        .insert(META_ARMS_GATED.into(), serde_json::json!(gated));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_impl::QueryEngineConfig;
    use crate::keywords::QueryIntent;
    use crate::mix_weights::{resolve_arm_plan, MixWeightOverride};

    #[test]
    fn attach_metadata_marks_all_arms_for_mix_factual() {
        // Mix Factual always runs local+global+naive (MIX_ARM_GATE default false /
        // LightRAG parity). Metadata must list all three arms as run, not gated.
        let plan = resolve_arm_plan(
            &QueryEngineConfig::default(),
            None,
            QueryIntent::Factual,
            true,
        );
        let mut ctx = QueryContext::new();
        attach_arm_metadata(&mut ctx, plan, 1, 2, 3, 5, 6, 4);
        assert_eq!(
            ctx.metadata.get(META_ARMS_RUN).unwrap(),
            "local,global,naive"
        );
        assert_eq!(ctx.metadata.get(META_ARMS_GATED).unwrap(), false);
        assert!(ctx.metadata.contains_key(META_ARM_LOCAL_MS));
        assert!(ctx.metadata.contains_key(META_ARM_GLOBAL_MS));
        assert!(ctx.metadata.contains_key(META_ARM_NAIVE_MS));
        assert_eq!(ctx.metadata.get(META_ARM_NAIVE_CHUNKS).unwrap(), 4);
    }

    #[test]
    fn fuse_skips_zero_weight_arms() {
        let mut naive = QueryContext::new();
        naive.add_chunk(RetrievedChunk::new("c1", "x", 1.0));
        let plan = resolve_arm_plan(
            &QueryEngineConfig::default(),
            Some(&MixWeightOverride {
                local: Some(0.0),
                global: Some(0.0),
                naive: Some(1.0),
            }),
            QueryIntent::Comparative,
            false,
        );
        let merged =
            fuse_mix_contexts(&QueryContext::new(), &QueryContext::new(), &naive, plan, 10);
        assert_eq!(merged.chunks.len(), 1);
        assert_eq!(merged.chunks[0].id, "c1");
    }
}
