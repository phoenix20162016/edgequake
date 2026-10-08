# LENS — Full Stack Developer

Parent: [README](../README.md) · Primary: [04-architecture](../04-architecture.md) · [07-implementation-plan](../07-implementation-plan.md)

## Job

Ship small modules with one reason to change. Reuse REST resolve and storage
`EntityId`. Keep MCP projections thin.

## Module ownership

| Module | Owns |
|--------|------|
| `entity_ref.rs` | Parse / format / resolve agent ids |
| `doc_scope.rs` | Hard document filters for MCP |
| `doc_text.rs` | Tenant-checked body load |
| `cursor.rs` | Typed pagination tokens |
| `edge_filter.rs` | Strong / weak edge split |
| `key_resolver.rs` | Fold merge at ingest |

## DRY checks

- Graph tools share one resolver.
- Search and graph share `agent_id_for_node`.
- Neighborhood and graph_image share `edge_filter`.
- Get / download / resources share `doc_text`.

## Test strategy

Red e2e first (worker harness, scoped ids). Then unit tests for fold and cursor.
Keep SPEC-152 and SPEC-161 green.
