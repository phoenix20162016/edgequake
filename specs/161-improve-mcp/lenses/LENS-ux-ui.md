# LENS — UX / UI Designer

Parent: [README](../README.md) · Primary: [05-contract-delta](../05-contract-delta.md) · [00-why](../00-why.md)

## Job

Shape the **agent-facing** copy and states so hosts and models can tell queued
from accepted from failed. This pack does not redesign the Web UI.

## Findings

- Stub `deleted: true` trains models to lie to users (WHY-161-2).
- Async delete returns acceptance, not completion (LAW-161-8).
- Truncation and `next_offset` must be honest for downloads.

## Decisions

| State | User/agent-facing meaning |
|-------|---------------------------|
| `queued` | Ingest accepted; poll `eq_task_get` |
| `accepted` | Delete job accepted; `deleted: false` until task completes |
| `completed` / `failed` | Terminal task status from `eq_task_get` |
| `eq/confirm_required` | Destructive call blocked; ask human; pass `confirm: true` |
| `next_offset` | More bytes remain; call again |

Labels on graph PNG: Title Case for display; entity slug may stay UPPER_SNAKE
in structured metadata (SPEC-152 Title Case rule).

Tool descriptions MUST say when to list before “what’s in the workspace”, and
that search returns evidence, not an essay.

## Success measures

- Instructions on control profile never say “query-only”.
- Delete response never claims `deleted: true` on accept (EC-161-27).
- Download chunks expose `next_offset` when truncated (EC-161-48).
