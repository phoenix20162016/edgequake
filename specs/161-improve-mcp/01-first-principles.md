# 01 — First principles

Parent: [README](README.md) · Prev: [00 WHY](00-why.md) · Next: [02 Surfaces](02-surfaces.md)

A first principle is a fact that does not depend on another rule. Each law
below follows from one of these facts, or extends a SPEC-152 law.

| Fact | Statement |
|------|-----------|
| FACT-1 | A tool result that claims a side effect without performing it is a defect. |
| FACT-2 | Tokens and file bytes have different cost and size regimes. |
| FACT-3 | MCP core has no session; cross-call state is an explicit handle. |
| FACT-4 | Write and read are different trust classes (LAW-152-7). |
| FACT-5 | Search returns evidence; the host writes the essay (LAW-152-8 grain). |

## Laws

| ID | Law | Derives from | Extends |
|----|-----|--------------|---------|
| LAW-161-1 | A claimed side effect is the REST service’s side effect. Stubs that mint ids or report `deleted: true` are defects. | FACT-1 | LAW-152-7 |
| LAW-161-2 | Tokens and bytes are different channels. `structuredContent` stays inside SPEC-152 budgets. Pixels and files use `ImageContent` or blob resources. | FACT-2 | LAW-152-2 |
| LAW-161-3 | The 1 MiB body cap stays. Large files use an upload handle. The cap is not raised. | FACT-3 | — |
| LAW-161-4 | A graph PNG is a second projection of `build_entity_neighborhood`, not a second graph query and not the WebGL exporter. | DRY | LAW-152-8 |
| LAW-161-5 | Search returns evidence. There is no `eq_answer`. | FACT-5 | LAW-152-4 |
| LAW-161-6 | Control mode is the default advertisement. Write execution still needs `edgequake:write`, workspace claim match, and `confirm: true` for delete. `EDGEQUAKE_MCP_PROFILE=query` is the opt-in hide. | FACT-4 | LAW-152-7 |
| LAW-161-7 | Illustration bytes are opt-in by id. Catalog search does not start returning drawings. | LAW-152 object model | LAW-152-3 |
| LAW-161-8 | Delete is the existing async task. The tool reports acceptance, then `eq_task_get` reports completion. | FACT-1 | LAW-152-7 |
| LAW-161-9 | The stdio package does not reimplement admission, delete, rendering, or budgets. | DRY | LAW-152 architecture |

## DRY rules

1. One admission path: `admit_document_for_processing`. Text ingest, base64
   ingest, and upload commit call it. No second pipeline in `mcp/`.
2. One upload commit function. `eq_upload_commit` and `eq_ingest.upload_ref`
   share it. Small `content_base64` uses the same commit after decode.
3. One neighborhood builder: `build_entity_neighborhood`. `eq_neighborhood`
   and `eq_graph_image` share it. Layout and raster do not query AGE.
4. One PNG encoder path for graph images. One downsample path for oversized
   illustrations (reuse the `image` crate already pulled by `edgequake-pdf`).
5. One envelope builder and one error code table from SPEC-152. New codes
   (`eq/not_implemented`, `eq/unsupported_media`) join that table; they do
   not fork a second envelope.
6. Schema SSOT remains `specs/152-new-mcp-contract/schemas/`. Rust
   `tools/list` matches those files. Stdio Zod stays a loose forwarder.

## SOLID rules

| Letter | Rule | Where |
|--------|------|-------|
| S | One module, one reason to change. Ingest, upload session, download, assets, layout, raster live in separate files under `mcp/project/`. | [04](04-architecture.md) |
| O | A new representation (e.g. SVG later) extends download / raster behind the same tool names. | [04](04-architecture.md) |
| L | MCP project functions return the SPEC-152 envelope. Callers cannot tell whether the path was REST or MCP. | LAW-161-1 |
| I | Layout does not touch storage. Raster does not query the graph. Dispatch only routes. | [04](04-architecture.md) |
| D | Project modules depend on service traits / functions (`admit_*`, `get_task_for_context`, `read_mm_asset_payload_by_id`), not Axum handlers. | [07](07-implementation-plan.md) |

## Size rule

Keep each new Rust file under 300 lines where possible. Split by reason to
change. The target module list is in [07](07-implementation-plan.md).

## How the laws map to edge cases

Each EC in [06](06-edge-cases.md) names the law it protects. Each law has at
least one test in [08](08-e2e-test-matrix.md).
