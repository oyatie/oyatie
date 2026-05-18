---
doc_class: Runbook
runbook_id: identity-scim-provisioning-debug
microservice: identity
sev: Sev-3
owner_team: axis-identity
date: 2026-05-18
---

# Runbook: SCIM provisioning debug

## Symptoms

- SCIM operations return 4xx/5xx for a specific tenant or bearer.
- Tenant IT reports "users not provisioning" or "lifecycle propagation broken."
- `IdentityScimRequestReceived` event rate dropped to 0 for a known-active tenant.

## Diagnostics

1. **Bearer check**:
   - `oya identity scim bearer-status --tenant <tenant>` — verify bearer not expired/revoked.
   - Check OpenBao `secret/identity/<pack>/<tenant>/scim-bearer` for rotation history.

2. **Recent request log**:
   - `oya identity scim recent-requests --tenant <tenant> --since 1h` — pull recent 100 SCIM ops.
   - Look at status codes, scimType field, and timestamps.

3. **Upstream IdP connectivity**:
   - From Okta admin console: System Log → SCIM provisioning logs → outbound to oyatie endpoint.
   - From Entra: Provisioning logs → outbound errors.

4. **Schema validation**:
   - Replay one failing request manually: `oya identity scim replay-single --request-id <id>`.
   - Compare against `microservices/identity/contracts/openapi/identity.yaml#paths./scim/v2/{tenant}/Users`.

## Common failure modes + fixes

| Symptom | Cause | Fix |
|---|---|---|
| 401 on every request | Bearer expired or rotated | Provision new bearer; update upstream IdP SCIM client config |
| 400 InvalidSyntax | Upstream IdP sends invalid JSON | Vendor-dialect quirk — file ticket with vendor; meanwhile add quirk-handling per IP-008 §"Dialect quirks" |
| 409 Uniqueness on every POST | Tenant has duplicate-userName-import problem | Tenant IT must resolve upstream duplicate before SCIM resumes |
| 429 Too Many | SCIM POST rate too high | Increase tenant quota OR upstream IdP must throttle |
| 500 Internal | Zitadel admin API issue | Check Zitadel pod logs; if pod down, restart per zitadel-restart runbook |
| `IdentityScimRequestReceived` not emitting | Audit emitter degraded | See `identity-audit-emit-backlog` runbook |
| Cross-tenant data appearing | Configuration bug — bearer scoped wrong | URGENT: revoke bearer immediately; rotate; reconcile per `tenant-admin-onboard` |

## Mitigation

- If a specific bearer is being abused (10x rate): hard-throttle to 10 rps via Envoy dynamic override.
- If upstream IdP is sending malformed payloads: contact tenant IT; pause provisioning until config fixed.
- If bearer leaked: revoke immediately + rotate + audit cascade.

## Verification

- `oya identity scim health --tenant <tenant>` — green status.
- Replay one mutating op end-to-end; verify Zitadel side-effect + audit event.

## Postmortem trigger

If outage > 4h or affected tenants > 5, schedule postmortem within 7 days.
