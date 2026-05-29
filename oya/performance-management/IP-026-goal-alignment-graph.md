---
doc_class: ImplementationPlan
ip_id: IP-026-goal-alignment-graph
microservice: performance-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-PM-26-cross-team-goal-alignment
status: proposed
date: 2026-05-20
owner: axis-performance-management
tenant_class: ["demo_trial", "paid"]
---

# IP-026: Goal Alignment Graph

## Context

This net-new slice models parent/child goal alignment across worker, team, department, and company levels. It displaces Lattice Goals, Workday Talent goals, and 15Five objectives while preserving Culture Amp engagement correlations as aggregate-only signals.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_goal_alignment` | `edge_id` | `uuid primary key` | One alignment edge. |
| `performance_goal_alignment` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_goal_alignment` | `parent_goal_id` | `uuid not null` | Parent goal. |
| `performance_goal_alignment` | `child_goal_id` | `uuid not null` | Child goal. |
| `performance_goal_alignment` | `alignment_weight_bps` | `integer not null` | Contribution basis points. |
| `performance_goal_alignment` | `owner_worker_ref` | `text not null` | Worker who owns child goal. |
| `performance_goal_alignment` | `effective_range` | `tstzrange not null` | Active period. |

## API Endpoints

REST `POST /v1/performance-management/goals/{goal_id}/alignment-edges`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "parent_goal_id": "018f8ad2-goal-company",
  "child_goal_id": "018f8ad2-goal-team",
  "alignment_weight_bps": 2500,
  "owner_worker_ref": "hris:worker:778"
}
```

gRPC `GoalAlignmentService.UpsertEdge(UpsertGoalAlignmentEdgeRequest)` returns `edge_id` and graph consistency warnings.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"manager"` | `performanceManagement::AlignGoal` | `PerformanceGoal::*` | `tenant_id`, `parent_goal_id`, `child_goal_id`, `owner_worker_ref` |
| `Service::"graph-worker"` | `ontology::WriteGoalAlignment` | `GoalAlignmentEdge::*` | `alignment_weight_bps`, `effective_range` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Goal Alignment | `GoalAlignmentEdge` | parent/child relation maps to edge. |
| Workday Talent Goal Cascade | `GoalAlignmentEdge` | organization goal cascade maps to edge. |
| 15Five Objective | `PerformanceGoal` | parent objective maps to alignment edge. |
| Culture Amp Driver | `EngagementGoalSignal` | aggregate driver maps to advisory signal, not edge. |

## Workflow Steps

1. `ValidateGoals` confirms both goals exist and belong to tenant.
2. `CheckManagerRelation` verifies actor can align child goal.
3. `DetectCycles` rejects cyclic graph edges.
4. `PersistAlignmentEdge` writes edge.
5. `ProjectOntologyEdge` updates graph.

Branches: cycle detected returns `409`; weight over 10000 returns `422`; cross-tenant parent denies.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-GOAL-ALIGNED` | `tenant_id`, `parent_goal_id`, `child_goal_id`, `alignment_weight_bps` |
| `EVT-PERFORMANCE-GOAL-ALIGNMENT-DENIED` | `deny_reason`, `parent_goal_id`, `child_goal_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Upsert alignment edge | 45 ms | 180 ms | 400 ms | 500 rps/cell | 99.95% |
| Cycle detection for 10k graph | 120 ms | 900 ms | 2 s | 200 checks/min/cell | 99.9% |

## Failure Modes + Recovery

- Graph cycle: reject edge and return cycle path.
- HRIS relation unavailable: queue pending alignment and retry.
- Parent goal archived: deny and require active parent selection.

## Migration Notes

Vendor goal hierarchies rarely share weight semantics. Migration imports parent/child relation and sets weight to advisory until manager confirms.

## Cross-µservice Handoffs

- `ontology` stores goal edges.
- `hris` supplies manager and org relations.
- `audit-chain` seals alignment events.
- `analytics` consumes aggregate alignment graph.
- `learning-management` can consume approved skill-gap links later.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-026-goal-alignment-graph.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-026-goal-alignment-graph.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
