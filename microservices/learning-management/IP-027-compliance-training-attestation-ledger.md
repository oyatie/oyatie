---
doc_class: ImplementationPlan
ip_id: IP-027-compliance-training-attestation-ledger
microservice: learning-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-27-regulated-training-attestation
status: proposed
date: 2026-05-20
owner: axis-learning-management
capability_tier: T3
---

# IP-027: Compliance Training Attestation Ledger

## Context

This net-new slice records immutable training attestations for regulated courses, including learner acknowledgement, assessment score, proctoring evidence, and renewal deadline. It supports Marcus Chen during an audit that expects Cornerstone transcript rigor, Workday Learning compliance campaigns, Docebo certification records, 360Learning completion reviews, and LinkedIn Learning Enterprise content usage to resolve into one auditable ledger.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_training_attestation` | `attestation_id` | `uuid primary key` | Immutable attestation. |
| `learning_training_attestation` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_training_attestation` | `worker_ref` | `text not null` | Learner worker ref. |
| `learning_training_attestation` | `course_ref` | `text not null` | Regulated course. |
| `learning_training_attestation` | `completion_evidence_ref` | `text not null` | Sealed completion evidence. |
| `learning_training_attestation` | `attested_at` | `timestamptz not null` | Attestation timestamp. |
| `learning_training_attestation` | `expires_at` | `timestamptz` | Renewal deadline. |

## API Endpoints

REST `POST /v1/learning-management/training-attestations`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00027",
  "worker_ref": "hris:worker:614",
  "course_ref": "course:anti-bribery-2026",
  "completion_evidence_ref": "completion:sealed:ab-614",
  "attested_at": "2026-05-20T17:45:00Z",
  "expires_at": "2027-05-20T17:45:00Z"
}
```

gRPC `LearningAttestationLedgerService.Record(RecordTrainingAttestationRequest)` returns `attestation_id`, `ledger_sequence`, and `audit_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"learner"` | `learningManagement::AttestTraining` | `TrainingAttestation::*` | `tenant_id`, `worker_ref`, `course_ref` |
| `User::"compliance-admin"` | `learningManagement::ExportAttestationLedger` | `TrainingAttestation::*` | `tenant_id`, `purpose`, `regulation_ref` |
| `Service::"assessment"` | `learningManagement::SealCompletionEvidence` | `CompletionEvidence::*` | `score`, `proctored`, `course_ref` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Transcript Record | `TrainingAttestation` | completion date and score map to attestation evidence. |
| Workday Learning Compliance Campaign | `TrainingRequirement` | campaign requirement maps to regulated course requirement. |
| Docebo Certification | `CredentialAssertion` | certification expiry maps to `expires_at`. |
| 360Learning Review Completion | `CompletionEvidence` | reviewer approval maps to evidence detail. |
| LinkedIn Learning Enterprise Completion | `ProviderCompletionSignal` | provider completion maps to supplemental evidence only. |

## Workflow Steps

1. `ValidateCompletionEvidence` ensures evidence is sealed and belongs to worker.
2. `EvaluateAttestationPolicy` checks learner self-attest or compliance-admin override.
3. `WriteAttestationLedger` persists immutable record.
4. `ScheduleRenewal` creates renewal task if `expires_at` exists.
5. `SealAuditEvent` records ledger sequence and event class.

Branches: unsealed evidence rejects; failing score rejects attestation; provider-only completion requires internal compliance course mapping.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-TRAINING-ATTESTED` | `tenant_id`, `attestation_id`, `worker_ref`, `course_ref`, `expires_at` |
| `EVT-LEARNING-ATTESTATION-EXPORT` | `tenant_id`, `purpose`, `regulation_ref`, `row_count` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Record attestation | 45 ms | 200 ms | 450 ms | 1k writes/s/cell | 99.95% |
| Export ledger page | 80 ms | 420 ms | 1 s | 200 pages/s/cell | 99.9% |

## Failure Modes + Recovery

- Completion evidence missing: reject and request evidence replay.
- Ledger write conflict: retry with idempotency key and preserve sequence ordering.
- Renewal scheduler degraded: persist attestation and enqueue renewal backfill job.

## Migration Notes

Cornerstone and Docebo certification histories can become attestations only when worker, course, completion date, and expiry are present. LinkedIn Learning Enterprise completions need a mapped regulated course before attestation.

## Cross-µservice Handoffs

- `assessment` supplies sealed completion evidence.
- `credentialing` issues certification credentials.
- `compliance-governance` consumes exportable attestation ledgers.
- `notification` sends renewal reminders.
- `audit-chain` seals immutable attestation events.
