---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-009-captcha-adapter
status: pending
execution_unit: ChangeSet
owner: axis-forms + ops-security
acceptance_lanes: [cargo-test, oya-forms-captcha-fail-closed-conformance, oya-forms-captcha-pack-resident-routing, oya-forms-recaptcha-forbidden-pack-eu-kr-us-hc]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: Captcha adapter (hCaptcha + Turnstile + Friendly Captcha; reCAPTCHA forbidden in eu/kr/us-hc)

## Intent

Multi-provider captcha verifier per ADR-FORMS-0002. Per-pack provider selection + fallback chain + fail-closed invariant + reCAPTCHA forbidden in pack-eu / pack-kr / pack-us-healthcare.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/adapter/captcha/provider.rs` | create — trait + provider enum |
| `microservices/forms/src/adapter/captcha/hcaptcha.rs` | create |
| `microservices/forms/src/adapter/captcha/turnstile.rs` | create |
| `microservices/forms/src/adapter/captcha/friendly.rs` | create |
| `microservices/forms/src/adapter/captcha/fail_closed.rs` | create — sidecar-down → 503 |
| `microservices/forms/src/adapter/captcha/pack_routing.rs` | create |
| `microservices/forms/tests/captcha_fail_closed.rs` | create |
| `microservices/forms/tests/captcha_pack_routing.rs` | create |

## Acceptance Gates

- `oya-forms-captcha-fail-closed-conformance`: sidecar killed → 503 (never accept without verification).
- `oya-forms-captcha-pack-resident-routing`: pack-kr never routes to non-KR-resident provider.
- `oya-forms-recaptcha-forbidden-pack-eu-kr-us-hc`: Cedar policy + adapter config refuse reCAPTCHA in those packs.

## References

- ADR-FORMS-0002.
- hCaptcha + Turnstile + Friendly Captcha provider docs.
- PRD FR-08 and AC-11.
- `microservices/forms/policy/data-residency.md`.
- `microservices/forms/policy/public-read.cedar`.
- `microservices/forms/runbooks/captcha-degraded.md`.
- `microservices/forms/runbooks/spam-flood-throttle.md`.
- `microservices/forms/slos/submission-latency.openslo.yaml`.

## Foundation A-G Substance

- A. Product scope: captcha is a public-submit admission control, not a post-submit spam label.
- B. Domain model: `CaptchaProvider`, `CaptchaChallenge`, `CaptchaVerification`, and `ProviderResidencyRoute` stay independent of HTTP clients.
- C. Contracts: REST exposes challenge-required and verification-failed states with stable error codes.
- D. Policy: provider choice is pack-routed; reCAPTCHA is denied where policy forbids it, even if tenant config requests it.
- E. Operations: provider timeout, degraded sidecar, and abuse flood runbooks define fail-closed behavior.
- F. Observability: emit provider latency, challenge solve rate, fail-closed count, forbidden-provider rejections, and bot-score distribution.
- G. Promotion: pack routing, reCAPTCHA ban, sidecar-kill drill, burst submit test, and public-read policy test gate completion.

## Counterpart Benchmark

- Counterpart: HubSpot Forms captcha controls, Salesforce Web-to-Lead spam protection, and Twilio Verify anti-abuse verification flows.
- Defensible parity claim: Oyatie must route captcha by residency and fail closed for anonymous writes.
- Differentiator: provider selection is governed by policy and compliance pack, not a tenant-visible toggle alone.
- Grep counterpart names: HubSpot Forms; Salesforce Web-to-Lead; Twilio Verify.

## Remediation Notes

- Added artifact-grounded captcha routing, policy, SLO, and runbook substance.
- Added A-G sections for domain, contracts, policy, operations, observability, and promotion.
- Added counterpart names for grep-recognized parity review.

## Verification Evidence Required

- Sidecar-kill drill returns 503 for anonymous public submit and never persists a response.
- Pack routing corpus proves pack-eu, pack-kr, and pack-us-healthcare never load forbidden providers.
- Bot-score and solve-rate metrics appear in the submission dashboard.
- Abuse-flood drill links rate-limit and captcha runbooks to the same incident timeline.
- Contract test proves challenge-required and verification-failed errors are stable.

## Next IP

[`IP-010-form-builder-leptos-wasm.md`](IP-010-form-builder-leptos-wasm.md)
