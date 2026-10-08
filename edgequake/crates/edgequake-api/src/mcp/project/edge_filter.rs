//! Strong / weak edge split for neighborhood and graph_image (SPEC-162 R11/R12).

use crate::handlers::entities_types::NeighborhoodEdge;

pub const WEAK_EDGE_HINT: &str = "Set include_weak_edges to true to show RELATED_TO edges.";

pub struct EdgeSplit<'a> {
    pub kept: Vec<&'a NeighborhoodEdge>,
    pub strong_count: usize,
    pub weak_count: usize,
    pub dropped_weak: usize,
}

pub fn is_weak_edge(relation_type: &str) -> bool {
    relation_type.eq_ignore_ascii_case("RELATED_TO")
}

pub fn split_edges(edges: &[NeighborhoodEdge], include_weak: bool) -> EdgeSplit<'_> {
    let mut kept = Vec::new();
    let mut strong_count = 0usize;
    let mut weak_count = 0usize;
    let mut dropped_weak = 0usize;
    for e in edges {
        if is_weak_edge(&e.relation_type) {
            weak_count += 1;
            if include_weak {
                kept.push(e);
            } else {
                dropped_weak += 1;
            }
        } else {
            strong_count += 1;
            kept.push(e);
        }
    }
    EdgeSplit {
        kept,
        strong_count,
        weak_count,
        dropped_weak,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(rel: &str) -> NeighborhoodEdge {
        NeighborhoodEdge {
            id: "e".into(),
            source: "A".into(),
            target: "B".into(),
            relation_type: rel.into(),
            weight: 1.0,
        }
    }

    #[test]
    fn drops_related_to_when_weak_off() {
        let edges = vec![edge("WORKS_AT"), edge("RELATED_TO")];
        let s = split_edges(&edges, false);
        assert_eq!(s.strong_count, 1);
        assert_eq!(s.weak_count, 1);
        assert_eq!(s.dropped_weak, 1);
        assert_eq!(s.kept.len(), 1);
    }
}
