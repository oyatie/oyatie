---
doc_class: Runbook
title: Credential rotation — provider vendor key / subscription cookie
microservice: foundry-providers
severity: "Sev-3 (planned) / Sev-1 (emergency — suspected compromise)"
status: Accepted
owner_team: ops-security
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/policy/credential-isolation.md (CI-INV-07 rotation without downtime)
  - microservices/intelligence/policy/openbao-credential.cedar (rotation runner permits)
  - microservices/intelligence/threat-model.md (T-01 credential theft)
  - microservices/intelligence/runbooks/provider-credentials-revoke.md
doc_status: published
---

# Runbook: Credential rotation

## Trigger

ONE of:

1. **Planned rotation** — quarterly per vendor per tenant per pack; coordinated by ops-security.
2. **Emergency rotation** — suspected compromise (T-01 in threat-model.md); credential observed in logs/chat/git/error message; OpenBao audit log shows unexpected resolution pattern.
3. **Vendor-mandated** — vendor announces key-rotation requirement.

## Severity

- Planned: **Sev-3** (no incident; document + execute).
- Emergency (compromise suspected): **Sev-1** (page; declare incident immediately; PrivacyLead engaged if data-class boundary may have been crossed).

## Pre-checks

1. Confirm 2-person authorization for the rotation: a `RotationRunner` principal must hold at least 2 approvals (per `policy/openbao-credential.cedar` PERMIT 2 and `policy/credential-isolation.md` CI-INV-09).
2. Confirm overlap-window configuration: default 60 s; adapter prefers new credential as soon as observed.
3. Confirm vendor permits two-credential-simultaneous: Anthropic API yes; OpenAI API yes; Gemini API yes; subscription transports — see vendor-specific section below.
4. Pull the current request rate for the tenant+vendor combination: `oya_foundry_providers_provider_invocations_total{tenant="<t>",vendor="<v>"}[5m]`; confirm we can complete rotation in the overlap window.

## Steps (API transports — Anthropic / OpenAI / Gemini)

| Step | Action | Time budget |
|---|---|---|
| 1 | If emergency: open `#inc-<id>`; assign IC; PrivacyLead engaged | ≤ 5 min |
| 2 | Verify 2-person approval recorded in OpenBao audit log | ≤ 2 min |
| 3 | Generate new credential at the vendor portal (Anthropic console / OpenAI dashboard / Google Cloud console) | ≤ 5 min |
| 4 | Write the new credential to OpenBao at the **next** version of the SecretReference path: `openbao://<pack>/<tenant>/providers/<vendor>/<credential-name>?version=N+1`. Use `cargo run -p oya-dev-cli -- providers rotate --tenant <t> --vendor <v> --credential <name>` (2-person signed; OpenBao audit-emitted) | ≤ 3 min |
| 5 | Confirm adapter pods observe the new version: `kubectl exec <adapter-pod> -- curl -s localhost:9090/internal/credential-state | jq '.[].latest_version'` returns `N+1` | ≤ 30 s after step 4 |
| 6 | Wait for overlap window to elapse (default 60 s) so any in-flight requests using version N complete | ≤ 60 s |
| 7 | Revoke version N at the vendor portal | ≤ 3 min |
| 8 | Confirm zero failed requests during the overlap window: `oya_foundry_providers_provider_invocations_total{status="error"}[<rotation-window>]` returns 0 (or only unrelated errors) | ≤ 2 min |
| 9 | Emit `CredentialRotated` audit-chain event (auto-emitted by the CLI) | – |
| 10 | If emergency: file postmortem within 5 business days; identify how the credential was compromised | per priority |

## Steps (Subscription transports — Claude Pro/Max / ChatGPT Plus / Gemini Advanced)

Subscription channels do not natively support two-cookie-simultaneous. The procedure is therefore:

| Step | Action | Time budget |
|---|---|---|
| 1 | Notify tenant operator of brief subscription-channel downtime (≤ 5 min) | ≤ 15 min advance |
| 2 | Pause router traffic to the subscription transport for this tenant: `cargo run -p oya-dev-cli -- providers pause --tenant <t> --vendor <v> --transport subscription --reason "<id>"`. Router routes to alternate (e.g., API transport if available, or alternate vendor) | ≤ 1 min |
| 3 | Re-authenticate the subscription at the vendor portal (manual cookie capture by ops-security; 2-person witness) | ≤ 5 min |
| 4 | Write the new cookie to OpenBao at the next version | ≤ 3 min |
| 5 | Confirm adapter pods observe the new version | ≤ 30 s |
| 6 | Resume router traffic: `cargo run -p oya-dev-cli -- providers resume --tenant <t> --vendor <v> --transport subscription` | ≤ 1 min |
| 7 | Old session cookie invalidated by re-auth automatically | – |
| 8 | Emit `CredentialRotated` audit-chain event | – |

If subscription downtime is unacceptable for the tenant, encourage migration to the API transport per `runbooks/adapter-version-pin.md` discussion.

## Rollback (of the rotation)

If new credential is rejected by the vendor:
1. Confirm credential format + scope at vendor portal.
2. Re-generate; restart from step 3 with a fresh credential.
3. If overlap window elapsed and rotation cannot complete: revert to version N at the vendor portal; revoke version N+1; restart.

If overlap-window violation observed (failed requests during rotation):
1. Investigate adapter pod-observability — was version N+1 observed by all replicas?
2. If clock skew or stale-config detected: extend overlap window; document in postmortem.

## Verification

- `oya_foundry_providers_credential_rotated_total{tenant="<t>",vendor="<v>"}` incremented by 1.
- `tests/integration/rotation_zero_downtime.rs` passes against staging cluster (continuous-drill).
- Quarterly rotation evidence: `evidence/runbook-drills/rotation/<unix_ts>.json` recorded.
- For emergency rotations: postmortem identifies root cause + closes the leak path.

## References

- `policy/credential-isolation.md` CI-INV-07.
- `policy/openbao-credential.cedar`.
- `threat-model.md` T-01.
- OpenBao versioned-secrets docs.
- Vendor-specific key-rotation docs (Anthropic / OpenAI / Google).
