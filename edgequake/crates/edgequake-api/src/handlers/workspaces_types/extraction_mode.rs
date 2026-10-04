//! SPEC-160 — workspace DTO fragments for the extraction mode.
//!
//! Both fragments are `#[serde(flatten)]`ed into the workspace request and
//! response DTOs, so the four fields are declared once (DRY).

use std::collections::HashMap;

use edgequake_pipeline::extraction_mode::ExtractionModeSource;
use edgequake_pipeline::extractor::decision::gate::META_DECISION_GATE_PRESET;
use edgequake_pipeline::extractor::decision::settings::{
    check_model_name, check_pack_size, parse_gate_preset_override, workspace_decision_enabled,
    DecisionActivation, DecisionGate, DecisionSettings, META_DECISION_ENABLED, META_DECISION_MODEL,
    META_DECISION_PACK_SIZE,
};
use edgequake_pipeline::{
    extraction_mode_from_metadata, parse_mode_override, resolve_extraction_mode_from_env,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::error::{ApiError, ApiResult};

/// Words that clear a workspace override.
const INHERIT_WORDS: [&str; 3] = ["", "inherit", "none"];

/// Request fragment (SPEC-160). Every field is optional; absent leaves the key unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct ExtractionModeFieldsDto {
    /// `llm`, `decision`, or `inherit` (clears the workspace default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extraction_mode: Option<String>,
    /// Decision gate preset: `strict`, `balanced`, `recall`, or `inherit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_gate_preset: Option<String>,
    /// Decision model name, for example `tev1:0.8b`, or `inherit`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_model: Option<String>,
    /// Questions per decision request, 1 to 16. `0` clears the override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_pack_size: Option<i64>,
    /// Turn the decision engine on for this workspace. The operator can still lock it off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_enabled: Option<bool>,
}

fn is_inherit(raw: &str) -> bool {
    INHERIT_WORDS.contains(&raw.trim().to_ascii_lowercase().as_str())
}

impl ExtractionModeFieldsDto {
    /// Check every field and return the first coded 400 (SPEC-160 08-api-contract).
    pub fn validate(&self) -> ApiResult<()> {
        if let Some(raw) = self.extraction_mode.as_deref() {
            parse_mode_override(Some(raw), ExtractionModeSource::Workspace).map_err(|e| {
                ApiError::bad_request_coded("invalid_extraction_mode", e.to_string())
            })?;
        }
        if let Some(raw) = self.decision_gate_preset.as_deref() {
            parse_gate_preset_override(Some(raw)).map_err(|e| {
                ApiError::bad_request_coded("invalid_decision_gate_preset", e.to_string())
            })?;
        }
        if let Some(raw) = self.decision_model.as_deref().filter(|m| !is_inherit(m)) {
            check_model_name(raw).map_err(|e| {
                ApiError::bad_request_coded("invalid_decision_model", e.to_string())
            })?;
        }
        if let Some(n) = self.decision_pack_size.filter(|n| *n != 0) {
            check_pack_size(n).map_err(|e| {
                ApiError::bad_request_coded("invalid_decision_pack_size", e.to_string())
            })?;
        }
        Ok(())
    }
}

/// The mode a new document would get, and who decided it.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct EffectiveExtractionModeDto {
    /// `llm` or `decision`.
    pub mode: String,
    /// `workspace`, `env`, or `default`.
    pub source: String,
}

/// Response fragment (SPEC-160).
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct ExtractionModeView {
    /// Workspace default word, or null when the workspace inherits.
    #[serde(default)]
    pub extraction_mode: Option<String>,
    /// Mode after the server default is folded in. Null if a stored value is invalid.
    #[serde(default)]
    pub effective_extraction_mode: Option<EffectiveExtractionModeDto>,
    #[serde(default)]
    pub decision_gate_preset: Option<String>,
    #[serde(default)]
    pub decision_model: Option<String>,
    #[serde(default)]
    pub decision_pack_size: Option<u32>,
    /// Stored workspace opt-in.
    #[serde(default)]
    pub decision_enabled: Option<bool>,
    /// `locked` | `inactive` | `active` | `forced`.
    #[serde(default)]
    pub decision_activation: Option<String>,
}

impl ExtractionModeView {
    /// Build the view from workspace metadata.
    pub fn from_metadata(meta: &HashMap<String, Value>) -> Self {
        let word = extraction_mode_from_metadata(meta);
        let effective = resolve_extraction_mode_from_env(None, word.as_deref())
            .ok()
            .map(|r| EffectiveExtractionModeDto {
                mode: r.mode.as_str().to_string(),
                source: r.source.as_str().to_string(),
            });
        let text = |key: &str| meta.get(key).and_then(Value::as_str).map(str::to_string);
        let gate = DecisionSettings::from_env()
            .map(|s| s.gate)
            .unwrap_or(DecisionGate::Locked);
        let enabled = meta.get(META_DECISION_ENABLED).and_then(Value::as_bool);
        let activation = DecisionActivation::of(gate, workspace_decision_enabled(meta));
        Self {
            extraction_mode: word,
            effective_extraction_mode: effective,
            decision_gate_preset: text(META_DECISION_GATE_PRESET),
            decision_model: text(META_DECISION_MODEL),
            decision_pack_size: meta
                .get(META_DECISION_PACK_SIZE)
                .and_then(Value::as_u64)
                .and_then(|n| u32::try_from(n).ok()),
            decision_enabled: enabled,
            decision_activation: Some(activation.as_str().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn dto(
        mode: Option<&str>,
        preset: Option<&str>,
        model: Option<&str>,
        pack: Option<i64>,
    ) -> ExtractionModeFieldsDto {
        ExtractionModeFieldsDto {
            extraction_mode: mode.map(str::to_string),
            decision_gate_preset: preset.map(str::to_string),
            decision_model: model.map(str::to_string),
            decision_pack_size: pack,
            decision_enabled: None,
        }
    }

    // T-160-U57 — each bad field maps to its own stable 400 code.
    #[test]
    fn validation_codes() {
        assert!(dto(None, None, None, None).validate().is_ok());
        assert!(
            dto(Some("decision"), Some("recall"), Some("tev1:4b"), Some(16))
                .validate()
                .is_ok()
        );
        assert!(dto(Some("inherit"), Some(""), Some("none"), Some(0))
            .validate()
            .is_ok());
        let code = |d: ExtractionModeFieldsDto| d.validate().unwrap_err().code();
        assert_eq!(
            code(dto(Some("tev1"), None, None, None)),
            "invalid_extraction_mode"
        );
        assert_eq!(
            code(dto(None, Some("loose"), None, None)),
            "invalid_decision_gate_preset"
        );
        assert_eq!(
            code(dto(None, None, Some("a b"), None)),
            "invalid_decision_model"
        );
        assert_eq!(
            code(dto(None, None, None, Some(17))),
            "invalid_decision_pack_size"
        );
        assert_eq!(
            code(dto(None, None, None, Some(-1))),
            "invalid_decision_pack_size"
        );
        assert_eq!(
            dto(Some("x"), None, None, None)
                .validate()
                .unwrap_err()
                .status_code(),
            axum::http::StatusCode::BAD_REQUEST
        );
    }

    // T-160-U58 — the view reads the stored keys; absent keys are null.
    #[test]
    fn view_from_metadata() {
        let mut m = HashMap::new();
        let empty = ExtractionModeView::from_metadata(&m);
        assert!(empty.extraction_mode.is_none() && empty.decision_pack_size.is_none());
        m.insert("extraction_mode".into(), json!("decision"));
        m.insert("decision_pack_size".into(), json!(6));
        m.insert("decision_model".into(), json!("tev1:4b"));
        let v = ExtractionModeView::from_metadata(&m);
        assert_eq!(v.extraction_mode.as_deref(), Some("decision"));
        assert_eq!(v.decision_pack_size, Some(6));
        assert_eq!(v.effective_extraction_mode.unwrap().source, "workspace");
    }

    // T-160-U59 — a stored invalid word does not crash the view; effective is null.
    #[test]
    fn invalid_stored_word_is_visible() {
        let mut m = HashMap::new();
        m.insert("extraction_mode".into(), json!("tev1"));
        let v = ExtractionModeView::from_metadata(&m);
        assert_eq!(v.extraction_mode.as_deref(), Some("tev1"));
        assert!(v.effective_extraction_mode.is_none());
    }

    // T-160-U60 — flatten keeps the wire format flat; empty fields are omitted on requests.
    #[test]
    fn flat_wire_format() {
        let d: ExtractionModeFieldsDto =
            serde_json::from_value(json!({"extraction_mode":"llm","decision_pack_size":3}))
                .unwrap();
        assert_eq!(d.decision_pack_size, Some(3));
        assert_eq!(
            serde_json::to_value(ExtractionModeFieldsDto::default()).unwrap(),
            json!({})
        );
    }
}
