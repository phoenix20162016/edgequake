# 08 — API contract

Parent: [README](README.md) · Prev: [07](07-data-model.md) · Next: [09 UX](09-ux-ui-spec.md)

## As built (2026-10-04)

| Item | Built shape |
|------|-------------|
| `GET /api/v1/decision/status[?model=]` | `{enabled, activation, settings_error?, provider?, backend?, limits, license_notice}`. `activation` is `locked`, `inactive`, `active`, or `forced`. `backend` holds `kind`, `base_url_host`, `model`, `contract`, plus health: `reachable`, `model_present`, `decision_capable`, `supported`, `latency_ms`, `cpu_pinned`. `limits` holds `pack_size_default`, `pack_size_min`, `pack_size_max`, `gate_presets`. `settings_error` carries the code of a bad env value. The `model` query probes a workspace model. |
| `GET /api/v1/decision/models[?model=]` | `{activation, provider?, models[], error?}`. Each model is `{name, decision_capable?}`. |
| Workspace PUT | `extraction_mode`, `decision_gate_preset`, `decision_model` (`inherit` clears), `decision_pack_size` (`0` clears). Responses flatten the same fields plus `effective_extraction_mode`. |
| Document list and detail | Flat `extraction_mode`, `extraction_mode_source`, `decision_stats`. Absent for plain LLM documents. |
| `extraction_mode_source` | `document` (the upload asked), `workspace`, `env`, or `default` (nobody asked: `llm`). Words outside this list read as `document`. |
| `decision_stats` | `{chunks, entities, relations, review, rejected, backend_calls, cache_hits, warnings, model, source}` |
| 422 codes | `decision_backend_unavailable`, `decision_model_missing`, `decision_disabled`, `decision_not_activated`, `invalid_extraction_mode`. Each has a readable message. |

All changes are additive. A client that sends no new field gets today's
behavior (LAW-160-6).

## Mode words

| Word | Meaning | Allowed at |
|------|---------|-----------|
| `llm` | Open extraction by chat LLM | document, workspace, env |
| `decision` | Closed decisions by a decision model | document, workspace, env |
| `inherit`, empty, `none` | No override. Pass control to the next layer. | document (workspace write clears the key) |
| Anything else | Error | — |

Case and surrounding spaces do not matter. The canonical word is lower case.

## Upload

### Multipart field

`extraction_mode` on four routes (`routes.rs:419-440`):

| Route | Handler |
|-------|---------|
| `POST /api/v1/documents/upload` | `upload_file` |
| `POST /api/v1/documents/upload/batch` | batch upload |
| `POST /api/v1/documents/pdf` | `upload_pdf_document` |
| `POST /api/v1/documents/pdf/batch` | PDF batch |

Handled in one place: `MultipartUploadFields::ingest_text_field`
(`document_admission.rs:576`). The separate PDF handler calls the same
validator (EC-160-12).

Optional field: `decision_gate_preset` (`strict`, `balanced`, `recall`).

### JSON text upload

`POST /api/v1/documents` body gains `extraction_mode` (string, optional).

### Responses

Success body gains:

```json
{
  "extraction_mode": "decision",
  "extraction_mode_source": "workspace"
}
```

Errors:

| Status | `code` | When |
|--------|--------|------|
| 422 | `invalid_extraction_mode` | Unknown word at any layer. The message names the layer and lists allowed words. |
| 422 | `decision_backend_unavailable` | Mode resolves to `decision` and the backend health check fails |
| 422 | `decision_model_missing` | Backend is up and the model is not pulled. The message includes the pull command. |
| 422 | `decision_disabled` | `EDGEQUAKE_DECISION_ENABLED` is `0` and mode resolves to `decision` |
| 422 | `decision_not_activated` | Enabled is `workspace` and this workspace has not opted in |

Example error:

```json
{
  "error": {
    "code": "invalid_extraction_mode",
    "message": "Unsupported extraction_mode 'tev1' from document. Allowed: inherit, llm, decision"
  }
}
```

The message text comes from `UnknownExtractionMode::fmt` (W1). One text, two
call sites (DRY).

Batch upload: the mode applies to every file in the request. A batch with a
bad word fails as a whole before any file is stored (EC-160-13).

## Workspace

### Request DTOs (create and update)

```json
{
  "extraction_mode": "decision",
  "decision_gate_preset": "balanced",
  "decision_model": "tev1:0.8b",
  "decision_pack_size": 4
}
```

| Rule | Behavior |
|------|----------|
| Field absent | Leave the key unchanged |
| `inherit`, empty, `none` | Delete the key |
| `llm`, `decision` | Store the canonical word |
| Unknown word | 400 `invalid_extraction_mode`. Metadata unchanged. |
| `decision_pack_size` outside 1–16 | 400 `invalid_decision_pack_size` |
| `decision_gate_preset` unknown | 400 `invalid_decision_gate_preset` |

The Postgres and in-memory workspace services call
`apply_extraction_mode_metadata` (W1). They never write the key by hand.

### Response DTO

`WorkspaceResponse` gains:

```json
{
  "extraction_mode": "decision",
  "effective_extraction_mode": { "mode": "decision", "source": "workspace" },
  "decision_gate_preset": "balanced",
  "decision_model": "tev1:0.8b",
  "decision_pack_size": 4
}
```

`extraction_mode` is `null` when the key is absent. `effective_extraction_mode`
folds in the env layer so the UI can show "inherited from server".

## Status endpoint

`GET /api/v1/decision/status` (authenticated, tenant scoped).

```json
{
  "enabled": true,
  "activation": "forced",
  "provider": {
    "kind": "ollama_system_one",
    "label": "Ollama",
    "base_url_host": "localhost:11434"
  },
  "backend": {
    "kind": "ollama_system_one",
    "base_url_host": "localhost:11434",
    "model": "tev1:0.8b",
    "contract": "edgextract.decision.2026-10-06",
    "reachable": true,
    "model_present": true,
    "decision_capable": true,
    "supported": true,
    "latency_ms": 41,
    "cpu_pinned": null
  },
  "limits": {
    "pack_size_default": 4,
    "pack_size_min": 1,
    "pack_size_max": 16,
    "gate_presets": ["strict", "balanced", "recall"]
  },
  "license_notice": "Tev1 weights license is not final. See the model card."
}
```

Rules:

1. Show host only. Never show credentials or full URLs (EC-160-35).
2. `cpu_pinned` is true only if the backend reports it. Else `null`.
3. The endpoint times out after 3 s and returns `reachable: false`. It never hangs.
4. It returns 200 even when the backend is down. The body carries the state.

## Document read

`GET /api/v1/documents/{id}` metadata gains `extraction_mode`,
`extraction_mode_source`, and `decision_stats` (when present).

```json
"decision_stats": {
  "chunks": 2,
  "entities": 41,
  "relations": 9,
  "review": 9,
  "rejected": 120,
  "backend_calls": 388,
  "cache_hits": 17,
  "warnings": [],
  "model": "tev1:0.8b",
  "source": "workspace"
}
```

## Environment variables

Document each in `.env.example` and the AGENTS.md table (W4).

| Variable | Default | Meaning |
|----------|---------|---------|
| `EDGEQUAKE_EXTRACTION_MODE` | unset (= `llm`) | Fleet default mode word |
| `EDGEQUAKE_DECISION_ENABLED` | unset (on) | `1` or unset forces the engine on. `0` locks it off. `workspace` lets each workspace opt in. |
| `EDGEQUAKE_DECISION_BACKEND` | `ollama_system_one` | `ollama_system_one` (`openai_logprobs` is deferred and refused at boot) |
| `EDGEQUAKE_DECISION_BASE_URL` | `http://localhost:11434` | Backend URL. Does not follow `OLLAMA_HOST`. |
| `EDGEQUAKE_DECISION_API_KEY` | unset | Bearer token for `openai_logprobs`. Never logged. |
| `EDGEQUAKE_DECISION_MODEL` | `tev1:0.8b` | Default model |
| `EDGEQUAKE_DECISION_PACK_SIZE` | `4` | Questions per request (1–16) |
| `EDGEQUAKE_DECISION_GATE_PRESET` | `balanced` | Fleet gate preset |
| `EDGEQUAKE_DECISION_TIMEOUT_SECS` | `600` | Per request timeout (matches local extraction) |
| `EDGEQUAKE_DECISION_KEEP_ALIVE` | `30m` | Passed to Ollama so the model stays loaded |
| `EDGEQUAKE_DECISION_CACHE_TTL_DAYS` | `30` | Cache TTL |
| `EDGEQUAKE_DECISION_CACHE_MAX_ROWS` | `200000` | Cache size per workspace |

An invalid value for any variable fails startup with a clear message. It does
not fall back (LAW-160-4). `EDGEQUAKE_EXTRACTION_MODE` is checked by the
resolver on every ingest and on a startup self-check.

## OpenAPI

Run `make codegen-openapi-refresh` after W4 and W5. Run
`cargo test -p edgequake-api --test spec027_api_contract`. Both are release
gates ([13](13-implementation-plan.md)).

Next: [09 UX](09-ux-ui-spec.md).
