# 08 — E2E test matrix

Parent: [README](README.md) · Prev: [07 Implementation plan](07-implementation-plan.md) · Next: [09 Cross-ref](09-cross-ref.md)

Harness: `edgequake-api/tests/spec162_mcp_agent_interface_e2e.rs`  
Worker: `create_test_app_with_extraction_only` + fixture markdown (no named paper).  
Unit: `ids.rs`, `entity_id.rs`, `entity_ref.rs`, `doc_scope.rs`, `cursor.rs`, `key_resolver.rs`.

## Acceptance (A1–A8)

| Acc | Test id | Assert |
|-----|---------|--------|
| A1 | T-162-01, T-162-03 | Search entity hit → get same id → graph_image PNG `node_count>0` |
| A2 | T-162-02 | Neighborhood returns center; not `entities=[]` |
| A3 | T-162-20 | Search with fixture `document_ids`; each hit has that id |
| A4 | T-162-23 | Entity_search with that doc id returns fixture entity with doc id |
| A5 | T-162-30 | `include=text` first page contains fixture title |
| A6 | T-162-35 | Download `max_bytes=65536` one complete page |
| A7 | T-162-45 | Second ingest hyphen/underscore does not create new entity |
| A8 | T-162-09 | Unknown entity id → `eq/not_found`; text contains agent id |

## EC → test

| EC | Test | Wave |
|----|------|------|
| EC-162-01 | T-162-01 | W1 |
| EC-162-02 | T-162-02 | W1 |
| EC-162-03 | T-162-03 | W1 |
| EC-162-04 | T-162-04 | W1 |
| EC-162-05 | T-162-05 | W1 |
| EC-162-06 | T-162-06 | W1 |
| EC-162-07 | T-162-07 | W1 |
| EC-162-08 | T-162-08 | W1 |
| EC-162-09 | T-162-09 | W1 |
| EC-162-10 | T-162-10 | W4 |
| EC-162-11 | T-162-11 | W4 |
| EC-162-12 | T-162-12 | W4 |
| EC-162-13 | T-162-13 | W4 |
| EC-162-14 | T-162-14 | W4 |
| EC-162-15 | T-162-15 | W4 |
| EC-162-16 | T-162-16 | W4 |
| EC-162-17 | T-162-17 | W4 |
| EC-162-18 | T-162-18 | W4 |
| EC-162-19 | T-162-19 | W4 |
| EC-162-20 | T-162-20 | W2 |
| EC-162-21 | T-162-21 | W2 |
| EC-162-22 | T-162-22 | W2 |
| EC-162-23 | T-162-23 | W2 |
| EC-162-24 | T-162-24 | W2 |
| EC-162-25 | T-162-25 | W2 |
| EC-162-26 | T-162-26 | W2 |
| EC-162-27 | T-162-27 | W2 |
| EC-162-28 | T-162-28 | W2 |
| EC-162-29 | T-162-29 | W2 |
| EC-162-30 | T-162-30 | W3 |
| EC-162-31 | T-162-31 | W3 |
| EC-162-32 | T-162-32 | W3 |
| EC-162-33 | T-162-33 | W3 |
| EC-162-34 | T-162-34 | W3 |
| EC-162-35 | T-162-35 | W3 |
| EC-162-36 | T-162-36 | W3 |
| EC-162-37 | T-162-37 | W3 |
| EC-162-38 | T-162-38 | W3 |
| EC-162-39 | T-162-39 | W3 |
| EC-162-40 | T-162-40 | W3 |
| EC-162-41 | T-162-41 | W3 |
| EC-162-42 | T-162-42 | W4 |
| EC-162-43 | T-162-43 | W4 |
| EC-162-44 | T-162-44 | W4 |
| EC-162-45 | T-162-45 | W5 |
| EC-162-46 | T-162-46 | W5 |
| EC-162-47 | T-162-47 | W5 |
| EC-162-48 | T-162-48 | W5 |
| EC-162-49 | T-162-49 | W5 |
| EC-162-50 | T-162-50 | W5 |
| EC-162-51 | T-162-51 | W6 |
| EC-162-52 | T-162-52 | W6 |

## Regression keep-green

- `spec152_mcp_conformance`
- `spec161_mcp_control_e2e` (especially T-161-33 graph PNG, T-161-43 download)
- `spec027_api_contract` (normalize_entity_name presence)
