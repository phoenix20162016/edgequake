//! eq_graph_image — neighborhood PNG. Hop clamp matches eq_neighborhood.

use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::services::entity_neighborhood::build_entity_neighborhood;
use crate::state::AppState;

use super::budget::BudgetClass;
use super::envelope::EnvelopeBuilder;
use super::errors::{eq_error, ErrorCode};
use super::graph::{clamp_neighborhood_hops, DEFAULT_NEIGHBOR_CAP};
use super::graph_layout::layout_graph;
use super::graph_raster::raster_png;
use super::ids::{is_artifact_type, resolve_entity_lookup, title_case_label};
use base64::{engine::general_purpose::STANDARD, Engine};

pub async fn eq_graph_image(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let budget = BudgetClass::parse(args.get("budget").and_then(|v| v.as_str()));
    let entity_id = args
        .get("entity_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("entity_id required".into()))?;
    let max_hops = clamp_neighborhood_hops(
        args.get("max_hops").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
        budget,
    );
    let include_artifacts = args
        .get("include_artifacts")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let include_weak_edges = args
        .get("include_weak_edges")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let lookup = resolve_entity_lookup(entity_id);

    if crate::handlers::isolation::load_node_for_tenant_context(
        state.storage.graph_storage.as_ref(),
        &lookup,
        tenant_ctx,
    )
    .await
    .is_err()
    {
        return Ok(eq_error(
            ErrorCode::NotFound,
            format!("Entity not found: {entity_id}"),
            None,
        ));
    }

    let (nodes, edges) =
        build_entity_neighborhood(&state.storage.graph_storage, tenant_ctx, &lookup, max_hops)
            .await?;

    let hops = hop_distances(&lookup, &nodes, &edges);
    let mut kept: Vec<_> = nodes
        .iter()
        .filter(|n| include_artifacts || !is_artifact_type(&n.entity_type))
        .cloned()
        .collect();
    kept.sort_by(|a, b| {
        b.degree
            .total
            .cmp(&a.degree.total)
            .then_with(|| a.id.cmp(&b.id))
    });
    let omitted = kept.len().saturating_sub(DEFAULT_NEIGHBOR_CAP);
    kept.truncate(DEFAULT_NEIGHBOR_CAP);
    let kept_ids: HashSet<_> = kept.iter().map(|n| n.id.clone()).collect();

    let layout_nodes: Vec<(String, u32, String)> = kept
        .iter()
        .map(|n| {
            let h = *hops.get(&n.id).or_else(|| hops.get(&lookup)).unwrap_or(&1);
            (n.id.clone(), h, title_case_label(&n.id))
        })
        .collect();
    let layout_edges: Vec<(String, String)> = edges
        .iter()
        .filter(|e| {
            if !include_weak_edges && e.relation_type.eq_ignore_ascii_case("RELATED_TO") {
                return false;
            }
            kept_ids.contains(&e.source) && kept_ids.contains(&e.target)
        })
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();

    let (laid, laid_edges) = layout_graph(&lookup, layout_nodes, layout_edges);
    let png = raster_png(&laid, &laid_edges);
    let mut env = EnvelopeBuilder::new("graph_image", BudgetClass::Standard)
        .insert("entity_id", json!(entity_id))
        .insert("max_hops_used", json!(max_hops))
        .insert("node_count", json!(laid.len()))
        .insert("media_type", json!("image/png"));
    if omitted > 0 {
        env = env.truncation(json!({
            "truncated": true,
            "omitted_entities": omitted,
        }));
    }
    let mut built = env.build();
    if let Some(obj) = built.as_object_mut() {
        obj.insert(
            "_mcp_image".into(),
            json!({
                "mimeType": "image/png",
                "data": STANDARD.encode(&png),
            }),
        );
    }
    Ok(built)
}

fn hop_distances(
    focus: &str,
    nodes: &[crate::handlers::entities_types::NeighborhoodNode],
    edges: &[crate::handlers::entities_types::NeighborhoodEdge],
) -> HashMap<String, u32> {
    let mut dist = HashMap::new();
    dist.insert(focus.to_string(), 0);
    for n in nodes {
        if resolve_entity_lookup(&n.id) == focus {
            dist.insert(n.id.clone(), 0);
        }
    }
    for hop in 1..=3 {
        let mut next = dist.clone();
        for e in edges {
            let ds = dist.get(&e.source).copied();
            let dt = dist.get(&e.target).copied();
            if ds == Some(hop - 1) {
                next.entry(e.target.clone()).or_insert(hop);
            }
            if dt == Some(hop - 1) {
                next.entry(e.source.clone()).or_insert(hop);
            }
        }
        dist = next;
    }
    dist
}
