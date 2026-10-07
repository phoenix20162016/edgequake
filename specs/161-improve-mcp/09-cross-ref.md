# 09 — Cross-ref matrix

Parent: [README](README.md) · Prev: [08 E2E test matrix](08-e2e-test-matrix.md)

Validate with `python3 scripts/validate-cross-ref.py`.

## Law ↔ WHY ↔ Findings ↔ ECs ↔ Wave ↔ Gate ↔ Lens

| Law | WHY | Findings | ECs | Wave | Gate (tests) | Lens |
|-----|-----|----------|-----|------|--------------|------|
| LAW-161-1 | WHY-161-1, WHY-161-2 | F-161-01, F-161-02, F-161-03, F-161-08 | 01–05, 09–11, 17–18, 23–24, 42, 47, 51 | W1, W3 | T-161-01…05, 09–11, 17–18, 23–24, 42, 47, 51 | product, full-stack |
| LAW-161-2 | WHY-161-3 | F-161-04, F-161-05, F-161-09 | 28–32, 39, 43–44, 48, 50 | W2, W4, W5 | T-161-28…32, 39, 43–44, 48, 50 | mcp, ux-ui |
| LAW-161-3 | WHY-161-4 | F-161-06 | 12–14, 16, 19, 52 | W3 | T-161-12…14, 16, 19, 52 | full-stack, mcp |
| LAW-161-4 | WHY-161-1 | F-161-05 | 33–36, 38, 40 | W5 | T-161-33…36, 38, 40 | front, full-stack |
| LAW-161-5 | WHY-161-1 | — | 46 | W7 | T-161-46 | ai-engineer, product |
| LAW-161-6 | WHY-161-5 | F-161-07 | 06–08, 15, 20–22, 29, 41, 49 | W1, W3 | T-161-06…08, 15, 20–22, 29, 41, 49 | security, product |
| LAW-161-7 | WHY-161-3 | F-161-04 | 28, 31, 37 | W4, W5 | T-161-28, 31, 37 | ai-engineer |
| LAW-161-8 | WHY-161-2 | F-161-02, F-161-03 | 21–27 | W1 | T-161-21…27 | ux-ui, full-stack |
| LAW-161-9 | — | F-161-10 | 45 | W6 | T-161-45 | mcp, full-stack |

## Finding ↔ symbol

| Finding | Symbol |
|---------|--------|
| F-161-01 | `dispatch.rs` `eq_ingest` |
| F-161-02 | `catalog.rs` `eq_document_delete` |
| F-161-03 | `dispatch.rs` `eq_task_get` |
| F-161-04 | missing vs `mm_assets.rs` |
| F-161-05 | missing vs `export.ts` / neighborhood JSON |
| F-161-06 | `body.rs` `MCP_MAX_BODY_BYTES` |
| F-161-07 | `profile.rs` `mcp_profile` |
| F-161-08 | `catalog.rs` `eq_workspace_delete` |
| F-161-09 | `resources.rs` placeholder |
| F-161-10 | `mcp/src/server.ts` |
| F-161-11 | `delete/single.rs` `delete_document` |
| F-161-12 | `file_validation.rs` PDF reject |
| F-161-13 | `summary.rs` `call_tool_result` |
| F-161-14 | `oauth/scopes.rs` `_` arm |
| F-161-15 | `edgequake-api/Cargo.toml` missing `image` |
| F-161-16 | `graph.rs` hop clamp |

## Prior specs

| Spec | Relationship | Link |
|------|--------------|------|
| SPEC-152 | Inherited contract; schemas SSOT; amended profile default | [../152-new-mcp-contract/](../152-new-mcp-contract/) |
| SPEC-028 MCP suite | Transport, OAuth, Streamable HTTP | [../028-edgequake-query-service/mcp/000-index.md](../028-edgequake-query-service/mcp/000-index.md) |
| SPEC-154 | Auth, scopes, tenant bind | [../154-sec-hardening/](../154-sec-hardening/) |
| SPEC-050 | Delete cascade / shared entities | [../050-pipeline-and-delete/](../050-pipeline-and-delete/) |
| SPEC-160 | Pack shape peer | [../160-tev1/](../160-tev1/) |
| SPEC-157 | Cross-ref / validator peer | [../157-side-by-side-query/](../157-side-by-side-query/) |

## Lens index

| Lens | Path |
|------|------|
| Product Owner | [lenses/LENS-product-owner.md](lenses/LENS-product-owner.md) |
| Full Stack | [lenses/LENS-full-stack.md](lenses/LENS-full-stack.md) |
| Database | [lenses/LENS-database.md](lenses/LENS-database.md) |
| UX / UI | [lenses/LENS-ux-ui.md](lenses/LENS-ux-ui.md) |
| Front | [lenses/LENS-front.md](lenses/LENS-front.md) |
| AI Engineer | [lenses/LENS-ai-engineer.md](lenses/LENS-ai-engineer.md) |
| Security | [lenses/LENS-security.md](lenses/LENS-security.md) |
| MCP | [lenses/LENS-mcp.md](lenses/LENS-mcp.md) |
