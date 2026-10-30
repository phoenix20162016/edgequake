//! Workspace entity-type color map helpers (SPEC-102 / FEAT-102).
//!
//! Pure validation + metadata apply shared by postgres and in-memory workspace
//! services. Hex parsing is delegated to [`crate::hex_color`] (one definition for
//! every color setting).

use std::collections::HashMap;

use crate::hex_color::{canonicalize_hex_color, is_hex_color};

/// Max entity-type color overrides per workspace.
pub const MAX_ENTITY_TYPE_COLORS: usize = 50;

/// True when `value` is `#RGB` or `#RRGGBB` (hex digits only).
pub fn is_valid_entity_type_hex(value: &str) -> bool {
    is_hex_color(value)
}

/// Expand `#RGB` → `#rrggbb` and lowercase `#RRGGBB`.
pub fn canonicalize_entity_type_hex(value: &str) -> Option<String> {
    canonicalize_hex_color(value)
}

/// Normalize entity type key for color map (UPPERCASE, spaces/hyphens → `_`).
pub fn normalize_entity_type_color_key(raw: &str) -> String {
    raw.trim().to_uppercase().replace([' ', '-'], "_")
}

/// Apply `entity_type_colors` map to workspace metadata.
///
/// - `None` → leave unchanged
/// - empty map → remove key
/// - invalid hex → `Err`
/// - keys normalized; values canonical `#rrggbb`; max 50 entries
pub fn apply_entity_type_colors_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    colors: Option<HashMap<String, String>>,
) -> Result<(), String> {
    let Some(colors) = colors else {
        return Ok(());
    };
    if colors.is_empty() {
        metadata.remove("entity_type_colors");
        return Ok(());
    }

    let mut normalized = serde_json::Map::new();
    for (raw_key, raw_val) in colors {
        let key = normalize_entity_type_color_key(&raw_key);
        if key.is_empty() || key == "DEFAULT" {
            continue;
        }
        let Some(hex) = canonicalize_entity_type_hex(&raw_val) else {
            return Err(format!(
                "Invalid entity_type_colors hex for '{}': '{}'. Expected #RGB or #RRGGBB",
                raw_key, raw_val
            ));
        };
        if normalized.len() >= MAX_ENTITY_TYPE_COLORS && !normalized.contains_key(&key) {
            return Err(format!(
                "entity_type_colors exceeds maximum of {} entries",
                MAX_ENTITY_TYPE_COLORS
            ));
        }
        normalized.insert(key, serde_json::Value::String(hex));
    }

    if normalized.is_empty() {
        metadata.remove("entity_type_colors");
    } else {
        metadata.insert(
            "entity_type_colors".to_string(),
            serde_json::Value::Object(normalized),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_entity_type_colors_normalizes_and_canonicalizes() {
        let mut meta = HashMap::new();
        let mut colors = HashMap::new();
        colors.insert("person".to_string(), "#0F0".to_string());
        apply_entity_type_colors_metadata(&mut meta, Some(colors)).unwrap();
        let map = meta.get("entity_type_colors").unwrap().as_object().unwrap();
        assert_eq!(map.get("PERSON").unwrap().as_str().unwrap(), "#00ff00");
    }

    #[test]
    fn apply_entity_type_colors_rejects_invalid_hex() {
        let mut meta = HashMap::new();
        let mut colors = HashMap::new();
        colors.insert("PERSON".to_string(), "#gg0000".to_string());
        let err = apply_entity_type_colors_metadata(&mut meta, Some(colors)).unwrap_err();
        assert!(err.contains("Invalid"));
    }

    #[test]
    fn apply_entity_type_colors_empty_clears() {
        let mut meta = HashMap::new();
        meta.insert(
            "entity_type_colors".to_string(),
            serde_json::json!({"PERSON": "#112233"}),
        );
        apply_entity_type_colors_metadata(&mut meta, Some(HashMap::new())).unwrap();
        assert!(!meta.contains_key("entity_type_colors"));
    }

    #[test]
    fn apply_entity_type_colors_caps_at_fifty() {
        let mut meta = HashMap::new();
        let mut colors = HashMap::new();
        for i in 0..51 {
            colors.insert(format!("TYPE_{i}"), "#abcdef".to_string());
        }
        let err = apply_entity_type_colors_metadata(&mut meta, Some(colors)).unwrap_err();
        assert!(err.contains("maximum"));
    }
}
