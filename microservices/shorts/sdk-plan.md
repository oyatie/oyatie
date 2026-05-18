---
doc_class: SDKPlan
title: SDK + Client Library Plan
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-shorts + gtm-developer-experience
deciders: axis-shorts, gtm-developer-experience, council-architecture
related_adrs: [ADR-0056, ADR-0105, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/shorts/contracts/openapi/shorts.yaml
  - microservices/shorts/contracts/asyncapi/shorts-events.yaml
  - microservices/shorts/contracts/proto/shorts.proto
review_cadence: per SDK release
doc_status: published
---

# SDK + Client Library Plan (shorts µservice)

## Purpose

Define the client library distribution model, language coverage, version policy, and transparency-label conformance for the shorts µservice across web, desktop, mobile, server-to-server, and Workflow Studio shell consumers.

## Distribution Targets

| Target | Language | Auth | Package | Use |
|---|---|---|---|---|
| Web client | TypeScript | OIDC + WSS | `@oyatie/shorts-sdk-web` (npm) | Browser app: upload + playback (HLS.js + dash.js wrap) + feed + composer |
| Desktop client | TypeScript / Tauri | OIDC + WSS | bundled via `@oyatie/shorts-sdk-web` | Desktop shell |
| Mobile (iOS) | Swift | OIDC + WSS | `OyatieShortsSDK` (SwiftPM) | iOS playback + composer + upload (URLSession multipart resume) |
| Mobile (Android) | Kotlin | OIDC + WSS | `dev.oyatie.shorts.sdk` (Maven Central) | Android playback + composer + upload (OkHttp multipart resume) |
| Server-to-server | Rust | mTLS + SPIFFE | `oya-shorts-sdk` crate (this repo) | Downstream µservices + tenant backend integration |
| Server-to-server | Go | mTLS + SPIFFE | `github.com/oyatie/shorts-sdk-go` | Tenant integration |
| Server-to-server | Python | mTLS + SPIFFE | `oyatie-shorts-sdk` (PyPI) | Analytics + creator-tool integration |
| Workflow Studio shell | TypeScript | OIDC + WSS (bundled with shell) | `@oyatie/shorts-sdk-web` re-exported | Web-shell built on top |
| gRPC clients | Multi (proto3 generation) | mTLS + SPIFFE | per-lang stub generation | Power-user / partner integrations |

## Version Policy

Per Bominal ADR-0084 + ADR-0056 (BNF v4.1) SDK versioning policy:

- Semver MAJOR for breaking interface changes (rare; require ADR + sunset notice ≥ 90d).
- Semver MINOR for additive surface (new endpoint, new event, new param).
- Semver PATCH for fixes + non-functional improvements.

LTS: shorts SDK LTS at v1.x (M03 launch). LTS support window: 18 months from release.

## Common SDK Capabilities

- OIDC token acquisition + refresh.
- WebSocket connection lifecycle (auto-reconnect, exponential backoff, heartbeat).
- HLS / DASH playback wrapper (HLS.js, dash.js for web; AVPlayer for iOS; ExoPlayer for Android).
- DRM EME integration (Widevine on Chrome/Edge/Android, FairPlay on Safari/iOS, PlayReady on Edge/Xbox).
- Resumable multipart upload (chunked; resume on connection drop; client-side hash for de-dup).
- Client-side video composition preview (clip/cut/sticker/caption overlay; non-destructive; finalised server-side).
- Action-card serializer / deserializer for Workflow Studio surface.
- Audit-chain seal verification helper (for client-side optional re-verification).
- Cedar-aware error surface — translates 403 into actionable user prompts.
- Idempotency key generation (UUID7 / ULID).
- Per-pack endpoint discovery (`social-{pack}.oyatie.dev` pattern; shorts equivalent).

## EU AI Act + EU DSA Conformance Helpers

Per ADR-SHORTS-0003 + ADR-SHORTS-0005 + `capabilities/T2-auto.yaml`:

- `getRankingExplanation(feedRender)`: returns the `contributing_signals` array per EU AI Act Art. 27 + EU DSA Art. 27.
- `getModerationLabel(verdict)`: returns the `eu_ai_act_label` for UI rendering ("AI-assessed" / "human-assessed" / "hybrid") per EU AI Act Art. 50.
- `getCaptionLabel(captionTrack)`: returns the `eu_ai_act_label` for auto-captions ("AI-generated").
- `openAppealUI(verdictId)`: opens the appeal UI per EU DSA Art. 20.
- `setMinorProtectionDefaults(account)`: applies chronological-only + algorithmic-opt-out + DM-restricted defaults (called automatically on minor age-attestation).

## DMCA Helpers

- `fileCopyrightClaim(targetVideoId, claimantIdentity, perjuryAttestation)`: handles DMCA §512(c)(3) statutory elements.
- `fileCounterNotice(claimId, creatorIdentity, jurisdictionConsent)`: handles DMCA §512(g) counter-notice.
- `getRepeatInfringerStatus(creatorRef)`: visible only to creators about their own status (transparency).

## Accessibility

- WCAG 2.2 Level AA compliance helpers.
- Auto-caption track surfaced as default; manual override UI affordance.
- Captioned-by-default playback; ARIA labels on all controls.
- Keyboard navigation across feed + composer + appeal UI.

## SDK Release Pipeline

1. OpenAPI / AsyncAPI / proto3 contracts authored (audit-grade in `contracts/`).
2. Code generation: `cargo run -p oya-dev-cli -- sdk generate --microservice shorts --target {ts,swift,kt,go,py,rust}`.
3. Per-target test pack: contract tests + happy path + error path + auth-failure.
4. Per-target package publish (npm, SwiftPM, Maven Central, crates.io, PyPI, Go modules).
5. SBOM + sigstore signing per `/specs/supply-chain.json`.

## Release Cadence

- Monthly minor releases (M03 launch onward).
- Quarterly patch consolidation.
- Major version bumps: per ADR + 90d sunset notice.

## SDK Quality Bars

| Bar | Threshold |
|---|---|
| Contract conformance | 100% (CI lane `oya-governance-contract-conformance`) |
| Test coverage | ≥ 85% line / 75% branch |
| Bundle size (web) | ≤ 250 KB gzipped core; ≤ 500 KB with HLS.js + dash.js |
| Mobile binary size | iOS ≤ 5 MB; Android ≤ 5 MB |
| API breaking-change rate | 0 within MAJOR version |
| Time-to-first-render (web first video) | ≤ 1.5s p95 |

## References

- ADR-0056 (BNF v4.1).
- ADR-0084 (Bominal SDK versioning).
- ADR-0135 (Connect dissolution).
- ADR-0131 (per-µservice flat layout).
- EU AI Act Arts. 27, 50.
- EU DSA Art. 27.
- DMCA Title II 17 USC §512(c)(3), §512(g).
- WCAG 2.2 Level AA `www.w3.org/TR/WCAG22`.
- HLS RFC 8216; MPEG-DASH ISO/IEC 23009-1.
- W3C EME 2017.
- `microservices/social/sdk-plan.md` (sibling pattern).
