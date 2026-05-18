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

## Next IP

[`IP-010-form-builder-leptos-wasm.md`](IP-010-form-builder-leptos-wasm.md)
