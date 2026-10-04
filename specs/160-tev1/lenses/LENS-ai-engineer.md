# LENS — AI Engineer (SPEC-160)

Parent: [README](../README.md) · Docs: [02](../02-source-study.md), [06](../06-architecture.md), [11](../11-ml-quality-spec.md)

## Job

Fit a closed-decision model into a pipeline built for open extraction. Keep the
interface, change the engine.

## Findings

1. **Interface fit.** `EntityExtractor::extract(chunk) -> ExtractionResult` is
   all downstream code needs. The decision extractor satisfies it (LAW-160-7).
2. **No free text.** The model chooses letters. Entity and relation
   descriptions come from the source sentence by template. Downstream summary
   steps that expect richer descriptions can see short ones. Check in W5 that
   summarization and embeddings still behave (OQ-160-2).
3. **Protocol, not prompt.** The questions are the product. The pure files
   (`propose`, `question`, `prune`, `gate`) hold the logic. Backends only carry
   questions.
4. **Pack validity.** Packing several questions per prompt changes answers
   (13 vs 27 relations). Pack size is a first-class setting and part of the cache
   key.
5. **Quadratic cost.** Prompt tokens grow faster than linearly with pack size
   on Ollama ([03](../03-cpu-inference-study.md)). Default pack is 4.
6. **Calibration.** Scores are ordered evidence. Cutoffs are uncalibrated.
   Presets are guesses until W8.
7. **Injection.** The model can only choose a listed option. A hostile chunk
   can flip one decision. It cannot write text into the graph.

## Backend contract

```text
  DecisionQuestion { kind: choice|noul|score, state, question, options[], pack_ctx }
  DecisionAnswer   { letter|yes_no, score, confidence?, model, contract, degraded: bool }
```

| Backend | Score source | Degrade rule |
|---------|--------------|--------------|
| Ollama System One | Candidate scoring | None |
| OpenAI logprobs | Softmax over option-letter logprobs | More than `top_logprobs` options: split rounds, or mark `degraded` and send to REVIEW |

## What to evaluate before shipping

| Check | Why |
|-------|-----|
| Letter agreement between backends (SP-160-2) | A second backend must not change answers |
| Pack 1 vs 4 (SP-160-3) | Sets the default |
| ACCEPT precision (W8) | The graph quality floor |
| Injection set (L01) | Closed-output claim |

## Not in scope

Fine-tuning, prompt search, tiered models (OP-160-2), hybrid description
generation (OP-160-4).

## Hand-off

The ML Engineer lens owns the evaluation protocol and the calibration.
See [LENS-ml-engineer](LENS-ml-engineer.md).
