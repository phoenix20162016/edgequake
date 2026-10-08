//! Document download as MCP blob chunks. Does not wait for ingest to finish.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;

use super::blob::{blob_max_bytes, EXTRA_CONTENT_KEY};
use super::budget::BudgetClass;
use super::doc_text::{load_markdown_for_tenant, load_original_for_tenant};
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

    let max_bytes_arg = args.get("max_bytes").and_then(|v| v.as_u64());
    if max_bytes_arg == Some(0) {
        return Ok(eq_error(
            ErrorCode::InvalidId,
            "max_bytes must be >= 1",
            None,
        ));
    }
    let max_bytes = max_bytes_arg
        .map(|n| n as usize)
        .unwrap_or_else(blob_max_bytes)
        .min(blob_max_bytes())
        .max(1);

    let (bytes, media_type) = match representation {
        "original" => match load_original_for_tenant(state, tenant_ctx, document_id).await {
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
        "markdown" => match load_markdown_for_tenant(state, tenant_ctx, document_id).await {
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
    let next_offset = if end < total { json!(end) } else { Value::Null };
    let sha = hex::encode(Sha256::digest(bytes));
    let blob_b64 = STANDARD.encode(slice);
    let mut env = EnvelopeBuilder::new("document_download", BudgetClass::Standard)
        .insert("document_id", json!(document_id))
        .insert("representation", json!(representation))
        .insert("media_type", json!(media_type))
        .insert("byte_length", json!(total))
        .insert("offset", json!(start))
        .insert("chunk_length", json!(slice.len()))
        .insert("next_offset", next_offset)
        .insert("sha256", json!(sha));
    if end < total {
        env = env.truncation(json!({
            "truncated": true,
            "next_cursor": format!("bytes:{end}"),
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

pub async fn read_text_resource(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
) -> ApiResult<String> {
    let (bytes, _) = load_markdown_for_tenant(state, tenant_ctx, document_id).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub async fn read_original_resource(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
) -> ApiResult<(Vec<u8>, String)> {
    load_original_for_tenant(state, tenant_ctx, document_id).await
}
