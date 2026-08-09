---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0243
  - ADR-0263
  - ADR-0296
companion_docs:
  - microservices/ops-dashboard-control-center/incident-response.md
  - microservices/ops-dashboard-control-center/runbooks/step-up-auth-bypass-attempt.md
  - microservices/ops-dashboard-control-center/policy/cedar/step-up-auth-required.cedar
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: Admin MFA Cascade

## A — Trigger conditions

- Mass step-up session expiry event: ≥20 operators' step-up sessions expire within the same 5-min window (e.g., after a planned maintenance window or OpenBao restart).
- UEBA detects bulk concurrent step-up failures across multiple principals (correlated with insider-risk alert).
- OpenBao transit key rotation causes bulk session token invalidation.
- Planned: org-wide hardware key rotation campaign requires all T3 sessions to re-authenticate.

## B — Pre-checks

1. **[≤30s]** Determine scope: `GET /ops/v1/auth/sessions/expiry-forecast?window=15m` → list operators with imminent expiry.
2. **[≤30s]** Check if this is planned (OpenBao rotation, maintenance) or unplanned.
3. **[≤30s]** Check current on-call queue depth: `GET /ops/v1/oncall/handoffs/current` — are there active incidents requiring T3 mutations?
4. **[≤30s]** Check OpenBao health: `oya-bao status`.

## C — Procedure

### Planned cascade (maintenance / key rotation)

1. **[≤30min before]** Notify all operators via `#ops-all-hands` Slack: "Step-up sessions will expire at [TIME]. Re-auth required for T2/T3 actions."
2. **[≤5min before]** Ensure no T3 mutations are in-flight: `GET /ops/v1/actions?state=pending&tier=T3`.
3. **[At expiry]** Operators re-authenticate via their FIDO2 devices.
4. **[≤15min after]** Verify all active T3 sessions re-established: `GET /ops/v1/auth/sessions/active-count?tier=T3`.
5. **[≤5min]** Confirm any incident command sessions re-established with correct scope.

### Unplanned cascade

1. **[≤5min]** Identify root cause: OpenBao outage? Network partition? Clock skew?
2. **[≤5min]** If OpenBao restart: verify OpenBao unsealed: `oya-bao status` → `Sealed: false`.
   If still sealed: invoke Shamir unseal quorum per `docs/runbooks/openbao-seal-recovery.md`.
3. **[≤10min]** Notify operators to re-authenticate.
4. **[≤5min]** If active incident requires T3 mutation during cascade:
   - Emergency T3 session issued by `oyatie.ops.admin-console` system principal (requires council-security approval).
   - Emit `AdminMfaCascadeEmergencySessionIssued` audit event.
5. **[≤15min]** Confirm recovery via session-active-count check (step C-planned step 4).

## D — Verification

- `GET /ops/v1/auth/sessions/active-count` → matches expected operator count.
- No `StepUpAuthFailed` events in last 5 min: `oya_ops_control_center_step_up_auth_failures_total == 0`.
- Active incidents unblocked: T3 mutations resuming normally.

## E — Rollback

If emergency T3 sessions were issued: revoke them immediately post-incident:
`POST /ops/v1/auth/sessions/revoke-all?tier=EMERGENCY_T3`

## F — Post-incident

- Review: was the cascade preventable with longer step-up TTLs? (Trade-off: security vs UX.)
- If OpenBao restart was root cause: add OpenBao health to pre-maintenance checklist.
- If planned: improve operator notification lead time.

## G — References

- `policy/cedar/step-up-auth-required.cedar`
- `ARCHITECTURE.md §credential-isolation`
- `compliance.md §key-rotation-cadence`
- `runbooks/step-up-auth-bypass-attempt.md`
