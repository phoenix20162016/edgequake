# LENS — MCP Expert

Parent: [README](../README.md) · Primary: [05-contract-delta](../05-contract-delta.md)

## Job

Keep the gateway honest to MCP `2026-07-28`: `structuredContent` for typed
JSON, `ImageContent` / blob resources for bytes, budgets for tokens.

## Official grounding

| Spec | URL |
|------|-----|
| Tools | https://modelcontextprotocol.io/specification/2026-07-28/server/tools |
| Resources | https://modelcontextprotocol.io/specification/2026-07-28/server/resources |
| Schema | https://modelcontextprotocol.io/specification/2026-07-28/schema |

## Decisions

1. Entity handles live in `structuredContent` (agent id strings).
2. Graph PNG stays `ImageContent` (SPEC-161).
3. Download pages stay blob resources with paging metadata in structured.
4. Text pages are structured (`text_page`), URI optional second field.
5. Schema SSOT remains under SPEC-152 `schemas/`.
6. Stdio is a bridge; no second budget engine.

## Cursor rule

`{object}:{offset}` where object ∈ {entities, relationships, chunks, bytes}.
Do not emit a chunks cursor when entities were omitted.
