//! `GET /api/v1/decision/status` and `GET /api/v1/decision/models` (SPEC-160).
//!
//! Both answers are always 200. A down backend is a state, not an error: the
//! workspace card shows it, and an admin may configure a workspace before the
//! backend is up. The probe stops after 3 s and shows the host only.

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::Json;
use edgequake_pipeline::extractor::decision::{DecisionModels, DecisionStatus};
use serde::Deserialize;
use serde_json::Value;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::middleware::TenantContext;
use crate::state::AppState;

/// Query of the status and models endpoints.
#[derive(Debug, Deserialize, IntoParams)]
pub struct DecisionStatusQuery {
    /// Probe this model tag instead of the server default (a workspace's own model).
    pub model: Option<String>,
}

/// Load workspace metadata from `X-Workspace-ID` when the header is a UUID.
async fn workspace_meta(state: &AppState, tenant_ctx: &TenantContext) -> HashMap<String, Value> {
    let Some(id) = tenant_ctx.workspace_id.as_deref().and_then(|raw| Uuid::parse_str(raw.trim()).ok())
    else {
        return HashMap::new();
    };
    match state.workspace_service.get_workspace(id).await {
        Ok(Some(ws)) => ws.metadata,
        _ => HashMap::new(),
    }
}

/// Probe the decision backend.
#[utoipa::path(
    get,
    path = "/api/v1/decision/status",
    tag = "Settings",
    params(DecisionStatusQuery),
    responses(
        (status = 200, description = "Decision backend state. Host only, no secrets.", body = Object)
    )
)]
pub async fn get_decision_status(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Query(query): Query<DecisionStatusQuery>,
) -> Json<DecisionStatus> {
    let meta = workspace_meta(&state, &tenant_ctx).await;
    Json(
        state
            .decision
            .status(query.model.as_deref(), Some(&meta))
            .await,
    )
}

/// List models on the decision host.
#[utoipa::path(
    get,
    path = "/api/v1/decision/models",
    tag = "Settings",
    params(DecisionStatusQuery),
    responses(
        (status = 200, description = "Models on the decision host. Host only, no secrets.", body = Object)
    )
)]
pub async fn get_decision_models(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Query(query): Query<DecisionStatusQuery>,
) -> Json<DecisionModels> {
    let meta = workspace_meta(&state, &tenant_ctx).await;
    Json(
        state
            .decision
            .list_models(query.model.as_deref(), Some(&meta))
            .await,
    )
}
