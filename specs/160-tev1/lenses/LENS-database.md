# LENS — Database Expert (SPEC-160)

Parent: [README](../README.md) · Docs: [07](../07-data-model.md)

## Job

Add the least schema that makes cache and review work. Keep rollout safe and
isolation strict.

## Findings

1. Settings fit in workspace metadata JSON. No column on `workspaces`.
2. Two new tables carry text from customer documents. They need delete
   cascades by code, because the tables follow the SPEC-149 style (no FK to
   `workspaces`).
3. Migration 166 is `expand` only. Old binaries ignore the tables.
4. `compat_serve_max` must rise to 166. `checksums.lock` must change.

## Schema

```text
  decision_cache  PK (workspace_id, key_hash)   IDX (workspace_id, last_used_at)
  decision_review PK review_id                  IDX (workspace_id, document_id)
```

Full column list: [07](../07-data-model.md) §Migration 166.

## Access patterns

| Pattern | Query shape | Cost |
|---------|-------------|------|
| Cache read per pack | `WHERE workspace_id = $1 AND key_hash = ANY($2)` | Primary key lookup |
| Cache write per pack | Upsert on the PK | One round trip |
| Cache touch | `last_used_at` update batched with read | No extra trip |
| TTL sweep | Delete by `(workspace_id, last_used_at)` in batches of 5,000 | Index range |
| Review write | Batch insert per chunk | One round trip |
| Review list | `(workspace_id, document_id)` | Small |

## Isolation

| Rule | Test |
|------|------|
| Every store method takes `workspace_id` | I20 |
| Same text in two workspaces gives two rows | I20 |
| Cache is never read across workspaces | I20 |
| Document delete deletes review rows | I18 |
| Workspace delete deletes both tables' rows | I19 |

Open: OQ-160-3. Check migration 019 to see if the `tasks` RLS policy should
apply. If it does, add policies in 166 and test them.

## Growth

| Table | Bound |
|-------|-------|
| `decision_cache` | TTL 30 days and 200,000 rows per workspace |
| `decision_review` | Deleted with document or on reanalyze |

Estimate: one answer row about 300 bytes. 200,000 rows is about 60 MB per
workspace before index. Check with real data in W3.

## Rollout and rollback

1. Deploy the new binary. It runs migration 166 (SPEC-150 gate).
2. Old pods keep running during a rolling deploy. They never read the tables.
3. Rollback: set `EDGEQUAKE_DECISION_ENABLED=0`. Tables stay. Drop them later by
   a new migration if needed.

## Privacy

The tables hold sentence text. Treat them like chunks: same retention rules,
same backup scope, same deletion on tenant removal.
