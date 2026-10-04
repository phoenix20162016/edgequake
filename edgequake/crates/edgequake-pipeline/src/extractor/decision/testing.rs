//! Test doubles for the decision mode (enabled by `test-support` and in unit tests).
//!
//! [`HandlerBackend`] answers System One requests with a function. Tests plug in
//! `edgextract::standin::rule_handler`, a rule-based answerer, so a whole run
//! needs no model and no network.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use edgextract::testing::HandlerFn;
use serde_json::Value;

use super::backend::{BackendDescriptor, BackendHealth, BackendKind, DecisionBackend};
use super::error::DecisionError;

/// A backend that calls a handler in process.
pub struct HandlerBackend {
    handler: HandlerFn,
    model: String,
    calls: AtomicUsize,
}

impl HandlerBackend {
    pub fn new(model: &str, handler: HandlerFn) -> Self {
        Self {
            handler,
            model: model.to_string(),
            calls: AtomicUsize::new(0),
        }
    }

    /// How many requests reached the handler.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl DecisionBackend for HandlerBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            kind: BackendKind::OllamaSystemOne,
            model: self.model.clone(),
            host: "in-process".into(),
        }
    }

    async fn health(&self) -> BackendHealth {
        BackendHealth {
            reachable: true,
            model_present: true,
            decision_capable: true,
            supported: true,
            latency_ms: Some(0),
            cpu_pinned: None,
            server_version: Some("test".into()),
            detail: None,
        }
    }

    async fn systemone(&self, request: &Value) -> Result<Value, DecisionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        (self.handler)(request.clone()).map_err(DecisionError::Contract)
    }
}

/// A backend that wraps another and counts nothing but fails every request.
pub struct DownBackend(pub String);

#[async_trait]
impl DecisionBackend for DownBackend {
    fn descriptor(&self) -> BackendDescriptor {
        BackendDescriptor {
            kind: BackendKind::OllamaSystemOne,
            model: self.0.clone(),
            host: "down:0".into(),
        }
    }

    async fn health(&self) -> BackendHealth {
        BackendHealth::unreachable("connection refused")
    }

    async fn systemone(&self, _request: &Value) -> Result<Value, DecisionError> {
        Err(DecisionError::Unavailable("connection refused".into()))
    }
}

/// A rule-based backend. `known` maps surface names to type ids; the rule
/// engine types a known name with high confidence and any other name with doubt.
pub fn rule_backend(
    ontology: &edgextract::Ontology,
    known: &[(&str, &str)],
) -> Arc<HandlerBackend> {
    let mut with_names = ontology.clone();
    for (name, type_id) in known {
        with_names
            .gazetteer
            .insert((*name).to_string(), (*type_id).to_string());
    }
    Arc::new(HandlerBackend::new(
        "test-model",
        edgextract::standin::rule_handler(&with_names),
    ))
}

/// Names used by the unit tests of this module.
pub const SAMPLE_NAMES: [(&str, &str); 3] = [
    ("Ada Lovelace", "PERSON"),
    ("Acme Inc", "ORGANIZATION"),
    ("Paris", "LOCATION"),
];
