//! Errors of the decision extraction mode (SPEC-160).
//!
//! Each variant has a stable API `code`. Admission maps the code to a 422 body.
//! No variant means "fall back to the LLM". LAW-160-4 forbids that fallback.

use crate::error::PipelineError;

/// Why a decision run, check, or setting failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecisionError {
    /// The operator locked the mode (`EDGEQUAKE_DECISION_ENABLED=0`).
    #[error("Decision extraction is locked by the operator (EDGEQUAKE_DECISION_ENABLED=0).")]
    Disabled,

    /// The workspace has not turned the mode on (`decision_enabled`).
    #[error("Turn on Decision extraction in workspace settings.")]
    NotActivated,

    /// The backend does not answer.
    #[error("Decision backend is not reachable: {0}")]
    Unavailable(String),

    /// The backend runs, and the model is not there.
    #[error("Decision model '{0}' is not available on the backend. Pull it first.")]
    ModelMissing(String),

    /// The backend cannot answer decision questions (old Ollama, wrong model).
    #[error("Decision backend cannot answer decision questions: {0}")]
    Unsupported(String),

    /// A request took too long.
    #[error("Decision request timed out: {0}")]
    Timeout(String),

    /// The backend answered with a body that breaks the decision contract.
    #[error("Decision backend broke the contract: {0}")]
    Contract(String),

    /// The run was cancelled by the caller.
    #[error("Decision run was cancelled")]
    Cancelled,

    /// A setting has a bad value.
    #[error("Invalid decision setting: {0}")]
    Config(String),

    /// The workspace schema cannot become a decision ontology.
    #[error("Invalid decision ontology: {0}")]
    Ontology(String),
}

impl DecisionError {
    /// Stable code for API bodies and logs.
    pub fn code(&self) -> &'static str {
        match self {
            DecisionError::Disabled => "decision_disabled",
            DecisionError::NotActivated => "decision_not_activated",
            DecisionError::Unavailable(_) => "decision_backend_unavailable",
            DecisionError::ModelMissing(_) => "decision_model_missing",
            DecisionError::Unsupported(_) => "decision_backend_unsupported",
            DecisionError::Timeout(_) => "decision_timeout",
            DecisionError::Contract(_) => "decision_contract_mismatch",
            DecisionError::Cancelled => "decision_cancelled",
            DecisionError::Config(_) => "invalid_decision_setting",
            DecisionError::Ontology(_) => "invalid_decision_ontology",
        }
    }

    /// Ingestion failure class token (`failure_class=` marker, SPEC-045 taxonomy).
    ///
    /// Only an unreachable or slow backend is worth a retry. Everything else needs a
    /// person to change a setting or pull a model, so the worker must not retry it.
    pub fn failure_class(&self) -> &'static str {
        match self {
            DecisionError::Unavailable(_) => "provider_unavailable",
            DecisionError::Timeout(_) => "timeout_phase_extract",
            DecisionError::Cancelled => "cancelled",
            _ => "provider_misconfigured",
        }
    }

    /// True when a retry can help (the backend may come back).
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DecisionError::Unavailable(_) | DecisionError::Timeout(_)
        )
    }
}

impl From<DecisionError> for PipelineError {
    fn from(err: DecisionError) -> Self {
        match err {
            DecisionError::Config(m) | DecisionError::Ontology(m) => PipelineError::ConfigError(m),
            other => PipelineError::ExtractionError(format!(
                "{other} [code={}] [failure_class={}]",
                other.code(),
                other.failure_class()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-160-U36 — codes are stable and retry flags are right.
    #[test]
    fn codes_and_retry_flags() {
        assert_eq!(DecisionError::Disabled.code(), "decision_disabled");
        assert_eq!(DecisionError::NotActivated.code(), "decision_not_activated");
        assert_eq!(
            DecisionError::ModelMissing("m".into()).code(),
            "decision_model_missing"
        );
        assert!(DecisionError::Unavailable("x".into()).is_retryable());
        assert!(DecisionError::Timeout("x".into()).is_retryable());
        assert!(!DecisionError::Contract("x".into()).is_retryable());
        assert!(!DecisionError::Cancelled.is_retryable());
    }

    // T-160-U72 — retry budget is spent only where a retry can help.
    #[test]
    fn failure_class_follows_retry_flag() {
        assert_eq!(
            DecisionError::Unavailable("x".into()).failure_class(),
            "provider_unavailable"
        );
        assert_eq!(
            DecisionError::Timeout("x".into()).failure_class(),
            "timeout_phase_extract"
        );
        for permanent in [
            DecisionError::Disabled,
            DecisionError::NotActivated,
            DecisionError::ModelMissing("m".into()),
            DecisionError::Unsupported("x".into()),
            DecisionError::Contract("x".into()),
        ] {
            assert_eq!(permanent.failure_class(), "provider_misconfigured");
        }
    }

    // T-160-U37 — pipeline errors carry the code and never read as a config fallback.
    #[test]
    fn maps_to_pipeline_error() {
        let e: PipelineError = DecisionError::Unavailable("down".into()).into();
        assert!(
            matches!(&e, PipelineError::ExtractionError(m) if m.contains("decision_backend_unavailable"))
        );
        let c: PipelineError = DecisionError::Config("bad".into()).into();
        assert!(matches!(c, PipelineError::ConfigError(_)));
    }
}
