# 03 — CPU inference study

Parent: [README](README.md) · Prev: [02](02-source-study.md) · Next: [04 Code as-is](04-code-as-is.md)

Question: can the EdgeQuake server run Tev1 on CPU?  
Answer: yes, at a measured cost. The cost is high for large documents. Pack
size and model size decide it.

## Method

| Item | Value |
|------|-------|
| Host | Apple M4 Max, CPU only for the CPU rows |
| Runtime | Ollama 0.35.1 |
| Client | edgextract Rust CLI |
| Data | Golden set of 14 documents, 2,317 bytes, `tech_docs` ontology |
| Time | Wall clock. It includes model load. |

Docs 13 and 14 are company news scored with a tech ontology. Use the numbers
only to compare rows with each other. Raw files:
[measurements/](measurements/README.md).

CPU rows used a derived model with `PARAMETER num_gpu 0` and
`PARAMETER num_thread N`. `ollama ps` showed 100% CPU. The derived model kept
`capabilities:["decision"]`.

## Results

| Config | Entities F1 | Relations F1 | Wall |
|--------|-------------|--------------|------|
| 4B Metal GPU, pack 64 | 0.914 | 0.439 | 53 s |
| 4B CPU t8, pack 64 | 0.914 | 0.450 | 146 s |
| 4B CPU t8, pack 4 | 0.914 | 0.465 | 87 s |
| 4B CPU t8, pack 1 | 0.914 | 0.431 | 278 s |
| 0.8B CPU t4, pack 64 | 0.906 | 0.140 | 55 s |
| 0.8B CPU t2, pack 64 | 0.906 | 0.140 | 77 s |
| 0.8B CPU t4, pack 4 | 0.906 | 0.269 | 26 s |
| 0.8B CPU t4, pack 1 | 0.906 | 0.247 | 43 s |

"Pack" is the number of questions in one request.

### What the numbers say

1. **Entity typing**: 0.8B matches 4B (0.906 vs 0.914).
2. **Relation links**: 0.8B is weak (0.14–0.27). 4B reaches 0.43–0.47.
3. **CPU vs GPU for 4B**: same entity F1. CPU at pack 64 takes 2.8× the GPU
   time. CPU at pack 4 takes 1.6× the GPU pack-64 time.
4. **Pack size 4** is the knee on CPU. Pack 64 is slow. Pack 1 is also slow.
5. **Packing changes answers.** Document `01_edgequake` (383 bytes) gave 13
   relations at pack 64 and 27 at pack 1. The pack composition affects output.

### Why pack 64 is slow: quadratic prompt growth

Ollama counts `input_tokens` that grow faster than linearly with the number
of questions in one request.

| Questions | `input_tokens` |
|-----------|----------------|
| 1 | 191 |
| 2 | 570 |
| 3 | 1,137 |
| 6 | 3,744 |
| 9 | 7,119 |
| 12 | 11,328 |

On document `01_edgequake`, pack 64 used 74,619 tokens in 24 s. Pack 1 used
9,903 tokens in 19.9 s. Pack 4 used 22,845 tokens in 19.2 s.

```text
  tokens per request
   │                                         ● 12 q: 11,328
   │                                  ● 9 q
   │                           ● 6 q
   │                  ● 3 q
   │            ● 2 q
   │      ● 1 q
   └──────────────────────────────────────────► questions per request
```

Cause: the requests repeat shared state per question, and the runtime scores
the questions as one sequence. Do not rely on this cause. W0 spike SP-160-3
checks it.

## Throughput model

Measured on 0.8B, CPU prompt throughput was about 1,000–4,000 tokens/s,
depending on batch. Use this planning formula:

```text
  doc_seconds ≈ (questions × tokens_per_question) / tokens_per_second
```

| Model | Pack | Measured rate (wall, with model load) |
|-------|------|----------------------------------------|
| 0.8B CPU | 64 | about 24 s per KB (55 s for 2.3 KB) |
| 0.8B CPU | 4 | about 11 s per KB (26 s for 2.3 KB) |
| 4B CPU | 64 | about 63 s per KB (146 s for 2.3 KB) |
| 4B CPU | 4 | about 38 s per KB (87 s for 2.3 KB) |

An M4 Max is a very fast CPU. Server x86 cores can be slower. Treat these
rates as a floor on time. Gate W2 on the x86 measurement (SP-160-6).

### Capacity example (planning only)

One PDF page holds about 4 KB of text. Rates come from the table above.

| Model | Pack | Time per page | Pages per hour, one worker |
|-------|------|---------------|----------------------------|
| 0.8B | 4 | about 45 s | about 80 |
| 4B | 4 | about 150 s | about 24 |

Model load time sits inside these rates. A warm model is faster on long
documents. W0 measures warm rates.

Local providers run at concurrency 1 (`LOCAL_MAX_CONCURRENT_EXTRACTIONS`).
CPU decision mode inherits that cap. See EC-160-18.

## Options

| ID | Option | CPU path | Verdict |
|----|--------|----------|---------|
| A | Ollama System One, CPU-pinned derived model | Measured | **Default** (W2) |
| B | `llama-server` plus letter scoring by logprobs | Not measured | **Second backend** (W6) |
| C | Embedded llama.cpp in the Rust binary | Build and licence cost | Rejected |
| D | ONNX CPU runtime | No Tev1 ONNX export in the sources read | Rejected for now |
| E | candle (Rust) | Not studied. No evidence of `qwen35` support in the sources read. | Rejected for now |
| F | Hosted serverless or vLLM | Not CPU | Optional `openai_logprobs` target |

### A — Ollama System One on CPU

Pros: measured, supports `choice`/`noul`/`score`, supports `keep_alive`.  
Cons: needs Ollama 0.35 or later. Needs a derived model for CPU pinning.  
Setup (operator):

```text
  FROM tev1:0.8b
  PARAMETER num_gpu 0
  PARAMETER num_thread 4
```

EdgeQuake ships this Modelfile text in docs. It does not run it (LAW-160-13).

### B — llama-server with letter scoring

`llama-server` serves the `qwen35` architecture on CPU. It has an
OpenAI-compatible API. The backend sends the Tev1 system prompt, reads the
first token logprobs, and maps the letters to options. No official Tev1 GGUF
exists. The operator supplies one. Spikes SP-160-2 and SP-160-4 gate it.

Limit: more than 5 options may exceed `top_logprobs`. The backend then splits
the option list into rounds, or reads the one-token greedy letter and sets the
score to 1.0. The rule is in [06](06-architecture.md) §Backends.

### C, D, E — rejected for now

| Option | Reason |
|--------|--------|
| C | Adds a C++ build, native weights handling, and a license surface to the Rust binary. |
| D | The model card and edgextract README list no ONNX export. A custom export has no support path. |
| E | This study did not test `candle`. The sources read do not mention it. |

This study did not research C, D, or E beyond the sources in [02](02-source-study.md).
W0 checks each once against current upstream docs. A change can reopen them.

### F — hosted

Together serverless exposes logprobs. It is not CPU. The `openai_logprobs`
backend can target it. LAW-160-4 still applies: the user chooses the backend
URL. The system never picks a hosted URL on its own.

## Tiered profile (optional, later)

Type with 0.8B. Relate with 4B. This profile uses the fast typing and the
better links.

Risks: two models in RAM (about 5.3 GB), and model switching in Ollama costs
load time. Not in W2–W6. Reopen after W8 data.

## Recommendation

1. Ship backend A first with 0.8B default and 4B as a quality option.
2. Set pack size 4.
3. Run one worker. Use `keep_alive` so the model stays loaded.
4. Gate W2 exit on SP-160-6 (x86 numbers) and SP-160-7 (derived model).
5. Add backend B in W6 for operators who run `llama-server`.

## Memory

| Model | Weights | Plan RAM |
|-------|---------|----------|
| 0.8B | 812 MB | 1.5 GB |
| 4B | 4.5 GB | 6 GB |

Plan RAM includes the KV state and runtime. W0 measures it with the real pack
size.
