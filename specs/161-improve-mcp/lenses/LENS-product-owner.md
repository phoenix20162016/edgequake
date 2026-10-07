# LENS — Product Owner

Parent: [README](../README.md) · Primary: [00-why](../00-why.md) · [05-contract-delta](../05-contract-delta.md)

## Job

Give an agent a discoverable control surface: add knowledge, remove a document,
open a figure, see a neighborhood as a picture, and search for evidence —
without leaving the MCP host for REST.

## Findings (from this lens)

- Write tools that lie are worse than missing tools (F-161-01, F-161-02).
- Default query profile hid the control jobs operators asked for (F-161-07).
- Search already works; do not sell a second “answer” tool (LAW-161-5).

## Decisions

1. Control mode is the default advertisement ([README](../README.md) locked #2).
2. Query profile remains for the public demo lockdown.
3. Workspace destroy stays out of scope; stop returning fake success.
4. Success is end-to-end honesty, not feature count.

## Success measures

| Measure | Evidence |
|---------|----------|
| Agent can ingest and see the document | EC-161-09 + eq_document_get |
| Agent cannot delete without confirm | EC-161-21 |
| Agent can open a figure | EC-161-31 |
| Agent can see a centered graph PNG | EC-161-36 |
| Agent can search evidence | EC-161-46 |

This lens does not invent laws. Laws live in [01](../01-first-principles.md).
