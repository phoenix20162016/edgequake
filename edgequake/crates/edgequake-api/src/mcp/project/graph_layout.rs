//! Deterministic graph layout. Does not open the database.

use super::graph_theme::{CANVAS_H, CANVAS_W, HOP1_FRAC, HOP2_FRAC, LABEL_MAX};

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutNode {
    pub id: String,
    pub hops: u32,
    pub x: f32,
    pub y: f32,
    pub is_focus: bool,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutEdge {
    pub source: String,
    pub target: String,
}

pub fn canvas_center() -> (f32, f32) {
    (CANVAS_W as f32 / 2.0, CANVAS_H as f32 / 2.0)
}

/// Place the focus at the center. Place other nodes on hop rings, sorted by id.
pub fn layout_graph(
    focus_id: &str,
    mut nodes: Vec<(String, u32, String)>,
    edges: Vec<(String, String)>,
) -> (Vec<LayoutNode>, Vec<LayoutEdge>) {
    nodes.sort_by(|a, b| a.0.cmp(&b.0));
    let (cx, cy) = canvas_center();
    let min_side = CANVAS_W.min(CANVAS_H) as f32;
    let r1 = (min_side / 2.0) * HOP1_FRAC;
    let r2 = (min_side / 2.0) * HOP2_FRAC;

    let mut laid = Vec::new();
    let hop1: Vec<_> = nodes.iter().filter(|n| n.1 == 1).cloned().collect();
    let hop2: Vec<_> = nodes.iter().filter(|n| n.1 >= 2).cloned().collect();

    laid.push(LayoutNode {
        id: focus_id.to_string(),
        hops: 0,
        x: cx,
        y: cy,
        is_focus: true,
        label: truncate_label(
            &nodes
                .iter()
                .find(|n| n.0 == focus_id)
                .map(|n| n.2.clone())
                .unwrap_or_else(|| focus_id.to_string()),
        ),
    });

    place_ring(&mut laid, &hop1, 1, cx, cy, r1);
    place_ring(&mut laid, &hop2, 2, cx, cy, r2);

    let layout_edges = edges
        .into_iter()
        .map(|(s, t)| LayoutEdge {
            source: s,
            target: t,
        })
        .collect();
    (laid, layout_edges)
}

fn truncate_label(s: &str) -> String {
    let count = s.chars().count();
    if count <= LABEL_MAX {
        return s.to_string();
    }
    format!(
        "{}…",
        s.chars()
            .take(LABEL_MAX.saturating_sub(1))
            .collect::<String>()
    )
}

fn place_ring(
    out: &mut Vec<LayoutNode>,
    nodes: &[(String, u32, String)],
    hops: u32,
    cx: f32,
    cy: f32,
    radius: f32,
) {
    let n = nodes.len().max(1) as f32;
    for (i, (id, _, label)) in nodes.iter().enumerate() {
        if out.iter().any(|p| p.id == *id) {
            continue;
        }
        let angle = (i as f32) * (std::f32::consts::TAU / n) - std::f32::consts::FRAC_PI_2;
        out.push(LayoutNode {
            id: id.clone(),
            hops,
            x: cx + radius * angle.cos(),
            y: cy + radius * angle.sin(),
            is_focus: false,
            label: truncate_label(label),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_is_at_center() {
        let (nodes, _) = layout_graph("FOCUS", vec![("FOCUS".into(), 0, "Focus".into())], vec![]);
        let (cx, cy) = canvas_center();
        let f = nodes.iter().find(|n| n.is_focus).unwrap();
        let tol_x = CANVAS_W as f32 * 0.05;
        let tol_y = CANVAS_H as f32 * 0.05;
        assert!((f.x - cx).abs() < tol_x);
        assert!((f.y - cy).abs() < tol_y);
    }

    #[test]
    fn layout_is_deterministic() {
        let input = vec![
            ("FOCUS".into(), 0, "F".into()),
            ("B".into(), 1, "B".into()),
            ("A".into(), 1, "A".into()),
        ];
        let (a, _) = layout_graph("FOCUS", input.clone(), vec![("FOCUS".into(), "A".into())]);
        let (b, _) = layout_graph("FOCUS", input, vec![("FOCUS".into(), "A".into())]);
        assert_eq!(a, b);
    }
}
