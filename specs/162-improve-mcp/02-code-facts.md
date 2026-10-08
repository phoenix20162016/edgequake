# 02 — Code facts

Parent: [README](README.md) · Prev: [01 First principles](01-first-principles.md) · Next: [03 Findings](03-findings.md)

Facts at revision `59ec526`. Do not treat them as opinions.

## Issue claims that match the code

| Claim | Symbol |
|-------|--------|
| `agent_entity_id` → `ent:{ws}:{slug}` | `mcp/project/ids.rs` |
| `resolve_entity_lookup` keeps text after second colon; uppercases; replaces space with `_`; does not replace `-` | `ids.rs` |
| `eq_entity_search` sets `document_ids: []` and ignores the argument | `graph.rs` |
| `eq_entity_get` / `eq_neighborhood` / `eq_graph_image` use `resolve_entity_lookup` then load by that string | `graph.rs`, `graph_image.rs` |
| `eq_neighborhood` does not test center exists; empty lists on miss; DEFAULT_NEIGHBOR_CAP=16; drops RELATED_TO | `graph.rs` |
| Search builds entity hit with `agent_entity_id(workspace, &ent.name)`; graph uses `node.id` | `search.rs`, `graph.rs` |
| `eq_document_get include=text` returns `text_resource` URI only | `catalog.rs`, `gateway/tools.rs` |
| README shows `graph_entity_neighborhood` with `entity_name` | `mcp/README.md`, `mcp/src/tools/graph.ts` |

## Corrections (issue claim ≠ code)

| Topic | Issue said | Code fact |
|-------|------------|-----------|
| D1 root cause | Slug vs name field mismatch only | Store keys are `{ws_uuid}::NAME` via `EntityId::graph_node_id_for_workspace`. REST uses `exact_lookup_candidates`. MCP skips it. |
| Tests hide D1 | — | `t161_33_35_39_graph_png` seeds bare `"FOCUS_NODE"`. |
| R4 hyphen | Change normalizer | `normalize_entity_name` keeps hyphens (`NEW-YORK`). Fold is compare-only. |
| D3/D4 | Filter never applied | Engine hard-filters via `allowed_document_ids` (`context_filter.rs`). Gaps: OR union; pattern title-only; unknown ids silent; singular `document_id` on hits. |
| D6 | No paging | `chunk_blob_envelope` already pages. Default cap 4 MiB; `next_offset` absent (not null) on last page. |
| Legacy tool | In gateway | `graph_entity_neighborhood` is stdio-only (`mcp/src/tools/graph.ts`). |

## Extra findings from audit

| ID | Fact |
|----|------|
| F-162-A | `load_markdown_bytes` / `read_text_resource` skip tenant check. |
| F-162-B | `apply_budget` overwrites `truncation` and emits `chunks:{kept}` for any omit. |
| F-162-C | Relationship merge keys endpoints independently of entity fold remap. |
| F-162-D | `eq_neighborhood` schema advertises `document_ids` / `edge_types`; code ignores both. |

## Existing reuse (do not reinvent)

- `EntityId::exact_lookup_candidates` — `edgequake-storage/src/entity_id.rs`
- `resolve_entity_node_exact` — `handlers/entities/mod.rs`
- `filter_context_by_document_ids` — `edgequake-query/src/context_filter.rs`
- `chunk_blob_envelope` — `mcp/project/download.rs`
- Worker harness — `tests/common/mod.rs` `create_test_app_with_extraction_only`
