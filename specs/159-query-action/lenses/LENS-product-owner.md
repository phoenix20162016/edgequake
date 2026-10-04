# LENS — Product Owner (SPEC-159)

Parent: [README](../README.md) · Product: [03](../03-product-spec.md)

## Job

Ship the shortest path from **seeing** an entity/page to **asking** about it,
without burning trust (auto-send, fake page filters) or scope (Studio embed).

## Value hypothesis

Analysts lose context when they leave Graph/Documents for a blank Query. A
one-click Ask that preserves selection evidence increases grounded questions
and reduces “what was I looking at?” friction.

## Scope decisions (locked)

| In | Out |
|----|-----|
| Graph menu + details Ask | Auto-submit |
| Document header Ask | Backend page filter |
| Neighborhood companion | Full Studio in Query |
| Document scope chip | Sticky mode change |
| W4 same-handoff surfaces | View Documents provenance fix (OPP-159-05) |

## Acceptance the PO signs

1. Demo: node → Ask → neighborhood + seed → user Enter → answer with citations.
2. Demo: page 12 → Ask → PDF page 12 + scope → user Enter.
3. Proof: Playwright shows **no** request before Enter.
4. Copy review: no “only this page” claims.

## Risks

| Risk | Mitigation |
|------|------------|
| Users expect page-only RAG | Honesty in UX + docs ([07](../07-retrieval-contract.md)) |
| Ask pollutes old threads | Force new conversation |
| Scope creep into Studio embed | LAW-159-5/8 |

## Success narrative

See [README](../README.md#success-narrative). Metric direction in [03](../03-product-spec.md#success-metrics-post-ship-observational).

## Cross-ref

WHY A–E · LAW-159-1…12 · US-159-01…04 · OPP-159-*
