//! SPEC-160 — storage contract for the decision extraction mode.
//!
//! Two kinds of data live here:
//!
//! | Data            | Why it exists                                             |
//! |-----------------|-----------------------------------------------------------|
//! | decision answer | Cache of model answers. Same input gives the same answer. |
//! | review row      | A fact the gate kept out of the graph (REVIEW band).      |
//!
//! The trait is the seam (DIP): the extractor knows [`DecisionStore`], never SQL.
//! [`MemoryDecisionStore`] serves tests and the in-process fallback.
//! The Postgres adapter lives in `adapters::postgres::decision_store`.

mod memory;
mod types;

pub use memory::MemoryDecisionStore;
pub use types::{DecisionScope, ReviewKind, ReviewRow};

use async_trait::async_trait;
use serde_json::Value;

use crate::error::StorageError;

/// Persistence for decision answers and review rows.
///
/// Every method takes a [`DecisionScope`]. No method reads across workspaces
/// (EC-160-38, EC-160-39).
#[async_trait]
pub trait DecisionStore: Send + Sync {
    /// Read one cached answer. A hit refreshes its LRU time.
    async fn get_answer(
        &self,
        scope: &DecisionScope,
        key: &str,
    ) -> Result<Option<Value>, StorageError>;

    /// Write one answer. Writing the same key again replaces the answer.
    async fn put_answer(
        &self,
        scope: &DecisionScope,
        key: &str,
        model: &str,
        contract: &str,
        answer: &Value,
    ) -> Result<(), StorageError>;

    /// Replace all review rows of one document. Re-running a document must not
    /// stack rows (EC-160-43).
    async fn replace_review(
        &self,
        scope: &DecisionScope,
        document_id: &str,
        rows: &[ReviewRow],
    ) -> Result<(), StorageError>;

    /// List review rows of one document, best score first.
    async fn list_review(
        &self,
        scope: &DecisionScope,
        document_id: &str,
    ) -> Result<Vec<ReviewRow>, StorageError>;

    /// Delete the review rows of one document. Returns the row count.
    async fn delete_document(
        &self,
        scope: &DecisionScope,
        document_id: &str,
    ) -> Result<u64, StorageError>;

    /// Delete all cache and review rows of a workspace. Returns the row count.
    async fn delete_workspace(&self, scope: &DecisionScope) -> Result<u64, StorageError>;

    /// Delete cache rows older than `ttl_days` and rows beyond `max_rows`
    /// (oldest `last_used_at` first). Returns the row count.
    async fn sweep_cache(
        &self,
        scope: &DecisionScope,
        ttl_days: u32,
        max_rows: u64,
    ) -> Result<u64, StorageError>;

    /// Number of cached answers in a workspace.
    async fn cache_len(&self, scope: &DecisionScope) -> Result<u64, StorageError>;
}
