# 07 — Implementation plan

Parent: [README](README.md) · Prev: [06 Edge cases](06-edge-cases.md) · Next: [08 E2E test matrix](08-e2e-test-matrix.md)

This file is the work order. Follow the waves in order. Do one step. Then run the named test.

Language note: sentences are short on purpose (ASD-STE100 style). Technical names stay as written in the code.

## Glossary

| Term | Meaning in this pack |
|------|----------------------|
| control profile | The default tool list. It includes write tools. |
| query profile | A lockdown. Set `EDGEQUAKE_MCP_PROFILE=query`. Write tools are absent. |
| track_id | The task key. `DocumentAdmissionAccepted.task_id` is the same string as `track_id` (`document_admission.rs` line 267). |
| ImageContent | An MCP content block. `type` is `image`. `data` is standard Base64. `mimeType` is `image/png` or `image/jpeg`. |
| upload handle | The `upload_id` string. The server stores the bytes. The client sends the id on the next call. |

## Stop. Do not do these things.

1. Do not call an Axum handler from `mcp/project/`.
2. Do not call `read_mm_asset_payload_by_id`. It is private.
3. Do not call `resolve_upload_content` for a PDF. That function rejects `.pdf` (`file_validation.rs`).
4. Do not put image bytes in `structuredContent`. The budget is 8, 24, or 80 KiB.
5. Do not use `URL_SAFE_NO_PAD` for image data. Use `base64::engine::general_purpose::STANDARD`.
6. Do not set `EDGEQUAKE_MCP_PROFILE` inside a parallel test. The variable is process-global.
7. Do not leave a new write tool in the `_` arm of `required_scope_for_tool`. That arm returns read scope.
8. Do not report `status: queued`. REST uses `pending`.
9. Do not reject `max_hops: 3` on a standard budget. `eq_neighborhood` clamps 3 to 2.
10. Do not delete the word `evidence` from instructions. `spec152_mcp_conformance.rs` requires that word.

## Roadblocks and the fix

| ID | Block in the current code | Fix before you call it |
|----|---------------------------|------------------------|
| RB-161-1 | `delete_document` is an Axum handler (`delete/single.rs`). | Extract `enqueue_document_deletion` in W1. The handler and MCP both call it. |
| RB-161-2 | `read_mm_asset_payload_by_id` is private (`mm_assets.rs`). | Call `load_mm_asset_bytes_by_id` and `list_mm_asset_summaries_for_document` in `document_mm_asset_persist.rs`. |
| RB-161-3 | `resolve_upload_content` rejects PDF (SPEC-121). | For `.pdf`, call a new `admit_pdf_bytes`. Extract it from `upload_pdf_document` in W3. |
| RB-161-4 | `call_tool_result` writes only a text block (`summary.rs`). | Add `call_tool_result_with_image`. Keep `content[0]` as text. Put the image in `content[1]`. |
| RB-161-5 | `image` is not a direct dependency of `edgequake-api`. | Add `image = "0.25"` to `edgequake-api/Cargo.toml` in W5. |
| RB-161-6 | `required_scope_for_tool` `_` arm returns read scope (`oauth/scopes.rs`). | Add each write tool name to the write arm in the same commit as the tool. |
| RB-161-7 | `mcp_profile()` reads a process env var. | Add `tools_list_for(profile)`. Tests pass the enum. Do not mutate the env. |
| RB-161-8 | Accept status in REST is `pending`, not `queued` (`text_upload.rs`). | Return `pending` or `duplicate_processing`. |
| RB-161-9 | `eq_neighborhood` clamps hop 3 to 2 (`graph.rs`). It does not reject hop 3. | Copy that clamp into `eq_graph_image`. |
| RB-161-10 | `ErrorCode` has no `NotImplemented` or `UnsupportedMedia` (`errors.rs`). | Add both variants before W1 and W4. |

## Wave order

```text
W0 spec pack (done)
  |
  v
W1 text ingest, task poll, delete, profile
  |
  +-- W2 download and resources/read
  +-- W3 upload handle (after W1 ingest exists)
  +-- W4 asset image
  +-- W5 graph PNG
  |
  v
W6 instructions and stdio bridge
  |
  v
W7 full EC file
```

You may start W2, W4, and W5 after W1. Start W3 after W1. Start W6 after the tools exist. Start W7 last.

Write a failing test first. Then change the code. Then run the test.

## W1 — Make ingest, task poll, and delete true

### W1.1 Extract delete

1. Open `handlers/documents/delete/single.rs`.
2. Move the body of `delete_document` into `pub async fn enqueue_document_deletion`.
3. Give it `state`, `tenant_ctx`, and `document_id`.
4. Return `DeleteDocumentResponse`.
5. Make `delete_document` call that function.
6. Run the existing delete tests. They must stay green.

### W1.2 Error codes

1. Open `mcp/project/errors.rs`.
2. Add `NotImplemented` with text `eq/not_implemented`.
3. Add `UnsupportedMedia` with text `eq/unsupported_media`.

### W1.3 Text ingest

1. Create `mcp/project/ingest.rs`.
2. Copy the `DocumentAdmissionInput` build from `text_upload.rs` (`upload_document`).
3. Set `text_content` from the argument `content`.
4. Set `title` from the argument `title`. Use `Untitled` when title is absent.
5. Call `crate::validation::validate_content` first.
6. Call `admit_document_for_processing` with track prefix `upload`.
7. On `Accepted`, return `document_id`, `task_id` = `accepted.track_id`, and `status` = `pending`.
8. On `DuplicateProcessing`, return that `document_id` and `status` = `duplicate_processing`.
9. Do not create a random UUID.
10. Map empty content to `eq_error(ErrorCode::InvalidId, ...)`.

### W1.4 Task poll

1. Replace the stub `eq_task_get` in `dispatch.rs`.
2. Call `get_task_for_context(state, task_id, tenant_ctx)`.
3. On `ApiError::NotFound`, return `eq_error(ErrorCode::NotFound, ...)`.
4. On success, copy `task.status` into the envelope.

### W1.5 Document delete tool

1. In `eq_document_delete`, require `confirm` to be JSON boolean `true`.
2. If confirm is false or absent, return `eq/confirm_required`. Do not call delete.
3. If confirm is true, call `enqueue_document_deletion`.
4. Copy `accepted`, `deleted`, and `track_id` into the envelope.
5. `deleted` stays false until the task ends. That matches `DeleteDocumentResponse`.

### W1.6 Workspace delete

1. If confirm is not true, return `eq/confirm_required`.
2. If confirm is true, return `eq/not_implemented` and `isError: true`.
3. Do not delete a workspace.

### W1.7 Profile

1. In `profile.rs`, add `McpProfile::Control`.
2. Map unset, `control`, and `memory` to `Control`.
3. Map `query` to `Query`.
4. In `tool_validation.rs`, reject write tools only when the profile is `Query`.
5. In `tools.rs` and `config.rs`, show write tools when the profile is `Control`.
6. Add `tools_list_for(profile: McpProfile)` for tests.
7. Keep `tools_list_result()` as a call to `tools_list_for(mcp_profile())`.

### W1.8 Scope

1. Open `oauth/scopes.rs`.
2. Keep `eq_ingest`, `eq_document_delete`, and `eq_workspace_delete` on write scope.
3. Do not add upload tool names yet. That is W3.

### W1.9 Tests

Run these tests. See [08](08-e2e-test-matrix.md).

- T-161-01, T-161-02, T-161-04, T-161-05, T-161-06, T-161-07, T-161-08, T-161-09, T-161-10
- T-161-11, T-161-21, T-161-22, T-161-23, T-161-24, T-161-25, T-161-26, T-161-27
- T-161-41, T-161-42

Also run:

```text
cargo test -p edgequake-api --test spec152_mcp_conformance --test spec028_mcp_e2e
```

### W1 exit

A text call to `eq_ingest` returns a real `document_id`. `eq_document_get` can see that id. `confirm: false` does not delete it.

## W2 — Download and resource read

1. Create `mcp/project/download.rs`.
2. Read original bytes the same way as `download_document_original` (`download.rs`).
3. Read markdown the same way as `download_document_markdown`.
4. Do not call the Axum functions. Copy the storage calls into a shared function if the handler is the only caller.
5. Put metadata in `structuredContent` (`byte_length`, `media_type`, `sha256`, `offset`, `next_offset`).
6. Put bytes in `content[1]` as a resource blob. Use standard Base64.
7. If the blob is larger than `EDGEQUAKE_MCP_BLOB_MAX_BYTES` (default 4194304), return one chunk and `next_offset`.
8. Take `media_type` from the stored row. Ignore a client type.
9. Update `resources.rs` so `.../text` and `.../original` return real bytes or `eq/not_found`.
10. Delete the placeholder note for those two URIs.

Tests: T-161-43, T-161-44, T-161-47, T-161-48, T-161-49, T-161-50, T-161-51.

Scope: `eq_document_download` stays on the read arm. The `_` arm already returns read. Add the name to the read arm anyway. Do not rely on `_`.

## W3 — Upload handle

### W3.1 PDF function

1. Open `handlers/pdf_upload/upload.rs`.
2. After the multipart loop, the handler has file bytes.
3. Move that byte path into `pub async fn admit_pdf_bytes(state, tenant_ctx, filename, bytes)`.
4. Make `upload_pdf_document` call `admit_pdf_bytes`.
5. Run the existing PDF upload tests.

### W3.2 Session store

1. Add a migration for `mcp_upload_sessions`.
2. Store `upload_id`, `tenant_id`, `workspace_id`, `user_id`, `filename`, `media_type`, `expected_sha256`, `expected_bytes`, `received_bytes`, `status`, `expires_at`.
3. Store chunk bytes in a temp file under a workspace directory. Do not store the PNG graph.
4. Reject a handle when tenant or workspace does not match the caller.

### W3.3 Tools

1. Add `eq_upload_begin`, `eq_upload_write`, `eq_upload_commit`, and `eq_upload_abort`.
2. In the same commit, add those four names to the write arm in `scopes.rs`.
3. `eq_upload_write` accepts only the next contiguous offset.
4. Reject a JSON body larger than `MCP_MAX_BODY_BYTES` (1048576). The gateway already does this in `body.rs`. Add a test that a huge chunk gets 413.
5. On commit, if the name ends with `.pdf`, call `admit_pdf_bytes`.
6. On commit, for other names, call `resolve_upload_content`, then `admit_document_for_processing`, then `persist_uploaded_original`.
7. `eq_ingest` with `content_base64` uses the same commit function after decode.
8. Reject a call that sets both `content` and `content_base64`.
9. A second commit of the same finished handle returns the same `document_id`.

Tests: T-161-03, T-161-12 through T-161-20, T-161-52.

## W4 — Illustration image

1. Call `list_mm_asset_summaries_for_document` when `eq_document_get` include contains `assets`.
2. Return id, kind, page, media type, and byte length. Do not return pixels.
3. Create `mcp/project/assets.rs`.
4. Call `load_mm_asset_bytes_by_id` from `eq_asset_get`.
5. If the type is not `image/png` or `image/jpeg`, return `eq/unsupported_media`.
6. If the byte length is above the blob cap, shrink the image with the `image` crate. Then return ImageContent.
7. Use `call_tool_result_with_image`.
8. Add `eq_asset_get` to the read arm in `scopes.rs`.

Tests: T-161-28 through T-161-32.

## W5 — Graph PNG

1. Add `image = "0.25"` to `edgequake-api/Cargo.toml`.
2. Create `mcp/project/graph_layout.rs`. This file does not open the database.
3. Create `mcp/project/graph_raster.rs`. This file does not query the graph.
4. Put colors and sizes in `mcp/project/graph_theme.rs`. Use the values in [lenses/LENS-front.md](lenses/LENS-front.md).
5. `eq_graph_image` calls `build_entity_neighborhood` the same way as `eq_neighborhood`.
6. Copy the hop clamp from `eq_neighborhood`. Hop 3 on a standard budget becomes 2.
7. Copy the artifact filter and the weak-edge filter from `eq_neighborhood`.
8. Place the focus node at the canvas center.
9. Return PNG bytes as ImageContent.
10. A unit test reads the focus coordinates. A 5 percent tolerance of the canvas is enough.

Tests: T-161-33 through T-161-40.

## W6 — Instructions and the stdio bridge

1. Write control instructions in `profile.rs`. Keep the word `evidence`.
2. Do not write `query-only` in the control text.
3. Write `query-only` only in the query-profile text.
4. In `mcp/src/server.ts`, register the new tool names as a forward to `POST /mcp`.
5. Do not admit, delete, or draw inside the TypeScript package.
6. Add a test that Rust `tools/list` names match `specs/152-new-mcp-contract/schemas/tools.catalog.json`.

Tests: T-161-45. Re-run `spec028_mcp_e2e` and `spec152_mcp_conformance`.

## W7 — Full matrix

1. Put every T-161 test in `edgequake/crates/edgequake-api/tests/spec161_mcp_control_e2e.rs`.
2. Include T-161-46. The search result has `retrieval_id` and `hits`. It has no essay field.
3. Run `python3 specs/161-improve-mcp/scripts/validate-cross-ref.py`.

## Done when all of these are true

1. `eq_ingest` of text creates a document that `eq_document_get` returns.
2. `confirm: false` leaves the document in place.
3. `confirm: true` returns `accepted: true`, `deleted: false`, and a `track_id`.
4. `eq_task_get` on that `track_id` returns the task status.
5. `eq_asset_get` returns an image block for PNG or JPEG.
6. `eq_graph_image` places the named node near the center.
7. `eq_search` still returns `retrieval_id`.
8. Unset profile lists write tools. Query profile does not.
