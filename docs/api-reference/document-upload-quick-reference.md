---
title: 'Document Upload Quick Reference'
---

# Document Upload Quick Reference

> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

> **Choose the Right Endpoint for Your Upload Method**

EdgeQuake provides multiple ingestion paths. **Production uploads are async by default** — file and PDF endpoints enqueue tasks and return `task_id` for progress/cancel. JSON text upload supports optional sync (`async_processing: false`) for small payloads only; do not assume sync-first for files or PDFs.

**Progress key:** subscribe to server **`task_id`** (e.g. `pdf-<uuid>`), not optional client batch `track_id`. WebSocket: `ws://localhost:8080/ws/progress/{task_id}`.

---

## Quick Decision Tree

```
What are you ingesting?
├─ Raw text / structured JSON (no file)?
│  └─ POST /api/v1/documents  (application/json)
│     • Set async_processing: true for production (returns task_id)
│     • async_processing: false only for small sync smoke tests
│
└─ Files from disk?
   ├─ PDF (vision convert → separate Insert ingest)?
   │  ├─ Single PDF
   │  │  └─ POST /api/v1/documents/pdf   ← required (not /documents/upload)
   │  └─ Multiple PDFs
   │     └─ POST /api/v1/documents/pdf/batch
   │
   ├─ Images (PNG/JPG/GIF/WEBP)?
   │  └─ POST /api/v1/documents/upload
   │
   └─ Text files (TXT, MD, JSON; API also CSV/HTML/XML/YAML)?
      ├─ Single file  → POST /api/v1/documents/upload
      └─ Multiple     → POST /api/v1/documents/upload/batch

   DOCX / Excel → not supported (SPEC-121). Export to PDF or Markdown.

After async admission:
  task_id → GET /api/v1/ingestion/{task_id}/progress
         → ws://localhost:8080/ws/progress/{task_id}
  PDF only → GET /api/v1/documents/pdf/progress/{task_id}
  Cancel   → POST /api/v1/tasks/{task_id}/cancel
```

**Convert → ingest (PDF):** `POST /documents/pdf` enqueues `TaskType::PdfProcessing` (convert only). After durable markdown + PDF `Completed`, the worker enqueues `TaskType::Insert` for KG ingest under a separate lease. PDF `Completed` means convert artifact only — doc stage continues through Insert. See [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md#convert-then-ingest-spec-057-p2).

---

## Method 1: Text/JSON Upload

**Endpoint**: `POST /api/v1/documents`  
**Content-Type**: `application/json`  
**Use When**: Programmatic text ingestion (API integration, pre-extracted markdown)

> **No sync-first myth:** OpenAPI default is `async_processing: false` for backward compatibility on JSON only. For anything non-trivial, set `async_processing: true` and poll/WebSocket on returned `task_id`.

### Example: Basic Text Upload

```bash
curl -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Marie Curie was a pioneering physicist...",
    "title": "Marie Curie Biography"
  }'
```

### Example: With Metadata

```bash
curl -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: workspace-uuid" \
  -d '{
    "content": "Your document text here...",
    "title": "Document Title",
    "metadata": {
      "source": "wikipedia",
      "author": "John Doe",
      "category": "science"
    },
    "enable_gleaning": true,
    "max_gleaning": 2
  }'
```

### Request Body Schema

```typescript
{
  content: string;              // Required: Document text
  title?: string;               // Optional: Document title
  metadata?: object;            // Optional: Custom metadata
  async_processing?: boolean;   // true = task queue (recommended); false = sync (small text only)
  track_id?: string;            // Optional batch correlation (not PDF progress key)
  enable_gleaning?: boolean;    // Optional: Multi-pass extraction (default: true)
  max_gleaning?: number;        // Optional: Max gleaning passes (default: 1)
  use_llm_summarization?: boolean; // Optional: LLM-powered descriptions (default: true)
}
```

---

## Method 2: Single File Upload

**Endpoint**: `POST /api/v1/documents/upload`  
**Content-Type**: `multipart/form-data`  
**Use When**: Non-PDF files or generic file upload (PDFs work but **`POST /documents/pdf`** is preferred for vision convert → ingest semantics)

Returns **`task_id`** when processing is async (typical for PDF/large files).

### Example: PDF Upload (preferred path)

```bash
curl -X POST http://localhost:8080/api/v1/documents/pdf \
  -H "X-Workspace-ID: workspace-uuid" \
  -F "file=@research_paper.pdf" \
  -F "title=My Research Paper"
```

Response includes `task_id` (`pdf-<uuid>`) — use for progress WebSocket and cancel.

---

## Method 2b: Single File Upload (generic)

**Endpoint**: `POST /api/v1/documents/upload`  
**Use When**: TXT, MD, JSON, or PDF when you do not need PDF-specific routes

### Example: With Configuration

```bash
curl -X POST http://localhost:8080/api/v1/documents/upload \
  -F "file=@document.pdf" \
  -F "title=Financial Report" \
  -F 'metadata={"category": "finance", "year": 2024}' \
  -F 'config={"enhance_tables": true, "mode": "Hybrid"}'
```

### Supported File Types

| Extension | MIME Type          | Max Size | Notes                     |
| --------- | ------------------ | -------- | ------------------------- |
| `.pdf`    | `application/pdf`  | 50 MB    | Supports vision/hybrid mode |
| `.txt`    | `text/plain`       | 10 MB    | Plain text               |
| `.md`     | `text/markdown`    | 10 MB    | Markdown formatting      |
| `.json`   | `application/json` | 10 MB    | Structured data          |

### Form Fields

| Field      | Type   | Required | Description                              |
| ---------- | ------ | -------- | ---------------------------------------- |
| `file`     | File   | Yes      | The file to upload                       |
| `title`    | String | No       | Custom title (defaults to filename)      |
| `metadata` | JSON   | No       | Custom metadata object                   |
| `config`   | JSON   | No       | PDF processing configuration             |

---

## Method 3: Batch File Upload (text / images)

**Endpoint**: `POST /api/v1/documents/upload/batch`  
**Content-Type**: `multipart/form-data`  
**Use When**: Uploading multiple **non-PDF** files at once (TXT/MD/JSON/images).  
**PDFs:** use `POST /api/v1/documents/pdf/batch` (or N× `/documents/pdf`). PDF parts on `/upload/batch` fail per-file with a SPEC-123 message (SPEC-132).

### Example: Multiple Files

```bash
curl -X POST http://localhost:8080/api/v1/documents/upload/batch \
  -F "files=@doc1.txt" \
  -F "files=@doc2.txt" \
  -F "files=@doc3.md"
```

### Response Format

```json
{
  "results": [
    {
      "filename": "doc1.pdf",
      "document_id": "doc-uuid-1",
      "status": "success",
      "chunk_count": 15
    },
    {
      "filename": "doc2.txt",
      "status": "duplicate",
      "duplicate_of": "doc-uuid-2"
    },
    {
      "filename": "doc3.md",
      "status": "failed",
      "error": "File too large"
    }
  ],
  "processed": 2,
  "duplicates": 1,
  "failed": 0
}
```

---

## Method 3b: Batch PDF Upload

**Endpoint**: `POST /api/v1/documents/pdf/batch`  
**Content-Type**: `multipart/form-data`  
**Use When**: Uploading multiple PDFs in one request while preserving PDF-specific processing semantics

### Example: Multiple PDFs

```bash
curl -X POST http://localhost:8080/api/v1/documents/pdf/batch \
  -F "files=@paper1.pdf" \
  -F "files=@paper2.pdf" \
  -F "enable_vision=true"
```

### Response Format

```json
{
  "total_files": 2,
  "accepted": 1,
  "duplicates": 1,
  "failed": 0,
  "results": [
    {
      "filename": "paper1.pdf",
      "status": "processing",
      "pdf_id": "pdf-uuid-1",
      "task_id": "task-uuid-1"
    },
    {
      "filename": "paper2.pdf",
      "status": "duplicate",
      "duplicate_of": "pdf-uuid-existing"
    }
  ]
}
```

---

## Method 4: Directory Scan

**Endpoint**: `POST /api/v1/documents/scan`  
**Content-Type**: `application/json`  
**Use When**: Bulk uploading from a server directory

### Example: Recursive Scan

```bash
curl -X POST http://localhost:8080/api/v1/documents/scan \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/data/documents",
    "recursive": true,
    "extensions": [".pdf", ".txt", ".md"],
    "max_files": 1000
  }'
```

### Request Schema

```typescript
{
  path: string;              // Required: Directory path
  recursive?: boolean;       // Optional: Scan subdirectories (default: true)
  extensions?: string[];     // Optional: File extensions to include
  max_files?: number;        // Optional: Max files to process (default: 1000)
}
```

---

## Common Errors and Fixes

### Error: "Expected request with `Content-Type: application/json`"

**Cause**: Using `-F` (multipart) with `/api/v1/documents`

**Fix**: Use `/api/v1/documents/upload` for file uploads

```bash
# ❌ WRONG
curl -X POST http://localhost:8080/api/v1/documents \
  -F "file=@doc.pdf"

# ✅ CORRECT
curl -X POST http://localhost:8080/api/v1/documents/upload \
  -F "file=@doc.pdf"
```

---

### Error: "Failed to parse the request body as JSON"

**Cause**: Using `-F` with a JSON endpoint or missing quotes

**Fix**: Use `-d` with properly formatted JSON

```bash
# ❌ WRONG
curl -X POST http://localhost:8080/api/v1/documents \
  -F "content=text here"

# ✅ CORRECT
curl -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -d '{"content": "text here"}'
```

---

### Error: "missing field `content`"

**Cause**: JSON upload missing required `content` field

**Fix**: Include `content` in request body

```bash
# ❌ WRONG
curl -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -d '{"title": "My Doc"}'

# ✅ CORRECT
curl -X POST http://localhost:8080/api/v1/documents \
  -H "Content-Type: application/json" \
  -d '{"content": "Document text...", "title": "My Doc"}'
```

---

## Extraction mode (SPEC-160)

Optional on every ingest route above. Omit it and the document follows the workspace, then `EDGEQUAKE_EXTRACTION_MODE`, then `llm`.

| Field | Where | Values |
| ----- | ----- | ------ |
| `extraction_mode` | JSON body or multipart field | `inherit`, `llm`, `decision` |
| `decision_gate_preset` | multipart field, optional | `strict`, `balanced`, `recall` |

`decision` requires the local decision backend. A missing model or a down backend returns **422** and stores nothing. See [Decision extraction](../concepts/decision-extraction.md).

---

## API Endpoint Summary

| Endpoint                          | Method | Content-Type               | Purpose                    |
| --------------------------------- | ------ | -------------------------- | -------------------------- |
| `/api/v1/documents`               | POST   | `application/json`         | Text/JSON (optional sync)  |
| `/api/v1/documents/pdf`           | POST   | `multipart/form-data`      | Single PDF (convert→ingest)|
| `/api/v1/documents/pdf/batch`     | POST   | `multipart/form-data`      | Multiple PDFs              |
| `/api/v1/documents/upload`        | POST   | `multipart/form-data`      | Single file (generic)      |
| `/api/v1/documents/upload/batch`  | POST   | `multipart/form-data`      | Multiple files             |
| `/api/v1/documents/scan`          | POST   | `application/json`         | Scan directory for files   |
| `/api/v1/ingestion/{task_id}/progress` | GET | N/A                   | Ingest progress (poll)     |
| `/ws/progress/{task_id}`          | WS     | N/A                        | Per-track progress + cancel|
| `/api/v1/tasks/{task_id}/cancel`  | POST   | N/A                        | Cancel (canonical)         |

---

## Best Practices

1. **PDFs:** use `POST /documents/pdf` (or `/pdf/batch`) for convert → ingest and PDF cancel routes
2. **Always capture `task_id`** from upload responses — sole progress/cancel/WebSocket key (SPEC-054)
3. **Prefer async** — sync JSON is for small tests; file/PDF paths enqueue tasks
4. **Subscribe early** — `ws://localhost:8080/ws/progress/{task_id}` before long converts
5. **Cancel via** `POST /api/v1/tasks/{task_id}/cancel` — see [cancel SSOT](../ingestion-cancel-and-fairness.md)
6. **Include tenant/workspace headers** on all authenticated routes

---

## Next Steps

- **OpenAPI SSOT**: [`openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) · [`/swagger-ui/`](http://localhost:8080/swagger-ui/)
- **Full API Reference**: [REST API Documentation](/docs/api-reference/rest-api/)
- **PDF tutorial**: [PDF Ingestion](/docs/tutorials/pdf-ingestion/)
- **Cancel & fairness**: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
- **Troubleshooting**: [Common Issues](/docs/troubleshooting/common-issues/#1-document-upload-errors)
