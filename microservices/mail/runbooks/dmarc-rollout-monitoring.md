---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0273]
companion_docs: [microservices/mail/compliance.md]
inbound_citations: [microservices/mail/ARCHITECTURE.md]
---

# Runbook: DMARC rollout monitoring (p=none → p=quarantine → p=reject)

## A. Trigger conditions

- New tenant onboarding (first 30 days): mandatory `p=none` monitoring window.
- DMARC report shows ≥1% legitimate-sender failures during rollout.
- Tenant requests promotion from p=quarantine to p=reject.

## B. Pre-checks

1. Confirm operator Cedar permit `oya.mail.dmarc-promote`.
2. Capture current DMARC policy: `dig +short TXT _dmarc.<tenant-subdomain>`.
3. Pull last 7d of DMARC aggregate reports from the report-ingest endpoint.

## C. Procedure

1. **Day 0-30: monitor.** Tenant runs `p=none; rua=mailto:dmarc-reports@oyatie.example`. Reports flow into `oya-mail-dmarc-report-ingest`. Timing budget: continuous.
2. **Day 30 review.** Aggregate report: legitimate-sender DKIM-pass rate ≥99.5% AND SPF-pass rate ≥99.5%. If yes → promote to p=quarantine. If no → diagnose senders failing alignment.
3. **Diagnose failing senders.** Identify the IP / sending-service; either onboard them to DKIM signing OR exclude them from SPF; emits `oya.mail.dmarc-sender-onboard`.
4. **Promote to p=quarantine.** Update tenant DNS via the tenant-DNS-management surface; emit `oya.mail.dmarc-policy-update`; soak 60s per ADR-0294.
5. **Day 60 review.** Quarantine-rate from legitimate senders ≤0.5%. If yes → promote to p=reject. If no → re-diagnose.
6. **Promote to p=reject.** Update DNS; emit `oya.mail.dmarc-policy-update`. Tenant-admin notified; tenant dashboard shows DMARC-protected status.
7. **Day 90 audit.** Run `oya mail dmarc-audit --tenant <id>`; verify all production senders are aligned; emit `oya.mail.dmarc-rollout-complete`.

## D. Verification

- DNS TXT `_dmarc.<tenant>` shows `p=reject`.
- Aggregate report shows ≥99.9% DKIM-pass + SPF-pass.
- Zero customer complaints about legitimate mail being rejected.

## E. Rollback

If post-promotion users report legitimate mail rejected: rollback to `p=quarantine` via DNS update (≤5 minutes propagation with low TTL); identify the broken sender; re-onboard; re-promote after fix.

## F. Post-incident

Update tenant's compliance.md if jurisdiction-specific deliverability constraints surfaced.

## G. References

- ADR-0273 per-tenant DKIM/SPF/DMARC
- `dashboards/dmarc-deliverability.json`
- `runbooks/dkim-key-rotation.md`
