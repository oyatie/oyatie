---
doc_class: RetiredMicroserviceMarker
microservice: shorts
status: Retired
retired_on: 2026-05-21
retirement_wave: Wave 15O
successor: microservices/social/
retirement_protocol: ADR-0138
retirement_adr: docs/decisions/ADR-0334-shorts-microservice-merged-into-social.md
---

# shorts µservice — RETIRED

Retired 2026-05-21 per ADR-0334.

Reason: shorts is a flavor of social (TikTok-style short-video), not a separate concern per ADR-0132 no-suite policy. Industry precedent (Instagram Reels inside Instagram, YouTube Shorts inside YouTube, X video inside X, LinkedIn video inside LinkedIn) places short-form video inside the social product surface, not in a sibling service.

Absorbed by: social µservice (`microservices/social/`).

See: `docs/decisions/ADR-0334-shorts-microservice-merged-into-social.md`.

## Successor authority

- Product requirements: `microservices/social/PRD.md` (short-video flavor section)
- Architecture: `microservices/social/ARCHITECTURE.md` (short-video composition, delivery, copyright-claim, DRM sections)
- Manifest: `microservices/social/manifest.json`
- Contracts: `microservices/social/contracts/{openapi,asyncapi,proto}/`
- Cedar fragments: `microservices/social/policy/*.cedar`
- SLOs: `microservices/social/slos/*.openslo.yaml`
- Runbooks: `microservices/social/runbooks/*.md`

## Migrated responsibilities

- Video upload, multi-bitrate HLS/DASH ABR transcode, video storage and CDN delivery
- Thumbnail generation and composition (clip, cut, sticker, caption overlay)
- Audio-track library (licensed + UGC) and audio attribution
- Feed timeline (algorithmic + chronological) for short-video posts
- Watch-time tracking as a ranking signal
- Like, share, comment, repost (Stitch and Duet variants via `Post.derives_from`)
- Hashtag and sound-of-the-week trending
- Content moderation classifiers (NSFW, violence, minor-protection video)
- Copyright-claim (Content-ID-class fingerprint matching, DMCA takedown, counter-notice, repeat-infringer registry)
- Age gate and parental controls
- Accessibility captions (auto-generated + manual)
- Notifications and creator analytics
- DRM substrate (Widevine + FairPlay + PlayReady), tenant-class gated per ADR-0330
- HG-SHORTS conformance gate folded into HG-SOCIAL

## Not migrated

- No live `oya-shorts-*` service crates are produced after this ADR.
- No standalone shorts OpenAPI / AsyncAPI / proto contract surface remains; clients use social contracts with media-kind discriminators.
- The `axis-shorts` ownership team folds into `axis-social`.

## Related retirements (this session)

- Wave 15K: `network` µservice retired; absorbed into `community` µservice (LinkedIn-class jobs and profiles surface).
- Wave 15L: `cell` µservice retired (ADR-0333); cellular architecture preserved as a pattern owned by tenancy + cloud-iac + observability + audit-chain + api-gateway + `oya-shuffle-sharding` crate.
- Wave 15O (this): `shorts` µservice retired; short-form video absorbed into `social`.

## Note for future agents

Historical content in this directory must not be treated as live. Any
short-video task starts from `microservices/social/` and ADR-0334.
