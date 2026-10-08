//! Fold-equivalent entity key resolution (SPEC-162 R13).
//!
//! Compare-only: does not change `EntityId::new` storage output. When an exact
//! key misses, probe separator variants (`-` ↔ `_`) against the graph.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use edgequake_storage::traits::GraphStorage;
use edgequake_storage::{fold_slug_key, EntityId, GraphNode};

use crate::error::Result;

static FOLD_LOCKS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn fold_lock_set() -> &'static Mutex<HashSet<String>> {
    FOLD_LOCKS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// In-process keyed lock per (workspace, fold_key) to reduce same-replica races.
pub struct FoldKeyGuard {
    key: String,
}

impl FoldKeyGuard {
    pub fn acquire(workspace: Option<&str>, fold: &str) -> Self {
        let key = format!("{}|{}", workspace.unwrap_or(""), fold);
        // Best-effort same-replica marker (no spin — avoids test deadlocks).
        if let Ok(mut set) = fold_lock_set().lock() {
            set.insert(key.clone());
        }
        Self { key }
    }
}

impl Drop for FoldKeyGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = fold_lock_set().lock() {
            set.remove(&self.key);
        }
    }
}

/// Remap exact keys to existing fold-equivalent graph nodes.
pub async fn resolve_fold_keys<G: GraphStorage + ?Sized>(
    graph: &G,
    workspace_id: Option<&str>,
    keys: &mut [String],
    existing_map: &mut HashMap<String, GraphNode>,
) -> Result<()> {
    let miss_idxs: Vec<usize> = keys
        .iter()
        .enumerate()
        .filter(|(_, k)| !existing_map.contains_key(k.as_str()))
        .map(|(i, _)| i)
        .collect();
    if miss_idxs.is_empty() {
        return Ok(());
    }

    let mut probes: Vec<String> = Vec::new();
    for &i in &miss_idxs {
        for v in EntityId::separator_variants(&keys[i], 16) {
            if !probes.iter().any(|p| p == &v) {
                probes.push(v);
            }
        }
    }
    if probes.is_empty() {
        return Ok(());
    }

    let found = graph.get_nodes_batch(&probes).await?;
    for &i in &miss_idxs {
        let fold = fold_slug_key(EntityId::bare_name_from_graph_node_id(&keys[i]));
        if fold.is_empty() {
            continue;
        }
        let _guard = FoldKeyGuard::acquire(workspace_id, &fold);
        let mut hit: Option<String> = None;
        for v in EntityId::separator_variants(&keys[i], 16) {
            if let Some(node) = found.get(&v) {
                let node_ws = node.properties.get("workspace_id").and_then(|x| x.as_str());
                let ws_ok = match (workspace_id, node_ws) {
                    (Some(w), Some(nw)) => w == nw,
                    (Some(_), None) => false,
                    (None, _) => true,
                };
                if ws_ok && fold_slug_key(EntityId::bare_name_from_graph_node_id(&node.id)) == fold
                {
                    hit = Some(node.id.clone());
                    existing_map.insert(node.id.clone(), node.clone());
                    break;
                }
            }
        }
        if let Some(target) = hit {
            if target != keys[i] {
                tracing::info!(
                    from = %keys[i],
                    onto = %target,
                    metric = "entity_fold_merge",
                    "SPEC-162: fold resolve onto existing graph node"
                );
                keys[i] = target;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::{ExtractedEntity, ExtractionResult, KnowledgeGraphMerger, MergerConfig};
    use edgequake_storage::traits::{GraphStorage, GraphStorageReadOps, VectorStorage};
    use edgequake_storage::MemoryVectorStorage;

    #[test]
    fn fold_keys_match_hyphen_underscore() {
        assert_eq!(
            fold_slug_key("SELF-ATTENTION"),
            fold_slug_key("SELF_ATTENTION")
        );
    }

    #[test]
    fn separator_variants_include_both() {
        let v = EntityId::separator_variants("SELF-ATTENTION", 16);
        assert!(v.iter().any(|s| s.contains('_')));
    }

    fn attn_entity(name: &str, doc: &str) -> ExtractedEntity {
        ExtractedEntity {
            name: name.to_string(),
            entity_type: "CONCEPT".to_string(),
            description: format!("fixture {name}"),
            importance: 0.9,
            source_spans: vec![],
            source_chunk_ids: vec![format!("{doc}-chunk-0")],
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            source_document_id: Some(doc.to_string()),
            source_file_path: None,
            display_name: None,
            page_num: None,
            figure_index: None,
            asset_id: None,
            mm_subtype: None,
        }
    }

    #[tokio::test]
    async fn t162_45_hyphen_then_underscore_merges_one_node() {
        let ws = "00000000-0000-0000-0000-000000000003";
        let graph = Arc::new(edgequake_storage::MemoryGraphStorage::new("fold-a7"));
        let vector = Arc::new(MemoryVectorStorage::new("fold-a7", 4));
        graph.initialize().await.unwrap();
        vector.initialize().await.unwrap();
        let merger = KnowledgeGraphMerger::new(MergerConfig::default(), graph.clone(), vector)
            .with_tenant_context(Some("t".into()), Some(ws.into()));

        let hyphen_id = EntityId::new("SELF-ATTENTION").graph_node_id_for_workspace(Some(ws));
        let uscore_id = EntityId::new("SELF_ATTENTION").graph_node_id_for_workspace(Some(ws));

        let mut first = ExtractionResult::new("chunk-1");
        first.add_entity(attn_entity("SELF-ATTENTION", "doc-1"));
        merger.merge(vec![first]).await.unwrap();
        assert!(
            graph.get_node(&hyphen_id).await.unwrap().is_some(),
            "first ingest stores hyphen key"
        );

        let mut second = ExtractionResult::new("chunk-2");
        second.add_entity(attn_entity("SELF_ATTENTION", "doc-2"));
        merger.merge(vec![second]).await.unwrap();
        let hyphen = graph.get_node(&hyphen_id).await.unwrap();
        let uscore = graph.get_node(&uscore_id).await.unwrap();
        assert_eq!(
            hyphen.is_some() as u8 + uscore.is_some() as u8,
            1,
            "A7 fold-equivalent ingest must not create a second node hyphen={hyphen:?} uscore={uscore:?}"
        );
    }
}
