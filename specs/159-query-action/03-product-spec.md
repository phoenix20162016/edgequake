# 03 — Product specification

Parent: [README](README.md) · Surfaces: [02](02-surfaces.md) · Next: [04-architecture](04-architecture.md)

## Personas

| Persona | Goal with Ask |
|---------|----------------|
| **Analyst** | From a node or page, ask a grounded question without retyping context |
| **Researcher** | Keep evidence (neighborhood / PDF page) visible beside the new question |
| **Operator** | Trust that Ask never auto-spends LLM tokens; CI proves edge cases |

## User stories

### US-159-01 — Ask about a graph node

**As** an analyst on Graph Studio,  
**I want** an **Ask about this** action on the selected entity,  
**so that** Query opens with that entity’s neighborhood beside chat and a
question ready for me to send.

**Acceptance**

1. Action appears in the node context menu and the node details panel.
2. Clicking navigates to `/query` with `tenant`/`workspace` preserved.
3. A **new** conversation is active (prior chat still listed in history).
4. Composer contains a seed question that includes the entity’s display label;
   caret focused; **no** network chat/query POST until Enter/Send.
5. Companion graph shows the entity’s 1-hop neighborhood with that node
   selected (when companion enabled).
6. Escape hatch “Open in Graph Studio” → `/graph?entity={id}` still works.
7. Right-click open of the menu still does not change selection before Ask
   (SPEC-155 invariant).

### US-159-02 — Ask about a document page

**As** an analyst on document detail at page N,  
**I want** an **Ask about this page** action,  
**so that** Query opens with that page in the source companion, the document
scoped, and a question ready for me to send.

**Acceptance**

1. Action appears in the document detail header next to View in graph.
2. Uses `pageSync.activePage` when known; if page unknown, seeds a
   document-level question and omits `page` (does not invent page 1).
3. New conversation; seed in composer; no auto-submit.
4. Companion `pane=pdf&doc&page` when companion enabled.
5. Document appears as a scope chip; existing scoped docs are **kept** (add,
   do not replace).
6. Entity Ask does **not** set document scope.

### US-159-03 — Honest retrieval

**As** a product owner,  
**I want** copy and behavior that do not claim page-only retrieval,  
**so that** users trust citations after they submit.

**Acceptance**

1. Seed may say “page N” / document title; UI does not say “search only this
   page”.
2. No change to saved default query mode.
3. After submit, answer citations behave as today (SPEC-142 / 157).

### US-159-04 — Kill switch and workspace routes

**As** an operator with companion disabled,  
**I want** Ask to still open Query with a ready question,  
**so that** the action never dead-ends.

**Acceptance**

1. Companion off → composer + (for docs) scope still applied; no false pane UI.
2. Handoff uses `/query`, not `/w/[slug]/query`, for companion sync.

---

## Seed question templates (normative defaults)

Pure function `buildSeedQuestion(ctx)` — i18n keys later; English defaults:

| Context | Template |
|---------|----------|
| Entity with label L and type T | `What is {L} ({T}) in this knowledge graph, and how is it related to neighbouring entities?` |
| Entity label only | `What is {L} in this knowledge graph, and how is it related to neighbouring entities?` |
| Document title D, page N | `What are the key claims in "{D}"? I am looking at page {N}.` When converted markdown for that page is already in the UI, prepend a ≤600 char quote and ask about the excerpt (do not ask “what does it say on page N” — retrieval is not a page filter). |
| Document title D, no page | `What are the key claims in "{D}"?` |
| Chunk selected (W4) | Append short quote block (≤600 chars) like `quotePassage`, then a question. |

Labels use `formatEntityLabel` / document title already shown in the UI —
never raw opaque ids in the visible seed (id may appear only if label missing).

---

## Opportunity register

| ID | Opportunity | Wave | Disposition | Rationale |
|----|-------------|------|-------------|-----------|
| **OPP-159-01** | Ask on node details panel | W2 | **Ship** | Keyboard / AT users may not open context menu |
| **OPP-159-02** | Ask from entity browser, graph table, graph search results | W4 | **Ship** | Same handoff; high traffic select surfaces |
| **OPP-159-03** | Ask on companion `GraphNodeCard` (already on `/query`) | W4 | **Ship** | Same `useQueryHandoff` path — **always** new conversation + seed + companion URL; do not keep the prior thread |
| **OPP-159-04** | Ask on PDF toolbar + hierarchy chunk row | W4 | **Ship** | Page/chunk SSOT already in those widgets |
| **OPP-159-05** | Fix View Documents to open entity source docs | — | **Defer** | Needs provenance UX; separate from Ask |
| **OPP-159-06** | Auto-submit toggle / backend page filter | — | **Defer** | Violates LAW-159-2 / schema non-goal |
| **OPP-159-07** | Edge / relationship Ask | — | **Defer** | No selected-edge model |
| **OPP-159-08** | Dashboard “Continue exploring” with last entity | — | **Defer** | Nice-to-have after P0 metrics |
| **OPP-159-09** | Suggest mode chip (`local` / `naive`) without persisting | — | **Defer** | Optional UX polish; LAW-159-7 |

```text
  Opportunity funnel
  ------------------
  P0  US-159-01 menu + details, US-159-02 header
   |
   v
  W4  browser / table / search / PDF / chunk / in-query card
   |
   v
  Later  View Documents fix, mode chip, edge Ask, auto-submit setting
```

---

## Non-goals (product)

- Replacing Find Related or View in graph.
- Teaching users a new Query mode taxonomy via Ask.
- Guaranteeing the model only reads one page.
- Shipping MCP/tooling Ask in this pack (WebUI first).

## Success metrics (post-ship, observational)

| Signal | Direction |
|--------|-----------|
| Ask → submit conversion (seed sent within session) | Up |
| Manual `/query` opens immediately after graph select | Down |
| Support tickets “query ignored my page” | Flat/Down (honesty) |
| Accidental auto-sends | Zero (hard gate) |

## Acceptance checklist (pack-level)

- [ ] All US-159-01…04 criteria met in Playwright `@spec159`
- [ ] Every EC-159 has a named gate ([10](10-e2e-test-matrix.md))
- [ ] Cross-ref matrix complete ([11](11-cross-ref.md))
- [ ] No `?q=` introduced
- [ ] No Graph Studio store mutation from neighborhood pane
- [ ] i18n keys en/fr/zh for new chrome ([05](05-ux-ui-spec.md))
