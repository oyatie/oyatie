---
doc_class: ImplementationPlan
ip_id: IP-034-workflow-canvas
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0245, ADR-0263, ADR-0321, ADR-0328]
bounded_context: workflow-canvas
journey_id: J-MA-34-visual-canvas-authoring
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-034: Workflow Visual Canvas (Authoring)

## Context

Diana Alvarez and Marcus Chen author nurture journeys via a drag-and-drop canvas. HubSpot Workflows, Marketo Smart Campaign Designer, and Mailchimp Customer Journey Builder all expose visual canvas authoring distinct from runtime execution. This slice separates the authoring surface (immutable published snapshots) from the runtime (workflow-engine + this µservice's journey bounded context). Without this separation, canvas edits while a journey runs can produce inconsistent step sequences.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_workflow_canvas` | `canvas_id` | `uuid primary key` | Canvas authoring id. |
| `marketing_workflow_canvas` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_workflow_canvas` | `canvas_name` | `text not null` | Unique per tenant. |
| `marketing_workflow_canvas` | `dag` | `jsonb not null` | Steps + edges. |
| `marketing_workflow_canvas` | `entry_triggers` | `jsonb not null` | Array of trigger conditions (form_submit, list_membership_change, page_view, custom_event, lifecycle_transition, scheduled). |
| `marketing_workflow_canvas` | `exit_goals` | `jsonb not null` | Array of goal conditions (deal_stage, custom_event, manual). |
| `marketing_workflow_canvas` | `scope` | `text not null` | subject / account / audience. |
| `marketing_workflow_canvas` | `version` | `int not null` | Monotonic per canvas_name. |
| `marketing_workflow_canvas` | `status` | `text not null` | draft / validated / published / retired. |
| `marketing_workflow_canvas_snapshot` | `snapshot_id` | `uuid primary key` | Immutable published snapshot. |
| `marketing_workflow_canvas_snapshot` | `canvas_id` | `uuid not null` | FK. |
| `marketing_workflow_canvas_snapshot` | `version` | `int not null` | Snapshot version. |
| `marketing_workflow_canvas_snapshot` | `dag_frozen` | `jsonb not null` | Immutable copy of dag at publish time. |
| `marketing_workflow_canvas_snapshot` | `published_at_hlc` | `hlc not null` | Immutable timestamp. |

## Step Type Registry

Canonical step types (validated at canvas publish):
- `send_email` (binds to marketing_email.email_id)
- `send_sms` (delegates to messenger)
- `send_push` (delegates to messenger)
- `send_in_app` (delegates to messenger)
- `wait` (HLC-based delay)
- `wait_until` (absolute HLC)
- `wait_until_subject_action` (event-triggered)
- `conditional_branch` (predicate)
- `update_crm_property` (crm contract)
- `adjust_lead_score` (lead-scoring bounded context)
- `progress_lifecycle` (lifecycle-stage bounded context)
- `reserve_frequency_touch` (frequency-cap bounded context)
- `invoke_webhook` (webhook-subscription bounded context)
- `add_to_list` (static-list bounded context)
- `tag_subject` (behavioral-profile)
- `enroll_in_journey` (recursive workflow-canvas trigger)
- `end` (terminal step)

## API Endpoints

REST `POST /v1/marketing-automation/workflow-canvases`:

```json
{
  "tenant_id": "...",
  "canvas_name": "trial-nurture-2026",
  "scope": "subject",
  "entry_triggers": [
    {"type": "form_submit", "form_id": "..."},
    {"type": "list_membership_change", "list_id": "...", "direction": "added"}
  ],
  "exit_goals": [
    {"type": "deal_stage", "stage": "closed_won"},
    {"type": "custom_event", "event": "trial.converted"}
  ],
  "dag": {
    "nodes": [
      {"id": "n1", "step_type": "wait", "duration": "PT1H"},
      {"id": "n2", "step_type": "send_email", "email_id": "..."},
      {"id": "n3", "step_type": "conditional_branch", "predicate": "{{email_clicked}}"},
      {"id": "n4", "step_type": "wait", "duration": "P2D"},
      {"id": "n5", "step_type": "send_email", "email_id": "..."},
      {"id": "n6", "step_type": "end"}
    ],
    "edges": [
      {"from": "n1", "to": "n2"},
      {"from": "n2", "to": "n3"},
      {"from": "n3", "to": "n4", "branch": "true"},
      {"from": "n3", "to": "n6", "branch": "false"},
      {"from": "n4", "to": "n5"},
      {"from": "n5", "to": "n6"}
    ]
  }
}
```

REST `POST /v1/marketing-automation/workflow-canvases/{canvas_id}:validate` runs DAG validation + step-type validation.

REST `POST /v1/marketing-automation/workflow-canvases/{canvas_id}:publish` creates an immutable snapshot.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::AmendWorkflowCanvas` | `MarketingWorkflowCanvas::canvas_id` | `status`, `tenant_class` |
| `User::"marketing.ops"` | `marketingAutomation::PublishWorkflowCanvas` | `MarketingWorkflowCanvas::canvas_id` | `step_types_validated`, `dag_acyclic`, `tenant_class` |

## Workflow Steps

1. `ValidateDagAcyclic` confirms no cycles.
2. `ValidateStepTypeRegistry` ensures every step type is canonical.
3. `ValidateStepBindings` checks email_id / form_id / list_id / canvas_id references resolve.
4. `ValidateEntryTriggers` ensures each trigger type is supported.
5. `ValidateExitGoals` ensures each goal type is supported.
6. `AuthorizePublish` calls Cedar.
7. `CreateSnapshot` copies dag → dag_frozen with new snapshot_id + version increment.
8. `SealPublish` emits `EVT-MARKETING-WORKFLOW-CANVAS-PUBLISHED`.

Decision branches:
- Cyclic DAG → 422 `dag_has_cycle` with cycle path.
- Unknown step type → 422 `unknown_step_type` with registry suggestion.
- Reference resolution failure → 422 `step_binding_unresolved`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-WORKFLOW-CANVAS-CREATED` | `tenant_id`, `canvas_id`, `canvas_name`, `tenant_class` |
| `EVT-MARKETING-WORKFLOW-CANVAS-VALIDATED` | `canvas_id`, `step_count`, `edge_count`, `step_types[]` |
| `EVT-MARKETING-WORKFLOW-CANVAS-PUBLISHED` | `canvas_id`, `snapshot_id`, `version`, `published_at_hlc`, `tenant_class` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Validate canvas | 80 ms | 300 ms | 800 ms | 100 rps/cell | 99.9% |
| Publish snapshot | 100 ms | 400 ms | 1 s | 50 rps/cell | 99.95% |

## Failure Modes + Recovery

- Step-type registry version mismatch → migration tool re-validates with new registry; canvases with retired step types remain at prior snapshot.
- Race on publish (two operators publish concurrently) → CAS on `version` column; second publish gets `409 version_conflict`.
- Trigger-condition compile failure → 422 with trigger position + reason.

## Migration Notes

HubSpot Workflow export uses HubSpot Workflow API; node types map mostly 1:1 to Oyatie step types. HubSpot "If/Then" branch becomes `conditional_branch`. HubSpot Delay becomes `wait`.

Marketo Smart Campaign export uses the Marketo Smart Campaign API; Filter + Trigger + Flow Step model maps as Trigger = entry_trigger, Filter = conditional_branch predicate, Flow Step = step.

Mailchimp Customer Journey export maps starting points to entry_triggers; journey points to dag nodes.

## Cross-µservice Handoffs

- `workflow-engine` consumes the published snapshot for runtime execution.
- `marketing-asset` provides email template references resolved at publish.
- `audit-chain` seals every lifecycle event.
- `journey` (this µservice's runtime bounded context) instantiates `marketing_journey_run` from snapshots.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-034-workflow-canvas.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-034-workflow-canvas.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
