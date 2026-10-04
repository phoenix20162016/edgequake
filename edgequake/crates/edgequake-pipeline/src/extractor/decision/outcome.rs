//! What a decision run produced, summed over the chunks of one document (SPEC-160).
//!
//! The mapper writes one [`ExtractionResult`] per chunk. A document view needs one
//! answer: how many facts entered the graph, how many wait for review, how many
//! calls the model really served. This module sums them and gathers the review rows.

use edgequake_storage::decision::ReviewRow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::map::{
    rejected_from_metadata, review_rows_from_metadata, META_DECISION_MODEL, META_DECISION_STATS,
    META_DECISION_WARNINGS,
};
use crate::extractor::ExtractionResult;

/// Metadata key on the document that holds [`DecisionStats`].
pub const META_DOCUMENT_DECISION_STATS: &str = "decision_stats";

/// Counts for one document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionStats {
    /// Chunks that ran in decision mode.
    pub chunks: u64,
    /// Entities that entered the graph.
    pub entities: u64,
    /// Relations that entered the graph.
    pub relations: u64,
    /// Facts kept out of the graph and stored for review.
    pub review: u64,
    /// Facts the gate dropped.
    pub rejected: u64,
    /// Questions the model answered.
    pub backend_calls: u64,
    /// Questions answered from the cache.
    pub cache_hits: u64,
    /// Distinct warnings, in order of first sight.
    pub warnings: Vec<String>,
    /// Model that answered the questions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Which layer chose the mode (`document`, `workspace`, `env`). Set by the caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Stats and review rows of one document.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionOutcome {
    pub stats: DecisionStats,
    pub review: Vec<ReviewRow>,
}

/// Sum the decision chunks. `None` when no chunk ran in decision mode.
pub fn outcome_from_extractions(extractions: &[ExtractionResult]) -> Option<DecisionOutcome> {
    let mut stats = DecisionStats::default();
    let mut review = Vec::new();
    for e in extractions.iter().filter(|e| is_decision(e)) {
        stats.chunks += 1;
        stats.entities += e.entities.len() as u64;
        stats.relations += e.relationships.len() as u64;
        stats.rejected += rejected_from_metadata(&e.metadata);
        stats.backend_calls += counter(e, "backend_calls");
        stats.cache_hits += counter(e, "bridge_cache_hits");
        add_warnings(&mut stats.warnings, e);
        if stats.model.is_none() {
            stats.model = e
                .metadata
                .get(META_DECISION_MODEL)
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        review.extend(review_rows_from_metadata(&e.metadata));
    }
    if stats.chunks == 0 {
        return None;
    }
    stats.review = review.len() as u64;
    Some(DecisionOutcome { stats, review })
}

fn is_decision(e: &ExtractionResult) -> bool {
    e.metadata.get("extraction_mode").and_then(Value::as_str) == Some("decision")
}

fn counter(e: &ExtractionResult, key: &str) -> u64 {
    e.metadata
        .get(META_DECISION_STATS)
        .and_then(|s| s.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn add_warnings(into: &mut Vec<String>, e: &ExtractionResult) {
    let Some(list) = e
        .metadata
        .get(META_DECISION_WARNINGS)
        .and_then(Value::as_array)
    else {
        return;
    };
    for w in list.iter().filter_map(Value::as_str) {
        if !into.iter().any(|seen| seen == w) {
            into.push(w.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::decision::ReviewKind;
    use serde_json::json;

    fn row(chunk: &str) -> ReviewRow {
        ReviewRow {
            chunk_id: chunk.into(),
            kind: ReviewKind::Entity,
            subject: "ADA".into(),
            label: "PERSON".into(),
            object: None,
            score: 0.5,
            reason: None,
            sentence: "Ada wrote.".into(),
            model: "tev1:0.8b".into(),
            contract: "c".into(),
        }
    }

    fn decision_chunk(
        id: &str,
        calls: u64,
        hits: u64,
        rejected: u64,
        rows: usize,
    ) -> ExtractionResult {
        let mut e = ExtractionResult::new(id);
        e.metadata
            .insert("extraction_mode".into(), json!("decision"));
        e.metadata.insert(
            META_DECISION_STATS.into(),
            json!({"backend_calls": calls, "bridge_cache_hits": hits}),
        );
        e.metadata
            .insert("decision_rejected".into(), json!(rejected));
        e.metadata.insert(
            "decision_review".into(),
            serde_json::to_value(vec![row(id); rows]).unwrap(),
        );
        e
    }

    // T-160-U83 — an LLM-only document has no decision outcome.
    #[test]
    fn llm_chunks_yield_none() {
        assert!(outcome_from_extractions(&[ExtractionResult::new("c1")]).is_none());
        assert!(outcome_from_extractions(&[]).is_none());
    }

    // T-160-U84 — counts add up over chunks; review rows keep their chunk id.
    #[test]
    fn counts_sum_over_chunks() {
        let mut a = decision_chunk("c1", 3, 1, 2, 1);
        a.metadata
            .insert("decision_warnings".into(), json!(["w1", "w2"]));
        let mut b = decision_chunk("c2", 4, 0, 1, 2);
        b.metadata
            .insert("decision_warnings".into(), json!(["w2", "w3"]));
        let llm = ExtractionResult::new("c3");
        let out = outcome_from_extractions(&[a, llm, b]).unwrap();
        assert_eq!(out.stats.chunks, 2, "LLM chunk is not counted");
        assert_eq!(out.stats.backend_calls, 7);
        assert_eq!(out.stats.cache_hits, 1);
        assert_eq!(out.stats.rejected, 3);
        assert_eq!(out.stats.review, 3);
        assert_eq!(out.review.len(), 3);
        assert_eq!(out.stats.warnings, vec!["w1", "w2", "w3"]);
        assert_eq!(out.review[0].chunk_id, "c1");
        assert_eq!(out.stats.model, None, "no model in test chunks");
    }

    // T-160-U85 — stats round-trip as JSON for the document metadata.
    #[test]
    fn stats_round_trip() {
        let out = outcome_from_extractions(&[decision_chunk("c1", 1, 0, 0, 0)]).unwrap();
        let v = serde_json::to_value(&out.stats).unwrap();
        let back: DecisionStats = serde_json::from_value(v).unwrap();
        assert_eq!(back, out.stats);
    }
}
