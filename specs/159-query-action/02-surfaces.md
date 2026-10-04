# 02 — Surfaces (code as-is)

Parent: [README](README.md) · Laws: [01](01-first-principles.md) · Next: [03-product-spec](03-product-spec.md)

## Purpose

Map every surface Ask must touch, with file/symbol citations. Findings
(`F-159-*`) are the code-grounded gaps this pack closes.

---

## Findings

| ID | Finding | Symbols / paths | Laws |
|----|---------|-----------------|------|
| **F-159-01** | Query has no inbound question / entity / page seed | `query/page.tsx` mounts `CompanionUrlSync` + `QueryInterface` only; no seed reader | LAW-159-1,4 |
| **F-159-02** | Opening `/query` resumes last chat; draft effect overwrites input | `useQueryUIStore` persist; `useQueryConversationLifecycle`; `useQueryInterface` draft effect ~L33–43 | LAW-159-3,4 |
| **F-159-03** | Companion graph requires `msg`; no entity target | `companion-pane.ts` `decodeCompanionSearch` / `COMPANION_PARAMS`; `GraphPane` answer-only | LAW-159-5,8 |
| **F-159-04** | Chat/`DocumentFilter` has no page/entity field | `build-chat-request.ts` `buildDocumentFilter`; API `DocumentFilter` = dates/pattern/ids | LAW-159-6,7 |
| **F-159-05** | Graph menu / document header lack Ask; View Documents ignores node | `node-context-menu.tsx`; `graph-viewer.tsx` `handleViewDocuments` / `handleFindRelated`; `documents/[id]/page.tsx` `handleViewInGraph` only | LAW-159-1,11 |
| **F-159-06** | `/w/[slug]/query` skips `CompanionUrlSync` | `app/w/[slug]/query/page.tsx` | LAW-159-10 |
| **F-159-07** | Workspace-scoped entity ids need encoding | `entityPath` in `lib/api/edgequake/graph.ts`; `uuid::NAME` | LAW-159-11 |
| **F-159-08** | In-query “Ask about this” exists only for passages | `quotePassage` in `query-interface.tsx`; `source-passage-strip.tsx` | LAW-159-2 (pattern to copy) |

---

## Graph Studio surfaces

### Node context menu

| Item | Path / symbol |
|------|----------------|
| Menu UI | `edgequake_webui/src/components/graph/node-context-menu.tsx` |
| Props | `onViewDetails`, `onExpandNeighborhood`, `onPruneNode?`, `onFindRelated`, `onViewDocuments`, `onCopyId`, `onDelete?` |
| Open | Right-click / Ctrl-click / Menu / Shift+F10; e2e `eq:e2e-open-node-menu` |
| Wiring | `graph-viewer.tsx` ~L635–719 |
| Selection | Single `selectedNodeId` in `use-graph-store.ts`; menu open does **not** change selection (SPEC-155) |

**Actions today (order):** View Details → Expand Neighborhood → Prune → Find Related → separator → View Documents → Copy ID → Delete.

**Ask insertion point:** After Find Related (or before View Documents separator) — see [05-ux-ui-spec](05-ux-ui-spec.md).

```text
  handleFindRelated(node)
    → setSearchQuery(label)     STAYS ON /graph
    → comment: query ignores ?q=

  handleViewDocuments()
    → router.push(/documents[?workspace=])
    → NODE IGNORED  (F-159-05)
```

### Node details panel

| Item | Path |
|------|------|
| Panel | `edgequake_webui/src/components/graph/node-details.tsx` |
| Today | Edit, Merge, Delete, copy label/id, related-node focus |
| Gap | No Ask (OPP-159-01 / P0 W2) |

### Other graph callers (W4)

| Surface | Path | Today |
|---------|------|-------|
| Graph as table | `graph-as-table.tsx` | `selectNode` only |
| Entity browser | `entity-browser-panel.tsx` | select + frame |
| Graph search | `graph-search.tsx` | select + frame |
| Graph deep link | `graph/page.tsx` `?entity=` / `?entities=` / `?focus=` | sets start node / search |

### Graph node data at click time

`GraphNode` (`types/graph.ts`): `id`, `label`, `node_type`, `description`,
`degree`, `community_id`, `properties`.

Helpers: `formatEntityLabel`, `source_document_ids` via `label-utils.ts` (first
doc id only — **not** required for entity Ask).

---

## Document detail surfaces

| Item | Path / symbol |
|------|----------------|
| Route | `app/(dashboard)/documents/[id]/page.tsx` → `/documents/{id}` |
| Page SSOT | `pageSync.activePage` from `usePageSyncController` |
| Inbound | `?page=` wins over chunk-resolved page (`parsePageParam`) |
| Header actions | Download, **View in knowledge graph** (`detail-view-in-graph` → `/graph?document={id}`), reprocess |
| Gap | No Ask beside graph button (P0 W3) |
| PDF toolbar | `pdf-viewer.tsx` page indicator — W4 slot |
| Hierarchy chunk | lineage tree chunk click — W4 slot (`chunk`/`lines`) |

```text
  Document detail header (today)
  +----------------------------------------------+
  | < Back | title | badge | Download | Graph | … |
  +----------------------------------------------+

  Target
  +--------------------------------------------------+
  | < Back | title | badge | Download | Ask | Graph | |
  +--------------------------------------------------+
```

---

## Query surfaces

| Item | Path / symbol |
|------|----------------|
| Dashboard route | `app/(dashboard)/query/page.tsx` — `CompanionUrlSync` + `QueryInterface` |
| Workspace route | `app/w/[slug]/query/page.tsx` — **no** CompanionUrlSync (F-159-06) |
| Composer / draft | `hooks/use-query-interface.ts` |
| Quote pattern | `query-interface.tsx` `quotePassage` — fill + focus, no submit |
| Scope | `hooks/use-query-scope.ts` → `scopedDocumentIds` |
| Chat build | `lib/query/build-chat-request.ts` → `document_filter.document_ids` |
| Companion codec | `lib/query/companion-pane.ts` |
| Companion store | `stores/use-companion-pane-store.ts` |
| Source pane | `components/query/companion/source-pane.tsx` |
| Graph pane | `components/query/companion/graph-pane.tsx` — answer `msg` only |
| Kill switch | `lib/query/companion-flag.ts` `isCompanionEnabled()` |
| Empty suggestions | `handleSuggestionClick` — fill only |

### Companion URL today

```text
  /query?pane=pdf&doc=<id>&page=3&chunk=<cid>&lines=12-30
  /query?pane=graph&msg=<messageId>
```

`COMPANION_PARAMS` = `pane`, `doc`, `page`, `chunk`, `lines`, `msg`.

### Companion URL target (SPEC-159)

```text
  /query?pane=graph&entity=<entityId>&workspace=…
  /query?pane=pdf&doc=<id>&page=<n>&…&workspace=…
```

Add `entity` to `COMPANION_PARAMS`. Sanitize rules unchanged (invalid → closed).

---

## Backend / API surfaces

| Endpoint / type | Path | Role for Ask |
|-----------------|------|--------------|
| `GET /api/v1/graph/entities/{entity_name}/neighborhood?depth=` | `entity_ops.rs` `get_entity_neighborhood`; depth clamp 1–3 | Graph companion load |
| `build_entity_neighborhood` | `services/entity_neighborhood.rs` | Age graph walk |
| WebUI client | `getEntityNeighborhood(id, depth)` + `entityPath` | Encoding `::` |
| `ChatCompletionRequest` | `chat_types.rs` | `message`, `conversation_id?`, `mode`, `document_filter` — **no** page/entity |
| `DocumentFilter` | `query_types.rs` | `date_from`, `date_to`, `document_pattern`, `document_ids` |
| `QueryRequest` keywords | `hl_keywords` / `ll_keywords` | Available on `/query` but **not** on chat path the WebUI uses |

```text
  Entity Ask retrieval path (honest)
  ----------------------------------
  User submits seed question containing entity label
       |
       v
  ChatCompletionRequest { message, mode (unchanged), document_filter? }
       |
       v
  Keyword extract sees entity name in message
       |
       X  no seed_entities on HTTP chat
       X  no page filter
```

---

## E2E surfaces (existing patterns to extend)

| Suite | Path | Pattern |
|-------|------|---------|
| Graph menu | `e2e/spec155/graph-interactions.spec.ts` | hermetic Sigma, right-click, selection invariant |
| Graph menu a11y | `e2e/graph-menus-tour-a11y.spec.ts` | `eq:e2e-open-node-menu` |
| Document page | `e2e/spec155/document-detail-pdf.spec.ts` | `?page=25`, `pdf-page-sheet` |
| Companion pdf | `e2e/spec157/companion-source.spec.ts` | `/query?pane=pdf&doc&page` |
| Quote no-send | `e2e/spec157/companion-a11y-ai.spec.ts` `ai_quote_to_ask` | textarea filled, no new message |

New suite: `e2e/spec159/` — see [10-e2e-test-matrix](10-e2e-test-matrix.md).

---

## ASCII — surface topology

```text
  +------------------+     +----------------------+     +------------------+
  | Graph Studio     |     | Document detail      |     | Query            |
  | menu / details   |     | header / (W4 tools)  |     | companion+compose|
  +--------+---------+     +----------+-----------+     +--------+---------+
           |                          |                          ^
           |                          |                          |
           +------------+-------------+                          |
                        |                                        |
                        v                                        |
             +----------+-----------+                            |
             | query-handoff (new)  | ---------------------------+
             +----------------------+
                        |
        +---------------+----------------+
        |                                |
        v                                v
  neighborhood GET                 scopedDocumentIds
  companion entity=                companion pdf=
```
