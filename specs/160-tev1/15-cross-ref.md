# 15 — Cross-reference

Parent: [README](README.md) · Prev: [14](14-e2e-test-matrix.md)

Generated from the EC table in [12](12-edge-cases.md) and the test tables in
[14](14-e2e-test-matrix.md). Re-check after every edit. Coverage on 2026-10-04:
60 ECs, each with at least one defined test, no ID gaps.

## Law ↔ WHY ↔ Feature ↔ Wave ↔ EC

| Law | Title | WHY | Features | Waves | ECs (EC-160-nn) | Detail |
|-----|-------|-----|----------|-------|-----------------|--------|
| LAW-160-1 | Vendor-neutral mode name | WHY-160-5 | F-160-01, F-160-02 | W1, W7 | 44 | [01](01-first-principles.md) |
| LAW-160-2 | One precedence chain | WHY-160-5 | F-160-01, F-160-02, F-160-15 | W1, W4 | 03, 04, 09, 11, 12, 14, 47 | [08](08-api-contract.md) §Mode words |
| LAW-160-3 | Closed options only | WHY-160-3 | F-160-03, F-160-09 | W2 | 32, 33 | [06](06-architecture.md) §Flow |
| LAW-160-4 | No silent fallback | WHY-160-4 | F-160-02, F-160-08 | W1, W4, W5 | 01, 02, 05, 08, 10, 13, 16, 17, 34, 45, 57 | [08](08-api-contract.md) §Errors |
| LAW-160-5 | Only ACCEPT enters graph | WHY-160-3 | F-160-06 | W2, W3 | 23, 24, 30, 55, 56 | [06](06-architecture.md) §Gate |
| LAW-160-6 | Default stays `llm` | WHY-160-1 | F-160-14 | W1 | 06, 40, 41, 48, 50, 53 | [07](07-data-model.md) §Fingerprint |
| LAW-160-7 | Same output contract | WHY-160-3 | F-160-03 | W2 | 52 | [06](06-architecture.md) §Output mapping |
| LAW-160-8 | Bounded work | WHY-160-2 | F-160-03, F-160-04 | W2 | 20, 21, 25, 26, 31, 42, 43, 49, 58, 59, 60 | [03](03-cpu-inference-study.md) |
| LAW-160-9 | Mode change marks stale | WHY-160-5 | F-160-14 | W1, W4 | 07, 15, 27, 28 | [07](07-data-model.md) §Fingerprint |
| LAW-160-10 | Determinism and cache | WHY-160-3 | F-160-07 | W3 | 22, 29 | [07](07-data-model.md) §Cache key |
| LAW-160-11 | Provenance | WHY-160-2 | F-160-10 | W2, W3 | 36, 37, 38, 39 | [07](07-data-model.md) §Provenance |
| LAW-160-12 | CPU first-class | WHY-160-4 | F-160-04, F-160-05 | W2, W6 | 18, 19 | [03](03-cpu-inference-study.md) |
| LAW-160-13 | No bundled weights | WHY-160-4 | F-160-11 | W7 | 54 | [05](05-product-spec.md) §Licence |
| LAW-160-14 | Observable | WHY-160-2 | F-160-08, F-160-10, F-160-13 | W5, W7 | 35, 46, 51 | [08](08-api-contract.md) §Status |

Every law has at least one EC.

## Wave ↔ EC

| Wave | ECs closed |
|------|-----------|
| W1 | 01, 02, 03, 04, 05, 06, 15, 27 |
| W2 | 19, 20, 21, 22, 23, 24, 25, 26, 31, 32, 33, 34, 52, 55, 58, 59 |
| W3 | 29, 36, 37, 38, 39, 40, 41, 42, 43 |
| W4 | 01, 02, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 28, 30, 48 |
| W5 | 16, 17, 18, 35, 49, 51, 53, 57, 60 |
| W7 | 44, 45, 46, 47, 50, 54, 56 |
| W8 | 56 |

## Feature ↔ doc ↔ wave

| Feature | Product | Architecture | Data | API | UX | Front | Wave |
|---------|---------|--------------|------|-----|----|-------|------|
| F-160-01 Workspace default | [05](05-product-spec.md) | [06](06-architecture.md) | [07](07-data-model.md) §Metadata | [08](08-api-contract.md) §Workspace | [09](09-ux-ui-spec.md) §Workspace card | [10](10-frontend-architecture.md) | W4, W7 |
| F-160-02 Upload override | [05](05-product-spec.md) | [06](06-architecture.md) §Resolution | [07](07-data-model.md) | [08](08-api-contract.md) §Upload | [09](09-ux-ui-spec.md) §Upload selector | [10](10-frontend-architecture.md) §Call sites | W4, W7 |
| F-160-03 DecisionExtractor | [05](05-product-spec.md) | [06](06-architecture.md) | — | — | — | — | W2 |
| F-160-04 Ollama backend | [05](05-product-spec.md) | [06](06-architecture.md) §Backends | — | [08](08-api-contract.md) §Env | — | — | W2 |
| F-160-05 Logprobs backend | [05](05-product-spec.md) | [06](06-architecture.md) §Backends | — | [08](08-api-contract.md) §Env | — | — | W6 |
| F-160-06 Gate and review | [11](11-ml-quality-spec.md) | [06](06-architecture.md) §Gate | [07](07-data-model.md) §Migration | [08](08-api-contract.md) | [09](09-ux-ui-spec.md) §Document surfaces | — | W3, W5 |
| F-160-07 Cache | — | [06](06-architecture.md) | [07](07-data-model.md) §Cache key | — | — | — | W3 |
| F-160-08 Status | [05](05-product-spec.md) | [06](06-architecture.md) | — | [08](08-api-contract.md) §Status | [09](09-ux-ui-spec.md) §Backend status | [10](10-frontend-architecture.md) | W5 |
| F-160-09 Ontology | [11](11-ml-quality-spec.md) §Ontology | [06](06-architecture.md) | — | — | — | — | W2 |
| F-160-10 Provenance, stats | — | [06](06-architecture.md) §Output mapping | [07](07-data-model.md) §Stats | [08](08-api-contract.md) §Document read | [09](09-ux-ui-spec.md) | — | W2, W3 |
| F-160-11 Workspace card | [05](05-product-spec.md) | — | — | — | [09](09-ux-ui-spec.md) | [10](10-frontend-architecture.md) | W7 |
| F-160-12 Upload selector | [05](05-product-spec.md) | — | — | — | [09](09-ux-ui-spec.md) | [10](10-frontend-architecture.md) | W7 |
| F-160-13 Badge, review count | [05](05-product-spec.md) | — | — | — | [09](09-ux-ui-spec.md) | [10](10-frontend-architecture.md) | W7 |
| F-160-14 Fingerprint | [05](05-product-spec.md) | — | [07](07-data-model.md) §Fingerprint | — | — | — | W1, W4 |
| F-160-15 Env vars | [05](05-product-spec.md) | — | — | [08](08-api-contract.md) §Env | — | — | W4 |

## Story ↔ test

See [05](05-product-spec.md) §Stories. Each US row lists its tests.

## Spike ↔ decision

| Spike | Decides | Doc |
|-------|---------|-----|
| SP-160-1 | Crate or port | [06](06-architecture.md) §Dependency choice |
| SP-160-2 | Ship `openai_logprobs` or mark experimental | [13](13-implementation-plan.md) W6 |
| SP-160-3 | Default pack size | [11](11-ml-quality-spec.md) §Why packing changes answers |
| SP-160-4 | Logprobs fallback rule | [06](06-architecture.md) §Backends |
| SP-160-5 | Release license text | [05](05-product-spec.md) §Licence |
| SP-160-6 | x86 capacity table | [03](03-cpu-inference-study.md) |
| SP-160-7 | Status text for derived models | [09](09-ux-ui-spec.md) |
| SP-160-8 | More than 25 entity types | [12](12-edge-cases.md) EC-160-25 |

## Lens ↔ doc

| Lens | Primary docs |
|------|--------------|
| [Product Owner](lenses/LENS-product-owner.md) | [00](00-why.md), [05](05-product-spec.md) |
| [Full Stack](lenses/LENS-full-stack.md) | [04](04-code-as-is.md), [06](06-architecture.md), [08](08-api-contract.md) |
| [Database](lenses/LENS-database.md) | [07](07-data-model.md) |
| [UX / UI](lenses/LENS-ux-ui.md) | [09](09-ux-ui-spec.md) |
| [Front](lenses/LENS-front.md) | [10](10-frontend-architecture.md) |
| [AI Engineer](lenses/LENS-ai-engineer.md) | [02](02-source-study.md), [06](06-architecture.md), [11](11-ml-quality-spec.md) |
| [ML Engineer](lenses/LENS-ml-engineer.md) | [03](03-cpu-inference-study.md), [11](11-ml-quality-spec.md), [measurements](measurements/README.md) |
