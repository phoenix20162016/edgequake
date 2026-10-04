# 05 — Product spec

Parent: [README](README.md) · Prev: [04](04-code-as-is.md) · Next: [06 Architecture](06-architecture.md)

## Personas

| ID | Persona | Need |
|----|---------|------|
| P1 | Self-hoster | Extract on a CPU server with no cloud key |
| P2 | Privacy owner | Text stays on the host |
| P3 | Cost owner | No token spend for extraction |
| P4 | Operator | Predictable, repeatable runs and a way to review doubtful facts |
| P5 | Quality owner | Know what quality the mode gives before choosing it |

## Features

| ID | Feature | Wave |
|----|---------|------|
| F-160-01 | Workspace default extraction mode | W4, W7 |
| F-160-02 | Per-upload mode override (file, text, PDF, batch) | W4, W7 |
| F-160-03 | `DecisionExtractor` behind `EntityExtractor` | W2 |
| F-160-04 | Ollama System One backend | W2 |
| F-160-05 | OpenAI-compatible logprobs backend (`llama-server`) | W6 |
| F-160-06 | Gate presets and review queue | W3, W5 |
| F-160-07 | Decision cache | W3 |
| F-160-08 | Status endpoint and health | W5 |
| F-160-09 | Ontology from workspace entity and relation types | W2 |
| F-160-10 | Provenance and per-document stats | W2, W3 |
| F-160-11 | Workspace settings card | W7 |
| F-160-12 | Upload mode selector | W7 |
| F-160-13 | Document badge and review count | W7 |
| F-160-14 | Fingerprint and reanalyze | W1 (done), W4 |
| F-160-15 | Environment variables | W4 |

## Stories

Format: **As** persona, **I want**, **so that**. Each story has an acceptance
check (AC) that maps to a test in [14](14-e2e-test-matrix.md).

| ID | Story | AC | Tests |
|----|-------|----|-------|
| US-160-1 | As P1, I want to set `decision` as my workspace default so that all uploads use it. | A new upload with no override shows mode `decision` and source `workspace`. | T-160-I03, T-160-E01 |
| US-160-2 | As P2, I want to upload one file in `decision` mode so that this file never reaches a cloud LLM. | The task metadata holds `decision`. No LLM extraction call occurs. | T-160-I01, T-160-E02 |
| US-160-3 | As P1, I want a clear error when the backend is down so that I do not lose the file silently. | Upload returns 422 with code `decision_backend_unavailable`. | T-160-I07, T-160-E04 |
| US-160-4 | As P4, I want to see how many facts the gate sent to review so that I can judge the run. | The document shows ACCEPT, REVIEW, REJECT counts. | T-160-I11, T-160-E06 |
| US-160-5 | As P4, I want to re-run a document in another mode so that I can compare. | Reanalyze with a changed mode purges derived data first. | T-160-I09 |
| US-160-6 | As P3, I want zero token cost reported so that the cost view stays honest. | Cost for a `decision` document is 0. | T-160-I12 |
| US-160-7 | As P5, I want the UI to say what the mode is good for so that I choose well. | Help text states the limits. | T-160-E07 |
| US-160-8 | As P1, I want to keep `llm` as the default so that nothing changes on upgrade. | Existing tests pass. The `llm` digest equals the old digest. | T-160-U12 |

## Scope

In scope:

1. Mode selection at workspace and upload level.
2. Decision extractor, two backends, cache, review table.
3. Status endpoint, UI controls, i18n in en, fr, zh.
4. Tests and a quality gate.

Out of scope:

1. Fine-tuning or training any model.
2. Bundling or downloading weights.
3. A review UI to accept or edit REVIEW rows (a later spec). W3 stores them.
   W7 shows counts only.
4. Query-time use of Tev1.
5. Non-English extraction.
6. Cross-sentence or coreference links beyond what edgextract does.

## Release criteria

| Criterion | Gate |
|-----------|------|
| The default behavior is unchanged | All existing tests pass. T-160-U12 passes. |
| The mode works end to end against a fake backend | T-160-I01 to I14 pass |
| The mode works against a live Ollama | T-160-L01 passes (opt-in) |
| The quality gap is measured | W8 report on SPEC-001 data |
| The UI states limits | T-160-E07 passes |
| The license risk is recorded | SP-160-5 is closed or the docs show the warning |

## Opportunity register

| ID | Idea | Status |
|----|------|--------|
| OP-160-1 | Review UI to accept REVIEW rows | Later spec |
| OP-160-2 | Tiered model profile (type 0.8B, relate 4B) | After W8 |
| OP-160-3 | Calibrate the gate per workspace from reviewed rows | After OP-160-1 |
| OP-160-4 | Hybrid: decision for typing, LLM for descriptions | After W8 |
| OP-160-5 | Other decision models (Nimble) behind the same trait | Open |

## Licence and attribution

Together AI states the Tev1 weights license is "being finalized". EdgeQuake
bundles no weights and downloads none. The settings card names the model and
links to the model card. Operators accept the license when they pull the model.
Closing SP-160-5 is a release item.
