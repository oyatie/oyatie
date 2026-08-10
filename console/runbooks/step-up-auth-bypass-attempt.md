---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0243
  - ADR-0263
  - ADR-0297
companion_docs:
  - microservices/ops-dashboard-control-center/incident-response.md
  - microservices/ops-dashboard-control-center/compliance.md
  - console/runbooks/admin-mfa-cascade.md
  - console/runbooks/forensic-investigation-handoff.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: Step-Up Auth Bypass Attempt

## A — Trigger conditions

- `StepUpAuthFailed` audit event rate > 5/min for a single principal (alert: `oya_ops_control_center_step_up_auth_failures_total{principal_id=~".+"} > 5` in 60s window).
- A mutation was attempted without a valid step-up session (Cedar `AdminActionDenied` with reason `step_up_expired_or_missing`).
- UEBA insider-risk score for an operator exceeds 80 (alert from detection µservice).
- `TenantScopeViolationDetected` event co-occurs with step-up failures (lateral movement pattern).

## B — Pre-checks

1. **[≤30s]** Check alert context: `GET /ops/v1/audit/events?filter=StepUpAuthFailed&principal={id}&window=5m` → get raw event list.
2. **[≤30s]** Check if operator session is currently active: `oya-bao token lookup --accessor {accessor}`.
3. **[≤30s]** Check UEBA score: `GET /ops/v1/detection/ueba/{operator_id}/score` → record current score.
4. **[≤15s]** Determine if this is: (a) legitimate expired session, (b) credential stuffing, (c) insider threat, or (d) automation bug.

## C — Procedure

### Path A — Legitimate expired session (operator locked out)

1. **[≤60s]** Contact operator via verified out-of-band channel (phone / Slack DM to verified account).
2. **[≤2min]** If confirmed legitimate: issue new step-up challenge via `POST /ops/v1/auth/step-up/challenge` with operator identity.
3. **[≤5min]** Operator completes step-up on their registered FIDO2 device.
4. Audit event `StepUpAuthCompleted` confirms success.

### Path B — Credential stuffing / possible account takeover

1. **[≤15s]** IMMEDIATELY revoke all active sessions for that operator:
   ```
   POST /ops/v1/auth/sessions/revoke-all
   Body: { "operator_id": "<id>", "reason": "suspected_credential_compromise" }
   Headers: X-Step-Up-Token: <your_T3_token>
   ```
2. **[≤30s]** Force password reset + HIBP check: notify Identity µservice via `POST /identity/v1/operators/{id}/force-reset`.
3. **[≤5min]** Notify operator via verified channel (NOT email if email may be compromised; use phone or manager chain).
4. **[≤5min]** Preserve evidence: `GET /ops/v1/audit/events?filter=StepUpAuthFailed&principal={id}&window=1h` → save to evidence ticket.
5. Escalate to `runbooks/forensic-investigation-handoff.md` if UEBA score > 80.

### Path C — Insider threat pattern

1. **[≤15s]** Revoke all sessions (as Path B step 1).
2. **[≤5min]** Escalate IMMEDIATELY to council-security via dedicated escalation channel.
3. **[≤5min]** Preserve session recording if T3 session was active: `GET /ops/v1/session-recordings/{session_id}` (requires `oyatie.ops.forensics` principal).
4. Do NOT notify the operator until council-security advises — notification may compromise investigation.
5. Follow `runbooks/forensic-investigation-handoff.md` for chain-of-custody.

### Path D — Automation bug (CI/tooling retry loop)

1. **[≤30s]** Identify automation source from `audit_event.principal_id` and `audit_event.user_agent`.
2. **[≤2min]** Disable the automation: revoke its service account token or kill the pipeline.
3. File bug with automation team.
4. Audit trail is sufficient evidence; no escalation required unless > 1000 attempts/hour (DoS risk).

## D — Verification

- `oya_ops_control_center_step_up_auth_failures_total` returns to 0/min for affected principal.
- `GET /ops/v1/auth/sessions/{operator_id}` → confirms no active sessions (paths B/C) OR one valid session (path A).
- UEBA score check: `GET /ops/v1/detection/ueba/{operator_id}/score` → score declining (detection µservice feedback loop).

## E — Rollback

This runbook does not modify production data. Rollback is: re-issue step-up challenge for legitimate operator (path A). No data rollback required.

## F — Post-incident

- SOC 2 CC6.1 control evidence: preserve `StepUpAuthFailed` events + remediation audit chain.
- If credential stuffing confirmed: add IP range to edge WAF block list in `iac/prod-edge-waf.yaml`.
- Review: should the affected operator's step-up class be upgraded (e.g., from TOTP to hardware key)?
- Update UEBA baseline if this was a false positive.

## G — References

- `policy/cedar/step-up-auth-required.cedar`
- `ARCHITECTURE.md §abuse-defence`
- `compliance.md §insider-threat-controls`
- `runbooks/admin-mfa-cascade.md`
- `runbooks/forensic-investigation-handoff.md`
