//! SPEC-160: remove decision data when its owner goes away.
//!
//! Review rows hold sentence text, so they follow the document (EC-160-36). The answer
//! cache and review rows follow the workspace (EC-160-37). Cleanup never blocks a
//! delete: a store error is logged, because the next sweep or re-run replaces the rows.

use edgequake_storage::decision::DecisionScope;
use tracing::{debug, warn};

use crate::state::AppState;

/// Delete the review rows of one document.
pub async fn purge_decision_document(state: &AppState, workspace_id: &str, document_id: &str) {
    let scope = DecisionScope::new(None, workspace_id);
    match state
        .decision
        .store()
        .delete_document(&scope, document_id)
        .await
    {
        Ok(0) => {}
        Ok(rows) => debug!(%workspace_id, %document_id, rows, "SPEC-160: review rows removed"),
        Err(error) => warn!(%workspace_id, %document_id, %error, "SPEC-160: review cleanup failed"),
    }
}

/// Delete the answer cache and all review rows of one workspace.
pub async fn purge_decision_workspace(state: &AppState, workspace_id: &str) {
    let scope = DecisionScope::new(None, workspace_id);
    match state.decision.store().delete_workspace(&scope).await {
        Ok(rows) => debug!(%workspace_id, rows, "SPEC-160: decision data removed"),
        Err(error) => warn!(%workspace_id, %error, "SPEC-160: workspace cleanup failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::decision::{ReviewKind, ReviewRow};

    fn row() -> ReviewRow {
        ReviewRow {
            chunk_id: "c".into(),
            kind: ReviewKind::Entity,
            subject: "ADA".into(),
            label: "PERSON".into(),
            object: None,
            score: 0.4,
            reason: None,
            sentence: "Ada wrote.".into(),
            model: "m".into(),
            contract: "k".into(),
        }
    }

    // T-160-U92 — document delete removes its rows only; workspace delete removes all.
    #[tokio::test]
    async fn cleanup_follows_the_owner() {
        let state = AppState::test_state();
        let store = state.decision.store();
        let scope = DecisionScope::new(None, "ws-1");
        let other = DecisionScope::new(None, "ws-2");
        for (s, d) in [(&scope, "d1"), (&scope, "d2"), (&other, "d1")] {
            store.replace_review(s, d, &[row()]).await.unwrap();
        }
        purge_decision_document(&state, "ws-1", "d1").await;
        assert!(store.list_review(&scope, "d1").await.unwrap().is_empty());
        assert_eq!(store.list_review(&scope, "d2").await.unwrap().len(), 1);
        assert_eq!(store.list_review(&other, "d1").await.unwrap().len(), 1);
        purge_decision_workspace(&state, "ws-1").await;
        assert!(store.list_review(&scope, "d2").await.unwrap().is_empty());
        assert_eq!(store.list_review(&other, "d1").await.unwrap().len(), 1);
    }
}
