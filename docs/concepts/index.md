---
title: Core Concepts
description: Understand the foundational concepts behind EdgeQuake's Graph-RAG approach.
---

> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Learn the key ideas that power EdgeQuake.

- **[Graph-RAG](/docs/concepts/graph-rag/)** — How graph-based retrieval augmented generation works.
- **[Knowledge Graphs](/docs/concepts/knowledge-graph/)** — Entities, relationships, and graph structure.
- **[Entity Extraction](/docs/concepts/entity-extraction/)** — Turning unstructured text into structured entities.
- **[Decision Extraction](/docs/concepts/decision-extraction/)** — Preview mode: closed questions on a local decision model (SPEC-160).
- **[Hybrid Retrieval](/docs/concepts/hybrid-retrieval/)** — Combining vector search with graph traversal.

**Operational concepts (v0.23.0):**

- **[Pipeline Progress](/docs/deep-dives/pipeline-progress/)** — Task IDs, WebSocket/SSE progress, `display_status` / `ui_phase` (SPEC-048 / SPEC-057).
- **[PDF Processing](/docs/deep-dives/pdf-processing/)** — Vision convert, mm-assets, convert→ingest split, `Cancelled` status (SPEC-047 / SPEC-057).
- **[Ingestion cancel & fairness](/docs/ingestion-cancel-and-fairness.md)** — Cancel SSOT, tenant fairness, cooperative abort.
