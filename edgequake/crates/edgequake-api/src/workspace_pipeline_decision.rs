//! SPEC-160 — the decision branch of the workspace pipeline factory.
//!
//! The factory asks [`resolve_mode`] first. When the answer is `decision` it calls
//! [`build_decision_pipeline`]. Every failure here is a *mode error*: the caller must
//! fail the document. It must never fall back to the global LLM pipeline (LAW-160-4).

use std::sync::Arc;

use edgequake_core::{resolve_embedding_choice, Tenant, Workspace};
use edgequake_pipeline::extractor::decision::{DecisionError, DecisionRuntime};
use edgequake_pipeline::{
    build_pipeline_with_extractor, extraction_mode_from_metadata, extraction_mode_from_value,
    prompts::EntityExtractionSchema, resolve_extraction_mode_from_env, IngestionPipelineOptions,
    Pipeline, ResolvedExtractionMode,
};
use edgequake_storage::decision::DecisionScope;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::safety_limits::create_safe_embedding_provider;

/// Marks a factory error as a mode error. Lenient callers check it before falling back.
pub const MODE_ERROR_PREFIX: &str = "extraction_mode_error";

/// Build a mode error string: `extraction_mode_error[code]: message [failure_class=…]`.
///
/// The `failure_class` token lets the worker retry only what a retry can fix.
pub fn mode_error(code: &str, message: impl std::fmt::Display, failure_class: &str) -> String {
    format!("{MODE_ERROR_PREFIX}[{code}]: {message} [failure_class={failure_class}]")
}

/// A setting or request that needs a person to fix it. Never retried.
fn misconfigured(code: &str, message: impl std::fmt::Display) -> String {
    mode_error(code, message, "provider_misconfigured")
}

/// True when a factory error must fail the document instead of using a fallback.
pub fn is_mode_error(message: &str) -> bool {
    message.starts_with(MODE_ERROR_PREFIX)
}

fn decision_error(err: DecisionError) -> String {
    mode_error(err.code(), &err, err.failure_class())
}

/// Resolve document > workspace > env > `llm` (LAW-160-2).
pub fn resolve_mode(
    ws: &Workspace,
    options: &IngestionPipelineOptions,
) -> Result<ResolvedExtractionMode, String> {
    let document = options
        .document_extraction
        .as_ref()
        .and_then(extraction_mode_from_value);
    let workspace = extraction_mode_from_metadata(&ws.metadata);
    resolve_extraction_mode_from_env(document.as_deref(), workspace.as_deref())
        .map_err(|e| misconfigured("invalid_extraction_mode", e))
}

/// Build the ingestion pipeline for a workspace whose mode resolved to `decision`.
///
/// Checks the backend before the first chunk (EC-160-16). The chat LLM is never created:
/// no text leaves for a cloud model in this mode (LAW-160-3).
pub async fn build_decision_pipeline(
    runtime: Option<&DecisionRuntime>,
    ws: &Workspace,
    tenant: Option<&Tenant>,
    options: IngestionPipelineOptions,
    cancel: CancellationToken,
) -> Result<Arc<Pipeline>, String> {
    let runtime = runtime.ok_or_else(|| {
        misconfigured(
            "decision_disabled",
            "this task runner has no decision runtime",
        )
    })?;
    let settings = runtime
        .effective_settings(&ws.metadata, options.document_extraction.as_ref())
        .map_err(decision_error)?;
    let prepared = runtime.prepare(settings).map_err(decision_error)?;
    DecisionRuntime::admit(&prepared)
        .await
        .map_err(decision_error)?;

    let emb = resolve_embedding_choice(None, None, None, Some(ws), tenant);
    let embedding = create_safe_embedding_provider(&emb.provider, &emb.model, emb.dimension)
        .map_err(|e| misconfigured("embedding_provider_unavailable", e))?;

    let schema = EntityExtractionSchema::from_workspace_metadata(&ws.metadata);
    let provider_name = prepared.settings.backend.provider_name();
    let model = prepared.settings.model.clone();
    let scope = DecisionScope::new(
        Some(&ws.tenant_id.to_string()),
        &ws.workspace_id.to_string(),
    );
    let extractor = runtime
        .extractor(prepared, &schema, scope, cancel)
        .map_err(decision_error)?;
    for warning in extractor.warnings() {
        info!(workspace_id = %ws.workspace_id, warning = %warning, "Decision ontology reduced");
    }
    info!(
        workspace_id = %ws.workspace_id,
        provider = provider_name,
        model = %model,
        "Resolved decision ingestion pipeline"
    );
    let options = options
        .with_llm_provider(provider_name)
        .with_extraction_mode(edgequake_pipeline::ExtractionMode::Decision);
    Ok(Arc::new(build_pipeline_with_extractor(
        Arc::new(extractor),
        embedding,
        options,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ws(meta: serde_json::Value) -> Workspace {
        let mut w = Workspace::new(uuid::Uuid::new_v4(), "w", "w");
        if let Some(obj) = meta.as_object() {
            for (k, v) in obj {
                w.metadata.insert(k.clone(), v.clone());
            }
        }
        w
    }

    fn opts(doc: serde_json::Value) -> IngestionPipelineOptions {
        IngestionPipelineOptions::from_document_size(10).with_document_extraction_from(&doc)
    }

    // T-160-U68 — document beats workspace; bad word is a mode error.
    #[test]
    fn resolve_layers_and_errors() {
        let w = ws(json!({"extraction_mode": "decision"}));
        let from_ws = resolve_mode(&w, &opts(json!({}))).unwrap();
        assert_eq!(from_ws.mode.as_str(), "decision");
        assert_eq!(from_ws.source.as_str(), "workspace");
        let from_doc = resolve_mode(&w, &opts(json!({"extraction_mode": "llm"}))).unwrap();
        assert_eq!(from_doc.mode.as_str(), "llm");
        assert_eq!(from_doc.source.as_str(), "document");
        let bad = resolve_mode(&w, &opts(json!({"extraction_mode": "tev1"}))).unwrap_err();
        assert!(is_mode_error(&bad));
        assert!(bad.contains("invalid_extraction_mode"));
    }

    // T-160-U69 — no runtime means a coded failure, never a silent LLM run.
    #[tokio::test]
    async fn missing_runtime_is_a_mode_error() {
        let w = ws(json!({}));
        let err =
            build_decision_pipeline(None, &w, None, opts(json!({})), CancellationToken::new())
                .await
                .err()
                .unwrap();
        assert!(is_mode_error(&err) && err.contains("decision_disabled"));
    }

    // T-160-U70 — workspace has not opted in: coded failure.
    #[tokio::test]
    async fn disabled_runtime_is_a_mode_error() {
        let rt = DecisionRuntime::with_settings(
            Ok(edgequake_pipeline::extractor::decision::DecisionSettings {
                gate: edgequake_pipeline::extractor::decision::DecisionGate::Locked,
                ..Default::default()
            }),
            Arc::new(edgequake_storage::decision::MemoryDecisionStore::new()),
        );
        let w = ws(json!({}));
        let err = build_decision_pipeline(
            Some(&rt),
            &w,
            None,
            opts(json!({})),
            CancellationToken::new(),
        )
        .await
        .err()
        .unwrap();
        assert!(err.contains("decision_disabled"), "{err}");
    }

    // T-160-U71 — enabled but the backend is down: coded 'unavailable', no fallback.
    #[tokio::test]
    async fn unreachable_backend_is_a_mode_error() {
        let settings = edgequake_pipeline::extractor::decision::DecisionSettings {
            gate: edgequake_pipeline::extractor::decision::DecisionGate::ForcedOn,
            base_url: "http://127.0.0.1:1".into(),
            ..Default::default()
        };
        let rt = DecisionRuntime::with_settings(
            Ok(settings),
            Arc::new(edgequake_storage::decision::MemoryDecisionStore::new()),
        );
        let w = ws(json!({}));
        let err = build_decision_pipeline(
            Some(&rt),
            &w,
            None,
            opts(json!({})),
            CancellationToken::new(),
        )
        .await
        .err()
        .unwrap();
        assert!(err.contains("decision_backend_unavailable"), "{err}");
        // Retry budget: a down backend may come back; a bad word never will.
        use edgequake_tasks::{classify_ingestion_failure, IngestionFailureClass};
        assert_eq!(
            classify_ingestion_failure(&err),
            IngestionFailureClass::ProviderUnavailable
        );
        let bad = resolve_mode(&ws(json!({})), &opts(json!({"extraction_mode": "x"}))).unwrap_err();
        assert!(
            edgequake_tasks::is_permanent_ingestion_failure(&bad),
            "{bad}"
        );
    }
}
