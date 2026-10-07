//! Graph-only entity admission: Ask sentence + seed_entity_ids without ANN rows.
//!
//! Reproduces the Gemma3-4b Ask failure mode: node exists in AGE, entity vector
//! index is empty (or hollow), Smart/mix must still return the entity (no canned
//! apology) and must not prefer popular hubs over exact labels.

use std::collections::HashMap;
use std::sync::Arc;

use edgequake_llm::MockProvider;
use edgequake_query::retrieval_telemetry::{META_POPULAR_NODE_ARM, META_POPULAR_NODE_FALLBACK};
use edgequake_query::{QueryEngine, QueryEngineConfig, QueryMode, QueryRequest, RetrievedEntity};
use edgequake_storage::traits::{GraphStorage, VectorStorage};
use edgequake_storage::{GraphStorageMutateOps, MemoryGraphStorage, MemoryVectorStorage};
use serde_json::json;

const APOLOGY: &str =
    "I'm sorry, but I couldn't find any relevant information in my knowledge base to answer your question.";

const ASK_SENTENCE: &str =
    "What is Gemma3-4b (Organization) in this knowledge graph, and how is it related to neighbouring entities?";

const ENTITY_ID: &str = "00000000-0000-0000-0000-000000000003::GEMMA3-4B";
const NEIGHBOR_ID: &str = "00000000-0000-0000-0000-000000000003::OLLAMA";
const HUB_ID: &str = "00000000-0000-0000-0000-000000000003::ACTION_FUSION";
const HOLLOW_ID: &str = "00000000-0000-0000-0000-000000000003::GHOST_ANN";

async fn seed_gemma_graph(graph: &MemoryGraphStorage) {
    let mut props = HashMap::new();
    props.insert("entity_type".into(), json!("ORGANIZATION"));
    props.insert(
        "description".into(),
        json!("Gemma3 4B language model family"),
    );
    props.insert("label".into(), json!("Gemma3-4b"));
    props.insert("source_chunk_ids".into(), json!(["doc-1-chunk-0"]));
    graph.upsert_node(ENTITY_ID, props).await.unwrap();

    let mut nprops = HashMap::new();
    nprops.insert("entity_type".into(), json!("ORGANIZATION"));
    nprops.insert("description".into(), json!("Ollama runtime"));
    nprops.insert("label".into(), json!("Ollama"));
    graph.upsert_node(NEIGHBOR_ID, nprops).await.unwrap();

    let mut eprops = HashMap::new();
    eprops.insert("relation_type".into(), json!("RELATED_TO"));
    eprops.insert("description".into(), json!("runs on"));
    graph
        .upsert_edge(ENTITY_ID, NEIGHBOR_ID, eprops)
        .await
        .unwrap();
}

async fn seed_popular_hub(graph: &MemoryGraphStorage) {
    let mut hprops = HashMap::new();
    hprops.insert("entity_type".into(), json!("CONCEPT"));
    hprops.insert("description".into(), json!("high-degree distractor hub"));
    hprops.insert("label".into(), json!("Action Fusion"));
    graph.upsert_node(HUB_ID, hprops).await.unwrap();

    for i in 0..8 {
        let leaf = format!("00000000-0000-0000-0000-000000000003::HUB_LEAF_{i}");
        let mut lp = HashMap::new();
        lp.insert("entity_type".into(), json!("CONCEPT"));
        lp.insert("description".into(), json!(format!("leaf {i}")));
        lp.insert("label".into(), json!(format!("Hub Leaf {i}")));
        graph.upsert_node(&leaf, lp).await.unwrap();
        let mut ep = HashMap::new();
        ep.insert("relation_type".into(), json!("RELATED_TO"));
        graph.upsert_edge(HUB_ID, &leaf, ep).await.unwrap();
    }
}

async fn graph_only_engine() -> QueryEngine {
    let dim = 1536;
    let vector = Arc::new(MemoryVectorStorage::new("graph-seed", dim));
    let graph = Arc::new(MemoryGraphStorage::new("graph-seed"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();
    seed_gemma_graph(&graph).await;

    let mock = Arc::new(MockProvider::default());
    mock.add_response("Gemma3-4b is an organization node linked to Ollama.")
        .await;

    QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    )
}

fn has_gemma(entities: &[RetrievedEntity]) -> bool {
    entities.iter().any(|e| {
        e.name.to_ascii_uppercase().contains("GEMMA3")
            || e.description.to_ascii_lowercase().contains("gemma")
    })
}

#[tokio::test]
async fn graph_only_entity_ask_sentence_admits_without_ann() {
    let engine = graph_only_engine().await;
    // No forced ll_keywords — hyphenated token from the Ask sentence is enough.
    let mut req = QueryRequest::new(ASK_SENTENCE);
    req.mode = Some(QueryMode::Mix);

    let resp = engine.query(req).await.expect("query");
    assert!(
        !resp.context.is_empty(),
        "context must contain graph-admitted entity"
    );
    assert!(
        has_gemma(&resp.context.entities),
        "entities={:?}",
        resp.context
            .entities
            .iter()
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    assert_ne!(resp.answer, APOLOGY);
    assert!(!resp.answer.is_empty());
}

#[tokio::test]
async fn graph_only_entity_seed_ids_admits_neighbors() {
    let engine = graph_only_engine().await;
    let mut req = QueryRequest::new(ASK_SENTENCE).with_seed_entity_ids(vec![ENTITY_ID.into()]);
    req.mode = Some(QueryMode::Mix);

    let resp = engine.query(req).await.expect("query");
    assert!(!resp.context.entities.is_empty());
    assert!(
        !resp.context.relationships.is_empty(),
        "1-hop edges must be admitted"
    );
    assert_ne!(resp.answer, APOLOGY);
}

#[tokio::test]
async fn graph_only_entity_stream_is_not_apology() {
    let engine = graph_only_engine().await;
    let mut req = QueryRequest::new(ASK_SENTENCE).with_seed_entity_ids(vec![ENTITY_ID.into()]);
    req.mode = Some(QueryMode::Mix);

    let (ctx, _mode, mut stream) = engine.query_stream_with_context(req).await.expect("stream");
    assert!(!ctx.is_empty());
    use futures::StreamExt;
    let mut answer = String::new();
    while let Some(tok) = stream.next().await {
        answer.push_str(&tok.expect("token"));
    }
    assert_ne!(answer, APOLOGY);
    assert!(!answer.is_empty());
}

#[tokio::test]
async fn hollow_ann_skips_popular_and_admits_label() {
    std::env::set_var("EDGEQUAKE_POPULAR_NODE_FALLBACK", "1");
    let dim = 1536;
    let vector = Arc::new(MemoryVectorStorage::new("hollow-ann", dim));
    let graph = Arc::new(MemoryGraphStorage::new("hollow-ann"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();
    seed_gemma_graph(&graph).await;
    seed_popular_hub(&graph).await;

    // ANN returns an entity id that is absent from the graph (hydration hole).
    vector
        .upsert(&[(
            "entity-ghost".into(),
            vec![0.9_f32; dim],
            json!({
                "type": "entity",
                "entity_name": "GHOST_ANN",
                "entity_id": HOLLOW_ID,
            }),
        )])
        .await
        .unwrap();

    let mock = Arc::new(MockProvider::default());
    mock.add_response("Gemma3-4b from label admit.").await;
    let engine = QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    );

    let mut req = QueryRequest::new(ASK_SENTENCE).context_only();
    req.mode = Some(QueryMode::Local);
    let resp = engine.query(req).await.expect("query");

    assert!(
        has_gemma(&resp.context.entities),
        "label/seed admit must fill context after hollow ANN; entities={:?}",
        resp.context
            .entities
            .iter()
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    assert!(
        !resp
            .context
            .metadata
            .contains_key(META_POPULAR_NODE_FALLBACK),
        "popular telemetry must not fire when labels/seeds can admit; meta={:?}",
        resp.context.metadata
    );
    assert!(!resp.context.metadata.contains_key(META_POPULAR_NODE_ARM));
    assert!(
        !resp.context.entities.iter().any(|e| e
            .name
            .to_ascii_uppercase()
            .contains("ACTION_FUSION")
            || e.name.to_ascii_uppercase().contains("GHOST")),
        "distractor/hollow must not dominate; entities={:?}",
        resp.context
            .entities
            .iter()
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    std::env::remove_var("EDGEQUAKE_POPULAR_NODE_FALLBACK");
}

#[tokio::test]
async fn global_popular_skipped_when_exact_label_candidates_exist() {
    std::env::set_var("EDGEQUAKE_POPULAR_NODE_FALLBACK", "1");
    let dim = 1536;
    let vector = Arc::new(MemoryVectorStorage::new("global-popular-skip", dim));
    let graph = Arc::new(MemoryGraphStorage::new("global-popular-skip"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();
    seed_gemma_graph(&graph).await;
    seed_popular_hub(&graph).await;

    let mock = Arc::new(MockProvider::default());
    mock.add_response("Gemma3-4b via global admit.").await;
    let engine = QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    );

    let mut req = QueryRequest::new(ASK_SENTENCE).context_only();
    req.mode = Some(QueryMode::Global);
    let resp = engine.query(req).await.expect("query");

    assert!(
        has_gemma(&resp.context.entities),
        "global arm must admit Gemma3-4b before popular; entities={:?}",
        resp.context
            .entities
            .iter()
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    assert!(
        !resp
            .context
            .entities
            .iter()
            .any(|e| { e.name.to_ascii_uppercase().contains("ACTION_FUSION") }),
        "global popular hub must not enter when labels exist"
    );
    assert!(!resp
        .context
        .metadata
        .contains_key(META_POPULAR_NODE_FALLBACK));
    std::env::remove_var("EDGEQUAKE_POPULAR_NODE_FALLBACK");
}

#[tokio::test]
async fn popular_skipped_when_exact_label_candidates_exist() {
    std::env::set_var("EDGEQUAKE_POPULAR_NODE_FALLBACK", "1");
    let dim = 1536;
    let vector = Arc::new(MemoryVectorStorage::new("popular-skip", dim));
    let graph = Arc::new(MemoryGraphStorage::new("popular-skip"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();
    seed_gemma_graph(&graph).await;
    seed_popular_hub(&graph).await;

    let mock = Arc::new(MockProvider::default());
    mock.add_response("Gemma3-4b without popular hubs.").await;
    let engine = QueryEngine::with_mock_keywords(
        QueryEngineConfig::default(),
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    );

    let mut req = QueryRequest::new(ASK_SENTENCE).context_only();
    req.mode = Some(QueryMode::Local);
    let resp = engine.query(req).await.expect("query");

    assert!(
        has_gemma(&resp.context.entities),
        "exact label must admit Gemma3-4b; entities={:?}",
        resp.context
            .entities
            .iter()
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    assert!(
        !resp.context.entities.iter().any(|e| {
            e.name.to_ascii_uppercase().contains("ACTION_FUSION")
                || e.description.to_ascii_lowercase().contains("distractor")
        }),
        "high-degree Action Fusion hub must not enter when labels exist; entities={:?}",
        resp.context
            .entities
            .iter()
            .map(|e| &e.name)
            .collect::<Vec<_>>()
    );
    assert!(
        !resp
            .context
            .metadata
            .contains_key(META_POPULAR_NODE_FALLBACK),
        "popular fallback telemetry must be absent"
    );
    std::env::remove_var("EDGEQUAKE_POPULAR_NODE_FALLBACK");
}
