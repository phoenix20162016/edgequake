# LENS — Front-end Designer (SPEC-160)

Parent: [README](../README.md) · Docs: [10](../10-frontend-architecture.md), [09](../09-ux-ui-spec.md)

## Job

Build the controls from existing parts. Keep one field builder, one select, one
card, and no new global store.

## Findings

1. The upload option pattern is local state in `document-manager.tsx` passed
   to `document-dropzone.tsx`. The mode follows it.
2. The rule in `perform-file-upload.ts` is "set a field only when it differs
   from workspace defaults". `buildExtractionModeField('default')` returns `{}`.
3. Text, PDF, batch, and reprocess paths all need the same field. One builder
   serves all (DRY).
4. The workspace card appears on two pages. One component serves both.
5. Locale files exist in three languages. Parity is a test.

## Component tree

```text
 WorkspacePage ─► WorkspaceExtractionModeCard ─► DecisionStatusIndicator
 DocumentManager ─► DocumentDropzone ─► ExtractionModeSelect ◄─ useDecisionStatus
                 └► ReprocessDialog  ─► ExtractionModeSelect
 DocumentTableRow ─► ExtractionModeBadge
 DocumentDetail   ─► DecisionStatsLine
```

## Props contract

| Component | Props |
|-----------|-------|
| `ExtractionModeSelect` | `value`, `onChange`, `workspaceMode`, `status` |
| `DecisionStatusIndicator` | `status`, `onRetry` |
| `WorkspaceExtractionModeCard` | `workspace`, `onSaved` |
| `ExtractionModeBadge` | `mode` |
| `DecisionStatsLine` | `stats` |

## Data

| Concern | Choice |
|---------|--------|
| Status fetch | react-query, 3 s timeout, 30 s stale time, retry on demand |
| Workspace save | Existing mutation. Invalidate the workspace query. |
| Upload choice | Local state. Reset to `default` after each upload action. |

## Tests

V01–V09 (vitest) and E01–E10 (Playwright, route mocks). See
[14](../14-e2e-test-matrix.md).

## Size and quality rules

1. Each component under 200 lines.
2. Types from `types/workspace.ts`. No `any`.
3. `bun run build` and lint pass.
4. No new dependency.
