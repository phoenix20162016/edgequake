# LENS — Security Expert

Parent: [README](../README.md) · Primary: [01-first-principles](../01-first-principles.md) LAW-161-6 · SPEC-154 · SPEC-152 `08-security-observability`

## Job

Keep write execution a separate trust class even when write tools are
advertised by default. Prevent cross-workspace handle theft and log leakage.

## Findings

- Residual risk: models see `eq_document_delete` in control mode.
- Mitigations: `destructiveHint`, required `confirm: true`, no default true,
  403 without `edgequake:write`, honest task status (never fake deleted).
- Upload handles must not work across workspaces (EC-161-15).
- Filename path traversal (EC-161-20).
- Public demo: set `EDGEQUAKE_MCP_PROFILE=query`.

## Decisions

| Control | Rule |
|---------|------|
| Scope | ingest/upload/delete → `edgequake:write` |
| Scope | download/asset/graph → `edgequake:read` |
| Scope | search/retrieve → `edgequake:query` |
| Workspace | `enforce_workspace_claim` on every tool arg |
| Confirm | Boolean `true` only; string `"true"` rejected |
| Logs | Never log full base64 blobs or full chunk content |
| Keys | Never pass third-party LLM keys through the model |
| Upload handle | Bound to tenant + workspace + user; TTL |
| Body | Keep 1 MiB; do not raise to “fix” large uploads |
| Query lockdown | Omits write tools from `tools/list` |

## Success measures

| EC | Assert |
|----|--------|
| EC-161-07 | Read-only 403 on ingest |
| EC-161-06 | Foreign workspace 403 |
| EC-161-15 | Cross-workspace upload denied |
| EC-161-21 | confirm false deletes nothing |
| EC-161-08 | Query profile hides/rejects writes |
| EC-161-20 | Path sanitized |

This lens does not invent laws. It applies LAW-161-6 and SPEC-154.
