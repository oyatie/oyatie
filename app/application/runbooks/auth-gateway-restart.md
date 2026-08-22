---
doc_class: Runbook
title: Auth-Gateway Restart — sign-in flow recovery
microservice: application
severity: "Sev-1 / Sev-2 depending on scope"
status: Accepted
owner_team: ops-security + axis-application
date: 2026-05-17
related_artifacts:
  - microservices/application/failure-modes.md (FM-05, FM-12, FM-14, FM-15)
  - microservices/application/incident-response.md
  - microservices/application/threat-model.md (S-01..S-04, T-04, T-06, E-03)
doc_status: published
---

# Runbook: Auth-Gateway Restart

## Trigger

ANY of:

1. **FM-05 OIDC IdP outage** — `application_oidc_idp_error_rate > 5%` for ≥ 1 min.
2. **FM-12 cookie scope misconfiguration** — lane / runtime probe alerts on `Domain=.oyatie.dev` drift.
3. **FM-14 SAML XSW attempt** — `application_saml_xsw_block_total > 0` (security event; no service action required, but escalate).
4. **FM-15 Cedar policy compile regression** — auth-gateway fails to start.
5. **Manual** — operator declares restart for session HMAC key rotation.

## Severity

- IdP outage: **Sev-2** (existing sessions OK; new sign-in fails).
- Cookie scope drift: **Sev-1** (cross-product cookie risk).
- Policy compile regression: **Sev-1** (service unavailable).
- HMAC rotation (planned): **Sev-3** (no impact if rolled correctly).

## Pre-checks

1. Identify which failure class (Pre-trigger above).
2. Confirm scope (one pack vs. all packs vs. one replica).
3. Check OpenBao reachability (secret fetches go through OpenBao).

## Steps — IdP outage fallback

| Step | Action | Time budget |
|---|---|---|
| 1 | Confirm IdP is the failure source: probe IdP `well-known/openid-configuration`; check IdP vendor status page | ≤ 5 min |
| 2 | If tenant has SAML fallback configured (per tenancy resolver), Application Shell auto-routes new sign-ins through SAML — no action required | n/a |
| 3 | If no fallback: status-page update; tenant comm; escalate to IdP vendor | ≤ 30 min |
| 4 | Existing sessions remain valid (cached JWKS); monitor for forced re-sign-in (session expiry) | ongoing |
| 5 | When IdP recovers: synthetic OIDC probe to confirm; monitor sign-in rate | ≤ 15 min |
| 6 | Postmortem capturing IdP RCA from vendor | ≤ 10 BDs |

## Steps — Cookie scope rotate (after FM-12 drift detected)

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Sev-1 (ops-security IC) | ≤ 5 min |
| 2 | Verify drift: probe `Set-Cookie` headers from production ingress | ≤ 2 min |
| 3 | Roll forward correct Helm value: PR with `cookieDomain: .app.oyatie.dev` (NOT `.oyatie.dev`) | ≤ 10 min |
| 4 | Rotate session HMAC key (invalidates all sessions): `cargo run -p dev-cli -- application auth rotate-session-hmac --pack <pack>`. CLI: (a) generates new key in OpenBao; (b) marks old key as decrypt-only with 5-min sunset; (c) signals auth-gateway worker to refresh. | ≤ 5 min |
| 5 | Verify deploy: probe `Set-Cookie` returns correct `Domain` | ≤ 5 min |
| 6 | CommsLead: forced-re-sign-in tenant comm (planned-feeling phrasing) | ≤ 30 min |
| 7 | If breach: PrivacyLead initiates GDPR / PIPA / etc. notification | per timeline |
| 8 | Postmortem; action item: why did drift reach deploy? | ≤ 5 BDs |

## Steps — Cedar policy revert

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Sev-1 | ≤ 5 min |
| 2 | Identify the PR that landed the regression | ≤ 5 min |
| 3 | `git revert <pr-sha>` on `release/application/production`; expedite through SLO gate (operator-approved fast-path with audit) | ≤ 10 min |
| 4 | Deployment rolls; verify auth-gateway pods enter Ready | ≤ 5 min |
| 5 | Synthetic sign-in probe | ≤ 5 min |
| 6 | CommsLead: status page resolved | ≤ 30 min |
| 7 | Postmortem within 5 BDs; action item: tighten policy-compile lane gate | – |

## Steps — Planned HMAC rotation

| Step | Action | Time budget |
|---|---|---|
| 1 | Announce planned re-sign-in window 24 h ahead | ≤ 24 h pre |
| 2 | Execute `rotate-session-hmac --pack <pack>` during low-traffic window | ≤ 5 min |
| 3 | Verify; on-call monitor sign-in success | ≤ 1 h |

## Verification

- `application_oidc_signin_success_rate > 99%` for ≥ 15 min.
- `application_saml_signin_success_rate > 99%` for ≥ 15 min.
- `application_session_create_p99_seconds <= 0.2` for ≥ 15 min.
- No `Set-Cookie` drift on probes.

## References

- `failure-modes.md` FM-05, FM-12, FM-14, FM-15.
- `threat-model.md` STRIDE S-01..S-04 (auth threats).
- ADR-0123 cross-product auth contract.
