---
doc_class: Runbook
title: Provider credentials — emergency revoke
microservice: foundry-providers
severity: "Sev-1 (confirmed credential compromise) / Sev-2 (suspected)"
status: Accepted
owner_team: ops-security
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/policy/credential-isolation.md
  - microservices/intelligence/runbooks/credential-rotation.md
  - microservices/intelligence/threat-model.md (T-01 credential theft)
doc_status: published
---

# Runbook: Provider credentials — emergency revoke

## Trigger

ONE of:

1. **Confirmed compromise** — credential observed outside the OpenBao isolation boundary (logs / chat / git / external pastebin / vendor breach notification).
2. **Suspected compromise** — anomalous OpenBao resolution pattern; spike in `oya_foundry_providers_credential_resolutions_total` outside normal envelope; OR observed cross-tenant request shape.
3. **Vendor-mandated revocation** — vendor breach notification or credential-format change.

## Severity

- Confirmed compromise: **Sev-1** (page; PrivacyLead engaged; regulatory-notification clock starts per GDPR Art. 33 — 72 h for personal-data breach).
- Suspected only: **Sev-2** (page during business hours; investigation).

## Pre-checks

1. Confirm whether tenant data may have been exposed: review `ProviderInvoked` event history for the affected (tenant, vendor, credential-ref) window; identify data classes touched.
2. Confirm 2-person authorization for the revoke (per `policy/openbao-credential.cedar`).
3. Identify the affected SecretReference path: `openbao://<pack>/<tenant>/providers/<vendor>/<credential-name>`.
4. Identify all tenant workloads that depend on this credential.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | If Sev-1: open `#inc-<id>`; IC + PrivacyLead + ops-security + axis-foundry | ≤ 5 min |
| 2 | Verify 2-person approval | ≤ 2 min |
| 3 | Revoke at the vendor: log in to vendor console; revoke the key/cookie; verify revocation via vendor's audit log (Anthropic: console → API keys → revoke; OpenAI: dashboard → API keys → revoke; Google: console → IAM → API keys → revoke) | ≤ 5 min |
| 4 | Remove the credential from OpenBao: `cargo run -p oya-dev-cli -- providers credential-revoke --tenant <t> --vendor <v> --credential <name> --reason "<id>" --approver <p1> --approver <p2>` (2-person signed; audit-emitted). The CLI: (a) marks the SecretReference path as REVOKED in OpenBao, (b) flushes lease cache in affected pods, (c) emits `CredentialRevoked` event | ≤ 3 min |
| 5 | Confirm adapter pods no longer hold a live lease: `kubectl exec <adapter-pod> -- curl -s localhost:9090/internal/credential-state | jq '.[] | select(.path | contains("<credential-name>"))'` returns empty | ≤ 30 s |
| 6 | Issue a replacement credential per `runbooks/credential-rotation.md` (mark as Sev-1-derived rotation) | ≤ 10 min |
| 7 | Notify tenant operator: credential compromised + revoked + replaced; data-class-touched summary if applicable | ≤ 30 min |
| 8 | If personal-data breach suspected: PrivacyLead opens regulatory-notification process per `compliance.md` §"Breach notification" + `incident-response.md` | ≤ 72 h per GDPR Art. 33 |
| 9 | Postmortem within 5 business days; identify how the credential leaked; close the leak path | per priority |

## Rollback

Revocation is non-reversible — once revoked, the credential is dead at the vendor and at OpenBao. If the revocation was a mistake:
1. Issue a fresh replacement credential.
2. Postmortem: how did the false-positive arise?

## Verification

- Credential confirmed revoked at vendor portal (audit-trail screenshot or vendor-emitted notification).
- `oya_foundry_providers_credential_revoked_total{tenant="<t>",vendor="<v>"}` incremented by 1.
- Tenant workload functional on the replacement credential.
- `evidence/runbook-drills/credential-revoke/<unix_ts>.json` recorded for the drill (quarterly).
- For Sev-1: regulatory-notification audit-chain record present (filing reference + DPO sign-off).

## Post-incident updates

- Close the leak path identified in postmortem.
- If credential leaked because of a logging / chat / git regression: harden `oya-check-no-raw-credentials` regex set; add the specific leak pattern as a permanent regression test.
- If pre-emption was insufficient: update `credential-isolation.md` invariants.

## References

- `microservices/intelligence/policy/credential-isolation.md`.
- `microservices/intelligence/runbooks/credential-rotation.md`.
- `microservices/intelligence/threat-model.md` T-01.
- GDPR Art. 33 (72 h personal-data breach notification).
- HIPAA §164.408 (breach notification — pack-us-healthcare).
