---
doc_class: ImplementationPlan
ip_id: IP-026-skills-graph-gap-analyzer
microservice: learning-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-26-role-skills-gap-remediation
status: proposed
date: 2026-05-20
owner: axis-learning-management
capability_tier: T3
---

# IP-026: Skills Graph Gap Analyzer

## Context

This net-new slice compares required role skills against learner evidence and maps gaps to approved learning content. It supports Diana Alvarez, the sales enablement director, replacing Cornerstone skills, Workday Skills Cloud learning suggestions, Docebo skill tags, 360Learning upskilling paths, and LinkedIn Learning Enterprise skill mappings with explainable gap analysis.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_skill_gap` | `gap_id` | `uuid primary key` | One worker skill gap. |
| `learning_skill_gap` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_skill_gap` | `worker_ref` | `text not null` | Learner worker ref. |
| `learning_skill_gap` | `role_ref` | `text not null` | Target role or job profile. |
| `learning_skill_gap` | `skill_ref` | `text not null` | Required skill. |
| `learning_skill_gap` | `gap_score_bps` | `integer not null` | 0-10000 basis point gap. |
| `learning_skill_gap` | `recommended_course_refs` | `text[] not null` | Approved remediation content. |

## API Endpoints

REST `POST /v1/learning-management/skill-gaps:analyze`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00026",
  "worker_ref": "hris:worker:881",
  "role_ref": "job-profile:enterprise-ae",
  "evidence_refs": ["completion:negotiation-101", "credential:salesforce-admin"],
  "catalog_scope": "global-sales"
}
```

gRPC `LearningSkillGapService.Analyze(AnalyzeSkillGapRequest)` returns `gap_ids`, `recommended_course_refs`, and `explanation_tokens`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"manager"` | `learningManagement::AnalyzeSkillGap` | `WorkerSkillProfile::*` | `tenant_id`, `worker_ref`, `role_ref` |
| `Service::"skills-graph"` | `learningManagement::WriteSkillGap` | `LearningSkillGap::*` | `skill_ref`, `gap_score_bps`, `catalog_scope` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Skill | `Skill` | skill id and proficiency map to graph node. |
| Workday Skills Cloud Skill | `Skill` | skill urn maps to canonical skill ref. |
| Docebo Skill Tag | `CourseSkillLink` | tag maps to course-to-skill edge. |
| 360Learning Path Skill | `LearningPathSkillCoverage` | path objective maps to skill coverage. |
| LinkedIn Learning Enterprise Skill | `ProviderSkillSignal` | provider skill maps to advisory signal. |

## Workflow Steps

1. `LoadRoleRequirements` reads required skills from HRIS and skills graph.
2. `LoadLearnerEvidence` reads completions, credentials, assessments, and prior skills.
3. `ComputeGapScore` calculates missing proficiency per skill.
4. `SelectApprovedCourses` chooses tenant-visible remediation content.
5. `PersistGapAnalysis` writes gaps with explanations.

Branches: no approved course creates uncovered gap; stale evidence is ignored; manager outside reporting line receives deny.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-SKILL-GAP-ANALYZED` | `tenant_id`, `worker_ref`, `role_ref`, `gap_count` |
| `EVT-LEARNING-SKILL-GAP-COURSE-SELECTED` | `tenant_id`, `gap_id`, `skill_ref`, `course_ref` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Gap analysis | 120 ms | 650 ms | 1.6 s | 250 analyses/min/cell | 99.9% |
| Course selection | 40 ms | 180 ms | 420 ms | 1k selections/s/cell | 99.95% |

## Failure Modes + Recovery

- Skills graph unavailable: return stale cached gap snapshot with freshness marker.
- Course coverage missing: persist uncovered gap and notify learning architect.
- Conflicting skill aliases: quarantine ambiguous skill until ontology mapping resolves.

## Migration Notes

Vendor skill tags import as advisory edges until mapped to canonical `Skill` records. LinkedIn Learning Enterprise skills cannot by themselves satisfy internal role requirements.

## Cross-µservice Handoffs

- `skills-graph` supplies skill and role edges.
- `hris` supplies worker, role, and manager relationships.
- `search` returns eligible remediation content.
- `performance-management` can consume aggregated development-plan gaps.
- `audit-chain` seals analysis and recommendation events.
