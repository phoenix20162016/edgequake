# LENS — AI Engineer (SPEC-159)

Parent: [README](../README.md) · Retrieval: [07](../07-retrieval-contract.md) · Product: [03](../03-product-spec.md)

## Job

Ensure the seeded question and retrieval settings give the model a fair chance
to answer about the selected entity/page **without lying about capabilities**
or fighting the user’s mode preferences.

## What the model actually receives (v1)

```text
  message  = seed (possibly user-edited)
  mode     = user default (often mix)
  document_filter.document_ids = [doc] | omitted
  NO page gate
  NO explicit seed_entities on chat path
```

Keyword extraction on the message typically surfaces the entity label /
document title as HL/LL keywords — that is the v1 “hint”.

## Seed design principles

1. **Name the object** — label/title first.
2. **Ask for relations / claims** — matches graph-RAG strengths.
3. **Mention page as location** — helps the model + user, not a filter. Quote converted page text when the UI already has it so the asked claim is grounded in the seed.
4. **Keep short** — user should want to edit, not delete a wall of text.
5. **No fabricated quotes** — only attach passage text when the UI already
   has it (W4 chunk Ask / quotePassage parity, ≤600 chars).

## Anti-patterns

| Anti-pattern | Why bad |
|--------------|---------|
| Auto-submit seed | Burns tokens; user cannot steer |
| Sticky force `local` / `naive` | Surprises power users; LAW-159-7 |
| “Answer using only page N” system prompt hijack | Dishonest vs retrieval |
| Stuffing full entity description into seed | Noisy; description is in graph pane |

## Future upgrades (out of v1)

| Upgrade | When |
|---------|------|
| Pass `ll_keywords=[entity]` on chat | API exposes it on WebUI path |
| Page/chunk allowed set | Engine + Acc ready |
| Optional mode suggestion chip | UX research |
| Structured tool: `ask_about_entity` for MCP | Separate pack |

## Evaluation note

SPEC-159 does not change Acc gold. Handoff is WebUI-only. If seed templates
regress answer quality, adjust templates — do not change retrieval defaults
silently.

## Cross-ref

LAW-159-2,6,7 · F-159-04 · EC-159-05,17 · OPP-159-06,09
