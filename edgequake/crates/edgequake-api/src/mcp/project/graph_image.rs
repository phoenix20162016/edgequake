//! eq_graph_image — neighborhood PNG. Hop clamp matches eq_neighborhood.

use std::collections::{HashMap, HashSet};

use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::services::entity_neighborhood::build_entity_neighborhood;
use crate::state::AppState;

use super::budget::BudgetClass;
use super::edge_filter::{self, split_edges};
use super::entity_ref::{not_found_message, resolve_entity_node};
use super::envelope::EnvelopeBuilder;
use super::errors::{eq_error, ErrorCode};
use super::graph::{clamp_neighborhood_hops, parse_string_list, DEFAULT_NEIGHBOR_CAP};
use super::graph_layout::layout_graph;
use super::graph_raster::raster_png;
use super::ids::{is_artifact_type, resolve_entity_lookup, title_case_label};
use base64::{engine::general_purpose::STANDARD, Engine};
use edgequake_storage::EntityId;

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
    let doc_filter = parse_string_list(args, "document_ids");
    let edge_types = parse_string_list(args, "edge_types");
    if !doc_filter.is_empty() {
        if let Some(scope) =
            super::doc_scope::DocumentScope::try_resolve_ids_only(state, tenant_ctx, &doc_filter)
                .await?
        {
            if let Some(unknown) = scope.unknown_ids.first() {
                return Ok(eq_error(
                    ErrorCode::NotFound,
                    format!("Document not found: {unknown}"),
                    None,
                ));
            }
        }
    }

    let center = match resolve_entity_node(
        state.storage.graph_storage.as_ref(),
        tenant_ctx,
        entity_id,
    )
    .await
    {
        Ok(n) => n,
        Err(e @ ApiError::NotFound(_)) => {
            return Ok(eq_error(
                ErrorCode::NotFound,
                not_found_message(&e, entity_id),
                None,
            ));
        }
        Err(e) => return Err(e),
    };

    let (nodes, edges) = build_entity_neighborhood(
        &state.storage.graph_storage,
        tenant_ctx,
        &center.id,
        max_hops,
    )
    .await?;

    let focus = center.id.clone();
    let hops = hop_distances(&focus, &nodes, &edges);
    let mut kept: Vec<_> = nodes
        .iter()
        .filter(|n| include_artifacts || !is_artifact_type(&n.entity_type))
        .cloned()
        .collect();
    // Ensure center is present even if filtered as artifact.
    if !kept.iter().any(|n| n.id == focus) {
        if let Some(c) = nodes.iter().find(|n| n.id == focus) {
            kept.insert(0, c.clone());
        } else {
            kept.insert(
                0,
                crate::handlers::entities_types::NeighborhoodNode {
                    id: focus.clone(),
                    label: EntityId::bare_name_from_graph_node_id(&focus).to_string(),
                    entity_type: center
                        .properties
                        .get("entity_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("CONCEPT")
                        .to_string(),
                    description: center
                        .properties
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    degree: crate::handlers::graph_types::DegreeBreakdown::from_total(0),
                },
            );
        }
    }
    kept.sort_by(|a, b| {
        let a_center = a.id == focus;
        let b_center = b.id == focus;
        b_center
            .cmp(&a_center)
            .then_with(|| b.degree.total.cmp(&a.degree.total))
            .then_with(|| a.id.cmp(&b.id))
    });
    // Center always kept; cap applies to others.
    let (center_nodes, mut others): (Vec<_>, Vec<_>) =
        kept.into_iter().partition(|n| n.id == focus);
    let omitted = others
        .len()
        .saturating_sub(DEFAULT_NEIGHBOR_CAP.saturating_sub(1));
    others.truncate(DEFAULT_NEIGHBOR_CAP.saturating_sub(1));
    let mut kept = center_nodes;
    kept.extend(others);
    let kept_ids: HashSet<_> = kept.iter().map(|n| n.id.clone()).collect();

    let mut edge_split = split_edges(&edges, include_weak_edges);
    if !edge_types.is_empty() {
        edge_split.kept.retain(|e| {
            edge_types
                .iter()
                .any(|t| t.eq_ignore_ascii_case(&e.relation_type))
        });
    }
    if !doc_filter.is_empty() {
        let ids: Vec<String> = kept.iter().map(|n| n.id.clone()).collect();
        let batch = state
            .storage
            .graph_storage
            .get_nodes_batch(&ids)
            .await
            .unwrap_or_else(|_| std::collections::HashMap::new());
        kept.retain(|n| {
            n.id == focus
                || batch.get(&n.id).is_some_and(|node| {
                    super::graph::entity_has_docs(&node.properties, &doc_filter)
                })
        });
    }
    let layout_nodes: Vec<(String, u32, String)> = kept
        .iter()
        .map(|n| {
            let h = *hops.get(&n.id).or_else(|| hops.get(&focus)).unwrap_or(&1);
            let bare = EntityId::bare_name_from_graph_node_id(&n.id);
            (n.id.clone(), h, title_case_label(bare))
        })
        .collect();
    let layout_edges: Vec<(String, String)> = edge_split
        .kept
        .iter()
        .filter(|e| kept_ids.contains(&e.source) && kept_ids.contains(&e.target))
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();

    let (laid, laid_edges) = layout_graph(&focus, layout_nodes, layout_edges);
    let png = raster_png(&laid, &laid_edges);
    let mut env = EnvelopeBuilder::new("graph_image", BudgetClass::Standard)
        .insert("entity_id", json!(entity_id))
        .insert("max_hops_used", json!(max_hops))
        .insert("node_count", json!(laid.len()))
        .insert("strong_edge_count", json!(edge_split.strong_count))
        .insert("weak_edge_count", json!(edge_split.weak_count))
        .insert("media_type", json!("image/png"));
    if edge_split.dropped_weak > 0 {
        env = env.insert("hint", json!(edge_filter::WEAK_EDGE_HINT));
    }
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
        if resolve_entity_lookup(&n.id) == resolve_entity_lookup(focus) || n.id == focus {
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
