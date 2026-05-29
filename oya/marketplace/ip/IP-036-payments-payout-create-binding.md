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
impl_plan_id: IP-036-payments-payout-create-binding
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-036: payments.payout.create Binding

## Intent
Replace the abstract "accrue" in IP-007 with an explicit `payments.payout.create` invocation contract so seller payouts actually happen. Closes audit gap §3.4.B.ii item 6.

## Boundary
- Owns: marketplace.payout.dispatch usecase + OpenAPI surface returning payout reference.
- Consumes: payments.payout.create gRPC; identity.kyc.artifact.get for KYC check.
- Does not own: payments rail (Stripe/Adyen/bank-rail) integration internals.

## Deliverables
1. New usecase `marketplace.payout.dispatch` triggered post-settlement-statement.
2. gRPC call contract:
   ```
   payments.payout.create({
     seller_tenant_id, currency, amount_minor_units,
     kyc_artifact_ref, source_settlement_statement_id,
     idempotency_key  // (tenant_id, statement_id)
   }) -> { payout_id, expected_clear_at }
   ```
3. OpenAPI surface `POST /marketplace/payouts/dispatch/{statement_id}` returns `{ payout_id, expected_clear_at, audit_chain_seal_id }`.
4. Cedar gate `marketplace.payout-dispatch.execute` requires `tenant_class==paid` AND `kyc_artifact_ref != ""` AND `payouts_enabled == true`.
5. Idempotency: 7-day key window on `(tenant_id, settlement_statement_id)`.
6. New AsyncAPI channel `MarketplaceSellerPayoutInitiated`.
7. Failure paths: KYC missing → 422 + `MarketplaceSellerPayoutBlockedKycMissing`; payment rail failure → exponential backoff + DLQ.

## Acceptance criteria
- Idempotency test passes (100 duplicates → 1 payout).
- Cedar deny verified for demo_trial + missing KYC.
- gRPC error mapping covers 6 named failure classes.
- Audit-chain seal present on every payout dispatch.

## Naming justifications
- BNF v4 action: `marketplace.payout-dispatch.execute`
- Layer enum: usecase + api + adapter + policy
- Crate name: `oya-marketplace-usecase-payout-dispatch`
