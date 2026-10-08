# 04 — Architecture

Parent: [README](README.md) · Prev: [03 Findings](03-findings.md) · Next: [05 Contract delta](05-contract-delta.md)

## Target module map

```text
 edgequake-storage/entity_id.rs
   + fold_slug_key(raw)
   + EntityId::separator_variants(cap=16)
           │
           v
 mcp/project/entity_ref.rs   (NEW)
   AgentEntityId parse/format
   agent_id_for_node(ws, &GraphNode)
   resolve_entity_node(graph, ctx, id)
           │
     ┌─────┴──────┬──────────────┐
     v            v              v
 eq_entity_get  eq_neighborhood  eq_graph_image
 eq_entity_search (ids via agent_id_for_node)
 eq_search hits   (ids via agent_id_for_node)

 mcp/project/doc_scope.rs   (NEW)
   DocumentScope::resolve → allowed / unknown / pattern
           │
     ┌─────┴──────┐
     v            v
  eq_search   eq_entity_search

 mcp/project/doc_text.rs    (NEW)
   tenant-checked markdown / original
           │
     ┌─────┴──────────┬────────────┐
     v                v            v
 eq_document_get  eq_document_download  resources/read

 mcp/project/cursor.rs      (NEW)
   {entities|relationships|chunks|bytes}:{offset}

 mcp/project/edge_filter.rs (NEW)
   split_edges → kept / strong / weak / dropped

 edgequake-pipeline/merger/key_resolver.rs (NEW)
   EntityKeyResolver → entity merge + relationship merge
```

## Resolve path

```text
 agent id
    │
    v  parse AgentEntityId (reject empty slug; foreign ws → not_found)
    │
    v  EntityId::exact_lookup_candidates(raw, workspace)
    │
    v  separator_variants(fold) → get_nodes_batch (one round)
    │
    v  each hit → load_node_for_tenant_context
    │
    v  miss → bounded search_nodes + fold equality
    │
    v  miss → NotFound { agent_id, tried storage id if changed }
```

## Id source (R3)

```text
  ContextEntity.graph_node_id  ─┐
  GraphNode.id                 ─┼─► agent_id_for_node(ws, id)
  NeighborhoodNode.id          ─┘
         │
         v
   ent:{ws}:{bare_slug}
```

`search.rs` must stop using `ent.name` alone when `graph_node_id` is present.

## Ingest fold merge (R13)

```text
  ExtractedEntity.name
         │
         v  EntityId::new → exact key
         │
         v  EntityKeyResolver
              exact hit? → use it
              else probe separator_variants via get_nodes_batch
              else create new
         │
         v  relationship endpoints use same resolver map
```

## Budgets and bytes

| Channel | Cap | Where |
|---------|-----|-------|
| `structuredContent` | 8 / 24 / 80 KiB | `budget.rs` (unchanged classes) |
| Text page | 8000 chars default, 32000 max | `eq_document_get` |
| Download chunk | 65536 bytes hard | `blob.rs` / `download.rs` |
| Graph PNG | `ImageContent` outside structured | SPEC-161 |
