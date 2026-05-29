---
doc_class: ImplementationPlan
ip_id: IP-038-lifecycle-stage
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: lifecycle-stage
journey_id: J-MA-38-lifecycle-funnel-progression
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-038: Lifecycle Stage Progression

## Context

Pat Lee enforces marketing funnel discipline: Subscriber → Lead → MQL → SQL → Opportunity → Customer → Evangelist. The progression touches both marketing (pre-MQL handoff) and sales (post-SQL handoff). HubSpot Lifecycle Stage is the canonical model; Marketo Engagement Score Buckets and Mailchimp CLV bands are narrower. This slice owns the marketing-side lifecycle ownership and the handoff seam to crm.lead + crm.opportunity.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_lifecycle_progression` | `progression_id` | `uuid primary key` | Per-subject lifecycle state. |
| `marketing_lifecycle_progression` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_lifecycle_progression` | `subject_hash` | `text not null` | Subject ref. |
| `marketing_lifecycle_progression` | `current_stage` | `text not null` | subscriber / lead / mql / sql / opportunity / customer / evangelist. |
| `marketing_lifecycle_progression` | `entered_stage_hlc` | `hlc not null` | When entered current stage. |
| `marketing_lifecycle_progression` | `previous_stage` | `text` | Prior stage. |
| `marketing_lifecycle_progression` | `monotonic_locked` | `boolean not null default true` | If true, prevents downgrade. |
| `marketing_lifecycle_transition` | `transition_id` | `uuid primary key` | Transition audit row. |
| `marketing_lifecycle_transition` | `progression_id` | `uuid not null` | FK. |
| `marketing_lifecycle_transition` | `from_stage` | `text not null` | Previous stage. |
| `marketing_lifecycle_transition` | `to_stage` | `text not null` | New stage. |
| `marketing_lifecycle_transition` | `kind` | `text not null` | progress / downgrade. |
| `marketing_lifecycle_transition` | `reason` | `text` | Required for downgrade. |
| `marketing_lifecycle_transition` | `principal_id` | `uuid not null` | Principal who applied. |
| `marketing_lifecycle_transition` | `transition_hlc` | `hlc not null` | HLC stamp. |

## API Endpoints

REST `POST /v1/marketing-automation/lifecycle-stage/{subject_hash}:progress`:

```json
{"to_stage": "mql"}
```

REST `POST /v1/marketing-automation/lifecycle-stage/{subject_hash}:downgrade`:

```json
{"to_stage": "lead", "reason": "Disqualified after discovery call — no budget"}
```

REST `GET /v1/marketing-automation/lifecycle-stage/{subject_hash}` returns current state + transition history.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::ProgressLifecycle` | `MarketingLifecycleProgression::progression_id` | `from_stage`, `to_stage`, `monotonic_allowed` |
| `User::"revops.manager"` | `marketingAutomation::DowngradeLifecycle` | `MarketingLifecycleProgression::progression_id` | `from_stage`, `to_stage`, `reason`, `step_up_authenticated` |

Downgrade requires Cedar step-up + `revops.manager` role + reason.

## Workflow Steps

1. `LoadCurrentStage` reads `current_stage`.
2. On progress, `ValidateProgressionMonotonic` confirms `to_stage` is forward.
3. On downgrade, `ValidateDowngradeAuth` confirms Cedar step-up + reason.
4. `WriteTransition` writes `marketing_lifecycle_transition` row + updates progression.
5. `EmitTransition` emits `EVT-MARKETING-LIFECYCLE-PROGRESSED` or `EVT-MARKETING-LIFECYCLE-DOWNGRADED`.
6. On MQL→SQL transition, `NotifyCrm` posts to crm.lead via crm contract; on SQL→Opportunity transition, posts to crm.opportunity.
7. `TriggerWorkflowsOnTransition` fires workflow-canvas entry_triggers with type `lifecycle_transition`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-LIFECYCLE-PROGRESSED` | `subject_hash`, `progression_id`, `from_stage`, `to_stage`, `principal_id`, `transition_hlc` |
| `EVT-MARKETING-LIFECYCLE-DOWNGRADED` | `subject_hash`, `progression_id`, `from_stage`, `to_stage`, `reason`, `principal_id`, `step_up_decision_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Progress stage | 50 ms | 200 ms | 500 ms | 1000 rps/cell | 99.95% |
| Downgrade stage | 100 ms | 400 ms | 1 s | 100 rps/cell | 99.95% |
| Read progression | 10 ms | 40 ms | 100 ms | 5000 rps/cell | 99.99% |

## Failure Modes + Recovery

- CRM unreachable on MQL→SQL → buffer the transition + retry; lifecycle state advances locally but crm.lead update is deferred with `EVT-MARKETING-LIFECYCLE-CRM-HANDOFF-DEFERRED`.
- Concurrent progress race → CAS on `current_stage`; second write gets `409 stage_conflict`.
- Downgrade without step-up → 403 `step_up_required`.

## Migration Notes

HubSpot Lifecycle Stage maps 1:1 to Oyatie stages. Marketo Engagement Score Buckets map by score-band → stage. Mailchimp CLV bands map narrower (Customer / Evangelist only).

## Cross-µservice Handoffs

- `crm.lead` consumes MQL→SQL transition.
- `crm.opportunity` consumes SQL→Opportunity transition.
- `workflow-canvas` consumes lifecycle_transition trigger.
- `audit-chain` seals every transition.
- `customer-analytics` reads progression history for funnel reports.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-038-lifecycle-stage.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-038-lifecycle-stage.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
