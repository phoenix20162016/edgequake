//! SPEC-160 — Knowledge-graph extraction mode (single source of truth).
//!
//! A document is extracted in exactly one mode:
//!
//! | Mode       | Meaning                                                        |
//! |------------|----------------------------------------------------------------|
//! | `llm`      | Open extraction by a chat LLM (default, pre-SPEC-160 behavior) |
//! | `decision` | Closed yes/no and pick-one decisions by a decision model       |
//!
//! WHY a vendor-neutral name: the first decision model is Tev1, but the model
//! is a preset. The mode name must outlive any one model (LAW-160-1).
//!
//! Precedence (LAW-160-2, same shape as SPEC-117 extract caps):
//! document override > workspace default > fleet env > `llm`.
//!
//! This module is pure. It reads no environment variable except in the one
//! thin wrapper [`resolve_extraction_mode_from_env`]. It never falls back
//! silently from an invalid value to `llm` (LAW-160-4): a typo must not send
//! private text to a cloud LLM.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Workspace and document metadata key (SPEC-160).
pub const META_EXTRACTION_MODE: &str = "extraction_mode";

/// Fleet environment variable (SPEC-160). Unset means `llm`.
pub const EXTRACTION_MODE_ENV: &str = "EDGEQUAKE_EXTRACTION_MODE";

/// Words that mean "no override" at the API boundary.
const INHERIT_WORDS: [&str; 3] = ["", "inherit", "none"];

/// How the knowledge graph is extracted from a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionMode {
    /// Open extraction by a chat LLM (default).
    #[default]
    Llm,
    /// Closed decisions by a decision model (Tev1, Nimble, ...).
    Decision,
}

impl ExtractionMode {
    /// Every mode, in display order.
    pub const ALL: [ExtractionMode; 2] = [ExtractionMode::Llm, ExtractionMode::Decision];

    /// Stable wire value (API, metadata, fingerprint).
    pub fn as_str(self) -> &'static str {
        match self {
            ExtractionMode::Llm => "llm",
            ExtractionMode::Decision => "decision",
        }
    }

    /// Parse one mode word. Case and surrounding spaces do not matter.
    ///
    /// Returns `None` for an unknown word. Inherit words are not modes.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "llm" => Some(ExtractionMode::Llm),
            "decision" => Some(ExtractionMode::Decision),
            _ => None,
        }
    }

    /// True when this mode needs a decision backend.
    pub fn needs_decision_backend(self) -> bool {
        matches!(self, ExtractionMode::Decision)
    }
}

impl fmt::Display for ExtractionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which layer supplied the winning mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionModeSource {
    /// Per-upload override.
    Document,
    /// Workspace default.
    Workspace,
    /// Fleet environment variable.
    Env,
    /// Built-in default (`llm`).
    Default,
}

impl ExtractionModeSource {
    /// Stable wire value.
    pub fn as_str(self) -> &'static str {
        match self {
            ExtractionModeSource::Document => "document",
            ExtractionModeSource::Workspace => "workspace",
            ExtractionModeSource::Env => "env",
            ExtractionModeSource::Default => "default",
        }
    }
}

/// The winning mode and the layer that supplied it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedExtractionMode {
    pub mode: ExtractionMode,
    pub source: ExtractionModeSource,
}

/// A layer held a word that is neither a mode nor an inherit word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownExtractionMode {
    pub value: String,
    pub source: ExtractionModeSource,
}

impl fmt::Display for UnknownExtractionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unsupported extraction_mode '{}' from {}. Allowed: inherit, llm, decision",
            self.value,
            self.source.as_str()
        )
    }
}

impl std::error::Error for UnknownExtractionMode {}

/// Outcome of reading one layer.
enum Layer {
    /// The layer sets no override.
    Inherit,
    /// The layer sets a mode.
    Set(ExtractionMode),
}

fn read_layer(
    raw: Option<&str>,
    source: ExtractionModeSource,
) -> Result<Layer, UnknownExtractionMode> {
    let Some(raw) = raw else {
        return Ok(Layer::Inherit);
    };
    let word = raw.trim().to_ascii_lowercase();
    if INHERIT_WORDS.contains(&word.as_str()) {
        return Ok(Layer::Inherit);
    }
    ExtractionMode::parse(&word)
        .map(Layer::Set)
        .ok_or_else(|| UnknownExtractionMode {
            value: raw.trim().to_string(),
            source,
        })
}

/// Validate one API word. `Ok(None)` means "clear the override".
///
/// Used by workspace writes and upload admission so both reject the same
/// words with the same message (DRY).
pub fn parse_mode_override(
    raw: Option<&str>,
    source: ExtractionModeSource,
) -> Result<Option<ExtractionMode>, UnknownExtractionMode> {
    Ok(match read_layer(raw, source)? {
        Layer::Inherit => None,
        Layer::Set(mode) => Some(mode),
    })
}

/// Pure precedence resolution (LAW-160-2).
///
/// Every layer is validated, even a layer that loses. A bad stored value is
/// reported instead of hidden behind a higher layer.
pub fn resolve_extraction_mode(
    document: Option<&str>,
    workspace: Option<&str>,
    env: Option<&str>,
) -> Result<ResolvedExtractionMode, UnknownExtractionMode> {
    let doc = read_layer(document, ExtractionModeSource::Document)?;
    let ws = read_layer(workspace, ExtractionModeSource::Workspace)?;
    let fleet = read_layer(env, ExtractionModeSource::Env)?;
    let picked = [
        (doc, ExtractionModeSource::Document),
        (ws, ExtractionModeSource::Workspace),
        (fleet, ExtractionModeSource::Env),
    ]
    .into_iter()
    .find_map(|(layer, source)| match layer {
        Layer::Set(mode) => Some(ResolvedExtractionMode { mode, source }),
        Layer::Inherit => None,
    });
    Ok(picked.unwrap_or(ResolvedExtractionMode {
        mode: ExtractionMode::Llm,
        source: ExtractionModeSource::Default,
    }))
}

/// Read the workspace default word from workspace metadata.
pub fn extraction_mode_from_metadata(metadata: &HashMap<String, Value>) -> Option<String> {
    metadata
        .get(META_EXTRACTION_MODE)
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Read the document override word from document or task metadata JSON.
pub fn extraction_mode_from_value(metadata: &Value) -> Option<String> {
    metadata
        .get(META_EXTRACTION_MODE)
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Resolve with the fleet variable read from the process environment.
pub fn resolve_extraction_mode_from_env(
    document: Option<&str>,
    workspace: Option<&str>,
) -> Result<ResolvedExtractionMode, UnknownExtractionMode> {
    let env = std::env::var(EXTRACTION_MODE_ENV).ok();
    resolve_extraction_mode(document, workspace, env.as_deref())
}

/// Apply the workspace default to metadata (shared by Postgres and in-memory).
///
/// - `None` leaves the metadata unchanged.
/// - An inherit word clears the key.
/// - `llm` or `decision` stores the canonical word.
/// - Any other word is an error and leaves the metadata unchanged.
pub fn apply_extraction_mode_metadata(
    metadata: &mut HashMap<String, Value>,
    requested: Option<&str>,
) -> Result<(), UnknownExtractionMode> {
    let Some(raw) = requested else {
        return Ok(());
    };
    match parse_mode_override(Some(raw), ExtractionModeSource::Workspace)? {
        None => {
            metadata.remove(META_EXTRACTION_MODE);
        }
        Some(mode) => {
            metadata.insert(META_EXTRACTION_MODE.to_string(), Value::from(mode.as_str()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn resolved(
        doc: Option<&str>,
        ws: Option<&str>,
        env: Option<&str>,
    ) -> (ExtractionMode, ExtractionModeSource) {
        let r = resolve_extraction_mode(doc, ws, env).expect("valid layers");
        (r.mode, r.source)
    }

    // T-160-U01 — default is llm when no layer speaks.
    #[test]
    fn default_is_llm() {
        assert_eq!(
            resolved(None, None, None),
            (ExtractionMode::Llm, ExtractionModeSource::Default)
        );
        assert_eq!(ExtractionMode::default(), ExtractionMode::Llm);
    }

    // T-160-U02 — precedence: document > workspace > env > default.
    #[test]
    fn precedence_document_workspace_env() {
        assert_eq!(
            resolved(Some("llm"), Some("decision"), Some("decision")),
            (ExtractionMode::Llm, ExtractionModeSource::Document)
        );
        assert_eq!(
            resolved(None, Some("decision"), Some("llm")),
            (ExtractionMode::Decision, ExtractionModeSource::Workspace)
        );
        assert_eq!(
            resolved(None, None, Some("decision")),
            (ExtractionMode::Decision, ExtractionModeSource::Env)
        );
    }

    // T-160-U03 — inherit words never win, they pass control down.
    #[test]
    fn inherit_words_pass_through() {
        for word in ["", "  ", "inherit", "INHERIT", "none"] {
            assert_eq!(
                resolved(Some(word), Some("decision"), None),
                (ExtractionMode::Decision, ExtractionModeSource::Workspace),
                "word {word:?}"
            );
        }
    }

    // T-160-U04 — case and spaces do not matter.
    #[test]
    fn parse_is_case_and_space_tolerant() {
        assert_eq!(
            ExtractionMode::parse(" Decision "),
            Some(ExtractionMode::Decision)
        );
        assert_eq!(ExtractionMode::parse("LLM"), Some(ExtractionMode::Llm));
        assert_eq!(ExtractionMode::parse("tev1"), None);
        assert_eq!(ExtractionMode::parse("inherit"), None);
    }

    // T-160-U05 — an unknown word is an error that names its layer (EC-160-01/02).
    #[test]
    fn unknown_word_is_rejected_with_source() {
        let err = resolve_extraction_mode(Some("tev1"), None, None).unwrap_err();
        assert_eq!(err.source, ExtractionModeSource::Document);
        assert_eq!(err.value, "tev1");
        let err = resolve_extraction_mode(None, None, Some("fast")).unwrap_err();
        assert_eq!(err.source, ExtractionModeSource::Env);
        assert!(err.to_string().contains("Allowed: inherit, llm, decision"));
    }

    // T-160-U06 — a bad layer is reported even when a higher layer wins (LAW-160-4).
    #[test]
    fn losing_layer_is_still_validated() {
        let err = resolve_extraction_mode(Some("llm"), Some("oops"), None).unwrap_err();
        assert_eq!(err.source, ExtractionModeSource::Workspace);
    }

    // T-160-U07 — wire values are stable (they enter the fingerprint).
    #[test]
    fn wire_values_are_stable() {
        assert_eq!(ExtractionMode::Llm.as_str(), "llm");
        assert_eq!(ExtractionMode::Decision.as_str(), "decision");
        assert_eq!(ExtractionModeSource::Document.as_str(), "document");
        assert_eq!(
            serde_json::to_value(ExtractionMode::Decision).unwrap(),
            json!("decision")
        );
        assert!(ExtractionMode::Decision.needs_decision_backend());
        assert!(!ExtractionMode::Llm.needs_decision_backend());
        assert_eq!(ExtractionMode::ALL.len(), 2);
    }

    // T-160-U08 — workspace write: set, replace, clear, reject, no-op.
    #[test]
    fn apply_metadata_set_clear_reject() {
        let mut meta: HashMap<String, Value> = HashMap::new();
        apply_extraction_mode_metadata(&mut meta, None).unwrap();
        assert!(meta.is_empty());

        apply_extraction_mode_metadata(&mut meta, Some(" Decision ")).unwrap();
        assert_eq!(meta.get(META_EXTRACTION_MODE), Some(&json!("decision")));
        assert_eq!(
            extraction_mode_from_metadata(&meta).as_deref(),
            Some("decision")
        );

        let err = apply_extraction_mode_metadata(&mut meta, Some("bogus")).unwrap_err();
        assert_eq!(err.source, ExtractionModeSource::Workspace);
        assert_eq!(
            meta.get(META_EXTRACTION_MODE),
            Some(&json!("decision")),
            "a rejected write must not change metadata"
        );

        apply_extraction_mode_metadata(&mut meta, Some("inherit")).unwrap();
        assert!(!meta.contains_key(META_EXTRACTION_MODE));
    }

    // T-160-U09 — document override is read from task metadata JSON.
    #[test]
    fn reads_document_override_from_value() {
        assert_eq!(
            extraction_mode_from_value(&json!({ "extraction_mode": "decision" })).as_deref(),
            Some("decision")
        );
        assert_eq!(
            extraction_mode_from_value(&json!({ "extraction_mode": null })),
            None
        );
        assert_eq!(extraction_mode_from_value(&json!({})), None);
    }

    // T-160-U10 — parse_mode_override maps inherit words to None.
    #[test]
    fn parse_mode_override_maps_inherit_to_none() {
        assert_eq!(
            parse_mode_override(Some("inherit"), ExtractionModeSource::Document).unwrap(),
            None
        );
        assert_eq!(
            parse_mode_override(None, ExtractionModeSource::Document).unwrap(),
            None
        );
        assert_eq!(
            parse_mode_override(Some("llm"), ExtractionModeSource::Document).unwrap(),
            Some(ExtractionMode::Llm)
        );
    }
}
