---
doc_class: ImplementationPlan
ip_id: IP-029-engagement-pulse-anonymity-guard
microservice: performance-management
related_adrs: [ADR-0243, ADR-0263, ADR-0321]
journey_id: J-PM-29-anonymous-engagement-pulse
status: proposed
date: 2026-05-20
owner: axis-performance-management
capability_tier: T3
---

# IP-029: Engagement Pulse Anonymity Guard

## Context

This net-new slice protects engagement survey anonymity before results appear in dashboards or performance reviews. It displaces Culture Amp anonymity thresholds, Lattice pulse surveys, Workday Talent surveys, and 15Five engagement checks with explicit cohort floors and audit evidence.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_engagement_pulse` | `pulse_id` | `uuid primary key` | Survey/pulse id. |
| `performance_engagement_pulse` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_engagement_pulse` | `cohort_ref` | `text not null` | HRIS cohort ref. |
| `performance_engagement_pulse` | `minimum_cohort_size` | `integer not null` | Default 8; pack override possible. |
| `performance_engagement_pulse` | `response_count` | `integer not null default 0` | Count at close. |
| `performance_engagement_pulse` | `release_state` | `text not null` | `collecting`, `held`, `released_aggregate`, `blocked`. |

## API Endpoints

REST `POST /v1/performance-management/engagement-pulses/{pulse_id}:release-summary`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "cohort_ref": "hris:cohort:engineering-emea",
  "minimum_cohort_size": 8,
  "requested_dimensions": ["team", "level"]
}
```

gRPC `EngagementPulseService.ReleaseSummary(ReleaseEngagementSummaryRequest)` returns `release_state`, `suppressed_dimensions[]`, and audit id.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"hrbp"` | `performanceManagement::ReleaseEngagementSummary` | `EngagementPulse::*` | `cohort_ref`, `minimum_cohort_size`, `requested_dimensions` |
| `User::"auditor"` | `performanceManagement::ReadEngagementEvidence` | `EngagementPulse::*` | `aggregate_only=true`, `ticket_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Culture Amp Survey | `EngagementPulse` | survey id and reporting group map to pulse/cohort. |
| Lattice Pulse | `EngagementPulse` | pulse id maps to source ref. |
| Workday Talent Survey | `EngagementPulse` | survey event maps to pulse. |
| 15Five Engagement Check | `EngagementPulse` | check-in sentiment maps to aggregate signal. |

## Workflow Steps

1. `LoadPulse` reads response count and dimensions.
2. `CalculateAnonymityFloor` evaluates cohort and requested dimensions.
3. `SuppressUnsafeDimensions` removes dimensions below floor.
4. `ReleaseAggregateSummary` writes released aggregate.
5. `SealReleaseEvidence` emits audit event.

Branches: response count below floor holds release; dimension below floor is suppressed; row-level export always denied.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-ENGAGEMENT-SUMMARY-RELEASED` | `tenant_id`, `pulse_id`, `cohort_ref`, `response_count`, `suppressed_dimensions` |
| `EVT-PERFORMANCE-ENGAGEMENT-RELEASE-HELD` | `pulse_id`, `minimum_cohort_size`, `response_count` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Release summary | 120 ms | 750 ms | 1.5 s | 300 releases/hour/cell | 99.9% |
| Read released aggregate | 35 ms | 160 ms | 320 ms | 1k rps/cell | 99.95% |

## Failure Modes + Recovery

- Cohort size below threshold: hold release and schedule reminder.
- Requested dimensions identify individuals: suppress dimension and emit warning.
- HRIS cohort drift: recompute floor before release.

## Migration Notes

Vendor anonymity settings differ. Migration imports raw response counts and dimensions but recalculates release eligibility under Oyatie labor-pack rules.

## Cross-µservice Handoffs

- `hris` supplies cohorts.
- `analytics` computes aggregate metrics.
- `policy-engine` gates release and read actions.
- `audit-chain` seals release/hold events.
- `privacy` consumes aggregate-only evidence for employee requests.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-029-engagement-pulse-anonymity-guard.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-029-engagement-pulse-anonymity-guard.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
