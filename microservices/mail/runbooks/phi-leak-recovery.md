---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0251]
companion_docs: [microservices/mail/policy/phi-dlp.cedar]
inbound_citations: [microservices/mail/ARCHITECTURE.md]
---

# Runbook: PHI leak recovery (HIPAA breach response)

## A. Trigger conditions

- `policy/phi-dlp.cedar` FORBID escaped (PHI sent to non-attested recipient).
- HIPAA-eligible mailbox compromised (credential reuse, lateral movement).
- Auditor escalation citing HHS Breach Notification Rule applicability.

## B. Pre-checks

1. Verify operator Cedar permit `oya.mail.phi-incident-respond` + PHI_OFFICER role.
2. Confirm tenant `audience_type=B2B_HIPAA_PHI` + BAA on file.
3. Capture incident scope (affected mailboxes, message-ids, recipients).

## C. Procedure

1. **Containment.** Freeze affected mailbox: `oya mail mailbox-freeze --tenant <id> --mailbox <id> --reason phi-incident`. Emits `oya.mail.mailbox-freeze`. Timing ≤60s.
2. **Quarantine in-flight outbound.** Halt outbound queue: `oya mail outbound-halt --tenant <id>`. Emits `oya.mail.outbound-halt`.
3. **Enumerate exposure.** Query audit chain for `oya.mail.outbound-send` events with `phi_detected=true` since suspected compromise; tabulate recipients.
4. **Legal hold.** Engage legal-hold on affected mailbox via `runbooks/legal-hold-engage.md`; chain-of-custody timestamp from TrueTime per ADR-0252.
5. **Recipient notification (HHS Breach Rule).** Within 60 days, notify each affected individual. Within 60 days if ≥500 records, also notify HHS + media in affected state per 45 CFR §164.408. Emit `oya.mail.phi-breach-notify`.
6. **Tenant BAA review.** Escalate to compliance officer for BAA addendum review.
7. **Forensics.** Pull all SPIFFE-attested access logs for the mailbox (per ADR-0295); cross-reference with abuse-defence outcomes.
8. **Credential rotation.** Rotate tenant encryption-key BYOK keys via `runbooks/e2e-encryption-key-recovery.md` (ADR-0251 §D-10); force re-auth for all mailbox sessions.
9. **DLP rule update.** If a Cedar gap was the root cause, add a new rule to `policy/phi-dlp.cedar`; soak 60s per ADR-0294.
10. **Postmortem.** Within 72h, populate `docs/runbooks/postmortem-template.md` with root cause + remediation + ADR amendments.

## D. Verification

- No further `oya.mail.outbound-send{phi_detected=true}` to non-attested recipients.
- Mailbox-freeze active; outbound-halt active until cleared.
- HHS notification submitted (if applicable).
- Tenant BAA addendum signed.

## E. Rollback

Unfreeze mailbox after forensics + tenant-admin acknowledgement + BAA addendum.

## F. Post-incident

File ADR amendment if doctrinal gap; update PHI-DLP training data.

## G. References

- `policy/phi-dlp.cedar`
- `runbooks/legal-hold-engage.md`
- ADR-0251 compliance pack primitive
- 45 CFR §164.408 (HHS Breach Notification Rule)
