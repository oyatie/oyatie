---
doc_class: Runbook
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0297]
companion_docs: [microservices/notes/policy/abuse-defence.cedar]
inbound_citations: [microservices/notes/ARCHITECTURE.md]
---

# Runbook: Notes bot-score recalibration

## A. Trigger conditions

- False-positive rate on `policy/abuse-defence.cedar` > 0.5% on sync surface or sign-up.
- UX-floor violation: typing / sync added latency > 2ms p99.
- Adversary fingerprint shift.

## B. Pre-checks

1. Verify operator Cedar permit `oya.notes.abuse-defence-tune`.
2. Pull last 24h block events.

## C. Procedure

1. Diagnose class.
2. For substrate calls (workflow-studio, intelligence): verify SPIFFE workload identity + `audience_type=INTERNAL_SUBSTRATE`.
3. For sync UX-floor regression: lower edge sensitivity in `iac/edge-waf.yaml:syncSensitivity`; hot-reload ≤30s.
4. For sign-up false-positive: tune `MailAccountSignup`-equivalent route.
5. Tactical Cedar rule for adversary signature; soak 60s.
6. Verify UX-floor: default-path latency ≤2ms; zero CAPTCHA on regular use.
7. a11y CI lane green.
8. Closure: `oya.notes.abuse-defence-recalibrate-complete`.

## D. Verification

False-positive rate < 0.1% within 24h.

## E. Rollback

`helm rollback <notes-edge-waf> 1`.

## F. Post-incident

Log signatures.

## G. References

- `policy/abuse-defence.cedar`
- ADR-0297
