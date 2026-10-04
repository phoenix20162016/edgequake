//! Gate presets (SPEC-160 §Gate). A preset is a named `edgextract::GateConfig`.
//!
//! Strict raises only the accept cutoffs. Reject cutoffs stay put, so a doubtful
//! row goes to REVIEW and is never lost. Recall lowers both.
//!
//! The values are starting points. `edgextract` ships them with `fitted=false`
//! and SPEC-160 W8 measures them. The UI shows "Uncalibrated" until then.

use edgextract::GateConfig;
use serde::{Deserialize, Serialize};

/// Metadata key of the workspace and document gate preset.
pub const META_DECISION_GATE_PRESET: &str = "decision_gate_preset";

/// How strict the gate is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GatePreset {
    /// Few facts, high certainty.
    Strict,
    /// `edgextract` defaults.
    #[default]
    Balanced,
    /// More facts, more doubt.
    Recall,
}

impl GatePreset {
    pub const ALL: [GatePreset; 3] = [GatePreset::Strict, GatePreset::Balanced, GatePreset::Recall];

    pub fn as_str(self) -> &'static str {
        match self {
            GatePreset::Strict => "strict",
            GatePreset::Balanced => "balanced",
            GatePreset::Recall => "recall",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "strict" => Some(GatePreset::Strict),
            "balanced" => Some(GatePreset::Balanced),
            "recall" => Some(GatePreset::Recall),
            _ => None,
        }
    }

    /// The `edgextract` gate for this preset.
    pub fn to_config(self) -> GateConfig {
        let base = GateConfig::default();
        match self {
            GatePreset::Balanced => base,
            GatePreset::Strict => GateConfig {
                accept_prob: 0.85,
                noul_yes: 0.90,
                ..base
            },
            GatePreset::Recall => GateConfig {
                accept_prob: 0.55,
                reject_prob: 0.30,
                noul_yes: 0.65,
                noul_no: 0.10,
                ..base
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-160-U16 — presets order the cutoffs: strict > balanced > recall.
    #[test]
    fn presets_are_ordered() {
        let s = GatePreset::Strict.to_config();
        let b = GatePreset::Balanced.to_config();
        let r = GatePreset::Recall.to_config();
        assert!(s.accept_prob > b.accept_prob && b.accept_prob > r.accept_prob);
        assert!(s.noul_yes > b.noul_yes && b.noul_yes > r.noul_yes);
        assert!(s.reject_prob >= b.reject_prob && b.reject_prob > r.reject_prob);
        assert!(s.noul_no >= b.noul_no && b.noul_no > r.noul_no);
        assert!(!s.fitted && !b.fitted && !r.fitted, "uncalibrated until W8");
    }

    // T-160-U39 — balanced equals the edgextract defaults.
    #[test]
    fn balanced_is_default() {
        let d = GateConfig::default();
        let b = GatePreset::Balanced.to_config();
        assert_eq!(
            (b.accept_prob, b.reject_prob, b.noul_yes, b.noul_no),
            (d.accept_prob, d.reject_prob, d.noul_yes, d.noul_no)
        );
    }

    // T-160-U40 — parse is tolerant and total.
    #[test]
    fn parse_words() {
        assert_eq!(GatePreset::parse(" Strict "), Some(GatePreset::Strict));
        assert_eq!(GatePreset::parse("x"), None);
        for p in GatePreset::ALL {
            assert_eq!(GatePreset::parse(p.as_str()), Some(p));
        }
    }

    // T-160-U41 — gate boundaries: exactly at the cutoff counts as accept/reject.
    #[test]
    fn noul_boundaries() {
        use edgextract::types::GateBand;
        let g = GatePreset::Balanced.to_config();
        assert_eq!(g.band_noul(0.80), GateBand::Accept);
        assert_eq!(g.band_noul(0.7999), GateBand::Review);
        assert_eq!(g.band_noul(0.20), GateBand::Reject);
        assert_eq!(g.band_noul(0.2001), GateBand::Review);
    }
}
