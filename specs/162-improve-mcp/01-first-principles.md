# 01 — First principles

Parent: [README](README.md) · Prev: [00 WHY](00-why.md) · Next: [02 Code facts](02-code-facts.md)

A first principle is a fact that does not depend on another rule.

| Fact | Statement |
|------|-----------|
| FACT-1 | An agent holds only handles. A handle is a value. Emit must equal accept. |
| FACT-2 | Storage keys and agent ids are different forms of one identity. |
| FACT-3 | Absence and emptiness are different states. |
| FACT-4 | A filter that does not reject is not a filter. |
| FACT-5 | Tokens and bytes have size limits. A page needs a typed cursor. |
| FACT-6 | Identity is decided once, at write time. |

## Laws

| ID | Law | Derives from | Extends |
|----|-----|--------------|---------|
| LAW-162-1 | Every tool that returns an entity uses `ent:{workspace_id}:{slug}`. Every tool that accepts an entity accepts that form. | FACT-1 | LAW-152 object model |
| LAW-162-2 | One function resolves an agent id to a storage node. Graph tools call that function. Do not load a node with a slug when the store key differs. | FACT-2 | REST `resolve_entity_node_exact` |
| LAW-162-3 | Return `eq/not_found` only when the entity is absent. A real entity with zero edges returns the center and `edge_count=0`. | FACT-3 | — |
| LAW-162-4 | `document_ids` and `document_pattern` are hard filters on MCP. Unknown ids are errors. Empty results name the filter outcome. | FACT-4 | SPEC-005 / SPEC-031 |
| LAW-162-5 | Text and download are paged. Cursors have form `{object}:{offset}`. Object matches what was omitted. | FACT-5 | LAW-161-2 |
| LAW-162-6 | Before a write, search for the fold-equivalent slug. If it exists, add evidence. Do not create a second entity. | FACT-6 | `EntityId` SSOT |

## DRY rules

1. One agent id builder: `agent_id_for_node(ws, &GraphNode)`.
2. One resolver: `resolve_entity_node` in `mcp/project/entity_ref.rs`.
3. One fold key: `fold_slug_key` in `edgequake-storage` (compare only).
4. One document scope resolver: `DocumentScope` for search and entity_search.
5. One tenant-checked text loader: `doc_text.rs` for get, download, resources/read.
6. One edge split: `edge_filter.rs` for neighborhood and graph_image.
7. One key resolver at ingest: `EntityKeyResolver` for entity and relationship merge.

## SOLID rules

| Letter | Rule | Where |
|--------|------|-------|
| S | `entity_ref`, `doc_scope`, `doc_text`, `cursor`, `edge_filter`, `key_resolver` each have one reason to change. | [04](04-architecture.md) |
| O | New id prefixes extend parse; storage keys stay. | [04](04-architecture.md) |
| L | Graph tools return the SPEC-152 envelope. Callers cannot tell REST vs MCP. | LAW-162-1 |
| I | Layout/raster do not resolve ids. Resolver does not raster. | [04](04-architecture.md) |
| D | Project modules depend on `GraphStorage` / loaders, not Axum handlers. | [07](07-implementation-plan.md) |

## Size rule

Keep each new Rust file under 300 lines where possible.
