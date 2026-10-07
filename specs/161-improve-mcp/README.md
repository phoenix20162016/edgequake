# SPEC-161 — Improve MCP Control Surface

> **Status:** Draft (specification pack; implementation waves W1–W7 not started)  
> **Date:** 2026-10-07  
> **Document control:** EQ-MCP-1.1 (delta on EQ-MCP-1.0)  
> **Protocol:** MCP `2026-07-28` (tools, `ImageContent`, blob resources, annotations; Tasks/elicitation deferred)  
> **Applies to:** Remote Streamable HTTP gateway (`POST /mcp`) **and** `@edgequake/mcp-server` (stdio bridge)  
> **Inherits:** [SPEC-152](../152-new-mcp-contract/) (EQ-MCP-1.0) · [SPEC-028 MCP suite](../028-edgequake-query-service/mcp/000-index.md) · [SPEC-154](../154-sec-hardening/) (auth / scopes) · [SPEC-050](../050-pipeline-and-delete/) (delete cascade)  
> **Peers:** [SPEC-160](../160-tev1/) (pack shape) · [SPEC-157](../157-side-by-side-query/) (cross-ref style)

## What operators and implementers need to know

| Goal | Do this |
|------|---------|
| Understand why stubs fail agents | Read [00-why](00-why.md) |
| Know the laws | Read [01-first-principles](01-first-principles.md) |
| See what ships today | Read [02-surfaces](02-surfaces.md) |
| Trace findings to symbols | Read [03-findings](03-findings.md) |
| Know the target modules | Read [04-architecture](04-architecture.md) |
| See the EQ-MCP-1.0 delta | Read [05-contract-delta](05-contract-delta.md) |
| Cover edge cases | Read [06-edge-cases](06-edge-cases.md) |
| Ship in waves | Follow [07-implementation-plan](07-implementation-plan.md). Start there if you will write the code. |
| Prove every EC | Run gates in [08-e2e-test-matrix](08-e2e-test-matrix.md) |
| Check cross-refs | [09-cross-ref](09-cross-ref.md) + `scripts/validate-cross-ref.py` |

## Start here

1. [00-why.md](00-why.md)
2. [01-first-principles.md](01-first-principles.md)
3. [02-surfaces.md](02-surfaces.md)
4. [03-findings.md](03-findings.md)
5. [04-architecture.md](04-architecture.md)
6. [05-contract-delta.md](05-contract-delta.md)
7. [06-edge-cases.md](06-edge-cases.md)
8. [07-implementation-plan.md](07-implementation-plan.md)
9. [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
10. [09-cross-ref.md](09-cross-ref.md)
11. Lenses → [`lenses/`](lenses/)
    - [Product Owner](lenses/LENS-product-owner.md)
    - [Full Stack](lenses/LENS-full-stack.md)
    - [Database](lenses/LENS-database.md)
    - [UX / UI](lenses/LENS-ux-ui.md)
    - [Front](lenses/LENS-front.md)
    - [AI Engineer](lenses/LENS-ai-engineer.md)
    - [Security](lenses/LENS-security.md)
    - [MCP](lenses/LENS-mcp.md)

## One-screen job

```text
  Host (Cursor / Claude / Codex / Grok)
           │
           ├─ Streamable HTTP ──► POST /mcp
           └─ stdio ───────────► @edgequake/mcp-server ──► same /mcp
                                      │
                                      v
                           mcp/project/  (AgentView projection)
                                      │
     ┌──────────────┬─────────────────┼──────────────────┬──────────────┐
     v              v                 v                  v              v
  admit_*      upload session    delete path      mm_assets       retrieve_context
  (ingest)     begin/write/commit  (async task)   + graph_raster   (eq_search)
     │              │                 │                  │              │
     └──────── real document / task ids; ImageContent for PNG/JPEG ─────┘
```

## Locked decisions (Wave 0)

| # | Decision | Law |
|---|----------|-----|
| 1 | Spec number is **161** (after SPEC-160). | — |
| 2 | **Control mode is the default** when `EDGEQUAKE_MCP_PROFILE` is unset. Write tools are advertised. | LAW-161-6 |
| 3 | `EDGEQUAKE_MCP_PROFILE=query` is an **opt-in lockdown** that omits write tools (public demo). | LAW-161-6 |
| 4 | Write execution still needs `edgequake:write`. Delete needs `confirm: true`. | LAW-161-6 |
| 5 | Claimed side effects call the existing admission / delete / asset services. Stubs are defects. | LAW-161-1 |
| 6 | Tokens stay in the SPEC-152 envelope. Bytes use `ImageContent` or blob resources. | LAW-161-2 |
| 7 | `MCP_MAX_BODY_BYTES` stays 1 MiB. Large files use an upload handle. | LAW-161-3 |
| 8 | Graph PNG is a pure layout + raster of `build_entity_neighborhood`. Not Sigma. | LAW-161-4 |
| 9 | No `eq_answer`. Search returns `retrieval_id` + hits. | LAW-161-5 |
| 10 | Schema SSOT stays under `specs/152-new-mcp-contract/schemas/`. | LAW-161-9 |
| 11 | MCP Tasks extension and MRTR elicitation are **out of this pack**. | — |
| 12 | `eq_workspace_delete` stops lying: `eq/not_implemented` after confirm. | LAW-161-1 |

## Non-goals

- Replacing REST or rewriting the Web UI
- An LLM essay tool (`eq_answer`)
- Pixel parity with the Sigma WebGL exporter
- Implementing `io.modelcontextprotocol/tasks` or elicitation/MRTR
- Raising the MCP JSON-RPC body cap above 1 MiB
- Destroying a workspace through MCP in this pack
- Reopening SPEC-152 retrieval budgets, modes, or envelope shape

## Status by wave

| Wave | Title | State |
|------|-------|-------|
| W0 | Spec pack, schemas, validator, SPEC-152 pointer | This pack |
| W1 | Stop the lies (ingest, task, delete, profile default) | Planned |
| W2 | Download + real `resources/read` | Planned |
| W3 | Upload session begin/write/commit | Planned |
| W4 | Assets catalog + `eq_asset_get` ImageContent | Planned |
| W5 | `eq_graph_image` layout + raster | Planned |
| W6 | Instructions, stdio, schema parity; keep SPEC-028/152 green | Planned |
| W7 | `spec161_mcp_control_e2e` covers every EC | Planned |

## Decision log

| Decision | Value | Notes |
|----------|-------|-------|
| Spec number | 161 | After SPEC-160 |
| Default profile | control (unset = write tools advertised) | Amends SPEC-152 query-default |
| Query profile | Explicit lockdown | `EDGEQUAKE_MCP_PROFILE=query` |
| Protocol pin | MCP 2026-07-28 | Same as SPEC-152 |

## Official protocol references

- [MCP tools (2026-07-28)](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)
- [MCP resources (2026-07-28)](https://modelcontextprotocol.io/specification/2026-07-28/server/resources)
- [MCP schema](https://modelcontextprotocol.io/specification/2026-07-28/schema)
- [Tasks extension (deferred)](https://tasks.extensions.modelcontextprotocol.io/specification/2026-07-28/tasks)
- [2026-07-28 blog](https://blog.modelcontextprotocol.io/posts/2026-07-28/)
