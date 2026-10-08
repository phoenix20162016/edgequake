# 00 — WHY

Parent: [README](README.md) · Next: [01 First principles](01-first-principles.md)

## The problem in one sentence

An agent holds only handles. A handle that one tool emits must be accepted by
every other tool. Today `eq_search` emits `ent:{ws}:{slug}` and the graph tools
cannot open that handle.

## Five WHYs

**WHY-162-1 — Why does the agent id break?**  
Search builds `ent:{ws}:{slug}` from the entity name. Graph tools strip the
prefix to a bare slug, then call `get_node(slug)`. The store key is
`{ws_uuid}::SLUG`. The lookup misses. Tests hide this by seeding bare ids.

```text
  eq_search hit.id = ent:{ws}:ACTION_FUSION
           |
           v  resolve_entity_lookup() → "ACTION_FUSION"
           |
           v  get_node("ACTION_FUSION")
           |
     store key = "{ws_uuid}::ACTION_FUSION"
           |
           X  miss → eq/not_found  OR  ok + entities=[]
```

ROOT → LAW-162-1 (emit == accept), LAW-162-2 (one resolver)

**WHY-162-2 — Why is empty success a defect?**  
`eq_neighborhood` returns `entities=[]` for an unknown id and for a real node
with no edges. The agent cannot tell absence from emptiness.

ROOT → LAW-162-3

**WHY-162-3 — Why do filters lie?**  
`document_ids` on `eq_entity_search` is ignored. Unknown document ids are
silent. Pattern match hits title only. The agent cannot see a bad id or a
filter that removed all hits.

ROOT → LAW-162-4

**WHY-162-4 — Why do payloads blow the channel?**  
`include=text` returns a resource URI, not text. Download defaults to 4 MiB.
An agent host may drop the result.

ROOT → LAW-162-5

**WHY-162-5 — Why do two names become two entities?**  
`SELF-ATTENTION` and `SELF_ATTENTION` normalize to different storage ids.
Ingest does not fold separators. The graph fragments.

ROOT → LAW-162-6

## Product statement

Fix the contract between tools before adding new tools. The intended id form
is already in `ids.rs`. The graph tools do not use that form for the store
lookup.
