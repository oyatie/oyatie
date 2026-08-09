---
doc_class: Runbook
title: E2E encryption key recovery (Personal-pillar S/MIME + OpenPGP)
microservice: mail
severity: "Sev-3 (single-user planned) / Sev-2 (user-locked-out) / Sev-1 (key compromise)"
status: Accepted
owner_team: axis-mail + council-privacy + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/mail/PRD.md Open Question 4 (personal-pillar key recovery)
  - microservices/mail/policy/dual-context-isolation.md Invariant DCI-03
  - microservices/mail/threat-model.md (T-T-03 personal-DEK escrow, T-S-05 user passphrase loss)
  - microservices/mail/dpia.md (R-06 personal-context confidentiality)
  - registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery (target; placeholder until ADR lands)
  - RFC 8551 (S/MIME 4.0), RFC 9580 + RFC 4880 (OpenPGP)
doc_status: published
---

# Runbook: E2E encryption key recovery

## Scope + load-bearing invariant

This runbook covers **Personal-pillar** E2E key recovery. Personal mailbox blobs are encrypted under a **user-derived DEK** per `policy/dual-context-isolation.md` Invariant DCI-03:

- DEK wrapping key derived from user passphrase + per-user salt (PBKDF2-HMAC-SHA256, 600 000 iterations; transition to Argon2id documented in registry/placeholder-debt/adr-follow-ups.yaml#passphrase-derivation-upgrade).
- Wrapped DEK stored in KMS with access scope = `subject == user.user_id`.
- **Org admin cannot decrypt user's personal-pillar mailbox** (Invariant DCI-03; CI lane `personal-pillar-kms-scope` enforces).
- Default recovery model at M03 launch (per PRD Open Question 4): **user-held-only with QR-code paper recovery**.

Professional-pillar mailboxes use tenant DEK (not user-derived); their recovery flow is the standard tenant DEK rotation (see `dkim-key-rotation.md` reference and `tenancy` µservice runbooks).

S/MIME (RFC 8551) and OpenPGP (RFC 9580 / 4880) signing/encryption keys are user-held for personal-pillar; signing-cert chain anchored at the user's CA preference (Let's Encrypt S/MIME pilot, Actalis free S/MIME, or self-signed).

## Trigger

| Trigger | Severity | Owner |
|---|---|---|
| User forgot passphrase; needs DEK recovery via QR paper backup | Sev-3 | user-self + axis-mail oncall (advisor only) |
| User device lost AND QR backup lost — total key loss | Sev-2 | axis-mail + user-self (advisory + outcome documentation) |
| User reports key compromise (device theft, malware indicator) | Sev-1 | ops-security + user-self |
| Hardware token (YubiKey / Nitrokey) failed | Sev-3 | user-self |
| Pack-mandated escrow recovery (only future ADR-decided mode; NOT M03 default) | Sev-3 | council-privacy + tenant ops-legal (BAR: only applies if user opted into escrow at onboarding) |

## Pre-checks

| # | Check | Source |
|---|---|---|
| 1 | Confirm user identity via second factor (recovery email, SMS, hardware-token presence challenge, or video KYC if high-value) | tenant onboarding KYC method |
| 2 | Confirm Personal-pillar mailbox exists for the user | `oya-mail-cli mailbox list --user=<u> --context=Personal` |
| 3 | Confirm context: Personal-pillar — **org admin path is FORBIDDEN regardless of org policy** per Invariant DCI-04 (legal-hold inapplicable; eDiscovery inapplicable; admin override unavailable by construction) | confirmed by Cedar policy `forbid (...)` |
| 4 | Confirm recovery mode the user enrolled in at mailbox creation: QR-paper-only (M03 default), QR-paper + hardware-token, QR-paper + escrow-with-2-person-rule (future opt-in) | from `mailbox.recovery_modes` metadata |
| 5 | If escrow mode: confirm escrow holders (must be 2 distinct OIDC subjects with `recovery_holder` entitlement) | from escrow ledger |

## Path A — User has QR paper backup + remembers passphrase OR has new passphrase ready

Standard recovery; no decryption-by-anyone-other-than-user.

| Step | Action | Time |
|---|---|---|
| 1 | User runs recovery client (web or mobile): scans QR backup → enters via UI → derives wrapping key | ≤ 5 min |
| 2 | Client unwraps DEK locally; can now decrypt mailbox blobs | ≤ 1 min |
| 3 | If user wants to ROTATE the passphrase: client generates new salt + new wrapping key; re-wraps DEK; uploads new wrapped-DEK to KMS (`PUT /v1/personal/dek/wrap`); old wrap is replaced | ≤ 2 min |
| 4 | Audit-emit `PersonalDekRecovered{user_id, recovery_method=qr_paper, recovered_at}` (Ed25519 sealed by user's session) | automatic |
| 5 | Old QR backup invalidated; user prompted to generate + print a new QR backup | ≤ 5 min |

## Path B — User lost device AND QR backup (total loss; no escrow)

Outcome: **mailbox is unrecoverable**. This is the M03 design tradeoff (user-held-only) per PRD Open Question 4.

| Step | Action | Time |
|---|---|---|
| 1 | Confirm with user that no QR backup, no second device, no hardware-token holds the keys | ≤ 10 min |
| 2 | Engage council-privacy to document the irrecoverable loss | ≤ 30 min |
| 3 | Offer user the choice: delete the inaccessible mailbox (default; data destroyed cryptographically) OR retain encrypted blobs indefinitely in case key materialises (storage cost to user; rarely useful) | ≤ 30 min |
| 4 | On delete: emit `PersonalMailboxCryptoErased{user_id, reason=key_loss, executed_at}` Ed25519-sealed; remove KMS-wrapped DEK; orphaned ciphertext aged out by retention worker within 7 days | ≤ 1 d |
| 5 | User can re-create a new Personal mailbox at the same address (or new) with a fresh key pair | ≤ 5 min |
| 6 | Post-incident: update onboarding flow to emphasise QR backup importance; consider opt-in escrow recommendation for future users | ≤ 1 wk |

## Path C — Hardware token (YubiKey / Nitrokey) failed

Cause: User uses hardware token for unwrap; token hardware failed.

| Step | Action | Time |
|---|---|---|
| 1 | Confirm token failure (cannot enumerate via WebAuthn / PIV / OpenPGP card) | ≤ 5 min |
| 2 | If user enrolled a BACKUP token at onboarding: switch to backup token; new token derives same wrapping key from same passphrase | ≤ 10 min |
| 3 | If no backup token: Path B applies UNLESS user also has QR paper backup (then Path A) | ≤ 10 min |
| 4 | Replace primary token + enrol fresh backup; rotate passphrase | ≤ 30 min |

## Path D — Suspected compromise (Sev-1)

Cause: User reports device theft, malware, or unauthorised access.

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security; open `#inc-sec-<id>` | immediate |
| 2 | Force-rotate the wrapping key: server-side, mark KMS-wrapped DEK as `compromised`; user must re-wrap with NEW passphrase + new salt | ≤ 5 min |
| 3 | Lock the mailbox for read/write until re-wrap completes | ≤ 5 min |
| 4 | User performs Path A with new passphrase + new QR backup | per user |
| 5 | Forensic: did anyone DECRYPT before the rotation? Audit KMS access log for wrap/unwrap events in suspect window. Any unauthorised access → notify per GDPR Art. 33 / PIPA Art. 34 timelines | ≤ 4 h |
| 6 | If S/MIME or OpenPGP signing key compromise: REVOKE certificate via CRL/OCSP (S/MIME) or upload revocation cert to keyservers (OpenPGP RFC 9580 §5.2.3.23). User must re-issue + re-distribute new public key. | ≤ 2 h |
| 7 | Update IMAP/JMAP session tokens; force re-auth on all sessions | ≤ 5 min |
| 8 | Audit-emit `PersonalDekCompromiseRotation{user_id, suspected_at, rotated_at}` Ed25519-sealed | automatic |
| 9 | Postmortem with user; recommend hardware token + paper backup if not already in place | per user |

## Path E — Escrow recovery (future opt-in mode; NOT M03 default)

This path is scheduled-for-distinct-tracked-work to registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery. Documented here for completeness; not operative at M03 launch.

| Step | Action | Notes |
|---|---|---|
| 1 | Confirm user opted into escrow at mailbox creation | from `mailbox.recovery_modes` |
| 2 | Confirm 2 distinct escrow holders + their entitlements | from escrow ledger |
| 3 | Both holders provide Ed25519-signed unwrap-request | both signatures required |
| 4 | KMS releases wrapping-key share fragments to escrow holders | audit-emit `PersonalDekEscrowReleased` |
| 5 | Combined fragments + KDF derive the wrapping key | client-side |
| 6 | Same as Path A Step 2-5 from there | – |

## Verification

After completion (any path except Path B "destroyed"):
- User can read + send mail from Personal mailbox via IMAP/JMAP/REST.
- New QR backup generated + confirmed (user must acknowledge).
- KMS-wrapped DEK is the new one (old wrap revoked).
- All audit events sealed + visible in user's audit log view.
- If S/MIME / OpenPGP: new public-key fingerprint published; old cert revoked; user re-distributed new key per their preference (keyserver + email signature footer).
- Threat-model T-T-03 + T-S-05 acceptance: a non-user principal cannot decrypt this mailbox (Cedar deny + KMS scope verified).

## Post-incident updates

- If Path B (total loss) recurs ≥ 3× / quarter: revisit PRD Open Question 4; escrow opt-in default may be warranted; registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery target.
- If Path D (compromise) shows pattern: update threat-model with new attack signature.
- Annual external pen-test must include "can org admin decrypt a Personal mailbox?" — must fail (i.e., no decryption path); failure-to-fail blocks pen-test sign-off.
- Update `policy/dual-context-isolation.md` if invariant DCI-03 requires refinement.

## References

- RFC 8551 (S/MIME 4.0)
- RFC 9580 (OpenPGP) and RFC 4880 (predecessor)
- RFC 5598 (Internet Mail Architecture)
- PBKDF2 / Argon2 / KDF references: NIST SP 800-132; OWASP Password Storage Cheat Sheet
- GDPR Arts. 32 + 33; KR PIPA Art. 28 + 34; HIPAA §164.312(a)(1) + §164.410
- ePrivacy Directive 2002/58/EC Art. 5 (e-mail confidentiality)
- `microservices/mail/PRD.md` Open Question 4
- `microservices/mail/policy/dual-context-isolation.md` Invariant DCI-03
- `microservices/mail/threat-model.md` T-T-03, T-S-05
- `microservices/mail/dpia.md` R-06
- Bominal ADR-0208 personal-pillar policy
- WebAuthn L3 — `https://www.w3.org/TR/webauthn-3/`
- ProtonMail key recovery model (precedent) — `https://proton.me/support/recover-encrypted-messages-files`
