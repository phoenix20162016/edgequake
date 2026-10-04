# 10 — E2E / unit test matrix

Parent: [README](README.md) · ECs: [08](08-edge-cases.md) · Plan: [09](09-implementation-plan.md)

## Conventions

| Kind | Location | Tag |
|------|----------|-----|
| Vitest pure | `edgequake_webui/src/lib/query/__tests__/` | — |
| Playwright | `edgequake_webui/e2e/spec159/` | `@spec159` |
| Hermetic helpers | Reuse `e2e/spec155/helpers/mock-api.ts`, `eq:e2e-open-node-menu` | — |

**Hard rule:** any Ask landing test must assert **no** `**/query/stream` /
chat completion POST until the test explicitly clicks Send (EC-159-05).

---

## Unit gates (Vitest)

| Gate ID | Asserts | ECs | Wave |
|---------|---------|-----|------|
| `vitest_companion_entity_decode` | `pane=graph&entity=E` → entityId | EC-159-16 | W0 |
| `vitest_companion_entity_over_msg` | both params → entity wins | EC-159-07 | W0 |
| `vitest_companion_encode_exclusive` | never writes entity+msg together | EC-159-07 | W0 |
| `vitest_companion_sanitize_entity` | bad id → CLOSED | EC-159-16 | W0 |
| `vitest_handoff_encodes_entity` | `::` → `%3A%3A` in href | EC-159-09 | W0 |
| `vitest_handoff_path_is_query` | path `/query`, keeps workspace | EC-159-14 | W0 |
| `vitest_handoff_seed_entity` | template contains label/type | EC-159-03 | W0 |
| `vitest_handoff_seed_document_page` | title + page N | EC-159-10 | W0 |
| `vitest_handoff_seed_document_no_page` | no invented page | EC-159-10 | W0 |
| `vitest_handoff_pdf_target` | companion pdf doc/page | EC-159-10 | W0 |

---

## Playwright gates

| Gate ID | Scenario | ECs | Wave |
|---------|----------|-----|------|
| `spec159_graph_menu_ask_visible` | Open menu via `eq:e2e-open-node-menu`; `node-context-menu-ask` visible | EC-159-01 | W2 |
| `spec159_menu_uses_target_node` | Right-click path; Ask uses menu node label in seed | EC-159-15 | W2 |
| `spec159_graph_details_ask` | `node-details-ask` navigates | EC-159-01 | W2 |
| `spec159_new_conversation` | Prior active chat → after Ask, empty thread; history still lists old | EC-159-02 | W2 |
| `spec159_seed_in_composer` | Composer textarea contains seed; focused | EC-159-03 | W2 |
| `spec159_no_autosubmit` | Route network: no chat/query POST before Send | EC-159-05 | W2 |
| `spec159_mode_unchanged` | Mode chip/settings remain prior value | EC-159-17 | W2 |
| `spec159_entity_neighborhood_pane` | Companion graph shows neighborhood; seed node selected | EC-159-08 path happy | W1/W2 |
| `spec159_entity_404_pane` | Mock 404 → pane error; seed present | EC-159-08 | W1 |
| `spec159_entity_double_colon` | Fixture id with `::`; neighborhood requested encoded | EC-159-09 | W2 |
| `spec159_edit_survives_refresh` | Edit seed → reload → edited text remains | EC-159-04 | W1 |
| `spec159_doc_ask_visible` | `detail-ask-about-page` on document detail | EC-159-01 | W3 |
| `spec159_doc_ask_page` | Ask from `?page=12` → companion `page=12` | EC-159-10 | W3 |
| `spec159_doc_ask_no_page` | No markers → no `page` param; document seed | EC-159-10 | W3 |
| `spec159_scope_replace_on_doc_ask` | Pre-scope A; Ask B → only B chip | EC-159-11 | W3 |
| `spec159_entity_no_doc_scope` | Entity Ask with leftover doc chips → chips gone | EC-159-12 | W2 |
| `spec159_companion_off` | Flag off → seed present; companion shell absent | EC-159-13 | W1 |
| `spec159_mid_stream_handoff` | Start stream mock; Ask from graph; new composer idle | EC-159-06 | W5 |
| `spec159_studio_not_clobbered` | After entity pane, `/graph` still full/fixture graph | EC-159-18 | W5 |
| `spec159_w4_browser_ask` | Entity browser Ask smoke | EC-159-01 | W4 |
| `spec159_w4_chunk_ask` | Hierarchy chunk Ask includes chunk/lines | W4 | W4 |
| `spec159_w4_card_ask_new_convo` | GraphNodeCard Ask clears active chat; seed in composer; no autosubmit | OPP-159-03 | W4 |

---

## Mapping EC → gate (must be 1:1+)

| EC | Primary gate(s) |
|----|-----------------|
| EC-159-01 | `spec159_graph_menu_ask_visible`, `spec159_doc_ask_visible` |
| EC-159-02 | `spec159_new_conversation` |
| EC-159-03 | `spec159_seed_in_composer`, `vitest_handoff_seed_*` |
| EC-159-04 | `spec159_edit_survives_refresh` |
| EC-159-05 | `spec159_no_autosubmit` |
| EC-159-06 | `spec159_mid_stream_handoff` |
| EC-159-07 | `vitest_companion_entity_over_msg` |
| EC-159-08 | `spec159_entity_404_pane` |
| EC-159-09 | `vitest_handoff_encodes_entity`, `spec159_entity_double_colon` |
| EC-159-10 | `spec159_doc_ask_page`, `spec159_doc_ask_no_page` |
| EC-159-11 | `spec159_scope_replace_on_doc_ask` |
| EC-159-12 | `spec159_entity_no_doc_scope` |
| EC-159-13 | `spec159_companion_off` |
| EC-159-14 | `vitest_handoff_path_is_query` |
| EC-159-15 | `spec159_menu_uses_target_node` |
| EC-159-16 | `vitest_companion_sanitize_entity` |
| EC-159-17 | `spec159_mode_unchanged` |
| EC-159-18 | `spec159_studio_not_clobbered` |

---

## Suggested file layout

```text
  edgequake_webui/e2e/spec159/
    graph-ask.spec.ts
    document-ask.spec.ts
    handoff-edges.spec.ts
    w4-surfaces.spec.ts
```

Run:

```bash
cd edgequake_webui
pnpm exec playwright test e2e/spec159 --grep @spec159
pnpm exec vitest run src/lib/query/__tests__/query-handoff src/lib/query/__tests__/companion-pane
```

## CI rule

Wave merge blocked if any gate for that wave’s ECs is missing or red.
