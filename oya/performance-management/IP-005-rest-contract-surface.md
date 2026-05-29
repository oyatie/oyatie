---
doc_class: ImplementationPlan
ip_id: IP-005-rest-contract-surface
microservice: performance-management
related_adrs: [ADR-0253, ADR-0258, ADR-0263, ADR-0321]
journey_id: J-PM-05-review-api-contract
status: proposed
date: 2026-05-20
owner: axis-performance-management
tenant_class: ["demo_trial", "paid"]
---

# IP-005: Performance Management REST Contract Surface

## Context

This slice defines the first REST surface for goals, reviews, feedback, and calibration. It displaces Lattice APIs, Culture Amp APIs, Workday Talent APIs, and 15Five APIs with tenant-scoped routes that expose idempotency, labor-pack context, Cedar decision id, and audit event id.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_api_idempotency` | `idempotency_key` | `text primary key` | Caller supplied key. |
| `performance_api_idempotency` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_api_idempotency` | `route_id` | `text not null` | Contract route id. |
| `performance_api_idempotency` | `request_hash` | `bytea not null` | Replay mismatch guard. |
| `performance_api_idempotency` | `response_body` | `jsonb` | Stored response. |
| `performance_api_idempotency` | `expires_at` | `timestamptz not null` | 24h for writes, 30d for review submissions. |

## API Endpoints

```http
POST /v1/performance-management/goals
POST /v1/performance-management/review-cycles
POST /v1/performance-management/reviews/{review_id}:submit
POST /v1/performance-management/calibrations/{cohort_id}:run
GET  /v1/performance-management/engagement-surveys/{survey_id}/summary
```

Example review submit:

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "review_id": "rev_2026_h1_884",
  "worker_ref": "hris:worker:778",
  "manager_ref": "hris:worker:102",
  "summary_rating": "meets",
  "evidence_refs": ["feedback:thread:44", "goal:obj:91"]
}
```

gRPC `PerformanceRestBridge.SubmitReview(SubmitReviewRequest)` keeps REST and worker schema parity.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"employee"` | `performanceManagement::CreateGoal` | `PerformanceGoal::*` | `tenant_id`, `worker_ref`, `goal_visibility` |
| `User::"manager"` | `performanceManagement::SubmitReview` | `ReviewRecord::*` | `manager_chain_ref`, `cycle_state` |
| `User::"hrbp"` | `performanceManagement::RunCalibration` | `CalibrationCohort::*` | `cohort_size`, `labor_pack_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Goal/Review | `PerformanceGoal` / `ReviewRecord` | ids map to source refs; scoring rubric maps to review schema. |
| Culture Amp Survey | `EngagementSurveySummary` | responses aggregate before exposure. |
| Workday Talent Review | `ReviewRecord` | worker ids map through HRIS refs. |
| 15Five Check-in/Review | `FeedbackThread` / `ReviewRecord` | check-ins become feedback evidence. |

## Workflow Steps

1. `AuthenticateGatewayPrincipal` validates HTTP/3 request context.
2. `ValidateContractVersion` rejects unsupported media types.
3. `CheckIdempotency` prevents duplicate review submissions.
4. `EvaluateCedar` authorizes action.
5. `DispatchCommand` writes command or queues async calibration.
6. `ReturnEvidence` includes audit and policy ids.

Branches: duplicate request hash returns stored response; cohort below anonymity floor returns `422`; review cycle closed returns `409`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-API-WRITE-ACCEPTED` | `tenant_id`, `route_id`, `idempotency_key`, `audit_event_id` |
| `EVT-PERFORMANCE-REVIEW-SUBMITTED` | `review_id`, `worker_ref_hash`, `manager_ref_hash`, `cycle_id` |
| `EVT-ERROR-PERFORMANCE-API` | `route_id`, `status_code`, `recovery_branch` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Review submit | 75 ms | 300 ms | 700 ms | 500 rps/cell | 99.95% |
| Calibration run accepted | 100 ms | 600 ms | 1.2 s | 100 rps/cell | 99.9% |

## Failure Modes + Recovery

- Duplicate review submission: return stored response and no second mutation.
- Manager relation stale: refresh HRIS relation and retry once.
- Calibration async worker timeout: keep job pending and expose progress.

## Migration Notes

Vendor APIs expose review and survey data at different granularity. Oyatie routes separate goal, review, survey summary, feedback, and calibration operations to keep permissions and audit evidence specific.

## Cross-µservice Handoffs

- `api-gateway` terminates HTTP/3.
- `hris` supplies workers and manager chains.
- `workflow-engine` receives review-cycle commands.
- `audit-chain` stores route-level evidence.
- `analytics` consumes aggregate survey summaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-005-rest-contract-surface.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-005-rest-contract-surface.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
