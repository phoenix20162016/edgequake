//! `edgextract` output to EdgeQuake output (SRP: shape only, no I/O).
//!
//! The mapper does three jobs:
//!
//! 1. Re-normalize every name with the EdgeQuake normalizer (one identity rule).
//! 2. Enforce graph rules the merger expects: no self loops, both endpoints
//!    present, five keywords at most, chunk lineage on every record.
//! 3. Turn REVIEW rows into [`ReviewRow`]s and keep them in result metadata.

use std::collections::HashMap;

use edgequake_storage::decision::{ReviewKind, ReviewRow};
use edgequake_storage::normalize_entity_name;
use edgextract::types::{ExtractedEntity as XEntity, ExtractedRelationship as XRelation, Sentence};
use edgextract::ExtractionResult as XResult;
use serde_json::{json, Value};

use crate::extractor::{ExtractedEntity, ExtractedRelationship, ExtractionResult};

/// Metadata keys written on every decision result.
pub const META_DECISION_STATS: &str = "decision";
pub const META_DECISION_REVIEW: &str = "decision_review";
pub const META_DECISION_REJECTED: &str = "decision_rejected";
pub const META_DECISION_WARNINGS: &str = "decision_warnings";
pub const META_DECISION_MODEL: &str = "decision_model";

const MAX_KEYWORDS: usize = 5;

/// `edgextract` marks a mention it does not type with this label.
const NOT_ENTITY_LABEL: &str = "NOT_ENTITY";
/// What a review row shows when the entity question itself was doubtful.
pub const ENTITY_UNDECIDED_LABEL: &str = "ENTITY";
pub const ENTITY_UNSURE_REASON: &str = "entity_unsure";

/// Facts the mapper needs about the run.
pub struct MapContext<'a> {
    pub chunk_id: &'a str,
    /// Known only when the caller knows it. `None` leaves stamping to the caller.
    pub document_id: Option<&'a str>,
    pub model: &'a str,
    pub contract: &'a str,
    pub sentences: &'a [Sentence],
    pub warnings: &'a [String],
}

/// Convert one chunk result.
pub fn to_eq_result(x: XResult, ctx: &MapContext<'_>) -> ExtractionResult {
    let (entities, renames) = map_entities(&x.entities, ctx);
    let relationships = map_relationships(&x.relationships, &renames, ctx);
    let review = review_rows(&x.review, ctx);

    let mut out = ExtractionResult::new(ctx.chunk_id);
    out.entities = entities;
    out.relationships = relationships;
    out.input_tokens = x.input_tokens.max(0) as usize;
    out.output_tokens = x.output_tokens.max(0) as usize;
    out.extraction_time_ms = x.extraction_time_ms.max(0) as u64;
    out.metadata
        .insert("extraction_mode".into(), json!("decision"));
    out.metadata.insert(META_DECISION_STATS.into(), x.metadata);
    out.metadata.insert(
        META_DECISION_REVIEW.into(),
        serde_json::to_value(&review).unwrap_or(Value::Null),
    );
    out.metadata
        .insert(META_DECISION_REJECTED.into(), json!(x.rejected.len()));
    out.metadata
        .insert(META_DECISION_MODEL.into(), json!(ctx.model));
    if !ctx.warnings.is_empty() {
        out.metadata
            .insert(META_DECISION_WARNINGS.into(), json!(ctx.warnings));
    }
    out
}

fn map_entities(
    source: &[XEntity],
    ctx: &MapContext<'_>,
) -> (Vec<ExtractedEntity>, HashMap<String, String>) {
    let mut by_name: HashMap<String, usize> = HashMap::new();
    let mut out: Vec<ExtractedEntity> = Vec::new();
    let mut renames = HashMap::new();
    for e in source {
        let name = normalize_entity_name(&e.name);
        if name.is_empty() {
            continue;
        }
        renames.insert(e.name.clone(), name.clone());
        if let Some(&i) = by_name.get(&name) {
            merge_entity(&mut out[i], e);
            continue;
        }
        let mut mapped =
            ExtractedEntity::new(name.clone(), e.entity_type.clone(), e.description.clone())
                .with_importance(e.importance as f32)
                .with_source_chunk_id(ctx.chunk_id);
        mapped.source_document_id = ctx.document_id.map(str::to_string);
        mapped.source_spans = e.source_spans.clone();
        mapped.display_name = e.display_name.clone();
        by_name.insert(name, out.len());
        out.push(mapped);
    }
    (out, renames)
}

fn merge_entity(into: &mut ExtractedEntity, other: &XEntity) {
    for span in &other.source_spans {
        if !into.source_spans.contains(span) {
            into.source_spans.push(span.clone());
        }
    }
    into.importance = into.importance.max(other.importance as f32);
}

fn map_relationships(
    source: &[XRelation],
    renames: &HashMap<String, String>,
    ctx: &MapContext<'_>,
) -> Vec<ExtractedRelationship> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for r in source {
        let (Some(src), Some(tgt)) = (renames.get(&r.source), renames.get(&r.target)) else {
            continue;
        };
        if src == tgt || !seen.insert((src.clone(), tgt.clone(), r.relation_type.clone())) {
            continue;
        }
        let mut rel = ExtractedRelationship::new(src.clone(), tgt.clone(), r.relation_type.clone());
        rel.description = r.description.clone();
        rel.weight = (r.weight as f32).clamp(0.0, 1.0);
        rel.keywords = r.keywords.iter().take(MAX_KEYWORDS).cloned().collect();
        rel.add_source_chunk_id(ctx.chunk_id);
        rel.source_chunk_id = Some(ctx.chunk_id.to_string());
        rel.source_document_id = ctx.document_id.map(str::to_string);
        out.push(rel);
    }
    out
}

fn sentence_text(ctx: &MapContext<'_>, id: &str) -> String {
    ctx.sentences
        .iter()
        .find(|s| s.id == id)
        .map(|s| ReviewRow::clip_sentence(&s.text))
        .unwrap_or_default()
}

fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn review_rows(raw: &[Value], ctx: &MapContext<'_>) -> Vec<ReviewRow> {
    raw.iter().filter_map(|v| review_row(v, ctx)).collect()
}

fn review_row(v: &Value, ctx: &MapContext<'_>) -> Option<ReviewRow> {
    let kind = ReviewKind::parse(str_field(v, "kind"))?;
    let (subject, label, object) = match kind {
        ReviewKind::Entity => (str_field(v, "text"), str_field(v, "type"), None),
        ReviewKind::Relation => {
            let asked = str_field(v, "asked");
            let label = if asked.is_empty() {
                str_field(v, "type")
            } else {
                asked
            };
            (
                str_field(v, "source"),
                label,
                Some(str_field(v, "target").to_string()),
            )
        }
    };
    if subject.is_empty() || label.is_empty() {
        return None;
    }
    let mut reason = v.get("reason").and_then(Value::as_str).map(str::to_string);
    let mut label = label.to_string();
    if kind == ReviewKind::Entity && label == NOT_ENTITY_LABEL {
        // The gate asked "is this an entity?" and was unsure. A reviewer must not read
        // "NOT_ENTITY" as the model's verdict, so show the open question instead.
        label = ENTITY_UNDECIDED_LABEL.to_string();
        reason.get_or_insert_with(|| ENTITY_UNSURE_REASON.to_string());
    }
    Some(ReviewRow {
        chunk_id: ctx.chunk_id.to_string(),
        kind,
        subject: subject.to_string(),
        label,
        object,
        score: v
            .get("prob")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0) as f32,
        reason,
        sentence: sentence_text(ctx, str_field(v, "sentence_id")),
        model: ctx.model.to_string(),
        contract: ctx.contract.to_string(),
    })
}

/// Read the review rows a decision result carries.
pub fn review_rows_from_metadata(meta: &HashMap<String, Value>) -> Vec<ReviewRow> {
    meta.get(META_DECISION_REVIEW)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Count of REJECT rows a decision result carries.
pub fn rejected_from_metadata(meta: &HashMap<String, Value>) -> u64 {
    meta.get(META_DECISION_REJECTED)
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgextract::types::ExtractedEntity as E;

    fn ctx<'a>(sentences: &'a [Sentence], warnings: &'a [String]) -> MapContext<'a> {
        MapContext {
            chunk_id: "c1",
            document_id: Some("d1"),
            model: "m",
            contract: "k",
            sentences,
            warnings,
        }
    }

    fn ent(name: &str, t: &str) -> E {
        E {
            name: name.into(),
            entity_type: t.into(),
            description: format!("{name} desc"),
            importance: 0.9,
            source_spans: vec![name.into()],
            source_chunk_ids: vec!["x".into()],
            source_document_id: None,
            display_name: Some(name.into()),
        }
    }

    fn rel(s: &str, t: &str) -> XRelation {
        XRelation {
            source: s.into(),
            target: t.into(),
            relation_type: "WORKS_AT".into(),
            description: "ev".into(),
            weight: 0.8,
            keywords: (0..8).map(|i| format!("k{i}")).collect(),
            source_chunk_ids: vec![],
            source_document_id: None,
        }
    }

    fn sentence() -> Sentence {
        Sentence {
            id: "s1".into(),
            text: "Ada works at Acme.".into(),
            start: 0,
            end: 18,
            heading_path: vec![],
            index: 0,
        }
    }

    // T-160-U25 — lineage lands on every entity and relation; counts match.
    #[test]
    fn maps_lineage() {
        let x = XResult {
            entities: vec![ent("ADA", "PERSON"), ent("ACME", "ORGANIZATION")],
            relationships: vec![rel("ADA", "ACME")],
            ..Default::default()
        };
        let out = to_eq_result(x, &ctx(&[], &[]));
        assert_eq!(out.entities.len(), 2);
        assert_eq!(out.entities[0].source_chunk_ids, vec!["c1"]);
        assert_eq!(out.entities[0].source_document_id.as_deref(), Some("d1"));
        assert_eq!(out.relationships[0].source_chunk_ids, vec!["c1"]);
        assert_eq!(out.relationships[0].keywords.len(), 5, "BR0004");
        assert_eq!(out.metadata["extraction_mode"], json!("decision"));
    }

    // T-160-U26 — self loops, dangling endpoints and duplicates are dropped.
    #[test]
    fn drops_bad_relations() {
        let x = XResult {
            entities: vec![ent("ADA", "PERSON"), ent("ACME", "ORGANIZATION")],
            relationships: vec![
                rel("ADA", "ADA"),
                rel("ADA", "GHOST"),
                rel("ADA", "ACME"),
                rel("ADA", "ACME"),
            ],
            ..Default::default()
        };
        assert_eq!(to_eq_result(x, &ctx(&[], &[])).relationships.len(), 1);
    }

    // T-160-U27 — names pass through the EdgeQuake normalizer; opaque ids vanish.
    #[test]
    fn renormalizes_names() {
        let x = XResult {
            entities: vec![
                ent("Ada Lovelace", "PERSON"),
                ent("42", "DATA"),
                ent("ADA_LOVELACE", "PERSON"),
            ],
            relationships: vec![rel("Ada Lovelace", "ADA_LOVELACE")],
            ..Default::default()
        };
        let out = to_eq_result(x, &ctx(&[], &[]));
        assert_eq!(out.entities.len(), 1, "merged by normalized name");
        assert_eq!(out.entities[0].name, "ADA_LOVELACE");
        assert!(
            out.relationships.is_empty(),
            "same entity after merge is a self loop"
        );
    }

    // T-160-U28 — review rows keep text, label, score and sentence.
    #[test]
    fn review_rows_roundtrip() {
        let x = XResult {
            review: vec![
                json!({"kind":"entity","text":"Acme","type":"ORGANIZATION","prob":0.62,"sentence_id":"s1"}),
                json!({"kind":"relation","source":"Ada","target":"Acme","type":"NONE","asked":"WORKS_AT","prob":0.5,"sentence_id":"s1","reason":"endpoint_not_accepted"}),
                json!({"kind":"entity","text":"","type":"PERSON"}),
            ],
            rejected: vec![json!({}), json!({})],
            ..Default::default()
        };
        let s = [sentence()];
        let out = to_eq_result(x, &ctx(&s, &[]));
        let rows = review_rows_from_metadata(&out.metadata);
        assert_eq!(rows.len(), 2, "empty subject is dropped");
        assert_eq!(rows[0].sentence, "Ada works at Acme.");
        assert_eq!(rows[1].label, "WORKS_AT");
        assert_eq!(rows[1].object.as_deref(), Some("Acme"));
        assert_eq!(rows[1].reason.as_deref(), Some("endpoint_not_accepted"));
        assert_eq!(rejected_from_metadata(&out.metadata), 2);
    }

    // T-160-U94 — an unsure "is it an entity?" is shown as that question, not as a verdict.
    #[test]
    fn unsure_entity_is_not_labeled_not_entity() {
        let x = XResult {
            review: vec![
                json!({"kind":"entity","text":"Ada","type":"NOT_ENTITY","prob":0.79,"sentence_id":"s1"}),
                json!({"kind":"entity","text":"Acme","type":"NOT_ENTITY","prob":0.5,"reason":"flipped","sentence_id":"s1"}),
            ],
            ..Default::default()
        };
        let s = [sentence()];
        let rows = review_rows_from_metadata(&to_eq_result(x, &ctx(&s, &[])).metadata);
        assert_eq!(rows[0].label, ENTITY_UNDECIDED_LABEL);
        assert_eq!(rows[0].reason.as_deref(), Some(ENTITY_UNSURE_REASON));
        assert_eq!(
            rows[1].reason.as_deref(),
            Some("flipped"),
            "a given reason wins"
        );
    }

    // T-160-U29 — warnings travel in metadata.
    #[test]
    fn warnings_in_metadata() {
        let w = vec!["w1".to_string()];
        let out = to_eq_result(XResult::default(), &ctx(&[], &w));
        assert_eq!(out.metadata[META_DECISION_WARNINGS], json!(["w1"]));
        assert!(!to_eq_result(XResult::default(), &ctx(&[], &[]))
            .metadata
            .contains_key(META_DECISION_WARNINGS));
    }
}
