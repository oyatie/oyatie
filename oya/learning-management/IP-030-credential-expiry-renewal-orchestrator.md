---
doc_class: ImplementationPlan
ip_id: IP-030-credential-expiry-renewal-orchestrator
microservice: learning-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-30-credential-renewal-automation
status: proposed
date: 2026-05-20
owner: axis-learning-management
tenant_class: ["demo_trial", "paid"]
---

# IP-030: Credential Expiry Renewal Orchestrator

## Context

This net-new slice turns expiring credentials into governed renewal workflows with reminders, reassignment, reassessment, and reissuance. It supports Marcus Chen replacing Cornerstone certification renewals, Workday Learning recertification campaigns, Docebo certification expiry, 360Learning refresher paths, and LinkedIn Learning Enterprise completion refreshes with one auditable renewal clock.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_credential_renewal` | `renewal_id` | `uuid primary key` | One renewal workflow. |
| `learning_credential_renewal` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_credential_renewal` | `credential_ref` | `text not null` | Expiring credential. |
| `learning_credential_renewal` | `worker_ref` | `text not null` | Credential holder. |
| `learning_credential_renewal` | `renewal_course_ref` | `text not null` | Required refresher course. |
| `learning_credential_renewal` | `renewal_status` | `text not null` | scheduled, assigned, completed, expired, waived. |
| `learning_credential_renewal` | `deadline_at` | `timestamptz not null` | Renewal deadline. |

## API Endpoints

REST `POST /v1/learning-management/credential-renewals:schedule`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00030",
  "credential_ref": "credential:forklift-operator:worker-221",
  "worker_ref": "hris:worker:221",
  "renewal_course_ref": "course:forklift-refresher-2026",
  "deadline_at": "2026-08-31T23:59:59Z",
  "reminder_offsets_days": [60, 30, 7, 1]
}
```

gRPC `LearningCredentialRenewalService.Schedule(ScheduleCredentialRenewalRequest)` returns `renewal_id`, `assignment_ref`, and `next_reminder_at`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"credentialing"` | `learningManagement::ScheduleCredentialRenewal` | `CredentialRenewal::*` | `tenant_id`, `credential_ref`, `deadline_at` |
| `User::"compliance-admin"` | `learningManagement::WaiveCredentialRenewal` | `CredentialRenewal::*` | `reason_code`, `worker_ref`, `deadline_at` |
| `Service::"workflow-engine"` | `learningManagement::AdvanceCredentialRenewal` | `CredentialRenewal::*` | `renewal_status`, `completion_evidence_ref` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Certification Renewal | `CredentialRenewal` | renewal window and certification id map directly. |
| Workday Learning Recertification | `CredentialRenewal` | campaign assignment maps to renewal workflow. |
| Docebo Certification Expiry | `CredentialRenewal` | expiry date maps to `deadline_at`. |
| 360Learning Refresher Path | `LearningWorkflowTemplate` | refresher path maps to renewal course workflow. |
| LinkedIn Learning Enterprise Completion Refresh | `ProviderCompletionSignal` | refresh completion maps to supplemental evidence. |

## Workflow Steps

1. `DetectExpiringCredential` receives credential expiry from credentialing.
2. `SelectRenewalCourse` chooses approved refresher content.
3. `ScheduleRenewalWorkflow` creates assignment and reminders.
4. `MonitorCompletionEvidence` advances status after refresher completion.
5. `ReissueOrExpireCredential` calls credentialing before deadline.

Branches: waiver requires compliance-admin reason; missed deadline marks credential expired; provider completion alone requires mapped refresher course.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-CREDENTIAL-RENEWAL-SCHEDULED` | `tenant_id`, `renewal_id`, `credential_ref`, `worker_ref`, `deadline_at` |
| `EVT-LEARNING-CREDENTIAL-RENEWAL-EXPIRED` | `tenant_id`, `renewal_id`, `credential_ref`, `worker_ref` |
| `EVT-LEARNING-CREDENTIAL-RENEWAL-WAIVED` | `tenant_id`, `renewal_id`, `reason_code`, `waived_by` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Schedule renewal | 55 ms | 240 ms | 520 ms | 800 rps/cell | 99.95% |
| Daily expiry scan 100k credentials | 2 s | 12 s | 30 s | 20 scans/hour/cell | 99.9% |
| Advance renewal status | 40 ms | 180 ms | 400 ms | 1k rps/cell | 99.95% |

## Failure Modes + Recovery

- Credentialing event delayed: expiry scan backfills missing renewal schedules.
- Renewal course retired: select replacement course and notify compliance admin.
- Reminder delivery fails: retry through notification and keep renewal status unchanged.

## Migration Notes

Cornerstone, Workday Learning, and Docebo certification expiry dates import into scheduled renewals only after current credential ownership is resolved. LinkedIn Learning Enterprise completion refreshes cannot extend credentials without internal credentialing approval.

## Cross-µservice Handoffs

- `credentialing` emits expiry and receives reissue commands.
- `workflow-engine` executes renewal workflows.
- `notification` sends reminder and escalation messages.
- `compliance-governance` consumes waiver and expiration evidence.
- `audit-chain` seals scheduled, expired, and waived renewal events.
