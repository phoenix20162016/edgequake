# LENS — Database Expert (SPEC-159)

Parent: [README](../README.md) · Contract: [07](../07-retrieval-contract.md)

## Job

Confirm Ask is a **read + existing chat write** path with **zero schema
change**, and that honesty about filters matches what Postgres/AGE can enforce
today.

## Findings

1. **Neighborhood** is already implemented (`build_entity_neighborhood`) —
   workspace/tenant scoped via existing handlers.
2. **`document_ids`** filter exists and is the only document scoping lever on
   chat.
3. **Page numbers** live on chunk metadata for citations/viewers — not on
   `document_filter`.
4. **No migration** in DoD. Adding page predicates is a future API+engine
   change with Acc impact.

## Data flow

```text
  AGE graph ──GET neighborhood──► companion (read)
  chunk KV  ──page_start────────► viewer URL only
  chat DB   ◄──on submit only─── conversation + messages
```

## Risks

| Risk | Mitigation |
|------|------------|
| Orphan entity id in URL | 404 pane; no DB write |
| Scope chip accumulates forever | Existing user can remove chips; Ask only adds |
| Accidental full-corpus queries | Document Ask adds id; entity Ask relies on keywords in text |

## Tests that matter to DB

- Existing neighborhood e2e (regression).
- No new SQL fixtures required for WebUI hermetic Playwright (mock API).

## Cross-ref

LAW-159-6,11 · F-159-04 · EC-159-08,10,11,12 · OPP-159-06
