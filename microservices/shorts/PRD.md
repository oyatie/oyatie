---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-shorts
microservice: shorts
status: Accepted
sales_segment: connect-suite-product
tier: hero-product
milestone_first_ship: M03-foundation
bominal_source: []  # NET-NEW per ADR-0135; no Bominal predecessor
related_adrs: [ADR-0008, ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-shorts
doc_status: published
---

# PRD-shorts: First-Party Short-Form-Video Platform (Upload + Transcode + Audio-Track + Feed + Watch-Time + Like-Share-Comment + Repost-Stitch-Duet + Hashtag + Trending + Moderation + Copyright-Claim + Age-Gate + Parental-Controls + Captions + Notifications + Creator-Analytics + DRM-Tenant-Tier)

## Purpose

The `shorts` microservice is oyatie's native TikTok-/Instagram-Reels-/YouTube-Shorts-/Snapchat-Spotlight-class short-form-video platform. Per parallel ADR-0238 (Connect dissolution), it is one of the first-class µservices factored out of the legacy Connect umbrella. It owns **video-upload + video-transcode (multi-bitrate HLS/DASH ABR ladder) + video-storage (S3 + CDN) + thumbnail-generation + audio-track-library (licensed + UGC) + audio-attribution + video-composition (clip + cut + sticker + caption overlay) + feed-timeline (algorithmic + chronological) + watch-time-tracking + like + share + comment + repost (stitch + duet) + hashtag + sound-of-the-week trending + content-moderation (NSFW + violence + minor-protection classifier) + copyright-claim (Content-ID-class hash matching) + age-gate + parental-controls + accessibility-captions (auto-generated + manual) + notifications + creator-analytics + monetization-stub (off by default; tip-jar M04-onward) + live-streaming-stub (M05-onward; off at M03) + DRM-stub (HLS + DASH + Widevine/FairPlay/PlayReady; tenant-tier gated)** across the 11 oyatie regulatory packs.

This µservice is **a hero product**, end-user-facing through Workflow Studio shell and standalone shorts clients (web + desktop + mobile). It is also consumable as a shared substrate by other oyatie products via the `shorts.video.v1` Workflow events and the `Video`, `Sound`, `Sticker`, `Hashtag` Ontology object types.

**shorts is NET-NEW** per ADR-0135 — no `oya-connect-shorts-*` crates exist; there is no migration-from-connect.md or deprecation-notice.md. Bominal had no short-video product; this is greenfield in oyatie.

## Tenant Value

- **Tenant Outcome 1 — Native short-form-video without third-party SDK lock-in.** Tenants and their end-users get TikTok-/Reels-/Shorts-class upload-transcode-feed-share UX inside the same shell as social, messenger, calendar, mail, workflow studio — no embedded TikTok SDK; no YouTube Shorts iframe; first-party.
- **Tenant Outcome 2 — Copyright-safe by design.** Content-ID-class fingerprint matching at ingest (Chromaprint audio + perceptual-hash video) detects claims before publication; DMCA Title II Safe Harbor compliance with takedown + counter-notice + repeat-infringer workflow.
- **Tenant Outcome 3 — Minor-protection at the regulatory floor.** Per-pack age thresholds (KR 14, EU 16-default member-state-adjustable 13-16, US COPPA 13); minor accounts default to chronological-only feed, restricted DM, no public profile + algorithmic-recommendation-opt-out per EU DSA Art. 28 + KR 청소년 보호법 + UK OSA + UT Social Media Regulation Act + CA AB-2273.
- **Tenant Outcome 4 — Real-time feed delivery at hyperscaler latency.** Feed-load p95 ≤ 250ms (top 10 videos); video-start p95 ≤ 400ms (first frame); like-action p99 ≤ 50ms — competitive with TikTok / Reels published benchmarks.
- **Tenant Outcome 5 — Moderation that is auditable.** Every NSFW / violence / minor-protection verdict + appeal-action emits an audit-chain record (Merkle / Ed25519); EU AI Act high-risk classification for content-moderation + ranking model carries transparency obligations per Art. 50.
- **Tenant Outcome 6 — Tenant-tier DRM gating.** Widevine + FairPlay + PlayReady substrate available per tenant tier (Premium tier only by default); content protected at the HLS/DASH manifest layer via EME (W3C 2017).
- **Tenant Outcome 7 — Multi-pack residency by design.** 11 region-pinned packs; video blobs + transcode variants follow the source post's pack pinning; cross-pack federation never crosses video boundaries (federation is metadata-only).

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | creator | to upload a short video (≤ 60s; ≤ 500MB) | I publish content | video-upload | Must |
| FR-02 | creator | to compose / clip / cut / add stickers + caption overlays before publishing | I produce polished content | video-composition | Must |
| FR-03 | system | to transcode the uploaded video into a multi-bitrate HLS/DASH ladder (360p/480p/720p/1080p + optional 1440p) within 30s p95 | ABR streaming works on all devices | video-transcode | Must |
| FR-04 | system | to generate a poster + animated GIF preview thumbnail | feed-tile preview UX | thumbnail-generation | Must |
| FR-05 | creator | to attach a licensed audio track (from audio-track-library) or UGC sound | I score the video | audio-track-library | Must |
| FR-06 | system | to attribute the audio to the original sound (creator or licensor) | rights chain preserved | audio-attribution | Must |
| FR-07 | viewer | to see a chronological + an algorithmic For-You feed | I choose how I consume | feed-timeline | Must |
| FR-08 | system | to track watch-time + completion-ratio per (viewer, video) | engagement signal for ranking | watch-time-tracking | Must |
| FR-09 | viewer | to like / share / comment on a video | low-overhead reactions | like-share-comment | Must |
| FR-10 | creator | to repost via Stitch (clip another's video + append my own) or Duet (side-by-side) | remix culture | repost-stitch-duet | Must |
| FR-11 | viewer | to use #hashtags for discoverability | content grouped by topic | hashtag | Must |
| FR-12 | viewer | to see trending sounds + hashtags per pack (sound-of-the-week) | discovery surface | trending | Must |
| FR-13 | viewer | to report abuse / NSFW / violence / minor-protection violation | community safety | content-moderation | Must |
| FR-14 | viewer | to appeal a moderation verdict (auto-hide / removal) | due-process EU DSA Art. 20 | content-moderation | Must |
| FR-15 | system | to fingerprint-match audio + video at ingest against Content-ID corpus | copyright pre-check | copyright-claim | Must |
| FR-16 | rights-holder | to file a copyright-claim takedown notice | DMCA Title II + EU DSA Art. 16 | copyright-claim | Must |
| FR-17 | creator | to file a counter-notice for a contested claim | DMCA counter-notice | copyright-claim | Must |
| FR-18 | end-user (per pack regulation) | to attest age at signup | age-gate enforced | age-gate | Must |
| FR-19 | parent | to enable parental-controls on a minor's account | minor-protection per EU DSA Art. 28 + COPPA + KR 청소년 보호법 | parental-controls | Must |
| FR-20 | viewer | to see auto-generated captions on every video (or manual override) | accessibility WCAG 2.2 Level AA | accessibility-captions | Must |
| FR-21 | viewer | to receive real-time + digest notifications | I stay engaged | notifications | Must |
| FR-22 | creator | to view creator-analytics (watch-time, audience, posting cadence, audience growth) | I grow my channel | creator-analytics | Must |
| FR-23 | tenant-admin | to configure pack-aware retention + moderation policy | regulatory bounds hold | content-moderation + cell | Must |
| FR-24 | compliance-officer | to issue eDiscovery hold on professional shorts | regulatory request satisfied | video-storage + audit-chain | Must |
| FR-25 | Workflow Studio | to consume `VideoPublished` / `CopyrightClaimFiled` / `ModerationVerdictEmitted` events | downstream automation | video-upload + copyright-claim + content-moderation | Must |
| FR-26 | social µservice | to cross-link short-video deep-link into a social post | cross-product flow | video-upload + social bridge | Must |
| FR-27 | messenger µservice | to share-to-DM bridge (deep-link a video into a DM) | cross-product flow | video-upload + messenger bridge | Must |
| FR-28 | tenant-operator | to query upload + transcode + feed + moderation + copyright metrics | I plan capacity + verify SLAs | observability | Must |
| FR-29 | tenant-admin | to enable Premium-tier DRM (Widevine + FairPlay + PlayReady) | content protection | drm-stub | Should |
| FR-30 | tenant-admin | to disable monetization-stub T2 capability (off by default) | tenants choose monetisation | monetization-stub | Must |
| FR-31 | tenant-admin | to disable live-streaming-stub (off at M03; M05-onward activation) | scope creep avoided | live-streaming-stub | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Feed-load latency (top 10 videos) | ≤ 80ms | ≤ 250ms | ≤ 500ms | ≤ 1.2s | Valkey hot-feed cache; ranking precomputed for hot accounts |
| Video-start latency (first frame) | ≤ 150ms | ≤ 400ms | ≤ 800ms | ≤ 1.8s | CDN-edge HLS segment + ABR start-up |
| Transcode 60s video (5 bitrate rungs) | ≤ 15s | ≤ 30s | ≤ 60s | ≤ 120s | ffmpeg 7.x sandboxed worker pool (gVisor) |
| Like-action latency | ≤ 15ms | ≤ 35ms | ≤ 50ms | ≤ 150ms | Redis-buffered + Postgres flush |
| Copyright-claim match (per ingest) | ≤ 800ms | ≤ 2s | ≤ 4s | ≤ 8s | Chromaprint audio-fingerprint + DCT perceptual-hash |
| Thumbnail-gen | ≤ 800ms | ≤ 2s | ≤ 4s | ≤ 8s | ffmpeg thumbnail extraction + animated-gif compositor |
| Auto-caption (per video duration × 0.3) | n/a | video_duration × 0.3 | video_duration × 0.5 | video_duration × 0.8 | foundry-runtime ASR (e.g., Whisper-class) |
| Moderation classifier verdict (NSFW + violence + minor-protection) | ≤ 200ms | ≤ 500ms | ≤ 1s | ≤ 2s | foundry-runtime T1/T2 |
| Comment / reply create | ≤ 30ms | ≤ 100ms | ≤ 250ms | ≤ 700ms | Postgres insert |
| Reaction add | ≤ 15ms | ≤ 50ms | ≤ 120ms | ≤ 300ms | Redis-buffered + Postgres flush |
| Trending-sound compute | n/a | n/a | n/a | n/a | batched 5min windowed |
| Notification fanout (10k followers) | ≤ 200ms | ≤ 1s | ≤ 2s | ≤ 5s | per-recipient async via Valkey Streams (Redis wire-compat) |
| Notification fanout (1M followers; celebrity) | ≤ 1s | ≤ 5s | ≤ 15s | ≤ 60s | sharded fanout workers |
| Feed-render content-policy correctness | 100 % | 100 % | 100 % | 100 % | zero-tolerance SLO for cross-context + minor-protection + DRM-tier violations |
| DRM license issuance | ≤ 50ms | ≤ 150ms | ≤ 300ms | ≤ 700ms | EME licence-acquisition |

### Security

- Upload + post + watch reads enforced server-side via Cedar policy (`policy/tenant-scope.cedar` + `policy/public-read.cedar`); client never trusted.
- Personal-tier shorts profile + posts: tenant operators + oyatie operators MUST NOT have plaintext disclosure access (inherited from Bominal ADR-0208).
- Professional-tier shorts posts tenant-DEK encrypted (envelope encryption per Bominal ADR-0111); admin disclosure requires four-eyes audit trail per Bominal ADR-0215.
- Media uploads scanned via OPSWAT MetaDefender or ClamAV before transcode; quarantine bucket pattern; transcode workers run in gVisor sandbox per threat-model T-E-05.
- All WebSocket connections mTLS-terminated; per-tenant API token bound at OpenBao with rotation 30d.
- Search index excludes redacted PII / PHI per `policy/redaction-phi.md` (pack-us-healthcare overlay where shorts ever ingest patient-ed content; default OFF).
- Cross-context routing forbidden: a Personal short cannot become a Professional short; enforced by `policy/dual-context-isolation.md`.
- Federation egress (ActivityPub video): inherits social posture per ADR-SOC-0004; Personal-tier NEVER federates.
- Minor-protection: Cedar policy refuses adult-only content surfacing to minor accounts; DRM cannot be bypassed via screen-scrape (best-effort EME).
- Copyright-claim worker accesses fingerprint corpus over mTLS + SPIFFE; corpus is `INTERNAL_ONLY` + `AUDIT`.

### Audit + Compliance

- Every video-upload / video-publish / video-delete / repost-stitch / repost-duet / moderation-verdict / appeal-action / copyright-claim / counter-notice / DMCA-takedown / hold event writes an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- Professional-tier disclosure requires two distinct approving principals + reason code (per Bominal ADR-0215).
- Retention: per-pack bounds in `policy/data-residency.md`. KR PIPA work-record floor satisfied. GDPR storage-limitation honored. HIPAA pack (if patient-ed use): PHI retention 6y.
- DSA Art. 23 (EU): every moderation action emits to a per-tenant transparency log; tenant publishes per Art. 24 obligations.
- EU AI Act 2024/1689 high-risk classification applies to: (a) NSFW + violence + minor-protection content-moderation classifier, (b) feed-ranking model, (c) copyright-claim auto-match (advisory-only; humans confirm). Transparency + risk-management + post-deployment monitoring obligations per Arts. 9–15 satisfied via `capabilities/T2-auto.yaml` evidence pipeline + ADR-SHORTS-0003.
- DMCA Title II Safe Harbor: takedown + counter-notice + repeat-infringer + designated agent registration (pack-us).
- EU AVMSD 2018/1808 + EU DSA Arts. 14/16/17/20/24/27/28: video-sharing-platform obligations satisfied (notice-and-action, Statement of Reasons, appeal workflow, transparency reports, minor protection).

### Availability + SLO

- Availability target: 99.95 % monthly for video-upload + feed-load + video-playback (CDN-fronted).
- Notification fanout is best-effort; 99.9 % monthly.
- Transcode pipeline: 99.5 % monthly (ffmpeg worker pool tolerates per-worker failures).
- Copyright-claim match: 99.9 % monthly (must complete pre-publication).
- RTO: ≤ 15 min for video-store. RPO: ≤ 5 min (cross-region replication for professional-tier).

### Data residency

- Per-tenant pack pinning per ADR-0117. Personal-tier user data follows the personal-residency model (per-user); professional follows tenant-residency.
- Video blobs + transcode variants follow source-post pack; CDN POPs within pack region.
- Federation egress (ActivityPub video) is per-tenant opt-in for the Professional tier only; subject to SCC + tenant attestation; Personal tier forbidden.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). Layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-redis`, `adapter-s3`, `adapter-cloudfront`, `adapter-meilisearch`, `adapter-ffmpeg`, `adapter-clamav`, `adapter-opswat`, `adapter-widevine`, `adapter-fairplay`, `adapter-playready`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `video-upload` | `oya-shorts-video-upload-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,sdk,app}` | Multipart upload session; scan-first lifecycle (quarantine→clean→prod); upload-resume; per-tenant rate limit | `UploadSession`, `VideoBlob`, `ScanVerdict`, `UploadETag` |
| `video-transcode` | `oya-shorts-video-transcode-{kernel,domain,usecase,api,adapter,adapter-ffmpeg,adapter-s3,worker,sdk}` | Multi-bitrate HLS + DASH ladder; H.264 + H.265 + AV1 + AAC + Opus; CMAF segment writer; gVisor-sandboxed workers | `TranscodeJob`, `BitrateRung`, `Manifest`, `Segment`, `Codec` |
| `video-storage` | `oya-shorts-video-storage-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-cloudfront,sdk}` | S3 with per-tenant prefix isolation + KMS SSE; CloudFront-class CDN tier; signed-URL TTL ≤ 15min | `BlobRef`, `CdnUrl`, `SignedManifest`, `PrefixScope` |
| `thumbnail-generation` | `oya-shorts-thumbnail-generation-{kernel,domain,usecase,api,adapter,adapter-ffmpeg,adapter-s3,worker,sdk}` | Poster JPEG + animated GIF + WebP preview; emoji + reaction overlays | `ThumbnailJob`, `PosterFrame`, `AnimatedPreview` |
| `audio-track-library` | `oya-shorts-audio-track-library-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,sdk}` | Licensed audio catalog + UGC sounds; per-pack licensing metadata; tenant-uploaded original sounds | `AudioTrack`, `LicensedTrack`, `UgcSound`, `LicensingTier` |
| `audio-attribution` | `oya-shorts-audio-attribution-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Per-video sound attribution; rights chain; sound-of-the-week derivation source | `AudioAttribution`, `SoundUsage`, `RightsChain` |
| `video-composition` | `oya-shorts-video-composition-{kernel,domain,usecase,api,adapter,adapter-ffmpeg,worker,sdk,app}` | Server-side clip + cut + sticker + caption overlay finalisation when client preview is partial; preview-only path retained for client-side composers | `Clip`, `Cut`, `StickerOverlay`, `CaptionOverlay`, `Composition` |
| `feed-timeline` | `oya-shorts-feed-timeline-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}` | Chronological + For-You algorithmic feed; precompute for hot accounts; fanout-on-read for cold | `FeedEntry`, `ForYouSlot`, `RankSnapshot`, `FanoutPlan` |
| `watch-time-tracking` | `oya-shorts-watch-time-tracking-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Per-(viewer, video) watch-seconds + completion-ratio + scroll-velocity signal; ranking input | `WatchSession`, `WatchTotal`, `CompletionRatio`, `ScrollVelocity` |
| `like-share-comment` | `oya-shorts-like-share-comment-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Like + share + comment CRUD; conflict-free reaction counters; per-user idempotent | `Like`, `Share`, `Comment`, `ReactionTally`, `ShareTarget` |
| `repost-stitch-duet` | `oya-shorts-repost-stitch-duet-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-ffmpeg,worker,sdk}` | Stitch (clip-and-append) + Duet (side-by-side composition); rights-check before composition | `StitchPost`, `DuetPost`, `RemixChain`, `SourceConsent` |
| `hashtag` | `oya-shorts-hashtag-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | #tag parse + per-tag corpus + trending input emission | `Hashtag`, `HashtagCorpus`, `HashtagEmission` |
| `trending` | `oya-shorts-trending-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Windowed trend compute over hashtags + sounds + entities; sound-of-the-week derivation | `TrendWindow`, `TrendRank`, `SoundOfTheWeek` |
| `notifications` | `oya-shorts-notifications-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}` | Real-time + digest delivery; per-recipient idempotent; backpressure-coalesced | `Notification`, `DigestBucket`, `RealtimeFrame` |
| `content-moderation` | `oya-shorts-content-moderation-{kernel,domain,usecase,api,adapter,adapter-clamav,adapter-opswat,worker,sdk}` | AI-classifier verdicts (NSFW + violence + minor-protection); manual reviewer queue; appeal workflow; abuse-report ingestion; EU AI Act high-risk | `ModerationVerdict`, `AbuseReport`, `Appeal`, `ClassifierVersion`, `MinorProtectionVerdict` |
| `copyright-claim` | `oya-shorts-copyright-claim-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Content-ID-class fingerprint match (Chromaprint audio + DCT perceptual-hash video); DMCA takedown + counter-notice + repeat-infringer | `FingerprintMatch`, `CopyrightClaim`, `CounterNotice`, `RepeatInfringerRecord`, `Strike` |
| `age-gate` | `oya-shorts-age-gate-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Pack-aware age-gate; signup attestation; minor-protection routing | `AgeAttestation`, `AgeBracket`, `MinorProtectionPolicy` |
| `parental-controls` | `oya-shorts-parental-controls-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Linked-account parental supervision; per-minor controls (screen-time, content-filter level, DM-restriction) | `ParentalLink`, `ParentalControlPolicy`, `MinorScreenTime` |
| `accessibility-captions` | `oya-shorts-accessibility-captions-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,worker,sdk}` | Auto-caption (foundry-runtime ASR T1 capability) + manual override; WebVTT + TTML emission | `Caption`, `CaptionTrack`, `WebVttManifest`, `TtmlManifest` |
| `creator-analytics` | `oya-shorts-creator-analytics-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Per-creator dashboards: watch-time, audience demographics, posting cadence, audience growth | `CreatorMetric`, `AudienceSlice`, `PostingCadence`, `AudienceGrowth` |
| `monetization-stub` | `oya-shorts-monetization-stub-{kernel,domain,usecase,api,sdk}` | Stub T2 capability (off by default); tip-jar + creator-fund placeholder; M04-onward activation per ADR successor-IP | `TipJarStub`, `CreatorFundStub` |
| `live-streaming-stub` | `oya-shorts-live-streaming-stub-{kernel,sdk}` | Stub-only at M03; M05-onward activation; signals scope-creep refusal | `LiveSessionStub` |
| `drm-stub` | `oya-shorts-drm-{kernel,domain,usecase,api,adapter,adapter-widevine,adapter-fairplay,adapter-playready,sdk}` | Widevine + FairPlay + PlayReady EME license issuance; tenant-tier gated (Premium tier) | `DrmLicense`, `KeySystem`, `LicenseRequest`, `DrmTier` |

Naming justification — `video-upload`:

```
NAME: oya-shorts-video-upload-<layer>
JUSTIFICATION:
- microservice = shorts: per ADR-0131 per-microservice flat layout.
- bc-tokens = video-upload: primary BC. ADR-0056 v4.1 BC-optionality rule honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres / -adapter-s3 / -adapter-cloudfront /
  -adapter-meilisearch / -adapter-ffmpeg / -adapter-clamav / -adapter-opswat /
  -adapter-widevine / -adapter-fairplay / -adapter-playready are canonical
  *-adapter-<backend> per ADR-0105 Amendment 3.
```

Total crates introduced: **~140** (22 BCs × ~6-8 layers per BC; backend-qualified adapters per ADR-0105 Amendment 3 swell post-composition + transcode + DRM substrates).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `VideoBlobStore` | `oya-shorts-video-upload-kernel` | `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT`, sometimes `PII_IDENTIFYING` (faces) |
| `UploadSessionRepository` | `oya-shorts-video-upload-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `INTERNAL_ONLY` |
| `MalwareScanner` | `oya-shorts-video-upload-kernel` | `-adapter-opswat` / `-adapter-clamav` | `INTERNAL_ONLY` |
| `VideoTranscoder` | `oya-shorts-video-transcode-kernel` | `-adapter-ffmpeg` | `INTERNAL_ONLY` |
| `ManifestWriter` | `oya-shorts-video-transcode-kernel` | `-adapter-s3` | `INTERNAL_ONLY` |
| `CdnInvalidator` | `oya-shorts-video-storage-kernel` | `-adapter-cloudfront` | `INTERNAL_ONLY` |
| `ThumbnailGenerator` | `oya-shorts-thumbnail-generation-kernel` | `-adapter-ffmpeg` | `INTERNAL_ONLY` |
| `AudioTrackRepository` | `oya-shorts-audio-track-library-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` (licensing metadata) |
| `AttributionStore` | `oya-shorts-audio-attribution-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `FeedCache` | `oya-shorts-feed-timeline-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `WatchTimeStore` | `oya-shorts-watch-time-tracking-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_QUASI_IDENTIFIER` |
| `LikeStore` | `oya-shorts-like-share-comment-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `FingerprintMatcher` | `oya-shorts-copyright-claim-kernel` | `-adapter` (Chromaprint + DCT) | `INTERNAL_ONLY` + `AUDIT` |
| `ClaimStore` | `oya-shorts-copyright-claim-kernel` | `-adapter-postgres` | `AUDIT`, `PII_IDENTIFYING` (claimant ref) |
| `ModerationClassifier` | `oya-shorts-content-moderation-kernel` | `-adapter` (foundry-runtime; T2) | `INTERNAL_ONLY` |
| `AgeAttestationStore` | `oya-shorts-age-gate-kernel` | `-adapter-postgres` | `PII_QUASI_IDENTIFIER` + `SENSITIVE_CHILD_PROTECTION` |
| `ParentalLinkStore` | `oya-shorts-parental-controls-kernel` | `-adapter-postgres` | `PII_IDENTIFYING` + `SENSITIVE_CHILD_PROTECTION` |
| `CaptionStore` | `oya-shorts-accessibility-captions-kernel` | `-adapter-postgres` + `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT` |
| `DrmLicenseIssuer` | `oya-shorts-drm-kernel` | `-adapter-widevine` / `-adapter-fairplay` / `-adapter-playready` | `INTERNAL_ONLY`, `SECRET` (per-content keys) |
| `CedarShortsPolicy` | `oya-shorts-video-upload-kernel` (cross-BC) | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `shorts` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice shorts` — dependency-direction
- `oya gate validate lean-a2 --microservice shorts` — cross-product-refusal
- `oya gate validate port-location --microservice shorts`
- `oya gate validate layer-correctness --microservice shorts`
- `oya gate validate per-microservice-layout --microservice shorts`
- `oya gate validate statelessness --microservice shorts`
- `oya gate validate shardability --microservice shorts`
- `oya gate validate authority-cohesion --microservice shorts` (HG-SHORTS)
- `oya gate validate dual-context-isolation --microservice shorts` (per parallel ADR-0238)
- `oya gate validate eu-ai-act-conformance --microservice shorts` (per ADR-SHORTS-0003)
- `oya gate validate eu-dsa-conformance --microservice shorts`
- `oya gate validate pack-aware-age-gate --microservice shorts` (per ADR-SHORTS-0006)
- `oya gate validate dmca-safe-harbor-conformance --microservice shorts`

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `VideoUploaded` | creator uploads | scan worker, transcode worker, copyright-claim worker, audit-chain | append-only |
| `VideoPublished` | publication post-scan + post-transcode + post-claim-clear | feed-timeline, search-index, mentions, hashtag, audit-chain, downstream Workflow engines | append-only |
| `VideoEdited` | creator edits caption / cover within edit-window | search-index, audit-chain | append-only delta |
| `VideoDeleted` | creator / admin deletes | search-index, audit-chain, retention-purge worker | tombstone |
| `StitchCreated` / `DuetCreated` | repost via stitch/duet | feed-timeline, audit-chain | append-only |
| `LikeAdded` / `LikeRemoved` | viewer reacts | feed-timeline, ranking input | append-only |
| `ShareEmitted` | viewer shares (in-app or external) | downstream engines, audit-chain | append-only |
| `CommentPublished` | viewer comments | feed-timeline, mentions, audit-chain | append-only |
| `WatchSessionRecorded` | viewer plays ≥ 1s | ranking input, creator-analytics | append-only batch |
| `MentionEmitted` | mentions BC resolves a mention | notifications, social/messenger bridges, action-card consumer | append-only |
| `HashtagEmission` | post carries hashtags | trending, search-index | append-only |
| `ModerationVerdictEmitted` | classifier or reviewer issues verdict | feed-timeline (hide/show), notifications (sender), audit-chain | append-only |
| `AppealOpened` / `AppealResolved` | end-user appeals; reviewer resolves | audit-chain, notifications | append-only |
| `AbuseReportFiled` | end-user files abuse report | content-moderation, audit-chain | append-only |
| `CopyrightClaimFiled` | rights-holder files claim | copyright-claim, feed-timeline (auto-hide candidate), audit-chain | append-only |
| `CounterNoticeFiled` | creator counter-notices | copyright-claim, audit-chain | append-only |
| `DmcaTakedownExecuted` | tenant-admin executes DMCA takedown | video-storage (block), feed-timeline (hide), audit-chain | append-only |
| `RepeatInfringerFlagged` | DMCA repeat-infringer policy threshold reached | tenant-admin, audit-chain | append-only |
| `AgeAttestationRecorded` | user signup attests age | age-gate, parental-controls (if minor), audit-chain | append-only |
| `ParentalLinkEstablished` | parent links a minor account | parental-controls, audit-chain | append-only |
| `CaptionGenerated` | accessibility-captions BC emits caption | feed-timeline, audit-chain | append-only |
| `EDiscoveryHoldOpened` / `Closed` | compliance-officer action | audit-chain, retention-purge worker | append-only |
| `FourEyesDisclosureExecuted` | tenant-admin pair approves Professional PII read | audit-chain | append-only |
| `DrmLicenseIssued` | EME license request fulfilled | audit-chain | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyEntityChanged` (Person/Sound/Hashtag/Sticker) | ontology | mentions + audio-attribution | refresh resolution cache |
| `SocialDeepLinkRequested` | social | video-upload | resolve video URL → embed metadata |
| `MessengerDeepLinkRequested` | messenger | video-upload | resolve video URL → embed metadata |
| `TenantRetentionPolicyUpdated` | tenancy | video-storage | reassign video retention bounds |
| `AuditChainSealed` | audit-chain | (read-only) | confirm audit-write durability |
| `WorkflowStudioRunStarted/Completed` | workflow-engine | notifications | post status into bound profile |
| `FoundryClassifierVersionPromoted` | foundry-runtime | content-moderation + copyright-claim | refresh model card + transparency label |

### Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Video{video_id, creator_ref, tenant_id, context_kind, visibility, posted_at, data_class}` | `video-upload` | Ed25519 |
| `Sound{sound_id, kind: licensed | ugc, attribution_ref, pack, first_seen_at}` | `audio-track-library` + `audio-attribution` | Ed25519 |
| `Sticker{sticker_id, tenant_id, kind}` | `video-composition` | Ed25519 |
| `Hashtag{hashtag, pack, first_seen_at}` (hashtag → topic promotion) | `hashtag` + `trending` | Ed25519 |
| `CopyrightClaim{claim_id, target_video_id, claimant_ref, status}` | `copyright-claim` | Ed25519 |
| `Mention{video_id, target_ref, mention_kind}` | mentions sub-flow within `like-share-comment` | Ed25519 |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Sound`, `Hashtag`, `Sticker` | mentions/audio-attribution | `find_by(@-handle | sound_id | hashtag, tenant_id, pack)` |
| `RetentionPolicy` | `video-storage` | `lookup(tenant_id, context_kind)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |
| `MinorProtectionPolicy` | `parental-controls` + `feed-timeline` + `content-moderation` | `lookup(pack, age_bracket)` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| TikTok | short-video feed + For-You ranking + stitch + duet + sounds | full feature parity; ranking sophistication; sound-of-the-week | (proprietary; benchmark via 3rd-party studies) |
| Instagram Reels (Meta) | short-video feed in Instagram | feed-tile + Reels-equivalent; Meta-graph leverage | `developers.facebook.com/docs/instagram-platform` |
| YouTube Shorts (Google) | short-video feed in YouTube | YouTube-graph leverage; long-form sibling | `developers.google.com/youtube` |
| Snapchat Spotlight | short-video feed in Snapchat | mobile-first; ephemeral | `kit.snapchat.com` |
| Twitter/X video | short-video + Reels-style on X | scale; algorithmic feed | `developer.x.com` |
| Likee | short-video; emerging-market focus | SEA + LATAM presence | (limited public docs) |
| Triller | short-video + creator-fund | creator-monetisation differentiator | (limited public docs) |
| Lemon8 (ByteDance) | lifestyle short-video | lifestyle vertical | (limited public docs) |
| Tangi (Google) | how-to short-video | how-to vertical | (limited public docs) |
| Kuaishou | short-video + live-streaming | CN-market; sister-product to TikTok | (limited public docs) |
| Douyin (ByteDance CN) | TikTok's CN sister | TikTok CN-specific | (limited public docs) |
| Spotify Stories (deprecated) | short stories with audio focus | accessibility-captions pattern reference | `developer.spotify.com` |
| Vimeo Short | short-video on Vimeo | creator-focused | `developer.vimeo.com` |

Key parity gaps to close (ordered by priority):

1. **Dual-context isolation by data-model** — none of the competitors enforce personal/professional context as a data-model invariant. Target: compile-time + LEAN-lane enforcement.
2. **Minor-protection at the regulatory floor** — TikTok / Reels have been fined repeatedly for COPPA + GDPR Art. 8 + UK OSA + CA AB-2273 + UT Social Media Regulation Act violations. Target: pack-aware age-gate by default; parental controls as first-class BC; minor accounts get chronological-only + algorithmic-recommendation-opt-out per default.
3. **Native Workflow + Ontology integration** — competitors expose webhooks/Graph APIs; oyatie exposes typed Workflow events + Ontology object writes natively.
4. **OpenSLO + agentic gate** — none gate feature rollouts on SLO compliance; oyatie does (per ADR-0139).
5. **Multi-pack residency + per-pack regulatory overlays** — competitors are SaaS-region-coarse; oyatie is per-pack jurisdiction-pinned.
6. **EU AI Act high-risk transparency** — competitors lag on Art. 50 transparency labels for moderation + ranking; oyatie ships from day-1 per `capabilities/T2-auto.yaml`.
7. **Copyright-claim integrity** — competitors' Content-ID systems are opaque to creators. oyatie publishes counter-notice path + repeat-infringer policy + audit-chain seal per claim.
8. **DRM tenant-tier gating** — competitors don't expose DRM at tenant granularity; oyatie's Widevine + FairPlay + PlayReady is per-tenant Premium-tier feature.
9. **Per-pack content moderation overlay** — competitors apply a global moderation policy with regional carve-outs; oyatie applies per-pack regulatory floor (KR PIPA + EU DSA + UK OSA + CA AB-2273 + UT SMRA).

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for video-upload + video-playback availability: 0.05 % (≈ 22 min/month).
- Burn-rate alarm on `shorts.video-upload.availability` is 14.4× burn rate over 1h.
- Error budget policy: `microservices/shorts/runbooks/error-budget-policy.md` (Slice B).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for upload sessions + claims + age + parental + audio-track-library + analytics; Valkey for feed cache + watch-position + like-counters + trending; S3 for video blobs + transcode variants; CloudFront-class CDN for delivery; Meilisearch for hashtag + sound search; ffmpeg worker pool for transcode; DRM-license issuer stateless beyond per-content-key Cache.

**Active-active compatibility**: stateless REST + worker pods + Postgres logical-replicated within pack; Valkey primary-replica HA; S3 cross-AZ replication; CDN naturally edge-distributed.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active creators / cell | 100k | 1M | upload-queue depth > 70% |
| Concurrent viewers / cell | 500k | 5M | feed-load p95 > 250ms |
| Video uploads/sec sustained | 50 | 1000 | transcode worker pool saturation |
| Video plays/sec | 5k | 100k | CDN egress > 70% |
| Total videos per pack | 10M | 1B | S3 prefix-cardinality limit |
| Audio-track corpus per pack | 100k | 10M | Meilisearch shard saturation |
| Total fingerprint-corpus entries | 10M | 1B | fingerprint-matcher latency p95 > 2s |

Scale-out policy:
- HPA on REST pods: CPU > 70 %, min 8, max 200 replicas.
- ffmpeg transcode worker pool: queue-depth-based (separate KEDA-style autoscaler); min 16, max 1000 workers.
- Postgres shard-by-tenant once cell hits 1000 upload/sec aggregate.
- Valkey cluster sharding by `(tenant_id, video_id) mod N`.
- CDN POP-presence per pack region; multi-region for high-fanout videos.

Sharding:
- Video metadata store partitions by `(tenant_id, creator_ref, year-month)`.
- Watch-time store partitions by `(tenant_id, viewer_ref mod N)`.
- Feed cache partitions by `(tenant_id, viewer_ref mod N)`.
- Fingerprint corpus partitions by `(pack, fingerprint_prefix mod N)`.
- `oya-check-shardability-cli` lane verifies partition keys are present in every kernel struct.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A profile-create + video-upload + transcode + publish + view + like + comment + share roundtrip completes within p99 < 5s end-to-end (transcode-dominant) | `microservices/shorts/tests/e2e/full-cycle.rs` |
| AC-02 | Personal-context shorts cannot post under Professional tenant context | `tests/e2e/dual-context-isolation.rs` |
| AC-03 | Professional shorts admin disclosure requires two distinct approving principals + audit-chain seal | `tests/e2e/four-eyes-disclosure.rs` |
| AC-04 | Video upload → scan → transcode (5 bitrate rungs) → finalize → revoke after retention TTL | `tests/e2e/video-lifecycle.rs` |
| AC-05 | Copyright fingerprint match on a known infringing clip flags within 2s p95 + auto-hide before publication | `tests/e2e/copyright-fingerprint-match.rs` |
| AC-06 | DMCA takedown + counter-notice + repeat-infringer policy threshold cycle works end-to-end with audit-chain seal | `tests/e2e/dmca-cycle.rs` |
| AC-07 | Notification fanout to 10k followers within 2s p99 | `tests/e2e/notification-fanout.rs` |
| AC-08 | Hashtag + sound search returns only Cedar-permitted results | `tests/e2e/search-cedar-scope.rs` |
| AC-09 | Moderation classifier verdict (NSFW + violence) → audit-chain seal within 2s + appeal-workflow opens | `tests/e2e/moderation-appeal.rs` |
| AC-10 | Age-gate: minor signup on pack-eu requires parental consent attestation + parental-link is created | `tests/e2e/age-gate-pack-eu.rs` |
| AC-11 | Minor account default = chronological feed + algorithmic-recommendation-opt-out + DM-restricted | `tests/e2e/minor-account-defaults.rs` |
| AC-12 | DRM Widevine + FairPlay + PlayReady license issuance for Premium-tier tenant succeeds; non-Premium-tier denied | `tests/e2e/drm-tier-gating.rs` |
| AC-13 | `oya gate validate per-microservice-layout --microservice shorts` exit 0 | ADR-0131 lane |
| AC-14 | `oya gate validate authority-cohesion --microservice shorts` exit 0 | ADR-0123 lane; HG-SHORTS registered |
| AC-15 | `oya gate validate dual-context-isolation --microservice shorts` exit 0 | per parallel ADR-0238 |
| AC-16 | EU AI Act transparency label appears on every moderation verdict + ranking explanation on pack-eu | `tests/e2e/eu-ai-act-transparency.rs` |
| AC-17 | Monetization-stub + live-streaming-stub T2 capabilities are disabled by default; admin opt-in required | `tests/e2e/stubs-default-off.rs` |
| AC-18 | Auto-caption (WebVTT) produced within video_duration × 0.3 p95 for English; manual-override path works | `tests/e2e/captions.rs` |
| AC-19 | Watch-time tracking with completion-ratio + scroll-velocity emitted to ranking input | `tests/e2e/watch-time.rs` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Live-streaming-stub: keep as stub vs activate in M05-onward vs split into separate sibling µservice | axis-shorts + council-architecture | ADR-SHORTS successor-IP after M04 |
| 2 | Monetization-stub: keep interface-only-pending-impl vs delete vs activate-with-tenant-opt-in (tip-jar + creator-fund) | council-architecture + gtm | ADR-SHORTS successor-IP after M03 |
| 3 | DRM substrate: ship all three (Widevine + FairPlay + PlayReady) at M03 or sequence; Premium-tier-only gating threshold | council-architecture + ops-security | ADR-SHORTS-0004 |
| 4 | Ranking-model openness: closed-weights vs published-weights for EU AI Act audit transparency (paired with social Open Q 1) | axis-shorts + council-privacy | ADR-SHORTS successor-IP subsequent-to-M04-completion |
| 5 | Federation (ActivityPub video): inherit social posture or shorts-specific (more conservative due to copyright)? | axis-shorts + council-architecture | ADR-SHORTS successor-IP post federation minimum-shippable-tier |
| 6 | Per-tenant ranking weights: scheduled-for-distinct-tracked-work to M04-onward per social pattern | axis-shorts | ADR-SHORTS successor-IP |
| 7 | Fingerprint-corpus governance: per-tenant private corpus + global licensed corpus split | axis-shorts + ops-legal | ADR-SHORTS-0002 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | personal/professional data-use invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum + Amendment 3 | layer + backend-qualified authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0135 | Connect dissolution (parallel) | dual-context isolation source; shorts as a sibling µservice |
| ADR-0139 | Agentic SLO-gated promotion | gates shorts releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | factored Connect into surfaces |
| ADR-0133 | Industry best-practice conformance | HG-SHORTS under this |
| ADR-SHORTS-0001 | Video transcode pipeline | this µservice |
| ADR-SHORTS-0002 | Copyright claim system (Content-ID-class) | this µservice |
| ADR-SHORTS-0003 | Content-moderation classifier bounds (EU AI Act high-risk) | this µservice |
| ADR-SHORTS-0004 | DRM substrate + tenant-tier gating | this µservice |
| ADR-SHORTS-0005 | Feed ranking algorithm (paired with social ADR-SOC-0001) | this µservice |
| ADR-SHORTS-0006 | Minor protection + age-gate (pack-aware) | this µservice |
| ADR-SOC-0006 | Media transcode + storage (sibling reference; image substrate pattern) | sibling |
| Bominal ADR-0208 | Connect dual-context unified channel hub | inherited |
| Bominal ADR-0215 | Connect retention legal-hold dual-context | inherited |
| Bominal ADR-0028 | Audit-chain Merkle + Ed25519 | inherited |
| Bominal ADR-0111 | Ciphertext property type + envelope encryption | inherited |
