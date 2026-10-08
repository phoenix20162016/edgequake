# LENS — UX / UI Designer

Parent: [README](../README.md) · Primary: [05-contract-delta](../05-contract-delta.md)

## Job

Agent-facing messages must name the state. Empty is not not-found. Hidden
edges must say why.

## Message contract

| State | Message shape |
|-------|---------------|
| Entity missing | `eq/not_found` + agent id in text |
| Entity exists, 0 edges | Center present, `edge_count=0` |
| Weak edges hidden | Exact hint: `Set include_weak_edges to true to show RELATED_TO edges.` |
| Pattern no match | `No document matches the pattern.` |
| Filter emptied hits | `filter_result: "empty"` |
| Unknown document id | `eq/not_found` for that id |

## Visual channel

Graph PNG is a second projection, not a second query. Counts in structured
content explain what the picture omitted.
