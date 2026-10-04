# 13 — Implementation plan

Parent: [README](README.md) · Prev: [12](12-edge-cases.md) · Next: [14 E2E matrix](14-e2e-test-matrix.md)

## Order and dependencies

```text
  W0 spikes ──► W1 mode SSOT ──► W2 extractor + Ollama ──► W3 storage ──► W4 API plumbing
   (gates)       (DONE)               │                        │              │
                                      └────────────────────────┴──────┬───────┘
                                                                      ▼
                                                             W5 factory + status
                                                               │            │
                                                               ▼            ▼
                                                        W6 CPU server    W7 web UI
                                                          backend             │
                                                               └─────┬────────┘
                                                                     ▼
                                                              W8 quality gate
```

W2 and W3 can run in parallel after W0. W6 and W7 can run in parallel after W5.

## Rules for every wave

1. Write the failing test first (red), then the code (green), then refactor.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and the crate tests.
3. Keep each file under 300 lines where possible. Split by reason to change.
4. No duplicate parser, key builder, or digest. See DRY rules in [01](01-first-principles.md).
5. Update [15](15-cross-ref.md) when a wave closes an EC.
6. Document each new env var in `.env.example` and the AGENTS.md table.

## W0 — Spikes and gates

Goal: close the risks that can change the design. No product code.

| Spike | Question | Output | Gate |
|-------|----------|--------|------|
| SP-160-1 | Does `edgextract` build with `--no-default-features` next to sqlx `libsqlite3-sys` 0.30.1? | Build log | Pass: use the crate. Fail: port the protocol. Decide the crate-or-port question ([06](06-architecture.md)). |
| SP-160-2 | Does letter scoring by logprobs match System One answers on 200 questions? | Agreement rate | At least 95% for the `openai_logprobs` backend to ship |
| SP-160-3 | Does the prefix cache help on the hybrid model? Pack 1 vs 4 on 50 chunks | F1 and time | Sets the default pack size |
| SP-160-4 | What is the `top_logprobs` limit for more than 5 options? | Limit | Sets the W6 fallback rule |
| SP-160-5 | What is the final Tev1 weights license? | Link and date | Closed before release |
| SP-160-6 | What are the x86 CPU numbers? | Table in [03](03-cpu-inference-study.md) | W2 exit |
| SP-160-7 | Does a derived CPU model keep `capabilities:["decision"]` after an Ollama update? | Yes or no | W5 status text |
| SP-160-8 | Decide >25 entity types: two rounds or validation error (EC-160-25) | One-line decision | W2 start |

Open questions (answer in W0, record in this file):

| ID | Question |
|----|----------|
| OQ-160-1 | Does a model switch purge derived data, or only the cache key change? |
| OQ-160-2 | Does the merger keep chunk metadata so the graph stores provenance? |
| OQ-160-3 | Does the workspace RLS pattern of migration 019 apply to the new tables? |
| OQ-160-4 | Does the status check call `GET /api/tags` or `POST /api/show` for capabilities? |

## W1 — Mode SSOT, options field, fingerprint — DONE

| File | Change |
|------|--------|
| `edgequake-pipeline/src/extraction_mode.rs` | New. Mode enum, source enum, resolver, validator, metadata helper. 10 tests. |
| `edgequake-pipeline/src/lib.rs` | Module and re-exports |
| `edgequake-pipeline/src/ingestion_pipeline.rs` | `extraction_mode` field, `with_extraction_mode`. 1 test. |
| `edgequake-api/src/services/process_fingerprint.rs` | `extraction_mode` field. Digest adds `\|em=` only for non-default. 4 tests. |

Evidence (2026-10-04): `cargo test -p edgequake-pipeline --lib` 545 passed.
`cargo test -p edgequake-api --lib process_fingerprint` 10 passed.
`cargo clippy -p edgequake-pipeline --all-targets -- -D warnings` clean.

Tests: U01–U15.

## W2 — `DecisionExtractor` and Ollama backend

| Step | File | Notes |
|------|------|-------|
| 1 | `extractor/decision/backend.rs` | Trait and types. No I/O. |
| 2 | `extractor/decision/cache_key.rs` | One key function. |
| 3 | `extractor/decision/{propose,question,prune,gate,map}.rs` | Pure files, test first. Use the crate or a port per SP-160-1. |
| 4 | `extractor/decision/mod.rs` | `DecisionExtractor : EntityExtractor`. Holds trait objects. |
| 5 | `extractor/decision/backends/ollama_system_one.rs` | HTTP adapter. Pack splitter. |
| 6 | `tests/support/fake_system_one.rs` | Axum fake server. It counts requests and can fail on demand. |

DoD:

1. U16–U31, U35 pass.
2. A fake-backend run turns a sample chunk into a valid `ExtractionResult` that
   the merger accepts (U30).
3. SP-160-6 table is in [03](03-cpu-inference-study.md).
4. No file over 300 lines. No `unwrap()` outside tests.

SOLID check: `DecisionExtractor` imports no HTTP or SQL crate.

## W3 — Storage

| Step | File |
|------|------|
| 1 | `edgequake/migrations/166_spec160_decision.sql` |
| 2 | `manifest.toml` entry, `compat_serve_max = 166`, `checksums.lock` |
| 3 | `edgequake-storage/src/decision/{mod,postgres,memory}.rs`: `DecisionStore` impls |
| 4 | Sweep job (TTL and max rows) |
| 5 | Delete hooks in document delete and workspace delete services |

DoD: I17–I21 pass. `make migrate` check passes on a fresh DB and on a DB at 165.
SPEC-150 gates pass.

## W4 — API, admission, workspace plumbing

| Step | File | Notes |
|------|------|-------|
| 1 | `document_admission.rs` | Parse and check in `ingest_text_field`. Add to `task_meta`. Append to fingerprint. |
| 2 | `handlers/pdf_upload/upload.rs` | Same validator and keys (EC-160-12) |
| 3 | `processor/text_insert/prepare.rs` | Read the stored word into `IngestionPipelineOptions` |
| 4 | `services/multimodal/reanalyze.rs` | Already reads the word through W1. Add the purge test. |
| 5 | `workspaces_types/{requests,responses,map}.rs`, `workspace_crud.rs` | New fields. Call `apply_extraction_mode_metadata`. |
| 6 | `edgequake-core/src/extract_budget_metadata.rs` sibling `decision_metadata.rs` | Preset, model, pack helpers (one place) |
| 7 | `config` and `.env.example` | Env vars from [08](08-api-contract.md) |
| 8 | OpenAPI | `make codegen-openapi-refresh`, `spec027_api_contract` |

DoD: I01–I14, I22, I28, I29 pass. The `llm` path shows no behavior change
(existing API tests pass).

## W5 — Factory wiring and status endpoint

| Step | File |
|------|------|
| 1 | `workspace_pipeline_factory.rs`: resolve the mode. If `decision`, build the decision pipeline. |
| 2 | `ingestion_pipeline.rs`: sibling builder `build_decision_pipeline` that takes `Arc<dyn DecisionBackend>` and `Arc<dyn DecisionStore>` |
| 3 | `pipeline/config.rs`: `decision:` prefix counts as local. Fairness resolver update. |
| 4 | `handlers/decision_status.rs` and route `GET /api/v1/decision/status` |
| 5 | Stats writer: `decision_stats` on the document. Cost zero. |
| 6 | Worker health check and cancel checks |

DoD: I11, I12, I15, I23, I24, I26, I27 pass. The status endpoint never hangs
(3 s timeout test).

## W6 — CPU server backend (`openai_logprobs`)

| Step | Notes |
|------|-------|
| 1 | `backends/openai_logprobs.rs` with the fallback rule from [06](06-architecture.md) |
| 2 | Fake OpenAI-compatible server in tests |
| 3 | Docs: a `llama-server` recipe for the operator. EdgeQuake does not run it. |

Gate: SP-160-2 agreement at least 95%. If it fails, mark the backend
"experimental" and keep it off by default.

DoD: U-tests for letter mapping, I-tests with the fake server, L03 live (opt-in).

## W7 — Web UI

| Step | Files |
|------|-------|
| 1 | `constants/extraction-mode.ts`, `lib/upload/extraction-mode-field.ts` (tests first) |
| 2 | Types and API client |
| 3 | `use-decision-status.ts`, `extraction-mode-select.tsx`, `decision-status-indicator.tsx` |
| 4 | Dropzone, manager, perform-upload, PDF form data, reprocess dialog |
| 5 | `workspace-extraction-mode-card.tsx` on both workspace pages |
| 6 | Badge and stats line |
| 7 | i18n en, fr, zh |
| 8 | Playwright `e2e/spec160/*.spec.ts` tagged `@spec160` |

DoD: V01–V09 and E01–E10 pass. `bun run build` passes. Locale parity passes.

## W8 — Quality gate, live test, docs

| Step | Output |
|------|--------|
| 1 | Run the protocol in [11](11-ml-quality-spec.md). Write `measurements/w8-report.md`. |
| 2 | Calibrate presets. Write `measurements/gate-presets.json`. |
| 3 | Run L01 and L02 live. |
| 4 | Update the docs: README, `docs/` guide, `.env.example`, AGENTS.md table. **Done** — [decision extraction](../../docs/concepts/decision-extraction.md). |
| 5 | Add `scripts/spec160-coverage.sh` (EC to test check). |
| 6 | Release gates: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `make release-gates`. Run SPEC-001 Acc (`make bench001-doctor` then `make bench`) per the release runbook, because the ingestion path changed. |

DoD: the report states the measured gap. Pass rules in [11](11-ml-quality-spec.md)
hold, or the mode ships with the "preview" label.

## Status (2026-10-04)

| Wave | State | Notes |
|------|-------|-------|
| W0 | Partly closed | SP-160-1: the published `edgextract` 0.1.0 builds with `default-features = false`, so the crate is used. SP-160-2…8 stay open. |
| W1–W5 | Done | Migration 166, `DecisionStore` (memory and Postgres), `DecisionExtractor` with the Ollama backend, admission, workspace fields, status endpoint, `decision_stats`. |
| W6 | Deferred | No `llama-server` backend yet. |
| W7 | Done | Steps 1–8 except the reprocess dialog select. 17 Playwright tests on the real backend. |
| W8 | Measured | [w8-report](measurements/w8-report.md). SPEC-001 Acc was not run. The mode keeps the "preview" label. Operator guide: [docs/concepts/decision-extraction.md](../../docs/concepts/decision-extraction.md). |

## Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|-----------|
| RK-160-1 | CPU too slow on x86 for real corpora | Medium | High | SP-160-6 gate. Pack 4. Tiered profile later. State limits in UI. |
| RK-160-2 | Weights license blocks use | Medium | High | Opt-in. No bundle. SP-160-5 before release. |
| RK-160-3 | Relation quality too low to be useful | Medium | Medium | W8 gate. "Preview" label. REVIEW band. |
| RK-160-4 | Upstream edgextract API changes | Medium | Low | Pin version. Port option. |
| RK-160-5 | `rusqlite` link conflict | Medium | Medium | SP-160-1. Port fallback. |
| RK-160-6 | Ollama System One changes | Low | Medium | Contract string. Health check. |
| RK-160-7 | Scope creep into a review UI | Medium | Low | Out of scope ([05](05-product-spec.md)) |

## Rollback

`EDGEQUAKE_DECISION_ENABLED=0` blocks new decision uploads. Existing decision
documents keep their graph data. The migration is additive. Dropping the two
tables is safe after disabling the mode.

Next: [14 E2E matrix](14-e2e-test-matrix.md).
