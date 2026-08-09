---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0263
companion_docs:
  - microservices/ops-dashboard-control-center/incident-response.md
  - microservices/ops-dashboard-control-center/policy/cedar/tenant-scope-enforcement.cedar
  - microservices/ops-dashboard-control-center/runbooks/forensic-investigation-handoff.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: Tenant Scope Violation Detected

## A — Trigger conditions

- `TenantScopeViolationDetected` audit event emitted (any occurrence triggers alert — zero tolerance).
- Cedar `FORBID` verdict on `tenant_isolation_posture_query` or `evidence_pack_read` with `resource.tenant_id NOT IN principal.scoped_tenants`.
- Prometheus alert: `oya_ops_control_center_tenant_scope_violations_total > 0` (SLO `tenant-isolation-visibility` SLO budget consumed).

## B — Pre-checks

1. **[≤30s]** Pull violation event: `GET /ops/v1/audit/events?filter=TenantScopeViolationDetected&window=10m`.
2. **[≤30s]** Determine: was the FORBID honored (action blocked) or was there a policy engine miss (action succeeded but logged)? Check `audit_event.action_succeeded == false`.
   - If `action_succeeded == true`: this is a CRITICAL BREACH — escalate immediately to council-security.
   - If `action_succeeded == false`: Cedar correctly blocked the attempt; still investigate principal.
3. **[≤30s]** Identify principal: `audit_event.principal_id`, `audit_event.principal_role`.
4. **[≤30s]** Identify target tenant: `audit_event.resource.tenant_id`. Determine if this is a high-value tenant (payments, healthcare).

## C — Procedure

### Path A — Cedar correctly blocked (action_succeeded = false)

1. **[≤2min]** Check if this is a legitimate bug (operator had wrong scope in their token vs intended scope):
   - Pull operator's `scoped_tenants` from their OpenBao session token: `oya-bao token lookup --accessor {accessor}`.
   - If scope is correct for their role: this may be a scripting error or UI bug. File bug ticket.
   - If scope is WRONG: their token was issued with incorrect scope. Revoke + re-issue.
2. **[≤5min]** Notify affected tenant (if the pivot was toward a high-value tenant) via transparency audit event.
3. Review: was this an insider-threat probe? Check UEBA score: `GET /ops/v1/detection/ueba/{operator_id}/score`.
4. If UEBA > 80: escalate to Path C (insider threat).

### Path B — CRITICAL: action succeeded despite scope violation

1. **[≤15s]** PAGE council-security IMMEDIATELY.
2. **[≤15s]** Revoke ALL sessions for the offending principal: `POST /ops/v1/auth/sessions/revoke-all`.
3. **[≤2min]** Determine what data was accessed: `GET /ops/v1/audit/events?principal={id}&window=1h`.
4. **[≤5min]** Notify affected tenant's primary contact via secure channel.
5. **[≤1h]** Begin GDPR/KR-PIPA breach notification assessment (72h clock may have started).
6. **[≤1h]** Preserve all evidence: follow `runbooks/forensic-investigation-handoff.md`.

### Path C — Insider threat pattern

1. Revoke sessions (Path A step 1 action / Path B step 2).
2. Preserve session recordings: `GET /ops/v1/session-recordings/{session_id}`.
3. Escalate to council-security with full audit chain.
4. Coordinate with HR per insider-threat policy.

## D — Verification

- `oya_ops_control_center_tenant_scope_violations_total` returns to 0 after containment.
- `GET /ops/v1/cedar/policy-bundle/version` → confirms policy bundle is current (not stale/cached incorrectly).
- RLS synthetic probe: `GET /ops/v1/health/rls-probe` → `cross_tenant_probe: pass`.

## E — Rollback

If data was accessed (Path B):
1. Notify affected tenant.
2. Determine if any data was exfiltrated (check egress audit events).
3. Data cannot be "un-accessed" — breach notification process per `compliance.md §pack-overlay-roster`.

## F — Post-incident

- This is a SOC 2 CC6.6 (logical access) control failure — document in SOC 2 evidence.
- Root cause: Cedar policy bug? Token issuance bug? UI routing bug? Address root cause before re-enabling operator.
- Regression test: add Cedar policy test for the violated permit/forbid path.

## G — References

- `policy/cedar/tenant-scope-enforcement.cedar`
- `ARCHITECTURE.md §tenant-scoping`
- `tenant-isolation.md`
- `runbooks/forensic-investigation-handoff.md`
