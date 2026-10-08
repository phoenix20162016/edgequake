# 06 — Edge cases

Parent: [README](README.md) · Prev: [05 Contract delta](05-contract-delta.md) · Next: [07 Implementation plan](07-implementation-plan.md)

Each EC names a law and maps to a test in [08](08-e2e-test-matrix.md).

## Id resolve (EC-162-01…09)

| ID | Case | Expect | Law |
|----|------|--------|-----|
| EC-162-01 | Search hit id → `eq_graph_image` | PNG, `node_count>0` | LAW-162-1 |
| EC-162-02 | Same hit → `eq_neighborhood` | Center present | LAW-162-1 |
| EC-162-03 | Same hit → `eq_entity_get` | Same id returned | LAW-162-1 |
| EC-162-04 | Bare slug / `{ws}::SLUG` / `ent:SLUG` | Resolve or clear not_found | LAW-162-2 |
| EC-162-05 | Lower case / whitespace | Resolve | LAW-162-2 |
| EC-162-06 | Hyphen vs underscore fold | Resolve to one node | LAW-162-2, 6 |
| EC-162-07 | Empty slug / `ent::` | `eq/invalid_id` or not_found | LAW-162-2 |
| EC-162-08 | Foreign workspace in id | `eq/not_found` | LAW-162-2 |
| EC-162-09 | Unknown agent id | `eq/not_found` text contains agent id | LAW-162-3 |

## Neighborhood / image (EC-162-10…19)

| ID | Case | Expect | Law |
|----|------|--------|-----|
| EC-162-10 | Center zero edges | Center + `edge_count=0` | LAW-162-3 |
| EC-162-11 | Unknown center | `eq/not_found` not empty lists | LAW-162-3 |
| EC-162-12 | Only weak edges, weak off | `relationships=[]`, weak_count>0, hint | LAW-162-3 |
| EC-162-13 | Neighbors > 16 | Center kept; omitted_entities set | LAW-162-3 |
| EC-162-14 | `document_ids` / `edge_types` on neighborhood | Honored or removed from schema | LAW-162-4 |
| EC-162-15 | Graph image unknown | `eq/not_found` | LAW-162-3 |
| EC-162-16 | Graph image counts | `strong_edge_count`, `weak_edge_count` present | LAW-162-3 |
| EC-162-17 | Graph image weak hidden | Exact hint string | LAW-162-3 |
| EC-162-18 | Graph image center kept under cap | Center in picture | LAW-162-3 |
| EC-162-19 | Hop 3 on standard | Clamps to 2 (SPEC-161) | LAW-162-3 |

## Document scope (EC-162-20…29)

| ID | Case | Expect | Law |
|----|------|--------|-----|
| EC-162-20 | Known `document_ids` | All hits in that doc | LAW-162-4 |
| EC-162-21 | Unknown document id | `eq/not_found` for that id | LAW-162-4 |
| EC-162-22 | Valid id, no hits | `hits=[]`, `filter_result:"empty"` | LAW-162-4 |
| EC-162-23 | `eq_entity_search` + doc id | Entity with that doc in `document_ids` | LAW-162-4 |
| EC-162-24 | Entity_search ignores other docs | Only scoped entities | LAW-162-4 |
| EC-162-25 | Pattern matches file_name | Hits / docs considered | LAW-162-4 |
| EC-162-26 | Pattern matches title | Hits / docs considered | LAW-162-4 |
| EC-162-27 | Pattern no match | `documents_considered=[]` + message | LAW-162-4 |
| EC-162-28 | ids ∩ pattern empty | empty / no_match | LAW-162-4 |
| EC-162-29 | `document_ids: []` | No filter (documented) | LAW-162-4 |

## Text / download (EC-162-30…41)

| ID | Case | Expect | Law |
|----|------|--------|-----|
| EC-162-30 | `include=text` first page | `text_page` has title | LAW-162-5 |
| EC-162-31 | Default limit 8000 | Enforced | LAW-162-5 |
| EC-162-32 | Multibyte at boundary | No panic; char-safe | LAW-162-5 |
| EC-162-33 | `text_limit` 0 / over max | invalid or clamped | LAW-162-5 |
| EC-162-34 | URI still present | `text_resource` optional second field | LAW-162-5 |
| EC-162-35 | Download max 65536 | One page complete | LAW-162-5 |
| EC-162-36 | Last page | `next_offset: null` | LAW-162-5 |
| EC-162-37 | Offset past end | ok, `chunk_length=0` | LAW-162-5 |
| EC-162-38 | `max_bytes=0` | `eq/invalid_id` | LAW-162-5 |
| EC-162-39 | Exact multiple of 65536 | next then null | LAW-162-5 |
| EC-162-40 | Foreign workspace text | `eq/not_found` | LAW-162-4 |
| EC-162-41 | Pending markdown | `eq/not_ready` | LAW-162-5 |

## Cursor (EC-162-42…44)

| ID | Case | Expect | Law |
|----|------|--------|-----|
| EC-162-42 | Entity omit → cursor | `entities:N` not `chunks:N` | LAW-162-5 |
| EC-162-43 | Wrong object for view | `eq/invalid_id` | LAW-162-5 |
| EC-162-44 | Malformed cursor | `eq/invalid_id` | LAW-162-5 |

## Ingest merge (EC-162-45…50)

| ID | Case | Expect | Law |
|----|------|--------|-----|
| EC-162-45 | A-B then A_B | One entity | LAW-162-6 |
| EC-162-46 | A_B then A-B | One entity | LAW-162-6 |
| EC-162-47 | Both in same batch | One entity | LAW-162-6 |
| EC-162-48 | Relationship uses other variant | Edge lands on existing | LAW-162-6 |
| EC-162-49 | Different workspace same name | No cross-ws merge | LAW-162-6 |
| EC-162-50 | >4 separators (variant cap) | No forced merge; metric | LAW-162-6 |

## Legacy / README (EC-162-51…52)

| ID | Case | Expect | Law |
|----|------|--------|-----|
| EC-162-51 | Stdio `entity_id` | Accepted | LAW-162-1 |
| EC-162-52 | Stdio `entity_name` still works | Resolved | LAW-162-1 |
