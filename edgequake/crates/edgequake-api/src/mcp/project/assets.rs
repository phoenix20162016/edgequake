//! Illustration assets as MCP ImageContent (SPEC-161). Bytes stay off structuredContent.

use image::{imageops::FilterType, DynamicImage, ImageFormat};
use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;

use super::blob::blob_max_bytes;
use super::budget::BudgetClass;
use super::envelope::EnvelopeBuilder;
use super::errors::{eq_error, ErrorCode};
use base64::{engine::general_purpose::STANDARD, Engine};

pub async fn eq_asset_get(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let document_id = args
        .get("document_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("document_id required".into()))?;
    let asset_id = args
        .get("asset_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("asset_id required".into()))?;
    if asset_id.contains("..") || asset_id.contains('/') || asset_id.contains('\\') {
        return Ok(eq_error(
            ErrorCode::InvalidId,
            "asset_id must not contain path separators",
            None,
        ));
    }

    let (bytes, mime) = match load_asset_bytes(state, tenant_ctx, document_id, asset_id).await {
        Ok(v) => v,
        Err(ApiError::NotFound(msg)) => {
            return Ok(eq_error(ErrorCode::NotFound, msg, None));
        }
        Err(e) => return Err(e),
    };

    let mime_lc = mime.to_ascii_lowercase();
    if mime_lc != "image/png" && mime_lc != "image/jpeg" && mime_lc != "image/jpg" {
        return Ok(eq_error(
            ErrorCode::UnsupportedMedia,
            format!("unsupported media type: {mime}"),
            None,
        ));
    }
    let mime_out = if mime_lc.contains("jpeg") || mime_lc.contains("jpg") {
        "image/jpeg"
    } else {
        "image/png"
    };

    let out_bytes = if bytes.len() > blob_max_bytes() {
        downsample(&bytes, mime_out, blob_max_bytes()).unwrap_or(bytes)
    } else {
        bytes
    };

    let mut env = EnvelopeBuilder::new("asset_get", BudgetClass::Standard)
        .insert("document_id", json!(document_id))
        .insert("asset_id", json!(asset_id))
        .insert("media_type", json!(mime_out))
        .insert("byte_length", json!(out_bytes.len()))
        .build();
    if let Some(obj) = env.as_object_mut() {
        obj.insert(
            "_mcp_image".into(),
            json!({
                "mimeType": mime_out,
                "data": STANDARD.encode(&out_bytes),
            }),
        );
    }
    Ok(env)
}

async fn load_asset_bytes(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
    asset_id: &str,
) -> ApiResult<(Vec<u8>, String)> {
    #[cfg(feature = "postgres")]
    {
        let workspace_id = uuid::Uuid::parse_str(&tenant_ctx.workspace_id_or_default()).ok();
        let storage = state.storage.mm_asset_storage.as_deref();
        return crate::services::document_mm_asset_persist::load_mm_asset_bytes_by_id(
            storage,
            document_id,
            workspace_id,
            asset_id,
        )
        .await;
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (state, tenant_ctx, document_id, asset_id);
        Err(ApiError::NotFound("mm-asset not found".into()))
    }
}

fn downsample(bytes: &[u8], mime: &str, cap: usize) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    let mut w = img.width().max(1);
    let mut h = img.height().max(1);
    let mut current = img;
    for _ in 0..8 {
        if encode(&current, mime)
            .map(|b| b.len() <= cap)
            .unwrap_or(false)
        {
            return encode(&current, mime);
        }
        w = (w / 2).max(1);
        h = (h / 2).max(1);
        current = current.resize(w, h, FilterType::Triangle);
    }
    encode(&current, mime)
}

fn encode(img: &DynamicImage, mime: &str) -> Option<Vec<u8>> {
    let mut buf = std::io::Cursor::new(Vec::new());
    let fmt = if mime == "image/jpeg" {
        ImageFormat::Jpeg
    } else {
        ImageFormat::Png
    };
    img.write_to(&mut buf, fmt).ok()?;
    Some(buf.into_inner())
}

pub async fn list_asset_metadata(
    state: &AppState,
    tenant_ctx: &TenantContext,
    document_id: &str,
) -> ApiResult<Vec<Value>> {
    #[cfg(feature = "postgres")]
    {
        let workspace_id = uuid::Uuid::parse_str(&tenant_ctx.workspace_id_or_default())
            .map_err(|_| ApiError::BadRequest("invalid workspace id".into()))?;
        let storage = state.storage.mm_asset_storage.as_deref();
        let rows =
            crate::services::document_mm_asset_persist::list_mm_asset_summaries_for_document(
                storage,
                document_id,
                workspace_id,
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|s| {
                json!({
                    "id": s.asset_id,
                    "kind": s.asset_kind,
                    "page": s.page_num,
                    "media_type": s.content_type,
                    "byte_length": s.file_size_bytes,
                })
            })
            .collect())
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (state, tenant_ctx, document_id);
        Ok(Vec::new())
    }
}
