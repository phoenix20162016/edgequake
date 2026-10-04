# LENS — Full Stack Developer (SPEC-160)

Parent: [README](../README.md) · Docs: [04](../04-code-as-is.md), [06](../06-architecture.md), [08](../08-api-contract.md)

## Job

Find the shortest safe path from a form field to a graph row. Keep each change
small and in one place.

## Request path

```text
 UI select ─► multipart field ─► MultipartUploadFields.ingest_text_field ─► check (W1 fn)
                                                                               │
 workspace PATCH ─► apply_extraction_mode_metadata (W1 fn)                      ▼
                                                                  task_meta.extraction_mode
                                                                               │
 prepare.rs ◄──────────────────────────────────────────────────────────────────┘
     │ IngestionPipelineOptions.with_extraction_mode
     ▼
 WorkspacePipelineFactory.resolve_for_ingestion ─► build_decision_pipeline | build_ingestion_pipeline
```

## Findings

1. The PDF route is a **separate handler** (`pdf_upload/upload.rs`). Skipping it
   would leave PDFs on `llm` while the UI says `decision`. EC-160-12.
2. The fairness clamp reads the **chat LLM provider name**. Without a fix, a
   decision run on a workspace with a cloud chat provider gets a high worker
   cap and overloads the CPU. EC-160-18.
3. `IngestionPipelineOptions` and `ProcessFingerprintInput` have no struct
   literals outside their own constructors. Adding fields was safe (W1).
4. The `compat_serve_max = 165` guard in the migration manifest will block
   boot after migration 166 unless raised. EC-160-40.
5. The merger citation gate needs source chunk ids on every row. The mapper
   sets them (U23).

## SRP/DRY map

| Concern | One home |
|---------|----------|
| Mode words and precedence | `extraction_mode.rs` |
| Workspace key write | `apply_extraction_mode_metadata` |
| Upload word validation | `parse_mode_override` |
| Fingerprint | `ProcessFingerprintInput::digest` |
| Decision cache key | `cache_key.rs` |
| Backend transport | `DecisionBackend` adapters |
| Gate | `gate.rs` |

## Review checklist per PR

- [ ] No new parser for mode words
- [ ] No `unwrap()` in non-test code
- [ ] A bad value never reaches an LLM call
- [ ] `llm` path diff is empty (or only a field pass-through)
- [ ] New env var in `.env.example` and AGENTS.md
- [ ] OpenAPI refreshed if a DTO changed
- [ ] File under 300 lines

## Failure behavior

| Failure | Result |
|---------|--------|
| Bad word | 422 at upload, 400 at workspace write |
| Backend down at upload | 422 `decision_backend_unavailable` |
| Backend down in worker | Task failed, retryable, no mode switch |
| Contract mismatch | Task failed, no retry |

## Observability

Log with `tracing`: mode, source, backend kind, model, pack size, band counts,
cache hits. Never log chunk text, API keys, or full URLs.
