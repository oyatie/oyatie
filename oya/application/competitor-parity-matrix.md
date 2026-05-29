---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-application + council-architecture
deciders: axis-application, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/PRD.md (§"Competitive Benchmark")
  - /specs/hyperscaler-gates.json (HG-APP)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (application µservice)

## Purpose

Quantitative + qualitative parity comparison vs industry-leading
application shells. Drives the `oya-governance-hyperscaler-maturity-claims`
gate (per ADR-0123 HG-APP) and tells gtm-customer-success what to claim
and what NOT to claim. Re-validated bi-annually.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Vercel | Vercel Platform | Edge-network frontend serve; per-project isolation; preview deployments; Edge Functions | `vercel.com/docs` |
| Next.js | Next.js App Router | File-system routing; server components; module federation | `nextjs.org/docs` |
| Stripe | Stripe Dashboard | Tenant-scoped shell; product-area switching; admin actions UX; financial-grade audit | `stripe.com/docs/dashboard` |
| Linear | Linear App Shell | TTI ≤ 1 s; module-isolated workspace; offline-first; keyboard-driven UX | `linear.app` |
| Notion | Notion App | Block-loader code-splitting; multi-workspace shell; lazy module fetch | `notion.so` |
| Palantir | Foundry App Shell | Per-tenant Workshop hosting; module registration protocol; per-org RBAC; Cedar-equivalent policy | `palantir.com/foundry/` |

## Feature Parity Matrix

### Performance (TTI + render)

| Capability | oyatie | Vercel | Next.js | Stripe | Linear | Notion | Foundry |
|---|---|---|---|---|---|---|---|
| TTI p99 ≤ 2 s warm | ✅ | ✅ | ✅ | ✅ | ✅ (<1 s) | ✅ | ✅ |
| Edge-network static serve | ✅ | ✅ | via Vercel | ✅ | ✅ | ✅ | partial |
| Code-split per module | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Server-side render (SSR) hydration | ✅ (Leptos SSR) | ✅ | ✅ | ✅ | ✅ | partial | ✅ |
| Offline-first | partial (read-only) | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| HTTP/3 + HSTS preload | ✅ | ✅ | depends | ✅ | ✅ | ✅ | ✅ |

### Tenant model

| Capability | oyatie | Vercel | Next.js | Stripe | Linear | Notion | Foundry |
|---|---|---|---|---|---|---|---|
| Per-tenant isolated shell | ✅ | per-project | per-app | per-merchant | per-workspace | per-workspace | per-tenant |
| Per-tenant origin (DNS) | ✅ (`<hash>.app.oyatie.dev`) | per-project | per-app | shared | shared | shared | per-tenant |
| Tenant scope in JWT | ✅ | ❌ | n/a | ✅ | ❌ | ❌ | ✅ |
| Pack residency (jurisdiction) | ✅ | partial (EU/US regions) | n/a | partial | ❌ | partial | ✅ |
| Cedar-gated routing | ✅ | ❌ | ❌ | bespoke | bespoke | bespoke | ✅ (PRPS) |

### Auth + Identity

| Capability | oyatie | Vercel | Next.js | Stripe | Linear | Notion | Foundry |
|---|---|---|---|---|---|---|---|
| OIDC SSO | ✅ | ✅ | DIY | ✅ | ✅ | ✅ | ✅ |
| SAML SSO | ✅ | ✅ (Enterprise) | DIY | ✅ | ✅ | ✅ | ✅ |
| MFA (TOTP + WebAuthn) | ✅ | ✅ | DIY | ✅ | ✅ | ✅ | ✅ |
| Two-cookie + PKCE + nonce contract | ✅ | partial | DIY | ✅ | ✅ | ✅ | ✅ |
| SCIM 2.0 provisioning | M04 | ✅ | DIY | ✅ | ✅ | ✅ | ✅ |
| Session revocation on HR termination | ✅ | partial | DIY | ✅ | ✅ | partial | ✅ |

### Module loader (the differentiator)

| Capability | oyatie | Vercel | Next.js | Stripe | Linear | Notion | Foundry |
|---|---|---|---|---|---|---|---|
| Cryptographically signed module manifest | ✅ Ed25519 | ❌ | ❌ | ❌ | ❌ | ❌ | partial (signed-bundle) |
| Subresource Integrity (SRI) hash verify | ✅ | partial | partial | partial | partial | partial | ✅ |
| Per-product publisher key | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | partial |
| iframe sandbox for module isolation | ✅ | n/a | n/a | n/a | n/a | n/a | ✅ |
| Module rollback on integrity failure | ✅ automated | manual | manual | manual | manual | manual | partial |
| Auditable module-load events | ✅ Ed25519 sealed | ❌ | ❌ | partial | partial | partial | ✅ |

### Audit + Compliance

| Capability | oyatie | Vercel | Next.js | Stripe | Linear | Notion | Foundry |
|---|---|---|---|---|---|---|---|
| Ed25519 audit-chain seal | ✅ | ❌ | ❌ | ✅ (proprietary) | partial | partial | ✅ |
| Tenant DSR cascade (erasure) | ✅ | ✅ | DIY | ✅ | ✅ | ✅ | ✅ |
| Per-pack residency forbid-by-default | ✅ | partial | ❌ | partial | ❌ | partial | ✅ |
| Auditor-scope JIT read | ✅ Cedar | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| External-auditor export (watermarked) | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| SOC 2 / ISO 27001 / GDPR / KR PIPA / HIPAA matrix | ✅ | ✅ | n/a | ✅ | partial | partial | ✅ |

### Operations (the operability differentiator)

| Capability | oyatie | Vercel | Next.js | Stripe | Linear | Notion | Foundry |
|---|---|---|---|---|---|---|---|
| Multi-window multi-burn-rate SLO gate | ✅ (via observability) | ❌ | ❌ | partial | partial | ❌ | ✅ |
| Automated rollback on production breach | ✅ | partial | ❌ | partial | partial | ❌ | partial |
| Per-pack BCDR posture documented | ✅ | partial | n/a | partial | partial | partial | ✅ |
| OpenSLO-native authoring | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | partial |
| CDN global purge ≤ 60 s | ✅ | ✅ | depends | ✅ | ✅ | ✅ | ✅ |

## Gap Analysis (ordered by priority)

1. **Offline-first** — Linear is ahead. M03 ships read-only offline (cached
   last-render); full offline-mutation subsequent-to-M03-completion.
2. **SCIM 2.0 provisioning** — Vercel + Stripe + Linear + Notion + Foundry
   ahead. M04 target.
3. **Edge Functions / Edge runtime** — Vercel ahead. Not in scope for
   Application Shell (would migrate to a separate edge µservice if
   pursued).
4. **TTI <1 s** — Linear ahead. M03 target ≤ 2 s; future budget tightening
   subsequent-to-M03-completion as bundle size shrinks.

## Differentiation (ordered)

1. **Cryptographically signed module manifest with per-product key
   isolation** — none of the consumer-facing competitors do this; Foundry
   has signed bundles but not per-product key isolation.
2. **Cedar-expressed routing policy** — auditable, externally verifiable
   policy (vs. bespoke RBAC).
3. **Pack-residency forbid-by-default with full 11-pack matrix** —
   industry-leading regulatory posture.
4. **Multi-window multi-burn-rate SLO gate driving auto-rollback** —
   inherited from oyatie observability substrate.

## HG-APP Hyperscaler Maturity Claims (claimable; CI-verified)

- ✅ "TTI p99 ≤ 2 s warm" (claim verified by `oya-application-tti-budget` lane).
- ✅ "Per-tenant origin DNS" (claim verified by per-pack ingress lint).
- ✅ "Ed25519 signed module manifest" (claim verified by module-loader integration test).
- ✅ "Cedar-gated default-deny routing" (claim verified by `oya-application-cedar-default-deny` lane).
- ✅ "Pack-residency forbid-by-default" (claim verified by `oya-application-residency-pin` lane).
- ✅ "Audit-chain seal latency ≤ 1 s p99" (claim verified by observability SLO).

## References

- ADR-0123 cross-product auth + hyperscaler maturity gate.
- `microservices/observability/competitor-parity-matrix.md` (precedent).
- `feedback_quality_performance_scalability_bar.md`.
