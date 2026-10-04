# LENS — ML Engineer (SPEC-160)

Parent: [README](../README.md) · Docs: [03](../03-cpu-inference-study.md), [11](../11-ml-quality-spec.md), [measurements](../measurements/README.md)

## Job

Say what the data shows, what it does not show, and how to close the gap.

## What the data shows

| Claim | Support |
|-------|---------|
| 0.8B types entities as well as 4B | F1 0.906 vs 0.914 |
| 0.8B links relations poorly | F1 0.14–0.27 vs 0.43–0.47 |
| CPU matches GPU on quality for 4B | Entity F1 equal. Relation F1 within 0.03. |
| Pack 4 is the time/quality knee on CPU | 4B: 87 s vs 146 s (pack 64) vs 278 s (pack 1) |
| Prompt tokens grow faster than linearly with pack | 191 to 11,328 tokens for 1 to 12 questions |

## What it does not show

| Gap | Why it matters |
|-----|----------------|
| One host (Apple M4 Max) | x86 server speed is unknown. SP-160-6. |
| 14 short documents | One document moves relation F1 by about 0.1 |
| One run per row | No variance |
| Two documents out of ontology | Only relative comparison is valid |
| No LLM baseline on this set | We cannot rank modes |
| Score calibration unknown | Gate cutoffs are guesses |

## Protocol to close the gaps

[11](../11-ml-quality-spec.md) §Evaluation protocol. Key points:

1. At least 50 hand-labeled chunks from a real workspace, plus SPEC-001 data.
2. Three runs per system. Report mean and range.
3. Cache off during evaluation.
4. Record CPU model, cores, RAM, Ollama version, model digest.
5. Report entity F1, relation F1 (direction-aware and not), ACCEPT precision,
   review rate, time per KB, and Acc score.

## Calibration plan

```text
  labeled decisions ─► (score, truth) pairs ─► choose accept_prob, reject_prob
        │                                              │
        │   target: ACCEPT precision >= 0.85           ▼
        │           true facts in REJECT <= 0.10   gate-presets.json {fitted: true}
        ▼
  per backend and model (scores are not comparable across them)
```

## Failure analysis buckets

Direction flip, pronoun miss, cross-sentence link, non-English, injection,
long state, entity explosion. Count each bucket in W8 and report.

## Reproducibility controls

Temperature 0. Cache off. Fixed pack size. Contract string stored. Model digest
recorded. Same golden set and ontology per run.

## Decision rules

| If | Then |
|----|------|
| ACCEPT precision under 0.85 at `balanced` | Raise `accept_prob`. Re-measure. |
| Still under 0.85 | Ship as "preview" with `strict` default |
| Relation F1 gap between 0.8B and 4B stays above 0.15 | Offer 4B as the quality model in the card. Reopen the tiered profile (OP-160-2). |
| Pack 1 beats pack 4 by more than 0.03 on relations | Make pack 1 the 4B default |

## Out of scope

Training or fine-tuning. This spec only consumes the released model.
