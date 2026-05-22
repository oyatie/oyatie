---
doc_class: ImplementationPlan
microservice: marketplace
status: Accepted
date: 2026-05-21
owner_team: axis-marketplace
primary_adr: ADR-0329
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0249, ADR-0263, ADR-0314, ADR-0329, ADR-0330, ADR-0331]
companion_docs: [microservices/marketplace/REMEDIATION-NOTES-2026-05-21.md]
planned_enforcement_ref: oya-governance-marketplace-doc-suite
naming_justifications: BNF v4 service_action_resource grammar and 13-layer-enum conformance are declared inline in this document
impl_plan_id: IP-032-revenue-share-cohort-id-fk
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-032: revenue_share_cohort_id FK on DealSet schema

## Intent
Add the immutable `revenue_share_cohort_id` FK (into cloud-billing) on every DealSet at creation. Closes audit gap §3.4.B.ii item 2.

## Boundary
- Owns: marketplace.DealSet domain + schema + kernel value-type.
- Consumes: cloud-billing.cohorts via typed read-only port (cohort existence verification at DealSet.create).
- Does not own: cohort creation, cohort updates, or per-tenant cohort lifecycle.

## Deliverables
1. Domain schema migration: `ALTER TABLE deal_sets ADD COLUMN revenue_share_cohort_id TEXT NOT NULL`.
2. Kernel value-type `RevenueShareCohortId(TEXT, ulid pattern)`.
3. Cedar gate at `marketplace.deal-offer-create.execute` requires `context.revenue_share_cohort_id != ""` for paid+revenue_share tenants.
4. Immutability invariant: any UPDATE of `revenue_share_cohort_id` raises domain error `MKT_E_COHORT_MUTATION_FORBIDDEN`.
5. FK presence check at DealSet.create via gRPC call `cloud-billing.cohort.exists`.
6. Audit-chain seal payload extended with `revenue_share_cohort_id`.

## Acceptance criteria
- Migration applies and rolls back cleanly.
- Cedar test verifies deny when cohort_id missing.
- Domain test verifies error on cohort mutation.
- gRPC stub correctly errors when cohort absent.
- Audit seal recovered from BLAKE3 chain contains cohort_id.

## Naming justifications
- BNF v4 action: `marketplace.deal-offer-create.execute` (gated on cohort_id presence)
- Layer enum: kernel + domain + adapter + policy + observability
- Crate name: `oya-marketplace-kernel-revenue-share-cohort-id`
