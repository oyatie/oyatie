---
doc_class: ImplementationPlan
ip_id: IP-003-ontology-projection
microservice: learning-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-03-learning-ontology-normalization
status: proposed
date: 2026-05-20
owner: axis-learning-management
capability_tier: T2
---

# IP-003: Ontology Projection

## Context

This slice defines how course, learning path, enrollment, completion, credential, skill, and provider content objects become Oyatie ontology objects. It supports Elena Garcia, the learning architect, consolidating Cornerstone curricula, Docebo learning plans, 360Learning collaborative courses, Workday Learning enrollments, and LinkedIn Learning Enterprise content without preserving vendor-specific object ambiguity.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_ontology_projection` | `projection_id` | `uuid primary key` | One projected source object. |
| `learning_ontology_projection` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_ontology_projection` | `source_vendor` | `text not null` | Cornerstone, Workday, Docebo, 360Learning, LinkedIn. |
| `learning_ontology_projection` | `source_object_ref` | `text not null` | Vendor object id. |
| `learning_ontology_projection` | `oya_object_type` | `text not null` | Course, LearningPath, Enrollment, CompletionEvidence, CredentialAssertion, Skill. |
| `learning_ontology_projection` | `field_delta` | `jsonb not null` | Field-level mapping. |
| `learning_ontology_projection` | `projection_status` | `text not null` | active, quarantined, superseded. |

## API Endpoints

REST `POST /v1/learning-management/ontology-projections:upsert`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00003",
  "source_vendor": "docebo",
  "source_object_ref": "learning-plan:lp-8801",
  "oya_object_type": "LearningPath",
  "field_delta": {
    "path_title": "Manager Safety Certification",
    "required_course_refs": ["course:safety-101", "course:hazmat-201"],
    "expires_after_days": 365
  }
}
```

gRPC `LearningOntologyProjectionService.Upsert(UpsertLearningProjectionRequest)` returns `projection_id`, `oya_object_ref`, and `quarantine_reason`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"catalog-sync"` | `ontology::ProjectLearningObject` | `LearningOntologyProjection::*` | `tenant_id`, `source_vendor`, `oya_object_type` |
| `User::"learning-architect"` | `learningManagement::ApproveProjection` | `LearningOntologyProjection::*` | `projection_status`, `source_vendor`, `field_delta_hash` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Curriculum | `LearningPath` | curriculum sections map to ordered path requirements. |
| Workday Learning Campaign | `LearningAssignmentCampaign` | campaign eligibility maps to audience criteria. |
| Docebo Course | `Course` | course code, branch, language, and credits map directly. |
| 360Learning Collaborative Course | `Course` | author cohort maps to instructor delegation. |
| LinkedIn Learning Enterprise Video | `ProviderContentAsset` | provider asset id maps to external content ref. |

## Workflow Steps

1. `ReadVendorObject` loads source payload and vendor metadata.
2. `NormalizeLearningType` chooses Oyatie object type.
3. `MapFieldDelta` writes explicit field deltas and rejects lossy mappings.
4. `EvaluateProjectionPolicy` checks source-vendor permit and tenant scope.
5. `PersistProjection` upserts active or quarantined projection.

Branches: missing required title quarantines; unsupported assessment format returns `422`; duplicate source ref supersedes the older projection.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-ONTOLOGY-PROJECTED` | `tenant_id`, `source_vendor`, `source_object_ref`, `oya_object_type` |
| `EVT-LEARNING-ONTOLOGY-QUARANTINED` | `tenant_id`, `source_vendor`, `source_object_ref`, `quarantine_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Project single object | 25 ms | 120 ms | 260 ms | 2k objects/s/cell | 99.95% |
| Batch projection commit | 180 ms | 1.2 s | 3 s | 250 batches/min/cell | 99.9% |

## Failure Modes + Recovery

- Vendor payload changes shape: quarantine projection and keep previous active object.
- Ontology write conflict: retry with source-object idempotency key.
- Lossy field mapping: reject and emit mapping-gap evidence for learning architect review.

## Migration Notes

Cornerstone curricula and Docebo plans often conflate course groups and mandatory paths. Migration must create separate `Course` and `LearningPath` projections before enrollments are imported.

## Cross-µservice Handoffs

- `ontology` stores normalized learning objects.
- `content-provider-integration` supplies raw provider catalog data.
- `skills-graph` consumes skills and credential links.
- `audit-chain` seals projection decisions.
- `search` indexes active course and path projections.
