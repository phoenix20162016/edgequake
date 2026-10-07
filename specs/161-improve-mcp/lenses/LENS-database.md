# LENS — Database Expert

Parent: [README](../README.md) · Primary: [04-architecture](../04-architecture.md) · [02-surfaces](../02-surfaces.md)

## Job

Reuse existing blob tables. Add only what stateless `POST /mcp` needs for
upload handles. Do not store graph PNGs.

## Findings

- Illustrations already live in `document_mm_assets` (BYTEA + `content_type`).
- Originals: `pdf_documents` / `document_originals`.
- No upload-session table exists today (F-161-06 implies multi-call state).

## Decisions

1. **No new blob table for illustrations or graph images.**
2. **New table `mcp_upload_sessions`** (name may vary in migration):
   - `upload_id` PK
   - `tenant_id`, `workspace_id`, `user_id`
   - `filename`, `media_type`, `expected_sha256`, `expected_bytes`
   - `received_bytes`, `status` (`open` / `committed` / `aborted`)
   - `expires_at`, `document_id` (after commit)
   - Chunks: temp files under a workspace-scoped root **or** child table
     `mcp_upload_chunks(upload_id, offset, bytes)` — prefer temp files if
     BYTEA bloats (SP-161-1).
3. RLS / bind: same tenant+workspace as the rest of SPEC-154.
4. TTL sweeper: delete expired open sessions (cron or on-access).
5. Delete path remains SPEC-050; MCP does not invent cascade SQL.

## Success measures

| Measure | Evidence |
|---------|----------|
| Upload handle bound to workspace | EC-161-15 |
| Expired handle gone | EC-161-14 |
| Asset read hits `document_mm_assets` | EC-161-28…32 |
| No graph PNG rows | Code review W5 |
