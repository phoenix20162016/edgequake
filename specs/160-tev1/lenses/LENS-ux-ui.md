# LENS — UX and UI Designer (SPEC-160)

Parent: [README](../README.md) · Docs: [09](../09-ux-ui-spec.md)

## Job

Let a user pick a mode with full knowledge of its trade-off. Keep the default
screen quiet.

## Findings

1. The "Workspace default (X)" select is a pattern users already see for PDF
   parser choice. Reuse it.
2. The hard moment is a **failed upload** when the backend is down. Say what
   failed, what did not happen, and what to do. Never say "falling back".
3. A "Ready" dot alone fails color-blind users. Pair it with text.
4. Scores are ordered evidence, not probabilities. Never print "87% sure".

## Journey

```text
  Admin: Workspace settings ─► pick Decision ─► sees status "Ready · tev1:0.8b · CPU"
                                          └─► status red ─► reads reason ─► fixes server
  User:  Documents ─► drop file ─► Extraction: Workspace default (Decision)
                                          └─► override to LLM for this file
  After: row badge "Decision" ─► detail: Accepted 41 · Review 9 · Rejected 120
```

## Copy rules (STE)

1. Use short sentences. One idea each.
2. Use the same word for the same thing: "decision model", "backend", "gate".
3. State limits in plain words: "finds fewer relations than a large LLM".
4. Errors say: what happened, what did not happen, what to do.

## State table

| State | Where | Must show |
|-------|-------|-----------|
| Ready | Card | Dot, text, model, latency |
| Disabled | Card, select | Grey dot, reason, option disabled |
| Unreachable | Card | Red dot, host, Retry |
| Model missing | Card | Amber dot, copyable pull command |
| Upload blocked | Toast | Reason and "Open settings" |
| Mode changed on reprocess | Dialog | Warning that graph data will be deleted |

## Accessibility

Labels on radio and select. Text with every status color. Keyboard-reachable
tooltips on disabled options. Copy button announces "Copied".

## Risks

| Risk | Mitigation |
|------|-----------|
| Users pick `decision` and expect LLM quality | Inline help and preview label |
| Disabled option confuses | Tooltip with the reason |
| Too many controls in the card | Pack size sits in an Advanced disclosure |

## Out of scope

Review queue screen. It needs its own spec.
