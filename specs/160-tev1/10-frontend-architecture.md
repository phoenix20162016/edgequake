# 10 — Frontend architecture

Parent: [README](README.md) · Prev: [09](09-ux-ui-spec.md) · ## As built (2026-10-04)

- One width policy for the selects in the upload settings bar: `settingTriggerWidth` in `document-dropzone.tsx`.
- Row/stack choice lives in `lib/documents/dropzone-fill-layout.ts` (`DROPZONE_ROW_MIN_WIDTH_PX = 700`, tested).
- `DecisionStatusIndicator` is shared by the workspace card and the upload bar (`singleLine` for short bands).
- Words and status helpers: `constants/extraction-mode.ts` is the only place that knows `llm`, `decision`, `inherit`.

Next: [11 ML quality](11-ml-quality-spec.md)

Paths are relative to `edgequake_webui/src/`.

## Module map

```text
 constants/extraction-mode.ts            pure: words, parse, label keys, field builder
 types/workspace.ts                      + extraction_mode, effective_extraction_mode, decision_*
 lib/api/edgequake/decision.ts           GET /decision/status client
 lib/api/edgequake/workspaces.ts         + new fields in create/update payloads
 hooks/use-decision-status.ts            react-query hook (3 s timeout, 30 s stale)
 components/shared/
     extraction-mode-select.tsx          one select used by dropzone and reprocess dialog
     decision-status-indicator.tsx       dot + text + retry/copy
 components/workspace/
     workspace-extraction-mode-card.tsx  card for both workspace pages
 components/documents/
     extraction-mode-badge.tsx           table badge
     decision-stats-line.tsx             detail stats
 lib/upload/perform-file-upload.ts       + extractionMode option
 lib/upload/pdf-upload-form-data.ts      + extraction_mode field
 lib/upload/extraction-mode-field.ts     buildExtractionModeField (shared by all upload paths)
 locales/{en,fr,zh}.json                 + extractionMode.* keys
```

## Pure helpers (`constants/extraction-mode.ts`)

```text
  type ExtractionModeWord = 'llm' | 'decision';
  type UploadModeChoice  = 'default' | ExtractionModeWord;

  parseExtractionMode(raw): ExtractionModeWord | null     // mirrors Rust parse
  effectiveModeLabelKey(choice, workspaceMode): string    // 'Workspace default (LLM)'
  buildExtractionModeField(choice): { extraction_mode?: ExtractionModeWord }
        // returns {} when choice === 'default' (LAW-160-6)
  isDecisionBlocked(status): { blocked: boolean; reason?: string }
```

The words mirror `ExtractionMode::parse` in Rust. A contract test compares the
word list to a JSON fixture shared with the Rust tests (T-160-V04, EC-160-44).

## Call sites

| Call site | Change | DRY note |
|-----------|--------|----------|
| `document-manager.tsx` | Add `extractionMode` state (`UploadModeChoice`, default `'default'`). Pass to dropzone and to `performFileUpload`. | Same shape as `pdfParserBackend` state |
| `document-dropzone.tsx` | Render `ExtractionModeSelect` | One component |
| `perform-file-upload.ts` | `extractionMode?: ExtractionModeWord`. Call `buildExtractionModeField`. | One builder |
| `pdf-upload-form-data.ts` | Append the same field | Uses the same builder |
| Text upload (JSON) | Spread `buildExtractionModeField(choice)` into the body | Same builder |
| Reprocess dialog | Render `ExtractionModeSelect` | One component |
| Workspace pages | Render `WorkspaceExtractionModeCard` | One card, two pages |

## State and data flow

```text
  useDecisionStatus() ──► DecisionStatusIndicator (card)
                    └───► ExtractionModeSelect (disables 'decision' option)

  Workspace card:   form state ── Save ──► PATCH workspace ──► invalidate ['workspace', id]
  Upload select:    local state ── Upload ──► form-data field ──► POST

  No global store. Upload choice lives in document-manager state (like pdfParserBackend).
```

## SOLID call-outs

| Rule | Application |
|------|-------------|
| S | The select, the indicator, and the card are separate components. The field builder is a pure function. |
| O | A new mode word adds one entry to the constants file. Components read the list. |
| I | `ExtractionModeSelect` takes `value`, `onChange`, `workspaceMode`, `status`. It does not see the workspace store. |
| D | Components receive status and values as props. The hook sits at the page or manager level. |

## Testing

| Layer | Tool | Tests |
|-------|------|-------|
| Pure helpers | vitest | T-160-V01 to V04 |
| Components | vitest + testing library | T-160-V05 to V09 |
| Flows | Playwright `@spec160`, route mocks | T-160-E01 to E09 |

Run: `cd edgequake_webui && bun test` and
`pnpm exec playwright test e2e/spec160`.

Files stay small: each component under 200 lines. Split if larger.

Next: [11 ML quality](11-ml-quality-spec.md).
