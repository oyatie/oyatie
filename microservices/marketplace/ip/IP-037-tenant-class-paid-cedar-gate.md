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
impl_plan_id: IP-037-tenant-class-paid-cedar-gate
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
---

# IP-037: tenant_class == paid Cedar Gate Across Revenue-Share Path

## Intent
Add `tenant_class == "paid"` and `"revenue_share" in tenant.billing_components` gates to every Cedar policy on the revenue-share path. Closes audit gap §3.4.B.ii item 7 and §3.2.A.

## Boundary
- Owns: amendment of 6 existing Cedar policies + the new ones from IPs 031-040.
- Consumes: cloud-iam principal-claim service for tenant_class delivery.
- Does not own: cloud-iam's tenant_class persistence.

## Deliverables
1. Amend `policies/revenue-share-accrue.cedar`:
   ```cedar
   permit(...) when {
     ...existing...
     && context.tenant_class == "paid"
     && context.tenant_billing_components has "revenue_share"
   };
   forbid(...) when {
     context.tenant_class == "demo_trial"
   };
   ```
2. Amend `policies/escrow-reserve.cedar`: forbid `tenant_class == "demo_trial"` (no money movement).
3. Amend `policies/escrow-release.cedar`: same as escrow-reserve.
4. Amend `policies/deal-accept.cedar`: forbid demo_trial accepting paid listings.
5. Amend `policies/deal-offer-create.cedar`: permit demo_trial only for sandbox + free listings + categories {plugins, workflows}.
6. Amend `policies/mediation-open.cedar`: permit both classes (mediation available to demo_trial too).
7. Amend `policies/revenue-share-clawback.cedar` (IP-034): same gates as accrue.
8. Add Cedar entity schema declaring `tenant_class: String` and `tenant_billing_components: Set<String>` as principal claims.
9. Add tests in `tests/policy/*.rs`: positive (paid+revenue_share), negative (demo_trial), negative (paid without revenue_share component).

## Acceptance criteria
- All 8 amended policies compile.
- Test matrix runs: 6 policies × 2 classes × 3 billing-component states = 36 cases.
- No regression on existing same-tenant + cross-tenant grant tests.

## Naming justifications
- BNF v4 actions retained
- Layer enum: policy (primary) + observability
- Crate name: `oya-marketplace-policy-tenant-class-gates`
