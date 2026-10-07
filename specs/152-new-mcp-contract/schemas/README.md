# EQ-MCP-1.0 JSON Schemas (SSOT)

Normative JSON Schema 2020-12 for tool `inputSchema` roots and the shared read envelope.

**Amended by SPEC-161** ([../161-improve-mcp/](../../161-improve-mcp/)): control profile
default, upload/download/asset/graph tools, `eq_ingest` payload variants,
`eq_document_get` `include: assets`. Document control delta: EQ-MCP-1.1.

| File | Role |
|------|------|
| [envelope.schema.json](envelope.schema.json) | Shared `structuredContent` skeleton + `$defs` |
| [eq_document_list.input.json](eq_document_list.input.json) | L1 |
| [eq_search.input.json](eq_search.input.json) | L1 |
| [eq_fetch.input.json](eq_fetch.input.json) | L1 |
| [eq_retrieve.input.json](eq_retrieve.input.json) | L1 |
| [eq_entity_search.input.json](eq_entity_search.input.json) | L1 |
| [eq_neighborhood.input.json](eq_neighborhood.input.json) | L1 |
| [eq_workspace_list.input.json](eq_workspace_list.input.json) | L2/catalog |
| [eq_workspace_stats.input.json](eq_workspace_stats.input.json) | L2/catalog |
| [eq_document_get.input.json](eq_document_get.input.json) | L2/catalog (+ `assets`) |
| [eq_entity_get.input.json](eq_entity_get.input.json) | L2 |
| [eq_ingest.input.json](eq_ingest.input.json) | L3 / control |
| [eq_upload_begin.input.json](eq_upload_begin.input.json) | L3 / control (SPEC-161) |
| [eq_upload_write.input.json](eq_upload_write.input.json) | L3 / control (SPEC-161) |
| [eq_upload_commit.input.json](eq_upload_commit.input.json) | L3 / control (SPEC-161) |
| [eq_upload_abort.input.json](eq_upload_abort.input.json) | L3 / control (SPEC-161) |
| [eq_document_download.input.json](eq_document_download.input.json) | L3 (SPEC-161) |
| [eq_asset_get.input.json](eq_asset_get.input.json) | L3 (SPEC-161) |
| [eq_graph_image.input.json](eq_graph_image.input.json) | L3 (SPEC-161) |
| [eq_task_get.input.json](eq_task_get.input.json) | L3 |
| [eq_document_delete.input.json](eq_document_delete.input.json) | L3 / control |
| [eq_workspace_delete.input.json](eq_workspace_delete.input.json) | L3 (not_implemented after confirm) |
| [tools.catalog.json](tools.catalog.json) | Name → schema + annotations index |

**Rules:**

1. Every input root MUST have `"additionalProperties": false`.
2. Rust `mcp/gateway/tools.rs` MUST match these files (conformance test).
3. Tool-specific `outputSchema` = envelope + tool fields from [04-tool-contract](../04-tool-contract.md); full per-tool output schemas MAY be added as `eq_*.output.json` in a follow-up without changing semantics.
4. Profile names in the catalog: `control` (default), `query` (lockdown), `memory` (alias of control). See [SPEC-161 05-contract-delta](../../161-improve-mcp/05-contract-delta.md).
