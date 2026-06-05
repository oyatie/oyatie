---
doc_class: ImplementationPlan
ip_id: IP-014-marketplace-dealset-settlement
microservice: marketing-automation
bounded_contexts: [marketplace-audience-license, ad-network-seam, customer-analytics, attribution]
related_adrs: [ADR-0244, ADR-0263, ADR-0314, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + axis-marketplace + finops
tenant_class_aware: true
---

# IP-014: Marketplace DealSet Settlement

## A. Problem

Marketing Automation can license marketplace audience dealsets, but the stamped IP did not specify settlement mechanics. The service must prevent a tenant from syncing, scoring, or exporting audience segments unless the DealSet is valid, paid components are resolved, and audit proof ties every audience-use event to marketplace and FinOps.

## B. Approach

Use the `marketplace-audience-license` capability, `MarketingAutomationCommand::LicenseMarketplaceAudience`, and `MarketplaceAudienceLicenseHeld` audit event as the µservice-owned edge. Marketplace owns deal terms; cloud-billing/finops own metering; Marketing Automation owns hold/release of licensed audience segments before journey or ad-network use.

## C. Deliverables

| Artifact | Change |
|---|---|
| `capabilities/marketplace-audience-license.yaml` | Ensure capability lists HubSpot/Marketo/Mailchimp parity pressure and ADR-0314 settlement. |
| `src/domain/mod.rs` | Keep `Capability::MarketplaceAudienceLicense` and `MarketingAutomationCommand::LicenseMarketplaceAudience`. |
| `src/usecase/mod.rs` | Emit `MarketplaceAudienceLicenseHeld` with segment id and tenant id before audience export. |
| `cost-budget.md` | Add per-usage meters for licensed audience checks and attribution runs. |
| `runbooks/marketplace-dealset-hold.md` | Define failed settlement, expired DealSet, and vendor-dispute recovery. |

## D. Implementation

1. Add `deal_set_id`, `audience_license_id`, and `licensed_use` to request schemas where marketplace segments enter the service.
2. Validate DealSet state through marketplace; do not persist marketplace terms locally.
3. Hold segment activation if settlement is pending, expired, disputed, or tenant_class does not permit the license.
4. Emit audit-chain record before any ad-network seam, social seam, or journey launch consumes licensed audience membership.
5. Increment FinOps meter classes for `audience_license_checks`, `segment_materializations`, and `attribution_runs`.
6. Add replay behavior: if marketplace later voids a DealSet, mark downstream audience uses as disputed and stop new sends.
7. Document operator recovery in `runbooks/marketplace-dealset-hold.md`.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app marketplace`
- `buck2 build //:quality-lane-registry-authority-check # lane=capability-publish --microservice marketing-automation --capability marketplace-audience-license`
- Manual evidence: no audience export or journey launch path can reference licensed audience membership without DealSet proof.

## F. Evidence

- Local source: `src/domain/mod.rs` marketplace command and capability.
- Local source: `src/usecase/mod.rs` marketplace event mapping.
- Local capability: `capabilities/marketplace-audience-license.yaml`.
- Local runbook: `runbooks/marketplace-dealset-hold.md`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | App marketplace audience use receives explicit settlement holds. |
| Adobe Marketo Engage | Partner/list acquisition workflows are auditable before activation. |
| Mailchimp | Audience sync and lookalike usage are connected to payment and consent proof. |

## H. Local Traceability

- Capability: `capabilities/marketplace-audience-license.yaml`.
- Domain command: `LicenseMarketplaceAudience`.
- Domain capability: `MarketplaceAudienceLicense`.
- Audit event: `MarketplaceAudienceLicenseHeld`.
- Billing component: `revenue_share`.
- Billing component: `per_usage`.
- Meter: `audience_license_checks`.
- Meter: `segment_materializations`.
- Meter: `attribution_runs`.
- Request field: `deal_set_id`.
- Request field: `audience_license_id`.
- Request field: `licensed_use`.
- Handoff: marketplace owns DealSet terms.
- Handoff: cloud-billing owns invoice state.
- Handoff: finops owns meter aggregation.
- Runbook: `marketplace-dealset-hold.md`.
- Failure state: expired DealSet blocks new audience use.
- Failure state: disputed DealSet marks downstream uses disputed.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-014-marketplace-dealset-settlement.md` matched [`payment`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-014-marketplace-dealset-settlement.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-014-marketplace-dealset-settlement.md` matched [`attribution`, `finops`, `cost`, `per_usage`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-014-marketplace-dealset-settlement.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
