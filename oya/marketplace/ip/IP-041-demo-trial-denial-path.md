---
doc_class: ImplementationPlan
microservice: marketplace
status: Accepted
date: 2026-05-21
owner_team: axis-marketplace
primary_adr: ADR-0331
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0249, ADR-0263, ADR-0314, ADR-0329, ADR-0330, ADR-0331]
companion_docs: [microservices/marketplace/REMEDIATION-NOTES-2026-05-21.md]
planned_enforcement_ref: oya-governance-marketplace-doc-set
naming_justifications: BNF v4 service_action_resource grammar and 13-layer-enum conformance are declared inline in this document
impl_plan_id: IP-041-demo-trial-denial-path
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-041: demo_trial Denial Path

## Intent
Make the demo_trial deny path explicit + observable across every revenue-share-touching surface so customers see consistent reason codes when they hit the paid-only wall, with an in-product conversion CTA. Closes audit gap §3.4.B.ii item 11.

## Boundary
- Owns: deny-reason taxonomy + observable deny-counter metric + the in-product conversion-CTA event.
- Consumes: Cedar deny decisions from IP-037 amendments.
- Does not own: pricing-page / checkout UI surfaces (consumed by tenant-portal product surface).

## Deliverables
1. New deny-reason enum:
   - `MKT_DENY_DEMO_TRIAL_NO_PAYOUTS`
   - `MKT_DENY_DEMO_TRIAL_NO_PAID_LISTING`
   - `MKT_DENY_DEMO_TRIAL_NO_ESCROW`
   - `MKT_DENY_DEMO_TRIAL_NO_REVENUE_SHARE`
   - `MKT_DENY_DEMO_TRIAL_AUTONOMY_CAPPED`     // agents category
   - `MKT_DENY_DEMO_TRIAL_MODEL_SIZE_CAPPED`   // models category
   - `MKT_DENY_DEMO_TRIAL_DATASET_SENSITIVITY_CAPPED`  // datasets category
2. Observable metric: `oya_marketplace_demo_trial_deny_total{reason="<enum>"}`.
3. AsyncAPI event `MarketplaceDemoTrialConversionCTAOpportunity` emitted per deny (with rate-limit per tenant 1/minute) so the tenant-portal surface can show a contextual upgrade CTA.
4. Audit-chain seal on every deny (deny is itself an audit-worthy event).
5. Per-deny-reason runbook: explains the customer-friendly message + recommended upgrade SKU.
6. Dashboard panel `demo-trial-conversion-funnel.json` shows deny-by-reason → CTA-opportunity → upgrade-completed funnel.

## Acceptance criteria
- All 7 deny reasons fire correctly in integration tests.
- Rate-limit on conversion-CTA event verified (no spam).
- Audit-chain seal recoverable for deny events.
- Dashboard panel renders with seeded test data.

## Naming justifications
- BNF v4 action: `marketplace.demo-trial-deny.observe`
- Layer enum: observability + worker + policy
- Crate name: `oya-marketplace-observability-demo-trial-deny`
