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
impl_plan_id: IP-034-clawback-compensation-surface
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-034: Clawback / Chargeback Compensation Public Capability

## Intent
Promote `marketplace.settlement.compensation_posted.v1` from internal ADR-MKT-001 event to a first-class public capability with Cedar policy + OpenSLO. Closes audit gap §3.4.B.ii item 4.

## Boundary
- Owns: `marketplace.revenue-share-clawback.execute` capability + Cedar + SLO + worker.
- Consumes: payments.chargeback events (subscribe); audit-chain seal.
- Does not own: payments rail-level chargeback dispute resolution (PSP handles).

## Deliverables
1. New capability YAML `capabilities/revenue-share-clawback.yaml` (BNF: `marketplace.revenue-share-clawback.execute`).
2. New Cedar policy `policies/revenue-share-clawback.cedar`: permit only when `tenant_class==paid`, `chargeback_evidence_ref != ""`, deny on cross-tenant without grant.
3. New OpenSLO `slos/revenue-share-clawback-accuracy.openslo.yaml` 0.9999 / 30d rolling.
4. New worker `marketplace.revenue_share.clawback.process` consumes payments.chargeback events + posts compensating ledger entries:
   - DR oyatie-revenue-share, CR escrow OR seller-payable
   - Audit-chain seal on every compensation
5. New AsyncAPI channel `MarketplaceRevenueShareClawbackPosted`.
6. Runbook `runbooks/revenue-share-clawback-spike.md` for >5% chargeback-rate burn.

## Acceptance criteria
- Capability + Cedar + SLO triplet present per existing pattern.
- Compensation always nets to 0 against original accrual (verified via property test).
- Cedar deny verified for missing chargeback_evidence_ref.
- Audit-chain seal recoverable for every compensation event.

## Naming justifications
- BNF v4 action: `marketplace.revenue-share-clawback.execute`
- Layer enum: worker + policy + observability + api
- Crate name: `oya-marketplace-worker-revenue-share-clawback`
