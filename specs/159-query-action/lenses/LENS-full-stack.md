# LENS — Full Stack Developer (SPEC-159)

Parent: [README](../README.md) · Frontend: [06](../06-frontend-architecture.md) · Arch: [04](../04-architecture.md)

## Job

Implement one pure handoff + one effectful hook + companion codec extension +
thin UI call sites. Prefer extraction over copy. No migration.

## Code map (change set)

```text
  NEW   lib/query/query-handoff.ts (+test)
  NEW   hooks/use-query-handoff.ts
  EDIT  lib/query/companion-pane.ts          (+entity)
  EDIT  companion store / url sync / GraphPane branch
  EDIT  node-context-menu, graph-viewer, node-details
  EDIT  documents/[id]/page.tsx
  EDIT  i18n en/fr/zh
  NEW   e2e/spec159/*
```

## Integration seams

| Seam | Contract |
|------|----------|
| Graph → handoff | Pass **menu/details node**, not ambient selection alone |
| Document → handoff | `document.id` + `pageSync.activePage` |
| Handoff → stores | `setActiveConversation(null)`, draft key, replace document chips |
| Handoff → router | `/query` + encoded companion params + workspace |
| Query → pane | Decode `entity` → neighborhood fetch via `getEntityNeighborhood` |
| Submit | Existing `useQueryStreamSession` unchanged |

## SOLID checklist for implementers

- Surfaces call `ask(ctx)` only.
- Pure module has zero React imports.
- `EntityNeighborhoodPane` does not import `GraphViewer`.
- Do not add `?q=`.
- Do not write `useGraphStore` graph data from the pane.

## Backend touch?

**None required.** Optional later: chat keyword overrides — out of v1.

## Failure modes to unit-test first

Encoding `::`, entity-over-msg precedence, sanitize closed, seed templates,
href path always `/query`.

## Cross-ref

F-159-01…08 · LAW-159-1,3,4,5,10,11 · EC-159-02…09,14,16
