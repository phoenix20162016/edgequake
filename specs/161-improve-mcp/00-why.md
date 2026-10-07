# 00 — WHY

Parent: [README](README.md) · Next: [01 First principles](01-first-principles.md)

## The problem in one sentence

Agents that connect to EdgeQuake over MCP can search the graph, but they
cannot honestly add knowledge, remove a document, open an illustration, or see
a neighborhood as a picture — and the write tools that advertise those jobs
return fake success.

## Five WHYs

**WHY-161-1 — Why expand the control surface at all?**  
The agent’s job is to act on a Graph-RAG store: add text or a file, delete a
document, fetch a figure, navigate the graph, and search for evidence. Today
only search is real on the default gateway. REST already admits, deletes, and
serves bytes. MCP does not call those paths.

```text
  Agent job                          Gateway today
  ─────────                          ─────────────
  add knowledge          ──►         stub eq_ingest (fake document_id)
  delete document        ──►         stub eq_document_delete (deleted:true)
  open illustration      ──►         no tool
  see graph around node  ──►         JSON neighborhood only
  query search           ──►         eq_search REAL → retrieve_context
```

ROOT → LAW-161-1 (claimed side effect = REST side effect)

**WHY-161-2 — Why do stubs fail worse than missing tools?**  
A missing tool forces the host to use REST. A stub that returns
`document_id` or `deleted: true` lets the model believe the corpus changed.
The next `eq_document_list` or `eq_search` contradicts that belief. Silence
and contradiction burn the session.

```text
  stub ok:true, deleted:true
           │
           v
  model: "document removed"
           │
           v
  eq_document_get → still there  ──►  trust collapse
```

ROOT → LAW-161-1

**WHY-161-3 — Why keep bytes out of the JSON envelope?**  
SPEC-152 budgets cap `structuredContent` at 8 / 24 / 80 KiB. A page PNG or
PDF fragment exceeds that. MCP `2026-07-28` already defines `ImageContent`
and blob resource contents. Mixing pixels into the envelope forces either
truncation lies or budget violations.

ROOT → LAW-161-2

**WHY-161-4 — Why keep the 1 MiB body cap and add an upload handle?**  
`MCP_MAX_BODY_BYTES` is 1 MiB. A PDF does not fit one `tools/call`. Raising
the cap fights every proxy and the protocol’s own guidance for stateful
handles. An explicit `upload_id` (begin / write / commit) is the first-
principles fit: the core protocol has no session, so state is a named handle.

ROOT → LAW-161-3

**WHY-161-5 — Why advertise writes by default and still require scope + confirm?**  
Operators asked for a control surface that agents can discover without setting
`EDGEQUAKE_MCP_PROFILE=memory`. Discovery is not authorization. Write calls
still need `edgequake:write`. Deletes still need `confirm: true`. The public
demo sets `query` to hide writes. That split is the trust class from
LAW-152-7, amended only on advertisement, not on execution.

ROOT → LAW-161-6

## Causal ASCII

```text
  Operator wants agent control
           │
           v
  +------------------+     stubs mint ids / deleted:true
  | advertise tools  |────► agent believes corpus changed
  +--------+---------+               │
           │                         v
           │                  list/search disagree ──► session failure
           v
  call real admit / delete / mm_assets / neighborhood
           │
           +── ImageContent / blob for bytes (LAW-161-2)
           +── upload_id for large files (LAW-161-3)
           +── write scope + confirm (LAW-161-6)
           +── evidence-only search (LAW-161-5)
```

## What this pack does not claim

- That extraction quality improves
- That every host will render `ImageContent`
- That graph PNG matches the Web UI Sigma export
- That MCP Tasks or elicitation are implemented
- That workspace destroy works through MCP

## Success narrative

An agent with a write-scoped key can ingest a note, poll the task until the
document appears, download a figure as PNG, render a neighborhood centered on
an entity, search for evidence, and delete the note only when it passes
`confirm: true`. A read-only key can search and fetch images, and receives
403 on write. A query-profile lockdown omits write tools from `tools/list`.

## Cross-ref

| WHY | Laws | Surfaces | Findings |
|-----|------|----------|----------|
| WHY-161-1 | LAW-161-1, 5, 8 | [02](02-surfaces.md) | F-161-01, F-161-02 |
| WHY-161-2 | LAW-161-1 | [02](02-surfaces.md) | F-161-01, F-161-02, F-161-08 |
| WHY-161-3 | LAW-161-2, 7 | [04](04-architecture.md) | F-161-04, F-161-05 |
| WHY-161-4 | LAW-161-3 | [04](04-architecture.md) | F-161-06 |
| WHY-161-5 | LAW-161-6 | [05](05-contract-delta.md) | F-161-07 |
