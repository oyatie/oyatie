---
id: ADR-FORMS-0002
title: Captcha selection — hCaptcha primary + Cloudflare Turnstile + Friendly Captcha fallback; reCAPTCHA forbidden in pack-eu/kr/us-hc
microservice: forms
status: Accepted
date: 2026-05-17
owner: axis-forms + ops-security + council-privacy
deciders: council-architecture, axis-forms, ops-security, council-privacy, council-legal-compliance, gtm-customer-success
supersedes: []
superseded_by: []
related: [ADR-0131, ADR-0140, ADR-FORMS-0003]
related_specs: [/specs/products/forms.json]
related_artifacts:
  - microservices/forms/PRD.md FR-08 + AC-11
  - microservices/forms/policy/data-residency.md §"Pack determines captcha provider"
  - microservices/forms/runbooks/captcha-degraded.md
  - microservices/forms/runbooks/spam-flood-throttle.md
  - microservices/forms/threat-model.md §"T-D-01" + §"T-D-03" + §"T-D-07" + §"T-S-05"
doc_status: published
---

# ADR-FORMS-0002: Captcha — hCaptcha primary + Cloudflare Turnstile + Friendly Captcha fallback; reCAPTCHA forbidden in pack-eu / pack-kr / pack-us-healthcare

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Forms exposes anonymous-submission endpoints to the public internet. Without captcha, the response store is trivially flooded by botnets, response analytics are poisoned by synthetic submits, and per-form rate-limits at the WAF alone are insufficient against high-IP-cardinality attacks. Captcha is a **load-bearing** anti-spam control.

Privacy posture is the dominant constraint:

1. **Google reCAPTCHA** (v2 + v3) requires loading `google.com/recaptcha/api.js` cross-origin AND sets Google cookies on submitter browsers. This contributes to Google's cross-site profile, includes US-resident data flows, and has been challenged under:
   - **Schrems II** (CJEU C-311/18): cross-Atlantic transfers to US-resident processors require supplementary measures.
   - **KR PIPA Art. 23-2**: cross-border sensitive data transfer is regulated; PIPC 2020 guidance disfavours US-resident analytics on KR-resident user flows.
   - **HIPAA BAA**: Google does not offer a BAA for reCAPTCHA loaded on patient-intake flows; using reCAPTCHA on a HIPAA form creates a covered-entity BAA gap.
   - **CNIL** (France DPA) ruled 2022 (R. n° SAN-2022-021): Google Analytics + reCAPTCHA combination violates Schrems II; affected sites fined.
2. **Cloudflare Turnstile** is a privacy-preserving captcha that does not require cookies and operates on behavioural signals + token exchange. EU-resident path available; HIPAA BAA not standard but technically privacy-preserving.
3. **hCaptcha** is a privacy-preserving captcha offering EU + KR regional routing; explicit BAA available for HIPAA tenants; SCC pre-signed for EU tenants.
4. **Friendly Captcha** is a proof-of-work-based captcha (no behavioural data leaves submitter's browser); EU-resident; best privacy posture but slower UX.

oyatie tenant base spans 11 packs; we cannot ship a single global captcha config without violating at least one pack's privacy posture.

## Decision

Adopt a **multi-provider captcha** strategy with per-pack defaults + explicit forbidden-providers list + fail-closed invariant.

### Provider defaults per pack

| Pack | Primary | Fallback 1 | Fallback 2 |
|---|---|---|---|
| pack-kr | hCaptcha (KR-routed) | Friendly Captcha | (none — manual review) |
| pack-eu | hCaptcha (EU-routed) | Friendly Captcha | (none — manual review) |
| pack-us | Cloudflare Turnstile | hCaptcha | Friendly Captcha |
| pack-us-healthcare | hCaptcha (HIPAA BAA) | Friendly Captcha | (none — manual review) |
| pack-jp | hCaptcha (JP-routed) | Friendly Captcha | Cloudflare Turnstile |
| pack-sg | hCaptcha + Turnstile | Friendly Captcha | – |
| pack-au | Cloudflare Turnstile | hCaptcha | Friendly Captcha |
| pack-in | hCaptcha | Friendly Captcha | Cloudflare Turnstile |
| pack-br | hCaptcha | Friendly Captcha | Cloudflare Turnstile |
| pack-ae | hCaptcha | Friendly Captcha | – |
| pack-ksa | hCaptcha | Friendly Captcha | – |

### Forbidden providers (per-pack)

- **pack-eu**: Google reCAPTCHA forbidden (Schrems II + CNIL ruling).
- **pack-kr**: Google reCAPTCHA forbidden (PIPA Art. 23-2 + PIPC 2020).
- **pack-us-healthcare**: Google reCAPTCHA forbidden (no BAA gap).

Enforced by Cedar policy `tenant-scope.cedar` FORBID-block (`Action::configure_recaptcha_provider` when `pack in [pack-eu, pack-kr, pack-us-healthcare]`).

### Fail-closed invariant

Captcha sidecar unavailable → submit returns 503; NEVER accept without verification. Asserted by `oya-forms-captcha-fail-closed-conformance` CI lane (T-D-07 invariant).

### Tenant choice within pack

Tenant may override pack-default within the pack's allow-list (e.g., pack-us tenant may choose hCaptcha over Turnstile). Tenant may NOT add a forbidden provider.

### Per-form challenge mode

- **Default**: invisible challenge (low friction, behavioural).
- **Elevated** (auto-engaged on rate-spike): visible puzzle (per `runbooks/spam-flood-throttle.md` Path D).
- **Manual review queue**: engaged when all providers degrade (per `runbooks/captcha-degraded.md` Path B).

### Pinned versions

- hCaptcha SDK 1.x LTS.
- Cloudflare Turnstile SDK 0.x LTS.
- Friendly Captcha SDK 1.x LTS.

## Alternatives Considered

### Alternative A — Google reCAPTCHA-only globally

Use reCAPTCHA v3 globally as a single provider.

- **Pros**
  - Massive scale; widely understood; cheap.
  - Single integration to maintain.
- **Cons**
  - Schrems II violation in pack-eu (post-CNIL 2022 ruling).
  - PIPA Art. 23-2 violation in pack-kr.
  - HIPAA BAA gap in pack-us-healthcare.
  - Cross-site profile contribution erodes submitter privacy without explicit consent.
  - Tenant operators in privacy-sensitive industries refuse Google-attached analytics.
- **Rejected reason**: violates the privacy-posture constraints of three packs (eu, kr, us-hc). Forms cannot ship reCAPTCHA globally and remain GA-eligible in those packs.

### Alternative B — Single privacy-preserving provider globally (e.g., hCaptcha only)

Use hCaptcha as the sole global provider.

- **Pros**
  - Privacy-posture-compatible across all 11 packs.
  - Single integration; simpler operations.
- **Cons**
  - Single-provider risk: outage = global Forms degradation. The 2024 hCaptcha service-disruption (provider-side, not oyatie-attributed) would have taken all 11 packs offline for ~3 hours.
  - hCaptcha pricing tier increases at high volume; lock-in.
  - Some pack-us tenants prefer Turnstile (Cloudflare-attached UX).
- **Rejected reason**: single-provider concentration risk. ADR-0117 (cloud-native) infrastructure principle for Forms recommends multi-provider for critical third-party deps.

### Alternative C — No captcha; rely on WAF rate-limit + behavioural ML only

Skip captcha entirely; use OCI WAF rate-limit + an internal behavioural ML classifier.

- **Pros**
  - Best UX (no challenge).
  - No third-party dependency.
- **Cons**
  - Cannot defend against high-IP-cardinality botnets (each IP submits 1/sec, defeating per-IP rate-limit; cluster sees millions/min).
  - Behavioural ML requires training data; Forms is net-new — no training data.
  - Industry standard (every competitor offers captcha as primary anti-spam): Google Forms, Microsoft Forms, Typeform, Jotform, etc.
- **Rejected reason**: no captcha = trivially-flooded response store from day 1. Behavioural ML is a future enhancement *on top of* captcha, not a replacement.

### Alternative D — Tenant-supplied captcha (BYO-captcha)

Tenant configures their own captcha integration (e.g., tenant brings their reCAPTCHA Enterprise account).

- **Pros**
  - Maximum tenant flexibility.
  - No oyatie cost.
- **Cons**
  - Privacy posture per pack now depends on individual tenant configuration — defeats the per-pack regulatory commitment.
  - oyatie cannot guarantee fail-closed invariant if tenant captcha provider down.
  - Audit-chain seal must include captcha verification — variability complicates audit.
- **Rejected reason**: per-pack regulatory posture must be platform-enforced, not tenant-discretionary.

### Alternative E — Proof-of-work only (e.g., Friendly Captcha globally)

Use proof-of-work captcha across all packs as primary.

- **Pros**
  - Best privacy posture (no third-party tracking).
  - Cookieless.
- **Cons**
  - Higher submitter-side compute cost; mobile UX slower (~1-3s on low-end devices).
  - Less effective vs well-resourced bot farms with dedicated GPUs.
  - Lower industry penetration; tenant operators less familiar.
- **Rejected reason**: UX cost on mobile is non-trivial; Friendly Captcha as primary in all packs hurts conversion. Friendly Captcha as fallback (when behavioural captcha degrades) is the right tier.

## Consequences

### Architectural

- The `oya-forms-captcha-adapter` exposes a single internal interface; per-pack routing selects provider at submit time.
- The captcha sidecar runs in the Forms namespace with the SDK + token-verification endpoint; sidecar HA per pack.
- Captcha token verification is a critical-path latency contributor (~50-200ms); SLO `oya-forms-submission-latency` budgets for it.
- The form-renderer Leptos-WASM includes per-pack captcha JS at runtime; CSP `script-src` allow-lists the pack-allowed provider domains.

### Downstream µservices

1. **tenancy**: per-tenant captcha-provider entitlement (within pack allow-list).
2. **observability**: dashboard `dashboards/response-pipeline.json` exposes per-provider health.
3. **audit-chain**: every submit includes `captcha_provider` field in seal.
4. **foundry-providers** is unaffected (captcha is not an LLM call).

### SLOs and CI lanes affected

- `oya-forms-captcha-fail-closed-conformance` — exit 0 on adversarial test (sidecar killed; expect 503).
- `oya-forms-recaptcha-forbidden-pack-eu-kr-us-hc` — exit 0 (Cedar policy compile + adversarial config attempt).
- `oya-forms-captcha-pack-resident-routing` — exit 0.
- `oya-forms-submission-latency` p95 ≤ 150ms budget includes captcha verify time.

### Compliance + audit

- Schrems II compliance: reCAPTCHA forbidden in pack-eu enforced at policy layer.
- KR PIPA Art. 23-2 compliance: reCAPTCHA forbidden in pack-kr enforced at policy layer.
- HIPAA BAA gap closed: reCAPTCHA forbidden in pack-us-healthcare; chosen providers offer BAA.
- LGPD / DPDPA / APPI compatibility: chosen providers operate within respective jurisdictions.

### Risk register

- **Risk**: hCaptcha pricing increase past oyatie cost budget. **Mitigation**: multi-provider; can shift traffic.
- **Risk**: All three providers simultaneously degrade. **Mitigation**: manual review queue (Path B); tenant comms.
- **Risk**: New competitor captcha emerges (e.g., MCaptcha self-host). **Mitigation**: ADR supersession path.
- **Risk**: Solver-farm bypass at scale. **Mitigation**: multi-provider challenge + per-submitter velocity check; per `runbooks/spam-flood-throttle.md` Path D.
- **Risk**: Cookieless mode regression in hCaptcha. **Mitigation**: per-release provider regression test in `oya-forms-captcha-cookie-conformance`.

## References

- Schrems II (CJEU C-311/18, 16 July 2020).
- CNIL Sanction R. n° SAN-2022-021 (Google Analytics + reCAPTCHA finding).
- KR PIPA Art. 23-2 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.308 + BAA requirements for processors.
- hCaptcha privacy policy + EU-routing + BAA — `hcaptcha.com/`.
- Cloudflare Turnstile docs — `developers.cloudflare.com/turnstile/`.
- Friendly Captcha docs — `friendlycaptcha.com/`.
- Google reCAPTCHA terms — `policies.google.com/`.
- EDPB Recommendations 01/2020.
- `microservices/forms/policy/data-residency.md`.
- `microservices/forms/policy/tenant-scope.cedar`.
- `microservices/forms/runbooks/captcha-degraded.md`.
- `microservices/forms/runbooks/spam-flood-throttle.md`.
- ADR-0140 Cedar default-deny.
- ADR-0131 per-microservice flat layout.
