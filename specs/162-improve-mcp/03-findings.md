# 03 — Findings

Parent: [README](README.md) · Prev: [02 Code facts](02-code-facts.md) · Next: [04 Architecture](04-architecture.md)

| ID | Finding | File / symbol | Wave | STE |
|----|---------|---------------|------|-----|
| F-162-01 | Agent id from search is not a valid graph input (scoped store key). | `ids.rs` `resolve_entity_lookup`; `graph.rs`; `graph_image.rs` | W1 | D1 |
| F-162-02 | Search and graph use different fields for `agent_entity_id`. | `search.rs` `ent.name` vs `graph.rs` `node.id` | W1 | D1/D10 |
| F-162-03 | `eq_entity_search` ignores `document_ids`; always writes `[]`. | `graph.rs` | W2 | D2 |
| F-162-04 | Unknown document id is silent; empty filter returns ok + hits=[]. | `document_filter_resolver.rs`; `search.rs` | W2 | D3 |
| F-162-05 | Pattern matches title only; not a hard MCP intersection with ids. | `document_filter_resolver.rs` | W2 | D4 |
| F-162-06 | `include=text` returns URI, not a text page. | `catalog.rs` `eq_document_get` | W3 | D5 |
| F-162-07 | Download default cap 4 MiB; last page omits `next_offset` null. | `blob.rs`; `download.rs` | W3 | D6 |
| F-162-08 | Hyphen and underscore create two entities. | `entity_id.rs` normalizer; merger | W5 | D7 |
| F-162-09 | Unknown entity and no-edge entity share neighborhood shape. | `graph.rs` `eq_neighborhood` | W4 | D8 |
| F-162-10 | Graph image drops RELATED_TO without counts/hint. | `graph_image.rs` | W4 | D9 |
| F-162-11 | README / stdio legacy tool use `entity_name`. | `mcp/README.md`; `mcp/src/tools/graph.ts` | W6 | D10 |
| F-162-12 | Markdown text loader has no tenant check. | `download.rs` `load_markdown_bytes` | W3 | F-162-A |
| F-162-13 | `apply_budget` overwrites truncation; wrong cursor object. | `budget.rs` | W4 | F-162-B |
| F-162-14 | Relationship merge ignores entity fold remap. | `merger/relationship.rs` | W5 | F-162-C |
| F-162-15 | Neighborhood schema advertises unused `document_ids`/`edge_types`. | `gateway/tools.rs`; `graph.rs` | W4 | F-162-D |
| F-162-16 | MCP tests seed bare node ids; D1 never fails. | `spec161_mcp_control_e2e.rs` | W1 | — |

## Cross-ref

| Finding | Laws | ECs |
|---------|------|-----|
| F-162-01 | LAW-162-1, 2 | EC-162-01…08 |
| F-162-02 | LAW-162-1, 2 | EC-162-01, 09 |
| F-162-03 | LAW-162-4 | EC-162-20…24 |
| F-162-04 | LAW-162-4 | EC-162-20…22 |
| F-162-05 | LAW-162-4 | EC-162-25…27 |
| F-162-06 | LAW-162-5 | EC-162-30…34 |
| F-162-07 | LAW-162-5 | EC-162-35…39 |
| F-162-08 | LAW-162-6 | EC-162-45…50 |
| F-162-09 | LAW-162-3 | EC-162-10…14 |
| F-162-10 | LAW-162-3 | EC-162-15…19 |
| F-162-11 | LAW-162-1 | EC-162-51…52 |
| F-162-12 | LAW-162-4 | EC-162-40…41 |
| F-162-13 | LAW-162-5 | EC-162-42…44 |
| F-162-14 | LAW-162-6 | EC-162-48…49 |
| F-162-15 | LAW-162-4 | EC-162-14 |
| F-162-16 | LAW-162-1 | EC-162-01 |
