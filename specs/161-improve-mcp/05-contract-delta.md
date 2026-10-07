# 05 — Contract delta (EQ-MCP-1.0 → 1.1)

Parent: [README](README.md) · Prev: [04 Architecture](04-architecture.md) · Next: [06 Edge cases](06-edge-cases.md)

Schema SSOT remains [../152-new-mcp-contract/schemas/](../152-new-mcp-contract/schemas/).
This document states what changes and what does not. Default row: **no semantic
change** unless listed.

## Profile amendment (normative)

| Topic | EQ-MCP-1.0 | EQ-MCP-1.1 (SPEC-161) |
|-------|------------|------------------------|
| Default profile | `query` (omit writes) | **control** (advertise writes) when env unset |
| Lockdown | N/A (was default) | `EDGEQUAKE_MCP_PROFILE=query` omits write tools |
| `memory` | Write profile | Alias of **control** |
| Write execution | `edgequake:write` | Unchanged |
| Delete confirm | `confirm: true` | Unchanged |
| Instructions line 8 | “query-only” on query profile | Control instructions: ingest → `eq_task_get`; deletes need confirm |

## Unchanged (do not reopen)

| Area | Reference |
|------|-----------|
| Envelope fields `ok`, `view`, `budget_used`, `truncation` | SPEC-152 `05-envelope-budget-errors` |
| Budgets 8 / 24 / 80 KiB on `structuredContent` | same |
| Summary text ≤2 KiB deviation | SPEC-152 `02-architecture` |
| Modes `naive\|local\|global\|hybrid\|mix` | SPEC-152 `06-retrieval-and-graph` |
| No `eq_answer` | LAW-161-5 / LAW-152 |
| OAuth / Streamable HTTP / PRM | SPEC-028 mcp suite |
| Aliases `edgequake_search\|fetch\|retrieve` | SPEC-152 |
| Artifact/DRAWING hidden unless `include_artifacts` | LAW-161-7 |

## Tool catalog delta

### Amended tools

| Tool | Change |
|------|--------|
| `eq_ingest` | Real admit. Inputs: `content` XOR (`content_base64`+`filename`+`media_type`) XOR `upload_ref`. Output: real `document_id`, `task_id` = `track_id`, `status` = `pending` or `duplicate_processing`. Profile: control. PDF does not use `content_base64` through `resolve_upload_content`. |
| `eq_task_get` | Real `get_task_for_context`. Unknown → `eq/not_found`. Profile: control (and query may keep it read-only for download jobs if advertised). |
| `eq_document_delete` | Real delete path. Output: `status: accepted`, `task_id`, `deleted: false` until task completes. Profile: control. |
| `eq_workspace_delete` | After `confirm: true` → `ok: false`, `eq/not_implemented`, `isError: true`. Never `deleted: true`. |
| `eq_document_get` | `include` gains `assets` (metadata only: id, kind, page, media_type, byte_length). |
| Instructions | Control vs query texts in `profile.rs`. |

### New tools

| Tool | Scope | Annotations | Job |
|------|-------|-------------|-----|
| `eq_upload_begin` | write | openWorld | Create `upload_id` handle |
| `eq_upload_write` | write | openWorld | Append contiguous chunk |
| `eq_upload_commit` | write | openWorld | Resolve + admit + persist original |
| `eq_upload_abort` | write | openWorld | Drop handle |
| `eq_document_download` | read | readOnly, idempotent | `representation`: `original` \| `markdown`; blob chunks |
| `eq_asset_get` | read | readOnly, idempotent | PNG/JPEG `ImageContent` by `document_id` + `asset_id` |
| `eq_graph_image` | read | readOnly, idempotent | PNG centered on `entity_id`. Hop 3 on standard budget clamps to 2. |

### Search

`eq_search` / `eq_fetch` / `eq_retrieve` — **no semantic change**. SPEC-161 adds
regression EC-161-46 that the result is evidence (`retrieval_id` + hits), not
an essay.

## Error codes (additions)

| Code | When |
|------|------|
| `eq/not_implemented` | `eq_workspace_delete` after confirm; reserved for other deferred writes |
| `eq/unsupported_media` | `eq_asset_get` when content type is not `image/png` or `image/jpeg` |

Existing codes (`eq/confirm_required`, `eq/not_found`, `eq/forbidden`,
`eq/invalid_id`, `eq/budget_exceeded`, …) stay as in SPEC-152.

## Resources

| URI template | Target behavior |
|--------------|-----------------|
| `eq://{ws}/documents/{id}/text` | Real markdown/text or honest size error |
| `eq://{ws}/documents/{id}/original` | Blob resource or chunking error pointing at `eq_document_download` |
| Other templates | Unchanged hydrate guidance |

Placeholder `{"ok":true,"note":"hydrate via…"}` is removed for text/original
(F-161-09).

## Annotations matrix (new / changed)

| Tool | readOnly | idempotent | destructive | openWorld |
|------|----------|------------|-------------|-----------|
| `eq_upload_begin` | F | F | F | T |
| `eq_upload_write` | F | F | F | T |
| `eq_upload_commit` | F | F | F | T |
| `eq_upload_abort` | F | T | F | F |
| `eq_document_download` | T | T | F | F |
| `eq_asset_get` | T | T | F | F |
| `eq_graph_image` | T | T | F | F |
| `eq_document_delete` | F | F | T | F |
| `eq_ingest` | F | F | F | T |

## Conformance level note

SPEC-152 L3 “Memory” becomes the default control surface for advertisement.
L1/L2 read conformance is unchanged. A deployment that sets
`EDGEQUAKE_MCP_PROFILE=query` remains an L1/L2 read connector and MUST say so
in `instructions`.
