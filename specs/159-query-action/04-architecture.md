# 04 — Architecture

Parent: [README](README.md) · Product: [03](03-product-spec.md) · Next: [05-ux-ui](05-ux-ui-spec.md)

## Goal

Specify a **typed, consume-once handoff** from Graph / Document into Query that
extends SPEC-157’s companion without violating answer-graph semantics or chat
isolation.

---

## URL contract

### Companion params (extended)

| Key | Values | Meaning |
|-----|--------|---------|
| `pane` | `pdf` \| `graph` | Companion kind |
| `doc` | document id | Source document |
| `page` | 1-indexed int | Viewer page |
| `chunk` | chunk id | Optional highlight |
| `lines` | `start-end` | Optional line range |
| `msg` | answer message id | Answer subgraph (SPEC-157) |
| **`entity`** | entity id | **Neighborhood graph (SPEC-159)** |

Preserve non-companion params (`tenant`, `workspace`, …) via existing
`encodeCompanionSearch` copy-base behaviour.

### Precedence when `pane=graph`

```text
  if entity valid  →  kind=graph, entityId=…, messageId=null   (Ask neighborhood)
  else if msg valid → kind=graph, messageId=…, entityId=null  (answer evidence)
  else              → CLOSED_TARGET
```

Invalid / incomplete params → closed pane (LAW-157-3 / LAW-159-5).

### Canonical examples

```text
  /query?pane=graph&entity=79d6e213-032d-402c-9325-aee3483d3185%3A%3AGPTSWARM&workspace=default

  /query?pane=pdf&doc=f6fa9cad-bbff-4892-a855-3bd7d70da044&page=12&workspace=default
```

Entity ids **must** be URI-encoded (`entityPath` / `encodeURIComponent`) so
`::` survives.

### Forbidden

| Param | Why forbidden |
|-------|----------------|
| `q` / `query` / `prompt` | Refresh would clobber edits; LAW-159-4 |
| Auto `submit=1` | LAW-159-2 |

---

## Handoff context type (normative)

```text
  QueryHandoffContext =
    | {
        kind: "entity"
        entityId: string
        label: string
        entityType?: string
        depth?: 1 | 2 | 3     // default 1
      }
    | {
        kind: "document"
        documentId: string
        title: string
        page?: number         // omit when unknown
        chunkId?: string
        startLine?: number
        endLine?: number
        passage?: string      // W4 quote
      }
```

### Pure outputs

| Function | Output |
|----------|--------|
| `buildSeedQuestion(ctx)` | string (composer text) |
| `buildCompanionTarget(ctx)` | `CompanionTarget` (+ entity field) |
| `buildQueryHandoffHref(ctx, baseSearch)` | `/query?…` string |

No I/O in the pure module.

---

## Effectful handoff sequence

```text
  useQueryHandoff.ask(ctx)
       |
       | 1. setActiveConversation(null)          LAW-159-3
       | 2. clear stream pending/optimistic       (best-effort)
       | 3. localStorage set query-draft:new      LAW-159-4
       |    = buildSeedQuestion(ctx)
       | 4. replace document chips for this Ask intent
       |    document → [that id]; entity → []            EC-159-11/12
       | 5. router.push(buildQueryHandoffHref(...))    LAW-159-10
       |
       v
  Query mounts
       |
       | CompanionUrlSync reads pane/entity|pdf
       | draft effect loads query-draft:new → input
       | focus composer (rAF)
       | mark seed consumed (session flag / one-shot store)
       |
       X  no submitQuery
```

### Consume-once semantics

After the draft is applied to the composer, set a session marker
`query-handoff-consumed` (or clear a one-shot zustand flag) so that:

1. Soft refresh after the user edits keeps the **edited** draft (normal
   draft persistence).
2. The template is **not** re-derived from `entity=` / `doc=` on every render.
3. Companion URL params **remain** addressable (pane still restores).

```text
  Seed source of truth     Companion source of truth
  --------------------     -------------------------
  query-draft:new          URL pane/entity|pdf
  (once at handoff)        (refresh restores pane)
```

---

## Graph pane: two modes

```text
                    GraphPane
                       |
         +-------------+-------------+
         |                           |
         v                           v
   messageId set               entityId set
   (answer evidence)           (Ask neighborhood)
         |                           |
         v                           v
   subgraphFromContext         getEntityNeighborhood(id, depth)
   buildAnswerGraphModel       map nodes/edges → AnswerGraphModel-compatible
         |                           |
         +-------------+-------------+
                       |
                       v
              EmbeddedAnswerGraph
              (isolated; selectedId = seed entity)
                       |
                       v
              Escape: /graph?entity= or answerMessage=
```

Depth default **1**. UI may later offer 2/3 without changing the URL contract
(optional `depth` query param deferred; keep depth in fetch options first).

404 / empty neighborhood → pane error/empty notice (LAW-157-11); composer seed
still present (EC-159-08).

---

## Document path

Reuses SPEC-157 Source pane unchanged:

```text
  pane=pdf&doc&page[&chunk&lines]
       → SourcePane → DocumentSourceView
       → scope chip via useQueryScope (handoff step 4)
```

---

## Conversation vs in-query Ask

| Origin | New conversation? | Seed write |
|--------|-------------------|------------|
| Graph Studio / Document detail (cross-route) | **Yes** | `query-draft:new` + handoff event |
| Companion GraphNodeCard Ask (already on `/query`) | **Yes** | `query-draft:new` + `eq:query-handoff-applied` |
| Passage strip Quote (already on `/query`) | **No** | `quotePassage` / `setInput` only |

OPP-159-03 ships GraphNodeCard Ask on the same new-conversation handoff.

---

## SOLID boundaries

```text
  +------------------+     depends on      +------------------------+
  | UI surfaces      | ------------------> | useQueryHandoff        |
  | (menu, header)   |                     | (DIP: router, stores)  |
  +------------------+                     +-----------+------------+
                                                       |
                                                       v
                                           +------------------------+
                                           | query-handoff.ts pure  |
                                           | companion-pane.ts      |
                                           +------------------------+
                                                       |
                                                       v
                                           +------------------------+
                                           | GraphPane / SourcePane |
                                           | (OCP: new entity mode) |
                                           +------------------------+
```

Surfaces must not call `localStorage` or companion encode directly.

---

## Security / tenancy notes

- Handoff preserves workspace query params; RLS / tenant headers unchanged.
- Entity id in URL is not a secret; same as `/graph?entity=`.
- No tokens in URL (inherits SPEC-154 / 158).
- Ask does not bypass document ACL — neighborhood and chat use existing APIs.

## Migration / backend

**None required for v1.** Optional future: chat `seed_entities` or page filter
— see [07](07-retrieval-contract.md).
