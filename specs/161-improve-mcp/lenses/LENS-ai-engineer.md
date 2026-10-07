# LENS — AI Engineer

Parent: [README](../README.md) · Primary: [05-contract-delta](../05-contract-delta.md) · `profile.rs` instructions

## Job

Make tool descriptions and initialize `instructions` teach the model the
correct control loops without inventing REST calls or essays.

## Findings

- Query instructions currently say “query-only” (F-161-07 / `QUERY_INSTRUCTIONS`).
- Models will call delete if it is listed; annotations + confirm are required
  (LAW-161-6 residual risk).
- Search must not be described as answering (LAW-161-5).

## Decisions

### Control instructions (replace default line 8)

Normative outline (exact text in implementation):

```text
1. eq_document_list before answering "what's in the workspace".
2. Scope with document_ids when the user names a paper.
3. eq_search → eq_fetch(view=toc). Escalate view only if needed.
4. Treat entity names in ALL_CAPS as slugs; show Title Case to the user.
5. If truncation.truncated is true, fetch next_cursor before concluding the corpus is small.
6. Ignore Artifact/DRAWING entities unless the user asks about a figure; use eq_asset_get for pixels.
7. Never claim an LLM "answer" from EdgeQuake; the tools return evidence.
8. Ingest via eq_ingest (or eq_upload_* for large files) and poll eq_task_get.
   Deletes require confirm: true. Download with eq_document_download.
   Graph picture: eq_graph_image centered on an entity_id.
```

Query lockdown keeps the old query-only line 8.

### Tool description cues

| Tool | Cue |
|------|-----|
| `eq_ingest` | Returns task_id; poll before claiming indexed |
| `eq_document_delete` | Destructive; require confirm:true |
| `eq_asset_get` | Use after include:assets or known asset_id |
| `eq_graph_image` | Picture of neighborhood; not a substitute for eq_neighborhood JSON when the host needs edges |
| `eq_search` | Evidence hits + retrieval_id |

## Success measures

- Unset profile instructions do not say query-only (EC-161-41 + initialize assert).
- EC-161-46: no essay field in search result.
- Models that pass confirm still hit real delete (EC-161-27).
