//! Graph-first entity admission for Ask / exact-name queries.
//!
//! When typed ANN misses (wrong embedding model key, missing vectors), the
//! selected entity and exact label hits still exist in AGE. Admit them before
//! the empty-context apology fires: resolve via [`EntityId::exact_lookup_candidates`],
//! load the node, expand one hop, and pull `source_chunk_ids` from KV.

use std::collections::HashSet;

use edgequake_storage::traits::GraphReadView;
use edgequake_storage::EntityId;

use crate::context::{QueryContext, RetrievedChunk};
use crate::graph_ppr::GraphWalkMode;
use crate::helpers::{build_entity_from_node, build_relationship_from_edge};
use crate::keywords::ExtractedKeywords;

fn bare_entity_norm(name: &str) -> String {
    let bare = name.rsplit("::").next().unwrap_or(name);
    EntityId::new(bare).as_str().to_string()
}

pub const META_GRAPH_SEED_ENTITIES: &str = "graph_seed_entities";
pub const META_GRAPH_SEED_CHUNK_IDS: &str = "graph_seed_chunk_ids";

const MAX_SEED_CHUNKS: usize = 8;
const MAX_LABEL_CANDIDATES: usize = 8;

/// Admit explicit seed ids and exact label matches into `context`.
pub async fn admit_into_context(
    graph: GraphReadView<'_>,
    kv: Option<&dyn edgequake_storage::traits::KVStorage>,
    context: &mut QueryContext,
    seed_entity_ids: &[String],
    query_text: &str,
    keywords: &ExtractedKeywords,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
) -> crate::error::Result<usize> {
    let mut seeds: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for raw in seed_entity_ids {
        let t = raw.trim();
        if t.is_empty() || !seen.insert(t.to_string()) {
            continue;
        }
        seeds.push(t.to_string());
    }

    for cand in exact_label_candidates(query_text, keywords) {
        if seen.insert(cand.clone()) {
            seeds.push(cand);
        }
    }

    if seeds.is_empty() {
        return Ok(0);
    }

    let mut resolved_ids: Vec<String> = Vec::new();
    let mut resolved_seen: HashSet<String> = HashSet::new();

    for seed in &seeds {
        if let Some(id) = resolve_seed_node_id(&graph, seed, workspace_id).await? {
            if resolved_seen.insert(id.clone()) {
                resolved_ids.push(id);
            }
        }
    }

    if resolved_ids.is_empty() {
        return Ok(0);
    }

    let existing: HashSet<String> = context
        .entities
        .iter()
        .map(|e| bare_entity_norm(&e.name))
        .collect();
    let (nodes_map, degrees) = tokio::join!(
        graph.get_nodes_batch(&resolved_ids),
        graph.node_degrees_batch(&resolved_ids),
    );
    let nodes_map = nodes_map?;
    let degrees: std::collections::HashMap<String, usize> = degrees?.into_iter().collect();

    let mut admitted: Vec<String> = Vec::new();
    let mut chunk_ids: Vec<String> = Vec::new();

    for id in &resolved_ids {
        if existing.contains(&bare_entity_norm(id)) {
            continue;
        }
        let Some(node) = nodes_map.get(id) else {
            continue;
        };
        let degree = degrees.get(id).copied().unwrap_or(0);
        let entity = build_entity_from_node(id, &node.properties, degree, 1.0);
        for cid in &entity.source_chunk_ids {
            if !chunk_ids.iter().any(|x| x == cid) {
                chunk_ids.push(cid.clone());
            }
        }
        admitted.push(id.clone());
        context.add_entity(entity);
    }

    if admitted.is_empty() {
        return Ok(0);
    }

    let edges = crate::graph_expand::expand_neighborhood_edges(
        &graph,
        &admitted,
        1,
        64,
        GraphWalkMode::Bfs,
        tenant_id,
        workspace_id,
    )
    .await?;

    let existing_rel: HashSet<(String, String, String)> = context
        .relationships
        .iter()
        .map(|r| (r.source.clone(), r.target.clone(), r.relation_type.clone()))
        .collect();

    for edge in edges {
        let key = (
            edge.source.clone(),
            edge.target.clone(),
            edge.properties
                .get("relation_type")
                .and_then(|v| v.as_str())
                .unwrap_or("RELATED_TO")
                .to_string(),
        );
        if existing_rel.contains(&key) {
            continue;
        }
        let rel = build_relationship_from_edge(&edge.source, &edge.target, &edge.properties);
        context.add_relationship(rel);
    }

    // Neighbor nodes referenced by new edges (1-hop star).
    let mut neighbor_ids: Vec<String> = Vec::new();
    let mut neighbor_seen: HashSet<String> = admitted.iter().cloned().collect();
    for rel in &context.relationships {
        for endpoint in [&rel.source, &rel.target] {
            if neighbor_seen.insert(endpoint.clone()) {
                neighbor_ids.push(endpoint.clone());
            }
        }
    }
    if !neighbor_ids.is_empty() {
        let n_map = graph.get_nodes_batch(&neighbor_ids).await?;
        let n_deg: std::collections::HashMap<String, usize> = graph
            .node_degrees_batch(&neighbor_ids)
            .await?
            .into_iter()
            .collect();
        let have: HashSet<String> = context
            .entities
            .iter()
            .map(|e| bare_entity_norm(&e.name))
            .collect();
        for id in &neighbor_ids {
            if have.contains(&bare_entity_norm(id)) {
                continue;
            }
            if let Some(node) = n_map.get(id) {
                let degree = n_deg.get(id).copied().unwrap_or(0);
                context.add_entity(build_entity_from_node(id, &node.properties, degree, 0.5));
            }
        }
    }

    materialize_seed_chunks(kv, context, &chunk_ids).await;

    context.metadata.insert(
        META_GRAPH_SEED_ENTITIES.to_string(),
        serde_json::json!(admitted),
    );
    if !chunk_ids.is_empty() {
        context.metadata.insert(
            META_GRAPH_SEED_CHUNK_IDS.to_string(),
            serde_json::json!(chunk_ids),
        );
    }

    tracing::info!(
        admitted = admitted.len(),
        seed_ids = seed_entity_ids.len(),
        chunks = chunk_ids.len(),
        "graph_seed_admit: AGE entities admitted before/alongside ANN"
    );

    Ok(admitted.len())
}

async fn resolve_seed_node_id(
    graph: &GraphReadView<'_>,
    raw: &str,
    workspace_id: Option<&str>,
) -> crate::error::Result<Option<String>> {
    for cand in EntityId::exact_lookup_candidates(raw, workspace_id) {
        if let Some(node) = graph.get_node(&cand).await? {
            return Ok(Some(node.id));
        }
    }

    // Lexical label search: first exact (case-insensitive) hit.
    let labels = graph
        .search_labels(raw, 5, None, workspace_id)
        .await
        .unwrap_or_default();
    let want = EntityId::new(EntityId::bare_name_from_graph_node_id(raw));
    for label in labels {
        let bare = EntityId::bare_name_from_graph_node_id(&label);
        if EntityId::new(bare).as_str() == want.as_str() || label.eq_ignore_ascii_case(raw) {
            for cand in EntityId::exact_lookup_candidates(&label, workspace_id) {
                if let Some(node) = graph.get_node(&cand).await? {
                    return Ok(Some(node.id));
                }
            }
        }
    }
    Ok(None)
}

/// Tokens / keywords that may be exact entity names (includes hyphenated unigrams).
pub fn exact_label_candidates(query_text: &str, keywords: &ExtractedKeywords) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let push = |s: &str, out: &mut Vec<String>, seen: &mut HashSet<String>| {
        let t = s.trim();
        if t.len() < 2 || !seen.insert(t.to_ascii_lowercase()) {
            return;
        }
        out.push(t.to_string());
    };

    for kw in keywords
        .low_level
        .iter()
        .chain(keywords.high_level.iter())
    {
        push(kw, &mut out, &mut seen);
        if out.len() >= MAX_LABEL_CANDIDATES {
            return out;
        }
    }

    // Hyphenated / underscored tokens from the question (e.g. Gemma3-4b).
    for tok in query_text.split(|c: char| c.is_whitespace() || "?,.;:!()[]{}\"'".contains(c)) {
        let t = tok.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_');
        if t.len() >= 3
            && (t.contains('-') || t.contains('_') || t.chars().any(|c| c.is_ascii_digit()))
        {
            push(t, &mut out, &mut seen);
            if out.len() >= MAX_LABEL_CANDIDATES {
                return out;
            }
        }
    }

    out
}

async fn materialize_seed_chunks(
    kv: Option<&dyn edgequake_storage::traits::KVStorage>,
    context: &mut QueryContext,
    chunk_ids: &[String],
) {
    let Some(kv) = kv else {
        return;
    };
    if chunk_ids.is_empty() {
        return;
    }
    let have: HashSet<&str> = context.chunks.iter().map(|c| c.id.as_str()).collect();
    let need: Vec<String> = chunk_ids
        .iter()
        .filter(|id| !have.contains(id.as_str()))
        .take(MAX_SEED_CHUNKS)
        .cloned()
        .collect();
    if need.is_empty() {
        return;
    }
    let Ok(fetched) =
        edgequake_storage::chunk_content::batch_fetch_chunk_contents(kv, &need).await
    else {
        return;
    };
    for id in need {
        if let Some(body) = fetched.get(&id) {
            if !body.is_empty() {
                context.add_chunk(RetrievedChunk::new(id, body.clone(), 1.0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keywords::ExtractedKeywords;

    #[test]
    fn exact_label_candidates_include_hyphenated_token() {
        let kw = ExtractedKeywords::new(
            vec![],
            vec!["Gemma3-4b".into()],
            crate::keywords::QueryIntent::Exploratory,
        );
        let c = exact_label_candidates(
            "What is Gemma3-4b (Organization) in this knowledge graph?",
            &kw,
        );
        assert!(
            c.iter().any(|s| s.eq_ignore_ascii_case("Gemma3-4b")),
            "candidates={c:?}"
        );
    }
}
