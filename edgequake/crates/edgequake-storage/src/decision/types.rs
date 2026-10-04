//! Value types of the decision store.

use serde::{Deserialize, Serialize};

/// Who owns the data. A workspace is the isolation unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DecisionScope {
    pub tenant_id: Option<String>,
    pub workspace_id: String,
}

impl DecisionScope {
    pub fn new(tenant_id: Option<&str>, workspace_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.map(str::to_string),
            workspace_id: workspace_id.to_string(),
        }
    }
}

/// What a review row is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewKind {
    Entity,
    Relation,
}

impl ReviewKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ReviewKind::Entity => "entity",
            ReviewKind::Relation => "relation",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "entity" => Some(ReviewKind::Entity),
            "relation" => Some(ReviewKind::Relation),
            _ => None,
        }
    }
}

/// A fact the gate kept out of the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewRow {
    pub chunk_id: String,
    pub kind: ReviewKind,
    /// Entity text, or relation source.
    pub subject: String,
    /// Proposed entity type, or proposed relation.
    pub label: String,
    /// Relation target. `None` for entity rows.
    pub object: Option<String>,
    /// Gate score in `0.0..=1.0`. It is ordered evidence, not a probability.
    pub score: f32,
    /// Why the row is here, when the gate has a reason beyond the score.
    pub reason: Option<String>,
    /// Source sentence, cut to [`ReviewRow::MAX_SENTENCE_CHARS`].
    pub sentence: String,
    pub model: String,
    pub contract: String,
}

impl ReviewRow {
    /// Upper bound of the stored sentence.
    pub const MAX_SENTENCE_CHARS: usize = 1000;

    /// Cut a sentence on a char boundary.
    pub fn clip_sentence(raw: &str) -> String {
        raw.chars().take(Self::MAX_SENTENCE_CHARS).collect()
    }
}
