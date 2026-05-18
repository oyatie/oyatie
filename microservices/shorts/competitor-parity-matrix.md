---
doc_class: CompetitorParityMatrix
title: Competitor Parity Matrix
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-shorts + gtm-strategy + council-architecture
deciders: axis-shorts, gtm-strategy, council-architecture
related_adrs: [ADR-0135, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/shorts/PRD.md (§Competitive Benchmark)
review_cadence: quarterly
doc_status: published
---

# Competitor Parity Matrix (shorts µservice)

## Purpose

Per-feature side-by-side parity check between shorts and short-form-video competitors. Drives priority decisions for IP scoping + roadmap. Sources cited per published documentation; proprietary features are inferred from public reverse-engineering / press analysis (caveat: oyatie does not access competitor private code).

## Competitor Set

| Competitor | Class | Owner | Reference docs |
|---|---|---|---|
| TikTok | First-class short-video platform | ByteDance | n/a (proprietary; press analysis) |
| Instagram Reels | Short-video feature in Instagram | Meta | `developers.facebook.com/docs/instagram-platform` |
| YouTube Shorts | Short-video feature in YouTube | Google | `developers.google.com/youtube` |
| Snapchat Spotlight | Short-video feature in Snapchat | Snap Inc | `kit.snapchat.com` |
| Twitter/X video | Short-form video on X | X Corp | `developer.x.com` |
| Likee | Short-video emerging-market | BIGO | (limited) |
| Triller | Short-video creator-fund focus | Triller Network | (limited) |
| Lemon8 | Lifestyle short-video | ByteDance | (limited) |
| Tangi | How-to short-video | Google (Area 120) | (limited) |
| Kuaishou | CN short-video | Kuaishou Tech | (limited; CN market) |
| Douyin | CN TikTok sister | ByteDance CN | (limited; CN market) |
| Vimeo Short | Creator-focused short-video | Vimeo | `developer.vimeo.com` |

## Parity Matrix

Legend: ✓ = parity; ◐ = partial; ✗ = absent; — = inherently inapplicable; ★ = oyatie differentiator.

### Core publishing

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Video upload ≤ 60s | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Multi-bitrate HLS/DASH ABR | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Video composition (clip/cut/sticker/caption overlay) | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ |
| Audio-track library (licensed + UGC) | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ |
| Audio attribution (sound rights chain) | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ |
| Thumbnail (poster + animated preview) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

### Feed + discovery

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Chronological feed | ◐ | ◐ | ✗ | ◐ | ✓ | ◐ | ✗ | ◐ | ✗ | ✗ | ✗ | ✓ | ✓ ★ default-fallback |
| Algorithmic For-You feed | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ |
| User-controllable chronological switch (EU DSA Art. 27) | ◐ | ◐ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | n/a | ✓ ★ EU DSA-conformant from day-1 |
| Hashtag discovery | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ |
| Sound-of-the-week trending | ✓ | ✓ | ✓ | ◐ | ✗ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✗ | ✓ |
| Repost via Stitch | ✓ | ◐ | ✓ | ✗ | ✗ | ◐ | ◐ | ◐ | ✗ | ✓ | ✓ | ✗ | ✓ |
| Repost via Duet | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ |

### Engagement + analytics

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Like + share + comment | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Watch-time tracking | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Completion-ratio signal | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ |
| Creator analytics dashboard | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ |
| Real-time notifications | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ◐ | ✓ | ✓ | ◐ | ✓ |
| k-anonymity ≥ 10 on analytics aggregates | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ |

### Safety + moderation

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| NSFW classifier | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Violence classifier | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Minor-protection classifier | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ◐ | ✓ ★ Cedar-bound + pack-aware |
| Appeal workflow (EU DSA Art. 20) | ◐ | ◐ | ◐ | ✗ | ◐ | ✗ | ✗ | ◐ | ✗ | ✗ | ✗ | ◐ | ✓ ★ ≤ 7d SLA |
| Statement of Reasons (EU DSA Art. 17) | ◐ | ◐ | ◐ | ✗ | ◐ | ✗ | ✗ | ◐ | ✗ | ✗ | ✗ | ◐ | ✓ ★ per verdict |
| EU AI Act Art. 50 transparency label | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ from day-1 |
| Transparency report (EU DSA Art. 24) | ◐ | ◐ | ◐ | ✗ | ◐ | ✗ | ✗ | ◐ | ✗ | ✗ | ✗ | ◐ | ✓ ★ per-tenant quarterly |
| Audit-chain seal per moderation verdict | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ Ed25519 |

### Copyright

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Content-ID-class fingerprint matching | ✓ | ✓ | ✓ | ✓ | ◐ | ◐ | ◐ | ◐ | ◐ | ✓ | ✓ | ✓ | ✓ |
| Pre-publication copyright pre-check | ✓ | ✓ | ✓ | ◐ | ✗ | ✗ | ✗ | ◐ | ✗ | ✓ | ✓ | ◐ | ✓ |
| DMCA Title II Safe Harbor compliance | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | n/a | n/a | ✓ | ✓ |
| Counter-notice workflow | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ◐ | ◐ | ✗ | n/a | n/a | ✓ | ✓ ★ creator-visible UI |
| Repeat-infringer policy enforcement | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ◐ | ◐ | ✗ | n/a | n/a | ✓ | ✓ ★ audit-chain seal |

### Minor protection

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Per-pack age threshold (KR 14, EU 16-default, US-COPPA 13) | ✗ | ✗ | ◐ | ✗ | ◐ | ✗ | ✗ | ✗ | ✗ | ◐ | ◐ | ✗ | ✓ ★ pack-aware |
| Parental-consent at signup | ◐ | ◐ | ◐ | ◐ | ◐ | ✗ | ✗ | ✗ | ✗ | ◐ | ◐ | ✗ | ✓ ★ |
| Parental-controls (linked-account) | ◐ | ◐ | ◐ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ◐ | ◐ | ✗ | ✓ ★ first-class BC |
| Minor accounts: chronological-only default | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ per EU DSA Art. 28 |
| Minor accounts: algorithmic-opt-out by default | ✗ | ◐ | ◐ | ◐ | ◐ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ |
| Minor accounts: DM-restricted by default | ◐ | ◐ | ◐ | ◐ | ◐ | ✗ | ✗ | ✗ | ✗ | ◐ | ◐ | ✗ | ✓ ★ |
| Screen-time supervision | ◐ | ◐ | ◐ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ◐ | ◐ | ✗ | ✓ ★ |

### Accessibility

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Auto-caption (ASR) | ✓ | ✓ | ✓ | ◐ | ✓ | ◐ | ◐ | ◐ | ◐ | ✓ | ✓ | ✓ | ✓ |
| Manual caption override | ✓ | ✓ | ✓ | ◐ | ✓ | ◐ | ◐ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ |
| WebVTT + TTML emission | ✓ | ✓ | ✓ | ◐ | ✓ | ◐ | ◐ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ |
| WCAG 2.2 Level AA conformance | ◐ | ◐ | ◐ | ◐ | ◐ | ✗ | ✗ | ◐ | ◐ | ◐ | ◐ | ✓ | ✓ ★ |

### DRM

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Widevine | ◐ | ◐ | ✓ | ◐ | ◐ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ |
| FairPlay | ◐ | ◐ | ✓ | ◐ | ◐ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ |
| PlayReady | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ◐ | ✓ | ✓ |
| Tenant-tier DRM gating (Premium tier only) | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ ★ |

### Data residency + tenancy

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Per-pack region pinning | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ 11 packs |
| Per-tenant DEK envelope encryption | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ Bominal ADR-0111 |
| Four-eyes Professional disclosure | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ Bominal ADR-0215 |
| Multi-tenancy as primitive | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ◐ | ✓ ★ |

### Cross-product integration

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Typed Workflow events native | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ |
| Ontology entity-write native | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ ★ |
| Share-to-DM bridge (messenger sibling) | ◐ | ◐ | ◐ | ✓ | ✓ | ◐ | ✗ | ◐ | ✗ | ◐ | ◐ | ✗ | ✓ ★ first-party bridge |
| Cross-link to social profile feed | ◐ | ◐ | ✓ | ◐ | ◐ | ◐ | ✗ | ◐ | ✗ | ◐ | ◐ | ✓ | ✓ ★ first-party |

### Federation

| Feature | TikTok | Reels | Shorts | Spotlight | X | Likee | Triller | Lemon8 | Tangi | Kuaishou | Douyin | Vimeo | **shorts** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ActivityPub federation (Professional tier opt-in) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ◐ (P02; tenant opt-in; metadata-only) |
| Personal-tier never-federates invariant | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ ★ compile-time DCI-08 |

## Key Parity Gaps (oyatie advantage)

1. **Dual-context isolation by data model** — none of competitors enforce Personal/Professional context as data-model invariant. Target: compile-time + LEAN-lane enforcement.
2. **Minor-protection at the regulatory floor** — TikTok/Reels have been fined repeatedly for COPPA / GDPR Art. 8 / UK OSA / CA AB-2273 / UT SMRA violations; oyatie pack-aware age-gate + parental-controls by default.
3. **Native Workflow + Ontology integration** — competitors expose webhooks/Graph APIs; oyatie exposes typed Workflow events + Ontology object writes natively.
4. **OpenSLO + agentic gate** — none gate feature rollouts on SLO compliance; oyatie does per ADR-0130.
5. **Multi-pack residency + per-pack regulatory overlays** — competitors are SaaS-region-coarse; oyatie is per-pack jurisdiction-pinned (11 packs).
6. **EU AI Act high-risk transparency** — competitors lag on Art. 50 transparency labels for moderation + ranking + ASR; oyatie ships from day-1.
7. **Copyright-claim integrity** — competitors' Content-ID systems are opaque to creators; oyatie publishes counter-notice + repeat-infringer audit-chain seal per claim.
8. **DRM tenant-tier gating** — competitors don't expose DRM at tenant granularity; oyatie's Widevine + FairPlay + PlayReady is per-tenant Premium-tier feature.
9. **Per-pack content moderation overlay** — competitors apply a global moderation policy with regional carve-outs; oyatie applies per-pack regulatory floor (KR PIPA + EU DSA + UK OSA + CA AB-2273 + UT SMRA).
10. **Four-eyes Professional disclosure + audit-chain Ed25519** — Bominal-inherited dual-context safety primitive.

## Parity Gaps (oyatie disadvantage; needs successor-IP)

- Federation (ActivityPub video) — scheduled-for-distinct-tracked-work to P02; competitors don't have it either, so neutral.
- Live-streaming — scheduled-for-distinct-tracked-work to M05-onward; competitors offer it (TikTok Live, Reels Live, YouTube Live). Acceptable for M03 scope.
- Monetization (tip-jar + creator-fund) — scheduled-for-distinct-tracked-work to M04-onward; competitors offer it (TikTok Creator Fund, Reels Bonuses, YouTube Shorts Fund). Acceptable for M03 scope.
- AT Protocol federation — out-of-scope (not in competitor set; aligned with social ADR successor-IP).

## References

- `microservices/shorts/PRD.md` §Competitive Benchmark.
- `microservices/social/competitor-parity-matrix.md` (sibling pattern).
- Public docs of each competitor (URLs above).
- Published EU DSA enforcement actions (DPC Ireland, BfDI Germany, etc.) — for regulatory-gap evidence.
- EU AI Act 2024/1689; EU AVMSD 2018/1808; EU DSA 2065/2022.
- DMCA Title II 17 USC §512.
- COPPA 15 USC §6501; CA AB-2273; UT Social Media Regulation Act.
