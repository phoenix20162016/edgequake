# SPEC-159 — Query Action Handoff (Graph Node · Document Page → Query)

> **Status:** Implemented (W0–W5 green — vitest + Playwright `@spec159`)  
> **Product pin:** EdgeQuake v0.30.0+  
> **Scope:** One shared **Ask** handoff that opens Query with a predefined
> question ready to execute (not auto-sent), companion open on the selected
> graph neighborhood or document page.  
> **Inherits:** [SPEC-157](../157-side-by-side-query/) ·
> [SPEC-155](../155-improve-ux-ui/) ·
> [SPEC-142](../142-precise-links-on-query/) ·
> [SPEC-033](../033-page-lineage/) ·
> [SPEC-032](../032-graph/)  
> **Peers:** [SPEC-158](../158-entreprise-grade-authentication/) (pack shape)

## Start here

1. [00-why.md](00-why.md) — Five WHYs + causal ASCII
2. [01-first-principles.md](01-first-principles.md) — LAW-159-1…12 + DRY/SOLID
3. [02-surfaces.md](02-surfaces.md) — Code map of graph / document / query / API
4. [03-product-spec.md](03-product-spec.md) — Stories, personas, opportunity register
5. [04-architecture.md](04-architecture.md) — URL contract, seed, neighborhood pane
6. [05-ux-ui-spec.md](05-ux-ui-spec.md) — Placement, labels, focus, empty/error, i18n
7. [06-frontend-architecture.md](06-frontend-architecture.md) — Modules, SOLID call sites
8. [07-retrieval-contract.md](07-retrieval-contract.md) — Neighborhood read, scope, honesty
9. [08-edge-cases.md](08-edge-cases.md) — EC-159 register + mitigations
10. [09-implementation-plan.md](09-implementation-plan.md) — Waves W0–W5 + DoD
11. [10-e2e-test-matrix.md](10-e2e-test-matrix.md) — One gate per EC
12. [11-cross-ref.md](11-cross-ref.md) — Law ↔ WHY ↔ F ↔ EC ↔ wave ↔ test ↔ lens
13. Lenses → [`lenses/`](lenses/)
    - [Product Owner](lenses/LENS-product-owner.md)
    - [Full Stack](lenses/LENS-full-stack.md)
    - [Database](lenses/LENS-database.md)
    - [UX / UI](lenses/LENS-ux-ui.md)
    - [Front](lenses/LENS-front.md)
    - [AI Engineer](lenses/LENS-ai-engineer.md)

## Locked decisions (Wave 0)

1. **One handoff, many surfaces** — `buildQueryHandoff` + one Query consumer.
   Graph node Ask and Document page Ask share the same module (LAW-159-1).
2. **Ready to execute, never auto-sent** — fill composer + focus; user submits
   (matches SPEC-157 `quotePassage`). No silent `POST /query` or chat stream
   (LAW-159-2).
3. **Ask always starts a new conversation** — `setActiveConversation(null)` then
   write `query-draft:new` (including in-query GraphNodeCard). Prior chat stays
   in history (LAW-159-3). Passage Quote stays in-place.
4. **Seed is consume-once, not `?q=`** — question is a pure function of
   context (`buildSeedQuestion`), applied once into draft storage. Free-text
   `q` is forbidden so refresh cannot clobber an edited draft (LAW-159-4).
5. **Graph companion extends with `entity=`** — `pane=graph&entity={id}` loads
   `GET /graph/entities/{id}/neighborhood?depth=1`. Answer graph (`msg=`) stays
   unchanged. Prefer `entity` over `msg` when both appear (LAW-159-5).
6. **Document companion reuses SPEC-157 codec** — `pane=pdf&doc&page` (+ optional
   `chunk`/`lines`). Page is a viewer coordinate; retrieval scopes the
   **document** via `scopedDocumentIds`, not a page predicate (LAW-159-6).
7. **No sticky mode change** — persisted query mode (`mix` default) is not
   overwritten. Entity name / document title live in the seed question
   (LAW-159-7).
8. **Isolate graph runtime** — neighborhood pane must not write into Graph
   Studio’s `useGraphStore` singleton (inherits LAW-157-9 / LAW-159-8).
9. **Companion kill switch is honest** — with companion disabled, still open
   `/query`, prefill, and scope; do not claim the pane opened (LAW-159-9).
10. **Handoff targets `/query`** — keep `tenant`/`workspace`. `/w/[slug]/query`
    does not mount `CompanionUrlSync` today; do not depend on it (LAW-159-10).
11. **Reuse before invent** — neighborhood API, `entityPath`, companion codec,
    `quotePassage` pattern, document scope chips (LAW-159-11).
12. **CI is proof** — every EC-159 maps to a named vitest and/or Playwright gate
    (LAW-159-12).

## Job in one screen

```text
  Graph Studio                          Document detail
  (node context menu / details)         (header Ask · page N)
              |                                    |
              +-----------+          +-------------+
                          |          |
                          v          v
                 +------------------------+
                 | buildQueryHandoff()    |
                 |  - new conversation    |
                 |  - seed draft once     |
                 |  - companion URL       |
                 |  - optional doc scope  |
                 +-----------+------------+
                             |
                             v
                 +------------------------+
                 | /query                 |
                 | Chat + companion       |
                 | Composer focused       |
                 | NO auto-submit         |
                 +------------------------+
                      |            |
                      v            v
               pane=graph       pane=pdf
               entity=ID        doc + page
               neighborhood     DocumentSourceView
```

## Non-goals (explicit)

- Auto-submitting the seeded question.
- Embedding full Graph Studio (filters, entity browser, export) inside Query.
- Backend `page` / `chunk` / `entity_id` filters on `ChatCompletionRequest`
  (deferred; honesty in [07-retrieval-contract](07-retrieval-contract.md)).
- Changing the user’s saved default query mode.
- Edge context menu (no selected-edge model in Graph Studio).
- Repairing “View Documents” (today ignores the node) — tracked as opportunity,
  out of P0.
- Three-pane Chat + PDF + Graph (SPEC-157 residual).

## Success narrative

When Waves W0–W5 land, an analyst can:

1. Right-click **Gptswarm** on the knowledge graph → **Ask about this** → land
   on `/query` with a new conversation, 1-hop neighborhood beside chat, that
   node selected, and a question about Gptswarm ready to send.
2. On a document at page 12 → **Ask about this page** → land on `/query` with
   the PDF companion open on page 12, that document as a scope chip, and a
   question naming the document and page ready to send.
3. Edit the question, press Enter — only then does a chat request fire.
4. Refresh after editing — the edited draft survives (seed was consume-once).
5. Trust CI: pure handoff vitest green, Playwright `@spec159` green, every
   EC-159 has a named gate.

## Opportunity register (summary)

| ID | Action | Disposition |
|----|--------|-------------|
| OPP-159-01 | Node details panel Ask | P0 (W2) — same handoff |
| OPP-159-02 | Entity browser / table / search Ask | W4 — same handoff |
| OPP-159-03 | Companion graph card Ask (in-query) | W4 — same handoff; always new conversation |
| OPP-159-04 | PDF toolbar / hierarchy chunk Ask | W4 — pdf target + chunk/lines |
| OPP-159-05 | View Documents → entity sources | Deferred (separate UX) |
| OPP-159-06 | Auto-submit / page filter API | Deferred (non-goal v1) |

Full register: [03-product-spec.md](03-product-spec.md#opportunity-register).

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-157](../157-side-by-side-query/) | Companion pane, URL codec, quote-to-ask, isolation |
| [SPEC-142](../142-precise-links-on-query/) | Page/doc locators from storage, not LLM |
| [SPEC-033](../033-page-lineage/) | `?page=` / chunk deep links |
| [SPEC-155](../155-improve-ux-ui/) | Graph interactions, answer↔graph |
| [SPEC-032](../032-graph/) | Graph Studio, entity model |

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done (this pack) |
| I1 | Pure handoff + companion codec extension | Done (W0) |
| I2 | Query consumer (new chat + seed + pane) | Done (W1) |
| I3 | Graph surfaces (menu + details) | Done (W2) |
| I4 | Document detail Ask | Done (W3) |
| I5 | Extra surfaces (OPP-159-02…04) | Done (W4) |
| T1 | Vitest + Playwright `@spec159` | Done (W5) |
| A1 | Acceptance checklist | See [03](03-product-spec.md) |
