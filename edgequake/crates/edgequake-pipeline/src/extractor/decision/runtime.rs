//! Server-side decision runtime (SPEC-160): settings, store, and the three steps a
//! caller needs: resolve the effective settings, check the backend, build the extractor.
//!
//! The API owns one [`DecisionRuntime`]. Upload admission, the pipeline factory, and
//! the status endpoint all go through it, so there is one place that knows how a
//! setting becomes a backend (SRP, DRY).

use std::collections::HashMap;
use std::sync::Arc;

use edgequake_storage::decision::{DecisionScope, DecisionStore};
use serde::Serialize;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::backend::{BackendDescriptor, BackendHealth, BackendKind, DecisionBackend, ListedModel};
use super::backends::backend_from_settings;
use super::error::DecisionError;
use super::gate::GatePreset;
use super::ontology::ontology_from_schema;
use super::settings::{
    check_model_name, workspace_decision_enabled, DecisionActivation, DecisionGate,
    DecisionOverrides, DecisionSettings, DEFAULT_PACK_SIZE, MAX_PACK_SIZE, MIN_PACK_SIZE,
};
use super::{CacheLink, DecisionExtractor, DecisionRunOptions};
use crate::prompts::EntityExtractionSchema;

/// Server settings parsed once at boot, plus the shared decision store.
#[derive(Clone)]
pub struct DecisionRuntime {
    settings: Arc<Result<DecisionSettings, DecisionError>>,
    store: Arc<dyn DecisionStore>,
}

/// What the status endpoint reports. Safe to show: host only, no secrets (EC-160-35).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionStatus {
    /// True when a run is allowed for this workspace (gate is active or forced).
    pub enabled: bool,
    /// `locked` | `inactive` | `active` | `forced`.
    pub activation: DecisionActivation,
    /// Stable code of a bad `EDGEQUAKE_DECISION_*` value, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings_error: Option<String>,
    /// The configured provider (present unless the operator locked the mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<DecisionProviderView>,
    /// Present when the settings are valid and the operator did not lock the mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<BackendStatus>,
    pub limits: DecisionLimits,
    pub license_notice: &'static str,
}

/// The decision provider this server uses (Ollama on a known host).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionProviderView {
    pub kind: BackendKind,
    pub label: &'static str,
    pub base_url_host: String,
}

/// One backend, probed now.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BackendStatus {
    pub kind: BackendKind,
    pub base_url_host: String,
    pub model: String,
    pub contract: &'static str,
    #[serde(flatten)]
    pub health: BackendHealth,
}

/// Limits the UI shows next to the controls.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionLimits {
    pub pack_size_default: usize,
    pub pack_size_min: usize,
    pub pack_size_max: usize,
    pub gate_presets: Vec<&'static str>,
}

impl DecisionLimits {
    fn current() -> Self {
        Self {
            pack_size_default: DEFAULT_PACK_SIZE,
            pack_size_min: MIN_PACK_SIZE,
            pack_size_max: MAX_PACK_SIZE,
            gate_presets: GatePreset::ALL.iter().map(|g| g.as_str()).collect(),
        }
    }
}

/// The status endpoint never waits longer than this for a backend (spec 08).
const STATUS_PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

const LICENSE_NOTICE: &str = "Tev1 weights license is not final. See the model card.";

/// A backend and the exact settings it was built from.
pub struct PreparedBackend {
    pub settings: DecisionSettings,
    pub backend: Arc<dyn DecisionBackend>,
}

impl DecisionRuntime {
    /// Read `EDGEQUAKE_DECISION_*` now. A bad value is kept as an error:
    /// [`Self::boot_check`] reports it, and decision runs refuse to start (LAW-160-4).
    pub fn from_env(store: Arc<dyn DecisionStore>) -> Self {
        Self::with_settings(DecisionSettings::from_env(), store)
    }

    /// Environment settings with an in-process store (memory mode, tests).
    pub fn in_memory_from_env() -> Self {
        Self::from_env(Arc::new(
            edgequake_storage::decision::MemoryDecisionStore::new(),
        ))
    }

    /// Build from explicit settings (tests, embedders).
    pub fn with_settings(
        settings: Result<DecisionSettings, DecisionError>,
        store: Arc<dyn DecisionStore>,
    ) -> Self {
        Self {
            settings: Arc::new(settings),
            store,
        }
    }

    /// The shared decision store (cache and review rows).
    pub fn store(&self) -> Arc<dyn DecisionStore> {
        Arc::clone(&self.store)
    }

    /// Fail startup on a bad `EDGEQUAKE_DECISION_*` value.
    pub fn boot_check(&self) -> Result<(), DecisionError> {
        self.settings
            .as_ref()
            .as_ref()
            .map(|_| ())
            .map_err(Clone::clone)
    }

    /// Server settings, or the boot error.
    pub fn server_settings(&self) -> Result<DecisionSettings, DecisionError> {
        self.settings.as_ref().clone()
    }

    /// True when a decision run is allowed for a workspace that has opted in
    /// (or when the operator forced the mode on).
    pub fn enabled(&self) -> bool {
        matches!(
            self.settings.as_ref(),
            Ok(s) if s.gate != DecisionGate::Locked
        )
    }

    /// Server settings with workspace and document overrides applied.
    ///
    /// Errors with [`DecisionError::Disabled`] when the operator locked the mode,
    /// and [`DecisionError::NotActivated`] when the workspace has not opted in.
    pub fn effective_settings(
        &self,
        workspace_meta: &HashMap<String, Value>,
        document_meta: Option<&Value>,
    ) -> Result<DecisionSettings, DecisionError> {
        let base = self.server_settings()?;
        let activation = base
            .gate
            .activation(workspace_decision_enabled(workspace_meta));
        if let Some(err) = activation.refusal() {
            return Err(err);
        }
        let mut overrides = DecisionOverrides::from_workspace_metadata(workspace_meta)?;
        if let Some(doc) = document_meta {
            overrides = overrides.with_document(doc)?;
        }
        Ok(base.with_overrides(&overrides))
    }

    /// Build the backend for a settings value.
    pub fn prepare(&self, settings: DecisionSettings) -> Result<PreparedBackend, DecisionError> {
        let backend = backend_from_settings(&settings)?;
        Ok(PreparedBackend { settings, backend })
    }

    /// Probe the backend. Returns the first problem as a typed error.
    pub async fn admit(prepared: &PreparedBackend) -> Result<BackendHealth, DecisionError> {
        let health = prepared.backend.health().await;
        match health.first_problem(&prepared.settings.model) {
            Some(problem) => Err(problem),
            None => Ok(health),
        }
    }

    /// Build the extractor for one document run.
    pub fn extractor(
        &self,
        prepared: PreparedBackend,
        schema: &EntityExtractionSchema,
        scope: DecisionScope,
        cancel: CancellationToken,
    ) -> Result<DecisionExtractor, DecisionError> {
        let options = DecisionRunOptions {
            gate: prepared.settings.gate_preset,
            pack_size: prepared.settings.pack_size,
            cache: Some(CacheLink {
                store: self.store(),
                scope,
            }),
            cancel,
        };
        let build = ontology_from_schema(schema)?;
        Ok(DecisionExtractor::new(prepared.backend, build, options))
    }

    /// Describe the server backend for the status endpoint.
    pub fn describe(&self) -> Option<BackendDescriptor> {
        let settings = self.server_settings().ok()?;
        backend_from_settings(&settings)
            .ok()
            .map(|b| b.descriptor())
    }

    /// Probe the backend for the status endpoint. Never errors: the state is the answer.
    ///
    /// `model` probes a workspace's own model tag instead of the server default.
    /// `workspace_meta` decides activation when the env gate is `WorkspaceChoice`.
    pub async fn status(
        &self,
        model: Option<&str>,
        workspace_meta: Option<&HashMap<String, Value>>,
    ) -> DecisionStatus {
        let limits = DecisionLimits::current();
        let empty = HashMap::new();
        let meta = workspace_meta.unwrap_or(&empty);
        let ws_on = workspace_decision_enabled(meta);
        let base = |activation: DecisionActivation, settings_error| DecisionStatus {
            enabled: activation.is_usable(),
            activation,
            settings_error,
            provider: None,
            backend: None,
            limits: limits.clone(),
            license_notice: LICENSE_NOTICE,
        };
        let mut settings = match self.server_settings() {
            Ok(s) => s,
            Err(e) => return base(DecisionActivation::Locked, Some(e.code().to_string())),
        };
        let activation = settings.gate.activation(ws_on);
        if activation == DecisionActivation::Locked {
            return base(activation, None);
        }
        if let Some(tag) = model {
            match check_model_name(tag) {
                Ok(valid) => settings.model = valid,
                Err(e) => return base(activation, Some(e.code().to_string())),
            }
        }
        let backend = match backend_from_settings(&settings) {
            Ok(b) => b,
            Err(e) => return base(activation, Some(e.code().to_string())),
        };
        let descriptor = backend.descriptor();
        let provider = DecisionProviderView {
            kind: descriptor.kind,
            label: "Ollama",
            base_url_host: descriptor.host.clone(),
        };
        let health = tokio::time::timeout(STATUS_PROBE_TIMEOUT, backend.health())
            .await
            .unwrap_or_else(|_| BackendHealth::unreachable("probe timed out"));
        DecisionStatus {
            provider: Some(provider),
            backend: Some(BackendStatus {
                kind: descriptor.kind,
                base_url_host: descriptor.host,
                model: descriptor.model,
                contract: edgextract::DECISION_CONTRACT,
                health,
            }),
            ..base(activation, None)
        }
    }

    /// List models on the decision host. Empty when locked or the host is down.
    pub async fn list_models(
        &self,
        model: Option<&str>,
        workspace_meta: Option<&HashMap<String, Value>>,
    ) -> DecisionModels {
        let status = self.status(model, workspace_meta).await;
        if status.activation == DecisionActivation::Locked {
            return DecisionModels {
                activation: status.activation,
                provider: status.provider,
                models: Vec::new(),
                error: Some("decision_disabled".into()),
            };
        }
        let settings = match self.server_settings() {
            Ok(mut s) => {
                if let Some(tag) = model.and_then(|m| check_model_name(m).ok()) {
                    s.model = tag;
                }
                s
            }
            Err(e) => {
                return DecisionModels {
                    activation: status.activation,
                    provider: status.provider,
                    models: Vec::new(),
                    error: Some(e.code().to_string()),
                };
            }
        };
        let backend = match backend_from_settings(&settings) {
            Ok(b) => b,
            Err(e) => {
                return DecisionModels {
                    activation: status.activation,
                    provider: status.provider,
                    models: Vec::new(),
                    error: Some(e.code().to_string()),
                };
            }
        };
        match tokio::time::timeout(STATUS_PROBE_TIMEOUT, backend.list_models()).await {
            Ok(Ok(models)) => DecisionModels {
                activation: status.activation,
                provider: status.provider,
                models,
                error: None,
            },
            Ok(Err(e)) => DecisionModels {
                activation: status.activation,
                provider: status.provider,
                models: Vec::new(),
                error: Some(e.code().to_string()),
            },
            Err(_) => DecisionModels {
                activation: status.activation,
                provider: status.provider,
                models: Vec::new(),
                error: Some("decision_timeout".into()),
            },
        }
    }
}

/// Body of `GET /decision/models`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionModels {
    pub activation: DecisionActivation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<DecisionProviderView>,
    pub models: Vec<ListedModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::decision::MemoryDecisionStore;
    use serde_json::json;

    fn runtime(gate: DecisionGate) -> DecisionRuntime {
        let settings = DecisionSettings {
            gate,
            ..Default::default()
        };
        DecisionRuntime::with_settings(Ok(settings), Arc::new(MemoryDecisionStore::new()))
    }

    // T-160-U63 — operator lock: decision runs are refused with a stable code.
    #[test]
    fn disabled_refuses() {
        let err = runtime(DecisionGate::Locked)
            .effective_settings(&HashMap::new(), None)
            .unwrap_err();
        assert_eq!(err.code(), "decision_disabled");
        assert!(!runtime(DecisionGate::Locked).enabled());
    }

    #[test]
    fn not_activated_without_workspace_opt_in() {
        let err = runtime(DecisionGate::WorkspaceChoice)
            .effective_settings(&HashMap::new(), None)
            .unwrap_err();
        assert_eq!(err.code(), "decision_not_activated");
        let mut on = HashMap::new();
        on.insert("decision_enabled".into(), json!(true));
        assert!(runtime(DecisionGate::WorkspaceChoice)
            .effective_settings(&on, None)
            .is_ok());
    }

    // T-160-U64 — workspace and document overrides stack; document wins on the gate.
    #[test]
    fn overrides_stack() {
        let rt = runtime(DecisionGate::ForcedOn);
        let mut ws = HashMap::new();
        ws.insert("decision_model".to_string(), json!("tev1:4b"));
        ws.insert("decision_pack_size".to_string(), json!(8));
        ws.insert("decision_gate_preset".to_string(), json!("strict"));
        let doc = json!({"decision_gate_preset": "recall"});
        let s = rt.effective_settings(&ws, Some(&doc)).unwrap();
        assert_eq!(s.model, "tev1:4b");
        assert_eq!(s.pack_size, 8);
        assert_eq!(s.gate_preset, super::super::GatePreset::Recall);
    }

    // T-160-U65 — a bad stored value is an error, never a silent default (LAW-160-4).
    #[test]
    fn bad_stored_value_errors() {
        let mut ws = HashMap::new();
        ws.insert("decision_pack_size".to_string(), json!(99));
        let err = runtime(DecisionGate::ForcedOn).effective_settings(&ws, None).unwrap_err();
        assert_eq!(err.code(), "invalid_decision_setting");
    }

    // T-160-U91 — status never errors: off, bad env, and bad model each give a body.
    #[tokio::test]
    async fn status_reports_every_state() {
        let off = runtime(DecisionGate::Locked).status(None, None).await;
        assert!(!off.enabled && off.backend.is_none() && off.settings_error.is_none());
        assert_eq!(off.activation, DecisionActivation::Locked);
        let inactive = runtime(DecisionGate::WorkspaceChoice)
            .status(None, None)
            .await;
        assert!(!inactive.enabled);
        assert_eq!(inactive.activation, DecisionActivation::Inactive);
        let bad_env = DecisionRuntime::with_settings(
            Err(DecisionError::Config("x".into())),
            Arc::new(MemoryDecisionStore::new()),
        )
        .status(None, None)
        .await;
        assert!(!bad_env.enabled);
        assert_eq!(
            bad_env.settings_error.as_deref(),
            Some("invalid_decision_setting")
        );
        let bad_model = runtime(DecisionGate::ForcedOn)
            .status(Some("bad model!"), None)
            .await;
        assert!(bad_model.enabled && bad_model.backend.is_none());
        assert!(bad_model.settings_error.is_some());
        assert_eq!(off.limits.pack_size_max, 16);
        assert!(off.limits.gate_presets.contains(&"balanced"));
    }

    // T-160-U66 — a bad environment value is kept and reported at boot.
    #[test]
    fn boot_error_is_reported() {
        let rt = DecisionRuntime::with_settings(
            Err(DecisionError::Config(
                "EDGEQUAKE_DECISION_PACK_SIZE='x'".into(),
            )),
            Arc::new(MemoryDecisionStore::new()),
        );
        assert!(rt.boot_check().is_err());
        assert!(!rt.enabled());
        assert!(rt.effective_settings(&HashMap::new(), None).is_err());
        assert!(runtime(DecisionGate::ForcedOn).boot_check().is_ok());
    }
}
