//! Agent entity id parse / emit / resolve (SPEC-162 R1–R5).

use edgequake_storage::{fold_slug_key, EntityId, GraphNode};

use crate::error::{ApiError, ApiResult};
use crate::handlers::isolation::load_node_for_tenant_context;
use crate::middleware::TenantContext;
use edgequake_storage::traits::GraphStorageReadOps;

use super::ids::{agent_entity_id, entity_slug};

/// Single source for agent-visible entity ids (R3).
pub fn agent_id_for_node(workspace: &str, storage_id: &str) -> String {
    let bare = EntityId::bare_name_from_graph_node_id(storage_id);
    agent_entity_id(workspace, bare)
}

/// Resolve an agent / legacy entity id to a tenant-scoped storage node (R2/R5).
pub async fn resolve_entity_node(
    graph: &dyn GraphStorageReadOps,
    ctx: &TenantContext,
    entity_id: &str,
) -> ApiResult<GraphNode> {
    let raw = entity_id.trim();
    if raw.is_empty() {
        return Err(ApiError::BadRequest("entity_id required".into()));
    }

    // Reject empty slug after ent: prefix (ent: / ent:ws:).
    if let Some(rest) = raw.strip_prefix("ent:") {
        // `ent:` or `ent:ws:` (empty slug after final colon).
        if rest.is_empty() || rest.ends_with(':') {
            return Err(ApiError::BadRequest("entity_id slug is empty".into()));
        }
        if let Some((ws_in_id, slug)) = rest.split_once(':') {
            if slug.is_empty() {
                return Err(ApiError::BadRequest("entity_id slug is empty".into()));
            }
            if let Some(ctx_ws) = ctx.workspace_id.as_deref() {
                if !ws_in_id.is_empty()
                    && ws_in_id != "default"
                    && ctx_ws != "default"
                    && ws_in_id != ctx_ws
                {
                    return Err(ApiError::NotFound(format!("Entity not found: {entity_id}")));
                }
            }
        }
    }

    let slug = entity_slug(raw);
    if slug.is_empty() {
        return Err(ApiError::BadRequest("entity_id slug is empty".into()));
    }

    let ws = ctx.workspace_id.as_deref();
    // Prefer bare slug / storage forms. Never run separator_variants on raw
    // `ent:uuid:slug` (UUID hyphens explode the probe set).
    let mut candidates: Vec<String> = Vec::new();
    for extra in [
        slug.as_str(),
        EntityId::new(&slug).as_str(),
        fold_slug_key(&slug).as_str(),
    ] {
        if extra.is_empty() {
            continue;
        }
        for c in EntityId::exact_lookup_candidates(extra, ws) {
            if !candidates.iter().any(|x| x == &c) {
                candidates.push(c);
            }
        }
    }
    // Legacy bare / scoped raw (only if it looks like a storage id).
    if !raw.starts_with("ent:") {
        for c in EntityId::exact_lookup_candidates(raw, ws) {
            if !candidates.iter().any(|x| x == &c) {
                candidates.push(c);
            }
        }
    }

    // Expand separator variants for each candidate (cap total probes).
    let mut probes: Vec<String> = Vec::new();
    for c in &candidates {
        for v in EntityId::separator_variants(c, 16) {
            if !probes.iter().any(|x| x == &v) {
                probes.push(v);
            }
        }
        if probes.len() >= 48 {
            break;
        }
    }

    let batch = graph
        .get_nodes_batch(&probes)
        .await
        .map_err(|e| ApiError::Internal(format!("entity lookup: {e}")))?;

    for probe in &probes {
        if let Some(node) = batch.get(probe) {
            if crate::handlers::isolation::properties_match_tenant_context(&node.properties, ctx) {
                return Ok(node.clone());
            }
        }
        // Fall through to load_node for adapters that omit batch misses.
        if let Ok(node) = load_node_for_tenant_context(graph, probe, ctx).await {
            return Ok(node);
        }
    }

    // Bounded search by bare name with fold equality.
    let search_q = EntityId::bare_name_from_graph_node_id(&slug);
    let fold_target = fold_slug_key(search_q);
    if !fold_target.is_empty() {
        let hits = graph
            .search_nodes(search_q, 20, None, ctx.tenant_id.as_deref(), ws)
            .await
            .unwrap_or_default();
        for (node, _) in hits {
            let bare = EntityId::bare_name_from_graph_node_id(&node.id);
            if fold_slug_key(bare) == fold_target || fold_slug_key(&node.id) == fold_target {
                if let Ok(checked) = load_node_for_tenant_context(graph, &node.id, ctx).await {
                    return Ok(checked);
                }
            }
        }
    }

    let tried = EntityId::new(&slug).graph_node_id_for_workspace(ws);
    if tried != raw && !tried.is_empty() {
        return Err(ApiError::NotFound(format!(
            "Entity not found: {entity_id} (storage id {tried})"
        )));
    }
    Err(ApiError::NotFound(format!("Entity not found: {entity_id}")))
}

/// Map ApiError::NotFound into eq/not_found envelope message (keeps agent id).
pub fn not_found_message(err: &ApiError, entity_id: &str) -> String {
    match err {
        ApiError::NotFound(msg) if msg.contains(entity_id) => msg.clone(),
        ApiError::NotFound(msg) => format!("{msg}; agent id {entity_id}"),
        _ => format!("Entity not found: {entity_id}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_id_for_node_strips_workspace_scope() {
        let ws = "e0270f5f-0b6c-4e90-882f-5f9b0eac8cff";
        let scoped = format!("{ws}::SELF-ATTENTION");
        assert_eq!(
            agent_id_for_node(ws, &scoped),
            format!("ent:{ws}:SELF-ATTENTION")
        );
        assert_eq!(
            agent_id_for_node("default", "ACTION_FUSION"),
            "ent:default:ACTION_FUSION"
        );
    }

    #[test]
    fn fold_roundtrip_hyphen_name() {
        assert_eq!(
            fold_slug_key("SELF-ATTENTION"),
            fold_slug_key("SELF_ATTENTION")
        );
    }
}
