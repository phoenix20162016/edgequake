# LENS — MCP Expert

Parent: [README](../README.md) · Primary: [05-contract-delta](../05-contract-delta.md) · [04-architecture](../04-architecture.md)

## Job

Keep the EdgeQuake gateway honest to MCP `2026-07-28`: content types,
annotations, deterministic tool lists, explicit handles, and budgets. Defer
Tasks and elicitation deliberately.

## Official grounding

| Spec | URL |
|------|-----|
| Tools | https://modelcontextprotocol.io/specification/2026-07-28/server/tools |
| Resources | https://modelcontextprotocol.io/specification/2026-07-28/server/resources |
| Schema (`ImageContent`) | https://modelcontextprotocol.io/specification/2026-07-28/schema |
| Tasks extension (deferred) | https://tasks.extensions.modelcontextprotocol.io/specification/2026-07-28/tasks |
| Blog (stateless core, MRTR, Tasks) | https://blog.modelcontextprotocol.io/posts/2026-07-28/ |

## Findings

- Gateway has no elicitation / `input_required` path today.
- `structuredContent` + short summary text is the SPEC-152 deviation; keep it.
- Body cap 1 MiB forces upload handles (protocol stateful-tools guidance).

## Decisions

1. **ImageContent** for png/jpeg illustrations and graph PNG.
2. **Blob resource / embedded resource** for original and markdown chunks.
3. **Upload `upload_id`** is an explicit handle; core MCP has no session.
4. **Do not implement** `io.modelcontextprotocol/tasks` in this pack; poll
   `eq_task_get` (allowed by SPEC-152 when hosts lack Tasks).
5. **Do not implement** elicitation/MRTR; confirm stays a tool argument.
6. **tools/list** order deterministic; control mode lists write tools even
   before confirm (scope enforced on call).
7. Schema SSOT remains SPEC-152 `schemas/`; Rust catalog must match.
8. Stdio is a bridge; no second budget engine (LAW-161-9).

## Success measures

- Tool annotations match [05](../05-contract-delta.md) matrix.
- EC-161-39 ImageContent PNG magic.
- EC-161-52 body cap enforced.
- Schema parity test in W6.
- SPEC-028 transport suites remain green.
