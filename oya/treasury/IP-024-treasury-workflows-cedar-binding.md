---
doc_class: ImplementationPlan
ip_id: IP-024
microservice: treasury
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0319]
journey_id: j120-tenant-treasury-multi-currency-fx-hedge
journey_link: docs/user-journeys/j120-tenant-treasury-multi-currency-fx-hedge/story.md
status: Accepted
date: 2026-05-20
owner: axis-treasury
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [TRM-CM Release Workflow, TRM-TM Transaction Approval, TRM-RM Limit Workflow]
---

# IP-024: Treasury workflows Cedar binding

## Intent
Implement a treasury workflow binding layer that makes Cedar policy decisions first-class workflow node inputs and outputs.
The binding lets cash pooling, payment release, netting, revaluation, bank-message replay, and hedge recommendation workflows enforce the same policy contract.
The feature displaces SAP workflow customizing, SAP BCM release strategy, TRM-TM transaction approval, and TRM-RM limit workflow patterns for treasury.
The implementation must not create a separate workflow engine; it binds treasury workflow nodes to existing workflow service and Cedar evaluation.
The implementation must persist policy decision evidence for every guarded node.
The implementation must support simulation mode so operators can preview why a workflow branch will deny.
The implementation must support principal, action, resource, and context serialization with versioned schemas.
The implementation must emit ADR-0263 audit events for binding creation, decision capture, simulation, and deny handling.
The implementation must be reusable by IP-016 through IP-025 workflows.
The implementation must keep all policy effects tenant-scoped and cell-local.

## Context
Why: treasury workflows are policy-heavy and cannot be safe if Cedar checks are scattered across handlers without workflow evidence.
Why: SAP workflow customizing gives approvals but does not expose portable policy evidence for every branch.
Why: Oyatie needs one binding contract so implementers do not invent a different Cedar call shape for each treasury use case.
Journey leg: j120 operations supervisor configures a policy-governed hedge workflow where large sweeps, payment releases, revaluation approvals, and message replays all call Cedar before state transitions.
Named persona: Nora Lind, Treasury Controls Manager, reviews workflow denial evidence during an internal audit.
Supporting persona: Ilya Novak, Platform Workflow Engineer, integrates treasury nodes with the workflow service.
Pain point: a denied approval today leaves different logs depending on which handler performed the check.
Pain point: operators need policy simulation before close windows to detect missing approvers or scope gaps.
Pain point: workflow retry must not re-evaluate against mutated context without recording both decisions.
SAP parity: SAP release strategy, workflow customizing, BCM approvals, TRM-TM deal release, and TRM-RM limit breach workflow.
Product outcome: every guarded workflow node has a stored binding, decision record, simulation result, and audit event.
Non-goal: authoring Cedar policies themselves remains in policy governance.
Non-goal: executing workflow graphs remains in workflow service.
Non-goal: identity lifecycle and role assignment remain in identity.
Invariant: guarded workflow nodes cannot transition without a policy decision record.
Invariant: decision input payloads are versioned and hashable.
Invariant: simulation decisions never mutate business resources.
Invariant: replayed nodes record a new decision attempt while retaining the original attempt.
Acceptance anchor: an intern can add schema, binding resolver, policy client, workflow node adapter, and tests from this file.

## Data Model Deltas
Table `treasury.workflow_policy_binding`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `binding_code TEXT NOT NULL`.
Column `workflow_name TEXT NOT NULL`.
Column `node_name TEXT NOT NULL`.
Column `action_name TEXT NOT NULL`.
Column `resource_type TEXT NOT NULL`.
Column `principal_schema_version TEXT NOT NULL`.
Column `resource_schema_version TEXT NOT NULL`.
Column `context_schema_version TEXT NOT NULL`.
Column `decision_mode TEXT NOT NULL CHECK (decision_mode IN ('Enforce','Simulate','AuditOnly'))`.
Column `active BOOLEAN NOT NULL DEFAULT true`.
Column `effective_from DATE NOT NULL`.
Column `effective_to DATE`.
Constraint `UNIQUE (tenant_id, workflow_name, node_name, effective_from)`.
Table `treasury.workflow_policy_decision`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `binding_id UUID NOT NULL REFERENCES treasury.workflow_policy_binding(id)`.
Column `workflow_instance_id UUID NOT NULL`.
Column `workflow_node_instance_id UUID NOT NULL`.
Column `business_resource_id UUID`.
Column `principal_id UUID NOT NULL`.
Column `action_name TEXT NOT NULL`.
Column `decision TEXT NOT NULL CHECK (decision IN ('Allow','Deny','Error'))`.
Column `decision_mode TEXT NOT NULL`.
Column `cedar_decision_id UUID`.
Column `input_hash TEXT NOT NULL`.
Column `principal_json JSONB NOT NULL`.
Column `resource_json JSONB NOT NULL`.
Column `context_json JSONB NOT NULL`.
Column `decision_reason TEXT`.
Column `evaluated_at TIMESTAMPTZ NOT NULL`.
Constraint `UNIQUE (tenant_id, workflow_node_instance_id, binding_id, input_hash)`.
Index `ix_workflow_policy_decision_resource` on `(tenant_id, business_resource_id, evaluated_at DESC)`.
Table `treasury.workflow_policy_simulation`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `binding_id UUID NOT NULL REFERENCES treasury.workflow_policy_binding(id)`.
Column `simulated_by_principal_id UUID NOT NULL`.
Column `scenario_name TEXT NOT NULL`.
Column `principal_json JSONB NOT NULL`.
Column `resource_json JSONB NOT NULL`.
Column `context_json JSONB NOT NULL`.
Column `decision TEXT NOT NULL CHECK (decision IN ('Allow','Deny','Error'))`.
Column `decision_reason TEXT`.
Column `input_hash TEXT NOT NULL`.
Column `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
Table `treasury.workflow_policy_binding_violation`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `workflow_instance_id UUID NOT NULL`.
Column `workflow_node_instance_id UUID NOT NULL`.
Column `binding_id UUID`.
Column `severity TEXT NOT NULL CHECK (severity IN ('Warning','Blocking','Security'))`.
Column `code TEXT NOT NULL`.
Column `message TEXT NOT NULL`.
Column `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
Storage rule: decision input JSON is retained as evidence and must not include secrets.
Partitioning rule: decisions partition by tenant cell and evaluated month.
Retention rule: enforce ten-year retention for Enforce-mode decisions tied to financial state transitions.

## API Endpoints
REST `POST /v1/treasury/workflow-policy-bindings`.
Request example:
```json
{
  "binding_code": "payment-release-dual-control",
  "workflow_name": "treasury.payment.release",
  "node_name": "cedar_dual_control_check",
  "action_name": "release_payment_batch",
  "resource_type": "PaymentBatch",
  "decision_mode": "Enforce",
  "effective_from": "2026-05-20"
}
```
Response example:
```json
{
  "binding_id": "14da0ec0-1111-42ef-a222-333344445555",
  "active": true,
  "decision_mode": "Enforce"
}
```
REST `POST /v1/treasury/workflow-policy-bindings/{binding_id}/simulate`.
Simulation request includes `scenario_name`, `principal`, `resource`, and `context`.
Simulation response includes decision, reason, input hash, and simulation id.
REST `POST /v1/treasury/workflow-policy-decisions:evaluate`.
Evaluate request includes workflow instance id, node instance id, binding code, business resource id, principal, resource, and context.
Evaluate response includes decision, Cedar decision id, input hash, and branch recommendation.
REST `GET /v1/treasury/workflow-policy-decisions?business_resource_id=...`.
REST `GET /v1/treasury/workflow-policy-bindings?workflow_name=treasury.payment.release`.
gRPC `TreasuryWorkflowPolicyBindingService.Evaluate(EvaluateWorkflowPolicyRequest) returns (WorkflowPolicyDecision)`.
gRPC `TreasuryWorkflowPolicyBindingService.Simulate(SimulateWorkflowPolicyRequest) returns (WorkflowPolicySimulation)`.
Error `404 WORKFLOW_POLICY_BINDING_NOT_FOUND` when no active binding exists.
Error `409 WORKFLOW_POLICY_INPUT_REPLAY_DIFFERENT_DECISION` when same node replay has different input hash.
Error `422 WORKFLOW_POLICY_SCHEMA_INVALID` when principal, resource, or context fails binding schema.

## Cedar Policy Hooks
Principal shape: `UserOrService::{ id, tenant_id, roles, workflow_admin, policy_simulation_scope }`.
Action `Action::"create_workflow_policy_binding"`.
Action `Action::"simulate_workflow_policy_binding"`.
Action `Action::"evaluate_workflow_policy_binding"`.
Resource `WorkflowPolicyBinding::{ tenant_id, workflow_name, node_name, action_name, decision_mode, active }`.
Context `WorkflowPolicyBindingContext::{ now, request_origin, simulation_only, target_resource_type, schema_versions }`.
Permit treasury workflow admins to create bindings for treasury workflows.
Permit operators to simulate bindings only for workflows in their simulation scope.
Permit workflow service principal to evaluate active bindings.
Forbid non-service principals from calling enforce evaluation endpoint directly.
Forbid AuditOnly bindings for nodes marked `state_transition_required`.
Forbid schema version downgrade unless workflow admin has break-glass role.
Emit `TreasuryWorkflowCedarBindingPolicyDenied` for every deny.
Policy fixture `policy/workflow-binding-direct-evaluate-deny.json`.
Policy fixture `policy/workflow-binding-audit-only-state-transition-deny.json`.
Policy fixture `policy/workflow-binding-schema-downgrade-deny.json`.

## Ontology Projection
SAP workflow task maps to `Oyatie::Treasury::WorkflowPolicyBinding`.
SAP release strategy step maps to guarded workflow node binding.
SAP BCM approval step maps to policy decision record.
SAP TRM-TM deal release limit check maps to workflow policy decision.
SAP TRM-RM limit breach workflow maps to binding violation.
Cedar policy decision maps to `WorkflowPolicyDecision`.
Workflow simulation maps to `WorkflowPolicySimulation`.
Ontology field `WorkflowPolicyBinding.workflowName` maps from `workflow_name`.
Ontology field `WorkflowPolicyBinding.nodeName` maps from `node_name`.
Ontology field `WorkflowPolicyBinding.actionName` maps from `action_name`.
Ontology field `WorkflowPolicyDecision.decision` maps from `decision`.
Ontology field `WorkflowPolicyDecision.inputHash` maps from `input_hash`.
Ontology field `WorkflowPolicyDecision.cedarDecision` maps from `cedar_decision_id`.
Ontology field `WorkflowPolicySimulation.scenarioName` maps from `scenario_name`.
Ontology edge `BINDING_GUARDS_WORKFLOW_NODE` connects binding to workflow node.
Ontology edge `DECISION_EVALUATED_BINDING` connects decision to binding.
Ontology edge `DECISION_GUARDED_RESOURCE` connects decision to business resource.
Ontology edge `SIMULATION_TESTED_BINDING` connects simulation to binding.
Projection must redact decision input JSON fields marked sensitive by schema.

## Workflow Steps
Workflow `treasury.workflow_policy_binding.evaluate_node`.
Node `load_active_binding` selects by tenant, workflow name, node name, and date.
Node `validate_principal_resource_context_schema` validates payload versions.
Node `compute_input_hash` hashes normalized principal, action, resource, and context.
Node `call_cedar_evaluator` gets allow, deny, or error.
Node `persist_decision_record` stores full input and decision evidence.
Node `emit_decision_captured`.
Branch `decision_allow` returns workflow branch `continue`.
Branch `decision_deny` returns workflow branch `deny_path`.
Branch `decision_error` returns workflow branch `retry_or_fail` based on node config.
Workflow `treasury.workflow_policy_binding.simulate`.
Node `load_binding_for_simulation`.
Node `cedar_simulation_permission_check`.
Node `validate_simulation_payload`.
Node `call_cedar_in_simulation_mode`.
Node `persist_simulation_record`.
Node `emit_simulation_completed`.
Workflow `treasury.workflow_policy_binding.validate_bindings`.
Node `list_active_treasury_workflows`.
Node `find_guarded_nodes_without_binding`.
Node `find_bindings_with_schema_drift`.
Node `persist_binding_violations`.
Node `emit_binding_validation_completed`.

## Audit Events
Audit event class `TreasuryWorkflowCedarBindingCreated`.
Audit event class `TreasuryWorkflowCedarBindingUpdated`.
Audit event class `TreasuryWorkflowCedarBindingRetired`.
Audit event class `TreasuryWorkflowCedarDecisionEvaluated`.
Audit event class `TreasuryWorkflowCedarDecisionDenied`.
Audit event class `TreasuryWorkflowCedarDecisionErrored`.
Audit event class `TreasuryWorkflowCedarSimulationRequested`.
Audit event class `TreasuryWorkflowCedarSimulationCompleted`.
Audit event class `TreasuryWorkflowCedarBindingViolationRaised`.
Audit event class `TreasuryWorkflowCedarBindingPolicyDenied`.
Audit payload must include tenant id, binding id, workflow name, node name, action name, decision, and input hash.
Audit payload for decisions must include workflow instance id and node instance id.
Audit payload for denies must include Cedar decision id and decision reason.
Audit payload for simulations must include scenario name and simulation-only flag.
Audit retention class is `TreasuryWorkflowPolicyEvidence`.
Audit ordering key is `tenant_id:workflow_name:node_name:workflow_node_instance_id`.

## SLO Targets
p50 policy binding evaluation latency excluding Cedar service time: 20 ms.
p95 policy binding evaluation latency excluding Cedar service time: 80 ms.
p99 policy binding evaluation latency excluding Cedar service time: 150 ms.
p50 simulation latency excluding Cedar service time: 30 ms.
p95 simulation latency excluding Cedar service time: 120 ms.
p99 simulation latency excluding Cedar service time: 250 ms.
Throughput target: 10000 policy decisions per minute per cell.
Throughput target: 2000 simulations per minute per cell.
Availability target for enforce evaluation API: 99.99 percent monthly.
Availability target for simulation API: 99.95 percent monthly.
Rationale: evaluation is on critical workflow state-transition paths and must add minimal overhead.
Rationale: simulation is operator-facing and can tolerate slightly lower availability.
Rationale: high throughput covers payment batches and bank-message delivery bursts.

## Failure Modes + Recovery
Failure `BINDING_NOT_FOUND`: detect missing active binding; recover by blocking guarded node and raising binding violation.
Failure `SCHEMA_INVALID`: detect JSON schema validation failure; recover by fixing node payload serializer.
Failure `CEDAR_EVALUATOR_UNAVAILABLE`: detect policy client timeout; recover by retrying according to workflow node policy.
Failure `DECISION_DENY`: detect deny result; recover by following configured deny branch and surfacing reason.
Failure `AUDIT_ONLY_ON_STATE_TRANSITION`: detect invalid binding config; recover by switching to Enforce or removing guarded transition flag.
Failure `INPUT_HASH_REPLAY_MISMATCH`: detect replay changed inputs; recover by recording new attempt and requiring supervisor review.
Failure `SENSITIVE_FIELD_CAPTURED`: detect schema sensitive marker violation; recover by quarantining decision record and redacting via repair migration.
Failure `AUDIT_APPEND_FAILED`: detect audit-chain error; recover by aborting workflow transition.
Failure `POLICY_SCHEMA_DOWNGRADE`: detect version decrease; recover by requiring break-glass approval.
Failure `SIMULATION_SCOPE_DENIED`: detect operator scope mismatch; recover by assigning correct simulation scope.
Recovery worker `treasury.workflow_policy_binding.violation_scanner` periodically scans guarded workflow nodes for missing bindings.
Runbook entry `runbooks/treasury-workflow-cedar-binding-failure.md` should cover missing binding, Cedar outage, and replay mismatch.

## Migration Notes
Source vendor surface: SAP workflow customizing.
Source vendor surface: SAP release strategy configuration.
Source vendor surface: SAP BCM approval workflow.
Source vendor surface: SAP TRM-TM transaction release limits.
Source vendor surface: SAP TRM-RM limit workflow.
Source vendor surface: Kyriba approval workflow rules.
Source vendor surface: GTreasury workflow approval rules.
Migration maps SAP workflow task id to workflow name and node name.
Migration maps release code to Cedar action name.
Migration maps approval amount limit to resource and context schema fields.
Migration maps SAP role requirement to role in principal schema.
Migration imports historical workflow approvals as decision records only when input evidence is available.
Migration dry-run report lists workflow nodes without enforceable policy mapping.
Migration dry-run report lists policies that require schema fields not present in workflow payload.
Migration acceptance requires all IP-016 through IP-025 guarded nodes to have active bindings or documented non-guarded status.

## Cross-microservice Handoffs
Handoff to `workflow`: evaluate policy for guarded treasury nodes and return branch instructions.
Handoff to `policy`: call Cedar evaluator and retrieve decision ids.
Handoff to `identity`: resolve principal attributes and scope.
Handoff to `audit-chain`: seal binding, decision, simulation, deny, and violation events.
Handoff to `ontology`: project binding, decision, simulation, and guarded resource edges.
Handoff to `cash-position`: guard rollup approval and supersede nodes.
Handoff to `payments`: guard payment release and payment factory routing nodes.
Handoff to `risk`: guard FX hedge recommendation approval nodes.
Handoff to `bank-message-ingestion`: guard replay and delivery nodes.
Handoff to `ops-dashboard`: show missing bindings, deny rate, and Cedar latency.

## Build Notes
Add database migration for binding, decision, simulation, and violation tables.
Add binding resolver by workflow name, node name, tenant, and date.
Add payload schema validator for principal, resource, and context versions.
Add decision input canonicalizer and hash utility.
Add Cedar client adapter with enforce and simulation modes.
Add REST handlers for binding create, simulate, evaluate, list, and decision read.
Add gRPC handlers for evaluate and simulate.
Add workflow node adapter used by treasury workflows.
Add contract tests for missing binding, schema invalid, deny branch, and replay mismatch.
Add workflow tests binding IP-017 payment release and IP-020 hedge approval nodes.
Add load fixture with 10000 decisions per minute target.
Add migration fixture with SAP release strategy export.
Add dashboard panels for decision latency, deny rate, missing binding count, and Cedar errors.
