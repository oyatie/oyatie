---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0297]
companion_docs: [microservices/social/policy/abuse-defence.cedar]
inbound_citations: [microservices/social/ARCHITECTURE.md]
---

# Runbook: Social bot-score recalibration

## A. Trigger conditions

- False-positive rate on `policy/abuse-defence.cedar` > 0.5% on the feed-read / profile-read / post-create surfaces.
- UX-floor violation: legitimate users seeing CAPTCHA on regular use.
- Adversary bot-farm signature shift (sock-puppet farm using new device-attestation bypass).

## B. Pre-checks

1. Operator Cedar permit `oya.social.abuse-defence-tune`.
2. Pull last 24h `oya.social.abuse-defence-block` events tabulated by `audience_type`, route, fingerprint, minor_age_band.
3. Cross-reference with edge bot-mgmt vendor dashboard.

## C. Procedure

1. **Diagnose class.** False-positives on substrate, on KOSA minor accounts (extra-sensitive), on verified-creator accounts, on sign-up?
2. **Substrate fix.** SPIFFE workload identity + `audience_type=INTERNAL_SUBSTRATE` headers verified.
3. **KOSA minor fix.** Lower sensitivity on `audience_type=B2C_PERSONAL_MINOR_KOSA_14_17` since minor accounts have lower baseline activity (don't penalize quiet usage).
4. **Verified-creator fix.** Higher base allowance for `B2B_CREATOR_VERIFIED` per per-tier sensitivity in `iac/edge-waf.yaml`.
5. **Sign-up false-positive.** Verify Turnstile invisible challenge is selecting correct cohort; tune threshold.
6. **Adversary signature.** Submit to vendor; tactical Cedar rule targeting signature only; soak 60s.
7. **Verify UX-floor.** Default-path latency ≤2ms p99; CAPTCHA-presentations on regular use = 0; a11y CI lane green.
8. **Tenant-admin notification.** Per ADR-0263, push the false-positive-rate metric to tenant dashboards.
9. **Closure.** `oya.social.abuse-defence-recalibrate-complete`.

## D. Verification

- False-positive rate < 0.1% over next 24h.
- a11y CI lane green.
- UX-floor synthetic latency unchanged.

## E. Rollback

`helm rollback <social-edge-waf> 1`; Cedar fragment roll back via 60s soak.

## F. Post-incident

Log signatures in `evidence/abuse-defence/social-adversary-signatures.md`.

## G. References

- `policy/abuse-defence.cedar`
- `docs/standards/documentation-rigor.md §3.2.3`
- ADR-0297
