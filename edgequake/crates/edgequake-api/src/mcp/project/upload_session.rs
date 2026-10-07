//! Chunked MCP upload handle. Commit admits asynchronously; it does not wait for the pipeline.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::error::{ApiError, ApiResult};
use crate::file_validation::sanitize_filename;
use crate::handlers::documents::upload::{
    admit_document_for_processing, DocumentAdmissionInput, GleaningAdmissionOptions,
    UploadExtraction,
};
use crate::middleware::TenantContext;
use crate::services::ContentHasher;
use crate::state::AppState;

use super::blob::upload_ttl_secs;
use super::budget::BudgetClass;
use super::envelope::EnvelopeBuilder;
use super::errors::{eq_error, ErrorCode};
use super::ingest::envelope_from_admission;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SessionStatus {
    Open,
    Committed,
    Aborted,
}

struct Session {
    tenant_id: Option<String>,
    workspace_id: Option<String>,
    user_id: Option<String>,
    filename: String,
    media_type: String,
    expected_sha256: Option<String>,
    expected_bytes: Option<u64>,
    received: u64,
    path: PathBuf,
    status: SessionStatus,
    expires_at: Instant,
    document_id: Option<String>,
    task_id: Option<String>,
    commit_status: Option<String>,
}

pub struct McpUploadStore {
    inner: Mutex<HashMap<String, Session>>,
}

impl McpUploadStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Mark a session expired (e2e / unit helpers for TTL denial).
    pub async fn mark_expired(&self, upload_id: &str) -> bool {
        let mut map = self.inner.lock().await;
        if let Some(sess) = map.get_mut(upload_id) {
            sess.expires_at = Instant::now()
                .checked_sub(Duration::from_secs(1))
                .unwrap_or_else(Instant::now);
            true
        } else {
            false
        }
    }
}

impl Default for McpUploadStore {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn eq_upload_begin(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let raw_name = args
        .get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("upload.bin");
    let filename = sanitize_filename(raw_name);
    let media_type = args
        .get("media_type")
        .and_then(|v| v.as_str())
        .unwrap_or("application/octet-stream")
        .to_string();
    let expected_sha256 = args
        .get("sha256")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let expected_bytes = args.get("byte_length").and_then(|v| v.as_u64());
    if let Some(n) = expected_bytes {
        if n as usize > state.config.max_document_size {
            return Ok(eq_error(
                ErrorCode::BudgetExceeded,
                "upload exceeds max document size",
                None,
            ));
        }
    }

    let upload_id = format!("upl_{}", uuid::Uuid::new_v4());
    let dir = std::env::temp_dir().join("edgequake-mcp-uploads");
    tokio::fs::create_dir_all(&dir).await.ok();
    let path = dir.join(&upload_id);
    tokio::fs::write(&path, b"")
        .await
        .map_err(|e| ApiError::Internal(format!("upload temp file: {e}")))?;

    let session = Session {
        tenant_id: tenant_ctx.tenant_id.clone(),
        workspace_id: tenant_ctx.workspace_id.clone(),
        user_id: tenant_ctx.user_id.clone(),
        filename,
        media_type,
        expected_sha256,
        expected_bytes,
        received: 0,
        path,
        status: SessionStatus::Open,
        expires_at: Instant::now() + Duration::from_secs(upload_ttl_secs()),
        document_id: None,
        task_id: None,
        commit_status: None,
    };
    state
        .mcp_uploads
        .inner
        .lock()
        .await
        .insert(upload_id.clone(), session);

    Ok(EnvelopeBuilder::new("upload_begin", BudgetClass::Standard)
        .insert("upload_id", json!(upload_id))
        .insert("filename", json!(raw_name))
        .insert("status", json!("open"))
        .insert("offset", json!(0))
        .build())
}

pub async fn eq_upload_write(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let upload_id = args
        .get("upload_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("upload_id required".into()))?;
    let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
    let chunk_b64 = args
        .get("chunk_base64")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let chunk = match STANDARD.decode(chunk_b64) {
        Ok(b) => b,
        Err(_) => {
            return Ok(eq_error(
                ErrorCode::InvalidId,
                "chunk_base64 must be standard Base64",
                None,
            ));
        }
    };

    let (path, received) = {
        let mut map = state.mcp_uploads.inner.lock().await;
        let Some(sess) = map.get_mut(upload_id) else {
            return Ok(eq_error(
                ErrorCode::NotFound,
                format!("upload_id not found: {upload_id}"),
                None,
            ));
        };
        if Instant::now() > sess.expires_at {
            return Ok(eq_error(ErrorCode::NotFound, "upload_id expired", None));
        }
        if sess.status != SessionStatus::Open {
            return Ok(eq_error(ErrorCode::NotFound, "upload_id is not open", None));
        }
        if !owner_matches(sess, tenant_ctx) {
            return Ok(eq_error(
                ErrorCode::Forbidden,
                "upload_id belongs to another workspace",
                None,
            ));
        }
        if offset != sess.received {
            return Ok(eq_error(
                ErrorCode::InvalidId,
                format!(
                    "offset must be the next contiguous byte ({}); got {offset}",
                    sess.received
                ),
                None,
            ));
        }
        let next = sess.received + chunk.len() as u64;
        if next as usize > state.config.max_document_size {
            return Ok(eq_error(
                ErrorCode::BudgetExceeded,
                "upload exceeds max document size",
                None,
            ));
        }
        if let Some(max) = sess.expected_bytes {
            if next > max {
                return Ok(eq_error(
                    ErrorCode::BudgetExceeded,
                    "chunk exceeds declared byte_length",
                    None,
                ));
            }
        }
        sess.received = next;
        (sess.path.clone(), sess.received)
    };
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .await
        .map_err(|e| ApiError::Internal(format!("upload write: {e}")))?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&chunk)
        .await
        .map_err(|e| ApiError::Internal(format!("upload write: {e}")))?;
    Ok(EnvelopeBuilder::new("upload_write", BudgetClass::Standard)
        .insert("upload_id", json!(upload_id))
        .insert("offset", json!(received))
        .insert("received_bytes", json!(received))
        .build())
}

pub async fn eq_upload_commit(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let upload_id = args
        .get("upload_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("upload_id required".into()))?;
    commit_upload_id(state, tenant_ctx, upload_id).await
}

pub async fn eq_upload_abort(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let upload_id = args
        .get("upload_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("upload_id required".into()))?;
    let mut map = state.mcp_uploads.inner.lock().await;
    let Some(sess) = map.get_mut(upload_id) else {
        return Ok(eq_error(
            ErrorCode::NotFound,
            format!("upload_id not found: {upload_id}"),
            None,
        ));
    };
    if !owner_matches(sess, tenant_ctx) {
        return Ok(eq_error(
            ErrorCode::Forbidden,
            "upload_id belongs to another workspace",
            None,
        ));
    }
    sess.status = SessionStatus::Aborted;
    let path = sess.path.clone();
    drop(map);
    let _ = tokio::fs::remove_file(path).await;
    Ok(EnvelopeBuilder::new("upload_abort", BudgetClass::Standard)
        .insert("upload_id", json!(upload_id))
        .insert("status", json!("aborted"))
        .build())
}

pub async fn commit_upload_id(
    state: &AppState,
    tenant_ctx: &TenantContext,
    upload_id: &str,
) -> ApiResult<Value> {
    let (path, filename, media_type, expected_sha, expected_bytes, already) = {
        let mut map = state.mcp_uploads.inner.lock().await;
        let Some(sess) = map.get_mut(upload_id) else {
            return Ok(eq_error(
                ErrorCode::NotFound,
                format!("upload_id not found: {upload_id}"),
                None,
            ));
        };
        if Instant::now() > sess.expires_at && sess.status == SessionStatus::Open {
            return Ok(eq_error(ErrorCode::NotFound, "upload_id expired", None));
        }
        if !owner_matches(sess, tenant_ctx) {
            return Ok(eq_error(
                ErrorCode::Forbidden,
                "upload_id belongs to another workspace",
                None,
            ));
        }
        if sess.status == SessionStatus::Aborted {
            return Ok(eq_error(ErrorCode::NotFound, "upload_id aborted", None));
        }
        if sess.status == SessionStatus::Committed {
            return Ok(idempotent_commit_envelope(sess));
        }
        (
            sess.path.clone(),
            sess.filename.clone(),
            sess.media_type.clone(),
            sess.expected_sha256.clone(),
            sess.expected_bytes,
            false,
        )
    };
    let _ = already;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| ApiError::NotFound("upload bytes missing".into()))?;
    if let Some(exp) = expected_bytes {
        if bytes.len() as u64 != exp {
            return Ok(eq_error(
                ErrorCode::InvalidId,
                "received bytes do not match declared byte_length",
                None,
            ));
        }
    }
    if let Some(exp) = expected_sha {
        let got = hex::encode(Sha256::digest(&bytes));
        if !got.eq_ignore_ascii_case(&exp) {
            return Ok(eq_error(ErrorCode::InvalidId, "sha256 mismatch", None));
        }
    }

    let envelope = admit_bytes(state, tenant_ctx, &filename, &media_type, bytes).await?;
    if envelope.get("ok").and_then(|v| v.as_bool()) != Some(false) {
        let mut map = state.mcp_uploads.inner.lock().await;
        if let Some(sess) = map.get_mut(upload_id) {
            sess.status = SessionStatus::Committed;
            sess.document_id = envelope
                .get("document_id")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            sess.task_id = envelope
                .get("task_id")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            sess.commit_status = envelope
                .get("status")
                .and_then(|v| v.as_str())
                .map(str::to_string);
        }
    }
    Ok(envelope)
}

pub async fn admit_bytes(
    state: &AppState,
    tenant_ctx: &TenantContext,
    filename: &str,
    media_type: &str,
    bytes: Vec<u8>,
) -> ApiResult<Value> {
    let is_pdf = filename.to_ascii_lowercase().ends_with(".pdf")
        || media_type.eq_ignore_ascii_case("application/pdf");
    if is_pdf {
        let resp = crate::handlers::pdf_upload::admit_pdf_bytes(
            state,
            tenant_ctx,
            filename.to_string(),
            bytes,
        )
        .await?;
        let status = if resp.status == "duplicate_processing" {
            "duplicate_processing"
        } else {
            "pending"
        };
        return Ok(EnvelopeBuilder::new("ingest", BudgetClass::Standard)
            .insert(
                "document_id",
                json!(resp.document_id.unwrap_or_else(|| resp.pdf_id.clone())),
            )
            .insert("task_id", json!(resp.task_id))
            .insert("status", json!(status))
            .insert("ready", json!(false))
            .insert(
                "poll",
                json!({
                    "tool": "eq_task_get",
                    "until": ["indexed", "failed", "cancelled"],
                }),
            )
            .build());
    }

    let resolved = crate::services::resolve_upload_content(
        state,
        tenant_ctx.workspace_id_uuid(),
        filename,
        &bytes,
    )
    .await?;
    let content_hash = ContentHasher::hash_bytes(&bytes);
    let outcome = admit_document_for_processing(
        state,
        tenant_ctx,
        DocumentAdmissionInput {
            text_content: resolved.text_content,
            title: filename.to_string(),
            source_type: resolved.meta.source_type,
            mime_type: Some(resolved.mime_type.clone()),
            raw_byte_size: bytes.len(),
            content_hash,
            custom_metadata: UploadExtraction {
                mode: Some("llm".into()),
                gate_preset: None,
            }
            .merge_into(None),
            track_id: None,
            expected_batch_count: None,
            gleaning: GleaningAdmissionOptions::default(),
            document_type: None,
            chunk_strategy: None,
            chunk_options: None,
            extract_max_entities: None,
            extract_max_records: None,
            multimodal: resolved.meta.multimodal,
            ingest_mode: resolved.meta.ingest_mode,
            multimodal_manifest: resolved.manifest,
        },
        "upload",
    )
    .await?;
    let env = envelope_from_admission(outcome);
    #[cfg(feature = "postgres")]
    if let Some(doc_id) = env.get("document_id").and_then(|v| v.as_str()) {
        let _ = crate::services::persist_uploaded_original(
            state,
            tenant_ctx,
            doc_id,
            filename,
            &resolved.mime_type,
            resolved.meta.source_type,
            &bytes,
        )
        .await;
    }
    Ok(env)
}

fn owner_matches(sess: &Session, tenant_ctx: &TenantContext) -> bool {
    sess.workspace_id == tenant_ctx.workspace_id && sess.tenant_id == tenant_ctx.tenant_id
}

fn idempotent_commit_envelope(sess: &Session) -> Value {
    EnvelopeBuilder::new("ingest", BudgetClass::Standard)
        .insert("document_id", json!(sess.document_id))
        .insert("task_id", json!(sess.task_id))
        .insert(
            "status",
            json!(sess.commit_status.as_deref().unwrap_or("pending")),
        )
        .insert("ready", json!(false))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::{default_tenant_uuid, default_workspace_uuid, TenantContext};

    fn ctx(ws: &str) -> TenantContext {
        TenantContext {
            tenant_id: Some(default_tenant_uuid().to_string()),
            workspace_id: Some(ws.to_string()),
            user_id: None,
        }
    }

    #[tokio::test]
    async fn expired_upload_id_is_not_found() {
        let state = AppState::test_state();
        let tenant = ctx(&default_workspace_uuid().to_string());
        let begin = eq_upload_begin(
            &state,
            &tenant,
            &json!({"filename": "a.txt", "byte_length": 3}),
        )
        .await
        .unwrap();
        let uid = begin["upload_id"].as_str().unwrap().to_string();
        assert!(state.mcp_uploads.mark_expired(&uid).await);
        let write = eq_upload_write(
            &state,
            &tenant,
            &json!({
                "upload_id": uid,
                "offset": 0,
                "chunk_base64": STANDARD.encode(b"abc"),
            }),
        )
        .await
        .unwrap();
        assert_eq!(write["ok"], false);
        assert_eq!(write["error"]["code"], "eq/not_found");
    }

    #[tokio::test]
    async fn cross_workspace_upload_denied() {
        let state = AppState::test_state();
        let owner = ctx(&default_workspace_uuid().to_string());
        let other = ctx("00000000-0000-0000-0000-000000000099");
        let begin = eq_upload_begin(&state, &owner, &json!({"filename": "a.txt"}))
            .await
            .unwrap();
        let uid = begin["upload_id"].as_str().unwrap();
        let write = eq_upload_write(
            &state,
            &other,
            &json!({
                "upload_id": uid,
                "offset": 0,
                "chunk_base64": STANDARD.encode(b"abc"),
            }),
        )
        .await
        .unwrap();
        assert_eq!(write["error"]["code"], "eq/forbidden");
    }

    #[tokio::test]
    async fn write_over_max_document_size_budget() {
        let mut state = AppState::test_state();
        state.config.max_document_size = 4;
        let tenant = ctx(&default_workspace_uuid().to_string());
        let begin = eq_upload_begin(&state, &tenant, &json!({"filename": "a.txt"}))
            .await
            .unwrap();
        let uid = begin["upload_id"].as_str().unwrap();
        let write = eq_upload_write(
            &state,
            &tenant,
            &json!({
                "upload_id": uid,
                "offset": 0,
                "chunk_base64": STANDARD.encode(b"0123456789"),
            }),
        )
        .await
        .unwrap();
        assert_eq!(write["error"]["code"], "eq/budget_exceeded");
    }
}
