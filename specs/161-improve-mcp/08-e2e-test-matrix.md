# 08 — E2E test matrix

Parent: [README](README.md) · Prev: [07 Implementation plan](07-implementation-plan.md) · Next: [09 Cross-ref](09-cross-ref.md)

Re-check this matrix after every edit to [06](06-edge-cases.md).

## Runners

| Runner | Path | Covers |
|--------|------|--------|
| Control e2e | `edgequake/crates/edgequake-api/tests/spec161_mcp_control_e2e.rs` | T-161-01…52 (HTTP `/mcp`) |
| Layout unit | `edgequake-api/src/mcp/project/graph_layout.rs` (or `#[cfg(test)]`) | T-161-36, T-161-40 |
| Stdio | `mcp/tests/server.test.ts` | T-161-45 |
| Prior suites | `spec028_mcp_*`, `spec152_mcp_conformance` | Must stay green; T-161-46 may wrap search assert |

Harness: extend `tests/common/spec028_mcp.rs` (`mcp_tools_call`, auth fixtures).

## Matrix

| Test | EC | Wave | Assert (one line) |
|------|-----|------|-------------------|
| T-161-01 | EC-161-01 | W1 | Empty content → `eq/invalid_id`; no new document |
| T-161-02 | EC-161-02 | W1 | Oversize → error; no admit |
| T-161-03 | EC-161-03 | W3 | content + content_base64 → `eq/invalid_id` |
| T-161-04 | EC-161-04 | W1 | No payload → `eq/invalid_id` |
| T-161-05 | EC-161-05 | W1 | Duplicate hash → same document_id as REST policy |
| T-161-06 | EC-161-06 | W1 | Foreign workspace_id → 403 |
| T-161-07 | EC-161-07 | W1 | Read-only key → 403 on ingest |
| T-161-08 | EC-161-08 | W1 | Profile=query → ingest rejected / not listed |
| T-161-09 | EC-161-09 | W1 | Unicode title stored |
| T-161-10 | EC-161-10 | W1 | Status is `pending`, not `queued` |
| T-161-11 | EC-161-11 | W1 | Unknown task → `eq/not_found` |
| T-161-12 | EC-161-12 | W3 | Out-of-order write rejected |
| T-161-13 | EC-161-13 | W3 | Gap rejected |
| T-161-14 | EC-161-14 | W3 | Expired upload_id → not_found |
| T-161-15 | EC-161-15 | W3 | Cross-workspace upload_id denied |
| T-161-16 | EC-161-16 | W3 | Over cap rejected |
| T-161-17 | EC-161-17 | W3 | Bad checksum → no document |
| T-161-18 | EC-161-18 | W3 | Second commit idempotent |
| T-161-19 | EC-161-19 | W3 | Abort then write → not_found |
| T-161-20 | EC-161-20 | W3 | `../evil.pdf` sanitized |
| T-161-21 | EC-161-21 | W1 | confirm false → confirm_required; doc remains |
| T-161-22 | EC-161-22 | W1 | confirm omitted → confirm_required |
| T-161-23 | EC-161-23 | W1 | Missing id → not_found; not deleted:true |
| T-161-24 | EC-161-24 | W1 | Second delete honest |
| T-161-25 | EC-161-25 | W1 | Delete while processing → accepted + task |
| T-161-26 | EC-161-26 | W1 | Shared entity follows SPEC-050 |
| T-161-27 | EC-161-27 | W1 | Shape: accepted, task_id, deleted:false |
| T-161-28 | EC-161-28 | W4 | Unknown asset → not_found |
| T-161-29 | EC-161-29 | W4 | Path traversal rejected |
| T-161-30 | EC-161-30 | W4 | Non image → unsupported_media |
| T-161-31 | EC-161-31 | W4 | Search hides artifact; asset_get returns image |
| T-161-32 | EC-161-32 | W4 | Oversized → downsampled ImageContent |
| T-161-33 | EC-161-33 | W5 | Unknown entity → not_found |
| T-161-34 | EC-161-34 | W5 | hops 3 on standard budget becomes 2 |
| T-161-35 | EC-161-35 | W5 | Solo focus PNG |
| T-161-36 | EC-161-36 | W5 | Focus within center tolerance |
| T-161-37 | EC-161-37 | W5 | Artifacts omitted by default |
| T-161-38 | EC-161-38 | W5 | Max-nodes truncation honest |
| T-161-39 | EC-161-39 | W5 | PNG magic bytes |
| T-161-40 | EC-161-40 | W5 | Deterministic layout coords |
| T-161-41 | EC-161-41 | W1 | Unset profile lists write tools |
| T-161-42 | EC-161-42 | W1 | workspace_delete → not_implemented |
| T-161-43 | EC-161-43 | W2 | resources/read text real |
| T-161-44 | EC-161-44 | W2 | resources/read original real or honest |
| T-161-45 | EC-161-45 | W6 | Stdio forwards control tools |
| T-161-46 | EC-161-46 | W7 | Search has retrieval_id + hits; no answer essay |
| T-161-47 | EC-161-47 | W2 | No original → not_found |
| T-161-48 | EC-161-48 | W2 | Large markdown → next_offset |
| T-161-49 | EC-161-49 | W2 | Cross-tenant denied |
| T-161-50 | EC-161-50 | W2 | Offset past end → empty end |
| T-161-51 | EC-161-51 | W2 | media_type from row |
| T-161-52 | EC-161-52 | W3 | Oversize write body → 413 / invalid |

## Coverage check

- EC count in [06](06-edge-cases.md): **52**
- Test count in this matrix: **52**
- Every LAW-161-1…9 appears in at least one EC (see [09](09-cross-ref.md))
