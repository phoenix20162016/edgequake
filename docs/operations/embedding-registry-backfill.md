---
title: Embedding registry audit & backfill
description: List embedding_models rows and fix name mismatches without silent cross-model ANN.
---

> **Product: v0.32.0+** · Related: [Embedding models deep dive](/docs/deep-dives/embedding-models/)

# Embedding registry audit & backfill

Typed ANN is keyed by `embedding_models(name, dimensions)`. If vectors were
ingested under `mistral-embed@1024` but the process env / preferred filter points
at `text-embedding-3-small`, query ANN returns empty. EdgeQuake does **not**
silently search another model’s space — fix the registry and row stamps instead.

## 1. List registry + per-workspace counts

Replace the demo workspace id as needed (`…0003` is the common local demo):

```sql
-- Registry rows
SELECT name, dimensions, created_at
FROM embedding_models
ORDER BY name, dimensions;

-- Chunk embedding counts by model for one workspace
SELECT em.name, em.dimensions, COUNT(*) AS chunk_rows
FROM chunk_embeddings ce
JOIN embedding_models em ON em.id = ce.model_id
WHERE ce.workspace_id = '00000000-0000-0000-0000-000000000003'
GROUP BY em.name, em.dimensions
ORDER BY chunk_rows DESC;

-- Entity / relationship / report (fleet) families — same join pattern
SELECT em.name, em.dimensions, COUNT(*) AS entity_rows
FROM entity_embeddings ee
JOIN embedding_models em ON em.id = ee.model_id
WHERE ee.workspace_id = '00000000-0000-0000-0000-000000000003'
GROUP BY em.name, em.dimensions
ORDER BY entity_rows DESC;
```

## 2. Detect common mismatches

| Symptom | Likely cause |
| ------- | ------------ |
| Rows under `text-embedding-3-small` or `''` but vectors are 1024-d | Env default stamped at ingest; real embedder was `mistral-embed` |
| Query empty with correct workspace embedder | Preferred filter name ≠ registry `name` at that dim |
| Empty Compose `EDGEQUAKE_EMBEDDING_MODEL=` | Must resolve via `embedding_model_key_from_env()`, not literal empty string |

## 3. Backfill (explicit rename — no cross-model search)

Only when you have confirmed the vectors were produced by the target model at
the target dimensions. Example: move a mistaken `text-embedding-3-small@1024`
stamp to `mistral-embed@1024` for one workspace.

```sql
BEGIN;

-- Ensure destination registry row exists
INSERT INTO embedding_models (name, dimensions)
VALUES ('mistral-embed', 1024)
ON CONFLICT (name, dimensions) DO NOTHING;

-- Point chunk rows at the destination model id (workspace-scoped)
UPDATE chunk_embeddings ce
SET model_id = (
  SELECT id FROM embedding_models
  WHERE name = 'mistral-embed' AND dimensions = 1024
)
WHERE ce.workspace_id = '00000000-0000-0000-0000-000000000003'
  AND ce.model_id IN (
    SELECT id FROM embedding_models
    WHERE name IN ('text-embedding-3-small', '')
      AND dimensions = 1024
  );

-- Repeat for entity_embeddings / relationship_embeddings / report_embeddings
-- with the same WHERE workspace_id + source model_id filter.

COMMIT;
```

After backfill, re-check the counts in §1. Leave orphan registry rows if other
workspaces still use them; do not delete globally without a fleet audit.

## 4. Verify ANN with the preferred filter

With `EDGEQUAKE_VECTOR_BACKEND=typed_embeddings`, a `query_filtered` call (or
Ask in the UI) using `MetadataFilter.embedding_model = 'mistral-embed'` and a
1024-d query vector must hit. The same query with a wrong name at that dim must
stay empty.

Automated proof: `cargo test -p edgequake-storage --features postgres --test e2e_typed_ann_model_name_hit_vs_miss`.

## 5. Demo audit snapshot (local)

Run against `DATABASE_URL` when available. Example result on a local demo DB
(2026-10-07): workspace `…0003` had only `text-embedding-3-small` registry rows
(768 and 1024), with chunk/entity counts under that name — **no** `mistral-embed`
row. Do **not** rename to `mistral-embed` unless you have confirmed the vectors
were produced by that model; mixed dims under one logical name still need a
fleet decision before backfill.

Note: a preferred workspace model that is **not** registered at the query dim
returns empty ANN (no fallthrough into `text-embedding-3-small`). Named-entity
Ask still works via graph label/seed admit; general chunk RAG needs a correct
registry stamp or rebuild.
