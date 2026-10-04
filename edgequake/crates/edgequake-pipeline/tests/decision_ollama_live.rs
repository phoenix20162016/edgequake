//! SPEC-160 L01/L02: the decision mode against a real Ollama with Tev1.
//!
//! Skipped unless `EDGEQUAKE_DECISION_LIVE=1`. Host and model come from
//! `EDGEQUAKE_DECISION_BASE_URL` and `EDGEQUAKE_DECISION_MODEL`.

use std::sync::Arc;

use edgequake_pipeline::chunker::TextChunk;
use edgequake_pipeline::extractor::decision::backends::OllamaBackend;
use edgequake_pipeline::extractor::decision::{
    ontology_from_schema, DecisionBackend, DecisionExtractor, DecisionRunOptions, DecisionSettings,
};
use edgequake_pipeline::extractor::EntityExtractor;
use edgequake_pipeline::prompts::{EntityExtractionSchema, RelationEdge};

fn live_settings() -> Option<DecisionSettings> {
    if std::env::var("EDGEQUAKE_DECISION_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipped: set EDGEQUAKE_DECISION_LIVE=1");
        return None;
    }
    Some(DecisionSettings::from_env().expect("valid EDGEQUAKE_DECISION_* settings"))
}

fn backend(s: &DecisionSettings) -> OllamaBackend {
    OllamaBackend::new(&s.base_url, &s.model, s.api_key.clone(), s.timeout_secs).unwrap()
}

fn schema() -> EntityExtractionSchema {
    EntityExtractionSchema {
        types: vec!["PERSON".into(), "ORGANIZATION".into(), "LOCATION".into()],
        strict: true,
        relation_types: Vec::new(),
        relation_strict: true,
        relation_edges: vec![
            RelationEdge {
                source: "PERSON".into(),
                relation: "WORKS_AT".into(),
                target: "ORGANIZATION".into(),
            },
            RelationEdge {
                source: "ORGANIZATION".into(),
                relation: "LOCATED_IN".into(),
                target: "LOCATION".into(),
            },
        ],
    }
}

// T-160-L01 — the real server is ready for the configured model.
#[tokio::test(flavor = "multi_thread")]
async fn live_health_is_ready() {
    let Some(s) = live_settings() else { return };
    let h = backend(&s).health().await;
    assert!(h.ready(), "{h:?}");
}

// T-160-L02 — a real extraction yields the expected people, places and a relation.
#[tokio::test(flavor = "multi_thread")]
async fn live_extraction_finds_the_obvious() {
    let Some(s) = live_settings() else { return };
    let text = "Sarah Chen works at Northwind in Paris. Northwind is located in France.";
    let ex = DecisionExtractor::new(
        Arc::new(backend(&s)),
        ontology_from_schema(&schema()).unwrap(),
        DecisionRunOptions {
            pack_size: s.pack_size,
            gate: s.gate_preset,
            ..Default::default()
        },
    );
    let chunk = TextChunk::new("live-chunk-0", text, 0, 0, text.len());
    let out = ex.extract(&chunk).await.unwrap();
    let names: Vec<_> = out
        .entities
        .iter()
        .map(|e| (e.name.as_str(), e.entity_type.as_str()))
        .collect();
    eprintln!(
        "entities: {names:?}\nrelations: {:?}",
        out.relationships
            .iter()
            .map(|r| (&r.source, &r.relation_type, &r.target))
            .collect::<Vec<_>>()
    );
    eprintln!("review: {}", out.metadata["decision_review"]);
    eprintln!("rejected: {}", out.metadata["decision_rejected"]);
    eprintln!("stats: {}", out.metadata["decision"]["stats"]);
    // A 0.8B model may defer a person to review (rotation check). Nothing may vanish.
    let review = edgequake_pipeline::extractor::decision::review_rows_from_metadata(&out.metadata);
    let seen_sarah = names.contains(&("SARAH_CHEN", "PERSON"))
        || review.iter().any(|r| r.subject == "Sarah Chen");
    assert!(
        seen_sarah,
        "Sarah Chen is neither accepted nor in review: {names:?} {review:?}"
    );
    assert!(
        names
            .iter()
            .any(|(n, t)| *n == "NORTHWIND" && *t == "ORGANIZATION"),
        "{names:?}"
    );
    assert!(
        !out.relationships.is_empty(),
        "at least one relation is accepted"
    );
}
