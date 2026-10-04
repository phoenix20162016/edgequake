# 07 — Data model

Parent: [README](README.md) · Prev: [06](06-architecture.md) · Next: [08 API](08-api-contract.md)

## Principle

Add no column to an existing table. Use metadata keys for settings. Add two new
tables for cache and review. Make migration 166 additive (`phase = "expand"`).

## Metadata keys

### Workspace metadata (JSON map)

| Key | Type | Values | Default | Writer |
|-----|------|--------|---------|--------|
| `extraction_mode` | string | `llm`, `decision` (absent = inherit) | absent | `apply_extraction_mode_metadata` (W1) |
| `decision_gate_preset` | string | `strict`, `balanced`, `recall` | absent = `balanced` | W4 |
| `decision_model` | string | Model name, such as `tev1:0.8b` | absent = env default | W4 |
| `decision_pack_size` | integer | 1–16 | absent = 4 | W4 |

The workspace never stores `inherit`. The helper deletes the key instead. Reads
treat a missing key and an inherit word as the same state.

### Task and document metadata

| Key | Type | Notes |
|-----|------|-------|
| `extraction_mode` | string | The **resolved** word (`llm` or `decision`), written at admission. The stored value is canonical. |
| `extraction_mode_source` | string | `document`, `workspace`, `env`, `default` |
| `decision_gate_preset` | string | Optional per-upload override (W4). Same word set. |
| `decision_stats` | object | Written at the end of a run: `{accept, review, reject, questions, cache_hits, model, contract}` |

Why store the resolved word: a later change to the workspace default must not
change how an old document reads. The fingerprint reads the stored word
(`ProcessFingerprintInput::from_document_metadata`).

## Precedence table

| Document field | Workspace key | Env | Result | Source |
|----------------|---------------|-----|--------|--------|
| `decision` | any | any | decision | document |
| `llm` | `decision` | `decision` | llm | document |
| absent or `inherit` | `decision` | `llm` | decision | workspace |
| absent | absent | `decision` | decision | env |
| absent | absent | absent | llm | default |
| `tev1` | any | any | error | document |
| absent | `oops` | any | error | workspace |

The resolver checks every layer, even a losing one ([`extraction_mode.rs`](../../edgequake/crates/edgequake-pipeline/src/extraction_mode.rs), test T-160-U06).

## Fingerprint

Format today: `cs|cts|co|mm|ch`. W1 appends `|em=decision` only when the mode
is not `llm`.

```text
  llm       "cs=recursive|cts=800|co=100|mm=ite|ch="                 (unchanged)
  decision  "cs=recursive|cts=800|co=100|mm=ite|ch=|em=decision"
```

| Property | Test |
|----------|------|
| The `llm` digest equals the pre-SPEC-160 digest (pinned hash) | T-160-U12 |
| A mode change makes the document stale in both directions | T-160-U13 |
| Unknown and inherit words read as `llm` | T-160-U14 |
| Old JSON without the field still loads | T-160-U15 |

Model name and gate preset are **not** in the fingerprint. Reason: they change
decisions but not the document shape, and the cache key includes the model
(below). A model switch does not purge. Reanalyze offers an explicit "force
purge" for that case. Open question OQ-160-1 in [13](13-implementation-plan.md).

## Migration 166

File: `edgequake/migrations/166_spec160_decision.sql`

```text
  decision_cache
  ┌────────────┬──────────┬──────────────────────────────────────────────┐
  │ tenant_id  │ UUID     │ NOT NULL                                     │
  │ workspace_id│ UUID    │ NOT NULL                                     │
  │ key_hash   │ TEXT     │ NOT NULL  sha256 of canonical question       │
  │ contract   │ TEXT     │ NOT NULL  e.g. edgextract.decision.2026-10-06│
  │ model      │ TEXT     │ NOT NULL                                     │
  │ answer     │ JSONB    │ NOT NULL  {letter, score, confidence}        │
  │ created_at │ TIMESTAMPTZ DEFAULT now()                               │
  │ last_used_at│ TIMESTAMPTZ DEFAULT now()                              │
  └────────────┴──────────┴──────────────────────────────────────────────┘
  PRIMARY KEY (workspace_id, key_hash)
  INDEX (workspace_id, last_used_at)

  decision_review
  ┌────────────┬──────────┬──────────────────────────────────────────────┐
  │ review_id  │ UUID PK                                                 │
  │ tenant_id, workspace_id │ UUID NOT NULL                              │
  │ document_id│ UUID     │ NOT NULL                                     │
  │ chunk_id   │ TEXT     │ NOT NULL                                     │
  │ kind       │ TEXT     │ CHECK IN ('entity','relation')               │
  │ subject    │ TEXT     │ source mention                               │
  │ label      │ TEXT     │ proposed type or relation                    │
  │ object     │ TEXT     │ NULL for entity rows                         │
  │ score      │ REAL     │ gate score                                   │
  │ band       │ TEXT     │ CHECK IN ('review')  (room for later bands)  │
  │ sentence   │ TEXT     │ source sentence (<= 1000 chars)              │
  │ model, contract │ TEXT NOT NULL                                      │
  │ created_at │ TIMESTAMPTZ DEFAULT now()                               │
  └────────────┴──────────┴──────────────────────────────────────────────┘
  INDEX (workspace_id, document_id)
```

Rules:

1. Use `CREATE TABLE IF NOT EXISTS` and `SET search_path = public;` like
   migrations 153 and 165.
2. Add the manifest entry: `version = 166`, `phase = "expand"`,
   `no_transaction = false`, `lock_class = "ddl_access_exclusive"`.
3. Raise `compat_serve_max` from 165 to 166.
4. Update `checksums.lock` (CI pin).
5. Add RLS only if the `tasks` policy pattern applies (migration 019). The
   store code always filters by `workspace_id`. A test proves it (EC-160-39).
6. Tables hold text from the document (`sentence`, `subject`). Treat them as
   customer data: delete with the document and with the workspace (EC-160-36,
   EC-160-37).

### Cache key

```text
  key_hash = sha256( contract | model | pack_size | question_kind | state | question | options_canonical )
```

One function builds it (`cache_key.rs`). Both backends call it. The model name
and the pack size are in the key. A new model never reads old answers.
A changed pack size also misses the cache, because pack composition changes
answers ([11](11-ml-quality-spec.md)).

### Retention

| Rule | Value |
|------|-------|
| Cache TTL on `last_used_at` | `EDGEQUAKE_DECISION_CACHE_TTL_DAYS`, default 30 |
| Cache size per workspace | `EDGEQUAKE_DECISION_CACHE_MAX_ROWS`, default 200000 |
| Eviction | Delete oldest `last_used_at` first, in a background sweep |
| Review rows | Delete with the document. Delete on reanalyze before re-run. |

### Query cost

| Query | Index | Expectation |
|-------|-------|-------------|
| Cache `get_many` by `(workspace_id, key_hash IN …)` | Primary key | One round trip per pack |
| Review list by document | `(workspace_id, document_id)` | Small |
| TTL sweep | `(workspace_id, last_used_at)` | Batched delete, limit 5,000 per run |

## Provenance

Each entity and relation carries `metadata` keys in the in-memory result
(`extraction_mode`, `decision_model`, `decision_contract`, `score`). Whether
the graph stores them depends on the existing merger. W5 checks the merger path
and stores `extraction_mode` on the document only if the merger drops chunk
metadata (OQ-160-2).

## Stats

Per document: `decision_stats` ([08](08-api-contract.md) shows the shape). The
dashboard cost view reads token cost. Decision runs report 0 cost and a
`decision` label (US-160-6).

Next: [08 API contract](08-api-contract.md).
