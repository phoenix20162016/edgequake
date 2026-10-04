# 07 — Retrieval & data contract

Parent: [README](README.md) · Architecture: [04](04-architecture.md) · Lens: [LENS-database](lenses/LENS-database.md)

## First principle

**Ask is a UX handoff onto existing retrieval.** It does not invent a new
query mode, migration, or filter predicate in v1.

---

## What the database already stores (reuse)

| Asset | Where | Ask use |
|-------|-------|---------|
| Entity nodes / edges | AGE graph (workspace-scoped) | Neighborhood GET |
| Entity descriptions | Node properties | Display in pane card |
| Document blobs + pages | Document + PDF storage | Source companion |
| Chunk ↔ page lineage | Chunk KV / lineage APIs | Optional `chunk`/`page` URL |
| Conversations | Chat storage | New conversation on submit |

**No new tables. No migrations in SPEC-159.**

---

## Neighborhood read path

```text
  GET /api/v1/graph/entities/{entity_name}/neighborhood?depth=1
       |
       v
  resolve_entity_node (name or workspace-scoped id)
       |
       v
  build_entity_neighborhood(graph_storage, tenant, id, depth∈[1,3])
       |
       v
  EntityNeighborhoodResponse { nodes, edges }
```

| Concern | Contract |
|---------|----------|
| Encoding | Path uses `encodeURIComponent` / `entityPath` for `uuid::NAME` |
| AuthZ | Existing tenant + workspace context on the handler |
| Missing entity | 404 → companion error state; seed still shown |
| Depth | Ask default 1; max 3 clamped server-side |

This is a **read**. Ask never writes graph edges.

---

## Chat / query write path (on user submit)

WebUI sends **chat completions stream**, not the raw `QueryRequest` keywords
API:

```text
  ChatCompletionRequest {
    message,                    // seeded then user-edited text
    conversation_id?: null,     // omitted → server creates
    mode,                       // UNCHANGED settings default
    document_filter?: {
      document_ids?: string[],  // document Ask only
      date_from?, date_to?, document_pattern?
    },
    llm_provider?, llm_model?, …
  }
```

| Field | Entity Ask | Document Ask |
|-------|------------|--------------|
| `message` | Contains entity label | Contains title (+ page words) |
| `document_ids` | **Cleared** (no leftover filter) | **Replaced** with this document only |
| `mode` | Unchanged | Unchanged |
| page / chunk | Absent | Absent (viewer only) |
| `hl_keywords` / `ll_keywords` | Not on chat path | Not on chat path |

### Honesty statement (normative copy constraint)

UI **may** say:

- “Ask about this entity”
- “Ask about this page” (meaning: open page + ask)

UI **must not** say:

- “Search only this page”
- “Answers restricted to page N”

---

## Document scope semantics

```text
  Before Ask (settings store)
  scopedDocumentIds = [A, B]

  Document Ask(doc=C)
       |
       v
  scopedDocumentIds = [C]         // replace leftovers  (EC-159-11)

  Entity Ask(entity=E)
       |
       v
  scopedDocumentIds = []          // clear leftovers; do not add a chip
                                  // (EC-159-12). Neighborhood GET stays
                                  // unfiltered; chat must match.
```

Empty `document_ids` remains a no-op on the server (existing behaviour).

---

## Why page is not a filter (database lens)

```text
  Chunk row:  document_id | chunk_id | page_start | page_end | text
       |
       +--> used for citation rewrite (SPEC-142) and viewer deep links
       |
       X  not accepted on ChatCompletionRequest.document_filter today

  Adding page filter would require:
    1. API field + OpenAPI
    2. Query engine allowed_chunk_ids or page predicate
    3. Acc / eval impact
  → Deferred (OPP-159-06), not silent half-measures
```

Until then, page words in the seed + document scope are the honest levers.

---

## Keyword / entity hint alternatives (AI lens pointer)

| Approach | Status |
|----------|--------|
| Entity name inside `message` | **v1** — keyword LLM sees it |
| Chat `ll_keywords` override | Not exposed on WebUI chat path |
| Eval-only `seed_entities` | Not on HTTP chat |
| Mode = `local` sticky | Forbidden by LAW-159-7 |

See [LENS-ai-engineer](lenses/LENS-ai-engineer.md).

---

## Observability

Reuse existing request logs. Optional client analytics events (post-ship):

| Event | Payload (non-PII) |
|-------|-------------------|
| `query_handoff_invoked` | `kind`, `has_page`, `companion_enabled` |
| `query_handoff_submitted` | `kind`, `edited` (bool seed≠sent) |

No new DB audit table required.

---

## Contract tests (data layer)

| Gate | Asserts |
|------|---------|
| Existing `e2e_entities` neighborhood tests | Unchanged behaviour depth/404 |
| Vitest `entityPath` / neighborhood URL | Encoding of `::` |
| Playwright: document Ask adds one id | Scope chip visible |
| Playwright: entity Ask | No new scope chip for that doc |

No SQL migrations in the DoD.
