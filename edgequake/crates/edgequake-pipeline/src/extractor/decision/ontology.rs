//! Workspace schema to decision ontology (SPEC-160, OCP).
//!
//! EdgeQuake already holds a per-workspace [`EntityExtractionSchema`]. The
//! decision mode reads that same schema, so a workspace has one source of truth
//! for types and relations in both modes (DRY).
//!
//! `edgextract` limits an ontology to 2-50 types and 100 type-pair edges. This
//! module respects both limits and reports every reduction as a warning.

use std::collections::BTreeMap;

use edgextract::ontology::{EntityType, Ontology, RelationType, MAX_RELATION_EDGES, MAX_TYPE_LIST};
use indexmap::IndexMap;

use super::error::DecisionError;
use crate::prompts::EntityExtractionSchema;

/// Relation id used when a workspace sets no relation types.
const GENERIC_RELATION: &str = "RELATED_TO";
/// Type used to pad a one-type schema up to the two types `edgextract` needs.
const PAD_TYPE: &str = "OTHER";

/// An ontology and the reductions made to build it.
#[derive(Debug, Clone)]
pub struct OntologyBuild {
    pub ontology: Ontology,
    /// Human-readable reductions. Empty when the schema fit as is.
    pub warnings: Vec<String>,
}

/// Plain-words description of a type id. Known types get a clear sentence.
fn type_description(id: &str) -> String {
    match id {
        "PERSON" => "A named person.".into(),
        "CREATURE" => "A named animal or fictional creature.".into(),
        "ORGANIZATION" => "A named company, team, institution, or other group.".into(),
        "LOCATION" => "A named city, country, region, or place.".into(),
        "EVENT" => "A named event that happens at a time.".into(),
        "CONCEPT" => "A named idea, theory, or abstract notion.".into(),
        "METHOD" => "A named method, algorithm, or technique.".into(),
        "CONTENT" => "A named document, book, article, or other work.".into(),
        "DATA" => "A named dataset, metric, or measured value.".into(),
        "ARTIFACT" => "A named tool, product, system, or made object.".into(),
        "NATURALOBJECT" => "A named natural object, such as a river or a star.".into(),
        "OTHER" => "A named thing that fits no other kind.".into(),
        other => format!("A named {}.", other.to_lowercase().replace('_', " ")),
    }
}

fn relation_description(id: &str) -> String {
    if id == GENERIC_RELATION {
        return "The two named things are related in the sentence.".into();
    }
    format!(
        "The first thing {} the second.",
        id.to_lowercase().replace('_', " ")
    )
}

fn entity_types(
    schema: &EntityExtractionSchema,
    warnings: &mut Vec<String>,
) -> Result<Vec<EntityType>, DecisionError> {
    let mut ids: Vec<String> = Vec::new();
    for t in &schema.types {
        if !ids.contains(t) {
            ids.push(t.clone());
        }
    }
    if ids.len() > MAX_TYPE_LIST {
        return Err(DecisionError::Ontology(format!(
            "The workspace lists {} entity types. Decision mode accepts at most {MAX_TYPE_LIST}. \
             Remove types in the workspace settings.",
            ids.len()
        )));
    }
    if ids.len() < 2 {
        let pad = if ids.iter().any(|i| i == PAD_TYPE) {
            "CONCEPT"
        } else {
            PAD_TYPE
        };
        warnings.push(format!(
            "Added the type {pad}: decision mode needs at least two types."
        ));
        ids.push(pad.to_string());
    }
    Ok(ids
        .into_iter()
        .map(|id| EntityType {
            description: type_description(&id),
            id,
            aliases: Vec::new(),
            color: "#64748b".into(),
        })
        .collect())
}

/// Group typed edges by relation id: `relation -> (domain, range)`.
fn relations_from_edges(
    schema: &EntityExtractionSchema,
    type_ids: &[String],
    warnings: &mut Vec<String>,
) -> Vec<RelationType> {
    let mut grouped: BTreeMap<String, (Vec<String>, Vec<String>)> = BTreeMap::new();
    let mut skipped = 0usize;
    for edge in &schema.relation_edges {
        if !type_ids.contains(&edge.source) || !type_ids.contains(&edge.target) {
            skipped += 1;
            continue;
        }
        let entry = grouped.entry(edge.relation.clone()).or_default();
        if !entry.0.contains(&edge.source) {
            entry.0.push(edge.source.clone());
        }
        if !entry.1.contains(&edge.target) {
            entry.1.push(edge.target.clone());
        }
    }
    if skipped > 0 {
        warnings.push(format!(
            "Skipped {skipped} relation edge(s) that name a type outside the type list."
        ));
    }
    grouped
        .into_iter()
        .map(|(id, (domain, range))| RelationType {
            description: relation_description(&id),
            id,
            domain,
            range,
            symmetric: false,
        })
        .collect()
}

/// Every listed relation may link any two types.
fn relations_unconstrained(relation_ids: &[String], type_ids: &[String]) -> Vec<RelationType> {
    relation_ids
        .iter()
        .map(|id| RelationType {
            id: id.clone(),
            description: relation_description(id),
            domain: type_ids.to_vec(),
            range: type_ids.to_vec(),
            symmetric: false,
        })
        .collect()
}

/// One generic relation over as many types as the pair limit allows.
fn relations_generic(type_ids: &[String], warnings: &mut Vec<String>) -> Vec<RelationType> {
    let mut usable: Vec<String> = type_ids
        .iter()
        .filter(|t| *t != PAD_TYPE)
        .cloned()
        .collect();
    let cap = (MAX_RELATION_EDGES as f64).sqrt() as usize;
    if usable.len() > cap {
        usable.truncate(cap);
        warnings.push(format!(
            "Generic relations cover the first {cap} types only. Set relation types to widen them."
        ));
    }
    if usable.len() < 2 {
        return Vec::new();
    }
    relations_unconstrained(&[GENERIC_RELATION.to_string()], &usable)
}

fn pair_count(relations: &[RelationType]) -> usize {
    relations
        .iter()
        .map(|r| r.domain.len() * r.range.len())
        .sum()
}

fn build_relations(
    schema: &EntityExtractionSchema,
    type_ids: &[String],
    warnings: &mut Vec<String>,
) -> Vec<RelationType> {
    let relations = if !schema.relation_edges.is_empty() {
        relations_from_edges(schema, type_ids, warnings)
    } else if !schema.relation_types.is_empty() {
        relations_unconstrained(&schema.relation_types, type_ids)
    } else {
        relations_generic(type_ids, warnings)
    };
    let pairs = pair_count(&relations);
    if pairs > MAX_RELATION_EDGES {
        warnings.push(format!(
            "The relation list allows {pairs} type pairs, above the limit of {MAX_RELATION_EDGES}. \
             Entities are extracted and relations are skipped. Add relation edges to narrow them."
        ));
        return Vec::new();
    }
    relations
}

/// Build the ontology for a workspace schema.
pub fn ontology_from_schema(
    schema: &EntityExtractionSchema,
) -> Result<OntologyBuild, DecisionError> {
    let mut warnings = Vec::new();
    let types = entity_types(schema, &mut warnings)?;
    let type_ids: Vec<String> = types.iter().map(|t| t.id.clone()).collect();
    let relations = build_relations(schema, &type_ids, &mut warnings);
    let ontology = Ontology {
        id: "edgequake_workspace".into(),
        version: 1,
        title: "EdgeQuake workspace schema".into(),
        types,
        relations,
        gazetteer: IndexMap::new(),
    }
    .validate()
    .map_err(|e| DecisionError::Ontology(e.to_string()))?;
    Ok(OntologyBuild { ontology, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::RelationEdge;

    fn schema(types: &[&str]) -> EntityExtractionSchema {
        EntityExtractionSchema {
            types: types.iter().map(|t| t.to_string()).collect(),
            strict: true,
            relation_types: Vec::new(),
            relation_strict: true,
            relation_edges: Vec::new(),
        }
    }

    fn edge(s: &str, r: &str, t: &str) -> RelationEdge {
        RelationEdge {
            source: s.into(),
            relation: r.into(),
            target: t.into(),
        }
    }

    // T-160-U17 — the default 12-type schema builds, with a capped generic relation.
    #[test]
    fn default_schema_builds_with_generic_relation() {
        let b = ontology_from_schema(&EntityExtractionSchema::server_default()).unwrap();
        assert_eq!(b.ontology.types.len(), 12);
        assert_eq!(b.ontology.relations.len(), 1);
        assert_eq!(b.ontology.relations[0].id, "RELATED_TO");
        assert!(pair_count(&b.ontology.relations) <= MAX_RELATION_EDGES);
        assert_eq!(b.warnings.len(), 1, "{:?}", b.warnings);
    }

    // T-160-U18 — typed edges become domain/range groups.
    #[test]
    fn edges_group_by_relation() {
        let mut s = schema(&["PERSON", "ORGANIZATION", "LOCATION"]);
        s.relation_edges = vec![
            edge("PERSON", "WORKS_AT", "ORGANIZATION"),
            edge("ORGANIZATION", "LOCATED_IN", "LOCATION"),
            edge("PERSON", "LOCATED_IN", "LOCATION"),
        ];
        let b = ontology_from_schema(&s).unwrap();
        assert_eq!(b.ontology.relations.len(), 2);
        let loc = b
            .ontology
            .relations
            .iter()
            .find(|r| r.id == "LOCATED_IN")
            .unwrap();
        assert_eq!(loc.domain, vec!["ORGANIZATION", "PERSON"]);
        assert_eq!(loc.range, vec!["LOCATION"]);
        assert!(b.warnings.is_empty());
    }

    // T-160-U19 — an edge with an unknown type is skipped and reported.
    #[test]
    fn edge_with_unknown_type_is_reported() {
        let mut s = schema(&["PERSON", "ORGANIZATION"]);
        s.relation_edges = vec![
            edge("PERSON", "WORKS_AT", "ORGANIZATION"),
            edge("PERSON", "OWNS", "SHIP"),
        ];
        let b = ontology_from_schema(&s).unwrap();
        assert_eq!(b.ontology.relations.len(), 1);
        assert!(b.warnings.iter().any(|w| w.contains("Skipped 1")));
    }

    // T-160-U20 — more than 50 types is an error that says what to do (EC-160-25).
    #[test]
    fn too_many_types_is_an_error() {
        let ids: Vec<String> = (0..51).map(|i| format!("T{i}")).collect();
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let err = ontology_from_schema(&schema(&refs)).unwrap_err();
        assert_eq!(err.code(), "invalid_decision_ontology");
        assert!(err.to_string().contains("at most 50"));
    }

    // T-160-U21 — one type is padded to two.
    #[test]
    fn single_type_is_padded() {
        let b = ontology_from_schema(&schema(&["PERSON"])).unwrap();
        assert_eq!(b.ontology.types.len(), 2);
        assert!(
            b.ontology.relations.is_empty(),
            "pad type is not a relation endpoint"
        );
        assert!(!b.warnings.is_empty());
    }

    // T-160-U22 — relation types without edges over many types exceed the pair limit: entity-only.
    #[test]
    fn too_many_pairs_drops_relations() {
        let mut s = EntityExtractionSchema::server_default();
        s.relation_types = vec!["A_TO".into(), "B_TO".into()];
        let b = ontology_from_schema(&s).unwrap();
        assert!(b.ontology.relations.is_empty());
        assert!(b
            .warnings
            .iter()
            .any(|w| w.contains("relations are skipped")));
    }

    // T-160-U23 — duplicate types collapse.
    #[test]
    fn duplicate_types_collapse() {
        let b = ontology_from_schema(&schema(&["PERSON", "PERSON", "LOCATION"])).unwrap();
        assert_eq!(b.ontology.types.len(), 2);
    }

    // T-160-U24 — unknown custom types get a plain description.
    #[test]
    fn custom_type_description() {
        let b = ontology_from_schema(&schema(&["DRUG_NAME", "DISEASE"])).unwrap();
        assert_eq!(b.ontology.types[0].description, "A named drug name.");
    }
}
