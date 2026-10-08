//! Agent-visible id forms (SPEC-152 object model / SPEC-162).

/// Storage / ContextEntity id → agent `ent:{workspace}:{slug}`.
///
/// Prefer [`super::entity_ref::agent_id_for_node`] when you have a graph node id
/// (strips workspace scope before slugifying).
pub fn agent_entity_id(workspace: &str, name_or_id: &str) -> String {
    let slug = entity_slug(name_or_id);
    let ws = if workspace.is_empty() {
        "default"
    } else {
        workspace
    };
    format!("ent:{ws}:{slug}")
}

/// Accept `ent:ws:slug`, `ent:NAME`, bare NAME, or `{ws}::NAME`.
///
/// Legacy helper: returns a bare slug for display / tests. Graph tools must use
/// [`super::entity_ref::resolve_entity_node`] for store lookup (SPEC-162 R2).
pub fn resolve_entity_lookup(entity_id: &str) -> String {
    entity_slug(entity_id)
}

/// Bare UPPERCASE slug (preserves `-`; spaces → `_`).
pub fn entity_slug(name_or_id: &str) -> String {
    let raw = name_or_id
        .strip_prefix("ent:")
        .map(|rest| rest.split_once(':').map(|(_, slug)| slug).unwrap_or(rest))
        .unwrap_or(name_or_id);
    let raw = edgequake_storage::EntityId::bare_name_from_graph_node_id(raw);
    raw.trim()
        .to_ascii_uppercase()
        .chars()
        .map(|c| if c.is_whitespace() { '_' } else { c })
        .collect()
}

/// Compare-only fold (SPEC-162 R4). Delegates to storage SSOT.
pub fn fold_slug_key(raw: &str) -> String {
    edgequake_storage::fold_slug_key(raw)
}

/// Title Case display from ALL_CAPS slug.
pub fn title_case_label(slug: &str) -> String {
    slug.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                Some(f) => {
                    let mut s = f.to_uppercase().collect::<String>();
                    s.push_str(&chars.as_str().to_lowercase());
                    s
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
}

pub fn is_artifact_type(entity_type: &str) -> bool {
    let t = entity_type.to_ascii_uppercase();
    matches!(
        t.as_str(),
        "ARTIFACT" | "DRAWING" | "FIGURE" | "IMAGE" | "CONTENT"
    ) || t.contains("DRAWING")
        || t.contains("FIGURE")
}

pub fn map_entity_type(raw: &str) -> &'static str {
    match raw.to_ascii_uppercase().as_str() {
        "PERSON" | "PEOPLE" => "Person",
        "ORGANIZATION" | "ORG" | "COMPANY" => "Organization",
        "CONCEPT" => "Concept",
        "METHOD" | "TECH" => "Method",
        "SYSTEM" => "System",
        "METRIC" | "DATA" => "Metric",
        "EVENT" => "Event",
        "ARTIFACT" | "DRAWING" | "CONTENT" | "FIGURE" => "Artifact",
        "LOCATION" | "GEO" => "Location",
        _ => "Concept",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_slug() {
        assert_eq!(entity_slug("ent:ws:action_fusion"), "ACTION_FUSION");
        assert_eq!(entity_slug("ACTION_FUSION"), "ACTION_FUSION");
        assert_eq!(title_case_label("ACTION_FUSION"), "Action Fusion");
    }

    #[test]
    fn hyphenated_name_folds_for_compare() {
        // SPEC-162 step 1 / R4: hyphen and underscore share one fold key.
        assert_eq!(fold_slug_key("SELF-ATTENTION"), "SELF_ATTENTION");
        assert_eq!(fold_slug_key("ent:ws:self-attention"), "SELF_ATTENTION");
        assert_eq!(
            fold_slug_key("SELF-ATTENTION"),
            fold_slug_key("SELF_ATTENTION")
        );
        // Storage slug still preserves hyphen.
        assert_eq!(entity_slug("ent:ws:self-attention"), "SELF-ATTENTION");
    }
}
