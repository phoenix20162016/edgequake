//! SPEC-160: keep what a decision run produced beside the document.
//!
//! After extraction, two things must outlive the worker:
//!
//! - the REVIEW rows (facts the gate kept out of the graph) → `decision_review` table;
//! - the per-document counts → `decision_stats` in the document metadata.
//!
//! An LLM run over a document that once ran in decision mode clears both, so the
//! screen never shows stale review rows (EC-160-43).

use edgequake_pipeline::extractor::decision::{
    outcome_from_extractions, DecisionOutcome, META_DOCUMENT_DECISION_STATS,
};
use edgequake_pipeline::{
    extraction_mode_from_metadata, extraction_mode_from_value, resolve_extraction_mode_from_env,
};
use edgequake_storage::decision::DecisionScope;
use serde_json::{json, Map, Value};

use super::super::*;
use super::types::TextInsertPersisted;

/// Warning added to the stats when the review rows could not be written.
pub(super) const REVIEW_NOT_SAVED: &str = "review_not_saved";

/// Write `decision_stats` into a document metadata map. `None` removes it.
fn apply_stats(meta: &mut Map<String, Value>, outcome: Option<&DecisionOutcome>, saved: bool) {
    let Some(outcome) = outcome else {
        meta.remove(META_DOCUMENT_DECISION_STATS);
        return;
    };
    let mut stats = outcome.stats.clone();
    if !saved {
        stats.warnings.push(REVIEW_NOT_SAVED.to_string());
    }
    meta.insert(META_DOCUMENT_DECISION_STATS.into(), json!(stats));
}

impl DocumentTaskProcessor {
    /// Store review rows and the document stats. Never fails the ingestion: the
    /// graph is already written, so a store error is shown as a warning instead.
    pub(super) async fn record_decision_outcome(&self, persisted: &TextInsertPersisted) {
        let document_id = &persisted.prepared.document_id;
        let mut outcome = outcome_from_extractions(&persisted.result.extractions);
        if let Some(out) = outcome.as_mut() {
            out.stats.source = self.mode_source(persisted).await;
        }
        if outcome.is_none() && !self.had_decision_stats(document_id).await {
            return;
        }
        let saved = self
            .save_review(persisted, outcome.as_ref().map(|o| o.review.as_slice()))
            .await;
        if outcome.is_some() {
            self.sweep_cache(persisted).await;
        }
        let patch = crate::services::patch_document_metadata(&self.kv_storage, document_id, |m| {
            apply_stats(m, outcome.as_ref(), saved)
        })
        .await;
        if let Err(error) = patch {
            warn!(document_id = %document_id, %error, "SPEC-160: decision_stats not saved");
        }
    }

    /// Keep the answer cache inside its TTL and row limit. One sweep per decision run
    /// is enough: the cache only grows when a run writes to it.
    async fn sweep_cache(&self, persisted: &TextInsertPersisted) {
        let Some(state) = self.app_state.as_ref() else {
            return;
        };
        let Ok(settings) = state.decision.server_settings() else {
            return;
        };
        let scope = DecisionScope::new(
            persisted.prepared.tenant_id.as_deref(),
            &persisted.workspace_id_meta,
        );
        let swept = state
            .decision
            .store()
            .sweep_cache(&scope, settings.cache_ttl_days, settings.cache_max_rows)
            .await;
        match swept {
            Ok(0) => {}
            Ok(rows) => {
                info!(workspace_id = %persisted.workspace_id_meta, rows, "SPEC-160: cache swept")
            }
            Err(error) => warn!(%error, "SPEC-160: cache sweep failed"),
        }
    }

    /// Which layer chose the mode, resolved the same way the pipeline factory did.
    async fn mode_source(&self, persisted: &TextInsertPersisted) -> Option<String> {
        let data = persisted.prepared.data.metadata.as_ref()?;
        let document = extraction_mode_from_value(data);
        let workspace = match (
            &self.app_state,
            uuid::Uuid::parse_str(&persisted.workspace_id_meta),
        ) {
            (Some(state), Ok(id)) => state
                .workspace_service
                .get_workspace(id)
                .await
                .ok()
                .flatten(),
            _ => None,
        };
        let workspace_word = workspace.and_then(|w| extraction_mode_from_metadata(&w.metadata));
        resolve_extraction_mode_from_env(document.as_deref(), workspace_word.as_deref())
            .ok()
            .map(|r| r.source.as_str().to_string())
    }

    async fn had_decision_stats(&self, document_id: &str) -> bool {
        matches!(
            crate::services::load_staging_first_metadata(self.kv_storage.as_ref(), document_id)
                .await,
            Ok(Some((_, meta))) if meta.get(META_DOCUMENT_DECISION_STATS).is_some()
        )
    }

    /// Replace (or clear, for `None`) the document's review rows.
    async fn save_review(
        &self,
        persisted: &TextInsertPersisted,
        rows: Option<&[edgequake_storage::decision::ReviewRow]>,
    ) -> bool {
        let Some(state) = self.app_state.as_ref() else {
            return rows.is_none_or(<[_]>::is_empty);
        };
        let store = state.decision.store();
        let scope = DecisionScope::new(
            persisted.prepared.tenant_id.as_deref(),
            &persisted.workspace_id_meta,
        );
        let document_id = &persisted.prepared.document_id;
        let result = match rows {
            Some(rows) => store.replace_review(&scope, document_id, rows).await,
            None => store.delete_document(&scope, document_id).await.map(|_| ()),
        };
        if let Err(error) = &result {
            warn!(document_id = %document_id, %error, "SPEC-160: review rows not saved");
        }
        result.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_pipeline::extractor::decision::DecisionStats;

    fn outcome() -> DecisionOutcome {
        DecisionOutcome {
            stats: DecisionStats {
                chunks: 2,
                entities: 5,
                ..Default::default()
            },
            review: vec![],
        }
    }

    // T-160-U86 — stats land under the stable key; a save failure is visible.
    #[test]
    fn stats_are_written_with_warning_on_failure() {
        let mut meta = Map::new();
        apply_stats(&mut meta, Some(&outcome()), true);
        assert_eq!(meta[META_DOCUMENT_DECISION_STATS]["entities"], 5);
        assert_eq!(
            meta[META_DOCUMENT_DECISION_STATS]["warnings"],
            json!([]),
            "no warning when saved"
        );
        apply_stats(&mut meta, Some(&outcome()), false);
        assert_eq!(
            meta[META_DOCUMENT_DECISION_STATS]["warnings"],
            json!([REVIEW_NOT_SAVED])
        );
    }

    // T-160-U87 — an LLM re-run removes the stale stats.
    #[test]
    fn llm_rerun_clears_stats() {
        let mut meta = Map::new();
        apply_stats(&mut meta, Some(&outcome()), true);
        apply_stats(&mut meta, None, true);
        assert!(!meta.contains_key(META_DOCUMENT_DECISION_STATS));
    }
}
