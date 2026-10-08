# 07 — Implementation plan

Parent: [README](README.md) · Prev: [06 Edge cases](06-edge-cases.md) · Next: [08 E2E test matrix](08-e2e-test-matrix.md)

Language note: short sentences (ASD-STE100). Technical names stay as in code.

## Glossary

| Term | Meaning |
|------|---------|
| Agent id | `ent:{workspace_id}:{slug}` |
| Storage id | Graph node id (`{ws_uuid}::SLUG` or bare legacy) |
| Fold key | Compare key where `-`, `_`, spaces map to one character |
| Filter result | `empty` or `no_match` on search when scope removes all hits |

## Stop. Do not do these things.

1. Do not change `EntityId::new` / `normalize_entity_name` output.
2. Do not change REST `DocumentFilter` OR-union semantics.
3. Do not set `EDGEQUAKE_MCP_PROFILE` inside a parallel test.
4. Do not drop the word `evidence` from instructions (`spec152` requires it).
5. Do not break `t161_43` offset-past-end (`chunk_length=0`).
6. Do not put image bytes in `structuredContent`.
7. Do not call Axum handlers from `mcp/project/`.
8. Do not invent a second store key format.
9. Do not use a named paper in acceptance tests. Use a fixture.
10. Do not raise the download hard cap above 65536.

## Roadblocks and the fix

| ID | Block | Fix |
|----|-------|-----|
| RB-162-1 | MCP skips `exact_lookup_candidates` | `entity_ref::resolve_entity_node` reuses it |
| RB-162-2 | Tests seed bare ids | Worker harness + scoped ingest for e2e |
| RB-162-3 | Hyphen kept by normalizer | `fold_slug_key` compare-only |
| RB-162-4 | `apply_budget` overwrites truncation | Preserve prior omit counts; typed cursor |
| RB-162-5 | Relationship keys independent | Shared `EntityKeyResolver` |
| RB-162-6 | Text loader no tenant check | `doc_text.rs` shared loader |
| RB-162-7 | Pattern title-only | Match file_name + title in `DocumentScope` |

## Wave order

```text
W0 spec pack (this directory)
  │
  v
W1 red tests → resolve green (R1–R5)
  │
  v
W2 document scope (R6–R8)
  │
  v
W3 text + download page (R9–R10)
  │
  v
W4 neighborhood / image / cursor (R11–R12, R14)
  │
  v
W5 ingest fold merge (R13)
  │
  v
W6 bridge + README + schemas (R15)
  │
  v
W7 gates
```

## Wave detail

### W0 — Spec pack

Write README, 00–10, lenses, validator. Pointers in SPEC-152 and SPEC-161 READMEs.

### W1 — Entity resolve (steps 1–5)

1. Unit test hyphen fold in `ids.rs` / storage.
2. E2E: fixture hit id → `eq_graph_image` + `eq_neighborhood`.
3. Confirm red on current code.
4. Add `fold_slug_key`, `separator_variants`, `entity_ref.rs`.
5. Wire `eq_entity_get`, `eq_neighborhood`, `eq_graph_image`; unify `agent_id_for_node`.

Gates: A1, A2, A8. Tests: T-162-01…09.

### W2 — Document scope (steps 6–7)

`doc_scope.rs`; hard filters; entity_search document_ids; hit `document_ids[]`.

Gates: A3, A4. Tests: T-162-20…29.

### W3 — Text / download (steps 8–9)

`doc_text.rs`; `text_page`; download 65536 + `next_offset: null`.

Gates: A5, A6. Tests: T-162-30…41.

### W4 — Graph counts + cursor (step 10)

`edge_filter.rs`; center/edge_count; image counts/hint; fix `apply_budget`; `cursor.rs`.

Tests: T-162-10…19, T-162-42…44.

### W5 — Ingest merge (step 11)

`EntityKeyResolver`; entity + relationship; keyed lock; A7.

Tests: T-162-45…50.

### W6 — Bridge / docs (step 13)

Stdio `entity_id`; README; schemas; `tools.rs` descriptions.

Tests: T-162-51…52.

### W7 — Gates (step 12)

```bash
python3 specs/162-improve-mcp/scripts/validate-cross-ref.py
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p edgequake-api --test spec152_mcp_conformance --test spec161_mcp_control_e2e --test spec162_mcp_agent_interface_e2e --test spec027_api_contract
cargo test -p edgequake-storage -p edgequake-pipeline --lib
cd mcp && npm test
```

## File checklist

| File | Wave |
|------|------|
| `edgequake-storage/src/entity_id.rs` | W1 |
| `mcp/project/entity_ref.rs` | W1 |
| `mcp/project/ids.rs` | W1 |
| `mcp/project/graph.rs` | W1, W2, W4 |
| `mcp/project/graph_image.rs` | W1, W4 |
| `mcp/project/search.rs` | W1, W2 |
| `mcp/project/doc_scope.rs` | W2 |
| `mcp/project/doc_text.rs` | W3 |
| `mcp/project/catalog.rs` | W3 |
| `mcp/project/download.rs` | W3 |
| `mcp/project/blob.rs` | W3 |
| `mcp/project/edge_filter.rs` | W4 |
| `mcp/project/cursor.rs` | W4 |
| `mcp/project/budget.rs` | W4 |
| `mcp/project/fetch.rs` | W4 |
| `edgequake-pipeline/.../key_resolver.rs` | W5 |
| `mcp/src/tools/graph.ts` | W6 |
| `mcp/README.md` | W6 |
| `specs/152-new-mcp-contract/schemas/*` | W6 |
| `gateway/tools.rs` | W6 |
| `tests/spec162_mcp_agent_interface_e2e.rs` | W1–W5 |
