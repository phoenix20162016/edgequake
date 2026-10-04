# 12 — Edge cases

Parent: [README](README.md) · Prev: [11](11-ml-quality-spec.md) · Next: [13 Plan](13-implementation-plan.md)

Each row has a scenario, the law it protects, a mitigation, one test, and the
wave that ships it. Test IDs are defined in [14](14-e2e-test-matrix.md).
Prefix: U unit, I integration, V vitest, E Playwright, L live.

## A. Mode words and precedence

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-01 | Document field holds an unknown word (`tev1`) | 4 | Resolver returns an error that names the layer. Upload returns 422. | U05, I02 | W1, W4 |
| EC-160-02 | Workspace key or env var holds an unknown word | 4 | Same error. Env is also checked at startup. Workspace write returns 400. | U05, I04 | W1, W4 |
| EC-160-03 | Word has different case or spaces (` Decision `) | 2 | Parser trims and lowercases. | U04 | W1 |
| EC-160-04 | Word is `inherit`, `none`, or empty | 2 | Treated as no override. | U03 | W1 |
| EC-160-05 | Workspace write has a bad word | 4 | Metadata stays unchanged. | U08, I04 | W1, W4 |
| EC-160-06 | No field anywhere | 6 | Mode is `llm`, source `default`. No new code path runs. | U01, I05 | W1, W4 |
| EC-160-07 | Workspace default changes after a document was admitted | 9 | Admission stores the resolved word on the task and document. Later reads use the stored word. | I03 | W4 |
| EC-160-08 | Env says `decision` and `EDGEQUAKE_DECISION_ENABLED` is off | 4 | Upload returns 422 `decision_disabled`. Startup logs a warning. | I10 | W4 |
| EC-160-09 | Multipart field is present and empty | 2 | Same as inherit. | I03 | W4 |
| EC-160-10 | Multipart field appears twice | 4 | 422 `invalid_extraction_mode` with message "duplicate field". | I29 | W4 |
| EC-160-11 | JSON text upload sends `extraction_mode: null` | 2 | Same as absent. | I05 | W4 |

## B. Upload paths and parity

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-12 | PDF upload uses a separate handler that skips `MultipartUploadFields` | 2 | The PDF handler calls the same validator and writes the same task keys. | I13 | W4 |
| EC-160-13 | Batch has a bad word | 4 | The whole batch fails before any file is stored. | I14 | W4 |
| EC-160-14 | In-memory and Postgres workspace services differ | 2 | Both call `apply_extraction_mode_metadata`. | U08, I04 | W4 |
| EC-160-15 | Reanalyze changes the mode | 9 | The fingerprint changes. Stale data is purged first. | U13, I09 | W1, W4 |

## C. Backend and runtime

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-16 | Backend was up at upload and is down at worker start | 4 | Worker health check. Task fails with a retryable error. No mode switch. | I15 | W5 |
| EC-160-17 | Backend dies in the middle of a document | 4 | Task fails. Completed chunks stay cached, so a retry is cheap. No LLM call occurs. | I15, I17 | W5 |
| EC-160-18 | Fairness clamp reads the chat LLM name (maybe a cloud provider) | 12 | `decision:` prefix counts as local. Resolver returns it when mode is `decision`. | U32, I24 | W5 |
| EC-160-19 | Cold start. First request loads the model and times out. | 12 | `keep_alive` default 30m. Timeout 600 s. Health check warms the model. | L01 | W2 |
| EC-160-34 | Ollama is older than 0.35 and has no `/v1/systemone` | 4 | Health check detects 404. Status shows "unsupported". Upload returns 422. | I16 | W2 |
| EC-160-57 | Ollama update drops `capabilities:["decision"]` from a derived model | 4 | Status shows "not decision-capable". Doc gives a re-create command. | I23 | W5 |
| EC-160-58 | One request body exceeds 64 KiB | 8 | Pack splitter cuts the pack. A single over-size question is skipped and counted. | U27 | W2 |
| EC-160-59 | Request times out | 8 | Retry with backoff (3 tries). Then task failure. | I15 | W2 |
| EC-160-60 | Task is cancelled mid-run | 8 | The loop checks the cancel token between packs. | I26 | W5 |

## D. Questions, ontology, and limits

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-20 | Sentence exceeds the context budget | 8 | Trim the state to the token budget. Count `truncated`. | U22 | W2 |
| EC-160-21 | Table or list holds hundreds of names | 8 | Per-sentence pair cap 24. SPEC-117 entity and record caps apply. | U21, I25 | W2 |
| EC-160-24 | Every mention is `NOT_ENTITY` | 5 | Empty result. Not a failure. | U31 | W2 |
| EC-160-25 | Workspace has more than 25 entity types | 8 | Split into two rounds by group, or return a clear validation error. Choose one in W0. | U18 | W2 |
| EC-160-26 | `relation_edges` is empty | 8 | No pruning. Count `unpruned`. The pair cap still applies. | U19 | W2 |
| EC-160-31 | Chunk is empty, whitespace, a code block, or a table row | 8 | Empty chunk skips the backend. Code blocks follow the existing chunk rules. | U28 | W2 |
| EC-160-55 | Relation has the same entity at both ends | 5 | Dropped (BR0006). | U20 | W2 |
| EC-160-53 | `enable_gleaning` is on | 6 | Gleaning applies to LLM mode only. Decision mode ignores it. The docs say so. | I11 | W5 |

## E. Model output and safety

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-32 | Backend returns a letter that is not listed, or malformed JSON | 3 | Reject the answer. Count `invalid_answer`. The pair goes to REJECT. | U24, U25 | W2 |
| EC-160-33 | Chunk text tries to steer the answer (injection) | 3 | Closed options only. No free text enters the graph. Gate and review limit effects. | U24, L01 | W2 |
| EC-160-30 | Text is not English | 5 | Admission warns when `extraction_language` is not English. The run proceeds if the user chose it. | U29 | W4 |
| EC-160-23 | Scores from two backends or models are compared | 5 | Store backend and model with each score. No cross-model compare. | U26 | W2 |
| EC-160-35 | Status or logs leak a credential or full URL | 14 | Show host only. Redact userinfo and API key. | U34, I23 | W5 |

## F. Fingerprint, cache, and storage

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-22 | A changed pack size reuses old answers | 10 | Pack size is in the cache key. | U26 | W2 |
| EC-160-27 | `llm` digest changes by accident | 9 | Pinned legacy hash test. | U12 | W1 |
| EC-160-28 | The user switches model and expects a purge | 9 | Model is in the cache key. Reanalyze offers "force purge". Decision in OQ-160-1. | I17 | W4 |
| EC-160-29 | Task retries after a crash | 10 | Cache hit makes the run idempotent. | I17 | W3 |
| EC-160-36 | Document is deleted | 11 | Review rows are deleted with it. | I18 | W3 |
| EC-160-37 | Workspace is deleted | 11 | Cache and review rows are deleted. | I19 | W3 |
| EC-160-38 | Two workspaces hold the same text | 11 | Cache is per workspace. No cross-read. | I20 | W3 |
| EC-160-39 | A store query forgets the workspace filter | 11 | Every store method takes `workspace_id`. Test asserts isolation. | I20 | W3 |
| EC-160-40 | Migration 166 skips manifest or checksum entries | 6 | Add manifest entry. Update `checksums.lock`. CI test. | I21 | W3 |
| EC-160-41 | Old binary meets schema 166 during rolling deploy | 6 | Migration is additive. `compat_serve_max` rises to 166. Old binaries ignore new tables. | I21 | W3 |
| EC-160-42 | Cache grows without bound | 8 | TTL and max rows. Background sweep. | I17 | W3 |
| EC-160-43 | Review table grows without bound | 8 | Delete on reanalyze and on document delete. Size shows in stats. | I18 | W3 |
| EC-160-48 | Workspace write drops other metadata keys | 6 | Helper touches one key only. | I22 | W4 |
| EC-160-47 | Two admins save the workspace at once | 2 | Last write wins per key. Each save sends only changed fields. | V07 | W7 |

## G. UI

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-44 | Web and Rust word lists drift | 1 | Shared JSON fixture and contract test. | V04, U07 | W7 |
| EC-160-45 | `decision` is selectable while the backend is off | 4 | Option is disabled with a tooltip. | E10, V06 | W7 |
| EC-160-46 | A locale lacks a key | 14 | Locale parity script. | V03 | W7 |
| EC-160-49 | A very large document (thousands of chunks) | 8 | Progress by chunk. Cancel works. A time estimate shows before upload. | I27 | W5 |
| EC-160-50 | Upload select state leaks across uploads | 6 | State lives in the manager. It resets to `default` after each upload action. | V05 | W7 |
| EC-160-51 | Cost view shows a token cost for decision runs | 14 | Cost is 0 with a `decision` label. | I12 | W5 |

## H. Governance

| ID | Scenario | Law | Mitigation | Test | Wave |
|----|----------|-----|------------|------|------|
| EC-160-52 | Merger dedupes names differently in the two modes | 7 | Use the entity-id SSOT normalizer. | U23 | W2 |
| EC-160-54 | Weights license forbids use | 13 | No bundled weights. Attribution and notice in the UI. SP-160-5 closes before release. | E07 | W7 |
| EC-160-56 | A user expects LLM-grade relation quality | 5 | Help text states the limit. W8 report states the gap. | E07 | W7, W8 |

## Coverage check

Every EC has one test ID. [15](15-cross-ref.md) lists the inverse map. A CI
script (`scripts/spec160-coverage.sh`, W8) fails when an EC has no test ID.

Next: [13 Plan](13-implementation-plan.md).
