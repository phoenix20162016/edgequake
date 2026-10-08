//! Graph tools: entity search / get / neighborhood (SPEC-152 / SPEC-162).

use serde_json::{json, Value};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::services::entity_neighborhood::build_entity_neighborhood;
use crate::state::AppState;

use super::budget::{apply_budget, BudgetClass};
use super::edge_filter::{self, split_edges};
use super::entity_ref::{agent_id_for_node, not_found_message, resolve_entity_node};
use super::envelope::EnvelopeBuilder;
use super::errors::{eq_error, ErrorCode};
use super::ids::{
    is_artifact_type, map_entity_type, resolve_entity_lookup, title_case_label, truncate_chars,
};

pub(crate) const DEFAULT_NEIGHBOR_CAP: usize = 16;

pub(crate) fn clamp_neighborhood_hops(max_hops: u32, budget: BudgetClass) -> u32 {
    let mut hops = max_hops.clamp(1, 3);
    if hops == 3 && budget != BudgetClass::Deep {
        hops = 2;
    }
    hops
}

fn document_ids_from_props(props: &std::collections::HashMap<String, Value>) -> Vec<String> {
    if let Some(arr) = props.get("source_document_ids").and_then(|v| v.as_array()) {
        let ids: Vec<String> = arr
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .filter(|s| !s.is_empty())
            .collect();
        if !ids.is_empty() {
            return ids;
        }
    }
    props
        .get("source_document_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| vec![s.to_string()])
        .unwrap_or_default()
}

pub(crate) fn entity_has_docs(
    props: &std::collections::HashMap<String, Value>,
    allowed: &[String],
) -> bool {
    if allowed.is_empty() {
        return true;
    }
    let docs = document_ids_from_props(props);
    docs.iter().any(|d| allowed.iter().any(|a| a == d))
}

pub(crate) fn parse_string_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn edge_type_allowed(relation_type: &str, allowed: &[String]) -> bool {
    allowed.is_empty()
        || allowed
            .iter()
            .any(|t| t.eq_ignore_ascii_case(relation_type))
}

pub async fn eq_entity_search(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let budget = BudgetClass::parse(args.get("budget").and_then(|v| v.as_str()));
    let q = args
        .get("q")
        .or_else(|| args.get("query"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if q.trim().is_empty() {
        return Ok(eq_error(ErrorCode::InvalidId, "q is required", None));
    }
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .clamp(1, 50) as usize;
    let include_artifacts = args
        .get("include_artifacts")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let type_filter = args.get("type").and_then(|v| v.as_str());
    let type_filter_storage = type_filter.map(|t| t.to_ascii_uppercase());
    let workspace = tenant_ctx.workspace_id.as_deref().unwrap_or("default");

    let doc_filter: Vec<String> = parse_string_list(args, "document_ids");

    // SPEC-162 R6/R7: unknown document ids → not_found (when scope provided).
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

    let nodes = state
        .storage
        .graph_storage
        .search_nodes(
            &q,
            limit * 3,
            type_filter_storage.as_deref(),
            tenant_ctx.tenant_id.as_deref(),
            tenant_ctx.workspace_id.as_deref(),
        )
        .await
        .map_err(|e| ApiError::Internal(format!("entity search: {e}")))?;

    let mut entities = Vec::new();
    for (node, degree) in nodes {
        let etype = node
            .properties
            .get("entity_type")
            .and_then(|v| v.as_str())
            .unwrap_or("CONCEPT")
            .to_string();
        if !include_artifacts && is_artifact_type(&etype) {
            continue;
        }
        let mapped = map_entity_type(&etype);
        if let Some(tf) = type_filter {
            if !mapped.eq_ignore_ascii_case(tf) {
                continue;
            }
        }
        if !entity_has_docs(&node.properties, &doc_filter) {
            continue;
        }
        let desc = node
            .properties
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let doc_ids = document_ids_from_props(&node.properties);
        entities.push(json!({
            "id": agent_id_for_node(workspace, &node.id),
            "name": title_case_label(edgequake_storage::EntityId::bare_name_from_graph_node_id(&node.id)),
            "slug": resolve_entity_lookup(&node.id),
            "type": mapped,
            "one_liner": truncate_chars(&desc, budget.max_one_liner()),
            "degree": degree,
            "document_ids": doc_ids,
        }));
        if entities.len() >= limit {
            break;
        }
    }

    let env = EnvelopeBuilder::new("entity_search", budget)
        .insert("entities", json!(entities))
        .build();
    let (capped, over) = apply_budget(env, budget);
    if over {
        return Ok(eq_error(
            ErrorCode::TruncateInvalid,
            "structuredContent exceeded budget",
            None,
        ));
    }
    Ok(capped)
}

pub async fn eq_entity_get(
    state: &AppState,
    tenant_ctx: &TenantContext,
    args: &Value,
) -> ApiResult<Value> {
    let budget = BudgetClass::parse(args.get("budget").and_then(|v| v.as_str()));
    let entity_id = args
        .get("entity_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("entity_id required".into()))?;
    let workspace = tenant_ctx.workspace_id.as_deref().unwrap_or("default");

    let node = match resolve_entity_node(
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

    let etype = node
        .properties
        .get("entity_type")
        .and_then(|v| v.as_str())
        .unwrap_or("CONCEPT")
        .to_string();
    let desc = node
        .properties
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let agent_id = agent_id_for_node(workspace, &node.id);
    let uri = format!("eq://{}/entities/{}", workspace, agent_id);
    let doc_ids = document_ids_from_props(&node.properties);

    Ok(EnvelopeBuilder::new("entity_get", budget)
        .insert(
            "entities",
            json!([{
                "id": agent_id,
                "name": title_case_label(edgequake_storage::EntityId::bare_name_from_graph_node_id(&node.id)),
                "slug": resolve_entity_lookup(&node.id),
                "type": map_entity_type(&etype),
                "one_liner": truncate_chars(&desc, budget.max_one_liner()),
                "description": truncate_chars(&desc, budget.max_one_liner().saturating_mul(3)),
                "document_ids": doc_ids,
            }]),
        )
        .insert("lineage_resources", json!([uri]))
        .build())
}

pub async fn eq_neighborhood(
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
    let workspace = tenant_ctx.workspace_id.as_deref().unwrap_or("default");
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

    let edge_split = split_edges(&edges, include_weak_edges);
    let kept_edges: Vec<_> = edge_split
        .kept
        .into_iter()
        .filter(|e| edge_type_allowed(&e.relation_type, &edge_types))
        .collect();

    let neighbor_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let neighbor_map = state
        .storage
        .graph_storage
        .get_nodes_batch(&neighbor_ids)
        .await
        .unwrap_or_else(|_| std::collections::HashMap::new());

    let mut entities = Vec::new();
    // Center first (R11).
    {
        let etype = center
            .properties
            .get("entity_type")
            .and_then(|v| v.as_str())
            .unwrap_or("CONCEPT");
        let desc = center
            .properties
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let degree = state
            .storage
            .graph_storage
            .node_degree(&center.id)
            .await
            .unwrap_or(0);
        entities.push(json!({
            "id": agent_id_for_node(workspace, &center.id),
            "name": title_case_label(edgequake_storage::EntityId::bare_name_from_graph_node_id(&center.id)),
            "slug": resolve_entity_lookup(&center.id),
            "type": map_entity_type(etype),
            "one_liner": truncate_chars(desc, budget.max_one_liner()),
            "degree": degree,
            "document_ids": document_ids_from_props(&center.properties),
            "is_center": true,
        }));
    }

    for n in &nodes {
        if n.id == center.id {
            continue;
        }
        if !include_artifacts && is_artifact_type(&n.entity_type) {
            continue;
        }
        let props = neighbor_map
            .get(&n.id)
            .map(|node| &node.properties)
            .cloned()
            .unwrap_or_default();
        if !entity_has_docs(&props, &doc_filter) {
            continue;
        }
        entities.push(json!({
            "id": agent_id_for_node(workspace, &n.id),
            "name": title_case_label(edgequake_storage::EntityId::bare_name_from_graph_node_id(&n.id)),
            "slug": resolve_entity_lookup(&n.id),
            "type": map_entity_type(&n.entity_type),
            "one_liner": truncate_chars(&n.description, budget.max_one_liner()),
            "degree": n.degree.total,
            "document_ids": document_ids_from_props(&props),
        }));
    }
    // Keep center; sort the rest by degree.
    let center_entity = entities.remove(0);
    entities.sort_by(|a, b| {
        let da = a.get("degree").and_then(|v| v.as_u64()).unwrap_or(0);
        let db = b.get("degree").and_then(|v| v.as_u64()).unwrap_or(0);
        db.cmp(&da)
    });
    let neighbor_cap = DEFAULT_NEIGHBOR_CAP.saturating_sub(1);
    let omitted = entities.len().saturating_sub(neighbor_cap);
    entities.truncate(neighbor_cap);
    entities.insert(0, center_entity);

    let entity_storage_ids: std::collections::HashSet<String> = {
        let mut s = std::collections::HashSet::new();
        s.insert(center.id.clone());
        for n in &nodes {
            if entities.iter().any(|e| {
                e.get("id").and_then(|v| v.as_str()) == Some(&agent_id_for_node(workspace, &n.id))
            }) {
                s.insert(n.id.clone());
            }
        }
        s
    };
    let mut relationships = Vec::new();
    for e in &kept_edges {
        if !entity_storage_ids.contains(&e.source) || !entity_storage_ids.contains(&e.target) {
            continue;
        }
        relationships.push(json!({
            "id": format!("rel:{}:{}:{}", e.source, e.relation_type, e.target),
            "type": e.relation_type,
            "source": agent_id_for_node(workspace, &e.source),
            "target": agent_id_for_node(workspace, &e.target),
            "score": e.weight,
        }));
    }

    let path: Vec<String> = entities
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(str::to_string))
        .collect();

    let mut env = EnvelopeBuilder::new("neighborhood", budget)
        .insert("entities", json!(entities))
        .insert("relationships", json!(relationships))
        .insert("path", json!(path))
        .insert("edge_count", json!(relationships.len()))
        .insert("strong_edge_count", json!(edge_split.strong_count))
        .insert("weak_edge_count", json!(edge_split.weak_count));
    if edge_split.dropped_weak > 0 {
        env = env
            .insert("omitted_weak_edges", json!(edge_split.dropped_weak))
            .insert("hint", json!(edge_filter::WEAK_EDGE_HINT));
    }
    if omitted > 0 {
        env = env.truncation(json!({
            "truncated": true,
            "omitted_entities": omitted,
        }));
    }
    let env = env.build();
    let (capped, over) = apply_budget(env, budget);
    if over {
        return Ok(eq_error(
            ErrorCode::TruncateInvalid,
            "structuredContent exceeded budget",
            None,
        ));
    }
    Ok(capped)
}
