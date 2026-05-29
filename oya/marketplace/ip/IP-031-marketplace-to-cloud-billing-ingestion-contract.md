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
impl_plan_id: IP-031-marketplace-to-cloud-billing-ingestion-contract
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-031: Marketplace → cloud-billing Ingestion Contract

## Intent
Bind marketplace revenue-share accrual emissions to the cloud-billing ingestion surface so monthly settlement statements roll up from per-DealSet cohorts. Closes audit gap §3.4.B.ii item 1.

## Boundary
- Owns: marketplace event emission contract, AsyncAPI subscribe-side declarations referenced by cloud-billing.
- Consumes: cloud-billing only via typed AsyncAPI; payments via payout reference; audit-chain via seal evidence.
- Does not own: cloud-billing's internal cohort tables.

## Deliverables
1. AsyncAPI subscribe-side declaration in cloud-billing's contract pointing at:
   - `marketplace.settlement.ledger_batch_posted.v1`
   - `marketplace.MarketplaceRevenueShareAccrued`
   - `marketplace.MarketplaceRevenueShareSettlementStatementSealed` (see IP-033)
2. Schema-registry compatibility constraints: `BACKWARD_TRANSITIVE` for marketplace publisher.
3. Idempotency: cloud-billing dedups on `(deal_set_id, accrual_sequence_no)`.
4. Cedar gate on subscribe side: `cloud-billing.marketplace-accrual.ingest` action requires `marketplace` SPIFFE workload identity.
5. Ordering contract: per-`revenue_share_cohort_id` FIFO; cross-cohort allowed parallel.
6. Backpressure: cloud-billing throttles via consumer-lag SLO; marketplace honors emit-suspend signal.

## Acceptance criteria
- AsyncAPI doc parses cleanly with `--strict`.
- Schema registry round-trip preserves all 30 fields.
- Idempotency test runs 1000 duplicate emissions and produces 1 accrual row.
- Cedar deny verified for non-marketplace SPIFFE identity attempting ingest.
- Per-cohort FIFO verified under chaos (event reordering).

## Naming justifications
- BNF v4 action: `marketplace.settlement.ledger_batch_posted.publish`
- Layer enum: events + api + observability layers consumed
- Crate name: `oya-marketplace-events-cloud-billing-bridge`
