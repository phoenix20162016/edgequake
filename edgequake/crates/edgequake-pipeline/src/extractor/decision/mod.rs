//! Decision extraction mode (SPEC-160): closed-decision KG extraction.
//!
//! The LLM mode asks a model to write entities. The decision mode never lets a
//! model write a type name. `edgextract` proposes mentions, asks closed
//! questions with legal answers, and gates each answer by its probability.
//! Only ACCEPT rows enter the graph. REVIEW rows are stored for people.
//!
//! ```text
//!   TextChunk ─▶ sentences ─▶ mentions ─▶ typing questions ─▶ prune by ontology
//!                                                                    │
//!        EdgeQuake ExtractionResult ◀─ map ◀─ gate ◀─ relation questions
//!                                                  every question goes through
//!                                                  BridgeTransport: cache, then backend
//! ```
//!
//! | File               | Duty                                           |
//! |--------------------|------------------------------------------------|
//! | `error.rs`         | Typed errors with stable codes                 |
//! | `backend.rs`       | `DecisionBackend` seam and health types        |
//! | `backends/`        | Adapters (Ollama System One)                   |
//! | `bridge.rs`        | Sync transport over async backend and cache    |
//! | `gate.rs`          | Strict, balanced, recall presets               |
//! | `ontology.rs`      | Workspace schema to ontology                   |
//! | `map.rs`           | `edgextract` result to EdgeQuake result        |
//! | `outcome.rs`       | Per-document counts and review rows            |
//! | `settings.rs`      | `EDGEQUAKE_DECISION_*` and workspace overrides |

pub mod backend;
pub mod backends;
pub mod bridge;
pub mod error;
pub mod gate;
pub mod map;
pub mod ontology;
pub mod outcome;
pub mod runtime;
pub mod settings;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

use std::sync::Arc;

use async_trait::async_trait;
use edgextract::candidates::discovering_proposers;
use edgextract::markdown::split_sentences;
use edgextract::{Extractor, Ontology, SystemOneClient, DECISION_CONTRACT};
use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;

use crate::chunker::TextChunk;
use crate::error::{PipelineError, Result};
use crate::extractor::{EntityExtractor, ExtractionResult};

pub use backend::{BackendDescriptor, BackendHealth, BackendKind, DecisionBackend, ListedModel};
pub use bridge::{BridgeCounters, CacheLink};
pub use error::DecisionError;
pub use gate::GatePreset;
pub use map::{rejected_from_metadata, review_rows_from_metadata};
pub use ontology::{ontology_from_schema, OntologyBuild};
pub use outcome::{
    outcome_from_extractions, DecisionOutcome, DecisionStats, META_DOCUMENT_DECISION_STATS,
};
pub use runtime::{
    BackendStatus, DecisionLimits, DecisionModels, DecisionProviderView, DecisionRuntime,
    DecisionStatus, PreparedBackend,
};
pub use settings::{
    DecisionActivation, DecisionGate, DecisionOverrides, DecisionSettings, META_DECISION_ENABLED,
};

use bridge::{BridgeTransport, SharedTransport};
use map::{to_eq_result, MapContext};

/// The prompt cap of the Tev1 host. `edgextract` splits requests below it.
const MAX_PROMPT_TOKENS: usize = 2000;

/// Per-run options.
#[derive(Clone)]
pub struct DecisionRunOptions {
    pub gate: GatePreset,
    pub pack_size: usize,
    pub cache: Option<CacheLink>,
    pub cancel: CancellationToken,
}

impl Default for DecisionRunOptions {
    fn default() -> Self {
        Self {
            gate: GatePreset::default(),
            pack_size: settings::DEFAULT_PACK_SIZE,
            cache: None,
            cancel: CancellationToken::new(),
        }
    }
}

/// [`EntityExtractor`] backed by a decision backend.
pub struct DecisionExtractor {
    backend: Arc<dyn DecisionBackend>,
    ontology: Arc<Ontology>,
    warnings: Arc<Vec<String>>,
    options: DecisionRunOptions,
    model: String,
}

impl DecisionExtractor {
    pub fn new(
        backend: Arc<dyn DecisionBackend>,
        build: OntologyBuild,
        options: DecisionRunOptions,
    ) -> Self {
        let model = backend.descriptor().model;
        Self {
            backend,
            ontology: Arc::new(build.ontology),
            warnings: Arc::new(build.warnings),
            options,
            model,
        }
    }

    /// Warnings from the ontology build (reductions made to fit the limits).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    fn run_blocking(&self, content: String, chunk_id: String) -> Result<ExtractionResult> {
        let transport = Arc::new(BridgeTransport::new(
            Arc::clone(&self.backend),
            Handle::current(),
            self.options.cache.clone(),
            self.options.cancel.clone(),
        ));
        let counters = transport.counters();
        let client = SystemOneClient::with_transport(
            self.model.clone(),
            Box::new(SharedTransport(Arc::clone(&transport))),
        );
        let extractor = Extractor::new((*self.ontology).clone(), client)
            .with_gate(self.options.gate.to_config())
            .with_max_questions(self.options.pack_size)
            .with_max_prompt_tokens(MAX_PROMPT_TOKENS)
            .with_proposers(discovering_proposers());
        let sentences = split_sentences(&content, &chunk_id);
        let started = web_time::Instant::now();
        let t0 = unix_now();
        let result = extractor.extract_sentences(&sentences, &chunk_id, &chunk_id, started, t0);
        match result {
            Ok(x) => {
                let ctx = MapContext {
                    chunk_id: &chunk_id,
                    document_id: None,
                    model: &self.model,
                    contract: DECISION_CONTRACT,
                    sentences: &sentences,
                    warnings: &self.warnings,
                };
                let mut mapped = to_eq_result(x, &ctx);
                record_counters(&mut mapped, &counters);
                Ok(mapped)
            }
            Err(err) => Err(PipelineError::from(pick_error(
                &err.to_string(),
                transport.take_errors(),
            ))),
        }
    }
}

fn unix_now() -> f64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Add the bridge's own call counts. They are the truth for cost and cache views.
fn record_counters(result: &mut ExtractionResult, c: &BridgeCounters) {
    if let Some(stats) = result
        .metadata
        .get_mut(map::META_DECISION_STATS)
        .and_then(serde_json::Value::as_object_mut)
    {
        stats.insert("backend_calls".into(), c.backend_calls().into());
        stats.insert("bridge_cache_hits".into(), c.cache_hits().into());
    }
}

/// Choose the typed error that explains a failed run.
fn pick_error(message: &str, recorded: Vec<DecisionError>) -> DecisionError {
    if recorded
        .iter()
        .any(|e| matches!(e, DecisionError::Cancelled))
    {
        return DecisionError::Cancelled;
    }
    recorded
        .into_iter()
        .rev()
        .find(|e| message.contains(&e.to_string()))
        .unwrap_or_else(|| DecisionError::Contract(message.to_string()))
}

#[async_trait]
impl EntityExtractor for DecisionExtractor {
    async fn extract(&self, chunk: &TextChunk) -> Result<ExtractionResult> {
        if chunk.content.trim().is_empty() {
            return Ok(ExtractionResult::new(&chunk.id));
        }
        if self.options.cancel.is_cancelled() {
            return Err(DecisionError::Cancelled.into());
        }
        let (content, chunk_id) = (chunk.content.clone(), chunk.id.clone());
        let this = DecisionExtractor {
            backend: Arc::clone(&self.backend),
            ontology: Arc::clone(&self.ontology),
            warnings: Arc::clone(&self.warnings),
            options: self.options.clone(),
            model: self.model.clone(),
        };
        tokio::task::spawn_blocking(move || this.run_blocking(content, chunk_id))
            .await
            .map_err(|e| PipelineError::ExtractionError(format!("decision worker failed: {e}")))?
    }

    fn name(&self) -> &str {
        "decision"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn provider_name(&self) -> &str {
        self.backend.descriptor().kind.provider_name()
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{rule_backend, DownBackend, HandlerBackend, SAMPLE_NAMES};
    use super::*;
    use crate::prompts::EntityExtractionSchema;
    use edgequake_storage::decision::{DecisionScope, DecisionStore, MemoryDecisionStore};

    fn chunk(text: &str) -> TextChunk {
        let mut c = TextChunk::new("doc-chunk-0", text, 0, 0, text.len());
        c.id = "doc-chunk-0".into();
        c
    }

    fn schema() -> EntityExtractionSchema {
        EntityExtractionSchema {
            types: vec!["PERSON".into(), "ORGANIZATION".into(), "LOCATION".into()],
            strict: true,
            relation_types: Vec::new(),
            relation_strict: true,
            relation_edges: vec![
                crate::prompts::RelationEdge {
                    source: "PERSON".into(),
                    relation: "WORKS_AT".into(),
                    target: "ORGANIZATION".into(),
                },
                crate::prompts::RelationEdge {
                    source: "ORGANIZATION".into(),
                    relation: "LOCATED_IN".into(),
                    target: "LOCATION".into(),
                },
            ],
        }
    }

    fn extractor(backend: Arc<dyn DecisionBackend>, opts: DecisionRunOptions) -> DecisionExtractor {
        DecisionExtractor::new(backend, ontology_from_schema(&schema()).unwrap(), opts)
    }

    const TEXT: &str = "Ada Lovelace works at Acme Inc. Acme Inc is located in Paris.";

    // T-160-I01 — a chunk becomes entities and relations with chunk lineage.
    #[tokio::test(flavor = "multi_thread")]
    async fn extracts_with_lineage() {
        let build = ontology_from_schema(&schema()).unwrap();
        let backend = rule_backend(&build.ontology, &SAMPLE_NAMES);
        let ex = DecisionExtractor::new(backend.clone(), build, DecisionRunOptions::default());
        let out = ex.extract(&chunk(TEXT)).await.unwrap();
        assert!(
            out.entities.iter().any(|e| e.name == "ADA_LOVELACE"),
            "{:?}",
            out.entities.iter().map(|e| &e.name).collect::<Vec<_>>()
        );
        assert!(!out.relationships.is_empty());
        assert!(out
            .entities
            .iter()
            .all(|e| e.source_chunk_ids == vec!["doc-chunk-0".to_string()]));
        assert!(out
            .relationships
            .iter()
            .all(|r| r.source_chunk_ids == vec!["doc-chunk-0".to_string()]));
        assert_eq!(
            out.metadata["extraction_mode"],
            serde_json::json!("decision")
        );
        assert!(backend.calls() > 0);
        assert_eq!((ex.name(), ex.model_name()), ("decision", "test-model"));
        assert!(ex.provider_name().starts_with("decision:"));
    }

    // T-160-I02 — an empty chunk makes no backend call (EC-160-12).
    #[tokio::test(flavor = "multi_thread")]
    async fn empty_chunk_costs_nothing() {
        let build = ontology_from_schema(&schema()).unwrap();
        let backend = rule_backend(&build.ontology, &SAMPLE_NAMES);
        let ex = DecisionExtractor::new(backend.clone(), build, DecisionRunOptions::default());
        let out = ex.extract(&chunk("  \n ")).await.unwrap();
        assert!(out.entities.is_empty() && out.relationships.is_empty());
        assert_eq!(backend.calls(), 0);
    }

    // T-160-I03 — a down backend fails the chunk with a typed code; no LLM fallback (LAW-160-4).
    #[tokio::test(flavor = "multi_thread")]
    async fn down_backend_fails_typed() {
        let ex = extractor(
            Arc::new(DownBackend("m".into())),
            DecisionRunOptions::default(),
        );
        let err = ex.extract(&chunk(TEXT)).await.unwrap_err();
        let text = err.to_string();
        assert!(text.contains("decision_backend_unavailable"), "{text}");
    }

    // T-160-I04 — the second run is served from the cache: zero backend calls (EC-160-31).
    #[tokio::test(flavor = "multi_thread")]
    async fn rerun_hits_cache() {
        let build = ontology_from_schema(&schema()).unwrap();
        let backend = rule_backend(&build.ontology, &SAMPLE_NAMES);
        let store: Arc<dyn DecisionStore> = Arc::new(MemoryDecisionStore::new());
        let opts = DecisionRunOptions {
            cache: Some(CacheLink {
                store: Arc::clone(&store),
                scope: DecisionScope::new(None, "ws"),
            }),
            ..Default::default()
        };
        let ex = DecisionExtractor::new(backend.clone(), build, opts);
        let first = ex.extract(&chunk(TEXT)).await.unwrap();
        let calls = backend.calls();
        assert!(calls > 0);
        let second = ex.extract(&chunk(TEXT)).await.unwrap();
        assert_eq!(backend.calls(), calls, "no new backend call");
        assert_eq!(first.entities.len(), second.entities.len());
        assert_eq!(
            second.metadata["decision"]["backend_calls"],
            serde_json::json!(0)
        );
    }

    // T-160-I05 — a cancelled token stops the run with the Cancelled code.
    #[tokio::test(flavor = "multi_thread")]
    async fn cancelled_run_stops() {
        let build = ontology_from_schema(&schema()).unwrap();
        let backend = Arc::new(HandlerBackend::new(
            "m",
            edgextract::standin::rule_handler(&build.ontology),
        ));
        let opts = DecisionRunOptions::default();
        opts.cancel.cancel();
        let ex = DecisionExtractor::new(backend.clone(), build, opts);
        let err = ex.extract(&chunk(TEXT)).await.unwrap_err();
        assert!(err.to_string().contains("decision_cancelled"), "{err}");
        assert_eq!(backend.calls(), 0);
    }

    // T-160-U30 — pick_error prefers Cancelled, then the message match, then Contract.
    #[test]
    fn pick_error_rules() {
        let un = DecisionError::Unavailable("refused".into());
        assert_eq!(
            pick_error("x", vec![un.clone(), DecisionError::Cancelled]),
            DecisionError::Cancelled
        );
        assert_eq!(pick_error(&format!("boom {un}"), vec![un.clone()]), un);
        assert!(matches!(
            pick_error("other", vec![un]),
            DecisionError::Contract(_)
        ));
    }
}
