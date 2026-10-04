# Measurements

Parent: [03 CPU study](../03-cpu-inference-study.md)

Raw output of the 2026-10-04 runs. Host: Apple M4 Max. Runtime: Ollama 0.35.1.
Client: edgextract Rust CLI. Data: golden set of 14 documents, 2,317 bytes,
`tech_docs` ontology. Wall time includes model load.

## Files

| File | Run |
|------|-----|
| `raw/eval_gpu4b.txt` | 4B, Metal GPU, pack 64 |
| `raw/eval_cpu4b_t8.txt` | 4B, CPU, 8 threads, pack 64 |
| `raw/eval_cpu4b_t8_mq4.txt` | 4B, CPU, 8 threads, pack 4 |
| `raw/eval_cpu4b_t8_mq1.txt` | 4B, CPU, 8 threads, pack 1 |
| `raw/eval_cpu08_t4.txt` | 0.8B, CPU, 4 threads, pack 64 |
| `raw/eval_cpu08_t2.txt` | 0.8B, CPU, 2 threads, pack 64 |
| `raw/eval_cpu08_t4_mq4.txt` | 0.8B, CPU, 4 threads, pack 4 |
| `raw/eval_cpu08_t4_mq1.txt` | 0.8B, CPU, 4 threads, pack 1 |
| `raw/bench2.py` | Quadratic-prompt benchmark. It adds a random ref to `state` per call to defeat the prompt cache. |
| `raw/w8_eval.py`, `raw/w8-results.json` | W8: EdgeQuake API run, 7 configs, see [w8-report](w8-report.md) |
| `raw/req12.json` | One 12-question System One request |
| `raw/extract_01_edgequake_mq1.json` | Extraction output for one document at pack 1 |

## Known limits of this data

1. One host. No x86 server run.
2. 14 short documents. Relation F1 moves by 0.1 with one document.
3. Documents 13 and 14 do not fit the ontology. Compare rows with each other.
4. One run per row. No variance estimate.
5. First timings were wrong because of the prompt cache. `bench2.py` fixes it.

## Reproduce (W0)

```text
  ollama pull tev1:0.8b
  ollama create tev1-cpu -f Modelfile      # FROM tev1:0.8b, num_gpu 0, num_thread N
  edgextract eval --model tev1-cpu --max-questions 4 <golden set>
```

Exact flags can differ by edgextract version. Check `edgextract --help`.
Record host CPU model, core count, and RAM next to every new row.
