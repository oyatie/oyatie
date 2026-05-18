---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-foundation
phase: P01-shorts-foundation
status: Active
entry_gate: |
  ADR-0135 (parallel Connect dissolution) + ADR-0131 + ADR-0132 accepted; observability µservice IP-001..IP-015 merged;
  social µservice IP-001..IP-015 merged (shorts depends on social for profile + follow-graph + mention bridge + federation);
  messenger µservice IP-001..IP-015 merged (shorts depends on messenger for share-to-DM bridge);
  audit-chain + tenancy + ontology + cedar substrate live; foundry-runtime T1/T2 deployment substrate ready.
exit_gate: |
  All 15 IPs merged; all ~140 crates compile + nextest green; oya gate validate per-microservice-layout --microservice shorts
  exits 0; oya gate validate dual-context-isolation --microservice shorts exits 0; HG-SHORTS gate registers green;
  end-to-end profile + upload + transcode + publish + view + like + comment + share + repost-stitch + repost-duet +
  copyright-claim + DMCA + moderation + appeal + age-gate + parental-link + caption-auto + DRM-license drill passes
  within performance budget; pack-kr overlay deployed to dedicated shorts cluster.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: shorts requires observability gate + tenancy + ontology + audit-chain + cedar
  - milestone: M02-foundation
    phase: P01-social-foundation
    reason: shorts depends on social profile + follow-graph + mention-bridge for cross-product flow
  - milestone: M02-foundation
    phase: P01-team-channels-dm-threads
    reason: shorts depends on messenger share-to-DM bridge
owner_team: axis-shorts
related_adrs: [ADR-0008, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
doc_status: published
---

# P01-shorts-foundation: Land the shorts µservice end-to-end (first-phase scope)

## Purpose

This phase ships the foundation of the shorts µservice per parallel ADR-0135 + ADR-0132: video-upload + video-transcode + video-storage + thumbnail + audio-track-library + audio-attribution + video-composition + feed-timeline + watch-time + like-share-comment + repost-stitch-duet + hashtag + trending + notifications + content-moderation + copyright-claim + age-gate + parental-controls + accessibility-captions + creator-analytics + DRM-stub, dual-context-safe across Personal (B2C) and Professional (B2B).

It advances master-plan principles:
- Hyperscaler-grade in every practice (TikTok / Reels / Shorts parity + native Workflow + Ontology integration).
- Nothing scheduled-for-distinct-tracked-work within scope (no FUTURE stubs except monetization-stub and live-streaming-stub explicitly off by default per PRD §"Out-of-scope").
- No silent regression (production-tier change gated by observability ADR-0139).
- Per-microservice flat layout (ADR-0131 native authoring).
- Dual-context isolation by data model (NOT runtime flag) per parallel ADR-0135.
- Minor-protection at regulatory floor — EU DSA Art. 28 + KR 청소년 보호법 + COPPA + UK OSA + CA AB-2273 + UT SMRA enforced by default.

## Scope

### In-scope (first phase)

| µservice | Bounded Contexts | Crate count |
|---|---|---|
| `shorts` | `video-upload`, `video-transcode`, `video-storage`, `thumbnail-generation`, `audio-track-library`, `audio-attribution`, `video-composition`, `feed-timeline`, `watch-time-tracking`, `like-share-comment`, `repost-stitch-duet`, `hashtag`, `trending`, `notifications`, `content-moderation`, `copyright-claim`, `age-gate`, `parental-controls`, `accessibility-captions`, `creator-analytics`, `monetization-stub`, `live-streaming-stub`, `drm-stub` | ~140 crates |

Plus cross-cutting:
- `.github/branch-protection.yaml` — add `release/shorts/*` pattern protection + 8 required checks per IP-015.
- `/specs/hyperscaler-gates.json` — register HG-SHORTS per ADR-0133.
- `Cargo.toml` (workspace) — register ~140 crates.
- `docs/standards/dual-context-isolation.md` (already authored cross-cutting per parallel ADR-0135).

### Out-of-scope (scheduled-for-distinct-tracked-work to successor-IP phases)

- **Live-streaming** — stub-only at M03; M05-onward activation per PRD Open Question 1.
- **Monetization (tip-jar + creator-fund)** — stub-only at M03; M04-onward activation per PRD Open Question 2.
- **Federation (ActivityPub video)** — scheduled-for-distinct-tracked-work to P02; opt-in per tenant; Personal-tier never federates. Posture inherits ADR-SOC-0004 by default.
- **Ranking model openness (closed-weights vs published-weights)** — P01 heuristic + ML ranking with closed-weights; per PRD Open Question 4 successor-IP.
- **Per-tenant ranking weights** — scheduled-for-distinct-tracked-work to M04-onward per social pattern.
- **Voice / image-only posts** — out-of-scope; shorts is video-only.

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-iac-bootstrap.md`](IP-001-iac-bootstrap.md) | Helm/Kustomize/Terraform for shorts cluster; Postgres + Valkey + Meilisearch + S3 + CloudFront + ClamAV/OPSWAT + ffmpeg + DRM substrate | pending | axis-shorts + ops-sre-reliability + cloud-secrets | observability IP-001 |
| [`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md) | Cargo workspace + ~140 crate scaffolds per ADR-0131 | pending | axis-shorts | IP-001 |
| [`IP-003-video-upload-bc.md`](IP-003-video-upload-bc.md) | `video-upload` kernel + domain + usecase + api + adapter-postgres + adapter-s3 + scan-first lifecycle + rest + sdk + app | pending | axis-shorts | IP-002 |
| [`IP-004-video-transcode-bc.md`](IP-004-video-transcode-bc.md) | `video-transcode` BC end-to-end; ffmpeg 7.x sandboxed worker; HLS/DASH ladder; H.264/H.265/AV1 + AAC/Opus; CMAF | pending | axis-shorts | IP-003 |
| [`IP-005-video-storage-and-cdn-bc.md`](IP-005-video-storage-and-cdn-bc.md) | `video-storage` BC end-to-end; S3 per-tenant prefix + KMS + CloudFront CDN + signed-URL TTL ≤ 15min | pending | axis-shorts + cloud-secrets | IP-004 |
| [`IP-006-thumbnail-and-composition-bc.md`](IP-006-thumbnail-and-composition-bc.md) | `thumbnail-generation` + `video-composition` BCs; poster + animated GIF; server-side clip/cut/sticker/caption-overlay finalisation | pending | axis-shorts | IP-004 |
| [`IP-007-audio-track-library-and-attribution-bc.md`](IP-007-audio-track-library-and-attribution-bc.md) | `audio-track-library` + `audio-attribution` BCs; licensed catalog + UGC sounds + per-pack licensing metadata + rights chain | pending | axis-shorts + ops-legal | IP-003 |
| [`IP-008-feed-timeline-and-watch-time-bc.md`](IP-008-feed-timeline-and-watch-time-bc.md) | `feed-timeline` + `watch-time-tracking` BCs; algorithmic For-You + chronological; per-(viewer, video) watch-session tracking | pending | axis-shorts | IP-005 + IP-007 |
| [`IP-009-like-share-comment-and-repost-bc.md`](IP-009-like-share-comment-and-repost-bc.md) | `like-share-comment` + `repost-stitch-duet` BCs; conflict-free counters; rights-check before stitch/duet composition | pending | axis-shorts | IP-008 |
| [`IP-010-hashtag-and-trending-bc.md`](IP-010-hashtag-and-trending-bc.md) | `hashtag` + `trending` BCs; sound-of-the-week derivation; windowed compute | pending | axis-shorts | IP-007 + IP-008 |
| [`IP-011-content-moderation-and-copyright-claim-bc.md`](IP-011-content-moderation-and-copyright-claim-bc.md) | `content-moderation` (NSFW + violence + minor-protection classifier; T2) + `copyright-claim` (Content-ID-class fingerprint match; DMCA cycle) | pending | axis-shorts + axis-foundry-runtime + ops-legal | IP-004 |
| [`IP-012-age-gate-and-parental-controls-bc.md`](IP-012-age-gate-and-parental-controls-bc.md) | `age-gate` + `parental-controls` BCs; per-pack thresholds; parental-link supervision; minor-account defaults | pending | axis-shorts + council-privacy | IP-002 |
| [`IP-013-accessibility-captions-bc.md`](IP-013-accessibility-captions-bc.md) | `accessibility-captions` BC; foundry-runtime ASR auto-caption (T1); WebVTT + TTML emission; manual override | pending | axis-shorts + axis-foundry-runtime | IP-004 |
| [`IP-014-notifications-and-creator-analytics-bc.md`](IP-014-notifications-and-creator-analytics-bc.md) | `notifications` + `creator-analytics` BCs; real-time WebSocket + digest worker; per-creator dashboards | pending | axis-shorts + axis-messenger | IP-008 + IP-009 |
| [`IP-015-drm-and-hg-shorts-registration.md`](IP-015-drm-and-hg-shorts-registration.md) | `drm-stub` BC + HG-SHORTS hyperscaler-grade conformance gate per ADR-0133 + branch-protection wiring | pending | axis-shorts + ops-security + ops-governance | IP-005 + IP-014 |

## Per-IP Test Coverage Threshold

| Class | Coverage line / branch | Test types required |
|---|---|---|
| kernel | 90 % / 80 % | per-port-trait + per-entity unit; sealed-trait smoke; data-class annotation check |
| domain | 90 % / 80 % | pure-math / pure-logic unit |
| usecase | 85 % / 75 % | orchestrator unit with port mocks; happy + error path |
| adapter | 80 % / 70 % | integration vs real backend (Postgres / Valkey / S3 / Meilisearch / ClamAV / ffmpeg / Widevine sandbox) where feasible; otherwise contract-mock |
| rest | 85 % / 75 % | per-endpoint happy + 401 + 403 + 422 |
| worker | 85 % / 75 % | event-loop unit + integration |
| app | 75 % / 65 % | smoke startup |

E2E: ≥ 1 per AC-NN row in PRD.

## Phase-Gate Verification Bundle

Required CI lanes green on every commit + on phase-exit:

- `oya gate validate per-microservice-layout --microservice shorts`
- `oya gate validate dual-context-isolation --microservice shorts`
- `oya gate validate authority-cohesion --microservice shorts` (HG-SHORTS)
- `oya gate validate hyperscaler-maturity-claims --microservice shorts`
- `oya gate validate shardability --microservice shorts`
- `oya gate validate statelessness --microservice shorts`
- `oya gate validate layer-correctness --microservice shorts`
- `oya gate validate port-location --microservice shorts`
- `oya gate validate bnf-v4-1 --microservice shorts`
- `oya gate validate cedar-policy-spec --microservice shorts`
- `oya gate validate version-pinning-conformance` (LTS pins for Postgres/Redis/Meilisearch/ClamAV/OPSWAT/ffmpeg/Widevine/FairPlay/PlayReady)
- `oya gate validate compliance-evidence-recency --microservice shorts`
- `oya gate validate eu-ai-act-conformance --microservice shorts`
- `oya gate validate eu-dsa-conformance --microservice shorts`
- `oya gate validate pack-aware-age-gate --microservice shorts`
- `oya gate validate dmca-safe-harbor-conformance --microservice shorts`

## Phase Exit Bundle

1. All 15 IPs merged.
2. All ~140 crates `cargo nextest` green; coverage thresholds met per class.
3. End-to-end drill in pack-kr cluster: profile → video-upload → transcode → publish → view → like → comment → share → repost-stitch → repost-duet → copyright-claim → counter-notice → moderation-verdict → appeal → age-gate-minor → parental-link → caption-auto → DRM-license completes within performance envelope.
4. Capacity tier XS deployed: 20 tenants, ~100k MAU, ~50 video-upload/sec sustained, ~5k video-plays/sec, OpenSLO burn-rate green for 7 days.
5. Postmortem + sign-off by council-architecture, ops-security, council-privacy, ops-legal (DMCA + EU AVMSD posture), axis-shorts lead.
