---
doc_class: ImplementationPlan
ip_id: IP-018-capacity-admission-control
microservice: marketing-automation
bounded_contexts: [journey, segment, attribution, deliverability, webhook-subscription, frequency-cap]
related_adrs: [ADR-0244, ADR-0248, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + ops-sre-reliability
tenant_class_aware: true
---

# IP-018: Capacity Admission Control

## A. Problem

Marketing campaigns create bursty traffic: journey launches after segment refresh, webhook fanout, attribution rollups after revenue imports, and deliverability probes during warmup. The stamped IP did not name any capacity bottlenecks. The real gap is an admission controller that protects SLOs and tenant fairness before a campaign overwhelms workers or downstream mail/messenger systems.

## B. Approach

Use `capacity-model.md`, local HPA/PDB IaC, SLO files, and dashboards to define capacity tokens by work class. Admission runs before accepted commands and returns defer/deny decisions with retry hints. This is separate from cost budget: a tenant can have budget but still be deferred when the cell lacks safe capacity.

## C. Deliverables

| Artifact | Change |
|---|---|
| `capacity-model.md` | Define capacity tokens for segment build, journey step, attribution run, webhook delivery, deliverability decision, and frequency reservation. |
| `iac/local-hpa.yaml` | Scale on backlog and latency metrics for journey and webhook workers. |
| `iac/local-pdb.yaml` | Preserve admission and suppression availability during node drains. |
| `slos/local-journey-trigger-latency.openslo.yaml` | Bind admission decisions to journey trigger SLO burn. |
| `runbooks/journey-backlog-saturation.md` | Add admission-control mitigation steps. |

## D. Implementation

1. Define work classes and token costs from capacity model, not from generic CPU guesses.
2. Add admission check before `CampaignJourneyInteractor::handle()` reserves idempotency.
3. Return `defer_until` for temporary saturation and `capacity_denied` for tenant cap exhaustion.
4. Reserve capacity atomically for journey launch, segment materialization, attribution rollup, webhook fanout, and frequency reservation.
5. Release capacity on completion, cancellation, or replay failure with audit event.
6. Feed HPA metrics from backlog and p95 latency panels.
7. Test no-starvation behavior for paid tenants and hard caps for demo_trial tenants.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app capacity`
- `cargo run -p oya-dev-cli -- gate validate capacity-model --microservice marketing-automation`
- `kubectl apply --dry-run=server -f microservices/marketing-automation/iac/local-hpa.yaml`
- Manual evidence: every bursty work class has a token, SLO, and runbook recovery path.

## F. Evidence

- Local docs: `capacity-model.md`.
- Local IaC: `iac/local-hpa.yaml`, `iac/local-pdb.yaml`.
- Local SLOs: `slos/local-journey-trigger-latency.openslo.yaml`, `slos/local-send-latency.openslo.yaml`.
- Local runbook: `runbooks/journey-backlog-saturation.md`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Workflow and list bursts are explicitly admitted rather than hidden inside SaaS queues. |
| Adobe Marketo Engage | Smart Campaign throughput receives deterministic backpressure. |
| Mailchimp | Campaign fanout and webhook delivery are rate-shaped per tenant and cell. |

## H. Local Traceability

- Capacity doc: `capacity-model.md`.
- IaC file: `iac/local-hpa.yaml`.
- IaC file: `iac/local-pdb.yaml`.
- SLO: `slos/local-journey-trigger-latency.openslo.yaml`.
- SLO: `slos/local-send-latency.openslo.yaml`.
- Runbook: `runbooks/journey-backlog-saturation.md`.
- Work class: segment build.
- Work class: journey step.
- Work class: attribution run.
- Work class: webhook delivery.
- Work class: deliverability decision.
- Work class: frequency reservation.
- Response field: `defer_until`.
- Response field: `capacity_denied`.
- Fairness rule: paid tenants cannot starve each other.
- Demo rule: demo_trial caps are hard limits.
- Failure state: capacity released without completion or cancellation evidence.
- Failure state: downstream mail backlog ignored by admission.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-018-capacity-admission-control.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-018-capacity-admission-control.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-018-capacity-admission-control.md` matched [`attribution`, `cost`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-018-capacity-admission-control.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
