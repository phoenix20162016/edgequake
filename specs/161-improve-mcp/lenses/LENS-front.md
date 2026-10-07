# LENS — Front Designer

Parent: [README](../README.md) · Primary: [04-architecture](../04-architecture.md) · W5 in [07](../07-implementation-plan.md)

## Job

Define the **server-side** graph PNG visual system: canvas, focus ring, hop
rings, label truncation, background. Tokens live in one Rust theme module
consumed by `graph_layout` / `graph_raster`. This is not a React change.

## Findings

- Web UI uses Sigma + `@sigma/export-image` (F-161-05). Pixel match is a
  non-goal ([README](../README.md) non-goals).
- Layout must be deterministic for tests (EC-161-40).

## Decisions

| Token | Proposed default |
|-------|------------------|
| Canvas | 1024 × 768 px |
| Background | `#ffffff` |
| Focus fill | brand primary (match web token if available) |
| Focus ring | 3 px stroke, high contrast |
| Hop-1 radius | 28% of min(width,height)/2 |
| Hop-2 radius | 55% of min(width,height)/2 |
| Node radius | focus 14 px; others 10 px |
| Label | max 24 chars + ellipsis; 12 px sans |
| Edge | 1 px `#94a3b8` |

Module name (target): `mcp/project/graph_theme.rs` (or constants in
`graph_layout.rs` if under size rule).

Focus node is placed at canvas center. Tests use 5% tolerance (SP-161-2).

## Success measures

- EC-161-36 focus center tolerance passes.
- EC-161-39 PNG magic.
- EC-161-40 deterministic coordinates.
- No dependency on WebGL or headless Chrome.
