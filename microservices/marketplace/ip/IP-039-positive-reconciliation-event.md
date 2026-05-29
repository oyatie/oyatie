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
impl_plan_id: IP-039-positive-reconciliation-event
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-039: Positive Reconciliation Evidence Event

## Intent
Author the `MarketplaceRevenueShareReconciliationSealed` positive-evidence event so successful reconciliation is provable (today only the failure event exists per ADR-MKT-001). Closes audit gap §3.4.B.ii item 9.

## Boundary
- Owns: `marketplace.revenue_share.reconcile` worker + the SUCCESS event (the FAILED event already exists).
- Consumes: payments.outbound + cloud-billing.invoice for reconciliation pairs.
- Does not own: payments or cloud-billing internal state.

## Deliverables
1. New AsyncAPI channel `MarketplaceRevenueShareReconciliationSealed` (success counterpart to the existing `…ReconciliationFailed`).
2. Worker `marketplace.revenue_share.reconcile`:
   - Pairs marketplace settlement statements ↔ payments outbound payouts ↔ cloud-billing invoice lines
   - Emits `…Sealed` when all three sides match within rounding tolerance
   - Emits `…Failed` (existing) when mismatch detected
3. Reconciliation cadence: daily; full month closes T+5 business days.
4. Cedar gate `marketplace.revenue-share-reconcile.execute` requires `tenant_class==paid` AND `billing_components has "revenue_share"`.
5. SLO `marketplace.revenue-share-reconciliation-success-rate` target 0.99 monthly.
6. Audit-chain seal for every success event; ledger position recorded.

## Acceptance criteria
- Three-way match property test (10k random matched + 1k mismatched).
- Tolerance rule documented (1 minor unit per currency) and enforced.
- Success event replayable; same audit_chain_seal_id.
- Runbook `runbooks/revenue-share-reconciliation-mismatch.md` linked.

## Naming justifications
- BNF v4 action: `marketplace.revenue-share-reconcile.execute`
- Layer enum: worker + observability + api + policy
- Crate name: `oya-marketplace-worker-revenue-share-reconcile`
