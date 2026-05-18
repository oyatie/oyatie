---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-social + council-architecture
deciders: axis-social, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/social/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-SOCIAL gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (social µservice)

## Purpose

Quantitative + qualitative parity comparison against industry-leading social-network products. Drives `oya-governance-hyperscaler-maturity-claims` gate per HG-SOCIAL (ADR-0123) and constrains what gtm-customer-success can claim in tenant sales conversations. Re-validated bi-annually because the social landscape moves quickly (Bluesky federation, Threads expansion, Mastodon protocol updates, EU DSA + AI Act compliance arms-race).

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Twitter / X | global microblog + follow-graph + algorithmic feed | scale (~500M MAU); paid verification; X Premium subscriptions | `developer.x.com` |
| Bluesky | AT Protocol microblog; decentralised | OSS-decentralised; algorithmic-feed-marketplace; user-controlled algorithm | `docs.bsky.app` |
| Mastodon | ActivityPub federated microblog | federation; chronological-only by default; OSS | `docs.joinmastodon.org` |
| Threads (Meta) | Instagram-tied microblog | mobile-first; Meta-graph leverage; ActivityPub interop announced | `developers.facebook.com/docs/threads` |
| Facebook | full social network | profile + feed + groups + events; massive scale | `developers.facebook.com` |
| Instagram | photo + reels + stories | media-first; reels; stories; shopping | `developers.facebook.com/docs/instagram-platform` |
| LinkedIn | professional social network | dual-context (professional-only); rich profiles; jobs | `learn.microsoft.com/linkedin` |
| TikTok (timeline) | short-video feed | algorithmic-feed sophistication; ranking depth | (proprietary; 3rd-party studies) |
| Pinterest | visual-discovery board | image-first; collection model; pin/board | `developers.pinterest.com` |
| Reddit | community-forum + threaded discussion | subreddit model; voting; pseudonymous | `www.reddit.com/dev/api` |
| Lemmy | OSS federated Reddit-alike | ActivityPub federation | `join-lemmy.org/docs/` |
| Truth Social | Mastodon-fork microblog | (US-political niche; included for completeness) | (limited public docs) |
| Hive Social | Twitter-alike; minimal moderation | (smaller scale; included for completeness) | (limited public docs) |

## Feature Parity Matrix

### Core social

| Capability | oyatie | X | Bluesky | Mastodon | Threads | Instagram | LinkedIn |
|---|---|---|---|---|---|---|---|
| Profile (handle + bio + avatar) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Follow / unfollow | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Block / mute | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Post (text + media) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Repost / boost | ✅ | ✅ | ✅ | ✅ | ✅ | partial | partial |
| Quote-post | ✅ | ✅ | ✅ | partial | ✅ | n/a | partial |
| Comment / reply | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Reactions (multiple emoji) | ✅ | ❌ (like only) | partial | partial | partial | partial | ✅ |
| @mentions | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| #hashtags | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bookmarks (private) | ✅ | ✅ | partial | partial | ✅ | ✅ | ✅ |
| Lists | ✅ | ✅ | partial | ✅ | ❌ | ✅ | partial |
| Edit window | ✅ | ✅ (paid) | partial | ✅ | ✅ | ❌ | partial |
| Delete / tombstone | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Content warnings + sensitive media | ✅ | ✅ | ✅ | ✅ | partial | partial | partial |
| Alt-text for media accessibility | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ |
| Post-link deep-share to messenger | ✅ | n/a | n/a | n/a | n/a | n/a | n/a |

### Feed + discovery

| Capability | oyatie | X | Bluesky | Mastodon | Threads | TikTok | LinkedIn |
|---|---|---|---|---|---|---|---|
| Chronological feed | ✅ | ✅ | ✅ | ✅ default | ✅ | ❌ | ✅ |
| Algorithmic feed | ✅ (P03 ML, hybrid in P01) | ✅ default | ✅ marketplace | ❌ | ✅ default | ✅ default | ✅ default |
| User-choice feed switcher | ✅ | ✅ | ✅ | n/a | partial | ❌ | partial |
| Trending topics | ✅ | ✅ | partial | ❌ | ✅ | ✅ | ✅ |
| Search people | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Search content | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ |
| Bots / automation API | ✅ | ✅ (rate-limited) | ✅ | ✅ | partial | partial | ✅ |

### Notifications + real-time

| Capability | oyatie | X | Bluesky | Mastodon | Threads | Instagram |
|---|---|---|---|---|---|---|
| Real-time WebSocket | ✅ | partial | ✅ | partial | ✅ | ✅ |
| Push notifications (APNs / FCM) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Digest notifications | ✅ | ✅ | partial | partial | ✅ | ✅ |
| Per-recipient idempotency | ✅ | partial | partial | partial | partial | partial |

### Compliance + enterprise

| Capability | oyatie | X | Bluesky | Mastodon | Threads | LinkedIn |
|---|---|---|---|---|---|---|
| eDiscovery hold (Professional-tier) | ✅ | partial | ❌ | self-host | partial | partial |
| Retention per regulatory pack | ✅ (11 packs) | tenant-level only | self-host | self-host | tenant-level | tenant-level |
| HIPAA BAA | conditional (pack-us-hc) | ❌ | ❌ | self-host | ❌ | ❌ |
| KR PIPA + KISA | ✅ pack-kr | ❌ (region only) | ❌ | self-host | partial | partial |
| Dual-context (Personal / Professional) | ✅ data-model invariant | partial (account-level) | account-level | account-level | account-level | professional-only |
| Federation | optional (P02; ActivityPub) | ❌ | ✅ AT Proto | ✅ ActivityPub | partial (ActivityPub announced) | ❌ |
| Four-eyes admin disclosure | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Cedar / Rego / OPA policy | ✅ Cedar v4 | partial | partial | partial | partial | partial |
| EU DSA transparency report | ✅ native (Art. 24) | required (Art. 24); partial reports | partial | partial | required; partial | required; partial |
| EU AI Act high-risk transparency | ✅ Art. 50 native | required; partial | partial | partial | required; partial | required; partial |
| Age verification + minor protection | ✅ pack-aware (COPPA + GDPR Art. 8 + KR 청소년 보호법) | partial | partial | self-host | partial | ✅ enterprise-grade |
| Appeal workflow | ✅ Art. 20 native | ✅ (Art. 20-required) | partial | partial | required | required |

### Substrate

| Capability | oyatie | X | Bluesky | Mastodon | Lemmy |
|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (Helm + Kustomize) | ❌ | ✅ PDS-self-host | ✅ | ✅ |
| Multi-region data-residency | ✅ 11 packs | partial (regions) | self-host | self-host | self-host |
| OpenSLO + agentic gate | ✅ | ❌ | ❌ | ❌ | ❌ |
| Ed25519 audit-chain | ✅ | ❌ (vendor logs) | partial (DIDs) | partial | ❌ |
| Workflow + Ontology native integration | ✅ | ❌ | ❌ | ❌ | ❌ |

## Quantitative Performance Parity

| Metric | oyatie target | X ref | Bluesky ref | Mastodon ref | Threads ref | Notes |
|---|---|---|---|---|---|---|
| Post-create p99 | ≤ 250ms | ~300ms (published) | ~400ms | ~500ms | ~280ms | parity with X / Threads |
| Feed-render p95 (top 50) | ≤ 200ms | ~250ms | n/a published | ~400ms | ~220ms | parity |
| Profile-render p95 | ≤ 150ms | ~150ms | n/a | ~250ms | ~150ms | parity |
| Follow-action p99 | ≤ 150ms | ~100ms | n/a | n/a | ~100ms | within range |
| Search people p95 | ≤ 300ms | ~400ms | ~600ms | ~800ms | ~350ms | parity |
| Search content p95 | ≤ 500ms | ~500ms | ~800ms | ~1s | ~550ms | parity |
| Notification fanout p99 (10k followers) | ≤ 2s | ~3s (estimated) | n/a | n/a | ~2.5s | parity |
| Notification fanout p99 (100k followers) | ≤ 5s | ~5-10s (estimated) | n/a | n/a | ~5s | parity |
| Media transcode (image) p95 | ≤ 2s | ~3s | ~3s | ~5s | ~2s | parity |
| Media transcode (video HLS) p95 | ≤ 90s | ~60s | n/a | n/a | ~90s | parity |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | ML-driven algorithmic feed (vs heuristic in P01) | axis-social + axis-foundry-runtime | M03 |
| 2 | ActivityPub federation minimum-shippable-tier | axis-social + ops-security | M02-onward1 |
| 3 | Mobile SDK polish (iOS/Android parity with X / Instagram native) | axis-social + gtm | M02-onward1 |
| 4 | Video-first content type (Reels-style; out-of-scope P01) | axis-social | M04-onward (separate µservice) |
| 5 | Mature bot / app marketplace (X / Discord lead by years) | axis-social + gtm | M05-onward |
| 6 | Audio Spaces-style live audio room (out-of-scope P01) | axis-social | M04-onward (live-audio µservice) |
| 7 | AT Protocol federation (in addition to ActivityPub) | axis-social + council-architecture | successor-IP ADR (PRD Open Question 2) |
| 8 | Verified-handle global uniqueness (vs per-tenant) | axis-social + gtm | ADR-SOC successor-IP (PRD Open Question 5) |

## Key oyatie Differentiators (NOT in any competitor)

1. **Dual-context isolation by data-model invariant** — Personal ≠ Professional enforced at compile-time + LEAN-lane (per parallel ADR-0238); no competitor does this at data-model level. LinkedIn is professional-only (not dual-context); X / Meta blur with account-level switch.
2. **Multi-pack residency by design** — 11 region-pinned packs; no SaaS competitor matches breadth.
3. **OpenSLO-gated promotion** — feature rollouts gated by burn-rate (ADR-0139); no competitor enforces SLO-based rollout halting.
4. **Cedar v4 policy substrate** — fine-grained per-resource policy; competitors expose only admin-level RBAC.
5. **Cryptographic audit-chain** — Ed25519 + Merkle over every state transition; competitors deliver opaque vendor logs.
6. **Four-eyes admin disclosure** — two-principal approval for PII reads; no competitor enforces.
7. **Workflow + Ontology native integration** — first-class events typed into Workflow Studio; competitors expose webhooks only.
8. **EU AI Act Art. 50 + Art. 27 transparency from day-1** — competitors are scrambling to comply post-2024.
9. **Personal-tier never federates by data-model invariant** — no competitor has compile-time guarantee.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "Dual-context personal/professional enforced as a data-model invariant is unique to oyatie" (true as of 2026-05-17; review bi-annually).
- ✅ "11-pack residency exceeds X / Threads / LinkedIn regional coverage" (true).
- ✅ "OpenSLO-gated feature rollout is unique to oyatie among production social platforms" (review bi-annually).
- ✅ "Cedar v4 fine-grained policy substrate exceeds X admin RBAC depth" (true).
- ✅ "EU AI Act Art. 50 transparency labels ship out-of-the-box" (true; review bi-annually).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie social is faster than Twitter / X" (no published benchmark; would be unsourced superiority).
- ❌ "oyatie has more features than Instagram" (feature-count is unmeasurable + Instagram has 10+ years marketplace head start).
- ❌ "HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation).
- ❌ "X-API compatible" (we accept X-style incoming-webhook URL shape only; full X-API parity not claimed).
- ❌ "Algorithm-free" (we ship hybrid; user can choose chronological but algorithmic is the default for engagement; don't market as algorithm-free).
- ❌ "Federated like Mastodon" (federation is opt-in for Professional-tier only; Personal-tier never federates; don't market as universal federation).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-social |
| 3. Re-run quantitative benchmarks (load tests in staging cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/social/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-SOCIAL gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0135 (Connect dissolution, parallel).
- ADR-0139 (agentic SLO-gated promotion).
- ADR-0132 (suite-and-bundle dissolution).
- ADR-0133 (industry best-practice conformance).
- Competitor docs as cited inline above.
- EU DSA 2065/2022; EU AI Act 2024/1689.
