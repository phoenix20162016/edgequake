//! Graph batch upsert contract (SPEC-017 P2).
//!
//! Verifies `upsert_nodes_batch` / `upsert_edges_batch` semantic parity across backends.
//! Helpers are shared across backend e2e binaries; not every binary imports every helper.

#![allow(dead_code)]

use std::collections::HashMap;

use edgequake_storage::traits::GraphStorage;

/// Batch-insert nodes and edges; assert all entities exist and counts match.
pub async fn assert_graph_batch_upsert<G: GraphStorage + ?Sized>(storage: &G) {
    let nodes: Vec<(String, HashMap<String, serde_json::Value>)> = (0..5)
        .map(|i| {
            let mut props = HashMap::new();
            props.insert("batch_idx".to_string(), serde_json::json!(i));
            (format!("BATCH_NODE_{i}"), props)
        })
        .collect();

    storage.upsert_nodes_batch(&nodes).await.unwrap();

    for (id, _) in &nodes {
        assert!(
            storage.has_node(id).await.unwrap(),
            "batch node missing: {id}"
        );
        let node = storage.get_node(id).await.unwrap().unwrap();
        assert_eq!(node.id, *id);
    }

    let edges: Vec<(String, String, HashMap<String, serde_json::Value>)> = (0..4)
        .map(|i| {
            (
                format!("BATCH_NODE_{i}"),
                format!("BATCH_NODE_{}", i + 1),
                HashMap::new(),
            )
        })
        .collect();

    storage.upsert_edges_batch(&edges).await.unwrap();

    for (src, tgt, _) in &edges {
        assert!(
            storage.has_edge(src, tgt).await.unwrap(),
            "batch edge missing: {src} -> {tgt}"
        );
    }

    assert_eq!(storage.node_count().await.unwrap(), 5);
    assert_eq!(storage.edge_count().await.unwrap(), 4);
}

/// D-30 multigraph: distinct `relation_type` values between the same endpoints
/// are separate edges. Duplicate `(source, target, relation_type)` rows in one
/// batch must last-write-wins (Postgres ON CONFLICT cardinality).
pub async fn assert_graph_batch_upsert_dedupes_duplicate_endpoints<G: GraphStorage + ?Sized>(
    storage: &G,
) {
    let mut props_a = HashMap::new();
    props_a.insert("label".to_string(), serde_json::json!("A"));
    let mut props_b = HashMap::new();
    props_b.insert("label".to_string(), serde_json::json!("B"));
    storage
        .upsert_nodes_batch(&[
            ("DEDUP_A".to_string(), props_a),
            ("DEDUP_B".to_string(), props_b),
        ])
        .await
        .unwrap();

    let mut first = HashMap::new();
    first.insert("relation_type".to_string(), serde_json::json!("KNOWS"));
    first.insert("description".to_string(), serde_json::json!("first"));
    let mut second_v1 = HashMap::new();
    second_v1.insert("relation_type".to_string(), serde_json::json!("WORKS_WITH"));
    second_v1.insert("description".to_string(), serde_json::json!("second-v1"));
    let mut second_v2 = HashMap::new();
    second_v2.insert("relation_type".to_string(), serde_json::json!("WORKS_WITH"));
    second_v2.insert("description".to_string(), serde_json::json!("second-wins"));

    storage
        .upsert_edges_batch(&[
            ("DEDUP_A".to_string(), "DEDUP_B".to_string(), first),
            ("DEDUP_A".to_string(), "DEDUP_B".to_string(), second_v1),
            ("DEDUP_A".to_string(), "DEDUP_B".to_string(), second_v2),
        ])
        .await
        .expect("duplicate (src,tgt,rel) batch must not fail ON CONFLICT cardinality");

    assert!(
        storage.has_edge("DEDUP_A", "DEDUP_B").await.unwrap(),
        "multigraph edge must exist"
    );

    let edges = storage.get_node_edges("DEDUP_A").await.unwrap();
    let ab: Vec<_> = edges
        .into_iter()
        .filter(|e| e.source == "DEDUP_A" && e.target == "DEDUP_B")
        .collect();
    assert_eq!(
        ab.len(),
        2,
        "D-30: KNOWS + WORKS_WITH are two edges between the same endpoints"
    );

    let works = ab
        .iter()
        .find(|e| e.properties.get("relation_type").and_then(|v| v.as_str()) == Some("WORKS_WITH"))
        .expect("WORKS_WITH edge");
    assert_eq!(
        works.properties.get("description").and_then(|v| v.as_str()),
        Some("second-wins"),
        "last-write-wins on duplicate (src,tgt,rel_type) upsert"
    );
}
