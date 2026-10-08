# LENS — Database Expert

Parent: [README](../README.md) · Primary: [04-architecture](../04-architecture.md) · LAW-162-6

## Job

Keep graph identity stable. Do not migrate existing node ids. Fold is for
lookup and merge compare only.

## Facts

- Production keys: `{workspace_uuid}::NORMALIZED_NAME` (AGE `eq_node_id`).
- Normalizer keeps hyphens (`NEW-YORK`).
- Vectors use `entity:NAME` derived from the same `EntityId`.

## Lookup cost

1. Candidate list from `exact_lookup_candidates` (≤3).
2. Separator variants (cap 16, max 4 separators) via one `get_nodes_batch`.
3. Optional bounded `search_nodes` with fold equality.

## Merge residual

In-process keyed lock per `(workspace, fold_key)` closes same-replica races.
Multi-replica concurrent variant ingest can still duplicate. Mitigations:

1. Metric `entity_fold_duplicate_suspect`.
2. Follow-up: property `fold_key` + unique index migration (out of this pack).

## Isolation

Tenant/workspace mismatch → not_found. No cross-workspace fold merge
(EC-162-49).
