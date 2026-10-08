# 05 — Contract delta (EQ-MCP-1.2)

Parent: [README](README.md) · Prev: [04 Architecture](04-architecture.md) · Next: [06 Edge cases](06-edge-cases.md)

Schema SSOT: `specs/152-new-mcp-contract/schemas/`. Rust `gateway/tools.rs` must match.

## Entity id

| Tool | Change |
|------|--------|
| All entity emitters | `id = agent_id_for_node(ws, storage_id)` |
| `eq_entity_get`, `eq_neighborhood`, `eq_graph_image` | Call `resolve_entity_node`. Not-found text contains agent id. If lookup changed the id, include storage id. |
| `eq_entity_search` | Emit real `document_ids` from node `source_document_ids`. Read input `document_ids`. |

## Search / scope

| Field | Behavior |
|-------|----------|
| `document_ids` | Hard. Unknown id → `eq/not_found`. Empty result with valid ids → `hits=[]`, `filter_result:"empty"`. |
| `document_pattern` | Hard on `file_name` and `title` (case-insensitive, comma OR). No match → `documents_considered=[]`, text `"No document matches the pattern."`, `filter_result:"no_match"`. |
| ids ∩ pattern | MCP intersection. REST union unchanged. |
| `documents_considered` | Sorted, limited to resolved scope. |
| Entity hit | `document_ids: string[]` (plural). |

## Neighborhood

| Field | Behavior |
|-------|----------|
| Center | Always present when entity exists. |
| `edge_count` | Integer; 0 is valid. |
| `strong_edge_count`, `weak_edge_count` | Counts before drop. |
| `include_weak_edges=false` | Drop RELATED_TO; set hint when any weak existed. |
| Cap | Center kept; neighbors truncated to 16; `truncation.omitted_entities`. |
| Unused schema args | Honor `document_ids`/`edge_types` or remove from schema (prefer honor). |

## Graph image

| Field | Behavior |
|-------|----------|
| `node_count` | Laid-out nodes (includes center). |
| `strong_edge_count`, `weak_edge_count` | Required. |
| Hint | Exactly: `Set include_weak_edges to true to show RELATED_TO edges.` when weak hidden and weak_count>0. |
| Truncation | `omitted_entities` equals omitted count. |

## Document get / download

| Tool | Change |
|------|--------|
| `eq_document_get` `include=text` | `text_page{text,offset,limit,total_chars,next_offset}`. Args `text_offset`, `text_limit` (default 8000, max 32000). `text_resource` URI may remain. |
| `eq_document_download` | Cap 65536. Always `offset`, `chunk_length`, `byte_length`, `next_offset` (`null` on last). `max_bytes=0` → `eq/invalid_id`. |

## Cursor

Form `{entities|relationships|chunks|bytes}:{offset}`. Object matches omitted class. Catalog `p:` cursors unchanged.

## Legacy stdio

`graph_entity_neighborhood` / `graph_get_entity` accept `entity_id`; still accept `entity_name`. README shows `entity_id`.

## Ingest

Fold-equivalent names merge into one entity. Relationships follow remapped keys.
