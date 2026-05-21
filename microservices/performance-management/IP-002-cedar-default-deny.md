---
doc_class: ImplementationPlan
ip_id: IP-002-cedar-default-deny
microservice: performance-management
related_adrs: [ADR-0243, ADR-0244, ADR-0246, ADR-0263, ADR-0294, ADR-0321]
journey_id: J-PM-02-review-access-policy
status: proposed
date: 2026-05-20
owner: axis-performance-management
capability_tier: T2
---

# IP-002: Performance Management Cedar Default Deny

## Context

This slice prevents HR review access from inheriting broad vendor admin roles. Hana Mori is the named auditor: she must prove that Lattice admins, Culture Amp people scientists, Workday Talent administrators, and 15Five reviewers map to explicit Cedar actions before anyone can read review evidence, calibration notes, or engagement survey data.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_policy_binding` | `binding_id` | `uuid primary key` | Immutable policy binding. |
| `performance_policy_binding` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_policy_binding` | `scope_id` | `uuid not null` | FK to performance scope. |
| `performance_policy_binding` | `action_name` | `text not null` | Cedar action. |
| `performance_policy_binding` | `worker_relation` | `text not null` | `self`, `manager`, `hrbp`, `calibration_panel`, `auditor`. |
| `performance_policy_binding` | `sensitive_field_mode` | `text not null` | `none`, `summary`, `full`, `aggregate_only`. |
| `performance_policy_binding` | `policy_version` | `bigint not null` | Fragment version. |

## API Endpoints

REST `POST /v1/performance-management/policy/evaluate`

```json
{
  "principal": "User::manager.42",
  "action": "performanceManagement::ReadReviewEvidence",
  "resource": "ReviewCycle::cycle_2026_h1",
  "context": {
    "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
    "worker_relation": "manager",
    "sensitive_field_mode": "summary",
    "labor_pack_id": "EU-worker-council"
  }
}
```

gRPC `PerformancePolicyService.Evaluate(EvaluatePerformancePolicyRequest)` returns `decision`, `policy_version`, and `audit_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"employee"` | `performanceManagement::ReadOwnReview` | `ReviewRecord::*` | `tenant_id`, `worker_relation=self` |
| `User::"manager"` | `performanceManagement::SubmitReview` | `ReviewRecord::*` | `manager_chain_ref`, `cycle_state` |
| `User::"hrbp"` | `performanceManagement::RunCalibration` | `CalibrationCohort::*` | `labor_pack_id`, `cohort_size`, `sensitive_field_mode` |
| `User::"auditor"` | `performanceManagement::ReadAggregateEvidence` | `EngagementSurvey::*` | `ticket_id`, `aggregate_only=true` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Role | `PerformancePermitBinding` | role permissions decompose into review, goal, feedback, and calibration actions. |
| Culture Amp Role | `PerformancePermitBinding` | survey read roles map to aggregate-only or full modes. |
| Workday Talent Security Group | `PerformancePermitBinding` | security group maps to worker relation plus action. |
| 15Five Permission | `PerformancePermitBinding` | review and check-in permissions map to specific resources. |

## Workflow Steps

1. `LoadBinding` reads active policy binding.
2. `CompileWorkerRelation` resolves self, manager, HRBP, panel, or auditor relation.
3. `EvaluateDefaultDeny` uses policy-engine library first.
4. `ExplainDeny` names relation, pack, or field-level mismatch.
5. `SealDecision` writes audit event.

Branches: cohort smaller than anonymity floor denies aggregate read; manager-chain mismatch denies review; stale policy version denies mutation.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-POLICY-DECISION` | `tenant_id`, `principal`, `action`, `resource`, `decision`, `policy_version` |
| `EVT-PERFORMANCE-POLICY-DENIED` | `deny_reason`, `worker_relation`, `labor_pack_id`, `cedar_decision_id` |
| `EVT-CAPABILITY-INVOKED` | Emitted before calibration or evidence export capability executes. |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Policy evaluate | 8 ms | 35 ms | 80 ms | 8k eval/s/cell | 99.99% |
| Policy publish | 90 ms | 500 ms | 950 ms | 20 publishes/min/cell | 99.95% |

## Failure Modes + Recovery

- Policy-engine unavailable: fail closed for review/calibration reads and writes.
- Worker relation stale: re-query identity/HRIS and retry once.
- Imported vendor role too broad: keep in shadow binding until HR admin approves action mapping.

## Migration Notes

Vendor admin roles often allow broad employee visibility. Migration must map only explicit review, goal, feedback, survey, and calibration actions and must default engagement survey access to aggregate-only.

## Cross-µservice Handoffs

- `policy-engine` evaluates Cedar.
- `identity` resolves manager chain.
- `hris` resolves worker population and relation.
- `audit-chain` seals decisions.
- `privacy` consumes denial/export evidence for employee data requests.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-002-cedar-default-deny.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-002-cedar-default-deny.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
