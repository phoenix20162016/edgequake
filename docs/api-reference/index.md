---
title: API Reference
description: Complete REST API documentation for EdgeQuake.
---

> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Full reference for the EdgeQuake REST API.

**Authoritative contract:** [`edgequake_webui/openapi/openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) (version `0.23.0`). **Interactive Try-it-out:** [`/swagger-ui/`](http://localhost:8080/swagger-ui/) when the backend is running.

- **[REST API](/docs/api-reference/rest-api/)** — v0.23.0 guided overlay (progress, cancel, status fields, **parse API**).
- **[Extended API](/docs/api-reference/extended-api/)** — Tasks, pipeline queue-metrics, WebSocket progress, PDF cancel.
- **[Document Upload](/docs/api-reference/document-upload-quick-reference/)** — JSON vs file vs PDF vs batch decision tree.
- **[Lineage Endpoints](/docs/api-reference/lineage-endpoints/)** — Provenance, mm-assets, convert vs ingest.
- **[Parse API (SPEC-094)](/docs/api-reference/rest-api/#parse-api-spec-094)** — stateless PDF→Markdown, no document residue.
- **[Ingestion cancel & fairness](/docs/ingestion-cancel-and-fairness.md)** — Cancel SSOT, claim/lease, tenant fairness, store contention.
- **[Decision extraction](/docs/concepts/decision-extraction/)** — `extraction_mode`, `GET /api/v1/decision/status`, `GET /api/v1/decision/models` (SPEC-160).
