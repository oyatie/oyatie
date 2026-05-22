---
doc_class: ImplementationPlan
ip_id: IP-028-continuous-feedback-ingestion
microservice: performance-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-PM-28-continuous-feedback-evidence
status: proposed
date: 2026-05-20
owner: axis-performance-management
tenant_class: ["demo_trial", "paid"]
---

# IP-028: Continuous Feedback Ingestion

## Context

This net-new slice ingests peer, manager, and project feedback as review evidence without turning chat or docs into performance systems. It displaces Lattice feedback, 15Five check-ins, Workday Talent feedback, and Culture Amp comments by requiring explicit worker relation, evidence class, and retention policy.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_feedback_thread` | `thread_id` | `uuid primary key` | Feedback thread id. |
| `performance_feedback_thread` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_feedback_thread` | `subject_worker_ref` | `text not null` | Feedback subject. |
| `performance_feedback_thread` | `author_worker_ref` | `text not null` | Feedback author. |
| `performance_feedback_thread` | `feedback_kind` | `text not null` | `peer`, `manager`, `self`, `project`. |
| `performance_feedback_thread` | `evidence_refs` | `text[] not null` | Links to docs/tasks/messages. |
| `performance_feedback_thread` | `retention_policy_id` | `text not null` | Labor-pack retention. |

## API Endpoints

REST `POST /v1/performance-management/feedback-threads`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "subject_worker_ref": "hris:worker:778",
  "author_worker_ref": "hris:worker:102",
  "feedback_kind": "manager",
  "body": "Delivered the migration rehearsal with clear rollback evidence.",
  "evidence_refs": ["task:proj:alpha:883", "doc:launch-readiness:44"]
}
```

gRPC `FeedbackIngestionService.CreateThread(CreateFeedbackThreadRequest)` returns `thread_id` and retention evidence.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"manager"` | `performanceManagement::CreateFeedback` | `FeedbackThread::*` | `subject_worker_ref`, `author_worker_ref`, `feedback_kind` |
| `User::"employee"` | `performanceManagement::ReadFeedback` | `FeedbackThread::*` | `worker_relation`, `cycle_id`, `retention_policy_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Feedback | `FeedbackThread` | note id maps to thread id. |
| 15Five Check-in | `FeedbackThread` | check-in answer maps to feedback message. |
| Workday Talent Feedback | `FeedbackThread` | feedback event maps to thread. |
| Culture Amp Comment | `FeedbackThread` | comment imports as aggregate-only unless author visibility allowed. |

## Workflow Steps

1. `ResolveWorkerRefs` maps author and subject through HRIS.
2. `EvaluateRelationPermit` checks Cedar.
3. `ClassifyEvidenceRefs` validates linked docs/tasks/messages.
4. `PersistFeedbackThread` writes thread and retention policy.
5. `SealFeedbackCreated` emits audit evidence.

Branches: author equals subject allowed only for self feedback; missing evidence refs allowed but flagged low-confidence; labor pack may require employee visibility delay.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-FEEDBACK-CREATED` | `tenant_id`, `thread_id`, `subject_worker_ref_hash`, `feedback_kind` |
| `EVT-PERFORMANCE-FEEDBACK-READ` | `thread_id`, `principal`, `worker_relation`, `reason_code` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Create feedback | 45 ms | 200 ms | 450 ms | 800 rps/cell | 99.95% |
| Read feedback thread | 25 ms | 120 ms | 250 ms | 2k rps/cell | 99.95% |

## Failure Modes + Recovery

- HRIS worker unresolved: reject write and return mapping task.
- Evidence ref access denied: store feedback without ref and mark evidence missing.
- Retention policy unavailable: fail closed for write.

## Migration Notes

Vendor check-ins and feedback often mix private notes and review evidence. Migration imports visibility metadata and defaults uncertain comments to private historical evidence until HR approves.

## Cross-µservice Handoffs

- `hris` resolves worker refs.
- `docs`, `tasks`, and `messenger` provide evidence refs.
- `policy-engine` gates read/write.
- `audit-chain` seals feedback events.
- `review-cycle` workflows consume approved feedback evidence.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-028-continuous-feedback-ingestion.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-028-continuous-feedback-ingestion.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
