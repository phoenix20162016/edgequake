# 06 — Edge cases

Parent: [README](README.md) · Prev: [05 Contract delta](05-contract-delta.md) · Next: [07 Implementation plan](07-implementation-plan.md)

Every EC names a law, a mitigation, a test id, and a wave. One gate per EC in
[08](08-e2e-test-matrix.md). IDs are contiguous EC-161-01 … EC-161-52.

## Ingest — EC-161-01 … 10

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-01 | Empty `content` string | LAW-161-1 | `eq/invalid_id`; no document row | T-161-01 | W1 |
| EC-161-02 | Content over `max_document_size` | LAW-161-1 | Same validation as REST; no admit | T-161-02 | W1 |
| EC-161-03 | Both `content` and `content_base64` | LAW-161-1 | `eq/invalid_id` | T-161-03 | W3 |
| EC-161-04 | Neither content nor upload_ref nor base64 | LAW-161-1 | `eq/invalid_id` | T-161-04 | W1 |
| EC-161-05 | Duplicate content hash | LAW-161-1 | Return existing document outcome (same as REST) | T-161-05 | W1 |
| EC-161-06 | Foreign `workspace_id` vs claim | LAW-161-6 | 403 workspace mismatch | T-161-06 | W1 |
| EC-161-07 | Read-only role / no write scope | LAW-161-6 | 403 `insufficient_scope` | T-161-07 | W1 |
| EC-161-08 | Query profile lockdown calls ingest | LAW-161-6 | Tool absent or reject “requires control” | T-161-08 | W1 |
| EC-161-09 | Unicode title | LAW-161-1 | Admit succeeds; title stored | T-161-09 | W1 |
| EC-161-10 | Client implies sync processing | LAW-161-1 | Still queues. Status is `pending`, same as REST. | T-161-10 | W1 |

## Task poll — EC-161-11

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-11 | `eq_task_get` unknown id | LAW-161-1 | `eq/not_found` (never `status: unknown` with `ok: true`) | T-161-11 | W1 |

## Upload session — EC-161-12 … 20

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-12 | Out-of-order chunk offset | LAW-161-3 | Reject; session unchanged | T-161-12 | W3 |
| EC-161-13 | Gap in offsets | LAW-161-3 | Reject until contiguous | T-161-13 | W3 |
| EC-161-14 | Expired TTL | LAW-161-3 | `eq/not_found` | T-161-14 | W3 |
| EC-161-15 | Other workspace’s `upload_id` | LAW-161-6 | `eq/forbidden` or not_found | T-161-15 | W3 |
| EC-161-16 | Total over REST upload cap | LAW-161-3 | Reject write/commit | T-161-16 | W3 |
| EC-161-17 | Checksum mismatch on commit | LAW-161-1 | `eq/invalid_id`; no document | T-161-17 | W3 |
| EC-161-18 | Commit twice | LAW-161-1 | Idempotent second success | T-161-18 | W3 |
| EC-161-19 | Abort then write | LAW-161-3 | `eq/not_found` | T-161-19 | W3 |
| EC-161-20 | Path in `filename` | LAW-161-6 | `sanitize_filename` | T-161-20 | W3 |

## Delete — EC-161-21 … 27

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-21 | `confirm: false` | LAW-161-6 | `eq/confirm_required`; document remains | T-161-21 | W1 |
| EC-161-22 | `confirm` omitted | LAW-161-6 | Same as false | T-161-22 | W1 |
| EC-161-23 | `confirm: true` on missing id | LAW-161-1 | `eq/not_found`; never `deleted: true` | T-161-23 | W1 |
| EC-161-24 | Second delete after accept | LAW-161-8 | Honest not_found or already-deleting status | T-161-24 | W1 |
| EC-161-25 | Delete while processing | LAW-161-8 | Accepted task; `eq_task_get` tracks | T-161-25 | W1 |
| EC-161-26 | Shared-entity cascade | LAW-161-8 | Inherit SPEC-050 behavior | T-161-26 | W1 |
| EC-161-27 | Response shape | LAW-161-8 | `status: accepted`, `task_id`, `deleted: false` | T-161-27 | W1 |

## Assets — EC-161-28 … 32

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-28 | Unknown `asset_id` | LAW-161-7 | `eq/not_found` | T-161-28 | W4 |
| EC-161-29 | Path traversal on asset path | LAW-161-6 | Reject (existing handler test) | T-161-29 | W4 |
| EC-161-30 | Non png/jpeg content type | LAW-161-2 | `eq/unsupported_media` + metadata | T-161-30 | W4 |
| EC-161-31 | Artifact hidden from search, fetchable by id | LAW-161-7 | `eq_search` omits; `eq_asset_get` returns image | T-161-31 | W4 |
| EC-161-32 | Over `EDGEQUAKE_MCP_BLOB_MAX_BYTES` | LAW-161-2 | Downsample illustration; still ImageContent | T-161-32 | W4 |

## Graph PNG — EC-161-33 … 40

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-33 | Unknown `entity_id` | LAW-161-4 | `eq/not_found` | T-161-33 | W5 |
| EC-161-34 | `max_hops: 3` on standard budget | LAW-161-4 | Clamp to 2. Same as `eq_neighborhood` in `graph.rs`. Do not reject. | T-161-34 | W5 |
| EC-161-35 | Empty neighborhood | LAW-161-4 | PNG of focus node alone | T-161-35 | W5 |
| EC-161-36 | Focus near canvas center | LAW-161-4 | Layout unit test tolerance | T-161-36 | W5 |
| EC-161-37 | Artifacts omitted by default | LAW-161-7 | No artifact nodes unless flag | T-161-37 | W5 |
| EC-161-38 | Max-nodes cap | LAW-161-4 | Truncation honest in envelope | T-161-38 | W5 |
| EC-161-39 | PNG signature | LAW-161-2 | Bytes start with `\x89PNG` | T-161-39 | W5 |
| EC-161-40 | Deterministic layout | LAW-161-4 | Two calls → same coordinates | T-161-40 | W5 |

## Profile / workspace / resources / stdio / search — EC-161-41 … 46

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-41 | Control default `tools/list` includes write tools | LAW-161-6 | Unset env advertises ingest/delete | T-161-41 | W1 |
| EC-161-42 | `eq_workspace_delete` with confirm | LAW-161-1 | `eq/not_implemented`; workspace remains | T-161-42 | W1 |
| EC-161-43 | `resources/read` document text | LAW-161-2 | Real text or honest size error | T-161-43 | W2 |
| EC-161-44 | `resources/read` original | LAW-161-2 | Blob or pointer to download tool | T-161-44 | W2 |
| EC-161-45 | Stdio forwards new tools | LAW-161-9 | Bridge registers / forwards; no local admit | T-161-45 | W6 |
| EC-161-46 | Search returns evidence only | LAW-161-5 | `retrieval_id` + hits; no essay field | T-161-46 | W7 |

## Download — EC-161-47 … 51

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-47 | No original row | LAW-161-1 | `eq/not_found` | T-161-47 | W2 |
| EC-161-48 | Markdown larger than blob cap | LAW-161-2 | `next_offset` + chunk | T-161-48 | W2 |
| EC-161-49 | Cross-tenant document id | LAW-161-6 | `eq/not_found` or forbidden | T-161-49 | W2 |
| EC-161-50 | Offset past end | LAW-161-2 | Empty chunk + `truncation: false` end | T-161-50 | W2 |
| EC-161-51 | `media_type` from stored row | LAW-161-1 | Ignore client-supplied type | T-161-51 | W2 |

## Body cap — EC-161-52

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-161-52 | `eq_upload_write` decoded chunk keeps JSON-RPC body ≤ 1 MiB | LAW-161-3 | 413 or `eq/invalid_id` if over | T-161-52 | W3 |

## Shared-entity note

EC-161-26 cites [SPEC-050 pipeline-and-delete](../050-pipeline-and-delete/).
MCP does not redefine cascade rules; it calls the same delete service.
