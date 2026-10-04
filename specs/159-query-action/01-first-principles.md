# 01 — First Principles (SPEC-159)

Parent: [README](README.md) · WHY: [00-why](00-why.md) · Next: [02-surfaces](02-surfaces.md)

## Axioms

1. **Context is the query.** What the user is looking at (entity, page) is the
   primary input to Ask — not a blank `/query` box.
2. **Human sends.** Prefill is assistance; auto-submit steals agency and can
   burn tokens / leak intent into the wrong thread.
3. **One intent, one module.** Every surface that “Asks about this” calls the
   same handoff builder; Query consumes it once.
4. **Honesty over magic.** If the API cannot filter by page, the UI must not
   claim it can.
5. **Reuse before invent.** Companion codec, neighborhood endpoint, document
   scope chips, and `quotePassage` already exist.
6. **Evidence beats vibes.** Every law maps to a named gate (LAW-159-12).

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-159-1** | One handoff — all cross-route Ask surfaces call a single pure builder (`buildQueryHandoff` / equivalent); they must not invent parallel `router.push` + draft hacks. |
| **LAW-159-2** | Ready, not sent — Ask fills the composer and focuses it; no chat/query stream starts until the user submits. |
| **LAW-159-3** | Ask always starts a new conversation — clear `activeConversationId` before writing the seed draft (graph, document, and in-query GraphNodeCard); prior conversations remain in history. Passage Quote stays in-place. |
| **LAW-159-4** | Consume-once seed — question text is derived from typed context (`buildSeedQuestion`) and written once to `query-draft:new`; free-text `?q=` is forbidden; refresh after edit must not re-paste the template. |
| **LAW-159-5** | Entity graph target — extend companion codec with `pane=graph&entity={id}` that loads neighborhood depth 1; `msg=` remains answer evidence; if both present, `entity` wins for Ask. |
| **LAW-159-6** | Document page target — reuse `pane=pdf&doc&page` (+ optional `chunk`/`lines`); set `scopedDocumentIds` to **that document only**; page is viewer + seed words, not a retrieval predicate. Entity Ask clears leftover document chips so chat is not filtered to a previous PDF. |
| **LAW-159-7** | Mode neutrality — Ask must not overwrite persisted `defaultQuerySettings.mode`; keyword extraction already sees entity/document names in the question. |
| **LAW-159-8** | Graph isolation — neighborhood companion uses an isolated engine/model; it must not mutate Graph Studio’s `useGraphStore` singleton (inherits LAW-157-9). |
| **LAW-159-9** | Kill-switch honesty — when companion is disabled, Ask still opens `/query`, seeds the composer, and applies document scope; it must not claim a pane is open. |
| **LAW-159-10** | Canonical route — handoff navigates to `/query` preserving `tenant`/`workspace`; do not depend on `/w/[slug]/query` for companion URL sync. |
| **LAW-159-11** | Reuse — `entityPath`, neighborhood GET, companion encode/decode, document scope hook, and quote-to-ask focus patterns are the building blocks; no fork of PDF/graph viewers. |
| **LAW-159-12** | CI is proof — every EC-159 has a named vitest and/or Playwright gate; green suite is DoD for each wave. |

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| **DRY** | One handoff builder, one seed-question builder, one companion codec (extended), one Query consumer hook (`useQueryHandoff` / equivalent). |
| **SRP** | Pure module owns encode/decode/seed text; hook owns navigation + store writes; panes own media; menu/header own only UI triggers. |
| **OCP** | New surfaces (table row, chunk row) register as callers of the handoff without rewriting Query. |
| **LSP** | Any Ask caller accepts the shared `QueryHandoffContext` (`kind: entity \| document`, ids, optional page/chunk). |
| **ISP** | Consumers subscribe narrowly: companion store for pane, query UI store for conversation, settings for scope. |
| **DIP** | Handoff depends on pure types + router/store interfaces — not on `GraphViewer` or `documents/[id]/page` internals. |

## Relationship to prior laws

| Prior law | How SPEC-159 extends it |
|-----------|-------------------------|
| LAW-157-1 Verify in place | Cross-route Ask lands **on** `/query` with companion open (verify context stays). |
| LAW-157-2 Chat never displaced | New conversation is intentional; composer/messages of the *new* thread stay mounted. |
| LAW-157-3 URL addressable | Add `entity` to companion params; preserve sanitize-to-closed. |
| LAW-157-4 One entry point | Handoff is the write path for Ask; menus call it. |
| LAW-157-8 Graph = answer evidence | Keep `msg=`; add parallel `entity=` neighborhood kind without merging Studio. |
| LAW-157-9 Isolate graph runtime | Neighborhood pane same isolation rule. |
| LAW-155-11 Answer ↔ graph | Studio escape hatch `/graph?entity=` remains. |
| SPEC-142 locators | Page/doc from storage/UI state — never invent page numbers. |

## Normative module sketch

```text
  Surfaces (callers)
    NodeContextMenu / NodeDetails / DocumentHeader / …
           |
           v
  lib/query/query-handoff.ts          (pure)
    - QueryHandoffContext
    - buildSeedQuestion(ctx) -> string
    - buildQueryHandoffUrl(ctx, baseParams) -> string
    - encodeCompanionSearch extended (entity)
           |
           v
  hooks/use-query-handoff.ts          (effects)
    - setActiveConversation(null)
    - write query-draft:new
    - replace document chips for this Ask intent
    - router.push(url)
           |
           v
  Query page consumer
    - CompanionUrlSync (entity target)
    - useQueryInterface draft load
    - GraphPane: msg OR entity neighborhood
    - focus composer, no submit
```

## Residual / deferred

| Item | Trigger |
|------|---------|
| Backend page / chunk filter on chat | Product requires true page-scoped retrieval |
| Sticky mode suggestion (`local` / `naive`) | UX research shows users want it; keep opt-in chip |
| View Documents → entity provenance | Separate OPP-159-05 pack |
| Auto-submit preference | Explicit user setting only |
| Edge “Ask about relationship” | Selected-edge model exists |
