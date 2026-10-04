# 09 — Implementation plan

Parent: [README](README.md) · Edges: [08](08-edge-cases.md) · Tests: [10](10-e2e-test-matrix.md)

## Principles for every wave

- **DRY / SOLID** — pure handoff first; thin call sites; no GraphViewer fork.
- **First principles** — ready not sent; honest page semantics; isolated graph.
- **Proof** — each wave ends with named gates green (LAW-159-12).
- **Incremental** — codec before UI; consumer before writers; P0 before W4.

```text
  W0 pure codec + handoff
       → W1 Query consumer (entity pane + seed apply)
            → W2 Graph writers (menu + details)
                 → W3 Document writer (header)
                      → W4 Extra surfaces
                           → W5 E2E hardening / a11y / i18n audit
```

---

## Wave 0 — Pure contract

**Goal:** Types, seed builder, URL builder, companion `entity` codec — no UI.

| Task | Detail |
|------|--------|
| W0.1 | Extend `CompanionTarget` + `COMPANION_PARAMS` with `entity` |
| W0.2 | Decode precedence entity > msg; encode exclusivity |
| W0.3 | Add `lib/query/query-handoff.ts` (`buildSeedQuestion`, `buildQueryHandoffHref`) |
| W0.4 | Vitest: encoding, sanitize, templates, path `/query` |

**Laws:** LAW-159-4,5,10,11  
**DoD gates:** `vitest_companion_entity_*`, `vitest_handoff_*`  
**ECs:** EC-159-07,09,14,16

---

## Wave 1 — Query consumer

**Goal:** `/query` understands entity neighborhood pane and consume-once seed.

| Task | Detail |
|------|--------|
| W1.1 | Companion store / URL sync carry `entityId` |
| W1.2 | `EntityNeighborhoodPane` (or GraphPane branch) + `getEntityNeighborhood` |
| W1.3 | Isolated engine; select seed node; escape hatch to Studio |
| W1.4 | Draft apply + consume-once; focus composer; **no** submit |
| W1.5 | Empty / 404 / loading pane notices |
| W1.6 | Kill-switch: seed works without pane chrome |

**Laws:** LAW-159-2,5,8,9  
**DoD gates:** unit + hermetic companion entity fixtures  
**ECs:** EC-159-04,05,08,13,18

---

## Wave 2 — Graph Ask surfaces (P0)

**Goal:** Context menu + details panel invoke `useQueryHandoff`.

| Task | Detail |
|------|--------|
| W2.1 | `use-query-handoff.ts` (clear conversation, draft, push) |
| W2.2 | `onAskAboutThis` in `node-context-menu.tsx` + i18n |
| W2.3 | Wire in `graph-viewer.tsx` with **menu node** |
| W2.4 | Ask button in `node-details.tsx` |
| W2.5 | Playwright: menu visible, seed, new conversation, no autosubmit |

**Laws:** LAW-159-1,2,3,4  
**DoD gates:** `spec159_graph_*`  
**ECs:** EC-159-01,02,03,05,15,17

---

## Wave 3 — Document Ask surface (P0)

**Goal:** Document detail header Ask.

| Task | Detail |
|------|--------|
| W3.1 | Header button; page vs document label |
| W3.2 | Handoff with `pageSync.activePage` / omit page |
| W3.3 | Replace document chips with the asked doc (or clear on entity Ask) |
| W3.4 | Playwright: pdf companion page, scope chip, no autosubmit |

**Laws:** LAW-159-6,7  
**DoD gates:** `spec159_doc_*`  
**ECs:** EC-159-10,11,12

---

## Wave 4 — Opportunity surfaces

**Goal:** Same handoff on high-traffic selectors; in-query card variant.

| Task | Detail |
|------|--------|
| W4.1 | Entity browser / table / search Ask |
| W4.2 | PDF toolbar + hierarchy chunk Ask (`chunk`/`lines`) |
| W4.3 | Companion `GraphNodeCard` Ask — **current** conversation fill only |
| W4.4 | Smoke Playwright per surface testid |

**Laws:** LAW-159-1,11  
**DoD gates:** `spec159_w4_*`  
**ECs:** extends EC-159-01 patterns

---

## Wave 5 — Hardening

**Goal:** Full EC matrix green; a11y; locale parity; mid-stream case.

| Task | Detail |
|------|--------|
| W5.1 | Complete [10-e2e-test-matrix](10-e2e-test-matrix.md) |
| W5.2 | axe on menu Ask + document Ask focus path |
| W5.3 | fr/zh string audit |
| W5.4 | Mid-stream handoff + studio isolation e2e |
| W5.5 | Update status board in README to Implemented |

**Laws:** LAW-159-12 (all)  
**DoD:** every EC-159 gate named and green

---

## Definition of Done (pack)

- [ ] W0–W5 complete
- [ ] No `?q=` in codebase for Query seed
- [ ] No Graph Studio clobber
- [ ] No autosubmit in any Ask path
- [ ] Cross-ref [11](11-cross-ref.md) updated with as-built symbols
- [ ] Product acceptance US-159-01…04 signed off

## Out of wave (explicit)

- OPP-159-05 View Documents provenance
- OPP-159-06 page filter API / autosubmit
- OPP-159-07 edge Ask
- Migrations

## Suggested PR slicing

| PR | Contents |
|----|----------|
| PR1 | W0 + W1 (codec + consumer) |
| PR2 | W2 graph Ask |
| PR3 | W3 document Ask |
| PR4 | W4 + W5 |
