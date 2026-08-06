---
doc_status: published
---

# Oyatie Runbook — Marketplace Listing Takedown

> **Status:** Production procedure authored for the M03-P04/M03-P08 SaaS operator-documentation gate; readiness remains `target_non_claim` until changeset evidence and `oya-ci-required` are green.
> **Owner:** `axis-saas + ops-security + legal/compliance when regulatory or sanctions scope is present`
> **Severity scope:** Sev 3 by default; escalate to Sev 1 for active exploit, sanctions/export-control, regulated data exposure, or cross-tenant settlement impact.
> **Authority:** ADR-0249 marketplace doctrine, ADR-0314 DealSet settlement doctrine, ADR-0036 plugin trust model, the SaaS Platform PRD, and M03-P04/M03-P08 planning references in `specs/masterplan.json`.
> **Last verified:** 2026-06-09 (SSOT chain checked against HANDOFF.md, registry/stores/*, specs/root-hub-pointers.json, specs/masterplan.json, and docs/products/saas-platform/PRD.md).

## Operator contract
- **Incident channel:** `#inc-saas-marketplace`.
- **Primary invariant:** takedown changes listing discoverability and installability without silently deleting tenant entitlements, DealSet settlement evidence, or audit history.
- **Tenant boundary:** every action is scoped by `listing_id`, `publisher_tenant_id`, affected `buyer_tenant_id` set, `deal_set_id` where applicable, `plugin_id` where applicable, `cell_id`, and jurisdiction pack.
- **Cloud authority:** production listing/install state changes flow through marketplace/cloud control-plane APIs and sealed audit evidence. Workstation diagnostics are supporting evidence only.
- **Audit event:** every hide, freeze, entitlement hold, install revoke, DealSet state change, tenant notice, and restore decision emits `EVT-SAAS-MARKETPLACE-LISTING-TAKEDOWN-INCIDENT` with `incident_id`, `listing_id`, `publisher_tenant_id`, `tenant_id`, `deal_set_id`, `operator_id`, `decision_id`, and `evidence_hash`.
- **Stop condition:** the listing cannot be discovered or newly installed where prohibited, affected tenants have a migration/restore decision, settlement state is explicit, audit evidence is sealed, and prevention ownership is recorded.

## Trigger conditions
- Security advisory, sandbox escape, malicious artifact, vulnerable dependency, or plugin trust-tier downgrade.
- Legal/regulatory order, sanctions/export-control hit, copyright/IP claim, privacy complaint, or jurisdiction-pack violation.
- Fraud, review manipulation, install-count manipulation, payment abuse, or KYC/KYB failure.
- Vendor-initiated delisting, end-of-life, or license withdrawal.
- Marketplace listing metadata diverges from the artifact digest, capability set, trust tier, or DealSet entitlement terms.

## First-response checklist
1. Assign incident commander, marketplace owner, security/legal owner as needed, and tenant communications owner.
2. Record `INCIDENT_ID`, `LISTING_ID`, `PUBLISHER_TENANT_ID`, affected tenant set, `PLUGIN_ID`/artifact digest if present, `DEAL_SET_ID` set if settlement is involved, and jurisdiction packs.
3. Snapshot listing metadata, review state, artifact digest, trust tier, capability declarations, active install count, DealSet/entitlement state, payment/refund exposure, and audit-chain window.
4. Choose the narrowest containment state: hide, freeze new installs, hold entitlements, revoke installs, suspend publisher, or jurisdiction-specific block.
5. Emit containment audit evidence before sending tenant/vendor notifications.

## Containment
- **Hide listing:** remove listing from search/discovery and category pages; direct detail URL returns a takedown state with tenant-safe explanation.
- **Freeze new installs:** deny new install attempts with a stable denial code and audit event; keep existing installations unchanged unless risk requires revocation.
- **Hold entitlements:** pause entitlement expansion, renewals, or settlement actions while preserving current tenant evidence.
- **Revoke installs:** for security or policy breach, revoke affected plugin/app installations by tenant and trigger the plugin runtime runbook if executable code is involved.
- **Publisher suspension:** block publisher submissions and payouts only when evidence shows publisher-level risk; preserve payout and tax evidence for legal review.
- **Jurisdiction-specific block:** apply regional pack deny state when the issue is jurisdictional; avoid global takedown if evidence proves a local block is sufficient.

## Diagnosis
Classify exactly one primary branch before recovery:

| Branch | Evidence | Required check |
|---|---|---|
| Security vulnerability | advisory, exploit signal, sandbox/runtime incident | Verify artifact digest, affected versions, install set, and plugin runtime containment. |
| Policy/trust-tier violation | listing claims exceed approved trust tier or capabilities | Compare marketplace metadata, ADR-0036 trust tier, capability grant, and tenant consent. |
| Legal/regulatory/sanctions | formal notice, sanctions hit, jurisdiction-pack deny | Preserve notice, pack rule, tenant geography, publisher identity, and decision authority. |
| Fraud/review manipulation | anomalous reviews, installs, payments, or KYC/KYB | Check publisher identity, install telemetry, payment state, and audit evidence. |
| Settlement/entitlement defect | DealSet, subscription, refund, or license state inconsistent | Verify ADR-0314 DealSet status, entitlement ledger, payment/refund obligations, and tenant notices. |
| Vendor-requested delisting | publisher request or end-of-life notice | Confirm authority, tenant migration window, refund/support obligations, and restore conditions. |

## Recovery
1. Keep containment active until branch owner signs off and affected tenant/publisher impact is known.
2. Update listing state to the final explicit outcome: `removed_security`, `removed_policy`, `removed_legal`, `jurisdiction_blocked`, `vendor_delisted`, or `restored`.
3. For plugin listings, align marketplace state with plugin runtime install state and artifact digest allow/deny state.
4. For DealSet-backed purchases, transition affected DealSets to the correct explicit state: entitlement hold, entitlement revoke, refund pending, settlement deferred, dispute open, or closed.
5. Send tenant notifications with impact, workaround/migration path, support owner, and evidence retention statement.
6. Send publisher notice with appeal/remediation route unless prohibited by security/legal hold.
7. If restored, release by tenant cohort or jurisdiction after discovery, install, entitlement, settlement, and audit states agree.
8. Add the missing prevention: listing metadata validator, trust-tier/capability drift check, sanctions/regulatory pack gate, fraud detection, or DealSet entitlement consistency gate.

## Verify recovery
- Listing search, category, direct-detail, and install APIs all return the expected final state for affected and unaffected jurisdictions.
- New install attempts for removed/frozen listings are denied with stable audit evidence.
- Existing tenant installations are either safe, revoked, or explicitly under entitlement hold; no tenant is left in an implicit state.
- DealSet/entitlement/payment/refund states match the takedown branch.
- Audit-chain contains sealed containment and resolution events.
- Tenant and publisher communication records are attached to the incident evidence bundle.
- The prevention gate that would have caught the issue has an owner and due date.

## Rollback guardrails
- Do not hard-delete listings, reviews, entitlements, DealSets, or audit rows.
- Do not restore discoverability before installability, entitlement, settlement, and audit states are consistent.
- Do not use global takedown for a jurisdiction-specific issue unless regional pack evidence cannot isolate the risk.
- Do not keep accepting payments or renewals while a listing is frozen for security, legal, or settlement reasons.
- Do not close a takedown incident with only marketplace UI evidence; API, entitlement, settlement, and audit states must be checked.

## Post-incident
- Author postmortem within the SLA from `docs/INCIDENT-MANAGEMENT.md` for the selected severity.
- Add a prevention row to `docs/MISTAKES-LEDGER.md` or the active prevention ledger with the mechanical gate owner.
- Update marketplace publisher guidance if listing requirements, trust tiers, or DealSet obligations were ambiguous.
- Update this runbook if the classified branch, state name, metric, or prevention gate was missing.

## Sources
`docs/products/saas-platform/PRD.md`, `docs/teams/axis-saas/CHARTER.md`, `specs/masterplan.json` M03-P04/M03-P08 entries, `docs/decisions/ADR-0705-product-protocol-live-apex.md`, `docs/decisions/ADR-0705-product-protocol-live-apex.md`, `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`, `docs/INCIDENT-MANAGEMENT.md`, `docs/SLO-CATALOG.md`, `docs/standards/prevention-doctrine.md`.
