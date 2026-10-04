//! The backend seam (DIP): anything that answers System One requests.
//!
//! WHY the System One wire format is the lingua franca: `edgextract` builds
//! System One requests and reads System One answers. A backend either speaks it
//! (Ollama) or translates to it (`openai_logprobs`). The extractor never sees
//! HTTP (SRP, ISP).

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

use super::error::DecisionError;

/// Which adapter serves the questions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    /// Ollama `POST /v1/systemone`.
    OllamaSystemOne,
    /// Chat completions plus first-token logprobs (`llama-server`, vLLM, Together).
    OpenAiLogprobs,
}

impl BackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BackendKind::OllamaSystemOne => "ollama_system_one",
            BackendKind::OpenAiLogprobs => "openai_logprobs",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ollama_system_one" | "ollama" => Some(BackendKind::OllamaSystemOne),
            "openai_logprobs" | "llama_server" => Some(BackendKind::OpenAiLogprobs),
            _ => None,
        }
    }

    /// Provider name for fairness and concurrency clamps (`decision:` prefix).
    pub fn provider_name(self) -> &'static str {
        match self {
            BackendKind::OllamaSystemOne => "decision:ollama",
            BackendKind::OpenAiLogprobs => "decision:openai_logprobs",
        }
    }
}

/// Safe-to-show facts about a backend. Never holds a secret or a full URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BackendDescriptor {
    pub kind: BackendKind,
    pub model: String,
    /// `host:port` only (EC-160-35).
    pub host: String,
}

/// Result of a health probe. It never errors: the state is the answer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BackendHealth {
    pub reachable: bool,
    pub model_present: bool,
    pub decision_capable: bool,
    /// False when the server is too old for the protocol.
    pub supported: bool,
    pub latency_ms: Option<u64>,
    /// `Some(true)` only when the backend reports CPU pinning.
    pub cpu_pinned: Option<bool>,
    pub server_version: Option<String>,
    pub detail: Option<String>,
}

impl BackendHealth {
    /// A backend that does not answer.
    pub fn unreachable(detail: impl Into<String>) -> Self {
        Self {
            reachable: false,
            model_present: false,
            decision_capable: false,
            supported: true,
            latency_ms: None,
            cpu_pinned: None,
            server_version: None,
            detail: Some(detail.into()),
        }
    }

    /// True when a run can start.
    pub fn ready(&self) -> bool {
        self.reachable && self.supported && self.model_present && self.decision_capable
    }

    /// The first reason the backend is not ready, as a typed error.
    pub fn first_problem(&self, model: &str) -> Option<DecisionError> {
        let detail = self.detail.clone().unwrap_or_default();
        if !self.reachable {
            return Some(DecisionError::Unavailable(detail));
        }
        if !self.supported {
            return Some(DecisionError::Unsupported(detail));
        }
        if !self.model_present {
            return Some(DecisionError::ModelMissing(model.to_string()));
        }
        if !self.decision_capable {
            return Some(DecisionError::Unsupported(format!(
                "model '{model}' cannot answer decision questions"
            )));
        }
        None
    }
}

/// A model the backend can name. Used by `GET /decision/models`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListedModel {
    pub name: String,
    /// `None` when the capability probe did not finish in time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_capable: Option<bool>,
}

/// Answers System One requests.
#[async_trait]
pub trait DecisionBackend: Send + Sync {
    /// Safe description for logs and the status endpoint.
    fn descriptor(&self) -> BackendDescriptor;

    /// Probe the backend. Must finish within a few seconds.
    async fn health(&self) -> BackendHealth;

    /// Send one System One request body. Return the System One response body.
    async fn systemone(&self, request: &Value) -> Result<Value, DecisionError>;

    /// Names of models on this host. Default: not supported.
    async fn list_models(&self) -> Result<Vec<ListedModel>, DecisionError> {
        Err(DecisionError::Unsupported(
            "this backend cannot list models".into(),
        ))
    }
}

/// Reduce a base URL to `host:port` and drop userinfo, path, and query.
pub fn redact_host(base_url: &str) -> String {
    let no_scheme = base_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(base_url);
    let authority = no_scheme.split(['/', '?', '#']).next().unwrap_or("");
    authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-160-U34 — a URL with credentials shows host only.
    #[test]
    fn redact_host_drops_secrets() {
        assert_eq!(
            redact_host("https://user:s3cret@gpu.example.com:8443/v1/x?key=abc"),
            "gpu.example.com:8443"
        );
        assert_eq!(redact_host("http://localhost:11434"), "localhost:11434");
        assert_eq!(redact_host("localhost:11434/"), "localhost:11434");
    }

    // T-160-U38 — the first problem is reported in a fixed order.
    #[test]
    fn health_first_problem_order() {
        let mut h = BackendHealth::unreachable("refused");
        assert!(matches!(
            h.first_problem("m"),
            Some(DecisionError::Unavailable(_))
        ));
        h.reachable = true;
        h.supported = false;
        assert!(matches!(
            h.first_problem("m"),
            Some(DecisionError::Unsupported(_))
        ));
        h.supported = true;
        assert!(matches!(
            h.first_problem("m"),
            Some(DecisionError::ModelMissing(_))
        ));
        h.model_present = true;
        assert!(matches!(
            h.first_problem("m"),
            Some(DecisionError::Unsupported(_))
        ));
        h.decision_capable = true;
        assert!(h.ready());
        assert!(h.first_problem("m").is_none());
    }

    #[test]
    fn backend_kind_words() {
        assert_eq!(
            BackendKind::parse("OLLAMA"),
            Some(BackendKind::OllamaSystemOne)
        );
        assert_eq!(
            BackendKind::parse("openai_logprobs"),
            Some(BackendKind::OpenAiLogprobs)
        );
        assert_eq!(BackendKind::parse("x"), None);
        assert!(BackendKind::OllamaSystemOne
            .provider_name()
            .starts_with("decision:"));
    }
}
