# 05 — UX / UI specification

Parent: [README](README.md) · Product: [03](03-product-spec.md) · Next: [06-frontend](06-frontend-architecture.md)

## Design principles

1. **Discoverable next to existing actions** — Ask sits with View / Find /
   Graph, not buried in overflow.
2. **Predictable** — same label pattern: **Ask about this** (entity) /
   **Ask about this page** (document).
3. **Calm** — no auto-send, no modal wizard; land ready.
4. **Accessible** — menu item + details button; keyboard path equals mouse.
5. **Honest** — empty/error panes and kill-switch states tell the truth.

---

## Graph — node context menu

### Placement

Insert **Ask about this** after **Find Related**, before the separator that
precedes View Documents:

```text
  ┌─────────────────────────────┐
  │ Gptswarm                    │
  │ ● Organization              │
  ├─────────────────────────────┤
  │ View Details                │
  │ Expand Neighborhood         │
  │ Prune Node                  │
  │ Find Related                │
  │ Ask about this          NEW │
  ├─────────────────────────────┤
  │ View Documents              │
  │ Copy Entity ID              │
  ├─────────────────────────────┤
  │ Delete Entity               │
  └─────────────────────────────┘
```

### Visual

| Property | Value |
|----------|-------|
| Icon | `MessageSquare` / `Sparkles` (lucide) — prefer `MessageSquareText` |
| Variant | Default menuitem (not destructive) |
| `data-testid` | `node-context-menu-ask` |
| Disabled | Never for a valid menu node |

### Copy (i18n)

| Key | en | fr | zh |
|-----|----|----|----|
| `graph.contextMenu.askAboutThis` | Ask about this | Poser une question | 就此提问 |
| `graph.details.askAboutThis` | Ask about this | Poser une question | 就此提问 |

### Node details panel

Primary or secondary button in the action cluster (beside Edit / Copy):

- Label: same i18n key `graph.details.askAboutThis`
- `data-testid`: `node-details-ask`
- Tooltip: “Open Query with this entity ready to ask”

### Focus / selection

- Opening the menu: selection unchanged (existing).
- Choosing Ask: navigate away; Graph Studio state may persist in memory but
  Query is the focus surface.
- On Query: companion selects the seed entity; composer focused.

---

## Document detail — header

```text
  ┌──────────────────────────────────────────────────────────────┐
  │ ← Documents    codebook_….pdf    [Completed]                 │
  │                                                              │
  │              [ Download ]  [ Ask about this page ]  [ Graph ]│
  │                              NEW                             │
  └──────────────────────────────────────────────────────────────┘
```

| Property | Value |
|----------|-------|
| Placement | Immediately left of View in knowledge graph |
| Label (has page) | Ask about this page |
| Label (no page) | Ask about this document |
| Icon | `MessageSquareText` |
| `data-testid` | `detail-ask-about-page` |
| Size | Match existing header button density |

### Copy (i18n)

| Key | en | fr | zh |
|-----|----|----|----|
| `documents.detail.askAboutPage` | Ask about this page | Poser une question sur cette page | 就此页提问 |
| `documents.detail.askAboutDocument` | Ask about this document | Poser une question sur ce document | 就此文档提问 |

### Page unknown

If `pageSync.activePage` is undefined / sync disabled without markers:

- Use document-level label and seed (omit `page` param).
- Do **not** show “page 1” as if verified.

---

## Query landing UX

```text
  +---------------------------+  +----------------------------+
  | New conversation (empty)  |  | Companion                  |
  |                           |  |  graph neighborhood OR PDF |
  | [seed question……………|]   |  |  seed node selected / page |
  |              [Send]       |  |                            |
  +---------------------------+  +----------------------------+
         ^
         composer focused; Send idle until click/Enter
```

| State | Behaviour |
|-------|-----------|
| Companion on | Pane open from URL |
| Companion off | Chat only; seed + scope still applied; no empty pane chrome |
| Neighborhood loading | Pane loading notice |
| Neighborhood 404 | Pane error: “Entity not found in this workspace” |
| Neighborhood empty | Pane empty: “No neighbours within 1 hop” |
| PDF loading / missing doc | Existing SourcePane notices |

Toast (optional, ≤2.5s): “Ready to ask about {label}” — non-blocking; skip if
reduced motion prefers quieter UX (product choice: default **on** for first
ship, dismissible).

---

## W4 surface sketches

| Surface | Control |
|---------|---------|
| Entity browser row | Overflow or trailing Ask icon button |
| Graph table row | Row action Ask |
| Graph search result | Ask beside Go-to |
| PDF toolbar | Compact Ask next to page indicator |
| Hierarchy chunk | Ask in chunk row menu |
| Companion GraphNodeCard | “Ask about this” fills current composer |

---

## Keyboard & a11y

| Path | Expectation |
|------|-------------|
| Context menu | Arrow to Ask → Enter activates |
| Details Ask | Tab-focusable button |
| Document Ask | Tab-focusable header button |
| Landing | Focus moves to query textarea (`data-testid` existing composer) |
| Screen reader | Menuitem / button name = visible label |
| `aria` | No new dialog; navigation is a route change |

Do not trap focus after navigation beyond normal Query focus management.

---

## Motion

- Prefer route transition already used by `router.push`.
- Respect `prefers-reduced-motion` for any toast / pane open animation
  (inherit LAW-157-13).

---

## Visual regression notes

- Menu grows by one item — ensure collision / flip still via Radix (existing).
- Header button cluster must not wrap awkwardly below `md`; if space-tight,
  icon-only + tooltip is acceptable with accessible name.
