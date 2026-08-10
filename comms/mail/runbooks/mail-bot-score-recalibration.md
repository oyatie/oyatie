---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0297]
companion_docs: [comms/mail/policy/abuse-defence.cedar]
inbound_citations: [microservices/mail/ARCHITECTURE.md]
---

# Runbook: Mail bot-score recalibration

## A. Trigger conditions

- False-positive rate on `policy/abuse-defence.cedar` > 0.5% on JMAP / IMAP / web client.
- UX-floor violation: legitimate users seeing CAPTCHA on regular sign-in.
- Adversary bot-farm signature shift.

## B. Pre-checks

1. Verify operator Cedar permit `oya.mail.abuse-defence-tune`.
2. Pull last 24h `oya.mail.abuse-defence-block` events; tabulate by `audience_type`, route, fingerprint.

## C. Procedure

1. **Diagnose.** Identify class: substrate calls blocked / legitimate IMAP clients blocked / web-sign-up false-positive.
2. **Substrate.** Verify SPIFFE workload identity + `audience_type=INTERNAL_SUBSTRATE` propagated.
3. **Legacy IMAP.** Add JA4 allow-list for known IMAP-client fingerprints (Thunderbird, Apple Mail, Outlook desktop) — these are passive scoring inputs, not blocks. Update `iac/edge-waf.yaml`.
4. **Sign-up false-positive.** Lower sensitivity for `MailAccountSignup` action; soak 60s; verify UX-floor synthetic.
5. **Adversary signature.** Submit to vendor; tactical Cedar rule in `policy/abuse-defence.cedar` targeting only the signature.
6. **Verify UX-floor.** Default-path latency ≤2ms p99; zero CAPTCHA presentations on regular login.
7. **Accessibility.** Re-run a11y CI lane.
8. **Emit closure.** `oya.mail.abuse-defence-recalibrate-complete`.

## D. Verification

- False-positive rate < 0.1% over next 24h.
- a11y CI lane green.

## E. Rollback

`helm rollback <mail-edge-waf> 1`; Cedar fragment rolls back via 60s soak.

## F. Post-incident

Log adversary signatures in `evidence/abuse-defence/mail-adversary-signatures.md`.

## G. References

- `policy/abuse-defence.cedar`
- `docs/standards/documentation-rigor.md §3.2.3`
- ADR-0297
