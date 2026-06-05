---
doc_class: ImplementationPlan
ip_id: IP-017-cost-budget-enforcer
microservice: marketing-automation
bounded_contexts: [segment, journey, attribution, deliverability, webhook-subscription, marketplace-audience-license]
related_adrs: [ADR-0244, ADR-0263, ADR-0314, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + finops
tenant_class_aware: true
---

# IP-017: Cost Budget Enforcer

## A. Problem

Marketing workloads can create unbounded cost through segment rebuilds, attribution reruns, webhook retries, warmup probes, landing-page views, and email sends. The stamped IP did not identify meters or enforcement points. The real gap is a budget enforcer tied to `manifest.json` paid billing components and demo_trial caps so evaluation tenants cannot accidentally create enterprise-scale spend.

## B. Approach

Use `cost-budget.md`, `manifest.json` `paid_billing_components.per_usage_meter_classes`, and `dashboards/tenant-cost-and-capacity.json` as the budget authority. Enforce budgets at command admission before expensive work and emit FinOps dimensions after accepted work. This IP does not implement billing; it prevents unmetered Marketing Automation work.

## C. Deliverables

| Artifact | Change |
|---|---|
| `cost-budget.md` | Replace tier-shaped assumptions with tenant_class x deployment_context meters. |
| `manifest.json` | Keep meter classes: email sends, attribution runs, segment materializations, journey executions, form submissions, webhook deliveries, deliverability decisions, frequency reservations, landing page views, A/B tests. |
| `src/usecase/mod.rs` | Add budget admission before audit/event append for expensive commands. |
| `dashboards/tenant-cost-and-capacity.json` | Show budget burn by tenant, meter class, and deployment context. |
| `runbooks/campaign-cost-spike.md` | Add rollback and pause path for runaway campaign spend. |

## D. Implementation

1. Define budget check inputs: tenant id, tenant class, meter class, estimated units, campaign id, journey id, and DealSet id where applicable.
2. Deny demo_trial over cap with a structured upgrade path; do not silently queue work.
3. For paid tenants, reserve budget before segment materialization, attribution rollup, and bulk webhook delivery.
4. Emit accepted and denied budget audit events with meter class and estimated units.
5. Add FinOps handoff to `cloud-billing` for per_usage and marketplace revenue_share cases.
6. Add dashboard alerts for sudden spend slope, cost per conversion, and attribution rerun storms.
7. Add test fixtures for demo_trial cap hit and paid tenant accepted reservation.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app budget`
- `buck2 build //:quality-lane-registry-authority-check # lane=cost-budget --microservice marketing-automation`
- Manual evidence: every paid per_usage meter in `manifest.json` has an admission or emission point.

## F. Evidence

- Local docs: `cost-budget.md`, `capacity-model.md`.
- Local manifest: `manifest.json` billing component and demo_trial cap blocks.
- Local dashboard: `dashboards/tenant-cost-and-capacity.json`.
- Local runbook: `runbooks/campaign-cost-spike.md`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Edition-like commercial constraints become explicit tenant_class and meter controls. |
| Adobe Marketo Engage | High-volume campaign and activity processing gets budget admission. |
| Mailchimp | Email send and audience operations are capped before spend spikes. |

## H. Local Traceability

- Budget doc: `cost-budget.md`.
- Capacity doc: `capacity-model.md`.
- Manifest source: `paid_billing_components`.
- Manifest source: `demo_trial_caps`.
- Dashboard: `dashboards/tenant-cost-and-capacity.json`.
- Runbook: `campaign-cost-spike.md`.
- Meter: `email_sends`.
- Meter: `attribution_runs`.
- Meter: `segment_materializations`.
- Meter: `journey_executions`.
- Meter: `form_submissions`.
- Meter: `webhook_deliveries`.
- Meter: `deliverability_admit_decisions`.
- Meter: `frequency_reservations`.
- Meter: `landing_page_views`.
- Meter: `ab_test_runs`.
- Failure state: expensive command accepted without reservation.
- Failure state: demo_trial over cap queues paid-scale work.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-017-cost-budget-enforcer.md` matched [`cost`, `attribution`, `finops`, `per_usage`, `metered`, `emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-017-cost-budget-enforcer.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
