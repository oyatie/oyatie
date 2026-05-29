---
doc_class: ImplementationPlan
microservice: marketplace
status: Accepted
date: 2026-05-21
owner_team: axis-marketplace
primary_adr: ADR-0329
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0249, ADR-0263, ADR-0314, ADR-0329, ADR-0330, ADR-0331]
companion_docs: [microservices/marketplace/REMEDIATION-NOTES-2026-05-21.md]
planned_enforcement_ref: oya-governance-marketplace-doc-set
naming_justifications: BNF v4 service_action_resource grammar and 13-layer-enum conformance are declared inline in this document
impl_plan_id: IP-038-revenue-share-rate-basis-points-field
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-038: revenue_share_rate_basis_points Field

## Intent
Persist the per-listing revenue-share rate in basis points (0..10000) so the rate is auditable per-DealSet, not embedded in code. Closes audit gap §3.4.B.ii item 8.

## Boundary
- Owns: listing schema `revenue_share_rate_basis_points` field on every category surface.
- Consumes: per-category default values from `capabilities/category-*.yaml`.
- Does not own: counterpart rate setting (that's the seller via the publish API).

## Deliverables
1. Add `revenue_share_rate_basis_points` to all 6 category listing schemas (already present in capabilities + contracts; this IP enforces persistence).
2. Database migration: `ALTER TABLE listings ADD COLUMN revenue_share_rate_basis_points INTEGER NOT NULL CHECK (revenue_share_rate_basis_points BETWEEN 0 AND 10000)`.
3. Per-category default values declared in `capabilities/category-{plugins=1500, apps=2000, workflows=1500, agents=2000, models=2500, datasets=2000}.yaml` (already authored).
4. Override path: tenant may set custom rate via private-offer flow; subject to `marketplace.revenue-share-rate-override.execute` Cedar action with manual review for rates exceeding 5000 (50%).
5. RevenueShareProvenance value-type (IP-035) carries this field as `oyatie_share_basis_points`.
6. Replay-fidelity: every historic accrual records the rate-as-of-DealSet-creation, immune to later contract amendments.

## Acceptance criteria
- Migration applies cleanly with default population from per-category default.
- Rate-override above 5000bp requires manual review (Cedar gate).
- Provenance round-trip preserves rate byte-identically.
- Historic accrual replay produces same Oyatie share given same input rate.

## Naming justifications
- BNF v4 action: `marketplace.revenue-share-rate-override.execute`
- Layer enum: kernel + domain + policy + adapter
- Crate name: `oya-marketplace-kernel-revenue-share-rate`
