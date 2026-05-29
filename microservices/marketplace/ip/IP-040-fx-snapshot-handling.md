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
impl_plan_id: IP-040-fx-snapshot-handling
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-040: FX Snapshot Handling on Revenue-Share Accruals

## Intent
Snapshot FX rate at every multi-currency accrual so audit-chain evidence is reproducible. Closes audit gap §3.4.B.ii item 10.

## Boundary
- Owns: marketplace adapter to cloud-billing.fx.snapshot.get + the snapshot reference embedded in RevenueShareProvenance.
- Consumes: cloud-billing.fx.snapshot.get gRPC.
- Does not own: FX provider integration (cloud-billing handles).

## Deliverables
1. New adapter `marketplace.adapter.fx.snapshot.fetch` calls `cloud-billing.fx.snapshot.get(base_currency, quote_currency, instant)`.
2. Snapshot returned has: `(base, quote, rate_numerator, rate_denominator, source, valid_at, evidence_blake3)`.
3. Snapshot embedded in RevenueShareProvenance (IP-035) when listing currency ≠ tenant settlement currency.
4. Settlement ledger per-currency sub-balance: every accrual posts (DR receivable in listing currency) + (CR oyatie-revenue-share in oyatie-base USD) + FX-difference entry.
5. Replay fidelity: snapshot is the source of truth; FX rate changes after snapshot do not retroactively alter accruals.
6. SLO `marketplace.fx-snapshot-fetch-availability` target 0.9999 (FX must always be available at accrual time; degrade-mode is to fail-close not retro-rate).
7. Multi-currency cohort: a single revenue_share_cohort_id can mix currencies; cloud-billing rolls up at statement time using snapshot evidence.

## Acceptance criteria
- Adapter handles 6 named failure classes (network, provider-stale, no-quote, sanctions-block, currency-unknown, rate-suspended).
- Property test on rate arithmetic (1M random rates × random amounts).
- Replay 1-year-old accrual produces byte-identical oyatie-share amount.
- FX-fail-close test forces accrual deferral, not retroactive rate.

## Naming justifications
- BNF v4 action: `marketplace.fx-snapshot-fetch.execute`
- Layer enum: adapter + kernel + observability
- Crate name: `oya-marketplace-adapter-fx-snapshot`
