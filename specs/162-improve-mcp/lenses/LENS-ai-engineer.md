# LENS — AI Engineer

Parent: [README](../README.md) · Primary: [00-why](../00-why.md) · [05-contract-delta](../05-contract-delta.md)

## Job

Design the tool surface so a host agent can chain tools without guessing.

## Agent loop (intended)

```text
  eq_document_list  →  know the corpus
  eq_search         →  hits + retrieval_id  (scoped by document_ids)
  eq_entity_get     →  same hit.id
  eq_neighborhood   →  center + edges
  eq_graph_image    →  PNG when the agent needs a picture
  eq_document_get   →  text_page for reading
  eq_document_download → byte pages for large bodies
```

## Token budgets

| Payload | Default | Why |
|---------|---------|-----|
| Text page | 8000 chars | Fits common context windows; page for more |
| Download | 65536 bytes | Survives typical host channel limits |
| structuredContent | 8/24/80 KiB | SPEC-152 budget classes |

## Anti-patterns

- Treating empty neighborhood as “no such entity”.
- Inventing document lists from search.
- Ignoring `filter_result` / not_found codes.
- Re-emitting a slug that was not returned as an agent id.
