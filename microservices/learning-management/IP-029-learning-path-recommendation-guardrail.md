---
doc_class: ImplementationPlan
ip_id: IP-029-learning-path-recommendation-guardrail
microservice: learning-management
related_adrs: [ADR-0243, ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-29-governed-learning-recommendations
status: proposed
date: 2026-05-20
owner: axis-learning-management
capability_tier: T3
---

# IP-029: Learning Path Recommendation Guardrail

## Context

This net-new slice governs recommended courses and learning paths so automated suggestions cannot bypass compliance, license, audience, manager-approval, or regional constraints. It supports Priya Nair replacing LinkedIn Learning Enterprise recommendations, Workday Learning suggestions, 360Learning path nudges, Docebo recommendations, and Cornerstone playlists with explainable, policy-bound outputs.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_recommendation` | `recommendation_id` | `uuid primary key` | One governed recommendation. |
| `learning_recommendation` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_recommendation` | `learner_ref` | `text not null` | Target learner. |
| `learning_recommendation` | `recommended_object_ref` | `text not null` | Course or learning path. |
| `learning_recommendation` | `reason_code` | `text not null` | skill-gap, compliance, manager, career-path. |
| `learning_recommendation` | `policy_outcome` | `text not null` | eligible, suppressed, approval_required. |
| `learning_recommendation` | `explanation` | `jsonb not null` | Human-readable explanation tokens. |

## API Endpoints

REST `POST /v1/learning-management/recommendations:generate`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00029",
  "learner_ref": "hris:worker:515",
  "signals": {
    "skill_gap_ids": ["gap:sales-negotiation"],
    "career_path_ref": "career-path:enterprise-ae",
    "manager_requested": true
  },
  "max_results": 5
}
```

gRPC `LearningRecommendationService.Generate(GenerateLearningRecommendationsRequest)` returns `recommendation_ids`, `suppressed_count`, and `approval_required_count`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"recommendation-engine"` | `learningManagement::RecommendLearningObject` | `Course::*` | `tenant_id`, `learner_ref`, `license_state`, `audience` |
| `User::"manager"` | `learningManagement::ApproveRecommendation` | `LearningRecommendation::*` | `learner_ref`, `recommended_object_ref`, `reason_code` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| LinkedIn Learning Recommendation | `LearningRecommendation` | provider reason maps to advisory reason code. |
| Workday Learning Suggested Learning | `LearningRecommendation` | worker context maps to learner ref and eligibility. |
| 360Learning Path Recommendation | `LearningRecommendation` | path id maps to recommended object ref. |
| Docebo Recommendation | `LearningRecommendation` | skill tag reason maps to skill-gap reason. |
| Cornerstone Playlist | `LearningPath` | playlist maps to recommended learning path. |

## Workflow Steps

1. `CollectRecommendationSignals` loads skill gaps, compliance due dates, and manager requests.
2. `RankCandidateContent` scores eligible courses and paths.
3. `EvaluateCedarGuardrail` suppresses disallowed content and marks approvals.
4. `PersistRecommendations` stores results and explanations.
5. `NotifyLearnerOrManager` emits the appropriate downstream event.

Branches: license exhausted suppresses recommendation; compliance overdue outranks career content; manager approval required pauses learner notification.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-RECOMMENDATION-GENERATED` | `tenant_id`, `learner_ref`, `recommendation_id`, `reason_code`, `policy_outcome` |
| `EVT-LEARNING-RECOMMENDATION-SUPPRESSED` | `tenant_id`, `learner_ref`, `recommended_object_ref`, `suppress_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Generate recommendations | 90 ms | 450 ms | 1.1 s | 500 rps/cell | 99.9% |
| Approve recommendation | 35 ms | 160 ms | 360 ms | 700 rps/cell | 99.95% |

## Failure Modes + Recovery

- Ranking service unavailable: return compliance-required recommendations only.
- License state stale: mark candidate approval_required and refresh entitlement.
- Explanation build failure: suppress recommendation until explainability payload is present.

## Migration Notes

Vendor recommendations import as historical signals, not active assignments. Active recommendations must be regenerated through Oyatie guardrails after catalog and skills mappings are available.

## Cross-µservice Handoffs

- `skills-graph` supplies skill-gap signals.
- `billing-entitlements` supplies provider license state.
- `notification` sends learner and manager actions.
- `performance-management` can consume approved development-plan recommendations.
- `audit-chain` seals generated and suppressed outcomes.
