---
doc_class: ImplementationPlan
ip_id: IP-005-rest-contract-surface
microservice: learning-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-05-learning-api-contract
status: proposed
date: 2026-05-20
owner: axis-learning-management
capability_tier: T2
---

# IP-005: REST Contract Surface

## Context

This slice defines the externally callable REST contract for catalog search, enrollment, completion evidence, transcript export, and credential issue. It supports Omar Watkins, the HR systems integrator, replacing Cornerstone and Workday Learning APIs while also ingesting Docebo, 360Learning, and LinkedIn Learning Enterprise content without undocumented side effects.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_api_request` | `request_id` | `uuid primary key` | API request id. |
| `learning_api_request` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_api_request` | `route_code` | `text not null` | Stable REST route code. |
| `learning_api_request` | `idempotency_key` | `text` | Required for writes. |
| `learning_api_request` | `principal_ref` | `text not null` | Caller identity. |
| `learning_api_request` | `response_status` | `integer not null` | HTTP status. |
| `learning_api_request` | `latency_ms` | `integer not null` | Request latency. |

## API Endpoints

REST `POST /v1/learning-management/enrollments`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00005",
  "learner_ref": "hris:worker:442",
  "course_ref": "course:privacy-annual-2026",
  "assignment_ref": "campaign:privacy-renewal",
  "due_at": "2026-06-30T23:59:59Z",
  "idempotency_key": "privacy-annual-2026-worker-442"
}
```

Additional REST routes: `GET /v1/learning-management/catalog`, `POST /v1/learning-management/completion-evidence`, `GET /v1/learning-management/transcripts/{worker_ref}`, `POST /v1/learning-management/credentials:issue`.

gRPC `LearningContractService.Dispatch(DispatchLearningCommandRequest)` mirrors write routes for internal services.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"learner"` | `learningManagement::SearchCatalog` | `CourseCatalog::*` | `tenant_id`, `audience`, `locale` |
| `User::"manager"` | `learningManagement::CreateEnrollment` | `Enrollment::*` | `tenant_id`, `learner_ref`, `course_ref` |
| `User::"auditor"` | `learningManagement::ReadTranscript` | `Transcript::*` | `tenant_id`, `purpose`, `worker_ref` |
| `Service::"credentialing"` | `learningManagement::IssueCredential` | `CredentialAssertion::*` | `completion_evidence_ref`, `expires_at` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Transcript API | `Transcript` | transcript rows map to completion evidence refs. |
| Workday Learning Enrollment | `Enrollment` | learning enrollment id maps to idempotent enrollment ref. |
| Docebo Course API | `Course` | code, status, language, and duration map to catalog result. |
| 360Learning Completion | `CompletionEvidence` | score and reviewer maps to evidence detail. |
| LinkedIn Learning Enterprise Asset | `ProviderContentAsset` | provider urn maps to catalog external ref. |

## Workflow Steps

1. `AuthenticateCaller` binds principal and tenant.
2. `ValidateRequestShape` enforces route-specific required fields.
3. `EvaluateCedar` checks route action against resource and context.
4. `ExecuteCommandOrQuery` calls the bounded handler.
5. `RecordApiEvidence` persists request, status, latency, and audit event.

Branches: missing idempotency key rejects write with `400`; duplicate write returns existing resource; unauthorized transcript read returns `403`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-API-REQUEST-ACCEPTED` | `tenant_id`, `route_code`, `request_id`, `principal_ref` |
| `EVT-LEARNING-API-REQUEST-DENIED` | `tenant_id`, `route_code`, `principal_ref`, `deny_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Catalog search | 45 ms | 180 ms | 400 ms | 3k rps/cell | 99.95% |
| Enrollment create | 55 ms | 220 ms | 500 ms | 1k rps/cell | 99.95% |
| Transcript export | 90 ms | 500 ms | 1.4 s | 300 rps/cell | 99.9% |

## Failure Modes + Recovery

- Duplicate idempotency key: return existing resource and original audit id.
- Catalog index stale: serve last indexed snapshot and emit degraded freshness event.
- Transcript store timeout: retry read with bounded backoff and return `503` after budget.

## Migration Notes

Cornerstone, Workday Learning, Docebo, 360Learning, and LinkedIn Learning Enterprise API clients must call these stable routes. Vendor-specific route names remain adapters, not canonical contracts.

## Cross-µservice Handoffs

- `api-gateway` enforces auth and rate limits.
- `policy-engine` evaluates Cedar route decisions.
- `search` answers catalog queries.
- `credentialing` issues credential assertions.
- `audit-chain` records accepted and denied API requests.
