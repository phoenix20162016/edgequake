# 09 — Cross-ref matrix

Parent: [README](README.md) · Prev: [08 E2E test matrix](08-e2e-test-matrix.md)

Validate with `python3 scripts/validate-cross-ref.py`.

## Law ↔ WHY ↔ Findings ↔ ECs ↔ Wave ↔ Gate ↔ Lens

| Law | WHY | Findings | ECs | Wave | Gate (tests) | Lens |
|-----|-----|----------|-----|------|--------------|------|
| LAW-162-1 | WHY-162-1 | F-162-01, F-162-02, F-162-11, F-162-16 | 01–09, 51–52 | W1, W6 | T-162-01…09, 51–52 | product, mcp |
| LAW-162-2 | WHY-162-1 | F-162-01, F-162-02 | 01–08 | W1 | T-162-01…08 | full-stack, database |
| LAW-162-3 | WHY-162-2 | F-162-09, F-162-10 | 09–19 | W4 | T-162-09…19 | ux-ui, ai-engineer |
| LAW-162-4 | WHY-162-3 | F-162-03, F-162-04, F-162-05, F-162-12, F-162-15 | 14, 20–29, 40–41 | W2, W3 | T-162-14, 20…29, 40–41 | product, full-stack |
| LAW-162-5 | WHY-162-4 | F-162-06, F-162-07, F-162-13 | 30–39, 42–44 | W3, W4 | T-162-30…39, 42–44 | mcp, ai-engineer |
| LAW-162-6 | WHY-162-5 | F-162-08, F-162-14 | 06, 45–50 | W1, W5 | T-162-06, 45…50 | database, full-stack |

## Finding ↔ symbol

| Finding | Symbol |
|---------|--------|
| F-162-01 | `ids.rs` `resolve_entity_lookup`; `graph.rs` |
| F-162-02 | `search.rs` vs `graph.rs` agent_entity_id field |
| F-162-03 | `graph.rs` `eq_entity_search` |
| F-162-04 | `document_filter_resolver.rs`; `search.rs` |
| F-162-05 | `document_filter_resolver.rs` title-only |
| F-162-06 | `catalog.rs` `eq_document_get` |
| F-162-07 | `blob.rs`; `download.rs` |
| F-162-08 | `entity_id.rs`; merger |
| F-162-09 | `graph.rs` `eq_neighborhood` |
| F-162-10 | `graph_image.rs` |
| F-162-11 | `mcp/README.md`; `mcp/src/tools/graph.ts` |
| F-162-12 | `download.rs` `load_markdown_bytes` |
| F-162-13 | `budget.rs` `apply_budget` |
| F-162-14 | `merger/relationship.rs` |
| F-162-15 | `gateway/tools.rs` unused args |
| F-162-16 | `spec161_mcp_control_e2e.rs` bare seed |

## Prior specs

| Spec | Relationship | Link |
|------|--------------|------|
| SPEC-152 | Inherited contract; schemas SSOT | [../152-new-mcp-contract/](../152-new-mcp-contract/) |
| SPEC-161 | Control surface peer; keep green | [../161-improve-mcp/](../161-improve-mcp/) |
| SPEC-028 MCP | Transport / OAuth | [../028-edgequake-query-service/mcp/000-index.md](../028-edgequake-query-service/mcp/000-index.md) |
| SPEC-005 / 031 | Document filter | REST keeps union; MCP hardens |

## Lens index

| Lens | Path |
|------|------|
| Product Owner | [lenses/LENS-product-owner.md](lenses/LENS-product-owner.md) |
| Full Stack | [lenses/LENS-full-stack.md](lenses/LENS-full-stack.md) |
| Database | [lenses/LENS-database.md](lenses/LENS-database.md) |
| UX / UI | [lenses/LENS-ux-ui.md](lenses/LENS-ux-ui.md) |
| MCP | [lenses/LENS-mcp.md](lenses/LENS-mcp.md) |
| AI Engineer | [lenses/LENS-ai-engineer.md](lenses/LENS-ai-engineer.md) |
