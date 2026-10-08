# SPEC-162 — Improve MCP Agent Interface

> **Status:** Implemented (W0–W7 gates green, 2026-10-08)  
> **Date:** 2026-10-08  
> **Document control:** EQ-MCP-1.2 (delta on EQ-MCP-1.1 / SPEC-161)  
> **Source:** EQ-STE-001 issue 2 (replaces issue 1)  
> **Revision pin:** `59ec526abed27c255e6edf9f9d9a403c18cd7a4d`  
> **Protocol:** MCP `2026-07-28` (tools, `structuredContent`, `ImageContent`, blob resources)  
> **Inherits:** [SPEC-152](../152-new-mcp-contract/) · [SPEC-161](../161-improve-mcp/) · [SPEC-028 MCP](../028-edgequake-query-service/mcp/000-index.md)  
> **Does not apply to:** model training, PDF parser

## What operators and implementers need to know

| Goal | Do this |
|------|---------|
| Understand why handles break | Read [00-why](00-why.md) |
| Know the laws | Read [01-first-principles](01-first-principles.md) |
| See code facts vs the issue | Read [02-code-facts](02-code-facts.md) |
| Trace findings to symbols | Read [03-findings](03-findings.md) |
| Know the target modules | Read [04-architecture](04-architecture.md) |
| See the contract delta | Read [05-contract-delta](05-contract-delta.md) |
| Cover edge cases | Read [06-edge-cases](06-edge-cases.md) |
| Ship in waves | Follow [07-implementation-plan](07-implementation-plan.md) |
| Prove A1–A8 and every EC | Run gates in [08-e2e-test-matrix](08-e2e-test-matrix.md) |
| Check cross-refs | [09-cross-ref](09-cross-ref.md) + `scripts/validate-cross-ref.py` |
| Read the source STE | [10-source-eq-ste-001](10-source-eq-ste-001.md) |

## Start here

1. [00-why.md](00-why.md)
2. [01-first-principles.md](01-first-principles.md)
3. [02-code-facts.md](02-code-facts.md)
4. [03-findings.md](03-findings.md)
5. [04-architecture.md](04-architecture.md)
6. [05-contract-delta.md](05-contract-delta.md)
7. [06-edge-cases.md](06-edge-cases.md)
8. [07-implementation-plan.md](07-implementation-plan.md)
9. [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
10. [09-cross-ref.md](09-cross-ref.md)
11. Lenses → [`lenses/`](lenses/)

## One-screen job

```text
  Agent holds handles only
           │
           ├─ eq_search ──► hit.id = ent:{ws}:{slug}
           │
           ├─ eq_entity_get / eq_neighborhood / eq_graph_image
           │         │
           │         v  resolve_entity_node()  (ONE function)
           │         storage id = {ws_uuid}::SLUG
           │
           ├─ document_ids / document_pattern = hard filter + filter_result
           ├─ include=text → text_page (offset/limit/total)
           └─ download ≤ 65536 bytes + next_offset null on last page
```

## Locked decisions (Wave 0)

| # | Decision | Law |
|---|----------|-----|
| 1 | Spec number is **162** (after SPEC-161). | — |
| 2 | Agent id form is `ent:{workspace_id}:{slug}`. Emit == accept. | LAW-162-1 |
| 3 | Resolver reuses `EntityId::exact_lookup_candidates` + fold variants. Do not invent a second store key. | LAW-162-2 |
| 4 | R4/R13 use `fold_slug_key` for compare only. Do not change `EntityId::new` output (`NEW-YORK` stays). | LAW-162-6 |
| 5 | MCP document_ids ∩ document_pattern (intersection). REST keeps union. | LAW-162-4 |
| 6 | Download hard cap 65536 bytes (env may only lower). | LAW-162-5 |
| 7 | Catalog `p:` page cursors stay. Typed cursors apply to search/fetch/neighborhood/download. | LAW-162-5 |
| 8 | Schema SSOT stays under `specs/152-new-mcp-contract/schemas/`. | LAW-161-9 |
| 9 | Multi-replica concurrent variant ingest residual is documented; in-process lock mitigates one replica. | LAW-162-6 |
| 10 | Fixture documents only in acceptance tests. No named paper. | A1–A8 |

## Non-goals

- Replacing REST or rewriting the Web UI
- Changing PDF parse
- Raising MCP JSON-RPC body above 1 MiB
- Changing REST `DocumentFilter` OR-union semantics
- Migrating existing storage ids to fold separators
- Implementing MCP Tasks / elicitation

## Status by wave

| Wave | Title | State |
|------|-------|-------|
| W0 | Spec pack, validator, SPEC-152/161 pointers | Done |
| W1 | Entity id resolve (R1–R5, A1, A2, A8) | Done |
| W2 | Document scope hard filters (R3, R6–R8, A3, A4) | Done |
| W3 | Text page + download page (R9, R10, A5, A6) | Done |
| W4 | Neighborhood / graph_image counts + typed cursor (R11, R12, R14) | Done |
| W5 | Fold merge at ingest (R13, A7) | Done |
| W6 | Legacy bridge + README + schemas (R15) | Done |
| W7 | Full gates green | Done |

## Official protocol references

- [MCP tools (2026-07-28)](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)
- [MCP resources (2026-07-28)](https://modelcontextprotocol.io/specification/2026-07-28/server/resources)
- [MCP schema](https://modelcontextprotocol.io/specification/2026-07-28/schema)
