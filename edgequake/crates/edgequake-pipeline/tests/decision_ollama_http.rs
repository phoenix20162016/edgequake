//! SPEC-160 W2: the Ollama backend and the extractor over real HTTP.
//!
//! A fake Ollama (axum) stands in for the server. The wire format is the one
//! measured on Ollama 0.35.1.

mod support {
    pub mod fake_ollama;
}

use std::sync::Arc;

use axum::http::StatusCode;
use edgequake_pipeline::chunker::TextChunk;
use edgequake_pipeline::extractor::decision::backends::OllamaBackend;
use edgequake_pipeline::extractor::decision::{
    ontology_from_schema, DecisionBackend, DecisionError, DecisionExtractor, DecisionRunOptions,
};
use edgequake_pipeline::extractor::EntityExtractor;
use edgequake_pipeline::prompts::{EntityExtractionSchema, RelationEdge};
use serde_json::json;
use support::fake_ollama::{json_reply, FakeOllama, Handler};

fn schema() -> EntityExtractionSchema {
    EntityExtractionSchema {
        types: vec!["PERSON".into(), "ORGANIZATION".into(), "LOCATION".into()],
        strict: true,
        relation_types: Vec::new(),
        relation_strict: true,
        relation_edges: vec![RelationEdge {
            source: "PERSON".into(),
            relation: "WORKS_AT".into(),
            target: "ORGANIZATION".into(),
        }],
    }
}

fn rule_reply() -> Handler {
    let mut build = ontology_from_schema(&schema()).unwrap().ontology;
    build
        .gazetteer
        .insert("Ada Lovelace".into(), "PERSON".into());
    build
        .gazetteer
        .insert("Acme Inc".into(), "ORGANIZATION".into());
    json_reply(edgextract::standin::rule_handler(&build))
}

fn backend(url: &str) -> OllamaBackend {
    OllamaBackend::new(url, "tev1:0.8b", Some("k3y".into()), 10).unwrap()
}

fn chunk(text: &str) -> TextChunk {
    TextChunk::new("d-chunk-0", text, 0, 0, text.len())
}

// T-160-I06 — a healthy server reports ready and the key is sent as a bearer token.
#[tokio::test(flavor = "multi_thread")]
async fn health_ready_and_auth_header() {
    let fake = FakeOllama::healthy(rule_reply());
    let seen = fake.seen_auth.clone();
    let url = fake.serve().await;
    let h = backend(&url).health().await;
    assert!(h.ready(), "{h:?}");
    assert_eq!(h.server_version.as_deref(), Some("0.35.1"));
    assert!(h.latency_ms.is_some());
    assert_eq!(seen.lock().unwrap()[0].as_deref(), Some("Bearer k3y"));
}

// T-160-I07 — health reports each failure state without an error.
#[tokio::test(flavor = "multi_thread")]
async fn health_failure_states() {
    // Old Ollama.
    let fake = FakeOllama::healthy(rule_reply());
    *fake.version.lock().unwrap() = (StatusCode::OK, r#"{"version":"0.34.9"}"#.into());
    let h = backend(&fake.serve().await).health().await;
    assert!(h.reachable && !h.supported && !h.ready());
    assert!(matches!(
        h.first_problem("tev1:0.8b"),
        Some(DecisionError::Unsupported(_))
    ));

    // Model not pulled.
    let fake = FakeOllama::healthy(rule_reply());
    *fake.show.lock().unwrap() = (
        StatusCode::NOT_FOUND,
        r#"{"error":"model not found"}"#.into(),
    );
    let h = backend(&fake.serve().await).health().await;
    assert!(h.supported && !h.model_present);
    assert!(matches!(
        h.first_problem("tev1:0.8b"),
        Some(DecisionError::ModelMissing(_))
    ));

    // Model without the decision skill.
    let fake = FakeOllama::healthy(rule_reply());
    *fake.show.lock().unwrap() = (StatusCode::OK, r#"{"capabilities":["completion"]}"#.into());
    let h = backend(&fake.serve().await).health().await;
    assert!(h.model_present && !h.decision_capable);

    // Nothing listens.
    let h = backend("http://127.0.0.1:1").health().await;
    assert!(!h.reachable);
    assert!(matches!(
        h.first_problem("m"),
        Some(DecisionError::Unavailable(_))
    ));
}

// T-160-I08 — a full extraction runs over HTTP and sends the key on every call.
#[tokio::test(flavor = "multi_thread")]
async fn extraction_over_http() {
    let fake = FakeOllama::healthy(rule_reply());
    let (calls, seen) = (fake.systemone_calls.clone(), fake.seen_auth.clone());
    let url = fake.serve().await;
    let build = ontology_from_schema(&schema()).unwrap();
    let ex = DecisionExtractor::new(
        Arc::new(backend(&url)),
        build,
        DecisionRunOptions::default(),
    );
    let out = ex
        .extract(&chunk("Ada Lovelace works at Acme Inc."))
        .await
        .unwrap();
    assert!(out.entities.iter().any(|e| e.name == "ADA_LOVELACE"));
    assert!(
        out.relationships
            .iter()
            .any(|r| r.relation_type == "WORKS_AT"),
        "{:?}",
        out.relationships
    );
    assert!(!calls.lock().unwrap().is_empty());
    assert!(seen
        .lock()
        .unwrap()
        .iter()
        .all(|a| a.as_deref() == Some("Bearer k3y")));
    assert_eq!(calls.lock().unwrap()[0]["model"], json!("tev1:0.8b"));
}

// T-160-I09 — HTTP 404 "not found, try pulling" gives ModelMissing with its code (EC-160-04).
#[tokio::test(flavor = "multi_thread")]
async fn model_missing_surfaces_code() {
    let fake = FakeOllama::healthy(Arc::new(|_| {
        (
            StatusCode::NOT_FOUND,
            r#"{"error":"model \"tev1:0.8b\" not found, try pulling it first"}"#.into(),
        )
    }));
    let url = fake.serve().await;
    let ex = DecisionExtractor::new(
        Arc::new(backend(&url)),
        ontology_from_schema(&schema()).unwrap(),
        DecisionRunOptions::default(),
    );
    let err = ex
        .extract(&chunk("Ada Lovelace works at Acme Inc."))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("decision_model_missing"), "{err}");
}

// T-160-I10 — a prompt-too-long 400 is split by the pack logic and the run succeeds (EC-160-17).
#[tokio::test(flavor = "multi_thread")]
async fn oversize_prompt_is_split() {
    let inner = rule_reply();
    let fake = FakeOllama::healthy(Arc::new(move |body| {
        let questions = body["questions"].as_object().map(|q| q.len()).unwrap_or(0);
        if questions > 1 {
            return (
                StatusCode::BAD_REQUEST,
                "prompt 0 has 2100 tokens; expected 1-2050 (input is never truncated)".into(),
            );
        }
        inner(body)
    }));
    let calls = fake.systemone_calls.clone();
    let url = fake.serve().await;
    let ex = DecisionExtractor::new(
        Arc::new(backend(&url)),
        ontology_from_schema(&schema()).unwrap(),
        DecisionRunOptions::default(),
    );
    let out = ex
        .extract(&chunk("Ada Lovelace works at Acme Inc."))
        .await
        .unwrap();
    assert!(!out.entities.is_empty());
    assert!(calls
        .lock()
        .unwrap()
        .iter()
        .any(|c| c["questions"].as_object().unwrap().len() == 1));
}

// T-160-I11 — 5xx and garbage bodies fail with typed errors.
#[tokio::test(flavor = "multi_thread")]
async fn server_faults_are_typed() {
    let fake = FakeOllama::healthy(Arc::new(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "boom".into())
    }));
    let b = backend(&fake.serve().await);
    let err = b
        .systemone(&json!({"model":"m","state":"s","questions":{}}))
        .await
        .unwrap_err();
    assert!(err.is_retryable(), "{err}");

    let fake = FakeOllama::healthy(Arc::new(|_| (StatusCode::OK, "not json".into())));
    let b = backend(&fake.serve().await);
    let err = b.systemone(&json!({})).await.unwrap_err();
    assert!(matches!(err, DecisionError::Contract(_)));

    let fake = FakeOllama::healthy(Arc::new(|_| (StatusCode::OK, "[1,2]".into())));
    let b = backend(&fake.serve().await);
    assert!(matches!(
        b.systemone(&json!({})).await.unwrap_err(),
        DecisionError::Contract(_)
    ));
}

// T-160-I12 — a slow server times out with a retryable error (EC-160-07).
#[tokio::test(flavor = "multi_thread")]
async fn slow_server_times_out() {
    let fake = FakeOllama::healthy(Arc::new(|_| {
        std::thread::sleep(std::time::Duration::from_millis(2500));
        (StatusCode::OK, "{}".into())
    }));
    let b = OllamaBackend::new(&fake.serve().await, "m", None, 1).unwrap();
    let err = b.systemone(&json!({})).await.unwrap_err();
    assert!(matches!(err, DecisionError::Timeout(_)), "{err}");
    assert!(err.is_retryable());
}

// T-160-I13 — a bad answer (probabilities that do not sum to 1) fails the chunk, no guess.
#[tokio::test(flavor = "multi_thread")]
async fn contract_breach_fails_chunk() {
    let fake = FakeOllama::healthy(Arc::new(|body| {
        let ids: Vec<String> = body["questions"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        let answers: serde_json::Map<String, serde_json::Value> = ids
            .into_iter()
            .map(|id| (id, json!({"type":"noul","noul":3.0})))
            .collect();
        (
            StatusCode::OK,
            json!({"model":"m","answers":answers}).to_string(),
        )
    }));
    let url = fake.serve().await;
    let ex = DecisionExtractor::new(
        Arc::new(backend(&url)),
        ontology_from_schema(&schema()).unwrap(),
        DecisionRunOptions::default(),
    );
    let err = ex
        .extract(&chunk("Ada Lovelace works at Acme Inc."))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("decision_contract_mismatch"),
        "{err}"
    );
}
