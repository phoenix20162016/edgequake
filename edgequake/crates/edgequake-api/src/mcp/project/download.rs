//! Document download as MCP blob chunks. Does not wait for ingest to finish.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::error::{ApiError, ApiResult};
use crate::handlers::documents::load_original_bytes;
use crate::middleware::TenantContext;
use crate::services::document_body_loader::load_document_body;
use crate::state::AppState;

use super::blob::{blob_max_bytes, EXTRA_CONTENT_KEY};
use super::budget::BudgetClass;
use super::envelope::EnvelopeBuilder;
use super::errors::{eq_error, ErrorCode};

pub async fn eq_document_download(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let document_id = args
        .get("document_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("document_id required".into()))?;
    let representation = args
        .get("representation")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let max_bytes = args
        .get("max_bytes")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or_else(blob_max_bytes)
        .min(blob_max_bytes());

    let (bytes, media_type) = match representation {
        "original" => match load_original_bytes(state, tenant_ctx, document_id).await {
            Ok(v) => v,
            Err(ApiError::NotFound(msg)) => {
                return Ok(eq_error(ErrorCode::NotFound, msg, None));
            }
            Err(ApiError::Forbidden(_)) => {
                return Ok(eq_error(
                    ErrorCode::Forbidden,
                    "not allowed to read this original",
                    None,
                ));
            }
            Err(e) => return Err(e),
        },
        "markdown" => match load_markdown_bytes(state, document_id).await {
            Ok(v) => v,
            Err(ApiError::NotFound(_)) => {
                return Ok(eq_error(
                    ErrorCode::NotReady,
                    "markdown is not ready; poll eq_task_get until indexed",
                    None,
                ));
            }
            Err(e) => return Err(e),
        },
        _ => {
            return Ok(eq_error(
                ErrorCode::InvalidId,
                "representation must be original or markdown",
                None,
            ));
        }
    };

    Ok(chunk_blob_envelope(
        document_id,
        representation,
        &media_type,
        &bytes,
        offset,
        max_bytes,
    ))
}

async fn load_markdown_bytes(state: &AppState, document_id: &str) -> ApiResult<(Vec<u8>, String)> {
    let metadata = match crate::services::load_staging_first_metadata(
        state.storage.kv_storage.as_ref(),
        document_id,
    )
    .await
    {
        Ok(Some((_, v))) => v,
        Ok(None) => {
            return Err(ApiError::NotFound(format!(
                "Document not found: {document_id}"
            )));
        }
        Err(e) => return Err(ApiError::Internal(e)),
    };
    let status = metadata
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("pending");
    let body = load_document_body(&state.storage, document_id, &metadata).await;
    match body {
        Some(b) if !b.markdown.trim().is_empty() => {
            Ok((b.markdown.into_bytes(), "text/markdown".to_string()))
        }
        _ if status == "indexed" || status == "completed" => {
            Err(ApiError::NotFound("Markdown content not found".into()))
        }
        _ => Err(ApiError::NotFound(
            "markdown is not ready; poll eq_task_get until indexed".into(),
        )),
    }
}

pub fn chunk_blob_envelope(
    document_id: &str,
    representation: &str,
    media_type: &str,
    bytes: &[u8],
    offset: usize,
    max_bytes: usize,
) -> Value {
    let total = bytes.len();
    let start = offset.min(total);
    let end = (start + max_bytes).min(total);
    let slice = &bytes[start..end];
    let next_offset = if end < total { Some(end) } else { None };
    let sha = hex::encode(Sha256::digest(bytes));
    let blob_b64 = STANDARD.encode(slice);
    let mut env = EnvelopeBuilder::new("document_download", BudgetClass::Standard)
        .insert("document_id", json!(document_id))
        .insert("representation", json!(representation))
        .insert("media_type", json!(media_type))
        .insert("byte_length", json!(total))
        .insert("offset", json!(start))
        .insert("chunk_length", json!(slice.len()))
        .insert("sha256", json!(sha));
    if let Some(n) = next_offset {
        env = env.insert("next_offset", json!(n)).truncation(json!({
            "truncated": true,
        }));
    }
    let mut built = env.build();
    if let Some(obj) = built.as_object_mut() {
        obj.insert(
            EXTRA_CONTENT_KEY.into(),
            json!([{
                "type": "resource",
                "resource": {
                    "uri": format!("eq://download/{document_id}/{representation}"),
                    "mimeType": media_type,
                    "blob": blob_b64
                }
            }]),
        );
    }
    built
}

pub async fn read_text_resource(state: &AppState, document_id: &str) -> ApiResult<String> {
    let (bytes, _) = load_markdown_bytes(state, document_id).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub async fn read_original_resource(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
) -> ApiResult<(Vec<u8>, String)> {
    load_original_bytes(state, tenant_ctx, document_id).await
}
