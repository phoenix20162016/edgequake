//! Delete, file upload, and batch operation DTOs.

use serde::Serialize;
use utoipa::ToSchema;

// ============================================================================
// Delete DTOs
// ============================================================================

/// Document deletion response.
///
/// Async path (default): HTTP returns 202 with `accepted=true` and `track_id`;
/// WebSocket `DeletionCompleted` / `DeletionFailed` is the terminal SSOT.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteDocumentResponse {
    /// Document ID.
    pub document_id: String,

    /// Whether the cascade has finished and the document is gone.
    /// False when the delete was accepted for async processing (`accepted=true`).
    pub deleted: bool,

    /// True when the delete job was accepted (async) — wait for WebSocket terminal.
    #[serde(default)]
    pub accepted: bool,

    /// Deletion operation track id (WebSocket correlation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_id: Option<String>,

    /// Number of chunks deleted (0 when only accepted).
    pub chunks_deleted: usize,

    /// Number of entities affected.
    pub entities_affected: usize,

    /// Number of relationships affected.
    pub relationships_affected: usize,

    /// Number of vector embeddings deleted.
    ///
    /// @implements SPEC-050: Richer delete stats for UI feedback.
    #[serde(default)]
    pub embeddings_deleted: usize,

    /// True when one or more non-fatal phases failed (e.g. graph cascade error).
    ///
    /// @implements SPEC-050: Partial failure visibility.
    #[serde(default)]
    pub partial_failure: bool,

    /// Human-readable description of the partial failure, if any.
    ///
    /// @implements SPEC-050: Partial failure detail for UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_failure_reason: Option<String>,
}

/// Bulk document deletion response.
///
/// WHY: Frontend "Clear All" button needs a bulk delete endpoint.
/// Returns aggregated deletion statistics across all documents.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeleteAllDocumentsResponse {
    /// When true, wipe was accepted and runs asynchronously (HTTP 202).
    /// Final counts arrive via WebSocket `BulkDeletionCompleted` / task poll.
    #[serde(default)]
    pub accepted: bool,

    /// Durable wipe correlation id (`TaskType::WorkspaceWipe` track_id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wipe_track_id: Option<String>,

    /// Planned document count at admit time when `accepted` (not final deleted).
    ///
    /// Final counts arrive via WebSocket `BulkDeletionCompleted` / task poll.
    /// Kept for backward-compatible clients that read `deleted_count` on 202.
    pub deleted_count: usize,

    /// Explicit planned wipe size (same as admit-time `deleted_count` when accepted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_delete_count: Option<usize>,

    /// Total number of chunks deleted across all documents.
    pub total_chunks_deleted: usize,

    /// Total number of entities removed (no other references).
    pub total_entities_removed: usize,

    /// Total number of relationships removed.
    pub total_relationships_removed: usize,

    /// Total number of PDF documents deleted from separate storage.
    pub total_pdfs_deleted: usize,

    /// Number of documents skipped (legacy; ForceCancelAll wipe leaves this 0).
    pub skipped_count: usize,

    /// Document IDs that were skipped due to active processing (legacy).
    pub skipped_documents: Vec<String>,
}

/// Document deletion impact analysis response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeletionImpactResponse {
    /// Document ID.
    pub document_id: String,

    /// Number of chunks that would be deleted.
    pub chunks_to_delete: usize,

    /// Number of entities that would be completely removed (no other sources).
    ///
    /// SPEC-050/EC-1: entities exclusive to this document → DELETED.
    pub entities_to_remove: usize,

    /// Number of entities that would be updated (some sources remaining).
    ///
    /// SPEC-050/EC-2: entities shared with other documents → SURVIVE with pruned sources.
    /// These entities are NOT deleted — they persist in the knowledge graph with
    /// their source_ids updated to exclude this document's chunks.
    pub entities_to_update: usize,

    /// Number of relationships that would be completely removed.
    ///
    /// Includes: (a) relationships exclusive to this document, and
    /// (b) relationships whose source or target entity will be removed (EC-3).
    pub relationships_to_remove: usize,

    /// Number of relationships that would be updated (some sources remaining).
    ///
    /// SPEC-050/EC-6: relationships shared with other documents → SURVIVE with pruned sources.
    pub relationships_to_update: usize,

    /// Preview is read-only; document NOT deleted.
    pub preview_only: bool,
}

// ============================================================================
// File Upload DTOs
// ============================================================================

/// File upload response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FileUploadResponse {
    /// Generated document ID.
    pub document_id: String,

    /// Original filename.
    pub filename: String,

    /// File size in bytes.
    pub size: usize,

    /// Content hash (SHA-256).
    pub content_hash: String,

    /// Processing status.
    pub status: String,

    /// Background task ID when upload is async (SPEC-024 Phase 1.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// Track ID for status polling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_id: Option<String>,

    /// Number of chunks created.
    pub chunk_count: usize,

    /// Number of entities extracted.
    pub entity_count: usize,

    /// Number of relationships extracted.
    pub relationship_count: usize,

    /// Whether this was a duplicate (already processed).
    pub is_duplicate: bool,

    /// Queue projection (SPEC-091 QW2 / LAW-Q4): 1-based FCFS pending position
    /// at admission time. Only set for async uploads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_position: Option<u64>,

    /// Estimated seconds until claim (measured drain; clamped when unknown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_seconds: Option<u64>,

    /// ETA basis: `measured` or `no_history` (honest uncertainty, R-15).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_basis: Option<String>,

    /// SPEC-160: the extraction mode this document will run with (`llm` or `decision`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_mode: Option<String>,

    /// SPEC-160: who chose the mode (`document`, `workspace`, `env`, `default`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_mode_source: Option<String>,
}

// ============================================================================
// Batch Upload DTOs
// ============================================================================

/// Batch file upload response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BatchUploadResponse {
    /// Total files received.
    pub total_files: usize,

    /// Successfully processed files.
    pub processed: usize,

    /// Duplicate files (skipped).
    pub duplicates: usize,

    /// Failed files.
    pub failed: usize,

    /// Results for each file.
    pub results: Vec<BatchFileResult>,
}

/// Result for a single file in batch upload.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BatchFileResult {
    /// Original filename.
    pub filename: String,

    /// Document ID if successful.
    pub document_id: Option<String>,

    /// Status: processed, duplicate, or failed.
    pub status: String,

    /// Error message if failed.
    pub error: Option<String>,
}
