# Changelog (edgequake-api)

All notable changes to the EdgeQuake API crate are tracked here. See the root CHANGELOG.md for workspace-wide changes.

## [Unreleased]

### Added

- **SPEC-161:** MCP control profile (default), async ingest/upload/delete/task
  poll, document download blobs, `eq_asset_get` ImageContent, `eq_graph_image`
  PNG, staging-first `eq_document_get`, `eq/not_ready` for pending markdown.
  Upload sessions in-process (`McpUploadStore`); PDF via `admit_pdf_bytes`.
  E2E: `tests/spec161_mcp_control_e2e.rs`.

### Fixed

- Document deletion task enqueue stamps canonical tenant/workspace UUIDs so
  MCP `eq_task_get` resolves delete tracks without explicit headers.
- MCP document delete prefers async Deletion enqueue for staging-only shells.

## [0.32.1] - 2026-10-07

### Fixed

- Default workspace and `ProviderContext` use the SSOT embedding model key from
  env (not a hardcoded string). Chat accepts `seed_entity_ids` for companion Ask.
- Empty `EDGEQUAKE_EMBEDDING_MODEL` (Compose `:-`) no longer fails serve as a
  missing provider context.

## [0.32.0] - 2026-10-06

### Changed

- Workspace catalogs require active membership. Scoped query deadlines and incremental RAG streaming.
- SPEC-027 source scans follow `request_authorization` and `auth_validation` instead of the old middleware names.

## [0.31.0] - 2026-10-04

### Added

- **SPEC-160** extraction-mode admission, workspace fields, document `decision_stats`, and `GET /api/v1/decision/status` plus `GET /api/v1/decision/models`.
- CHANGELOG.md for API crate.

## [0.1.0] - 2026-02-12

### Added

- Health API now includes build version and git metadata.
- Entity type count endpoint for dashboard KPI.
- Orphaned document recovery logic on startup.
- PDF cancel endpoint supports both `Pending` and `Processing` states.

### Changed

- Entity type KPI now uses backend aggregate count.

### Fixed

- Stuck uploading/cancel state for documents after restart or cancel.
