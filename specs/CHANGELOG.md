# Changelog (specs)

All notable changes to the EdgeQuake specs directory are tracked here. See the root CHANGELOG.md for workspace-wide changes.

## [Unreleased]

### Added

- **SPEC-160 / decision extraction (2026-10-04):** Preview KG mode `decision`.
  Closed questions on Ollama System One (default `tev1:0.8b`). Accept rows enter
  the graph; review rows land in `decision_review` (migration **166**). Chat-LLM
  extraction stays the default. `llama-server` / `openai_logprobs` is deferred
  and refused at boot. Gate presets stay uncalibrated. Operator guide:
  [docs/concepts/decision-extraction.md](../docs/concepts/decision-extraction.md).
  Pack: `specs/160-tev1/`. Quality probe:
  [measurements/w8-report.md](160-tev1/measurements/w8-report.md).

- **SPEC-144 / Next.js 16.3.3 Active LTS (2026-08-30):** Pin `next` +
  `eslint-config-next` to **16.3.3** (August Critical RCEs); unify auth +
  swagger into `src/proxy.ts`; Docker `next build --webpack`; Instant
  Navigations prepared but flags off. Pack: `specs/144-update-nextjs/`.
  Product cut: **v0.26.4**.

- **SPEC-129 / touch_document_status CHECK (#381) (2026-08-16):** Dual-write
  SSOT gap — slim-checkpoint resume writes KV `re_embedding` while
  `touch_document_status` bypassed `normalize_documents_column_status`,
  violating `documents_valid_status`. Fix: `relational_documents_status_for_write`
  (normalize + `completed`→`indexed`) on all relational status writers; no CHECK
  widen. Pack: `specs/129-touchd_document_faill/`.

- **SPEC-128 / PDF precision + layout overlay (2026-08-16):** Close the
  figure-filter control loop (classify → prune `figure_map`), tighten Image/Form
  area+aspect gates in pdf2md, persist `document_pages` / `page_layout_regions`
  (migration 148), derive `bbox_norm` at read, overlay on `PDFViewer` (O + chips).
  Text-page quads → `paragraph` + derived `column`. L2 ONNX is **out of this
  slice**.   Proof: `make spec128-proof` (unit + HTTP/RLS/cascade contracts +
  G-overlay fixture IoU ≥ 0.8 `S01–S05`; live real-PDF overlay `R01–R05` /
  SPEC-049 ingest `I01–I05` when `make dev` is up); screenshots under
  `specs/128-improve-pdf-parsing/e2e/screenshots/`.

- **SPEC-108 / extraction density vs LightRAG (2026-08-04):** Partner
  ~12k entities/relations question — first-principles pack under
  `specs/108-extraction-compared-light-rag/` (LAW-X1..X5: M≠U count SSOT,
  adaptive geometry, fair dual-SUT). Code compare EQ↔LR chunk/extract/merge;
  arms A/B/C measurements; French partner reply. Docs + measurements only.

- **SPEC-107 / partner prod error report (2026-08-04):** First-principles
  partner-incident pack under `specs/107-issue/` for Quantalogic prod log classes
  (workspaces.id 42703, edgequake.Node 42P01, INV-03 CRITICAL, tenants_slug 23505).
  DRY against SPEC-104 (code fixed ≥0.24.0); residual INV-03 ops runbook + French
  reply; INV-03 `RepairAction::LogOnly` guidance in StorageInspector; E2E-107
  absence gates via `contract_spec104_datalayer` + `e2e_107_03_inv03_logonly_repair`.
  Post-assessment harden: INV-03 covers `indexed|completed`; INV-C skip is
  fail-visible; [07-residual-risks.md](107-issue/07-residual-risks.md).
  **R2 57014:** INV-C chunks via public `SOURCE_PREFIX_BATCH_LIMIT` (LAW-H1);
  list soft-fail tags dataop; [08-r2-node-count-57014.md](107-issue/08-r2-node-count-57014.md)
  — no timeout raise.

- **SPEC-049 / P3 StructTree L0 (2026-07-12):** pdf2md **0.9.7**
  `PdfiumStructTreeProposer` (public FFI via `get_handle_from_page`); tagged
  fixture E13; telemetry `pages_with_struct_tree` / `l0_regions`. Untagged arXiv
  corpora remain ObjectCluster-only. First principles: geometry from MCID/BBox
  only — Alt labels, never invents regions.

- **SPEC-049 / P2 ink residual propose (2026-07-12):** Chart crops proposed from
  pages lacking fig/table + ink area gates (`chart_residual_candidate_pages`);
  `text_suggests_chart` is specialize-router only. Clippy `-D warnings` clean on
  `edgequake-pdf` + pdf2md visual path; e2e 12 + contract 7 green.

- **SPEC-049 / P1a–P1c non-flaky lift (2026-07-12):** pdf2md **0.9.6** placement-first
  L1 (atomic Image/Form seeds) + Form-first precision (`refine_proposals`); EdgeQuake
  IoU merge write policy (no any-embed page skip); include pages = all markdown markers.
  E2E: `e2e_spec049_visual_regions` (11) green.

- **SPEC-049 / non-flaky improvement brainstorm (2026-07-12):** First-principles
  levers (CTM Do inventory, ink residual, Form-first precision, StructTree L0)
  without English detectors — `005-non-flaky-improvement-brainstorm.md`.

- **SPEC-049 / improve figure extraction (2026-07-12):** Formal plan under
  `specs/049-improve-figure-extraction/` (cascade L0–L3, ban caption/keyword
  detectors). pdf2md **0.9.5** `pipeline::visual` object-cluster extract +
  caption labeling; EdgeQuake consumes via existing region_assets facade.
  Tests: `contract_spec049_visual_regions`, `e2e_spec049_visual_regions`.

- **SPEC-047 / figure images in markdown viewer (2026-07-12):** Bind VLM-hallucinated
  `![…](…)` to durable `assets/page-NNNN.png`; emit viewer image at page top;
  `ContentRenderer` rewrites to `GET …/assets/{asset_id}` (auth blob). Fixes broken
  Figure images in document detail split view.
- **SPEC-047 / durable mm-assets asset_id REST (2026-07-12):** Stable `asset_id`
  (filename stem) + migration `085`; `GET /documents/{id}/assets` list and
  `GET /documents/{id}/assets/{asset_id}` binary; persist on vision ingest;
  explicit delete with document (DB + FS). E2E: `e2e_spec047_mm_assets_db`.
- **SPEC-047 / durable mm-assets (2026-07-12):** Vision page/chart PNGs persist in
  Postgres `document_mm_assets` (BYTEA) with `page_num` lineage + workspace RLS;
  cascade-delete with document. Serve/analyze prefer DB (FS cache fallback).
  Lineage API returns `mm_assets` summaries. Migration `084`. E2E:
  `e2e_spec047_mm_assets_db`.
- **SPEC-047 / MV-26/27/28 (2026-07-11):** Routing hygiene (Pass A body in drawing
  caption), specialize soft-fail (keep Pass A numeric dump), page-local viewer
  images (`![alt](assets/…)` + `<drawing/>`, `GET …/mm-assets/…`,
  `AuthenticatedMarkdownImage`). DRY SSOT in `drawing_tags.rs` /
  `vision_markdown.rs` / `image_specialize.rs`. Next: chart-fixture re-ingest Acc.
- **SPEC-047 / MV-24 Acc gate (2026-07-11):** Full chart-fixture re-vision
  (`17d517a4-…`). Crops fired 8/8 docs. Acc **0.433** (was 0.423), page_hit@5
  **0.80** (was 0.72), Chart Acc **0.182** flat, Chart a_in_e **0.409** flat —
  G-A still FAIL. Artifacts: `smoke-post-mv24-chart-crops`. Next: MV-26/27/28.
- **SPEC-047 / MV-24 chart crops (2026-07-11):** Pass A–gated hi-res page re-render
  (200dpi / 3600px) + deterministic ink-bbox crop → `assets/page-NNNN-chart.png`
  drawing override; specialize-time `maybe_chart_specialize_bytes` fail-open.
  SSOT: `edgequake-pdf/chart_crop.rs`; DRY `text_suggests_chart` shared with
  multimodal routing. E2E: `e2e_spec047_chart_crop`. Next: re-ingest + G-A.
- **SPEC-047 / 023 next-from-MV18 FP (2026-07-11):** Deep assessment after Chart
  a_in_e 0.41 — root causes (whole-page PNG, no crop, fail-open, gold shape);
  lawful queue MV-24→29; rejects Acc heuristics. Canvas: `spec047-next-from-mv18`.
- **SPEC-047 / 015 MV-18/19 full chart smoke (2026-07-11):** Soft-resume 8-doc
  re-ingest + dscope query. Acc **0.423**, Chart Acc **0.182**, Chart
  answer_in_evidence **0.409** (n=22; was ~0.32). G-A (≥0.50) still open.
  Artifacts: `smoke-post-mv18-full-chart`. Workspace `be4c40a9-…`.
- **SPEC-047 / 015 Chart Rep harden (2026-07-11):** First-principles number-landing —
  Pass A RAG page prompt (`edgequake-pdf/vision_prompts.rs` → pdf2md `system_prompt`);
  Pass B denser chart schema (`data_table_md`) + figure `visible_text`; caption/context
  chart routing (`should_specialize_as_chart`). E2E: `e2e_spec047_chart_number_landing`.
  Live 1-doc probe (`smoke-post-mv18-chart-prompts`): mm 17/17, Chart/KeyValues/DataTable
  10/10/9, Chart answer_in_evidence **0.50** (n=6). **Full 8-doc Acc gate still open.**
  Tickets MV-18/19.
- **SPEC-047 / 022 re-assessment (2026-07-11):** Authoritative Acc chain
  (0.384→0.436→0.429→0.427), ticket board, next queue (015 Chart ‖ B3 Mix ‖
  L-B2). Updates `000-index`, `e2e/README`, `013`/`015`/`020`/`021` status.
  Canvas: `spec047-reassessment-20260711`.
- **SPEC-047 / 021 lineage (2026-07-11):** First-principles plan for
  Entity→Chunk→Document→Page. Multi-doc entities must keep doc/chunk **unions**;
  page only via chunks; query scope fail-closed (L-A2→L-A1→L-A3). Canvas:
  `spec047-lineage-first-principles`.
- **SPEC-047 / 021 L-A1/A2/A3 code (2026-07-11):** DRY `lineage_scope` + ingest
  `source_document_ids[]` merge; fail-closed `context_filter`; scoped
  `kg_chunk_pick`. E2E: `e2e_spec047_lineage_scope`; contract:
  `contract_spec047_lineage_docs`.
- **SPEC-047 / 021 query-only smoke (2026-07-11):** No re-ingest (derive from
  chunk ids). Acc **0.429→0.427** (≈noise), Unans **0.81→0.83**, page_hit@5
  **0.75→0.76**, Pure-text **0.255→0.192**. Artifacts: `smoke-post-lineage-la2`.
- **SPEC-047 / 021 L-A4 (2026-07-11):** Doc-diverse KEEP — `truncate_keep_doc_diverse`
  round-robins across `{doc}-chunk-*` so minority parents survive SOURCE_IDS
  caps. E2E: `e2e_spec047_lineage_diverse_keep`.
- **SPEC-047 / 020 A3 Acc recovery (2026-07-11):** Fail-open
  `prune_empty_arm_graph` + `truncation_config_for_intent` (Factual chunk floor
  0.55 / entity cap 2k). Smoke: Acc 0.393→**0.429**, Pure-text 0.192→**0.255**,
  mean `n_sources` 108→35. Artifacts: `smoke-post-a3-acc-recovery`. B3 Mix now lawful.
- **SPEC-047 / 020 B2 smoke (2026-07-11):** Hybrid `intent_arm_mask_hybrid`
  (Factual→local+naive; Mix stays cost-gated). Query-only smoke:
  `planned_naive_only` 0.85→**0.00**, false_refusal 0.33→0.28, Acc 0.436→**0.393**
  (block B3 until Acc recovers). Artifacts: `smoke-post-b2-arm-gate`. Diagnostics
  now split planned vs productive arm rates.
- **SPEC-047 / 020 A1–A2–B1 code (2026-07-11):** Entailment-first calibrated
  grounding (`grounding.rs`); false-refusal + arm-gate metrics in
  `diagnostics.py` → SUMMARY; e2e `e2e_a1_*` + pytest diagnostics.
- **SPEC-047 / 020 (2026-07-11):** Post-Q1 first-principles plan — calibrated
  selective refusal (A1), false-refusal metric (A2), arm-gate honesty (B1–B2),
  Mix ablation (B3), parallel 015 Chart hand-off. Evidence: Acc 0.384→0.436,
  Unanswerable 0.69→0.81, Pure-text 0.27→0.19. Canvas: `spec047-post-q1-first-principles`.
- **SPEC-047 / 019 Q1–Q2 code (2026-07-11):** Query grounding — `context_format` +
  `grounding` modules; page/modality in LLM context; 40% chunk budget floor;
  `P1_mix_rrf` bench profile; OpenAPI mode semantics. E2E:
  `e2e_spec047_query_grounding`.
- **SPEC-047 / 019 (2026-07-11):** Query first-principles improvement plan —
  failure taxonomy R/G/Gen/Rep, decision tree, phases Q0–Q6 (ground before fuse).
  Canvas: `spec047-query-first-principles`.
- **SPEC-047 / 017–018 (2026-07-11):** Code-is-law LightRAG↔EdgeQuake query+ingest
  assessment (`017-…`) and quality/speed improvement plan (`018-…`: phases A–E,
  gates Chart Acc / page_hit / soft-resume). Index + reading order updated.
  Canvas: `spec047-query-pipeline-eq-vs-lightrag`.
- **SPEC-048 improve UX (2026-07-11):** Transparent ingestion progress — 5 WHYs,
  code-is-law inventory, six lenses (PO/UX/UI/FE/BE/FS), ASCII screens/components/
  state machines, normative FE↔BE contract, AC matrix, P0–P3 roadmap under
  `specs/048-improve-ux/` (000–014).
- **SPEC-048 implementation (2026-07-11):** ProgressFacade + `GET /ingestion/{id}/progress`
  + `GET /pipeline/activity` + reprocess stage reset + WS Chunk/Graph bridge;
  FE `IngestionRunView` + ActiveRunsPanel/ServerStageStepper; contract + Playwright
  screenshots in `specs/048-improve-ux/e2e/screenshots/`. Plan: `015-implementation-plan.md`.
- **SPEC-047 G-A live (2026-07-10):** `P0_mm_ite` chart-subset re-ingest — **G-A FAIL**
  (Chart fidelity 0.32 vs 0.36 baseline; zero `<drawing>` refs → `ite` no-op). Artifacts in
  `e2e/artifacts/smoke/`; baseline preserved in `smoke_p0_baseline/`.
- **SPEC-047 / 015 Phase A+B (code):** `P0_mm_ite` + chart/figure specialize; `[Chart Name]`/`[Figure Name]`;
  doctor `VLM_PROCESS_ENABLE` gate; `wait_pdf` waits for document pipeline completion.
- **SPEC-047 / 013** code-is-law improvement roadmap + **W0** harness diagnostics:
  `tools/bench047/bench047/diagnostics.py` computes `page_hit@k` / `page_recall@k` from
  API `sources[].page_start` vs gold `evidence_pages` (offline only). Wired into
  `run.py` predictions + scorecard `ops.retrieval`. CLI `--document-scope` for W2
  document-filtered retrieve (not gold pages).
- **SPEC-047 / 015** modality-aware vision improvement plan (typed Chart/Figure
  prompts, phased gates on fidelity, tickets EQ-047-MV-*). Canvas:
  `spec047-modality-vision-plan`.
- **SPEC-047 / 014** ingest+query first-principles pipeline study (code anchors +
  ranked improvements) linked from index; canvas `spec047-pipeline-first-principles`.
- **SPEC-047 W1a probe**: `bench047 fidelity` audits whether gold answers appear in
  ingested evidence-page markdown (`fidelity.py` / `FIDELITY.md`) to separate
  representation miss (W1) from retrieval miss (W2).
- **SPEC-047 W0b**: HTTP `QueryStats` now projects engine `context_empty`, `arms_run`,
  `arm_*_ms`, `arm_*_chunks` via DRY `query_stats_mapper::from_engine_stats` (used by
  `/query` + chat). Hybrid/Mix record pre-merge arm chunk counts in context metadata.
- **SPEC-047** RAG evaluation pack + harness (`specs/047-rag-evaluation/`, `tools/bench047/`):
  MMLongBench-Doc RAG adaptation for EdgeQuake with Mistral Small (LLM+vision) +
  `mistral-embed`, hybrid query, smoke→core→full progression, official Acc/F1 scoring.
  Make targets: `bench047-smoke`, `bench047-doctor`, `bench047-freeze-smoke`.
- CHANGELOG.md for specs directory.
- `specs/021-storage-study/06-first-principles/19-ingestion-query-improvement-plan.md` §11:
  multi-perspective assessment (GraphRAG / LightRAG / AI Engineer / System Engineer)
  of the implemented P-G1, P-G3, P-G6, P-G2b changes, with a verification matrix.
- P-G7: index-friendly KV scans. `keys()` + in-memory filter replaced by
  `keys_with_prefix` / `keys_with_suffix` in `reprocess.rs`, `pdf_processing.rs`,
  `stuck.rs`, `storage_helpers.rs`, and `delete/single.rs` (incl. a rewritten
  `resolve_kv_key_prefix`).
- P-G9: query embedding cache. New
  `edgequake-query/src/cache/embedding_cache.rs` (`CachingEmbeddingProvider`,
  LRU 10k / 1h TTL, model identity folded into the key) wired into the
  production query engine via `QueryEngine::with_embedding_cache`. Contract
  tests in `edgequake-query/tests/contract_embedding_cache.rs`.
- P-G11: streaming vision parity. `stream_answer_from_context` now delegates
  image-attached requests to `stream_vision_answer` (vision `chat` path with
  E30 text fallback). Contract tests in
  `edgequake-query/tests/contract_streaming_vision.rs`.
- P-G1b: legacy entity reconciliation. New
  `edgequake-storage/src/entity_reconcile.rs` (dry-run `plan` + confirm-token
  `execute`, idempotent) and admin endpoints
  `GET/POST /api/v1/admin/entities/reconcile` in `admin.rs` + `routes.rs`.
  5 unit tests covering E5/E6/E7, edge rewrite, vector re-key, idempotency.

### Changed

- **SPEC-047:** First-principles improvement roadmap (`013-…`) — anti-heuristic workstreams
  W0–W4 from smoke failure physics (false refusal, chart/figure misses).
- **SPEC-047:** First valid smoke baseline documented (Acc≈0.45 / F1≈0.29); score-reading
  guidance in `000-index`, `012`, `e2e/README`, and generated `SUMMARY.md`. Hybrid/Mix
  stack-overflow fix noted (`Box::pin` arms + 8 MiB tokio worker stack).
- Marked P-G1 (EntityId newtype) and P-G3 (Global N+1 fix) as ✅ DONE/TESTED in
  plan-19, with code-level evidence and acceptance-test verification notes.
- Marked P-G7, P-G9, P-G11, P-G1b as ✅ DONE/TESTED in plan-19.
- Marked P-G2 as ◑ PARTIAL: P-G2b + shared EntityId/batch/compensation
  invariants are done across both remaining persistence paths; the literal
  `IngestionPersister` trait extraction is deferred (structural DRY only — no
  correctness delta) and documented in plan-19.

### Fixed

- `edgequake-api/src/handlers/documents/recovery/reprocess.rs`: gated the new
  empty-markdown fallback's `state.storage.pdf_storage` access behind
  `#[cfg(feature = "postgres")]` to match the pre-existing pattern at line 444.
  Without this, `cargo build -p edgequake-api --lib --no-default-features` failed
  (feature-gating regression caught during pre-commit verification).
- `edgequake-api/src/error.rs`: added the `StorageError::InvalidInput` arm to
  `storage_error_category` (new variant introduced by P-G1b's confirm-token
  error path).
