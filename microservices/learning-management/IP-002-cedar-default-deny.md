---
doc_class: ImplementationPlan
ip_id: IP-002-cedar-default-deny
microservice: learning-management
related_adrs: [ADR-0243, ADR-0257, ADR-0263]
journey_id: J-LMS-02-governed-learning-administration
status: proposed
date: 2026-05-20
owner: axis-learning-management
tenant_class: ["demo_trial", "paid"]
---

# IP-002: Cedar Default Deny

## Context

This slice replaces permissive LMS administration defaults with explicit Cedar decisions for enrollment, course assignment, transcript export, completion override, and credential issue. It supports Marcus Chen, the compliance learning owner, who needs Workday Learning campaign parity and Cornerstone transcript governance without allowing a manager to inspect unrelated learner records.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_policy_decision` | `decision_id` | `uuid primary key` | One authorization decision. |
| `learning_policy_decision` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_policy_decision` | `principal_ref` | `text not null` | Actor or service principal. |
| `learning_policy_decision` | `learning_action` | `text not null` | Assign, enroll, export, override, issue. |
| `learning_policy_decision` | `resource_ref` | `text not null` | Course, enrollment, transcript, or credential. |
| `learning_policy_decision` | `decision` | `text not null check (decision in ('permit','deny'))` | Cedar result. |
| `learning_policy_decision` | `context_hash` | `bytea not null` | Canonicalized context digest. |

## API Endpoints

REST `POST /v1/learning-management/policy:evaluate`

```json
{
  "principal": "User::learning-admin-771",
  "action": "learningManagement::ExportTranscript",
  "resource": "Transcript::worker-991",
  "context": {
    "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00002",
    "learning_org_ref": "hris:org:manufacturing",
    "purpose": "regulatory-audit",
    "region": "US"
  }
}
```

gRPC `LearningPolicyService.Evaluate(EvaluateLearningPolicyRequest)` returns `decision`, `policy_id`, `decision_id`, and `deny_reason`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"manager"` | `learningManagement::AssignCourse` | `Course::*` | `tenant_id`, `worker_scope`, `required_by_date` |
| `User::"learner"` | `learningManagement::CompleteCourse` | `Enrollment::*` | `tenant_id`, `enrollment_owner_ref`, `assessment_score` |
| `User::"auditor"` | `learningManagement::ExportTranscript` | `Transcript::*` | `tenant_id`, `purpose`, `region` |
| `Service::"credential-worker"` | `learningManagement::IssueCredential` | `CredentialAssertion::*` | `completion_status`, `assessment_proctored` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Security Role | `LearningRoleBinding` | role and OU scope map to Cedar principal context. |
| Workday Learning Security Group | `LearningRoleBinding` | domain security group maps to action set. |
| Docebo Power User | `LearningRoleBinding` | branch and course permissions map to resource scope. |
| 360Learning Coach Role | `InstructorDelegation` | coach grants map to limited assignment actions. |
| LinkedIn Learning Enterprise Admin | `ProviderAdminGrant` | provider admin maps to catalog-read only unless elevated. |

## Workflow Steps

1. `BuildLearningContext` loads tenant, org, audience, resource owner, and region.
2. `EvaluateCedarPolicy` applies default deny if no permit matches.
3. `PersistPolicyDecision` records canonical context hash and outcome.
4. `AttachDecisionToCommand` forwards permit token to the command handler.
5. `EmitPolicyAudit` writes ADR-0263 class event.

Branches: missing context denies; learner completing own course can permit; manager exporting direct-report transcript requires purpose; service credential issue requires sealed completion.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-POLICY-PERMIT` | `tenant_id`, `principal_ref`, `learning_action`, `resource_ref`, `policy_id` |
| `EVT-LEARNING-POLICY-DENY` | `tenant_id`, `principal_ref`, `learning_action`, `deny_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Policy evaluation | 3 ms | 12 ms | 30 ms | 50k evals/s/cell | 99.99% |
| Decision persistence | 8 ms | 40 ms | 90 ms | 15k writes/s/cell | 99.95% |

## Failure Modes + Recovery

- Cedar bundle unavailable: deny all mutating actions and allow cached read-only learner views for 5 minutes.
- Context enrichment timeout: deny and enqueue diagnostic event for missing source.
- Audit write degraded: command fails closed for export and credential issue; learner self-completion retries.

## Migration Notes

Vendor role matrices from Cornerstone, Workday Learning, Docebo, 360Learning, and LinkedIn Learning Enterprise must be translated into explicit action/resource grants. Broad admin grants import disabled until reviewed.

## Cross-µservice Handoffs

- `policy-engine` evaluates Cedar bundles.
- `hris` provides manager and org context.
- `identity-access` maps principals and groups.
- `audit-chain` seals permit and deny outcomes.
- `compliance-governance` consumes transcript-export decisions.
