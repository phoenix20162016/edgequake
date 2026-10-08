//! SPEC-162 MCP agent interface e2e (A1–A8 / T-162-*).

mod common;

use std::collections::HashMap;

use axum::http::{header, Request, StatusCode};
use base64::{engine::general_purpose::STANDARD, Engine};
use common::spec028_mcp::{
    build_mcp_app, mcp_tools_call, parse_json, tool_structured, tools_call_body,
};
use edgequake_api::middleware::{default_tenant_uuid, default_workspace_uuid};
use edgequake_api::AppState;
use edgequake_storage::EntityId;
use serde_json::{json, Value};
use tower::ServiceExt;

const FIXTURE_TITLE: &str = "Fixture Graph Doc";
const FIXTURE_ENTITY: &str = "SELF-ATTENTION";

async fn seed_completed_markdown(
    state: &AppState,
    tid: &str,
    wid: &str,
    doc_id: &str,
    title: &str,
    body: &str,
) {
    state
        .storage
        .kv_storage
        .upsert(&[
            (
                edgequake_storage::kv_keys::doc_metadata(doc_id),
                json!({
                    "id": doc_id,
                    "title": title,
                    "file_name": "fixture.md",
                    "status": "completed",
                    "tenant_id": tid,
                    "workspace_id": wid,
                }),
            ),
            (
                edgequake_storage::kv_keys::doc_content(doc_id),
                json!({ "content": body }),
            ),
        ])
        .await
        .unwrap();
}

async fn call_scoped(
    app: axum::Router,
    tid: &str,
    wid: &str,
    name: &str,
    args: Value,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("Content-Type", "application/json")
        .header("X-Tenant-ID", tid)
        .header("X-Workspace-ID", wid)
        .header(header::ACCEPT, "application/json")
        .body(axum::body::Body::from(
            tools_call_body(name, args).to_string(),
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    (status, parse_json(response).await)
}

#[tokio::test]
async fn t162_01_02_03_09_agent_id_roundtrip_graph_tools() {
    let state = AppState::test_state();
    state.workspace_service.seed_default_workspace().await;
    let tid = default_tenant_uuid().to_string();
    let wid = default_workspace_uuid().to_string();
    let storage_id = EntityId::new(FIXTURE_ENTITY).scoped_graph_node_id(&wid);
    let mut props = HashMap::new();
    props.insert("entity_type".into(), json!("CONCEPT"));
    props.insert("description".into(), json!("fixture attention"));
    props.insert("tenant_id".into(), json!(tid.clone()));
    props.insert("workspace_id".into(), json!(wid.clone()));
    props.insert("source_document_ids".into(), json!(["doc-fixture-1"]));
    state
        .storage
        .graph_storage
        .upsert_node(&storage_id, props)
        .await
        .unwrap();

    let agent_id = format!("ent:{wid}:{FIXTURE_ENTITY}");
    let app = build_mcp_app(state);

    // A8 / T-162-09
    let (_, miss) = call_scoped(
        app.clone(),
        &tid,
        &wid,
        "eq_entity_get",
        json!({"entity_id": format!("ent:{wid}:NO_SUCH_ENTITY")}),
    )
    .await;
    let miss_sc = tool_structured(&miss);
    assert_eq!(miss_sc["ok"], false, "A8 unknown must fail: {miss_sc}");
    assert_eq!(miss_sc["error"]["code"], "eq/not_found");
    assert!(
        miss_sc["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("NO_SUCH_ENTITY"),
        "A8 error text must contain agent id: {miss_sc}"
    );

    // A1 / T-162-03 get
    let (_, get) = call_scoped(
        app.clone(),
        &tid,
        &wid,
        "eq_entity_get",
        json!({"entity_id": agent_id}),
    )
    .await;
    let get_sc = tool_structured(&get);
    assert_eq!(get_sc["ok"], true, "entity_get: {get_sc}");
    assert_eq!(get_sc["entities"][0]["id"], agent_id);

    // A2 / T-162-02 neighborhood
    let (_, nb) = call_scoped(
        app.clone(),
        &tid,
        &wid,
        "eq_neighborhood",
        json!({"entity_id": agent_id}),
    )
    .await;
    let nb_sc = tool_structured(&nb);
    assert_eq!(nb_sc["ok"], true, "neighborhood: {nb_sc}");
    let ents = nb_sc["entities"].as_array().unwrap();
    assert!(!ents.is_empty(), "A2 center must be present: {nb_sc}");
    assert_eq!(ents[0]["id"], agent_id);
    assert_eq!(nb_sc["edge_count"], 0);

    // A1 / T-162-01 graph_image
    let (_, img) = call_scoped(
        app,
        &tid,
        &wid,
        "eq_graph_image",
        json!({"entity_id": agent_id}),
    )
    .await;
    assert_ne!(img["result"]["isError"], true, "graph_image: {img}");
    let img_sc = tool_structured(&img);
    assert!(
        img_sc["node_count"].as_u64().unwrap_or(0) > 0,
        "A1 node_count>0: {img_sc}"
    );
    let content = img["result"]["content"].as_array().unwrap();
    let png = content.iter().find(|c| c["type"] == "image").expect("PNG");
    let data = STANDARD.decode(png["data"].as_str().unwrap()).unwrap();
    assert!(data.starts_with(b"\x89PNG"), "PNG magic");
}

#[tokio::test]
async fn t162_06_hyphen_underscore_resolve() {
    let state = AppState::test_state();
    state.workspace_service.seed_default_workspace().await;
    let tid = default_tenant_uuid().to_string();
    let wid = default_workspace_uuid().to_string();
    let storage_id = EntityId::new("SELF-ATTENTION").scoped_graph_node_id(&wid);
    let mut props = HashMap::new();
    props.insert("entity_type".into(), json!("CONCEPT"));
    props.insert("description".into(), json!("hyphen stored"));
    props.insert("tenant_id".into(), json!(tid.clone()));
    props.insert("workspace_id".into(), json!(wid.clone()));
    state
        .storage
        .graph_storage
        .upsert_node(&storage_id, props)
        .await
        .unwrap();
    let app = build_mcp_app(state);
    let underscore_id = format!("ent:{wid}:SELF_ATTENTION");
    let (_, get) = call_scoped(
        app,
        &tid,
        &wid,
        "eq_entity_get",
        json!({"entity_id": underscore_id}),
    )
    .await;
    let sc = tool_structured(&get);
    assert_eq!(sc["ok"], true, "fold resolve: {sc}");
    assert_eq!(sc["entities"][0]["id"], format!("ent:{wid}:SELF-ATTENTION"));
}

#[tokio::test]
async fn t162_15_16_17_graph_image_counts() {
    let state = AppState::test_state();
    state.workspace_service.seed_default_workspace().await;
    let tid = default_tenant_uuid().to_string();
    let wid = default_workspace_uuid().to_string();
    let a = EntityId::new("ALPHA").scoped_graph_node_id(&wid);
    let b = EntityId::new("BETA").scoped_graph_node_id(&wid);
    for (id, label) in [(&a, "alpha"), (&b, "beta")] {
        let mut props = HashMap::new();
        props.insert("entity_type".into(), json!("CONCEPT"));
        props.insert("description".into(), json!(label));
        props.insert("tenant_id".into(), json!(tid.clone()));
        props.insert("workspace_id".into(), json!(wid.clone()));
        state
            .storage
            .graph_storage
            .upsert_node(id, props)
            .await
            .unwrap();
    }
    let mut eprops = HashMap::new();
    eprops.insert("relation_type".into(), json!("RELATED_TO"));
    eprops.insert("tenant_id".into(), json!(tid.clone()));
    eprops.insert("workspace_id".into(), json!(wid.clone()));
    state
        .storage
        .graph_storage
        .upsert_edge(&a, &b, eprops)
        .await
        .unwrap();

    let app = build_mcp_app(state);
    let agent_id = format!("ent:{wid}:ALPHA");
    let (_, img) = call_scoped(
        app,
        &tid,
        &wid,
        "eq_graph_image",
        json!({"entity_id": agent_id, "include_weak_edges": false}),
    )
    .await;
    let sc = tool_structured(&img);
    assert_eq!(sc["ok"], true, "{sc}");
    assert!(sc["strong_edge_count"].is_number());
    assert_eq!(sc["weak_edge_count"], 1);
    assert_eq!(
        sc["hint"],
        "Set include_weak_edges to true to show RELATED_TO edges."
    );
}

#[tokio::test]
async fn t162_20_23_document_ids_hard_scope() {
    let state = AppState::test_state();
    state.workspace_service.seed_default_workspace().await;
    let tid = default_tenant_uuid().to_string();
    let wid = default_workspace_uuid().to_string();
    let doc_a = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
    let doc_b = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
    seed_completed_markdown(
        &state,
        &tid,
        &wid,
        doc_a,
        FIXTURE_TITLE,
        &format!("# {FIXTURE_TITLE}\n\n{FIXTURE_ENTITY}"),
    )
    .await;
    seed_completed_markdown(&state, &tid, &wid, doc_b, "Other Doc", "# Other\n").await;

    let a_id = EntityId::new(FIXTURE_ENTITY).scoped_graph_node_id(&wid);
    let b_id = EntityId::new("OTHER_CONCEPT").scoped_graph_node_id(&wid);
    for (id, name, doc) in [
        (&a_id, FIXTURE_ENTITY, doc_a),
        (&b_id, "OTHER_CONCEPT", doc_b),
    ] {
        let mut props = HashMap::new();
        props.insert("entity_type".into(), json!("CONCEPT"));
        props.insert("description".into(), json!(name));
        props.insert("tenant_id".into(), json!(tid.clone()));
        props.insert("workspace_id".into(), json!(wid.clone()));
        props.insert("source_document_ids".into(), json!([doc]));
        state
            .storage
            .graph_storage
            .upsert_node(id, props)
            .await
            .unwrap();
    }

    let app = build_mcp_app(state);

    // A3/A4: entity_search is the graph-scoped analog of search hits (no embedding call).
    let (_, ents) = call_scoped(
        app,
        &tid,
        &wid,
        "eq_entity_search",
        json!({"q": "SELF-ATTENTION", "document_ids": [doc_a]}),
    )
    .await;
    let esc = tool_structured(&ents);
    assert_eq!(esc["ok"], true, "A4 entity_search: {esc}");
    let found = esc["entities"].as_array().unwrap();
    assert!(!found.is_empty(), "A4 expected fixture entity: {esc}");
    for ent in found {
        let docs = ent["document_ids"].as_array().unwrap();
        assert!(
            docs.iter().any(|d| d.as_str() == Some(doc_a)),
            "A4 entity missing scoped doc: {ent}"
        );
        assert!(
            !docs.iter().any(|d| d.as_str() == Some(doc_b)),
            "A4 leaked other doc: {ent}"
        );
    }
}

#[tokio::test]
async fn t162_21_unknown_document_id() {
    let state = AppState::test_state();
    state.workspace_service.seed_default_workspace().await;
    let tid = default_tenant_uuid().to_string();
    let wid = default_workspace_uuid().to_string();
    let app = build_mcp_app(state);
    let (_, bad) = call_scoped(
        app,
        &tid,
        &wid,
        "eq_search",
        json!({"query": "x", "document_ids": ["00000000-0000-0000-0000-000000000099"]}),
    )
    .await;
    let bsc = tool_structured(&bad);
    assert_eq!(bsc["ok"], false, "{bsc}");
    assert_eq!(bsc["error"]["code"], "eq/not_found");
}

#[tokio::test]
async fn t162_27_pattern_no_match() {
    let state = AppState::test_state();
    state.workspace_service.seed_default_workspace().await;
    let tid = default_tenant_uuid().to_string();
    let wid = default_workspace_uuid().to_string();
    let app = build_mcp_app(state);
    let (_, call) = call_scoped(
        app,
        &tid,
        &wid,
        "eq_search",
        json!({"query": "x", "document_pattern": "zzz_no_such_pattern_zzz"}),
    )
    .await;
    let sc = tool_structured(&call);
    assert_eq!(sc["ok"], true, "{sc}");
    assert_eq!(sc["filter_result"], "no_match");
    assert_eq!(sc["documents_considered"], json!([]));
    assert!(sc["message"]
        .as_str()
        .unwrap_or("")
        .contains("No document matches the pattern"));
}

#[tokio::test]
async fn t162_30_35_36_text_and_download_paging() {
    let state = AppState::test_state();
    state.workspace_service.seed_default_workspace().await;
    let tid = default_tenant_uuid().to_string();
    let wid = default_workspace_uuid().to_string();
    let doc = "cccccccc-cccc-cccc-cccc-cccccccccccc";
    let body = format!("# {FIXTURE_TITLE}\n\n{}", "x".repeat(100));
    seed_completed_markdown(&state, &tid, &wid, doc, FIXTURE_TITLE, &body).await;
    let app = build_mcp_app(state);

    let (_, get) = call_scoped(
        app.clone(),
        &tid,
        &wid,
        "eq_document_get",
        json!({"document_id": doc, "include": ["text"], "text_limit": 40}),
    )
    .await;
    let g = tool_structured(&get);
    assert_eq!(g["ok"], true, "A5 document_get: {g}");
    let page = &g["text_page"];
    assert!(
        page["text"].as_str().unwrap_or("").contains(FIXTURE_TITLE),
        "A5 text_page must contain fixture title: {g}"
    );
    assert!(page["total_chars"].as_u64().unwrap_or(0) > 0);
    assert_eq!(page["offset"], 0);
    assert_eq!(page["limit"], 40);

    let (_, dl) = call_scoped(
        app,
        &tid,
        &wid,
        "eq_document_download",
        json!({
            "document_id": doc,
            "representation": "markdown",
            "max_bytes": 65536,
            "offset": 0
        }),
    )
    .await;
    let sc = tool_structured(&dl);
    assert_eq!(sc["ok"], true, "A6 download: {sc}");
    assert!(sc["chunk_length"].as_u64().unwrap_or(0) <= 65536);
    assert!(sc.get("byte_length").is_some());
    assert!(sc.as_object().unwrap().contains_key("next_offset"));
    assert_eq!(sc["chunk_length"], sc["byte_length"]);
    assert!(sc["next_offset"].is_null());
}

#[tokio::test]
async fn t162_38_max_bytes_zero_invalid() {
    let app = build_mcp_app({
        let state = AppState::test_state();
        state.workspace_service.seed_default_workspace().await;
        state
    });
    let (_, ingest) = mcp_tools_call(
        &app,
        "/mcp",
        "eq_ingest",
        json!({"content": "hi fixture", "title": "t"}),
    )
    .await;
    let doc = tool_structured(&ingest)["document_id"].as_str().unwrap();
    let (_, dl) = mcp_tools_call(
        &app,
        "/mcp",
        "eq_document_download",
        json!({
            "document_id": doc,
            "representation": "markdown",
            "max_bytes": 0
        }),
    )
    .await;
    let sc = tool_structured(&dl);
    assert!(
        sc["error"]["code"] == "eq/invalid_id" || sc["error"]["code"] == "eq/not_ready",
        "{sc}"
    );
}
