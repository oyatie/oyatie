---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0243
  - ADR-0263
companion_docs:
  - console/runbooks/incident-command.md
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
  - console/runbooks/forensic-investigation-handoff.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: Admin Action Rollback

## A — Trigger conditions

- An admin action (deployment approve, rollback execute, tenant-isolation-policy-change) was applied with incorrect parameters.
- Operator requests rollback within the undo-window (15 min default; configurable per tenant).
- Automated anomaly detection flagged the action as suspicious post-execution.
- Cedar audit chain shows a PERMIT was granted in error (e.g., policy fragment bug).

## B — Pre-checks

1. Confirm the action is within rollback eligibility window: `GET /ops/v1/actions/{action_id}/rollback-eligibility` → budget ≤10s.
2. Confirm your step-up auth is active (T3 hardware key, ≤1h window): check `oyatie.ops.dashboard-operator` session in OpenBao UI or `oya-bao token lookup`.
3. Verify audit seal ref for original action: `GET /ops/v1/actions/{action_id}/audit-seal` → must return `sealed: true`.
4. If rolling back a deployment approve: confirm the downstream GitOps controller has NOT yet applied the change (`argocd app get {app} --show-operation`). ≤2 min.
5. Alert `#ops-incidents` Slack channel that rollback is in progress.

## C — Procedure

1. **[≤30s]** Issue rollback intent with idempotency key:
   ```
   POST /ops/v1/actions/{action_id}/rollback
   Headers: X-Idempotency-Key: <uuid4>  X-Step-Up-Token: <token>
   Body: { "rationale": "<reason>", "requested_by": "<operator_id>" }
   ```
   Expected: `202 Accepted` with `rollback_ticket_id`.
   If `409 Conflict`: rollback already in progress — proceed to step 4.
   If this step fails: escalate to `runbooks/forensic-investigation-handoff.md`.

2. **[≤60s]** Monitor rollback outbox drain:
   ```promql
   oya_ops_control_center_outbox_pending_total{action_type="rollback"} == 0
   ```
   Timeout: 60s. If not drained: check Kafka consumer lag `oya_ops_control_center_kafka_consumer_lag`.

3. **[≤30s]** Verify compensating event emitted in audit chain:
   ```
   GET /ops/v1/actions/{action_id}/audit-chain
   ```
   Expected: event `AdminActionRolledBack` with `sealed: true` and `compensating_action_id` set.
   If missing: STOP — escalate to council-security. Audit chain break is SEV1.

4. **[≤60s]** For deployment rollbacks: verify ArgoCD sync status reverted:
   ```
   argocd app get {app_name} --show-operation
   ```
   Expected: `Health: Healthy`, `Sync: Synced` to pre-rollback revision.
   If `OutOfSync`: trigger manual sync `argocd app sync {app_name}`.

5. **[≤30s]** For tenant-isolation-policy-change rollbacks: verify Cedar policy bundle refreshed:
   ```
   GET /ops/v1/cedar/policy-bundle/version
   ```
   Expected: version returned matches pre-change bundle hash.

6. **[≤30s]** Update incident ticket with rollback completion + audit-seal ref.

7. **[≤5s]** Emit `AdminActionRollbackCompleted` event (automatic on step 3 success, verify manually if step 3 required escalation).

## D — Verification

- `GET /ops/v1/actions/{action_id}` → `state: ROLLED_BACK`.
- `GET /ops/v1/actions/{action_id}/audit-chain` → chain contains both `AdminActionExecuted` and `AdminActionRolledBack` events, both sealed.
- SLO: `oya-ops-control-center-operator-action-audit-completeness` still at 1.0 (no unsealed events).

## E — Rollback of this rollback

If the rollback itself was applied in error:
1. Issue a forward-apply with the original parameters via normal admin action workflow.
2. Both the original action AND the rollback are preserved in the audit chain (append-only); no deletion.
3. Forensic chain: original → rollback → re-apply. All three sealed.

## F — Post-incident

- File blameless post-mortem if rollback was triggered by a policy bug (Cedar fragment error).
- Update Cedar fragment test set with regression test for the failing permit.
- Review: was the step-up auth class sufficient for this action's risk level?
- SLO error-budget impact: check `slos/command-availability.openslo.yaml` burn rate.

## G — References

- `ARCHITECTURE.md §rollback-path`
- `runbooks/forensic-investigation-handoff.md`
- `policy/cedar/admin-action-authorization.cedar`
- `contracts/openapi/ops-dashboard-control-center.yaml` → `POST /ops/v1/actions/{id}/rollback`
