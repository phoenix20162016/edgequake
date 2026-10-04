# LENS — Product Owner (SPEC-160)

Parent: [README](../README.md) · Docs: [00](../00-why.md), [05](../05-product-spec.md)

## Job

Decide who gets value, what we promise, and what we refuse to promise.

## Findings

1. The value is **private, cheap, repeatable** extraction on a CPU host. It is
   not "better graphs".
2. The measured relation quality is modest ([03](../03-cpu-inference-study.md)).
   A wrong promise costs trust. The UI and docs state the limit.
3. The control pattern already exists (workspace default plus upload override).
   Users learn nothing new.
4. The weights license is open. This is the largest release risk.

## Value and cost

| Persona | Value | Cost to them |
|---------|-------|--------------|
| Self-hoster | Extraction with no cloud key | Install Ollama 0.35+ and pull a model |
| Privacy owner | Text stays on the host | Fewer relations, English only |
| Cost owner | No token spend | CPU time. Slow on big corpora. |

## Decisions I need from the team

| ID | Decision | Default |
|----|----------|---------|
| PO-1 | Ship W8 "preview" label if ACCEPT precision misses 0.85 | Yes |
| PO-2 | Hold release until SP-160-5 (license) closes | Yes |
| PO-3 | Review UI is a later spec | Yes ([05](../05-product-spec.md) OP-160-1) |
| PO-4 | Mode name is `decision`, not `tev1` | Yes (LAW-160-1) |

## Success measures

| Measure | Target |
|---------|--------|
| Share of new workspaces that try `decision` | Track only. No target at launch. |
| ACCEPT-band relation precision | At least 0.85 |
| Upload failures from `decision_backend_unavailable` | Track. High values mean setup friction. |
| Support questions about quality | Track. Feeds the help text. |

## Risks

| Risk | Mitigation |
|------|-----------|
| Users expect LLM-grade relations | Help text, W8 report, preview label |
| Setup friction (Ollama version, model pull) | Status card with copy button |
| License change blocks the feature | Opt-in design. No bundled weights. |

## Out of scope reminder

Training, bundling weights, review UI, query-time use, non-English.
