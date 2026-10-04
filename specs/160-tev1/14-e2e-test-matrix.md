# 14 — E2E test matrix

Parent: [README](README.md) · Prev: [13](13-implementation-plan.md) · Next: [15 Cross-ref](15-cross-ref.md)

Test IDs in this file are short. The full ID adds the prefix `T-160-`. Example:
`U12` is `T-160-U12`.

## Layers

```text
  U  Rust unit        pure files, no I/O                      cargo test -p <crate> --lib
  I  Rust integration fake System One server, real handlers   cargo test -p edgequake-api --test spec160_*
  V  vitest           pure helpers and components             cd edgequake_webui && bun test
  E  Playwright       UI flows with route mocks, @spec160     pnpm exec playwright test e2e/spec160
  L  Live (opt-in)    real Ollama and tev1                    EQ_LIVE_TEV1=1 ...
```

The fake server (`tests/support/fake_system_one.rs`) accepts `POST /v1/systemone`.
It returns scripted answers, counts requests, and can return 404, 500, a slow
reply, a bad letter, or a malformed body. Tests also count calls to a mock LLM
to prove no fallback occurs.

## U — Rust unit

| ID | Check | EC | Status |
|----|-------|----|--------|
| U01 | Default mode is `llm` | 06 | Done |
| U02 | Precedence document > workspace > env | 06 | Done |
| U03 | Inherit words pass through | 04 | Done |
| U04 | Parser tolerates case and spaces | 03 | Done |
| U05 | Unknown word errors with layer name | 01, 02 | Done |
| U06 | A losing layer is still checked | 02 | Done |
| U07 | Wire values are stable | 44 | Done |
| U08 | Workspace metadata set, clear, reject | 05, 14 | Done |
| U09 | Document override from task JSON | 07 | Done |
| U10 | `parse_mode_override` inherit mapping | 04 | Done |
| U11 | Options default to `llm`, builder sets mode | 06 | Done |
| U12 | `llm` digest equals pinned legacy hash | 27 | Done |
| U13 | Mode change makes document stale both ways | 15 | Done |
| U14 | Metadata read of mode for fingerprint | 15 | Done |
| U15 | Legacy JSON still deserializes | 27 | Done |
| U16 | Gate bands at exact boundary values | 5 | W2 |
| U17 | Question builder: options at most 26, `NOT_ENTITY`/`NONE` added, order stable | 25 | W2 |
| U18 | More than 25 entity types gives the chosen outcome | 25 | W2 |
| U19 | Pruning by domain/range. Empty edges counts `unpruned`. | 26 | W2 |
| U20 | Self-loop relation is dropped | 55 | W2 |
| U21 | Mention and pair caps hold | 21 | W2 |
| U22 | State trim to token budget counts `truncated` | 20 | W2 |
| U23 | Mapping fills `ExtractionResult`, uses the entity-id normalizer, sets chunk ids | 52 | W2 |
| U24 | Unlisted letter is rejected | 32, 33 | W2 |
| U25 | Malformed backend body is an error | 32 | W2 |
| U26 | Cache key stable. Differs on model, contract, pack size. | 22, 23 | W2 |
| U27 | Pack splitter respects 64 questions and 64 KiB | 58 | W2 |
| U28 | Empty or whitespace chunk makes no backend call | 31 | W2 |
| U29 | Non-English warning flag | 30 | W4 |
| U30 | Extractor with fake backend turns a sentence into entities and relations | 52 | W2 |
| U31 | All `NOT_ENTITY` gives an empty result | 24 | W2 |
| U32 | `decision:` prefix counts as local provider | 18 | W5 |
| U33 | Env validation rejects bad pack size and preset | 02 | W4 |
| U34 | Status DTO redacts userinfo and key | 35 | W5 |
| U35 | `provider_name` and `model_name` | 18 | W2 |

## I — Rust integration

| ID | Check | EC | Wave |
|----|-------|----|------|
| I01 | Upload with `extraction_mode=decision` writes decision to task metadata | 12 | W4 |
| I02 | Unknown word returns 422 `invalid_extraction_mode` | 01 | W4 |
| I03 | No field and workspace `decision` gives source `workspace`. Empty field is inherit. | 07, 09 | W4 |
| I04 | Workspace create and update: set, clear, invalid, in-memory parity | 02, 05, 14 | W4 |
| I05 | Document `llm` beats workspace `decision`. JSON null is absent. | 06, 11 | W4 |
| I06 | Env `decision` with no other layer | 02 | W4 |
| I07 | Backend down returns 422 `decision_backend_unavailable` | 16 | W4 |
| I08 | Model missing returns 422 `decision_model_missing` | 34 | W4 |
| I09 | Reanalyze with a changed mode purges first | 15 | W4 |
| I10 | `EDGEQUAKE_DECISION_ENABLED=0` returns 422 `decision_disabled` | 08 | W4 |
| I11 | `decision_stats` written. Gleaning is ignored. | 53 | W5 |
| I12 | Cost is zero for decision documents | 51 | W5 |
| I13 | PDF upload carries the mode | 12 | W4 |
| I14 | Batch with a bad word fails whole, nothing stored | 13 | W4 |
| I15 | Backend fails mid-run: task failed, retryable, mock LLM calls = 0. Timeout retried 3 times. | 16, 17, 59 | W5 |
| I16 | `/v1/systemone` returns 404: status "unsupported", upload 422 | 34 | W2 |
| I17 | Second run hits the cache. Fake server request count is 0. TTL sweep deletes old rows. | 17, 28, 29, 42 | W3 |
| I18 | Document delete deletes review rows. Reanalyze clears them. | 36, 43 | W3 |
| I19 | Workspace delete deletes cache and review rows | 37 | W3 |
| I20 | Same text in two workspaces: no cross-read | 38, 39 | W3 |
| I21 | Migration 166 applies on fresh DB and on 165. Manifest and checksums agree. | 40, 41 | W3 |
| I22 | Workspace write keeps other metadata keys | 48 | W4 |
| I23 | Status endpoint: ready, disabled, unreachable, model missing, not capable. Redacts secrets. 3 s bound. | 35, 57 | W5 |
| I24 | Worker clamp is 1 for decision mode even with a cloud chat provider | 18 | W5 |
| I25 | SPEC-117 caps apply to decision output | 21 | W5 |
| I26 | Cancel stops requests between packs | 60 | W5 |
| I27 | 500-chunk document: progress per chunk, bounded memory | 49 | W5 |
| I28 | OpenAPI contract test (`spec027_api_contract`) passes | — | W4, W5 |
| I29 | Duplicate multipart field returns 422 | 10 | W4 |

## V — vitest

| ID | Check | EC |
|----|-------|----|
| V01 | `parseExtractionMode` words | 44 |
| V02 | `buildExtractionModeField('default')` returns `{}` | 06 |
| V03 | Locale parity en, fr, zh for `extractionMode.*` | 46 |
| V04 | Word list equals shared fixture used by Rust U07 | 44 |
| V05 | Select shows inherit label with the workspace mode. Resets after upload. | 50 |
| V06 | Select disables `decision` when status blocks | 45 |
| V07 | Card sends only changed fields | 47 |
| V08 | Status indicator renders all six states with text | 45 |
| V09 | Badge shows only for `decision` | 06 |

## E — Playwright (`@spec160`, route mocks)

| ID | Flow | EC |
|----|------|----|
| E01 | Set workspace default to decision, save, reload, value persists | 07 |
| E02 | Upload with `Decision model` sends the field | 12 |
| E03 | Upload with default sends no field | 06 |
| E04 | Backend unavailable error toast with "Open settings" | 16 |
| E05 | Card shows each backend status from mocked responses | 45, 57 |
| E06 | Document detail shows mode, model, and Accept/Review/Reject counts | 51 |
| E07 | Help text states the limits. License notice shows. | 54, 56 |
| E08 | Reprocess dialog warns on a changed mode | 15 |
| E09 | fr and zh render the card without missing keys | 46 |
| E10 | `Decision model` option is disabled when status is Disabled | 45 |

## As built — web UI gates (2026-10-04)

The Playwright specs run against the **real backend** (`:8095`, Ollama, `tev1:0.8b`).
`page.route` only forwards `/api/v1` calls. The failure variants of
`GET /decision/status` are the only mocked answers, because a test cannot stop Ollama.
Specs: `edgequake_webui/e2e/spec160/`. Screenshots: [`e2e/screenshots`](e2e/screenshots/).

| ID | Spec file | What it proves | Real backend |
|----|-----------|----------------|--------------|
| E01 | `workspace-card.spec.ts` | Set Decision, save, reload: the value persists | yes |
| E01b | `workspace-card.spec.ts` | Clearing the model sends `inherit` | yes |
| E01c | `workspace-card.spec.ts` | A bad pack size blocks Save and says why | yes |
| E02 | `upload.spec.ts` | Upload with Decision sends the field; the document ends as `decision` with stats; row badge and detail stats show | yes |
| E03 | `upload.spec.ts` | Upload with the workspace default sends no field; the server records `llm` / `default` | yes |
| E04 | `upload.spec.ts` | A missing model is refused by the server; the page shows the message | yes (422) |
| E04b | `upload.spec.ts` | Decision is not offered while the backend is unreachable | status mocked |
| E05 | `workspace-card.spec.ts` | Card shows disabled, unreachable, model_missing, not_capable, settings_error | status mocked |
| P01 / P01b | `polish.spec.ts` | Card in French and Chinese; the license notice is localized | yes |
| P02 | `polish.spec.ts` | Card and upload bar in the dark theme | yes |
| P03 | `polish.spec.ts` | Card on a 390 px phone, no horizontal overflow | status mocked |
| P04 | `polish.spec.ts` | Upload bar at 390 and 820 px: no overlap, nothing clipped without a scroll, full warning text present | status mocked |

Not built: E06 as specified (the table-row badge and detail stats are covered inside E02), E07 (help text and license notice are covered by E01 and P01b),
E08 (reprocess dialog warning, deferred), E09 (covered by P01 and P01b), E10 (covered by E04b and E05).

## L — Live (opt-in)

Skipped unless `EQ_LIVE_TEV1=1` and `tev1:0.8b` is pulled.

| ID | Check | EC |
|----|-------|----|
| L01 | Upload a small English document in decision mode. The graph holds at least 3 entities. An injection sentence adds no new entity type. | 19, 33 |
| L02 | CPU-pinned derived model reports `cpu_pinned: true` | 57 |
| L03 | `llama-server` backend returns the same decisions on 20 questions (W6) | — |

## Gate summary

| Wave | Gate |
|------|------|
| W1 | U01–U15 |
| W2 | U16–U31, U35, I16 |
| W3 | I17–I21 |
| W4 | I01–I10, I13, I14, I22, I28, I29, U29, U33 |
| W5 | I11, I12, I15, I23–I27, U32, U34 |
| W6 | Letter-mapping unit tests, fake OpenAI integration, L03 |
| W7 | V01–V09, E01–E10 |
| W8 | L01, L02, report, coverage script |

Next: [15 Cross-ref](15-cross-ref.md).
