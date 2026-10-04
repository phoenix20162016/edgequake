# 06 — Architecture

Parent: [README](README.md) · Prev: [05](05-product-spec.md) · Next: [07 Data model](07-data-model.md)

## Components

```text
 edgequake-pipeline/src/
 ├── extraction_mode.rs            W1  mode SSOT (done)
 └── extractor/decision/
     ├── mod.rs                    W2  DecisionExtractor : EntityExtractor
     ├── question.rs               W2  build choice/noul questions (pure)
     ├── propose.rs                W2  mention proposal (pure)
     ├── prune.rs                  W2  ontology domain/range pruning (pure)
     ├── gate.rs                   W2  GatePreset, GateBand, apply_gate (pure)
     ├── map.rs                    W2  decisions → ExtractionResult (pure)
     ├── backend.rs                W2  trait DecisionBackend, DecisionQuestion, DecisionAnswer
     ├── store.rs                  W3  trait DecisionStore (cache + review)
     └── cache_key.rs              W2  one key function (DRY)

 edgequake-llm or edgequake-pipeline/src/extractor/decision/backends/
     ├── ollama_system_one.rs      W2  POST /v1/systemone
     └── openai_logprobs.rs        W6  chat completions + logprobs letter scoring

 edgequake-storage/src/decision/   W3  Postgres DecisionStore
 edgequake-api/src/
     ├── workspace_pipeline_factory.rs   W5  resolve mode, pick extractor
     └── handlers/decision_status.rs     W5  GET /api/v1/decision/status
```

Each file is one reason to change (SRP). The pure files have no I/O and no
async. They carry most tests.

## Traits

```text
  trait DecisionBackend: Send + Sync {
      async fn ask(&self, batch: &[DecisionQuestion]) -> Result<Vec<DecisionAnswer>, DecisionError>;
      async fn health(&self) -> BackendHealth;       // reachable, model present, capabilities
      fn descriptor(&self) -> BackendDescriptor;     // kind, model, contract
  }

  trait DecisionStore: Send + Sync {
      async fn get_many(&self, keys: &[CacheKey]) -> Result<Vec<Option<DecisionAnswer>>>;
      async fn put_many(&self, rows: &[(CacheKey, DecisionAnswer)]) -> Result<()>;
      async fn record_review(&self, rows: &[ReviewRow]) -> Result<()>;
  }
```

`DecisionExtractor` holds `Arc<dyn DecisionBackend>`, `Arc<dyn DecisionStore>`
(a no-op store in tests), `EntityExtractionSchema`, `GatePreset`, and limits
(DIP). It knows no HTTP and no SQL.

## Per-chunk flow

```text
  TextChunk
     │ split sentences (pure)
     ▼
  propose mentions ─────────────► M mentions per sentence (cap)
     │
     ▼
  type questions  (1 per mention) ──┐
     │                              │  cache lookup first
     ▼                              ▼
  DecisionStore.get_many ◄──── misses ──► DecisionBackend.ask (pack N=4)
     │                                         │
     ▼                                         ▼
  typed mentions (drop NOT_ENTITY)      DecisionStore.put_many
     │
     ▼
  prune pairs by ontology (relation_edges) ─► legal pairs (cap 24 per sentence)
     │
     ▼
  relation questions (1 per legal pair) ─► same cache/ask path
     │
     ▼
  gate: ACCEPT │ REVIEW │ REJECT
     │            │         └─► counted
     │            └─► DecisionStore.record_review
     ▼
  map to ExtractionResult (entities, relationships, metadata)
```

## Output mapping

LAW-160-7: the output is `ExtractionResult`.

| Field | Source |
|-------|--------|
| `entities[].name` | Mention text, normalized by `edgequake-storage/src/entity_id.rs:226` |
| `entities[].entity_type` | Chosen type label, upper snake case |
| `entities[].description` | Template: the source sentence, cut to 200 chars. No model text. |
| `entities[].importance` | Gate score of the type decision |
| `entities[].source_spans` | The sentence |
| `relationships[].relation_type` | Chosen relation label |
| `relationships[].description` | Template: the source sentence |
| `relationships[].weight` | Gate score of the relation decision |
| `relationships[].keywords` | The relation label |
| `source_chunk_ids` | The chunk id (the merger citation gate needs it) |
| `metadata` | `extraction_mode`, `decision_contract`, `decision_model`, `band_counts` |
| `input_tokens`, `output_tokens` | Sum of backend usage. `output_tokens` is 0 if the backend gives none. |

## Gate

```text
            prob/score
  1.0 ┬──────────────────  ACCEPT   (enters graph)
 0.70 ┼ accept_prob
      │                    REVIEW   (stored, not in graph)
 0.40 ┼ reject_prob
      │                    REJECT   (dropped, counted)
  0.0 ┴──────────────────
```

Presets map to edgextract `GateConfig` values:

| Preset | `accept_prob` | `reject_prob` | Use |
|--------|---------------|---------------|-----|
| `strict` | 0.85 | 0.50 | Few, certain facts |
| `balanced` (default) | 0.70 | 0.40 | edgextract defaults |
| `recall` | 0.55 | 0.30 | More facts, more doubt |

Preset values other than `balanced` are proposals. W8 calibrates all three on
labeled data ([11](11-ml-quality-spec.md)). Until then the UI labels them
"uncalibrated".

## Backends

### `ollama_system_one` (W2)

1. Build a System One request: 1–64 questions, 2–26 options, body ≤ 64 KiB.
2. Split a batch to pack size N (default 4). Split again if the body is too large.
3. `POST {base}/v1/systemone`. Read one answer per question.
4. Map `choice` letter and `noul`/`score` values to `DecisionAnswer`.
5. `health`: `GET /api/tags` plus a model check for `capabilities:["decision"]`.

### `openai_logprobs` (W6)

1. Build the Tev1 system prompt and the JSON user content.
2. Call chat completions with `temperature 0`, `max_tokens 8`, `logprobs true`,
   `top_logprobs 5`.
3. Read the first token logprobs. Map letters to options. Softmax over letters.
4. If options exceed the `top_logprobs` limit, take the greedy letter and set
   score 1.0 only when the top letter has no competitor. Else mark the answer
   `degraded` and send it to REVIEW.
5. `health`: `GET /v1/models` and one probe question.

## Resolution at ingest

```text
  upload field ──┐
  workspace key ─┼─► resolve_extraction_mode(doc, ws, env)  ──► Err ► 422 (upload) / task fail (stale data)
  env var ───────┘                │
                                  ▼ Ok(mode, source)
                  mode == llm ──► existing path (no change)
                  mode == decision ──► backend ready? ──no──► 422 decision_backend_unavailable
                                              │yes
                                              ▼
                                   DecisionExtractor pipeline
```

Admission checks readiness at upload time (fast fail). The worker checks again
before the first chunk (EC-160-16). If the backend fails during a run, the task
fails with a retryable error. It never switches mode (LAW-160-4).

## Concurrency

`is_local_extraction_provider` (`pipeline/config.rs:102`) matches exact names:
`ollama`, `lmstudio`, `lm-studio`, `lm_studio`. The fairness clamp reads the
**extract provider name from workspace LLM config**. It does not read the
extractor. A decision run would then use the cap of the chat LLM provider, which
can be a cloud provider with a high cap.

Fix (W5):

1. `DecisionExtractor::provider_name()` returns `decision:ollama` or
   `decision:openai_logprobs`.
2. `is_local_extraction_provider` accepts any name with prefix `decision:`.
   Both backends are CPU-bound by design.
3. The fairness resolver returns the `decision:` name when the resolved mode is
   `decision`. Test: EC-160-18.

The cap is 1 worker by default. An operator can raise it with
`EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS` (existing behavior).

## Dependency choice: edgextract crate or own port

| Option | Pros | Cons |
|--------|------|------|
| Use `edgextract` as a dependency (default features off) | Reuses tested logic. Gets upstream fixes. | Sync `Transport`. v0.1.0 API may move. `rusqlite` conflict risk (SP-160-1). |
| Port the decision protocol into `extractor/decision/` | Async-native. No external API risk. Fits the traits above. | We own the maintenance. Risk of drift from the upstream contract. |

Decision rule (W0): if SP-160-1 passes and the `Transport` seam wraps cleanly in
`spawn_blocking`, use the crate for propose, question, and gate logic. Else
port. The file layout above stays the same in both cases. The pure files wrap or
replace the crate calls. Callers see no difference (OCP).

## Error taxonomy

| Error | Source | Handling |
|-------|--------|----------|
| `UnknownExtractionMode` | W1 | 422 at upload. 400 at workspace write. |
| `BackendUnavailable` | Health check | 422 at upload. Task failure at worker. |
| `ModelMissing` | Health check | Same, with the pull hint |
| `ContractMismatch` | Backend reply | Task failure. No retry. |
| `BackendTimeout` | Request | Retry with backoff, then task failure |
| `QuestionTooLarge` | Builder | Split. If one question still exceeds the limit, skip the pair and count it. |

Next: [07 Data model](07-data-model.md).
