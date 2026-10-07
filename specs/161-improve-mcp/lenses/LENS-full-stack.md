# LENS — Full Stack Developer

Parent: [README](../README.md) · Primary: [02-surfaces](../02-surfaces.md) · [04-architecture](../04-architecture.md) · [07-implementation-plan](../07-implementation-plan.md)

## Job

Wire MCP tools to existing service functions. Keep dispatch thin. Do not call
Axum handlers from `mcp/project/`.

## Findings

- Stubs live in `dispatch.rs` and `catalog.rs` (F-161-01…03, F-161-08).
- Admission, delete, mm-assets, and retrieve already exist ([02](../02-surfaces.md)).
- Stdio must stay a bridge (LAW-161-9).

## Decisions

| Ingest text | `admit_document_for_processing`. Copy the input build from `upload_document` in `text_upload.rs`. |
| File commit, not PDF | `resolve_upload_content`, then admit, then `persist_uploaded_original` |
| File commit, PDF | `admit_pdf_bytes` (extract from `upload_pdf_document` first) |
| Task poll | `get_task_for_context` with `track_id` |
| Delete | `enqueue_document_deletion` (extract from `delete_document` first) |
| Asset bytes | `load_mm_asset_bytes_by_id` |
| Asset list | `list_mm_asset_summaries_for_document` |
| Graph | `build_entity_neighborhood`, then layout, then raster |
| Search | `retrieve_context` (unchanged) |
| Image block | `call_tool_result_with_image` in `summary.rs` |

New modules: `ingest.rs`, `upload_session.rs`, `download.rs`, `assets.rs`,
`graph_layout.rs`, `graph_raster.rs` — each under 300 lines where possible.

## Success measures

- No second admission pipeline in `mcp/`.
- W1–W7 file table in [07](../07-implementation-plan.md) completed.
- Prior SPEC-028 / SPEC-152 suites green after each wave.
