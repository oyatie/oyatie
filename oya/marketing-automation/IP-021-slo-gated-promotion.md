---
doc_class: ImplementationPlan
ip_id: IP-021-slo-gated-promotion
microservice: marketing-automation
bounded_contexts: [journey, segment, suppression, attribution, deliverability, webhook-subscription]
related_adrs: [ADR-0139, ADR-0244, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-sre-reliability
tenant_class_aware: true
---

# IP-021: SLO-Gated Promotion

## A. Problem

The service cannot move from docs/scaffold to production claims merely because IP files exist. Marketing Automation has SLO files for availability, read/write latency, replay freshness, audit emission lag, policy decision latency, and local differentiator flows. The stamped IP did not connect these SLOs to promotion gates. HubSpot, Marketo, and Mailchimp customers expect reliable sends, fast list updates, and report freshness; Oyatie must prove that with promotion evidence.

## B. Approach

Use service-local OpenSLO files, `dashboards/slo-and-error-budget.json`, `dashboards/local-slo-burn.json`, and `scorecards/overrides.json` to gate promotion. Each capability moves only when its SLO has observed data, error-budget policy, dashboard coverage, and runbook ownership.

## C. Deliverables

| Artifact | Change |
|---|---|
| `slos/*.openslo.yaml` | Ensure each critical path has objective, window, and burn policy. |
| `dashboards/slo-and-error-budget.json` | Add service-level promotion view. |
| `dashboards/local-slo-burn.json` | Add local differentiator views for segment, consent, deliverability, attribution, and send latency. |
| `scorecards/overrides.json` | Declare Marketing Automation-specific scorecard thresholds. |
| `PHASE-01-MARKETING-AUTOMATION-OPERATING-BAR.md` | Reference SLO-gated promotion as a phase exit condition. |

## D. Implementation

1. Inventory SLO files: availability, read/write latency, replay freshness, policy decision latency, audit emission lag, local send latency, local consent propagation, local attribution freshness, local suppression enforcement, local deliverability success, and journey trigger latency.
2. Map each SLO to a capability and runbook owner.
3. Require 7-day observed data before paid tenant promotion; demo_trial can use best-effort labeling but cannot be called production-grade.
4. Add release gate that blocks promotion when any P0 SLO is missing telemetry or has exhausted error budget.
5. Tie burn alerts to runbooks: deliverability drop, journey backlog saturation, suppression miss, attribution lag, webhook degradation.
6. Add dashboard panels by tenant_class and deployment_context.
7. Record promotion evidence in audit-chain or the repo's evidence bundle path once implementation lands.

## E. Acceptance

- `cargo run -p oya-dev-cli -- gate validate slo-gated-promotion --microservice marketing-automation`
- `cargo run -p oya-dev-cli -- gate validate slo-catalog --microservice marketing-automation`
- Manual evidence: each SLO has an owning runbook and dashboard panel.

## F. Evidence

- Local SLOs: `slos/*.openslo.yaml`.
- Local dashboards: `dashboards/slo-and-error-budget.json`, `dashboards/local-slo-burn.json`.
- Local scorecards: `scorecards/overrides.json`.
- Local runbooks: `runbooks/deliverability-drop.md`, `runbooks/journey-backlog-saturation.md`, `runbooks/suppression-list-drift.md`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Marketing send/list reliability claims become evidence-gated. |
| Adobe Marketo Engage | Smart Campaign and engagement program promotion requires observed SLOs. |
| Mailchimp | Campaign and audience automation readiness is tied to error budget, not document presence. |

## H. Local Traceability

- SLO glob: `slos/*.openslo.yaml`.
- Dashboard: `dashboards/slo-and-error-budget.json`.
- Dashboard: `dashboards/local-slo-burn.json`.
- Scorecard: `scorecards/overrides.json`.
- Phase doc: `PHASE-01-MARKETING-AUTOMATION-OPERATING-BAR.md`.
- SLO: availability.
- SLO: read latency.
- SLO: write latency.
- SLO: replay freshness.
- SLO: policy decision latency.
- SLO: audit emission lag.
- SLO: local send latency.
- SLO: local suppression enforcement.
- SLO: local deliverability success.
- Runbook: `deliverability-drop.md`.
- Runbook: `journey-backlog-saturation.md`.
- Failure state: production claim without observed data.
- Failure state: exhausted error budget still promotes.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-021-slo-gated-promotion.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-021-slo-gated-promotion.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-021-slo-gated-promotion.md` matched [`attribution`, `emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-021-slo-gated-promotion.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
