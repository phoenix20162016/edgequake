# 00 — WHY (5-WHY)

Parent: [README](README.md) · Next: [01-first-principles](01-first-principles.md)

## The job to be done

An analyst exploring the knowledge graph or a document must be able to **ask
EdgeQuake about what they are looking at** without:

1. Manually navigating to `/query`,
2. Re-typing the entity or page context,
3. Losing selection / page position,
4. Accidentally continuing an unrelated prior conversation.

Industry bar: contextual “Ask AI” handoffs that preserve selection, open a
split evidence surface, and leave the human in control of send (Copilot /
Notion AI / Multigrid citation patterns). EdgeQuake already has Query, the
SPEC-157 companion, neighborhood APIs, and document page sync — the gap is the
**cross-route handoff**, not the retrieval engine.

---

## 5-WHY chain A — No inbound query seed

| # | Question | Answer |
|---|----------|--------|
| 1 | Why can’t a graph or document surface open Query with a ready question? | `/query` and `/w/[slug]/query` read no question / entity / page seed from the URL. |
| 2 | Why does Graph Studio avoid navigating to Query with `?q=`? | `handleFindRelated` in `graph-viewer.tsx` documents that the query page ignores `?q=` and instead sets the local graph search. |
| 3 | Why was seed never added? | SPEC-157 shipped companion restore (`pane`, `doc`, `page`, `msg`) and in-query `quotePassage`, not a cross-route Ask contract. |
| 4 | Why does that break the analyst journey? | Selection and page are already in working memory; retyping them is friction and error. |
| 5 | **Root cause** | **Query has no typed inbound handoff contract (seed + companion + conversation policy).** |

```text
  Graph node / Document page
           |
           X  no Ask action
           |
           v
  Analyst manually opens /query
           |
           +--> last conversation resumes  (wrong thread)
           +--> empty or stale draft
           +--> no companion on the selection
           |
           X  ?q= ignored (documented in handleFindRelated)
```

Cross-ref: [LAW-159-1](01-first-principles.md) · [LAW-159-4](01-first-principles.md) ·
[F-159-01](02-surfaces.md#findings) · [EC-159-01](08-edge-cases.md).

---

## 5-WHY chain B — Opening `/query` continues the wrong chat

| # | Question | Answer |
|---|----------|--------|
| 1 | Why does Ask-by-navigation feel broken? | `activeConversationId` persists in `useQueryUIStore`; lifecycle selects the most recent conversation when null is not set intentionally. |
| 2 | Why does a prefilled question disappear? | `useQueryInterface` loads `query-draft:{activeConversationId\|new}` on conversation change and overwrites `input`. |
| 3 | Why can’t we only `setInput` after `router.push`? | Race: mount effect restores draft after navigation; one-shot React state is lost. |
| 4 | Why is continuing the prior chat wrong for Ask? | Ask is a new intent about a new object; mixing threads buries the question under unrelated history. |
| 5 | **Root cause** | **Ask must force a new conversation and write `query-draft:new` after clearing the active id (including in-query GraphNodeCard).** |

```text
  Today (naive router.push("/query"))
  -----------------------------------
  persist activeConversationId = C42
       |
       v
  Query mounts → load C42 messages
       |
       v
  draft effect → query-draft:C42 overwrites any hoped-for seed

  Target (SPEC-159)
  -----------------
  setActiveConversation(null)
  write query-draft:new = seedQuestion
  router.push(/query?pane=...)
       |
       v
  Query mounts empty thread + seeded composer
```

Cross-ref: [LAW-159-3](01-first-principles.md) · [F-159-02](02-surfaces.md#findings) ·
[EC-159-02](08-edge-cases.md) · [EC-159-03](08-edge-cases.md).

---

## 5-WHY chain C — Graph companion cannot show a studio node

| # | Question | Answer |
|---|----------|--------|
| 1 | Why can’t Ask open the companion on the selected node? | `decodeCompanionSearch` requires `pane=graph&msg={messageId}` — an answer, not an entity. |
| 2 | Why was `msg` chosen? | LAW-157-8: graph pane = evidence for **this answer**, not the whole knowledge graph. |
| 3 | Why not embed Graph Studio? | `useGraphStore` is a singleton; embedding Studio would clobber `/graph` (LAW-157-9). |
| 4 | Why is a neighborhood enough? | Analysts Ask about a node’s local meaning; 1-hop neighborhood is addressable and already served by `GET …/neighborhood`. |
| 5 | **Root cause** | **Companion graph has no `entity=` target; Ask must extend the codec without violating answer-graph semantics.** |

```text
  SPEC-157 today                 SPEC-159 extension
  ----------------               -------------------
  pane=graph&msg=MID    -->      pane=graph&msg=MID     (answer evidence)
                                 pane=graph&entity=EID  (Ask neighborhood)
                                      |
                                      v
                                 GET /graph/entities/{EID}/neighborhood?depth=1
                                      |
                                      v
                                 Embedded neighborhood graph
                                 (isolated engine; node selected)
                                      |
                                      +--> escape: /graph?entity=EID
```

Cross-ref: [LAW-159-5](01-first-principles.md) · [LAW-159-8](01-first-principles.md) ·
[F-159-03](02-surfaces.md#findings) · [EC-159-07](08-edge-cases.md).

---

## 5-WHY chain D — Page is not a retrieval predicate

| # | Question | Answer |
|---|----------|--------|
| 1 | Why can’t Ask “filter the query to this page”? | `DocumentFilter` / chat request carry `document_ids`, dates, pattern — no page/chunk field. |
| 2 | Why does the companion show a page then? | `page` is a **viewer** coordinate (SPEC-033 / 157), restored via URL, not a retrieval gate. |
| 3 | Why is pretending otherwise dangerous? | Users would believe the model only saw that page while retrieval still walked the document/workspace. |
| 4 | What can v1 honestly do? | Open the page beside chat, add a document scope chip, put document+page words in the seed question. |
| 5 | **Root cause** | **Page is evidence location; document scope is the only retrieval lever available without a schema change.** |

```text
  User expectation (unsafe if lied about)     Honest v1 contract
  ---------------------------------------     ------------------
  "Only answer from page 12"                  Companion shows page 12
                                              Scope chip = whole document
                                              Seed mentions "page 12"
                                              Mode unchanged (user default)
                                              No page field on ChatCompletionRequest
```

Cross-ref: [LAW-159-6](01-first-principles.md) · [LAW-159-7](01-first-principles.md) ·
[F-159-04](02-surfaces.md#findings) · [07-retrieval-contract](07-retrieval-contract.md).

---

## 5-WHY chain E — Adjacent actions leave the journey incomplete

| # | Question | Answer |
|---|----------|--------|
| 1 | Why does “View Documents” feel broken for a node? | `handleViewDocuments` navigates to `/documents` and **ignores the node** (no source filter). |
| 2 | Why is “Find Related” not Ask? | It stays on `/graph` and fills the graph search — correct for visual relatedness, wrong for RAG Q&A. |
| 3 | Why isn’t document detail Ask present? | Detail header has “View in knowledge graph” only (`detail-view-in-graph`); no Query handoff. |
| 4 | Why do in-query quote actions not cover cross-route? | `quotePassage` / SourcePane “Ask about this” only work when already on `/query`. |
| 5 | **Root cause** | **Surfaces lack a shared Ask intent; adjacent actions were never unified under one handoff.** |

```text
  Node menu today                         Target menu
  ---------------                         -----------
  View Details                            View Details
  Expand Neighborhood                     Expand Neighborhood
  Prune Node                              Prune Node
  Find Related   (stays on graph)         Find Related
  View Documents (unfiltered list)        Ask about this  <-- NEW (P0)
  Copy ID                                 View Documents  (OPP later)
  Delete Entity                           Copy ID
                                          Delete Entity
```

Cross-ref: [LAW-159-1](01-first-principles.md) · [F-159-05](02-surfaces.md#findings) ·
[03-product-spec](03-product-spec.md#opportunity-register).

---

## What “good” looks like vs today

| Capability | Today | Target (SPEC-159) |
|------------|-------|-------------------|
| Graph → Query with entity context | Not possible (`?q=` ignored) | Ask → new chat + neighborhood pane + seed |
| Document page → Query | Not possible | Ask → new chat + pdf pane at page + scope + seed |
| Auto-send | N/A | Never (user submits) |
| Conversation policy | Resume last | New conversation for cross-route Ask |
| Companion graph without answer | Impossible (`msg` required) | `entity=` neighborhood target |
| Page as filter | Not in API | Honest: viewer + doc scope + words in seed |
| In-query quote | `quotePassage` | Unchanged; companion card Ask reuses fill pattern |

## Non-goals (from WHY)

- Auto-submit.
- Full Studio embed in Query.
- Backend page/entity filter fields in v1.
- Sticky query-mode override.
- Fixing View Documents as P0 (opportunity, separate UX).

## Traceability

| WHY | Laws | Findings | Primary ECs |
|-----|------|----------|-------------|
| A no seed | LAW-159-1,4 | F-159-01 | EC-159-01 |
| B wrong chat / draft | LAW-159-3,4 | F-159-02 | EC-159-02,03,04 |
| C no entity pane | LAW-159-5,8 | F-159-03 | EC-159-07,08,09 |
| D page honesty | LAW-159-6,7 | F-159-04 | EC-159-10,11,12 |
| E incomplete journey | LAW-159-1,11 | F-159-05 | EC-159-13,14 |
