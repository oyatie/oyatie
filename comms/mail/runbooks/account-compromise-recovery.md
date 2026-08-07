---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0243, ADR-0297]
companion_docs: [microservices/mail/policy/anti-phishing.cedar]
inbound_citations: [microservices/mail/ARCHITECTURE.md]
---

# Runbook: Mail account compromise recovery

## A. Trigger conditions

- HIBP credential match for an active mailbox user.
- Anomalous geo / device login per anti-phishing classifier.
- User self-report via support channel.

## B. Pre-checks

1. Verify operator Cedar permit `oya.mail.account-compromise-respond`.
2. Confirm user identity via out-of-band channel (recovery phone, support escalation).
3. Capture compromise indicators (IP, UA, timestamp).

## C. Procedure

1. **Force sign-out everywhere.** `oya mail session-revoke --user <id>`; emits `oya.mail.session-revoke-all`. Timing ≤30s.
2. **Disable IMAP/POP3 app-passwords.** `oya mail app-password-revoke --user <id> --all`; emits `oya.mail.app-password-revoke`.
3. **Step up auth.** Enforce WebAuthn passkey on next sign-in via `oya mail auth-policy-set --user <id> --require WEBAUTHN_PASSKEY`.
4. **HIBP check.** Confirm password not in dump; if it is, force reset.
5. **Recent outbound audit.** Query audit chain for `oya.mail.outbound-send` events in the last 30 days from this user; surface to user for review.
6. **Forwarding rule audit.** Inspect Sieve filters for auto-forward rules; show to user; remove if not user-authored.
7. **Mailbox-rule audit.** Inspect any new mailbox-creation, label-creation, or auto-archive rules; remove if not user-authored.
8. **OAuth grant audit.** List third-party apps with mailbox access; require user confirmation; revoke unrecognized.
9. **Notify per ADR-0263.** Emit `oya.mail.account-compromise-recovery-complete`.
10. **If PHI mailbox.** Cross-reference `runbooks/phi-leak-recovery.md`.

## D. Verification

- No active sessions other than the one re-authenticated post-recovery.
- Sieve filters audited.
- Audit-chain shows recovery sequence.

## E. Rollback

User can restore audited-and-removed filters via the recovery dashboard within 30 days (stored under audit-evidence).

## F. Post-incident

Track HIBP-match → compromise rate; surface to security KPIs.

## G. References

- `policy/anti-phishing.cedar`
- `policy/abuse-defence.cedar`
- ADR-0297
