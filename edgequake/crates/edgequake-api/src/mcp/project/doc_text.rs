//! Tenant-checked document body loaders for MCP (SPEC-162 R9 / F-162-A).

use crate::error::{ApiError, ApiResult};
use crate::handlers::documents::load_original_bytes;
use crate::middleware::TenantContext;
use crate::services::document_body_loader::load_document_body;
use crate::state::AppState;

pub const DEFAULT_TEXT_LIMIT: usize = 8000;
pub const MAX_TEXT_LIMIT: usize = 32_000;

/// Load markdown bytes only when metadata matches the tenant context.
pub async fn load_markdown_for_tenant(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
) -> ApiResult<(Vec<u8>, String)> {
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

    if !crate::workspace_scope::metadata_matches_tenant_context(&metadata, tenant_ctx) {
        return Err(ApiError::NotFound(format!(
            "Document not found: {document_id}"
        )));
    }

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

pub async fn load_original_for_tenant(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
) -> ApiResult<(Vec<u8>, String)> {
    load_original_bytes(state, tenant_ctx, document_id).await
}

/// Character-based page of UTF-8 text (SPEC-162 R9).
pub fn text_page(full: &str, offset: usize, limit: usize) -> serde_json::Value {
    let chars: Vec<char> = full.chars().collect();
    let total = chars.len();
    let start = offset.min(total);
    let end = (start + limit).min(total);
    let text: String = chars[start..end].iter().collect();
    let next_offset = if end < total {
        serde_json::Value::from(end)
    } else {
        serde_json::Value::Null
    };
    serde_json::json!({
        "text": text,
        "offset": start,
        "limit": limit,
        "total_chars": total,
        "next_offset": next_offset,
    })
}
