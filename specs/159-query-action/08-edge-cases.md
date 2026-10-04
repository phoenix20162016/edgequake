# 08 — Edge cases (EC-159)

Parent: [README](README.md) · Laws: [01](01-first-principles.md) · Tests: [10](10-e2e-test-matrix.md)

Every EC has: trigger, expected behaviour, mitigation, primary law, gate id.

---

## Register

### EC-159-01 — No seed contract today / Ask missing

| | |
|--|--|
| **Trigger** | User on graph/document wants to ask |
| **Expected** | Ask action visible and wired |
| **Mitigation** | Ship menu + header actions (W2/W3) |
| **Law** | LAW-159-1 |
| **Gate** | `spec159_graph_menu_ask_visible`, `spec159_doc_ask_visible` |

### EC-159-02 — Prior conversation would resume

| | |
|--|--|
| **Trigger** | User has `activeConversationId` set; invokes Ask |
| **Expected** | New empty conversation; prior chat remains in history list |
| **Mitigation** | `setActiveConversation(null)` before navigate |
| **Law** | LAW-159-3 |
| **Gate** | `spec159_new_conversation` |

### EC-159-03 — Draft effect would clobber seed

| | |
|--|--|
| **Trigger** | Navigate with hoped-for React state only |
| **Expected** | Composer shows seed from `query-draft:new` |
| **Mitigation** | Write localStorage draft after clearing conversation |
| **Law** | LAW-159-4 |
| **Gate** | `spec159_seed_in_composer` |

### EC-159-04 — Refresh after user edits seed

| | |
|--|--|
| **Trigger** | User edits composer, then reloads `/query?pane=…` |
| **Expected** | Edited text survives; template not re-pasted from URL |
| **Mitigation** | No `?q=`; consume-once seed; normal draft persistence |
| **Law** | LAW-159-4 |
| **Gate** | `spec159_edit_survives_refresh` |

### EC-159-05 — Auto-submit never happens

| | |
|--|--|
| **Trigger** | Ask handoff completes |
| **Expected** | Zero chat/query POST until user Send/Enter |
| **Mitigation** | Handoff never calls `submitQuery`; e2e network assert |
| **Law** | LAW-159-2 |
| **Gate** | `spec159_no_autosubmit` |

### EC-159-06 — Mid-stream Ask from another surface

| | |
|--|--|
| **Trigger** | Query streaming in tab A; user Ask from graph (same SPA) |
| **Expected** | Navigate to new conversation; prior conversation intact; no auto-send on new |
| **Mitigation** | Clear active id; do not abort unless existing stream lifecycle already does on conversation change — document behaviour; never submit seed |
| **Law** | LAW-159-2,3 |
| **Gate** | `spec159_mid_stream_handoff` |

### EC-159-07 — `msg` and `entity` both present

| | |
|--|--|
| **Trigger** | Malformed / stale URL with both params |
| **Expected** | `entity` wins; answer graph not shown |
| **Mitigation** | Decode precedence in codec |
| **Law** | LAW-159-5 |
| **Gate** | `vitest_companion_entity_over_msg` |

### EC-159-08 — Entity neighborhood 404

| | |
|--|--|
| **Trigger** | Deleted / wrong-workspace entity id in URL |
| **Expected** | Pane error notice; composer seed still present; no throw |
| **Mitigation** | Pane error state; handoff still wrote draft from label known at click (or generic) |
| **Law** | LAW-159-5,8 |
| **Gate** | `spec159_entity_404_pane` |

### EC-159-09 — Workspace-scoped id with `::`

| | |
|--|--|
| **Trigger** | Entity id `uuid::NAME` |
| **Expected** | URL encodes `%3A%3A`; neighborhood GET succeeds |
| **Mitigation** | `entityPath` / `encodeURIComponent` in href builder |
| **Law** | LAW-159-11 |
| **Gate** | `vitest_handoff_encodes_entity`, `spec159_entity_double_colon` |

### EC-159-10 — Document page unknown

| | |
|--|--|
| **Trigger** | No page markers / `activePage` undefined |
| **Expected** | Document-level Ask; omit `page`; seed without fake page number |
| **Mitigation** | Branch in `buildSeedQuestion` / href builder |
| **Law** | LAW-159-6 |
| **Gate** | `spec159_doc_ask_no_page` |

### EC-159-11 — Document Ask replaces leftover scope

| | |
|--|--|
| **Trigger** | `scopedDocumentIds` already `[A]`; Ask about doc `B` |
| **Expected** | Scope becomes `[B]` only. Prior chips do not ride along. User may `@` more after. |
| **Mitigation** | `scopedDocumentsForAsk` + `scope.replaceDocuments` |
| **Law** | LAW-159-3, axiom 1 |
| **Gate** | `spec159_scope_replace_on_doc_ask` |

### EC-159-12 — Entity Ask must not keep a document filter

| | |
|--|--|
| **Trigger** | Entity Ask while `scopedDocumentIds` is `[A]` (e.g. prior document Ask) |
| **Expected** | No document scope chip; chat omits `document_ids`. Do not add a chip for the entity. |
| **Mitigation** | `kind=entity` calls `scope.setDocumentIds([])` |
| **Law** | LAW-159-3, axiom 1 (context is the query), honesty vs companion neighborhood |
| **Gate** | `spec159_entity_no_doc_scope` |

### EC-159-13 — Companion kill switch

| | |
|--|--|
| **Trigger** | `NEXT_PUBLIC_QUERY_COMPANION=0` or localStorage disabled |
| **Expected** | `/query` opens; seed (+ doc scope) applied; no companion chrome |
| **Mitigation** | Existing flag; Ask still runs steps 1–5 |
| **Law** | LAW-159-9 |
| **Gate** | `spec159_companion_off` |

### EC-159-14 — `/w/[slug]/query` has no CompanionUrlSync

| | |
|--|--|
| **Trigger** | Someone navigates workspace-prefixed query |
| **Expected** | Handoff never depends on it; always `/query?workspace=` |
| **Mitigation** | LAW-159-10 in href builder |
| **Law** | LAW-159-10 |
| **Gate** | `vitest_handoff_path_is_query` |

### EC-159-15 — Right-click does not select before Ask

| | |
|--|--|
| **Trigger** | Open menu via right-click then Ask |
| **Expected** | Pre-Ask selection invariant holds (SPEC-155); Ask still uses menu node |
| **Mitigation** | Pass menu `node` into handoff, not `selectedNodeId` |
| **Law** | LAW-159-1 (inherits SPEC-155) |
| **Gate** | `spec159_menu_uses_target_node` |

### EC-159-16 — Invalid companion params

| | |
|--|--|
| **Trigger** | `pane=graph` without entity/msg; bad ids |
| **Expected** | Closed pane; if draft present, composer still works |
| **Mitigation** | Existing sanitize → `CLOSED_TARGET` |
| **Law** | LAW-159-5 / LAW-157-3 |
| **Gate** | `vitest_companion_sanitize_entity` |

### EC-159-17 — Mode unchanged

| | |
|--|--|
| **Trigger** | Ask with settings mode = `hybrid` |
| **Expected** | Mode still `hybrid` on landing and on submit |
| **Mitigation** | Handoff never calls `setQuerySettings` for mode |
| **Law** | LAW-159-7 |
| **Gate** | `spec159_mode_unchanged` |

### EC-159-18 — Graph Studio store isolation

| | |
|--|--|
| **Trigger** | Open entity neighborhood pane then visit `/graph` |
| **Expected** | Studio graph not replaced by neighborhood subgraph |
| **Mitigation** | Isolated engine / no `useGraphStore.setGraph` from pane |
| **Law** | LAW-159-8 |
| **Gate** | `spec159_studio_not_clobbered` |

---

## ASCII — failure modes vs mitigations

```text
  Failure                         Mitigation
  -------                         ----------
  Wrong chat                      clear activeConversationId
  Lost seed                       write query-draft:new
  Refresh re-templates            ban ?q= ; consume-once
  Auto token burn                 never submitQuery
  Studio clobber                  isolated neighborhood pane
  Fake page filter                honesty + doc scope only
  :: breaks URL                   entityPath encoding
  Dead-end if companion off       seed + scope still apply
```

## Coverage rule

LAW-159-12: merging a wave without its EC gates red is incomplete.
