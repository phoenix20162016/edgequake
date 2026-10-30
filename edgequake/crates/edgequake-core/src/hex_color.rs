//! Strict `#RGB` / `#RRGGBB` color validation shared by every hex-color setting
//! (entity-type graph colors, tenant brand color).
//!
//! WHY one implementation: the UI, the API and the metadata writers must agree on
//! exactly which strings are accepted and how they are stored (canonical lowercase
//! `#rrggbb`). Diverging copies would let one surface store a value another rejects.

/// True when `value` is `#RGB` or `#RRGGBB` (hex digits only, surrounding whitespace ignored).
pub fn is_hex_color(value: &str) -> bool {
    let bytes = value.trim().as_bytes();
    if bytes.first() != Some(&b'#') {
        return false;
    }
    let body = &bytes[1..];
    if body.len() != 3 && body.len() != 6 {
        return false;
    }
    body.iter().all(|b| b.is_ascii_hexdigit())
}

/// Expand `#RGB` → `#rrggbb` and lowercase `#RRGGBB`. `None` when invalid.
pub fn canonicalize_hex_color(value: &str) -> Option<String> {
    let v = value.trim();
    if !is_hex_color(v) {
        return None;
    }
    let body = &v[1..];
    if body.len() == 3 {
        let mut out = String::with_capacity(7);
        out.push('#');
        for c in body.chars() {
            let lower = c.to_ascii_lowercase();
            out.push(lower);
            out.push(lower);
        }
        Some(out)
    } else {
        Some(format!("#{}", body.to_ascii_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_short_and_long_forms() {
        assert!(is_hex_color("#abc"));
        assert!(is_hex_color("  #A1B2C3 "));
    }

    #[test]
    fn rejects_malformed_values() {
        for bad in [
            "",
            "abc",
            "#ab",
            "#abcd",
            "#abcdeg",
            "#abcdef00",
            "# abc",
            "rgb(0,0,0)",
        ] {
            assert!(!is_hex_color(bad), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn canonicalizes_to_lowercase_long_form() {
        assert_eq!(canonicalize_hex_color("#0F0").as_deref(), Some("#00ff00"));
        assert_eq!(
            canonicalize_hex_color("#4B5BD6").as_deref(),
            Some("#4b5bd6")
        );
        assert_eq!(canonicalize_hex_color("#zzz"), None);
    }
}
