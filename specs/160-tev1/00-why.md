# 00 — WHY

Parent: [README](README.md) · Next: [01 First principles](01-first-principles.md)

## The problem in one sentence

KG extraction with a chat LLM costs tokens, varies between runs, and needs a
large model. Many users cannot send documents to a cloud LLM. Many users also
cannot run a large local model on the server CPU.

## Five WHYs

**WHY-160-1 — Why add a mode at all?**  
Today every document passes through one open-ended LLM prompt per chunk. The
LLM writes entities and relations as free text. EdgeQuake then parses, caps,
and merges that text ([04](04-code-as-is.md) §Extractor).

**WHY-160-2 — Why is open extraction a problem?**  
Open extraction has three costs. Each cost grows with corpus size.

| Cost | Cause | Evidence in the repo |
|------|-------|----------------------|
| Tokens (money) | One large prompt and one large answer per chunk | `pipeline/extraction.rs` pricing path |
| Variance | The model chooses what to list and how to word it | SPEC-117 caps and SPEC-091 gates exist to contain it |
| Model size | Open extraction needs a strong model | Local providers run at concurrency 1 (`LOCAL_MAX_CONCURRENT_EXTRACTIONS`) |

**WHY-160-3 — Why does a decision model change this?**  
A decision model picks one option from a short list. Tev1 returns one option
letter ([02](02-source-study.md)). The answer space is closed. A closed answer
is bounded in cost, repeatable at temperature 0, and cacheable.

```text
  open extraction                         closed decisions
  ───────────────                         ────────────────
  chunk ──► "list all entities           sentence ──► mention "Alice"
            and relations"                    │
            │                                 ▼
            ▼                            "Is Alice: A PERSON  B ORG  C NOT_ENTITY?"
     free text (unbounded)                    │ one letter
            │                                 ▼
     parse, cap, merge                   "Does Alice WORKS_AT Acme: A YES  B NONE?"
                                              │ one letter + score
                                              ▼
                                         gate: ACCEPT / REVIEW / REJECT
```

**WHY-160-4 — Why run it on the server CPU?**  
Self-hosted EdgeQuake runs in Docker without a GPU. The 0.8B model is 812 MB.
It runs on CPU. We measured it ([03](03-cpu-inference-study.md)). On an Apple
M4 Max CPU the 0.8B model processed 14 short test documents (2.3 KB) in 26 s at pack size 4.
That speed is a lower bound on server CPU time. W0 re-measures on x86.

**WHY-160-5 — Why select the mode per upload and per workspace?**  
Users differ by document, not only by tenant. A legal workspace may want
`decision` for every file. A mixed workspace may want `llm` by default and
`decision` for one sensitive upload. The existing system already offers this
shape for extract caps (SPEC-117) and entity types (SPEC-114). The user already
knows the pattern.

## Causal chain

```text
  privacy need ─┐
  cost need ────┼─► want extraction without a cloud LLM
  CPU-only host ┘                    │
                                     ▼
                   closed decisions by a small model (Tev1)
                                     │
                    ┌────────────────┼───────────────────┐
                    ▼                ▼                   ▼
             bounded cost     repeatable output     runs on CPU
                    └────────────────┼───────────────────┘
                                     ▼
                     new mode: extraction_mode = decision
                                     │
                     ┌───────────────┴────────────────┐
                     ▼                                ▼
              per upload override              workspace default
```

## What this spec does not claim

1. It does not claim better graph quality than a frontier LLM. Our CPU run
   scored relation F1 between 0.14 and 0.47 on a 14-document set
   ([measurements](measurements/README.md)). That set is too small to rank
   modes.
2. It does not claim a license for Tev1 weights. Together AI states the
   license is "being finalized" ([02](02-source-study.md)).
3. It does not replace the `llm` mode. The default stays `llm`.

## Who benefits

| Persona | Gain | Section |
|---------|------|---------|
| Self-hoster without GPU | Extraction with no cloud key | [05](05-product-spec.md) |
| Privacy-bound team | Text never leaves the host | [05](05-product-spec.md) |
| Cost owner | Zero token spend for extraction | [05](05-product-spec.md) |
| Operator | Repeatable runs and a review queue | [11](11-ml-quality-spec.md) |

Cross-reference: [15-cross-ref.md](15-cross-ref.md).
