# 01 — First principles

Parent: [README](README.md) · Prev: [00 WHY](00-why.md) · Next: [02 Source study](02-source-study.md)

A first principle is a fact that does not depend on another rule. Each law
below follows from one of three facts.

| Fact | Statement |
|------|-----------|
| FACT-1 | A graph edge is a claim. A wrong claim costs more than a missing claim. |
| FACT-2 | A closed question has a finite answer set. An open question has none. |
| FACT-3 | A user who picks a private mode expects that text stays private. |

## Laws

| ID | Law | Derives from |
|----|-----|--------------|
| LAW-160-1 | The mode name is vendor-neutral. A model is a preset. | Models change faster than API words. |
| LAW-160-2 | One chain resolves the mode: document > workspace > env > `llm`. | SPEC-117 precedent. One rule, one place. |
| LAW-160-3 | The model chooses from listed options. It never writes free text into the graph. | FACT-2 |
| LAW-160-4 | A bad value or a down backend is an error. The system never falls back to another mode. | FACT-3 |
| LAW-160-5 | Only ACCEPT rows enter the graph. REVIEW rows wait for a human. | FACT-1 |
| LAW-160-6 | The default stays `llm`. Existing behavior stays byte-identical. | Backward compatibility |
| LAW-160-7 | The decision extractor returns `ExtractionResult`. Downstream code does not change. | Liskov, DRY |
| LAW-160-8 | Work is bounded before the call: questions, pairs, options, tokens. | FACT-2 |
| LAW-160-9 | A mode change makes derived data stale. The `llm` fingerprint does not change. | SPEC-046 |
| LAW-160-10 | Same input, same model, same contract gives the same decision. The cache stores it. | FACT-2 |
| LAW-160-11 | Each decision records model, contract, band, and score. | Auditability |
| LAW-160-12 | CPU inference is a first-class path. GPU is optional. | WHY-160-4 |
| LAW-160-13 | EdgeQuake bundles no weights. The user opts in. | License unknown |
| LAW-160-14 | Every run exposes mode, backend, and band counts. | Operability |

## DRY rules

1. One function resolves the mode: `resolve_extraction_mode`
   (`edgequake-pipeline/src/extraction_mode.rs`). Upload, prepare, reanalyze,
   and workspace writes call it. No second parser exists.
2. One function checks a mode word: `parse_mode_override`. Admission and
   workspace writes share it and share its error text.
3. One function writes the workspace key: `apply_extraction_mode_metadata`.
   The Postgres and in-memory workspace services both call it.
4. One digest builder exists: `ProcessFingerprintInput::digest`. The mode joins
   it. No new fingerprint exists.
5. One web helper builds the upload field: `buildExtractionModeField`. The file,
   text, PDF, and batch paths call it ([10](10-frontend-architecture.md)).
6. One cache key function builds decision keys. The backend adapters share it.

## SOLID rules

| Letter | Rule | Where |
|--------|------|-------|
| S | One module, one reason to change. Mode parsing, question building, transport, gating, and storage live in separate files. | [06](06-architecture.md) |
| O | A new backend implements `DecisionBackend`. No existing file changes. | [06](06-architecture.md) |
| L | `DecisionExtractor` replaces `SOTAExtractor` behind `EntityExtractor`. Callers cannot tell. | LAW-160-7 |
| I | `DecisionBackend` has one method set: `ask` and `health`. The extractor does not see HTTP. | [06](06-architecture.md) |
| D | The extractor depends on the `DecisionBackend` and `DecisionStore` traits. Concrete Ollama and Postgres types attach at the factory. | [06](06-architecture.md) |

## Size rule

Keep each Rust file under 300 lines where possible. Split by reason to change.
The target module list is in [13](13-implementation-plan.md).

## How the laws map to edge cases

Each EC in [12](12-edge-cases.md) names the law it protects. Each law has at
least one test in [14](14-e2e-test-matrix.md).
