---
doc_class: Runbook
title: DKIM key rotation (per-tenant signing key lifecycle)
microservice: mail
severity: "Sev-3 (planned) / Sev-1 (compromised key — emergency)"
status: Accepted
owner_team: axis-mail + ops-deliverability + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/mail/threat-model.md (T-S-02 DKIM key compromise, T-T-04 signing-key tampering)
  - microservices/mail/policy/data-residency.md
  - microservices/mail/contracts/openapi.yaml §"/v1/tenants/{tenantId}/dkim/rotate"
  - ADR-0133 cross-tenant mail-server pattern
  - ADR-0140 (retired per ADR-0145) Cedar policy enforcement
doc_status: published
---

# Runbook: DKIM key rotation

## Purpose

DKIM (RFC 6376) signs every outbound message with a tenant-specific private key. The public key is published as DNS TXT under `<selector>._domainkey.<tenant-domain>`. Keys MUST rotate per `policy/data-residency.md` schedule (≤ 90 days; emergency immediate on compromise) per ADR-0133 §"per-tenant signing key lifecycle". This runbook covers planned + emergency rotation, with dual-publish overlap to preserve verifiability of in-flight mail.

CI lane `oya-governance-dkim-key-rotation-conformance` refuses any DKIM key older than 90 days; this runbook is the operator procedure that satisfies that lane.

## Trigger

| Trigger | Severity | Owner |
|---|---|---|
| Scheduled 90-day rotation (cron `0 3 * * 0` weekly sweep) | Sev-3 | axis-mail + ops-deliverability |
| Key compromise suspected (e.g., HSM intrusion alert; OpenBao breach indicator) | Sev-1 | ops-security |
| Tenant offboarding (revoke key before deprovision) | Sev-3 | ops-onboarding |
| DKIM selector exhaustion (rare; > 8 historic selectors retained) | Sev-3 | axis-mail |
| Audit finding (rotation overdue) | Sev-2 | ops-security |

## Pre-checks

| # | Check | Command / source |
|---|---|---|
| 1 | Current selector + age | `kubectl exec -n mail <pod> -- oya-mail-cli dkim list --tenant=<t>` |
| 2 | Tenant domain ownership verified (recent SPF/DMARC/MX records confirm tenant control) | `dig +short TXT <tenant-domain>` |
| 3 | DNS publish path operational (Route53 / Cloudflare / OCI DNS API reachable) | `kubectl exec <pod> -- oya-mail-cli dns probe --tenant=<t>` |
| 4 | OpenBao key path accessible | `bao kv get secret/mail/<tenant>/dkim/<selector>` (mTLS) |
| 5 | Recent outbound volume (so we can compute dual-publish overlap window) | `oya_mail_outbound_message_volume_total{tenant_id=<t>}[7d]` |
| 6 | Active inbound replies expected to in-flight messages (estimates verifiability needs) | estimate from prior-window response rate |

## Steps — Planned rotation (Sev-3)

| Step | Action | Time |
|---|---|---|
| 1 | Open ChangeSet via `oya vcs claim --agent <id> --intent "dkim-rotate:<tenant>:<rfc>" --paths "microservices/mail/evidence/dkim/<tenant>/**"`. | ≤ 2 min |
| 2 | Generate new keypair: Ed25519 (RFC 8463) + 2048-bit RSA fallback. `kubectl exec <pod> -- oya-mail-cli dkim genkey --tenant=<t> --selector=<new-selector> --algo=ed25519+rsa2048` | ≤ 2 min |
| 3 | Store private key in OpenBao under `secret/mail/<tenant>/dkim/<new-selector>` with KMS-wrap; access-policy = `mail-outbound-signer` role only. | ≤ 2 min |
| 4 | Publish public-key DNS TXT: `<new-selector>._domainkey.<tenant-domain>  IN  TXT  "v=DKIM1; k=ed25519; p=<base64>"` (Ed25519); analogously for RSA fallback as separate selector `<new-selector>-rsa`. | ≤ 5 min |
| 5 | Verify DNS propagation: `kubectl exec <pod> -- oya-mail-cli dkim verify-dns --tenant=<t> --selector=<new-selector>` returns ✅ from ≥ 8 global resolvers (Google, Cloudflare, Quad9, OpenDNS, Verisign, Comodo, AdGuard, OCI). | ≤ 15 min (depends on TTL) |
| 6 | Promote new selector to `active_selector` in `oya-mail-outbound-smtp-app` config: `oya-mail-cli dkim activate --tenant=<t> --selector=<new-selector>`. From this moment, NEW outbound mail signs with new selector. | ≤ 1 min |
| 7 | Mark old selector `retired_at = now`. Old selector REMAINS in DNS for overlap window (default 14 days; longer for tenants with high outbound volume to ensure delivery receipts + DSNs verify). | ≤ 1 min |
| 8 | Emit audit-chain `DkimKeyRotated{tenant_id, old_selector, new_selector, rotated_at, reason}` (Ed25519 sealed). | automatic |
| 9 | Notify tenant: tenant admin email + tenant deliverability dashboard banner "DKIM key rotated <date>; new selector `<new>`; old selector retired in 14 days". | ≤ 30 min |
| 10 | Schedule decommission: 14 days from rotation, the worker `dkim-decommission` removes old selector TXT record + revokes OpenBao key + emits `DkimKeyDecommissioned`. | T+14d automatic |
| 11 | `oya vcs done` ChangeSet; evidence at `microservices/mail/evidence/dkim/<tenant>/<rotation-id>.json`. | ≤ 2 min |

## Steps — Emergency rotation (Sev-1; compromise)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-sec-<id>`; engage ops-security IC. | immediate |
| 2 | Pause outbound submission for the tenant (`oya-mail-cli outbound pause --tenant=<t> --reason="suspected-dkim-compromise"`). | ≤ 2 min |
| 3 | Run planned-rotation Steps 2-6 (generate + publish + activate new selector). | ≤ 15 min |
| 4 | Revoke old selector IMMEDIATELY: remove TXT record from DNS; delete OpenBao key path; emit `DkimKeyRevokedEmergency` audit-chain event with full forensic context (who, when, source IP, attack vector). | ≤ 5 min |
| 5 | Resume outbound submission once new selector confirmed live in DNS + first test send verifies. | ≤ 30 min |
| 6 | Forensic: in-flight mail signed with the compromised key — assume any such mail since the suspected compromise window is potentially fraudulent. Notify recipients per `incident-response.md` §"Phishing-Adjacent" template. | ≤ 4 h |
| 7 | Tenant comms: status-page entry + tenant admin email + per-pack regulator notification IF the compromise affected PHI/PII per pack overlay (HIPAA 60-day breach window; GDPR 72-hour notification; KR PIPA 72-hour notification). | per pack |
| 8 | Postmortem within 24 hours (Sev-1 SLA). | – |

## Steps — Tenant offboarding rotation

When a tenant offboards (per `tenancy` µservice's offboarding workflow):

| Step | Action |
|---|---|
| 1 | At offboarding T+0: pause outbound submission; emit `MailTenantOffboardingStarted`. |
| 2 | At T+30 days (mailbox retention window): rotate DKIM key (final-rotation; new selector); old selector remains in DNS to verify any DSNs that arrive late. |
| 3 | At T+90 days: revoke DKIM key entirely; remove TXT records; OpenBao key deleted; emit `MailTenantDecommissioned`. |
| 4 | Audit-chain retention of these events: ≥ 6y for HIPAA-pack tenants; ≥ 5y for KR-pack-FSS; ≥ 1y otherwise. |

## Verification

After rotation completes:
- DNS verify ✅ from ≥ 8 resolvers (Step 5 above).
- First outbound test message signed with new selector — DKIM-Signature header contains `s=<new-selector>`.
- Recipient MX verifies signature: send test to `dkimtest@mail-tester.com` (or `dkim-validator.com`); score = 10/10.
- `oya_mail_outbound_dkim_sign_total{tenant_id=<t>,selector="<new>"}` > 0; old selector counter flat.
- Audit-chain seal: `kubectl exec <pod> -- oya-mail-cli audit verify --tenant=<t> --event=DkimKeyRotated` returns ✅.
- 7 days post-rotation: `oya_mail_outbound_dkim_verification_failure_total{tenant_id=<t>}[7d]` ≤ 0.01% (no recipient infrastructure rejecting new key).
- 14 days post-rotation: old selector decommissioned automatically (Step 10 verified); DNS no longer publishes old TXT.

## Post-incident updates

- If emergency (Sev-1): root-cause the compromise vector; update `threat-model.md` if new T-* identified; review HSM/OpenBao access logs.
- If planned recurring overdue findings: tighten `oya-governance-dkim-key-rotation-conformance` lane warning thresholds (e.g., warn at 60 days, block merge at 90).
- Verify ARC chain (RFC 8617) continuity is not broken across rotation — ARC sets are signed with the OUTGOING key at message-creation time; rotations don't retroactively invalidate.
- Update `policy/data-residency.md` if a per-pack regulator changed the rotation cadence requirement.

## References

- RFC 6376 (DKIM Signatures) — `https://datatracker.ietf.org/doc/html/rfc6376`
- RFC 8463 (Ed25519 for DKIM) — `https://datatracker.ietf.org/doc/html/rfc8463`
- RFC 8617 (ARC) — `https://datatracker.ietf.org/doc/html/rfc8617`
- RFC 7208 (SPF) — alignment context — `https://datatracker.ietf.org/doc/html/rfc7208`
- RFC 7489 (DMARC) — alignment context — `https://datatracker.ietf.org/doc/html/rfc7489`
- ADR-0133 (cross-tenant per-tenant signing key lifecycle)
- ADR-0140 (Cedar policy enforcement; signer role authority)
- `microservices/mail/threat-model.md` T-S-02, T-T-04
- `microservices/mail/policy/data-residency.md` §"DKIM key rotation cadence per pack"
- OpenBao docs — `https://openbao.org/docs/`
- M3AAWG Sender Best Common Practices v3 (key rotation) — `https://www.m3aawg.org`
