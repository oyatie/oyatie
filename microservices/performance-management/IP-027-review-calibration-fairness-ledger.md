---
doc_class: ImplementationPlan
ip_id: IP-027-review-calibration-fairness-ledger
microservice: performance-management
related_adrs: [ADR-0243, ADR-0263, ADR-0321]
journey_id: J-PM-27-calibration-fairness-review
status: proposed
date: 2026-05-20
owner: axis-performance-management
tenant_class: ["demo_trial", "paid"]
---

# IP-027: Review Calibration Fairness Ledger

## Context

This net-new slice records calibration movement and fairness checks before performance outcomes feed compensation or promotion decisions. It displaces Lattice calibration, Workday Talent calibration, 15Five review calibration, and Culture Amp fairness analytics as standalone vendor truth.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_calibration_ledger` | `ledger_id` | `uuid primary key` | Immutable calibration event. |
| `performance_calibration_ledger` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_calibration_ledger` | `cohort_id` | `uuid not null` | Calibration cohort. |
| `performance_calibration_ledger` | `worker_ref` | `text not null` | Hashed worker ref. |
| `performance_calibration_ledger` | `before_rating` | `text not null` | Input rating. |
| `performance_calibration_ledger` | `after_rating` | `text not null` | Output rating. |
| `performance_calibration_ledger` | `fairness_check_ref` | `text not null` | Check result id. |
| `performance_calibration_ledger` | `reason_code` | `text not null` | Required movement reason. |

## API Endpoints

REST `POST /v1/performance-management/calibrations/{cohort_id}:record-adjustment`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "worker_ref": "hris:worker:778",
  "before_rating": "exceeds",
  "after_rating": "meets",
  "reason_code": "evidence_recalibrated",
  "fairness_check_ref": "fairness:check:2026h1:44"
}
```

gRPC `CalibrationFairnessService.RecordAdjustment(RecordAdjustmentRequest)` returns `ledger_id` and cohort distribution summary.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"calibration.panelist"` | `performanceManagement::RecordCalibrationAdjustment` | `CalibrationCohort::*` | `cohort_id`, `reason_code`, `labor_pack_id` |
| `User::"auditor"` | `performanceManagement::ReadCalibrationLedger` | `CalibrationLedger::*` | `ticket_id`, `aggregate_only` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Calibration Change | `CalibrationAdjustment` | before/after rating maps to ledger row. |
| Workday Talent Calibration | `CalibrationAdjustment` | talent review movement maps to reason-coded row. |
| 15Five Calibration | `CalibrationAdjustment` | rating changes map to ledger. |
| Culture Amp Fairness Report | `FairnessCheckResult` | aggregate fairness outputs map to check ref. |

## Workflow Steps

1. `LoadCohort` verifies cohort is open and above anonymity floor.
2. `EvaluatePanelPermit` checks Cedar and labor pack.
3. `RunFairnessCheck` calculates distribution and protected-class constraints where lawful.
4. `AppendLedgerRow` writes immutable adjustment.
5. `SealAdjustment` emits audit evidence.

Branches: cohort below floor denies adjustment; missing reason denies; fairness check red result requires HRBP approval.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-CALIBRATION-ADJUSTED` | `tenant_id`, `cohort_id`, `worker_ref_hash`, `before_rating`, `after_rating` |
| `EVT-PERFORMANCE-CALIBRATION-DENIED` | `cohort_id`, `deny_reason`, `labor_pack_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Record adjustment | 60 ms | 250 ms | 550 ms | 300 rps/cell | 99.95% |
| Fairness check 1k cohort | 300 ms | 2 s | 5 s | 100 checks/hour/cell | 99.9% |

## Failure Modes + Recovery

- Fairness check worker timeout: leave adjustment pending and retry before publish.
- Cohort membership changed mid-run: version cohort and rerun distribution.
- Labor pack denies individual field: store aggregate-only evidence.

## Migration Notes

Vendor calibration history may not include reason codes. Migration imports rating movement as historical evidence and requires reason codes for new adjustments.

## Cross-µservice Handoffs

- `hris` supplies cohort and worker refs.
- `policy-engine` gates panel actions.
- `analytics` computes distribution checks.
- `audit-chain` seals ledger rows.
- `compensation` consumes only approved readiness handoff from IP-030.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-027-review-calibration-fairness-ledger.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-027-review-calibration-fairness-ledger.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
