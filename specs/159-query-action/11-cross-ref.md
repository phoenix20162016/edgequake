# 11 — Cross-reference matrix

Parent: [README](README.md)

## Law ↔ Finding ↔ EC ↔ Wave ↔ Test ↔ Lens

| Law | Finding | ECs | Wave | Primary test | Primary lens |
|-----|---------|-----|------|--------------|--------------|
| LAW-159-1 | F-159-01, F-159-05 | 01, 15 | W2–W4 | `spec159_graph_menu_ask_visible` | Product / Full stack |
| LAW-159-2 | F-159-08 | 05, 06 | W1–W2, W5 | `spec159_no_autosubmit` | UX / AI |
| LAW-159-3 | F-159-02 | 02, 06 | W2, W5 | `spec159_new_conversation` | Full stack / Product |
| LAW-159-4 | F-159-01, F-159-02 | 03, 04 | W0–W2 | `spec159_seed_in_composer`, `spec159_edit_survives_refresh` | Full stack |
| LAW-159-5 | F-159-03 | 07, 08, 16 | W0–W1 | `vitest_companion_entity_over_msg`, `spec159_entity_404_pane` | Full stack / Front |
| LAW-159-6 | F-159-04 | 10, 11, 12 | W3 | `spec159_doc_ask_page`, `spec159_scope_replace_on_doc_ask` | Database / AI |
| LAW-159-7 | F-159-04 | 17 | W2 | `spec159_mode_unchanged` | AI / Product |
| LAW-159-8 | F-159-03 | 08, 18 | W1, W5 | `spec159_studio_not_clobbered` | Full stack |
| LAW-159-9 | — | 13 | W1 | `spec159_companion_off` | UX / Front |
| LAW-159-10 | F-159-06 | 14 | W0 | `vitest_handoff_path_is_query` | Full stack |
| LAW-159-11 | F-159-07, F-159-05 | 09 | W0, W4 | `vitest_handoff_encodes_entity` | Full stack / Database |
| LAW-159-12 | all | all | all | [10-e2e-test-matrix](10-e2e-test-matrix.md) | Full stack |

## WHY chain ↔ Law

| WHY | Laws |
|-----|------|
| A no seed | LAW-159-1, LAW-159-4 |
| B wrong chat / draft | LAW-159-3, LAW-159-4 |
| C no entity pane | LAW-159-5, LAW-159-8 |
| D page honesty | LAW-159-6, LAW-159-7 |
| E incomplete journey | LAW-159-1, LAW-159-11 |

## Finding ↔ code

| Finding | Path / symbol |
|---------|----------------|
| F-159-01 | `app/(dashboard)/query/page.tsx` — no seed reader |
| F-159-02 | `use-query-interface.ts` draft effect; `use-query-ui-store.ts` |
| F-159-03 | `lib/query/companion-pane.ts`; `graph-pane.tsx` |
| F-159-04 | `build-chat-request.ts`; API `DocumentFilter` |
| F-159-05 | `node-context-menu.tsx`; `graph-viewer.tsx` `handleViewDocuments`; `documents/[id]/page.tsx` |
| F-159-06 | `app/w/[slug]/query/page.tsx` |
| F-159-07 | `entityPath` / `getEntityNeighborhood` |
| F-159-08 | `query-interface.tsx` `quotePassage` |

## User story ↔ wave ↔ EC

| Story | Wave | ECs |
|-------|------|-----|
| US-159-01 Graph Ask | W2 | 01–05, 07–09, 12, 15, 17 |
| US-159-02 Document Ask | W3 | 01, 05, 10, 11 |
| US-159-03 Honest retrieval | docs + W3 | 10, 12, 17 |
| US-159-04 Kill switch / route | W0–W1 | 13, 14 |

## Opportunity ↔ disposition

| OPP | Wave / status | Laws |
|-----|---------------|------|
| OPP-159-01 details Ask | W2 ship | LAW-159-1 |
| OPP-159-02 browser/table/search | W4 ship | LAW-159-1 |
| OPP-159-03 in-query card | W4 ship (always new chat) | LAW-159-3 |
| OPP-159-04 PDF/chunk | W4 ship | LAW-159-6 |
| OPP-159-05 View Documents | Defer | — |
| OPP-159-06 page filter / autosubmit | Defer | conflicts 159-2/6 |
| OPP-159-07 edge Ask | Defer | — |
| OPP-159-08 dashboard | Defer | — |
| OPP-159-09 mode chip | Defer | LAW-159-7 |

## Prior specs

| Prior | Topic | SPEC-159 link |
|-------|-------|---------------|
| [SPEC-157](../157-side-by-side-query/) | Companion, quote-to-ask, isolation | Extend codec; reuse panes |
| [SPEC-142](../142-precise-links-on-query/) | Page locators | Viewer honesty |
| [SPEC-033](../033-page-lineage/) | `?page=` deep links | Document Ask page SSOT |
| [SPEC-155](../155-improve-ux-ui/) | Graph interactions | Menu selection invariant |
| [SPEC-032](../032-graph/) | Graph Studio | Escape hatch `/graph?entity=` |
| [SPEC-158](../158-entreprise-grade-authentication/) | Pack shape | Documentation pattern |

## Target symbols (as-designed)

| Symbol | Path | Role |
|--------|------|------|
| `buildSeedQuestion` | `lib/query/query-handoff.ts` | Pure seed |
| `buildQueryHandoffHref` | `lib/query/query-handoff.ts` | Pure URL |
| `useQueryHandoff` | `hooks/use-query-handoff.ts` | Effects |
| `decodeCompanionSearch` (+entity) | `lib/query/companion-pane.ts` | Codec |
| `EntityNeighborhoodPane` | `components/query/companion/…` | Pane |
| `onAskAboutThis` | `node-context-menu.tsx` | UI |
| `detail-ask-about-page` | `documents/[id]/page.tsx` | UI |
| `getEntityNeighborhood` | `lib/api/edgequake/graph.ts` | API client |

## Lens index

| Lens | File | Focus |
|------|------|-------|
| Product Owner | [lenses/LENS-product-owner.md](lenses/LENS-product-owner.md) | Scope, acceptance, risks |
| Full Stack | [lenses/LENS-full-stack.md](lenses/LENS-full-stack.md) | Modules, seams, SOLID |
| Database | [lenses/LENS-database.md](lenses/LENS-database.md) | No migration, filters |
| UX / UI | [lenses/LENS-ux-ui.md](lenses/LENS-ux-ui.md) | Journey, a11y, honesty |
| Front | [lenses/LENS-front.md](lenses/LENS-front.md) | Placement, density, states |
| AI Engineer | [lenses/LENS-ai-engineer.md](lenses/LENS-ai-engineer.md) | Seed, mode, future hints |

## Document graph

```text
  README
    ├── 00-why ◄──► 01-first-principles
    ├── 02-surfaces (F-159-*)
    ├── 03-product-spec (US / OPP)
    ├── 04-architecture (URL / handoff)
    ├── 05-ux-ui-spec
    ├── 06-frontend-architecture
    ├── 07-retrieval-contract
    ├── 08-edge-cases (EC-159-*)
    ├── 09-implementation-plan (W0–W5)
    ├── 10-e2e-test-matrix (gates)
    ├── 11-cross-ref (this file)
    └── lenses/*
```

## Completeness checklist

- [x] Every LAW has ≥1 EC and ≥1 gate
- [x] Every WHY maps to laws
- [x] Every F-159 cites code
- [x] Every US maps to wave
- [x] Every OPP has disposition
- [x] Six lenses present
- [x] ASCII diagrams in WHY / README / architecture / UX
