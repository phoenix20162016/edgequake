//! Decision backend adapters (OCP: add a file, add a backend).

pub mod ollama;

use std::sync::Arc;

pub use ollama::OllamaBackend;

use super::backend::{BackendKind, DecisionBackend};
use super::error::DecisionError;
use super::settings::DecisionSettings;

/// The one place a setting becomes a backend.
pub fn backend_from_settings(
    settings: &DecisionSettings,
) -> Result<Arc<dyn DecisionBackend>, DecisionError> {
    match settings.backend {
        BackendKind::OllamaSystemOne => Ok(Arc::new(
            OllamaBackend::new(
                &settings.base_url,
                &settings.model,
                settings.api_key.clone(),
                settings.timeout_secs,
            )?
            .with_keep_alive(settings.keep_alive.clone()),
        )),
        BackendKind::OpenAiLogprobs => Err(DecisionError::Unsupported(
            "the openai_logprobs backend is not available in this release".into(),
        )),
    }
}
