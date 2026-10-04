# SPEC-160 — Decision Extraction Mode (Tev1 and CPU inference)

> **Status:** Implemented as a preview. W1–W5 and W7 are done and wired end to end (Ollama backend, storage, API, web UI). W6 (`llama-server` backend) is deferred. W8 quality report: [measurements/w8-report.md](measurements/w8-report.md). Operator guide: [docs/concepts/decision-extraction.md](../../docs/concepts/decision-extraction.md).  
> **Product pin:** EdgeQuake v0.30.0+  
> **Scope:** A second knowledge-graph (KG) extraction mode, `decision`. A
> small decision model (Tev1) answers closed questions. The model never writes
> free text into the graph. The mode is selectable per upload and as a
> workspace default. It runs on CPU on the server.  
> **Inherits:** [SPEC-117](../117-extraction-budget/) · [SPEC-114](../114-config-entity-type/) ·
> [SPEC-096](../096-multi-language-extraction/) · [SPEC-046](../046-graphrag-study/) ·
> [SPEC-150](../150-reliable-migration-system/) · [SPEC-001](../001-benchmark/)  
> **Peers:** [SPEC-159](../159-query-action/) (pack shape)  
> **Source study:** [edgextract](https://github.com/raphaelmansuy/edgextract) (Apache-2.0)

## Start here

1. [00-why.md](00-why.md) — Five WHYs and the causal chain
2. [01-first-principles.md](01-first-principles.md) — LAW-160-1…14 and DRY/SOLID rules
3. [02-source-study.md](02-source-study.md) — edgextract, Tev1, System One contract
4. [03-cpu-inference-study.md](03-cpu-inference-study.md) — CPU options A–F, measurements, capacity
5. [04-code-as-is.md](04-code-as-is.md) — Code anchors (file and line)
6. [05-product-spec.md](05-product-spec.md) — Personas, stories, scope
7. [06-architecture.md](06-architecture.md) — Components, data flow, traits
8. [07-data-model.md](07-data-model.md) — Metadata keys, migration 166, fingerprint
9. [08-api-contract.md](08-api-contract.md) — Fields, DTOs, status endpoint, env vars
10. [09-ux-ui-spec.md](09-ux-ui-spec.md) — Controls, labels, states, i18n
11. [10-frontend-architecture.md](10-frontend-architecture.md) — Modules and call sites
12. [11-ml-quality-spec.md](11-ml-quality-spec.md) — Gates, bands, evaluation protocol
13. [12-edge-cases.md](12-edge-cases.md) — EC-160 register with mitigation and test
14. [13-implementation-plan.md](13-implementation-plan.md) — Waves W0–W8 and DoD
15. [14-e2e-test-matrix.md](14-e2e-test-matrix.md) — One gate per EC
16. [15-cross-ref.md](15-cross-ref.md) — Law ↔ WHY ↔ F ↔ EC ↔ wave ↔ test ↔ lens
17. [measurements/README.md](measurements/README.md) — Raw CPU and GPU benchmark data
18. Lenses → [`lenses/`](lenses/)
    - [Product Owner](lenses/LENS-product-owner.md)
    - [Full Stack](lenses/LENS-full-stack.md)
    - [Database](lenses/LENS-database.md)
    - [UX / UI](lenses/LENS-ux-ui.md)
    - [Front](lenses/LENS-front.md)
    - [AI Engineer](lenses/LENS-ai-engineer.md)
    - [ML Engineer](lenses/LENS-ml-engineer.md)

## One-screen picture

```text
  upload / workspace                      server (CPU first)
 ┌──────────────────────┐   mode word   ┌────────────────────────────────────┐
 │ extraction_mode      │──────────────►│ resolve_extraction_mode()          │
 │  inherit|llm|decision│  doc>ws>env   │  document > workspace > env > llm  │
 └──────────────────────┘               └──────────────┬─────────────────────┘
                                                       │
                              ┌────────────────────────┴───────────────┐
                              │ llm                       decision     │
                              ▼                                  ▼     │
                     SOTAExtractor (unchanged)         DecisionExtractor (new)
                              │                                  │
                              │                     propose ► type ► prune ► relate
                              │                                  │  (closed questions)
                              │                         DecisionBackend (trait)
                              │                          ├─ Ollama System One
                              │                          └─ OpenAI-compat logprobs
                              │                                  │
                              │                      gate: ACCEPT │ REVIEW │ REJECT
                              ▼                                  ▼
                       ExtractionResult  ◄───────────── ACCEPT rows only
                              │                  (REVIEW rows go to decision_review)
                              ▼
                  merger → embeddings → graph (unchanged)
```

## Locked decisions (Wave 0)

1. **Vendor-neutral mode.** The mode word is `decision`. Tev1 is a model
   preset, not a mode name (LAW-160-1).
2. **One precedence chain.** Document > workspace > environment > `llm`. The
   chain is one pure function (LAW-160-2). W1 implements it.
3. **Default stays `llm`.** Every existing document, workspace, and test keeps
   today's behavior (LAW-160-6).
4. **No silent fallback.** A bad mode word returns an error. A down backend
   fails the task. The system never routes `decision` text to a cloud LLM
   (LAW-160-4).
5. **Same output contract.** The decision extractor returns `ExtractionResult`.
   Merger, embeddings, and storage do not change (LAW-160-7).
6. **Only ACCEPT rows enter the graph.** REVIEW rows go to a review table.
   REJECT rows are dropped and counted (LAW-160-5).
7. **CPU first.** The default backend is Ollama System One with a CPU-pinned
   derived model. A second backend serves `llama-server` over OpenAI-compatible
   logprobs (LAW-160-12).
8. **Pack size 4 by default.** The measured time and quality knee on CPU is
   4 questions per request ([03](03-cpu-inference-study.md)).
9. **Opt-in weights.** EdgeQuake does not bundle weights. The Tev1 license is
   open until Together AI publishes it (LAW-160-13).
10. **Honest claim.** The mode is for private, offline, cheap, and
    repeatable extraction. It does not claim parity with a frontier LLM. Wave
    W8 measures the gap on SPEC-001 before any claim ([11](11-ml-quality-spec.md)).

## Status by wave

| Wave | Title | State |
|------|-------|-------|
| W0 | Spikes and gates | SP-160-1 closed (published `edgextract` 0.1.0, `default-features = false`). Others open. |
| W1 | Mode SSOT, options field, fingerprint | **Done** |
| W2 | `DecisionExtractor` and Ollama backend | **Done** |
| W3 | Storage: migration 166, cache, review | **Done** |
| W4 | API, admission, workspace plumbing | **Done** |
| W5 | Factory wiring and status endpoint | **Done** |
| W6 | CPU server backend (`llama-server`) | Deferred (Ollama is the only backend) |
| W7 | Web UI | **Done** (Playwright against the real backend, screenshots in [e2e/screenshots](e2e/screenshots/)) |
| W8 | Quality gate, live test, docs | Measured: [w8-report](measurements/w8-report.md). Presets stay "uncalibrated" in the UI. |

Detail: [13-implementation-plan.md](13-implementation-plan.md).
