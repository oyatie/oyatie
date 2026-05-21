---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0251]
companion_docs: [microservices/social/compliance.md]
inbound_citations: [microservices/social/ARCHITECTURE.md]
---

# Runbook: EU DSA transparency report generation

## A. Trigger conditions

- DSA Art. 24 deadline (semi-annual): every 6 months for all platforms.
- DSA Art. 42 (VLOP) deadline if applicable: annual + on-request.
- Member-state regulator request.

## B. Pre-checks

1. Operator Cedar permit `oya.social.dsa-report-generate`.
2. Confirm reporting window + jurisdiction-scope.
3. Pull aggregate audit-event metrics: `oya.social.moderation-action`, `oya.social.appeal-submit`, `oya.social.appeal-resolve`, `oya.social.csam-detect`, `oya.social.account-suspend`, `oya.social.cluster-takedown-complete`.

## C. Procedure

1. Aggregate per-jurisdiction metrics: moderation-actions by content-class, appeals, takedowns, account suspensions, ad-targeting transparency.
2. Compute Statement-of-Reasons coverage; verify ≥95% of moderation actions have a Statement-of-Reasons emitted.
3. Recommender-system parameters: publish the ranking-feature manifest + opt-out availability.
4. Crisis-protocol activations (Art. 36): list activations in window.
5. Designated point-of-contact + legal representative confirmation.
6. Generate the DSA report PDF via `oya social dsa-report build --window <Y-MM>`.
7. Submit to EU Commission + publish on platform's transparency-report page.
8. Emit `oya.social.dsa-report-publish`.

## D. Verification

- Report meets DSA Art. 24 minimum structure.
- Published on platform with stable URL.
- Submitted to EU Commission via official channel.

## E. Rollback

Reports are immutable post-publication; corrections via supplemental publication.

## F. Post-incident

Update reporting tooling + next-window queue.

## G. References

- EU Digital Services Act Art. 15, 24, 42
- `runbooks/sock-puppet-cluster-takedown.md`
- `runbooks/csam-detect-and-ncmec-report.md`
