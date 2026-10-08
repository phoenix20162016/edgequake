# 10 — Source: EQ-STE-001 issue 2

Parent: [README](README.md)

This pack implements EQ-STE-001 issue 2 (2026-10-08). Issue 2 replaces issue 1.
Issue 1 named one document. Issue 2 does not name a document.

Repository: https://github.com/raphaelmansuy/edgequake  
Revision: `59ec526abed27c255e6edf9f9d9a403c18cd7a4d`

## Scope files

- `edgequake/crates/edgequake-api/src/mcp/project/ids.rs`
- `edgequake/crates/edgequake-api/src/mcp/project/graph.rs`
- `edgequake/crates/edgequake-api/src/mcp/project/graph_image.rs`
- `edgequake/crates/edgequake-api/src/mcp/project/search.rs`
- `edgequake/crates/edgequake-api/src/mcp/project/catalog.rs`
- `edgequake/crates/edgequake-api/src/mcp/gateway/tools.rs`
- `mcp/README.md`

## Requirements map

| STE | Pack |
|-----|------|
| R1 | LAW-162-1 |
| R2 | LAW-162-2 · W1 |
| R3 | LAW-162-1 · W1 |
| R4 | LAW-162-6 · W1 fold |
| R5 | LAW-162-3 · W1 |
| R6 | LAW-162-4 · W2 |
| R7 | LAW-162-4 · W2 |
| R8 | LAW-162-4 · W2 |
| R9 | LAW-162-5 · W3 |
| R10 | LAW-162-5 · W3 |
| R11 | LAW-162-3 · W4 |
| R12 | LAW-162-3 · W4 |
| R13 | LAW-162-6 · W5 |
| R14 | LAW-162-5 · W4 |
| R15 | LAW-162-1 · W6 |

## Defects map

| STE | Finding |
|-----|---------|
| D1 | F-162-01 |
| D2 | F-162-03 |
| D3 | F-162-04 |
| D4 | F-162-05 |
| D5 | F-162-06 |
| D6 | F-162-07 |
| D7 | F-162-08 |
| D8 | F-162-09 |
| D9 | F-162-10 |
| D10 | F-162-11 |

## Acceptance map

| STE | Test |
|-----|------|
| A1 | T-162-01, T-162-03 |
| A2 | T-162-02 |
| A3 | T-162-20 |
| A4 | T-162-23 |
| A5 | T-162-30 |
| A6 | T-162-35 |
| A7 | T-162-45 |
| A8 | T-162-09 |

## Procedure (STE §7)

Follow [07-implementation-plan](07-implementation-plan.md) waves W1–W7 in order.
Code-fact corrections that refine the STE live in [02-code-facts](02-code-facts.md).
