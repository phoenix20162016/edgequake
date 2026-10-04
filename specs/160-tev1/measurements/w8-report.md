# W8 report — Decision mode quality through the EdgeQuake API

Parent: [measurements](README.md) · Protocol: [11 ML quality spec](../11-ml-quality-spec.md)

Date: 2026-10-04. Host: Apple M4 Max, Ollama 0.35.1 (Metal GPU, **not** CPU-pinned).
Backend: EdgeQuake v0.30.0 at the working tree, isolated stack on `:8095`.
Script: [`raw/w8_eval.py`](raw/w8_eval.py). Raw rows: [`raw/w8-results.json`](raw/w8-results.json).

## What was run

One throw-away workspace per row. Each ingested the same two short English
documents (7 + 5 sentences) through `POST /api/v1/documents`. The graph was then
read back through `/api/v1/graph/entities` and `/api/v1/graph/relationships`.

- Gold entities: 11. Gold relations: 10 (direction ignored, names matched by substring).
- `llm gemma4` is the control: `gemma4:latest` through the normal LLM extractor.
- `0.8b` is `tev1:0.8b`. `4b` is `tev1:latest` (4.5 GB).
- Presets are the three server gate presets (`strict`, `balanced`, `recall`). Pack size is the default (4).

## Results

| Config | Entities hit (of 11) | Relations hit (of 10) | Graph entities | Graph relations | Review rows | Model calls | Wall time |
|--------|---------------------:|----------------------:|---------------:|----------------:|------------:|------------:|----------:|
| llm gemma4 (control) | 11 | 9 | 13 | 14 | — | — | 18.3 s |
| 0.8b strict | 2 | 0 | 2 | 0 | 21 | 8 | 3.4 s |
| 0.8b balanced | 5 | 0 | 5 | 2 | 16 | 12 | 6.4 s |
| 0.8b recall | 8 | 4 | 11 | 14 | 7 | 17 | 6.4 s |
| 4b strict | 8 | 5 | 9 | 10 | 12 | 16 | 18.6 s |
| 4b balanced | 10 | 8 | 11 | 22 | 8 | 19 | 15.5 s |
| 4b recall | 10 | 8 | 13 | 23 | 5 | 19 | 12.6 s |

Nothing was rejected in any row. One 0.8b strict document landed as
`partial_failure` ("Extracted 0 entities"), which is the existing status for a
document with no entities.

## Reading

1. **The 4B model is close to the LLM control on this set.** 4b balanced finds
   10 of 11 entities and 8 of 10 relations, against 11 and 9 for the control.
2. **The 0.8B model is not usable at the default `balanced` preset.** It finds 5
   of 11 entities and no gold relation. The gate moves 16 facts to review.
   With `recall` it reaches 8 and 4. It is a CPU and demo option, not a quality option.
3. **The preset matters more than any other knob.** On both models `strict` ⟶
   `recall` raises recall a lot and lowers the review count.
4. **Relation counts above 10 are not precision.** 4b balanced adds 22 relations
   for 10 gold ones. The extra edges were not judged by hand, so this report makes
   no precision claim.
5. **Speed is not the point on this host.** The GPU run of the 4B model took
   12–19 s, the same range as the control. The CPU numbers stay in
   [03](../03-cpu-inference-study.md).

## Limits

1. Two documents, 11 gold entities, 10 gold relations. One entity or relation is 9–10 points.
2. One run per row. No variance estimate.
3. Gold matching is by substring on names, so it can over-count slightly.
4. English only, one ontology (the default entity types).
5. SPEC-001 LightRAG Acc was **not** run. The release runbook still requires
   `make bench001-doctor` then `make bench` before a tag.

## Decisions

- `GATE_PRESETS_CALIBRATED` stays `false`. The UI keeps the "Uncalibrated" badge.
- The mode keeps the "preview" label.
- Operators who want quality should set `EDGEQUAKE_DECISION_MODEL` to the 4B model
  and use `recall` or `balanced`. The 0.8B model is for smoke tests and low-memory hosts.
- Follow-up (not done): run the protocol of [11](../11-ml-quality-spec.md) on the
  14-document golden set and on the SPEC-001 corpus before any preset is called calibrated.
