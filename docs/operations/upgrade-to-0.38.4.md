---
title: "Upgrade to EdgeQuake v0.38.4"
description: "Upgrade from v0.38.3 to v0.38.4. Schema stays at 224."
---

# Upgrade to EdgeQuake v0.38.4

> **From:** v0.38.3 (schema **224**) · **To:** v0.38.4 (schema **224**)

`edgequake migrate` applies nothing new. Restart every API replica after the image bump so they serve the new binary.

**demo.edgequake.com:** this cut installs the 0.38.4 API and WebUI. Password auth remains the demo default, and no operator step runs beyond the image bump.

A provider whose base URL already ends in `/v1` now refreshes correctly. Before this cut, discovery probed `/v1/v1/models` and then required a chat ping, so a Mistral, OpenRouter, or NVIDIA provider reported `chat ping failed` and its newly pulled models stayed invisible. A provider that exists only in the environment also gets a `provider_connections` row on refresh, so its discovered models can be registered. The refresh control appears for a Platform Admin only; that matches the endpoint's `ApiRequireAdmin` gate, which SPEC-179 introduced.

The data hierarchy panel no longer reports "no lineage" while a document is still finishing ingestion. A 404 during the run reads as pending and polls until the record lands.

**SPEC-001 Acc:** not re-scored on this cut. Mix retrieval is unchanged. The latest valid pack remains [`medical-mid-20261024T031725Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20261024T031725Z/) (`valid: true`). EdgeQuake **0.8084**, LightRAG **0.7681**, delta **+0.0403**, 95% CI **[-0.0241, +0.1036]** (n_paired 72). The interval includes 0, so this is a statistical tie. EdgeQuake does not claim a win.
