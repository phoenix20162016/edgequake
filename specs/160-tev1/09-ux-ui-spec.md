# 09 — UX and UI spec

Parent: [README](README.md) · Prev: [08](08-api-contract.md) · ## As built (2026-10-04)

Evidence: Playwright against the real backend, screenshots in
[`e2e/screenshots`](e2e/screenshots/). Specs: `edgequake_webui/e2e/spec160/`.

| Spec item | Built |
|-----------|-------|
| Mode radio | A select (`LLM` or `Decision · closed questions`) with a badge that names the source ("set by the workspace"). The card is a standalone editable card, not a wizard step. |
| Gate preset | A select. It carries an "Uncalibrated" badge while `GATE_PRESETS_CALIBRATED` is false ([W8 report](measurements/w8-report.md)). |
| Backend state | One indicator for six states. A missing model shows a copyable `ollama pull` line and a "Check again" button. |
| Upload bar | An "Extraction" select next to the parser select. Choosing "Workspace default" sends nothing. A blocked backend warns on the bar and the server still refuses with a coded 422. |
| Narrow screens | The intake band shows one row only from 700 px. A narrower short band stacks and scrolls. On a phone the controls stack. The warning wraps (one truncated line in a row band, full text in the tooltip). |
| License notice | Localized with a UI key. The server text is the fallback. |
| Reprocess dialog warning (E08) | Deferred. |
| Review rows | Shown as a count only. A review screen is out of scope. |

Next: [10 Front architecture](10-frontend-architecture.md)

## Principles

1. Reuse the pattern users know: "Workspace default (X)" select, as in the PDF
   parser select (`document-dropzone.tsx`). No new interaction model.
2. State the trade-off at the point of choice. Do not hide limits in docs.
3. Never imply a fallback. Say what happens when the backend is down.
4. Keep the default quiet. A user who never touches the mode sees no change
   except one extra select.

## Surfaces

```text
  Workspace settings page                     Upload dropzone
 ┌─────────────────────────────────┐        ┌───────────────────────────────────┐
 │ Extraction mode                 │        │ [ drop files here ]               │
 │  (•) LLM extraction (default)   │        │ PDF parser:  [Workspace default ▾]│
 │  ( ) Decision model (Tev1)      │        │ Extraction:  [Workspace default   │
 │                                 │        │               (LLM) ▾]            │
 │ Decision model settings         │        │   ├ Workspace default (LLM)       │
 │  Model:     [tev1:0.8b      ▾]  │        │   ├ LLM extraction                │
 │  Gate:      [Balanced       ▾]  │        │   └ Decision model (Tev1)         │
 │  Backend:   ● Ready  tev1:0.8b  │        │ hint: "Runs on the server CPU.    │
 │             CPU · 41 ms         │        │  Best for private, cheap runs."   │
 │ [ Save ]                        │        └───────────────────────────────────┘
 └─────────────────────────────────┘
```

## Workspace card (F-160-11)

Location: next to `WorkspaceExtractBudgetCard` on both workspace pages
(`app/(dashboard)/workspace/page.tsx:294`, `app/w/[slug]/workspace/page.tsx:256`).
One component renders on both (DRY).

| Element | Behavior |
|---------|----------|
| Mode radio | `LLM extraction` or `Decision model`. First load shows the stored value. "Inherited from server (X)" shows when the key is absent and env sets a mode. |
| Model select | Shows only when mode is `decision`. Free text allowed (an operator can run a custom tag). |
| Gate select | `Strict`, `Balanced`, `Recall`. A badge says "Uncalibrated" until W8 closes. |
| Pack size | Advanced disclosure. Number 1–16. Default 4. |
| Backend status | Reads `GET /api/v1/decision/status`. States below. |
| License notice | One line, with a link to the model card |
| Save | Disabled until a change. Sends only changed fields. |

### Help text (STE style)

- LLM extraction: "A chat model reads each chunk and lists entities and relations. Uses your LLM provider."
- Decision model: "A small model answers yes/no and pick-one questions. It runs on the server. It uses no tokens. It finds fewer relations than a large LLM. It works in English."

### Backend status states

| State | Dot | Text | Action |
|-------|-----|------|--------|
| Ready | green | "Ready · tev1:0.8b · CPU · 41 ms" | none |
| Disabled | grey | "Off on this server. Ask your admin to set EDGEQUAKE_DECISION_ENABLED." | none |
| Unreachable | red | "Cannot reach the decision backend at localhost:11434." | Retry button |
| Model missing | amber | "Model tev1:0.8b is not pulled." + copy button for `ollama pull tev1:0.8b` | Copy |
| Not decision-capable | amber | "This model cannot answer decision questions." | none |
| Loading | skeleton | — | none |

Saving `decision` while the state is not Ready is allowed. The card shows an
amber note: "Uploads in decision mode fail until the backend is ready." Reason:
an admin may configure the workspace before the backend is up. Uploads fail
fast (LAW-160-4).

## Upload selector (F-160-12)

| Element | Behavior |
|---------|----------|
| Select | Options: `Workspace default (X)`, `LLM extraction`, `Decision model (Tev1)`. X is the effective workspace mode. |
| Visible for | File, folder, text, PDF, batch. Same control. |
| Inherit hint | `showInheritHint` pattern: grey line "Using the workspace setting." |
| Disabled option | `Decision model` shows disabled with a tooltip when status is Disabled or Unreachable. A user can still pick `Workspace default`. |
| Form data | The field is sent **only when the choice is not `default`** (rule in `perform-file-upload.ts`). |
| Persistence | Per session. Do not persist to the workspace. |
| Mixed batch | One choice per upload action. All files in the action share it. |

### Upload errors

| Code | Toast |
|------|-------|
| `decision_backend_unavailable` | "Decision backend is not reachable. File not uploaded." + "Open settings" |
| `decision_model_missing` | "Model not pulled. Run: ollama pull tev1:0.8b" |
| `decision_disabled` | "Decision mode is off on this server." |
| `invalid_extraction_mode` | "Unknown extraction mode." (a bug if shown) |

No toast says "falling back". No fallback exists.

## Document surfaces (F-160-13)

| Surface | Change |
|---------|--------|
| Documents table row | A small badge `Decision` when `extraction_mode = decision`. No badge for `llm` (keeps the default quiet). |
| Document detail | Section "Extraction": mode, source (document, workspace, server), model, contract. |
| Run stats | `Accepted 41 · Review 9 · Rejected 120`. The review number links to nothing in this spec (OP-160-1). It shows a tooltip: "Doubtful facts kept out of the graph." |
| Cost | Cost cell shows `—` and tooltip "No token cost" for decision documents. |
| Reprocess dialog | Adds the same mode select. A changed mode shows: "Changing the mode deletes the current graph data for this document." |

## Accessibility

1. Radio group and select have labels. Status dot has text, not only color.
2. The copy button has an accessible name and announces "Copied".
3. Disabled options keep a tooltip reachable by keyboard focus.
4. Contrast follows the existing design tokens.

## i18n

Add keys to `locales/en.json`, `fr.json`, `zh.json`. Prefix `extractionMode.`.

| Key | en |
|-----|----|
| `extractionMode.title` | Extraction mode |
| `extractionMode.llm` | LLM extraction |
| `extractionMode.decision` | Decision model (Tev1) |
| `extractionMode.inherit` | Workspace default ({{mode}}) |
| `extractionMode.helpLlm` | A chat model reads each chunk and lists entities and relations. |
| `extractionMode.helpDecision` | A small model answers closed questions on the server. It uses no tokens. It finds fewer relations than a large LLM. English only. |
| `extractionMode.status.ready` | Ready · {{model}} · {{ms}} ms |
| `extractionMode.status.disabled` | Off on this server. |
| `extractionMode.status.unreachable` | Cannot reach the decision backend at {{host}}. |
| `extractionMode.status.modelMissing` | Model {{model}} is not pulled. |
| `extractionMode.gate.uncalibrated` | Uncalibrated |
| `extractionMode.badge` | Decision |
| `extractionMode.licenseNotice` | The Tev1 weights license is not final. See the model card. |

Test: a script fails the build if a key exists in one locale only (T-160-V03).

## Empty, loading, and error states

| State | Behavior |
|-------|----------|
| Status request fails | Show "Unreachable" with Retry. Do not block the page. |
| Workspace not loaded | Card skeleton |
| Save fails with 400 | Inline field error from the server message |
| Save fails with network error | Toast "Could not save" and keep the form dirty |

Next: [10 Frontend architecture](10-frontend-architecture.md).
