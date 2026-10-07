# 02 — Surfaces (as-is)

Parent: [README](README.md) · Prev: [01 First principles](01-first-principles.md) · Next: [03 Findings](03-findings.md)

This document maps the code as it exists before SPEC-161 implementation.
Baseline agent contract: [SPEC-152 01-current-surfaces](../152-new-mcp-contract/01-current-surfaces.md).
Transport: [SPEC-028 mcp/000-index](../028-edgequake-query-service/mcp/000-index.md).

## Gateway layout

| Path | Role |
|------|------|
| `edgequake/crates/edgequake-api/src/mcp/mod.rs` | Module root |
| `…/mcp/gateway/mod.rs` | `handle_mcp_request` |
| `…/mcp/gateway/tools.rs` | Tool catalog SSOT (`tools_list_result`) |
| `…/mcp/gateway/dispatch.rs` | `tools/call` routing; stub `eq_ingest` / `eq_task_get` |
| `…/mcp/gateway/tool_validation.rs` | Memory tools gated on profile |
| `…/mcp/gateway/resources.rs` | `eq://` list/read (mostly placeholder) |
| `…/mcp/gateway/body.rs` | `MCP_MAX_BODY_BYTES` = 1 MiB |
| `…/mcp/auth/gateway_auth.rs` | Bearer / API key, MCP audience |
| `…/mcp/project/profile.rs` | `query` default vs `memory` |
| `…/mcp/project/search.rs` | `eq_search` → `retrieve_context` |
| `…/mcp/project/catalog.rs` | list/get/delete stubs |
| `…/mcp/project/graph.rs` | entity search / get / neighborhood |
| `mcp/src/server.ts` | Stdio bridge; registers read tools only |

## Profile advertisement (today)

```text
EDGEQUAKE_MCP_PROFILE unset or "query"
  → tools/list: 10 eq_* read tools + 3 aliases
  → write tools rejected in validate_tool_call

EDGEQUAKE_MCP_PROFILE=memory
  → appends eq_ingest, eq_task_get, eq_document_delete, eq_workspace_delete
  → implementations are stubs (see findings)
```

Source: `mcp/project/profile.rs`, `gateway/tools.rs`, `gateway/tool_validation.rs`.

## Capability matrix

| Capability | REST | MCP today | Target (SPEC-161) |
|------------|------|-----------|-------------------|
| Inline text ingest | `POST /api/v1/documents` → `upload_document` → `admit_document_for_processing` | `eq_ingest` stub mints UUID | Real admit; control default |
| File upload | `POST /documents/upload` → `upload_file` | Not wrapped; 1 MiB body | `eq_upload_*` handle + small base64 |
| Download original / markdown | `GET …/download/original`, `…/markdown` | Metadata / `eq://` note only | `eq_document_download` + real resources |
| Delete document | `DELETE /documents/{id}` → 202 + deletion task | Stub `deleted: true` | Real accept + `task_id` |
| Task poll | `GET /tasks/{track_id}` | Stub `status: unknown` | `get_task_for_context` |
| Illustration PNG/JPEG | `GET …/assets/{asset_id}` → `load_mm_asset_bytes_by_id` (public). `read_mm_asset_payload_by_id` is private. | None | `eq_asset_get` ImageContent |
| Graph PNG centered on node | Browser Sigma export only | JSON `eq_neighborhood` | `eq_graph_image` layout+raster |
| Query search | `POST /query/context` | **Real** `eq_search` | Unchanged; regression only |
| Workspace delete | REST exists | Stub `deleted: true` | `eq/not_implemented` |

## REST service anchors (call these, not handlers)

| Job | Function / module |
|-----|-------------------|
| Admit text/file | `admit_document_for_processing` — `handlers/documents/upload/document_admission.rs` |
| Resolve file bytes | `resolve_upload_content` for non-PDF only. `.pdf` is rejected here (SPEC-121). Use `admit_pdf_bytes` after you extract it. |
| Persist original | `persist_uploaded_original` |
| Delete | Extract `enqueue_document_deletion` from `delete_document` in `handlers/documents/delete/single.rs`. MCP calls the extract. It does not call the Axum handler. |
| Task | `get_task_for_context` — `services/task_scope.rs` |
| MM asset bytes | `load_mm_asset_bytes_by_id` and `list_mm_asset_summaries_for_document` in `services/document_mm_asset_persist.rs`. Do not call the private helper in `mm_assets.rs`. |
| Neighborhood | `build_entity_neighborhood` — used by `mcp/project/graph.rs` |
| Search | `retrieve_context` — `services/query_context` via `mcp/project/search.rs` |

## Binary storage (read path)

| What | Table / store | Notes |
|------|---------------|-------|
| PDF blob | `pdf_documents.pdf_data` | Download original when `pdf_id` present |
| Non-PDF original | `document_originals.original_data` | Images, text files |
| Page / figure rasters | `document_mm_assets` (`asset_data`, `content_type`, `asset_id`, `asset_kind`) | Kinds: `page_full`, `page_chart_crop`, `embedded_figure`, `table_crop` |

## Graph image (today)

- Server: no raster. `eq_neighborhood` returns JSON.
- Client: `edgequake_webui/src/lib/graph/engine/export.ts` → `@sigma/export-image` `downloadAsPNG`.
- Camera focus: `use-graph-keyboard-navigation.ts` (`focusCameraOnNode`).

## Official protocol surface used

| Protocol piece | Use in SPEC-161 |
|----------------|-----------------|
| `tools/list`, `tools/call` | All control tools |
| `ImageContent` (`type: image`, base64, `mimeType`) | Illustrations, graph PNG |
| Resource blob contents | Original / markdown chunks |
| Tool annotations | `destructiveHint` on delete |
| Explicit handles (stateful tools guidance) | `upload_id` |
| `io.modelcontextprotocol/tasks` | **Not** implemented; poll `eq_task_get` |
| Elicitation / MRTR | **Not** implemented; `confirm` arg |

## Stdio package

`mcp/src/server.ts` registers the ten L1 read names and forwards to `POST /mcp`.
Legacy tools under `mcp/src/tools/document.ts` (`document_upload`, …) are
unwired. SPEC-161 keeps the bridge-only rule (LAW-161-9).
