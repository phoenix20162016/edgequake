# 06 — Frontend architecture

Parent: [README](README.md) · Architecture: [04](04-architecture.md) · UX: [05](05-ux-ui-spec.md)

## Modules to add

| Module | Responsibility | Pure? |
|--------|----------------|-------|
| `lib/query/query-handoff.ts` | `QueryHandoffContext`, `buildSeedQuestion`, `buildQueryHandoffHref`, companion target helpers | Yes |
| `lib/query/query-handoff.test.ts` | Vitest for seed text, URL encode, sanitize | Yes |
| `hooks/use-query-handoff.ts` | Conversation clear, draft write, scope add, `router.push` | No |
| Companion codec extension | `entity` in `COMPANION_PARAMS`, decode/encode | Yes (edit `companion-pane.ts`) |
| GraphPane entity mode | Fetch neighborhood, map to model, select seed | No |
| Tests for codec | Extend existing companion-pane unit tests | Yes |

## Modules to touch (call sites only)

| File | Change |
|------|--------|
| `node-context-menu.tsx` | New item + `onAskAboutThis` prop |
| `graph-viewer.tsx` | Wire `useQueryHandoff().ask` |
| `node-details.tsx` | Ask button |
| `documents/[id]/page.tsx` | Header Ask button |
| `query/page.tsx` | Optional: mount handoff consume helper if not inside interface |
| `use-query-interface.ts` / `query-interface.tsx` | Focus after seed; consume-once marker |
| `companion-url-sync.tsx` | Understand entity target via codec |
| `graph-pane.tsx` / new `entity-graph-pane.tsx` | Entity neighborhood branch |
| `use-companion-pane-store.ts` | Store `entityId` if not derived solely from URL |
| i18n locale files | en / fr / zh keys from [05](05-ux-ui-spec.md) |

W4 only: entity-browser, graph-as-table, graph-search, pdf-viewer toolbar,
document-hierarchy-tree, graph-node-card.

---

## Companion model extension

```text
  CompanionTarget (today)
    kind: none | pdf | graph
    source: SourceLocation | null
    messageId: string | null

  CompanionTarget (SPEC-159)
    kind: none | pdf | graph
    source: SourceLocation | null
    messageId: string | null
    entityId: string | null     // NEW — graph Ask
```

`CLOSED_TARGET` sets `entityId: null`.

Decode:

```text
  pane=graph
    entity = cleanId(params.get("entity"))
    if entity → { kind:graph, entityId, messageId:null, source:null }
    else msg = cleanId(params.get("msg"))
    if msg → { kind:graph, messageId:msg, entityId:null, … }
    else CLOSED
```

Encode writes `entity` **or** `msg`, never both.

---

## Hook sketch (`useQueryHandoff`)

```text
  ask(ctx: QueryHandoffContext): void
    dependencies:
      - useRouter
      - useQueryUIStore.setActiveConversation
      - useQueryScope.addDocument (document only)
      - useSearchParams / window.location.search for base params
      - isCompanionEnabled (for toast honesty only — URL still carries pane)

  Invariants:
    - never call submitQuery
    - always target path "/query"
    - preserve tenant & workspace from current search
```

SRP: hook does not know about Sigma or PDF viewers.

---

## GraphPane branching (OCP)

Prefer a thin branch inside `GraphPane` **or** a sibling `EntityNeighborhoodPane`
selected by `CompanionShell`:

```text
  CompanionShell
    | kind=pdf     → SourcePane
    | kind=graph && messageId → GraphPane (answer)
    | kind=graph && entityId  → EntityNeighborhoodPane
```

`EntityNeighborhoodPane`:

1. `getEntityNeighborhood(entityId, 1)`
2. Map to nodes/edges compatible with `EmbeddedAnswerGraph`
3. `selectedId` initial = `entityId` (match by id or label fallback)
4. Escape hatch link → `/graph?entity=`

Must use isolated engine instance (LAW-159-8).

---

## Draft consume-once

```text
  Option A (preferred): handoff writes draft; Query draft effect loads it;
    a ref `seedAppliedRef` prevents re-deriving from URL.

  Option B: zustand `pendingHandoffSeed: string | null` written by hook,
    consumed once by QueryInterface on mount, then nullled.

  Either way: URL must NOT contain the free-text question.
```

---

## DRY checklist

| Anti-pattern | Correct |
|--------------|---------|
| Each menu builds its own URL string | Call `buildQueryHandoffHref` |
| Document page duplicates companion encode | Use handoff / codec |
| Entity pane copies GraphViewer | Reuse `EmbeddedAnswerGraph` / `GraphRenderer` |
| In-query card invents a parallel seed path | Call `useQueryHandoff().ask` (OPP-159-03 / LAW-159-3) |

---

## Dependency direction

```text
  components/graph/* ──┐
  documents/[id]/* ────┼──► hooks/use-query-handoff
                       │         │
                       │         ▼
                       │    lib/query/query-handoff.ts
                       │         │
                       │         ▼
                       │    lib/query/companion-pane.ts
                       │
  components/query/* ──┴──► companion panes (read URL/store only)
```

Graph components must **not** import QueryInterface.

---

## Feature flag

Reuse `isCompanionEnabled()`. No new env var for Ask itself — Ask is always
available; companion is optional chrome.
