---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-security + ops-sre-reliability
---

# Incident Response — identity µservice

Playbooks for the highest-blast-radius incident classes specific to the identity µservice. Generic SRE response (paging, sev classification, comms) follows the company-wide IR plan; this document covers identity-specific actions.

## Sev definitions

| Sev | Trigger | SLA |
|---|---|---|
| Sev-1 | Identity µservice down in ≥1 pack OR signing key compromise OR mass credential exfil suspected | ≤5min ack, ≤30min mitigation |
| Sev-2 | Token-issuance latency > 5× p99 OR JWKS rotation failed OR SCIM endpoint 5xx>1% sustained | ≤15min ack, ≤2h mitigation |
| Sev-3 | Single-tenant disruption OR aaguid-refresh > 48h stale OR HRIS poll backed up | ≤1h ack, ≤4h mitigation |

## Incident #1 — Leaked JWT signing key

**Detection**: OpenBao access-log anomaly; security partner / red-team report; abnormal token verification rate from unknown egress IP; HSM tamper alarm.

**Immediate actions** (≤15min):
1. Rotate JWT signing key in OpenBao (`oya cloud-secrets rotate identity-<pack>-jwt-signing`) — adds new `kid` to JWKS, marks old key for sunset.
2. Force JWKS endpoint to expire cache (`Cache-Control: no-cache` for 24h).
3. Push new JWKS to all consumers via the JWKS-rotation broadcast event.

**Containment** (≤2h):
4. Revoke all in-flight tokens issued under the old `kid` by adding `kid` to the OIDC introspection deny-list.
5. Force re-authentication of all active sessions (set `acr_event_at=0` server-side).
6. Push notice to status page; communicate to tenants via the per-tenant emergency channel.

**Recovery** (≤24h):
7. Pen test re-validates posture.
8. Audit-chain forensics: enumerate every token-issuance event under the compromised `kid`; cross-reference Cedar deny audits for anomalous use.
9. If proof of exfiltration: GDPR Art. 33 (72h breach notification); HIPAA breach assessment.

**Postmortem**: blameless RCA within 7 days; ADR proposal if architectural fix needed.

## Incident #2 — Compromised tenant admin (account takeover)

**Detection**: SCIM activity from unusual IP/geo; brute-force counter on tenant-admin OIDC login; phishing report from tenant.

**Immediate** (≤15min):
1. Identify the tenant admin's user ID (X-Scope-OrgID + claim audit).
2. Force-revoke all sessions for the user (`oya identity user revoke-sessions <tenant> <user>`).
3. Disable WebAuthn credentials (`oya identity webauthn-disable <user> --all`).
4. Pin the user to `acr=critical` for next re-auth (forces IT-approval gate).
5. Audit-trail: enumerate every action taken by this user in the last 7 days.

**Containment** (≤2h):
6. Cross-tenant action audit: did this user perform any cross-tenant API calls (Cedar policy should have denied; verify)?
7. SCIM lifecycle scan: did this user provision/escalate any other accounts?
8. Group membership audit: revoke any unusual group memberships.

**Recovery** (≤24h):
9. Tenant identity-proofing: out-of-band verification of the admin's identity (call known phone; check via known IdP federation).
10. New Passkey + hardware key registration after IT-approval gate.
11. Quarter-end review of access patterns.

**Postmortem**: tenant CISO notification; offer credit monitoring if PII export was performed.

## Incident #3 — Mass passkey reset event (e.g., authenticator vendor recall, AAGUID compromised)

**Detection**: FIDO-MDS3 marks an AAGUID as `REVOKED` or `ATTESTATION_KEY_COMPROMISE`.

**Immediate** (≤30min):
1. Remove the compromised AAGUID from the allowlist for all regulated packs.
2. Enumerate affected credentials (`SELECT credential_id, tenant_id, user_id FROM webauthn_credentials WHERE aaguid = $1`).
3. Mark all affected credentials as `revoked=true`; refuse them at next assertion.

**Communication** (≤2h):
4. Per-pack tenant notice: list of affected user counts; advise re-registration with a different authenticator.
5. Per-affected-user notice: "Your security key (model XYZ) was withdrawn by its manufacturer; please add a new Passkey."

**Containment** (≤24h):
6. Force step-up to `sensitive` for all affected users until they register a non-revoked credential.
7. Account-recovery path for users whose ONLY credential was the revoked one: human-mediated operator flow (audit-trailed).

**Recovery** (within 30 days):
8. Drive credential re-registration via in-app prompt + email.
9. Cull stale credentials after 30 days.

## Incident #4 — SCIM provisioning loop / runaway operation

**Detection**: SCIM POST/PATCH rate >100x p99 from a single bearer; tenant complains "users keep flipping active/inactive."

**Immediate** (≤15min):
1. Identify the offending SCIM bearer (audit emit `IdentityScimRequestReceived`).
2. Hard-throttle the bearer to 10 req/min via dynamic Envoy rate-limit override.
3. Contact tenant IT to ask if a bad SCIM config push is in progress.

**Containment** (≤1h):
4. If the upstream IdP is misconfigured (e.g., Okta SCIM target URL wrong): coordinate with tenant IT to pause provisioning.
5. Quiesce SCIM operations on the affected tenant for 30min.
6. Roll back any unintended mass operations (`oya identity scim revert <tenant> <since-time>`).

**Recovery** (≤8h):
7. Resume SCIM with fixed upstream config.
8. Reconcile with upstream IdP (full SCIM resync).
9. Postmortem with tenant IT.

## Incident #5 — Upstream federated IdP outage (Google Workspace, Okta, Entra)

**Detection**: OIDC discovery to upstream returns 5xx; sign-in failures spike from federated tenant.

**Immediate** (≤15min):
1. Identify affected tenants (`oya identity federation list-affected <upstream-idp>`).
2. Switch federated tenants to fallback mode: direct WebAuthn at Zitadel (if user has a registered Passkey, bypass federation).
3. Communicate to tenants: "Your upstream IdP is degraded; affected users may sign in directly to oyatie identity with their Passkey."

**Containment** (≤4h):
4. Monitor upstream status page.
5. If upstream resolves: bring federation back; verify token exchange end-to-end.

**Recovery** (post-resolution):
6. Audit: enumerate sign-ins that used fallback path; confirm no PII drift.
7. Postmortem if outage > 4h (cross-vendor RCA).

## Incident #6 — Edge authz drift (a Cedar policy starts blocking what edge should block, or vice versa)

**Detection**: `oya-check-authz-tier-discipline` advisory lane reports findings; OnCall page from contradictory denies.

**Immediate** (≤30min):
1. Identify the policy file with the violation.
2. Open a revert PR to remove the misplaced concern.
3. Re-run `oya-check-authz-tier-discipline` to confirm clean.

**Containment** (≤2h):
4. Add unit test for the boundary violation (regression).
5. Push the revert through admission gate.

**Recovery**: post-PR.

## Incident #7 — Audit-chain seal backlog

**Detection**: audit-emit-completeness SLO < 1.0 sustained > 5min; events queued in emitter buffer.

**Immediate** (≤15min):
1. Verify audit-chain µservice is reachable (`oya audit-chain health`).
2. Drain emitter buffer in priority order: critical events first (StepUpGranted, OidcTokenIssued).
3. If audit-chain is down: enter degraded mode — emit to local WAL until audit-chain recovers; replay after.

**Containment**: emitter retries with exponential backoff + DLQ.

**Recovery**: post audit-chain recovery; verify Merkle continuity.

## Incident #8 — HRIS poller infinite-loop or duplicate provisioning

**Detection**: `IdentityHrisHirePulled` event rate >10× baseline; same `employee_id` provisioned N times.

**Immediate** (≤30min):
1. Pause HRIS poller for affected tenant.
2. Identify duplicate users; soft-delete duplicates.
3. Verify HRIS adapter idempotency on the affected vendor.

**Containment** (≤4h):
4. Patch adapter idempotency check (the `external_id` MUST be the dedup key).
5. Reconcile with HRIS.

**Recovery**: HRIS resumes; postmortem with vendor SDK note.

## Communication matrix

| Sev | Internal | Customer | Regulator |
|---|---|---|---|
| Sev-1 | ops-on-call + axis-identity + council-architecture + council-compliance | per-tenant emergency channel ≤30min; status page ≤15min | GDPR Art. 33 (72h) for PII breach; HIPAA Breach Rule (60d); KR PIPC for pack-kr |
| Sev-2 | ops-on-call + axis-identity | status page if customer-visible | only if regulator-triggering threshold met |
| Sev-3 | axis-identity | per-tenant if scoped to a tenant | none unless regulator-triggering |

## Forensic data retention

- Per-incident: full audit-chain export for the affected window + 24h before + 24h after.
- Tagged in evidence store: `evidence/incident-<id>-identity-<pack>-<date>.tar.zst`.
- Retention: 7 years (SOC 2 + HIPAA + KR PIPA-FSS sector).
