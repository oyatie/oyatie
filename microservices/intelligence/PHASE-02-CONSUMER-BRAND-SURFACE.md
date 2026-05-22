---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P02-intelligence-brand-ux-surface
status: Active
entry_gate: |
  P01-intelligence-two-layer-mvp exit_gate declared; dispatch SDK live in Rust/TS/Python/Swift/
  Kotlin; refusal floors live; audit tap emitting; first-token latency p99 < 2.0 s; brand-ux-surface
  crate family scaffolded.
exit_gate: |
  brand-ux-surface SDK published in TS (React + Vue + Solid + vanilla), Swift (iOS + macOS),
  Kotlin (Android + KMP), Flutter (Dart) and integrated into consumer brand product (Application
  Shell), developer console (Forge), and at least 3 first-party tenant apps; design-token export
  to Figma; accessibility (WCAG 2.2 AAA) verified by axe-pa11y-runner; per-tier badge rendering
  validated against design system; cost-floor disclosure copy reviewed by ops-legal for B2C.
depends_on:
  - milestone: M01-foundation
    phase: P01-intelligence-two-layer-mvp
    reason: brand-ux-surface renders against dispatch SDK output; that surface must exist first
owner_team: axis-intelligence + design-platform
related_adrs: [ADR-0255, ADR-0255-amendment-library-first]
date: 2026-05-20
doc_status: published
---

# P02-intelligence-brand-ux-surface: Land the consumer brand UX surface

## Purpose

ADR-0255 splits the AI surface into Layer-A (substrate) and Layer-B (consumer brand UX). P01 ships
Layer-A. P02 ships Layer-B: the cohesive consumer brand UX surface that every oyatie product (the
Application Shell, Forge developer console, and tenant-facing surfaces) renders to communicate AI
provenance, tier, streaming state, refusal copy, citation rendering, and cost-floor disclosure
(when B2C and platform-default cost float applies).

This phase ships the brand-ux-surface SDK in five frontend ecosystems and binds it to the dispatch
SDK output from P01.

## Scope

### In-scope

| Component | Description | Language / framework |
|---|---|---|
| `oya-intelligence-brand-ux-surface-sdk-ts` | React + Vue + Solid + vanilla TS components | TypeScript |
| `oya-intelligence-brand-ux-surface-sdk-swift` | SwiftUI components | Swift |
| `oya-intelligence-brand-ux-surface-sdk-kotlin` | Jetpack Compose + KMP components | Kotlin |
| `oya-intelligence-brand-ux-surface-sdk-flutter` | Flutter widgets | Dart |
| `oya-intelligence-brand-ux-surface-adapter` | Adapter that consumes dispatch SDK output | Rust |
| Design tokens | Figma + Style Dictionary export | JSON + SVG |
| WCAG 2.2 AAA compliance | axe-pa11y-runner E2E suite | n/a |
| Documentation | Storybook + per-component MDX | n/a |

### Out-of-scope

- Custom branding for tenant white-label (deferred to a marketplace pack per ADR-0249).
- 3D / spatial AI components (deferred until Vision Pro target tier reaches GA).
- Voice-only AI UX (the audio modality is implemented at Layer-A; the dedicated voice UX is a
  successor phase).

## Brand-UX component inventory

| Component | Purpose | First product |
|---|---|---|
| `SparkleIcon` | AI presence indicator (gradient sparkle; tier-aware colour) | every AI-touched surface |
| `TierBadge` | Display current tier (Opus / Sonnet / Haiku / open-weight / on-device) | Forge developer console |
| `StreamingText` | Token-by-token text rendering with cursor + skeleton fallback | Application Shell chat |
| `CitationChip` | Inline citation chip with hover + click-to-source | Application Shell + Forge |
| `RefusalBanner` | Render refusal reason + pack-localized copy | every AI-touched surface |
| `CostFloorDisclosure` | "Powered by oyatie AI (platform-default cost float)" disclosure | B2C surfaces only |
| `AudienceTagBadge` | Internal-only debug badge — never customer-facing | Foundry-internal |
| `RouterDecisionPopover` | Developer-only — explains why a provider was selected | Forge developer console |
| `EvalScoreRibbon` | Show eval canonicalen-set score on responses (developer-only) | Forge developer console |
| `BYOKBadge` | Show "Your provider keys in use" when provider-credential BYOK is active for the tenant (ADR-0255 §D-4) | Forge tenant settings |

## Implementation plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| IP-101 | Design tokens (SparkleGradient, TierColourScale, CitationChipStyle) | pending | design-platform | — |
| IP-102 | SDK-TS scaffold + 3 framework targets (React, Vue, Solid) | pending | axis-intelligence | IP-101 |
| IP-103 | SDK-Swift scaffold | pending | axis-intelligence | IP-101 |
| IP-104 | SDK-Kotlin scaffold | pending | axis-intelligence | IP-101 |
| IP-105 | SDK-Flutter scaffold | pending | axis-intelligence | IP-101 |
| IP-106 | SparkleIcon + TierBadge components × 4 SDKs | pending | axis-intelligence | IP-102..IP-105 |
| IP-107 | StreamingText + CitationChip × 4 SDKs | pending | axis-intelligence | IP-102..IP-105 |
| IP-108 | RefusalBanner + CostFloorDisclosure × 4 SDKs | pending | axis-intelligence | IP-102..IP-105 |
| IP-109 | Storybook (TS) + Swift Previews + Android Previews + DartPad | pending | design-platform | IP-106..IP-108 |
| IP-110 | axe-pa11y-runner E2E + WCAG 2.2 AAA verification | pending | ops-quality + design-platform | IP-109 |
| IP-111 | Integration with Application Shell consumer surface | pending | axis-application-shell | IP-106..IP-110 |
| IP-112 | Integration with Forge developer console | pending | axis-forge | IP-106..IP-110 |

(IP-101..IP-112 numbering is reserved for P02; P01 owned IP-001..IP-025.)

## Acceptance gates

```bash
# Per-SDK
pnpm -r --filter "@oyatie/brand-ux-surface-*" build && pnpm -r --filter "@oyatie/brand-ux-surface-*" test
xcodebuild -workspace BrandUXSurface.xcworkspace -scheme BrandUXSurface test
./gradlew :brand-ux-surface:assembleRelease :brand-ux-surface:test
flutter test packages/brand_ux_surface
cargo nextest run -p oya-intelligence-brand-ux-surface-adapter

# Accessibility
npx axe-pa11y-runner --target https://storybook-staging.oyatie.dev/intelligence-brand-ux-surface
                    --wcag 2.2 --level AAA --threshold 100

# Visual regression
npx percy storybook https://storybook-staging.oyatie.dev/intelligence-brand-ux-surface
```

### Brand-fidelity gate

Design-platform reviews every component against `design-tokens/intelligence/brand-ux-surface/`
canonical Figma library. Drift > 0.01 in WCAG-AAA contrast or > 1 px in spacing tokens fails the
gate.

### Localization gate

Refusal copy + cost-floor disclosure copy translated into all active packs (ko-KR, en-US, en-EU,
de-DE, fr-FR, ja-JP, zh-CN, …). Translation review by ops-legal per pack for refusal-floor accuracy.

## References

- ADR-0255 §"Layer-B — Consumer Brand UX Surface".
- ADR-0255 amendment — Library-first network-opt-in clarification.
- `microservices/intelligence/PRD.md`.
- `microservices/intelligence/ARCHITECTURE.md` §2 (two-layer model).
- `docs/standards/documentation-rigor.md`.
