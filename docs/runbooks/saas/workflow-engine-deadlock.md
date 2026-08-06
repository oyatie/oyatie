---
doc_status: published
---

# Oyatie Runbook — Workflow Engine Deadlock

> **Status:** Production procedure authored for the M03-P04/M03-P08 SaaS operator-documentation gate; readiness remains `target_non_claim` until changeset evidence and `oya-ci-required` are green.
> **Owner:** `axis-saas`
> **Severity scope:** Sev 2 by default; escalate to Sev 1 for cross-cell, regulated-pack, or revenue-impacting workflow stalls.
> **Authority:** ADR-0035 workflow engine semantics, the SaaS Platform PRD, and M03-P04/M03-P08 planning references in `specs/masterplan.json`.
> **Last verified:** 2026-06-09 (SSOT chain checked against HANDOFF.md, registry/stores/*, specs/root-hub-pointers.json, specs/masterplan.json, and docs/products/saas-platform/PRD.md).

## Operator contract
- **Incident channel:** `#inc-saas-workflow-engine`.
- **Primary invariant:** never edit or replay a sealed step in place. ADR-0035 requires sealed steps to remain immutable evidence; recovery uses state-vector restore, explicit compensation, or a new workflow-definition version.
- **Tenant boundary:** every action is scoped by `tenant_id`, `cell_id`, `workflow_id`, and `workflow_version`.
- **Cloud authority:** runtime mitigation is applied through the cloud control-plane / Kubernetes cell for the affected tenant workload. Workstation diagnostics are supporting evidence only and are not merge, production, or hyperscaler authority.
- **Audit event:** every containment, replay, compensation, and release decision emits `EVT-SAAS-WORKFLOW-DEADLOCK-INCIDENT` with `incident_id`, `tenant_id`, `cell_id`, `workflow_id`, `workflow_version`, `operator_id`, `decision_id`, and `evidence_hash`.
- **Stop condition:** affected workflow backlog is draining, no new stuck instances appear for three evaluator windows, the audit event is sealed, and the post-incident prevention ticket has an owner.

## Trigger conditions
- `saas_workflow_stuck_instances{state!="terminal"}` exceeds the per-tenant threshold for two evaluator windows.
- `saas_workflow_transition_lag_seconds` breaches the SaaS charter workflow SLO for a tenant, cell, or regulated pack.
- Workflow instances remain in `Waiting`, `Compensating`, or `HumanApproval` beyond their ADR-0035 timer/SLA.
- Support, tenant admin, or a downstream axis reports tenant-visible workflow stalls.
- The cross-axis contract fitness lane for SaaS↔Cloud, SaaS↔Search, or SaaS↔Agent-runtime detects a workflow handoff that cannot advance.

## First-response checklist
1. Acknowledge the page in `#inc-saas-workflow-engine`; assign incident commander and scribe.
2. Record `INCIDENT_ID`, `TENANT_ID`, `CELL_ID`, `WORKFLOW_ID`, and `WORKFLOW_VERSION` before changing state.
3. Snapshot queue depth, transition lag, state-vector hashes, and audit-chain seal status for the affected window.
4. Determine blast radius: single workflow instance, single tenant, tenant cohort, cell, or cross-cell.
5. Apply the narrowest containment branch below; prefer tenant/workflow quarantine over fleet rollback.
6. Emit the containment audit event before resuming traffic or replaying state.

## Containment
- **Single definition:** freeze new instance creation for the affected `workflow_id@workflow_version`; let unrelated workflow definitions continue.
- **Single tenant:** pause the tenant's affected workflow definition and keep other tenants on the same definition version running.
- **Cross-tenant definition drift:** block promotion of the definition version, pin tenants to the last known-good version, and open founder-governed architecture review if the cross-axis contract changed.
- **Saga participant outage:** stop new transitions that call the failing capability, keep completed sealed steps immutable, and queue compensations behind the participant recovery gate.
- **Regulated-pack impact:** page privacy/compliance on-call and preserve all state-vector/audit evidence before replay.

## Diagnosis
Classify exactly one primary branch before recovery:

| Branch | Evidence | Required check |
|---|---|---|
| Idempotency-key collision | duplicate transition attempts with the same idempotency key | Verify the transition guard and idempotency namespace include tenant, workflow, version, state, and step. |
| Outbox or eventing backpressure | transition state is ready but outbound events are delayed | Check eventing backlog and audit-chain emit latency before replaying workflow state. |
| Saga participant unavailable | workflow waits on a capability or downstream axis | Confirm the capability registry route, downstream SLO, and compensation definition. |
| Human-approval SLA miss | pending approval exceeds timer | Verify approver identity, Cedar decision, notification delivery, and escalation policy. |
| Definition or overlay drift | only one workflow version / jurisdiction overlay stalls | Diff the compiled definition, regional overlay, and cross-axis contract version. |
| Audit evidence delay | user symptoms recover but audit event is missing | Treat as unresolved; audit-chain integrity is part of recovery. |

## Recovery
1. Keep the containment branch active while fixing the classified root cause.
2. If the workflow definition is defective, publish a new definition version; do not mutate the in-flight version.
3. If replay is required, restore from the last non-sealed state vector and re-run only non-sealed steps.
4. If external state was mutated, run ADR-0035 saga compensations in reverse order and seal each compensation event.
5. If the incident is caused by a downstream capability, keep workflow intake paused until that capability's owner provides green SLO evidence.
6. Re-enable workflow intake gradually by tenant or definition version; do not release a cross-tenant fleet switch first.
7. Add or update the prevention gate that would have caught the branch: idempotency regression, definition-drift fitness, outbox lag guard, approval-SLA monitor, or audit-seal assertion.

## Verify recovery
- `saas_workflow_stuck_instances` returns below the Sev 2 threshold for three evaluator windows.
- `saas_workflow_transition_lag_seconds` and workflow execution p99 are back within the SaaS charter SLO.
- No affected workflow remains in `Waiting`, `Compensating`, or `HumanApproval` past its timer.
- Audit-chain contains sealed `EVT-SAAS-WORKFLOW-DEADLOCK-INCIDENT` containment and resolution events.
- The affected tenant can start a canary workflow on the corrected definition version.
- Cross-axis contract fitness passes for any SaaS↔Cloud, SaaS↔Search, or SaaS↔Agent-runtime handoff touched by the incident.

## Rollback guardrails
- Do not delete stuck workflow rows; use compensating transitions or explicit terminal failure states.
- Do not replay sealed steps.
- Do not widen from tenant-level to fleet-level mitigation without incident commander approval and current blast-radius evidence.
- Do not close the incident if audit-chain evidence is delayed or missing.
- Do not mark workstation checks as production authority; cloud control-plane / Kubernetes cell status and sealed audit evidence are required.

## Post-incident
- Author the postmortem within the Sev 2 SLA from `docs/INCIDENT-MANAGEMENT.md`.
- Add a prevention row to `docs/MISTAKES-LEDGER.md` or the active prevention ledger with the mechanical gate owner.
- Update this runbook if the classified branch, metric, or prevention gate was missing.
- Reference the M03-P04/M03-P08 implementation-plan IDs when the fix changes workflow runtime or cross-axis contract behavior.

## Sources
`docs/products/saas-platform/PRD.md`, `docs/teams/axis-saas/CHARTER.md`, `specs/masterplan.json` M03-P04/M03-P08 entries, `docs/decisions/ADR-0700-ci-admission-live-apex.md`, `docs/INCIDENT-MANAGEMENT.md`, `docs/SLO-CATALOG.md`, `docs/standards/prevention-doctrine.md`.
