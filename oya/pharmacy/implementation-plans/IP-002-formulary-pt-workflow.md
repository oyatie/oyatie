# IP-002 — Formulary kernel + P&T workflow + therapeutic interchange

- **Status**: draft
- **Owner**: axis-pharmacy
- **Authority**: ADR-0332
- **Depends on**: IP-001
- **Estimated complexity**: M

## Goal

Implement the formulary classifier (preferred / non-preferred / restricted / non-formulary), the P&T committee workflow, prior-auth criteria evaluation, and therapeutic interchange.

## Acceptance criteria

- AC-1: `oya-pharmacy-formulary-kernel` types: `FormularyEntry`, `TherapeuticInterchangeRule`, `PriorAuthCriterion`, `PTCommitteeReview`.
- AC-2: `oya-pharmacy-formulary-domain::classify(tenant, cell, rxcui)` returns `FormularyStatus`.
- AC-3: P&T workflow state machine: `Proposed → Reviewed → Voted → Effective | Rejected | Deferred`.
- AC-4: Therapeutic interchange usecase: `propose_interchange(source_rxcui, indication) → target_rxcui` with rationale.
- AC-5: REST `GET /Formulary` and admin POST endpoints for P&T.
- AC-6: AsyncAPI event `oya.pharmacy.formulary.non-formulary-override` emitted when an override is accepted.
- AC-7: Unit tests for classifier with per-cell overlay; P&T workflow state-machine tests.

## Tasks

1. Kernel + domain types.
2. P&T state machine.
3. Interchange rule engine.
4. Per-cell overlay merging.
5. REST + AsyncAPI wiring.
6. Cedar gate for non-formulary override.
7. Tests.

## Risks

- Cell overlay ordering ambiguity → strict precedence rule documented in domain.
- Effective-date scheduling drift → use HLC + deterministic effective-date worker.
