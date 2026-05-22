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
impl_plan_id: IP-033-monthly-settlement-statement-event
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-033: Monthly Settlement Statement Event

## Intent
Author the per-month (or per-contract-period) settlement statement event so cloud-billing has a sealed roll-up to invoice from. Closes audit gap §3.4.B.ii item 3.

## Boundary
- Owns: `MarketplaceRevenueShareSettlementStatementSealed` event + the marketplace.revenue_share.statement.emit worker.
- Consumes: SettlementLedger projection for cohort aggregation; audit-chain for seal id.
- Does not own: cloud-billing invoice generation (that happens after this event).

## Deliverables
1. New AsyncAPI channel `MarketplaceRevenueShareSettlementStatementSealed` carrying:
   - `tenant_id`, `revenue_share_cohort_id`, `period_start`, `period_end`
   - `gross_cohort_revenue_minor_units`, `currency`, `oyatie_share_basis_points`
   - `oyatie_share_amount_minor_units`, `net_to_tenant_amount_minor_units`
   - `accrual_count`, `audit_chain_seal_id`, `payment_intent_id` (nullable until payout dispatched)
   - `fx_snapshot_ref` (when multi-currency cohort; see IP-040)
2. Worker `marketplace.revenue_share.statement.emit` runs daily cron + outbox-pattern transactional emit.
3. Cedar gate `marketplace.revenue-share-statement-emit.execute` requires `tenant_class==paid` and `billing_components has "revenue_share"`.
4. SLO `marketplace.revenue-share-statement-emit-timeliness` target 99% within 24h of period_end.
5. Idempotency on `(tenant_id, period_start, period_end)`.

## Acceptance criteria
- AsyncAPI channel publishes against schema registry without breaking change.
- Per-cohort aggregation reconciles against SettlementLedger sum to 0 cents.
- Statement event replay produces same audit_chain_seal_id.
- Cron + outbox tested with downstream failure scenarios.

## Naming justifications
- BNF v4 action: `marketplace.revenue-share-statement-emit.execute`
- Layer enum: worker + observability + api + policy
- Crate name: `oya-marketplace-worker-revenue-share-statement-emit`
