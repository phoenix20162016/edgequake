//! MCP resources/list + resources/read for eq:// URIs (SPEC-152 / SPEC-161).

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::middleware::TenantContext;

use super::dispatch::DispatchContext;
use super::json_rpc::GatewayError;

pub fn resources_list(tenant_ctx: &TenantContext) -> Value {
    let ws = tenant_ctx.workspace_id.as_deref().unwrap_or("default");
    json!({
        "resources": [
            {
                "uri": format!("eq://{ws}/documents"),
                "name": "documents",
                "description": "Workspace document catalog",
                "mimeType": "application/json"
            },
            {
                "uri": format!("eq://{ws}/workspaces"),
                "name": "workspaces",
                "description": "Visible workspaces",
                "mimeType": "application/json"
            }
        ],
        "resourceTemplates": [
            {
                "uriTemplate": "eq://{workspace}/documents/{doc_id}",
                "name": "document",
                "description": "Document metadata",
                "mimeType": "application/json"
            },
            {
                "uriTemplate": "eq://{workspace}/documents/{doc_id}/text",
                "name": "document-text",
                "description": "Document markdown (available after processing is indexed)",
                "mimeType": "text/plain"
            },
            {
                "uriTemplate": "eq://{workspace}/documents/{doc_id}/original",
                "name": "document-original",
                "description": "Original upload bytes",
                "mimeType": "application/octet-stream"
            },
            {
                "uriTemplate": "eq://{workspace}/chunks/{chunk_id}",
                "name": "chunk",
                "mimeType": "application/json"
            },
            {
                "uriTemplate": "eq://{workspace}/entities/{entity_id}",
                "name": "entity",
                "mimeType": "application/json"
            },
            {
                "uriTemplate": "eq://{workspace}/retrievals/{ret_id}",
                "name": "retrieval",
                "mimeType": "application/json"
            }
        ]
    })
}

pub async fn resources_read(
    ctx: &DispatchContext<'_>,
    params: Option<Value>,
) -> Result<Value, GatewayError> {
    let uri = params
        .as_ref()
        .and_then(|p| p.get("uri"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| GatewayError::Api(ApiError::BadRequest("uri required".into())))?;

    if !uri.starts_with("eq://") {
        return Err(GatewayError::Api(ApiError::BadRequest(
            "uri must start with eq://".into(),
        )));
    }

    if let Some(ret_id) = uri
        .rsplit('/')
        .next()
        .filter(|_| uri.contains("/retrievals/"))
    {
        if ret_id.starts_with("ret_") {
            let args = json!({
                "retrieval_id": ret_id,
                "view": "toc",
                "budget": "standard",
                "include_subgraph": false
            });
            let ws = ctx.tenant_ctx.workspace_id.as_deref().unwrap_or("default");
            let body = crate::mcp::project::fetch::eq_fetch(&args, ws)
                .await
                .map_err(GatewayError::Api)?;
            return Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string(&body).unwrap_or_else(|_| "{}".into())
                }]
            }));
        }
    }

    if let Some(doc_id) = document_id_from_uri(uri, "/text") {
        return match crate::mcp::project::download::read_text_resource(
            ctx.state,
            ctx.tenant_ctx,
            doc_id,
        )
        .await
        {
            Ok(text) => Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/markdown",
                    "text": text
                }]
            })),
            Err(ApiError::NotFound(_)) => Err(GatewayError::Api(ApiError::NotFound(
                "markdown is not ready; poll eq_task_get until indexed".into(),
            ))),
            Err(e) => Err(GatewayError::Api(e)),
        };
    }

    if let Some(doc_id) = document_id_from_uri(uri, "/original") {
        return match crate::mcp::project::download::read_original_resource(
            ctx.state,
            ctx.tenant_ctx,
            doc_id,
        )
        .await
        {
            Ok((bytes, mime)) => Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": mime,
                    "blob": STANDARD.encode(bytes)
                }]
            })),
            Err(e) => Err(GatewayError::Api(e)),
        };
    }

    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": "{\"ok\":true,\"note\":\"hydrate via eq_document_get / eq_entity_get / eq_fetch\"}"
        }]
    }))
}

fn document_id_from_uri<'a>(uri: &'a str, suffix: &str) -> Option<&'a str> {
    let rest = uri.strip_prefix("eq://")?;
    let path = rest.split_once('/')?.1;
    let marker = "/documents/";
    let after = path.split_once(marker)?.1;
    after.strip_suffix(suffix)
}
