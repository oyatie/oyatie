---
doc_class: ImplementationPlan
ip_id: IP-003-ontology-projection
microservice: performance-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-PM-03-review-goal-import
status: proposed
date: 2026-05-20
owner: axis-performance-management
capability_tier: T2
---

# IP-003: Performance Management Ontology Projection

## Context

This slice maps performance vendor records into Oyatie primitives. Diana Alvarez needs a migration preview that turns Lattice goals and reviews, Culture Amp survey responses, Workday Talent goals/reviews, and 15Five check-ins into `PerformanceGoal`, `ReviewCycle`, `FeedbackThread`, `CalibrationCohort`, and `EngagementSurvey` objects.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_object_projection` | `projection_id` | `uuid primary key` | Source object projection. |
| `performance_object_projection` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_object_projection` | `source_vendor` | `text not null` | `lattice`, `culture_amp`, `workday_talent`, `fifteenfive`. |
| `performance_object_projection` | `source_object_type` | `text not null` | Goal, review, survey, check-in, calibration. |
| `performance_object_projection` | `oyatie_object_type` | `text not null` | Ontology target. |
| `performance_object_projection` | `field_delta` | `jsonb not null` | Mapping details. |
| `performance_object_projection` | `projection_hash` | `bytea not null` | Replay guard. |

## API Endpoints

REST `POST /v1/performance-management/projections/preview`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "source_vendor": "lattice",
  "source_object_type": "goal",
  "source_object_id": "goal_884",
  "target_object_type": "PerformanceGoal"
}
```

gRPC `PerformanceProjectionService.PreviewProjection(PreviewPerformanceProjectionRequest)` returns `field_delta`, `rejected_fields[]`, and `ontology_write_plan`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"migration.operator"` | `performanceManagement::PreviewProjection` | `VendorPerformanceObject::*` | `tenant_id`, `source_vendor`, `source_object_type` |
| `Service::"projection-worker"` | `ontology::WriteProjection` | `PerformanceObject::*` | `projection_hash`, `data_classes`, `labor_pack_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Goal | `PerformanceGoal` | goal id maps to source ref; parent goal maps to alignment edge. |
| Lattice Review | `ReviewCycle` | review packet maps to cycle and review records. |
| Culture Amp Survey | `EngagementSurvey` | survey id maps to survey; responses become aggregate-only signals until anonymity passes. |
| Workday Talent Goal/Review | `PerformanceGoal` / `ReviewCycle` | worker ids map to HRIS refs. |
| 15Five Check-in | `FeedbackThread` | check-in answers map to feedback messages. |

## Workflow Steps

1. `FetchSourceMetadata` reads schema and counts.
2. `MapWorkerRefs` resolves source workers through HRIS.
3. `ClassifyFields` tags review evidence, engagement signal, or goal record.
4. `PreviewOntologyWrite` validates node and edge types.
5. `CommitProjection` writes projection and ontology request.

Branches: unresolved worker ref blocks commit; survey response below anonymity floor stays aggregate-only; manager note without relation proof is rejected.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-PROJECTION-PREVIEWED` | `tenant_id`, `source_vendor`, `source_object_id`, `projection_hash` |
| `EVT-PERFORMANCE-PROJECTION-COMMITTED` | `projection_id`, `oyatie_object_type`, `ontology_write_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Projection preview | 150 ms | 800 ms | 1.8 s | 200 previews/min/cell | 99.9% |
| Projection commit | 80 ms | 350 ms | 800 ms | 100 commits/min/cell | 99.95% |

## Failure Modes + Recovery

- Worker ref unresolved: quarantine projection and request HRIS mapping.
- Survey anonymity floor violated: store aggregate-only and deny row-level projection.
- Ontology write rejected: keep pending projection and retry with same hash.

## Migration Notes

Lattice and Workday Talent focus on goals/reviews, Culture Amp on survey analytics, and 15Five on check-ins. Migration must map object intent, not product module names.

## Cross-µservice Handoffs

- `ontology` owns projected objects.
- `hris` resolves worker ids and manager chains.
- `data-boundary` labels employee-sensitive fields.
- `audit-chain` seals projection events.
- `analytics` consumes aggregate engagement projections only after anonymity checks.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-003-ontology-projection.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-003-ontology-projection.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
