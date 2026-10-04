//! SPEC-160 — Workspace extraction-mode metadata apply (shared by Postgres + in-memory).
//!
//! One function writes all four keys. The validators live in `edgequake-pipeline`
//! (`apply_extraction_mode_metadata`, `apply_decision_metadata`), so the upload
//! path and the workspace path reject the same words with the same text (DRY).

use std::collections::HashMap;

use edgequake_pipeline::apply_extraction_mode_metadata;
use edgequake_pipeline::extractor::decision::settings::{
    apply_decision_metadata, DecisionMetadataRequest,
};
use serde_json::Value;

/// The workspace fields SPEC-160 adds to create and update requests.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtractionModeFields<'a> {
    /// `llm`, `decision`, or an inherit word. `None` leaves the key unchanged.
    pub extraction_mode: Option<&'a str>,
    /// `strict`, `balanced`, `recall`, or an inherit word.
    pub decision_gate_preset: Option<&'a str>,
    /// Decision model name, or an inherit word.
    pub decision_model: Option<&'a str>,
    /// `1`-`16`. `0` clears the override.
    pub decision_pack_size: Option<i64>,
    /// Workspace opt-in for the decision engine.
    pub decision_enabled: Option<bool>,
}

impl ExtractionModeFields<'_> {
    /// True when the request touches none of the SPEC-160 keys.
    pub fn is_empty(&self) -> bool {
        self.extraction_mode.is_none()
            && self.decision_gate_preset.is_none()
            && self.decision_model.is_none()
            && self.decision_pack_size.is_none()
            && self.decision_enabled.is_none()
    }
}

/// Apply the fields to workspace metadata.
///
/// All-or-nothing: on error the metadata is unchanged (EC-160-14).
pub fn apply_extraction_mode_fields(
    metadata: &mut HashMap<String, Value>,
    fields: &ExtractionModeFields<'_>,
) -> Result<(), String> {
    if fields.is_empty() {
        return Ok(());
    }
    let mut staged = metadata.clone();
    apply_extraction_mode_metadata(&mut staged, fields.extraction_mode)
        .map_err(|e| e.to_string())?;
    apply_decision_metadata(
        &mut staged,
        &DecisionMetadataRequest {
            model: fields.decision_model,
            pack_size: fields.decision_pack_size,
            gate_preset: fields.decision_gate_preset,
            enabled: fields.decision_enabled,
        },
    )
    .map_err(|e| e.to_string())?;
    *metadata = staged;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T-160-U54 — all four keys are written together.
    #[test]
    fn writes_all_keys() {
        let mut m = HashMap::new();
        apply_extraction_mode_fields(
            &mut m,
            &ExtractionModeFields {
                extraction_mode: Some("Decision"),
                decision_gate_preset: Some("strict"),
                decision_model: Some("tev1:4b"),
                decision_pack_size: Some(6),
                decision_enabled: Some(true),
            },
        )
        .unwrap();
        assert_eq!(m["extraction_mode"], json!("decision"));
        assert_eq!(m["decision_gate_preset"], json!("strict"));
        assert_eq!(m["decision_model"], json!("tev1:4b"));
        assert_eq!(m["decision_pack_size"], json!(6));
        assert_eq!(m["decision_enabled"], json!(true));
    }

    // T-160-U55 — a bad second field leaves the first one unwritten (EC-160-14).
    #[test]
    fn rejects_atomically() {
        let mut m = HashMap::new();
        m.insert("extraction_mode".to_string(), json!("llm"));
        let err = apply_extraction_mode_fields(
            &mut m,
            &ExtractionModeFields {
                extraction_mode: Some("decision"),
                decision_pack_size: Some(99),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(err.contains("pack size"), "{err}");
        assert_eq!(m["extraction_mode"], json!("llm"), "unchanged");
        let err = apply_extraction_mode_fields(
            &mut m,
            &ExtractionModeFields {
                extraction_mode: Some("tev1"),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(err.contains("Unsupported extraction_mode 'tev1'"), "{err}");
    }

    // T-160-U56 — an inherit word clears; no fields is a no-op.
    #[test]
    fn clears_and_noops() {
        let mut m = HashMap::new();
        m.insert("extraction_mode".to_string(), json!("decision"));
        m.insert("decision_pack_size".to_string(), json!(4));
        apply_extraction_mode_fields(&mut m, &ExtractionModeFields::default()).unwrap();
        assert_eq!(m.len(), 2);
        apply_extraction_mode_fields(
            &mut m,
            &ExtractionModeFields {
                extraction_mode: Some("inherit"),
                decision_pack_size: Some(0),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(m.is_empty());
    }
}
