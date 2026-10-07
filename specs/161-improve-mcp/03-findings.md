# 03 — Findings

Parent: [README](README.md) · Prev: [02 Surfaces](02-surfaces.md) · Next: [04 Architecture](04-architecture.md)

Each finding is a code fact before implementation. Mitigations land in
[07](07-implementation-plan.md). Tests land in [08](08-e2e-test-matrix.md).

| ID | Finding | File / symbol | Wave |
|----|---------|---------------|------|
| F-161-01 | `eq_ingest` ignores `state` / content and returns a random `document_id` with `status: queued`. | `gateway/dispatch.rs` `eq_ingest` (`let _ = (state, tenant_ctx, &body, &title)`) | W1 |
| F-161-02 | `eq_document_delete` returns `deleted: true` without calling the delete path. | `project/catalog.rs` `eq_document_delete` | W1 |
| F-161-03 | `eq_task_get` returns `status: unknown` for any id; does not call `get_task_for_context`. | `gateway/dispatch.rs` `eq_task_get` | W1 |
| F-161-04 | No MCP tool returns illustration bytes; figure REST returns caption text, mm-assets are REST-only. | gap vs `mm_assets.rs` `download_document_asset_by_id` | W4 |
| F-161-05 | No server graph PNG; only browser Sigma export. | gap vs `export.ts`; `eq_neighborhood` is JSON | W5 |
| F-161-06 | `MCP_MAX_BODY_BYTES` = 1 MiB blocks whole-file PDF upload in one JSON-RPC call. | `gateway/body.rs` | W3 |
| F-161-07 | Default profile is `query`; write tools hidden unless `EDGEQUAKE_MCP_PROFILE=memory`. | `project/profile.rs` `mcp_profile` | W1 |
| F-161-08 | `eq_workspace_delete` reports `deleted: true` without deleting. | `project/catalog.rs` `eq_workspace_delete` | W1 |
| F-161-09 | `resources/read` returns a placeholder JSON note for most `eq://` URIs. | `gateway/resources.rs` | W2 |
| F-161-10 | Stdio `@edgequake/mcp-server` registers only the ten read tools. It does not forward write tools. | `mcp/src/server.ts` | W6 |
| F-161-11 | `delete_document` is an Axum handler. There is no shared enqueue function yet. | `delete/single.rs` `delete_document` | W1 |
| F-161-12 | PDF bytes must not use `resolve_upload_content`. SPEC-121 rejects `.pdf` on that path. | `file_validation.rs` `validate_extension` | W3 |
| F-161-13 | `call_tool_result` emits only a text content block. | `project/summary.rs` | W2 |
| F-161-14 | New write tool names fall through to read scope in `required_scope_for_tool`. | `oauth/scopes.rs` `_` arm | W3 |
| F-161-15 | `image` is not a direct dependency of `edgequake-api`. | `edgequake-api/Cargo.toml` | W5 |
| F-161-16 | Hop 3 on a standard budget is clamped to 2, not rejected. | `project/graph.rs` `eq_neighborhood` | W5 |

## Severity notes

- F-161-01, F-161-02, F-161-08 are **honesty defects**: they violate LAW-161-1
  even if the memory profile is rarely enabled.
- F-161-07 is a **product advertisement** gap relative to the locked control-
  default decision. Execution trust (scope + confirm) remains.
- F-161-04, F-161-05 are missing jobs, not lies.
- F-161-06 is a protocol constraint to design around, not a bug to remove.

## Cross-ref

| Finding | Laws | ECs |
|---------|------|-----|
| F-161-01 | LAW-161-1 | EC-161-01…10 |
| F-161-02 | LAW-161-1, 8 | EC-161-21…27 |
| F-161-03 | LAW-161-1, 8 | EC-161-11, EC-161-25 |
| F-161-04 | LAW-161-2, 7 | EC-161-28…32 |
| F-161-05 | LAW-161-4 | EC-161-33…40 |
| F-161-06 | LAW-161-3 | EC-161-12…20, EC-161-52 |
| F-161-07 | LAW-161-6 | EC-161-08, EC-161-41 |
| F-161-08 | LAW-161-1 | EC-161-42 |
| F-161-09 | LAW-161-2 | EC-161-43…44, EC-161-47…51 |
| F-161-10 | LAW-161-9 | EC-161-45 |
| F-161-11 | LAW-161-1 | EC-161-21…27 |
| F-161-12 | LAW-161-3 | EC-161-16, EC-161-20 |
| F-161-13 | LAW-161-2 | EC-161-32, EC-161-39 |
| F-161-14 | LAW-161-6 | EC-161-07 |
| F-161-15 | LAW-161-4 | EC-161-39 |
| F-161-16 | LAW-161-4 | EC-161-34 |
