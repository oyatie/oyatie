---
doc_class: ImplementationPlan
ip_id: IP-022-chaos-drill-pack
microservice: marketing-automation
bounded_contexts: [journey, suppression, attribution, deliverability, webhook-subscription, marketplace-audience-license]
related_adrs: [ADR-0244, ADR-0248, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-sre-reliability
tenant_class_aware: true
---

# IP-022: Chaos Drill Pack

## A. Problem

Marketing Automation failure modes can harm tenants quickly: sends can go to suppressed subjects, journeys can backlog, attribution can drift, marketplace DealSets can hold campaigns, and webhook providers can degrade. The stamped IP gave no drill scenarios. The real gap is a drill pack that proves fail-closed behavior and recovery paths before production promotion.

## B. Approach

Turn existing `failure-modes.md`, `incident-response.md`, `multi-region.md`, and runbooks into explicit chaos drills. Drills inject failures into consent, suppression, webhook, deliverability, marketplace, and replay paths. Each drill has stop conditions and expected audit/SLO evidence.

## C. Deliverables

| Artifact | Change |
|---|---|
| `failure-modes.md` | Add drill ids for suppression miss, deliverability pause, webhook degradation, attribution lag, marketplace hold, and cell failover. |
| `incident-response.md` | Add drill execution and evidence collection protocol. |
| `runbooks/*.md` | Ensure named runbooks own each drill recovery. |
| `multi-region.md` | Add regional failover drill with consent and audit continuity gates. |
| `dashboards/operating-bar-overview.json` | Show drill status and last successful execution. |

## D. Implementation

1. Define drill MA-CHAOS-001: consent graph unavailable; expected result is fail-closed marketing send denial.
2. Define drill MA-CHAOS-002: webhook provider returns 5xx for 30 minutes; expected result is retries, no duplicate deliveries, and operator alert.
3. Define drill MA-CHAOS-003: deliverability DMARC failure; expected result is paused sends and audit event.
4. Define drill MA-CHAOS-004: attribution revenue event delay; expected result is stale report flag, not fabricated revenue attribution.
5. Define drill MA-CHAOS-005: marketplace DealSet disputed; expected result is audience license hold.
6. Define drill MA-CHAOS-006: home-cell failover; expected result is no cross-pack data movement and audit continuity.
7. Add evidence capture: SLO burn before/after, audit event ids, runbook owner, and rollback completion.

## E. Acceptance

- `cargo run -p oya-dev-cli -- gate validate chaos-drill-pack --microservice marketing-automation`
- Manual evidence: each P0 runbook has at least one drill id and expected audit event.
- Dry-run evidence can be accepted before runtime implementation, but production promotion requires observed drill runs.

## F. Evidence

- Local docs: `failure-modes.md`, `incident-response.md`, `multi-region.md`.
- Local runbooks: `local-consent-propagation-lag.md`, `local-webhook-provider-degradation.md`, `deliverability-drop.md`, `marketplace-dealset-hold.md`.
- Local dashboard: `dashboards/operating-bar-overview.json`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Enterprise marketing reliability expectations get explicit failure drills. |
| Adobe Marketo Engage | Campaign automation failures have rehearsed recovery paths. |
| Mailchimp | Audience, campaign, and webhook failures are tested before production claims. |

## H. Local Traceability

- Failure catalog: `failure-modes.md`.
- Incident doc: `incident-response.md`.
- Placement doc: `multi-region.md`.
- Dashboard: `dashboards/operating-bar-overview.json`.
- Drill id: MA-CHAOS-001 consent graph unavailable.
- Drill id: MA-CHAOS-002 webhook provider 5xx.
- Drill id: MA-CHAOS-003 deliverability DMARC failure.
- Drill id: MA-CHAOS-004 attribution revenue event delay.
- Drill id: MA-CHAOS-005 marketplace DealSet disputed.
- Drill id: MA-CHAOS-006 home-cell failover.
- Runbook: `local-consent-propagation-lag.md`.
- Runbook: `local-webhook-provider-degradation.md`.
- Runbook: `deliverability-drop.md`.
- Runbook: `marketplace-dealset-hold.md`.
- Evidence: audit event ids.
- Evidence: SLO burn before and after.
- Failure state: drill passes without rollback evidence.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-022-chaos-drill-pack.md` matched [`multi-region`, `SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-022-chaos-drill-pack.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-022-chaos-drill-pack.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-022-chaos-drill-pack.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
