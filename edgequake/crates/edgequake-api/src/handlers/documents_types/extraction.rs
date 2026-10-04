//! SPEC-160: how a document was (or will be) extracted, as the document views show it.

use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;

use edgequake_pipeline::extractor::decision::META_DOCUMENT_DECISION_STATS;
use edgequake_pipeline::ExtractionMode;

/// Fields flattened into the document list row and the document detail.
///
/// All fields are absent for a plain LLM document, so existing clients see no change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, ToSchema)]
pub struct DocumentExtractionView {
    /// `decision` once a decision run left stats, or the word the upload asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "decision")]
    pub extraction_mode: Option<String>,

    /// Who chose the mode: `document`, `workspace`, `env`, `default`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "workspace")]
    pub extraction_mode_source: Option<String>,

    /// Per-document counts of a decision run: graph facts, review rows, model calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub decision_stats: Option<Value>,
}

impl DocumentExtractionView {
    /// Read the view from the document metadata map.
    ///
    /// Stats written by a finished run win: they say what really happened. Before that
    /// the document shows the word the upload asked for, if any.
    pub fn from_metadata(meta: &Map<String, Value>) -> Self {
        if let Some(stats) = meta
            .get(META_DOCUMENT_DECISION_STATS)
            .filter(|v| v.is_object())
        {
            return Self {
                extraction_mode: Some("decision".into()),
                extraction_mode_source: stats
                    .get("source")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                decision_stats: Some(stats.clone()),
            };
        }
        let asked = meta
            .get("extraction_mode")
            .and_then(Value::as_str)
            .and_then(ExtractionMode::parse)
            .map(|m| m.as_str().to_string());
        Self {
            extraction_mode_source: asked.as_ref().map(|_| recorded_source(meta)),
            extraction_mode: asked,
            decision_stats: None,
        }
    }
}

/// Source words the admission step writes (`ExtractionModeSource::as_str`).
const SOURCE_WORDS: [&str; 4] = ["document", "workspace", "env", "default"];

/// The layer admission recorded for the mode. A record from before it stored one is `document`.
fn recorded_source(meta: &Map<String, Value>) -> String {
    meta.get("extraction_mode_source")
        .and_then(Value::as_str)
        .filter(|word| SOURCE_WORDS.contains(word))
        .unwrap_or("document")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(v: Value) -> Map<String, Value> {
        v.as_object().cloned().unwrap()
    }

    // T-160-U88 — a plain LLM document shows nothing new.
    #[test]
    fn plain_document_is_silent() {
        let view = DocumentExtractionView::from_metadata(&map(json!({"status": "completed"})));
        assert_eq!(view, DocumentExtractionView::default());
        assert_eq!(serde_json::to_value(&view).unwrap(), json!({}));
    }

    // T-160-U89 — stats of a finished run say what happened.
    #[test]
    fn finished_run_reports_stats() {
        let view = DocumentExtractionView::from_metadata(&map(json!({
            "extraction_mode": "llm",
            "decision_stats": {"entities": 4, "review": 2, "source": "workspace"}
        })));
        assert_eq!(view.extraction_mode.as_deref(), Some("decision"));
        assert_eq!(view.extraction_mode_source.as_deref(), Some("workspace"));
        assert_eq!(view.decision_stats.unwrap()["review"], 2);
    }

    // T-160-U95 — the layer admission recorded is shown, not guessed.
    #[test]
    fn recorded_source_is_reported() {
        for word in ["workspace", "env", "default", "document"] {
            let view = DocumentExtractionView::from_metadata(&map(json!({
                "extraction_mode": "llm",
                "extraction_mode_source": word
            })));
            assert_eq!(view.extraction_mode_source.as_deref(), Some(word));
        }
        let junk = DocumentExtractionView::from_metadata(&map(json!({
            "extraction_mode": "llm",
            "extraction_mode_source": "somewhere"
        })));
        assert_eq!(junk.extraction_mode_source.as_deref(), Some("document"));
    }

    // T-160-U90 — before the run, the requested word is shown as a document choice.
    #[test]
    fn queued_document_shows_requested_word() {
        let view =
            DocumentExtractionView::from_metadata(&map(json!({"extraction_mode": "decision"})));
        assert_eq!(view.extraction_mode.as_deref(), Some("decision"));
        assert_eq!(view.extraction_mode_source.as_deref(), Some("document"));
        let junk = DocumentExtractionView::from_metadata(&map(json!({"extraction_mode": "x"})));
        assert!(
            junk.extraction_mode.is_none(),
            "unknown words are not echoed"
        );
    }
}
