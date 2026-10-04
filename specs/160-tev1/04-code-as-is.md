# 04 — Code as-is

Parent: [README](README.md) · Prev: [03](03-cpu-inference-study.md) · Next: [05 Product](05-product-spec.md)

Paths are relative to `edgequake/crates/` unless marked `web:` (relative to
`edgequake_webui/src/`). Line numbers are from 2026-10-04. Use symbol names if
a line moves.

## Seam map

```text
 upload handler ──► document_admission ──► task_meta ──► prepare.rs ──► IngestionPipelineOptions
 (file/text/batch)   (fingerprint)        (JSON)         (reads meta)         │
 pdf_upload/upload.rs  ▲                                                      ▼
   (separate path!)    │                                  WorkspacePipelineFactory.resolve_for_ingestion
                       │                                         │  schema + caps from workspace metadata
                       └── ProcessFingerprintInput               ▼
                                                        build_ingestion_pipeline
                                                                 │
                                                                 ▼
                                                        SOTAExtractor (EntityExtractor)
```

## Extractor

| Anchor | Fact |
|--------|------|
| `edgequake-pipeline/src/extractor/mod.rs:61` | `trait EntityExtractor`: `extract`, `extract_batch`, `name`, `model_name`, `provider_name`. The decision extractor implements this trait. |
| `edgequake-pipeline/src/extractor/types.rs:8` | `ExtractionResult { entities, relationships, source_chunk_id, metadata, input_tokens, output_tokens, extraction_time_ms }` |
| `edgequake-pipeline/src/extractor/types.rs:135` | `ExtractedEntity { name, entity_type, description, importance, source_spans, embedding, source_chunk_ids, source_document_id, … }` |
| `edgequake-pipeline/src/extractor/types.rs:273` | `ExtractedRelationship { source, target, relation_type, description, weight, keywords, embedding, source_chunk_ids, … }` |
| `edgequake-pipeline/src/chunker/types.rs:132` | `TextChunk` fields the extractor reads |

Gap: the entity `description` is free text in the LLM mode. The decision mode
has no description from a model. It builds a short template from the source
sentence ([06](06-architecture.md) §Output mapping).

## Pipeline build

| Anchor | Fact |
|--------|------|
| `edgequake-pipeline/src/ingestion_pipeline.rs:15` | `IngestionPipelineOptions`. W1 added `extraction_mode` and `with_extraction_mode`. |
| `edgequake-pipeline/src/ingestion_pipeline.rs:200` | `build_ingestion_pipeline`. It builds the LLM extractor only. W5 adds a sibling that takes a `DecisionBackend`. |
| `edgequake-api/src/workspace_pipeline_factory.rs:58` | `resolve_for_ingestion`. It reads workspace metadata and builds a per-document pipeline. W5 reads the mode here. |
| `edgequake-pipeline/src/prompts/entity_type_policy.rs:55` | `EntityExtractionSchema::from_workspace_metadata`: `entity_types`, `relation_types`, `relation_edges` (SPEC-114/114b). The decision ontology comes from this schema. |
| `edgequake-pipeline/src/prompts/extract_caps.rs:118` | `ExtractionCaps::resolve_for_ingestion` (document > workspace > env). **Template for the mode resolver.** |
| `edgequake-pipeline/src/pipeline/config.rs:94,102` | `LOCAL_MAX_CONCURRENT_EXTRACTIONS = 1` and `is_local_extraction_provider` |
| `edgequake-pipeline/src/pipeline/extraction.rs` | Cost estimate with a gpt-4.1-nano pricing fallback. The decision mode reports zero cost. |
| `edgequake-pipeline/src/merger/lineage.rs` | Citation gate (SPEC-091 RM2): rows need source chunk ids. |
| `edgequake-storage/src/entity_id.rs:226` | SSOT for entity name normalization. Decision output must use it. Two older normalizers exist. Do not add a fourth. |

## W1 additions (done)

| File | Content |
|------|---------|
| `edgequake-pipeline/src/extraction_mode.rs` | `ExtractionMode`, `ExtractionModeSource`, `resolve_extraction_mode`, `parse_mode_override`, `apply_extraction_mode_metadata`, `extraction_mode_from_metadata`, `extraction_mode_from_value`, constants `META_EXTRACTION_MODE`, `EXTRACTION_MODE_ENV` |
| `edgequake-pipeline/src/lib.rs` | Module and re-exports |
| `edgequake-pipeline/src/ingestion_pipeline.rs` | `extraction_mode` field (default `Llm`) and builder |
| `edgequake-api/src/services/process_fingerprint.rs` | `extraction_mode` field. Digest adds `\|em=decision` only for a non-default mode. |

## Upload and admission

| Anchor | Fact |
|--------|------|
| `edgequake-api/src/handlers/documents/upload/document_admission.rs:219` | `task_meta` JSON built here. W4 adds `extraction_mode` (resolved word). |
| `…/document_admission.rs:235` | SPEC-117 pattern: add keys only when set. Copy it. |
| `…/document_admission.rs:397` | `ProcessFingerprintInput::from_ingest_fields`. W4 appends `.with_extraction_mode`. |
| `…/document_admission.rs:576` | `MultipartUploadFields::ingest_text_field`: the one place that maps a multipart text field to input. W4 adds `extraction_mode`. |
| `edgequake-api/src/handlers/documents/upload/file_upload.rs`, `batch_upload.rs` | Callers of the multipart parser |
| `edgequake-api/src/handlers/pdf_upload/upload.rs` | **A separate PDF upload path.** It does not use `MultipartUploadFields`. W4 must wire it too (EC-160-12). |
| `edgequake-api/src/processor/pdf_processing.rs` | PDF conversion, then text ingest |
| `edgequake-api/src/processor/text_insert/prepare.rs:208` | Builds `IngestionPipelineOptions` from task metadata. SPEC-117 caps are read at line ~236. W4 reads the mode the same way. |
| `edgequake-api/src/processor/text_insert/prepare.rs:482` | Second fingerprint call site |
| `edgequake-api/src/services/multimodal/reanalyze.rs:148` | Third fingerprint call site (`from_document_metadata`, which W1 extended) |

## Workspace

| Anchor | Fact |
|--------|------|
| `edgequake-api/src/handlers/workspaces_types/requests.rs` | Request DTOs. W4 adds `extraction_mode`. |
| `edgequake-api/src/handlers/workspaces_types/responses.rs:73` | `WorkspaceResponse`. W4 adds the stored word and the resolved effective mode. |
| `edgequake-api/src/handlers/workspaces_types/map.rs` | Entity-to-DTO mapping |
| `edgequake-api/src/handlers/workspaces/workspace_crud.rs` | Create and update handlers |
| `edgequake-core/src/extract_budget_metadata.rs` | Metadata helper pattern (SPEC-117). Model for the mode helper. |
| `edgequake-core/src/types/multitenancy/requests.rs:29,308` | Core `CreateWorkspaceRequest`, `UpdateWorkspaceRequest` |

## Migrations

| Anchor | Fact |
|--------|------|
| `edgequake/migrations/manifest.toml` | Latest version 165. `compat_serve_max = 165`. W3 adds 166 and raises `compat_serve_max`. |
| `edgequake/migrations/checksums.lock` | CI pin. W3 updates it (EC-160-40). |
| Entry shape | `phase = "expand"`, `no_transaction = false`, `lock_class = "ddl_access_exclusive"` |

## Web UI

| Anchor | Fact |
|--------|------|
| `web: constants/extract-budget.ts` | SPEC-117 helper pattern: `inherit`/`custom`, check pair |
| `web: components/workspace/workspace-extract-budget-card.tsx` | Workspace card pattern. Rendered in `app/(dashboard)/workspace/page.tsx:294` and `app/w/[slug]/workspace/page.tsx:256`. |
| `web: lib/upload/perform-file-upload.ts` | `PerformFileUploadOptions`. Rule: set a field only when it differs from workspace defaults. |
| `web: lib/upload/pdf-upload-form-data.ts` | `buildPdfUploadFormData` |
| `web: components/documents/document-manager.tsx` | Holds upload option state, such as `pdfParserBackend` |
| `web: components/documents/document-dropzone.tsx` | Renders the per-upload select with a `workspaceDefaultLabel` and `showInheritHint`. **UX precedent for the mode select.** |
| `web: components/settings/vision-extract-controls.tsx` | Second user of `workspaceDefaultLabel` |
| `web: types/workspace.ts`, `lib/api/edgequake/workspaces.ts` | Types and client |
| `web: locales/{en,fr,zh}.json` | i18n. Add keys to all three. |
| `edgequake_webui/e2e/spec159/` | Playwright style: route mocks, tag `@spec159`. SPEC-160 uses `e2e/spec160/` and `@spec160`. |

## Gaps this spec closes

| Gap | Closed by |
|-----|-----------|
| No second extractor exists | W2 |
| No mode field on options | W1 (done) |
| Fingerprint ignores mode | W1 (done) |
| Workspace has no mode key | W4 |
| Upload has no mode field | W4 |
| PDF upload is a separate path | W4 |
| No decision storage | W3 |
| No UI | W7 |

Next: [05 Product](05-product-spec.md).
