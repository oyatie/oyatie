---
id: ADR-0334
status: Superseded
superseded_by: [ADR-700]
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - axis-social
  - axis-shorts
deciders:
  - user-directive-2026-05-21
  - council-architecture
  - axis-social
  - axis-shorts
supersedes:
  - microservices/shorts/PRD.md
  - microservices/shorts/ARCHITECTURE.md
amends:
  - ADR-0238
  - ADR-0132
related:
  - ADR-0132-product-platform-and-bundle-dissolution.md
  - ADR-0138-intelligence-six-path-deprecation.md
  - ADR-0238-connect-super-app-expansion.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0317-role-based-projection-unified-ux-shell.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
related_sources:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md
  - microservices/shorts/PRD.md
  - microservices/shorts/ARCHITECTURE.md
  - microservices/shorts/manifest.json
  - microservices/social/PRD.md
  - microservices/social/ARCHITECTURE.md
  - microservices/social/manifest.json
doc_class: Architecture-Decision-Record
purpose: >
  Retire the standalone shorts microservice and absorb its short-form video
  capabilities into the social microservice as the TikTok-style short-video
  flavor of the social product, consistent with ADR-0132 no-grouping policy.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0334: shorts µservice retired; absorbed into social as short-video flavor

## Status

Accepted — 2026-05-21.

This ADR executes the 2026-05-21 user directive:

`shorts is social?` — user 2026-05-21 (confirmed: shorts is a subset of social,
not a separate µservice concern).

`lets separate the tiktok/instagram flavor to social` — user 2026-05-21.

The retirement removes a service boundary.

The retirement does not remove short-form video capabilities.

The retirement does not remove the TikTok-class user experience.

The retirement does not remove copyright-claim enforcement.

The retirement does not remove DRM, content moderation, or minor protection.

The retirement does not remove the audio-track library, feed timeline, or
creator analytics.

The retirement consolidates ownership of the short-video surface into the
social µservice, where it belongs as one media flavor within a unified social
product.

## Context

The prior `shorts` PRD described a rich short-form-video product.

It owned video upload.

It owned multi-bitrate HLS/DASH transcoding.

It owned video storage and CDN delivery.

It owned thumbnail and composition.

It owned an audio-track library and audio attribution.

It owned the For-You feed timeline and watch-time tracking.

It owned likes, shares, comments, and stitch/duet reposts.

It owned hashtags and trending sounds.

It owned content moderation, copyright-claim, age-gate, parental controls,
captions, notifications, and creator analytics.

It exposed OpenAPI, AsyncAPI, and protobuf contracts.

It carried SLOs for transcode, feed load, video start, like action,
moderation classifier, auto-caption, copyright match, and DRM license issuance.

It carried policy fragments for tenant scope, public read, data residency,
auditor scope, and CI scope.

It carried runbooks for transcode-queue, copyright-claim storm, CDN cache,
moderation rollback, DRM key rotation, and age-gate incident response.

It carried 15 baseline implementation plans plus 13 journey-spanning plans.

The 2026-05-20 coherence audit found the artifacts substantive.

The issue is not lack of substance.

The issue is scope shape.

The user revisited the service boundary on 2026-05-21.

The confirmed decision: shorts is a flavor of social, not a separate concern.

The reasoning is direct.

Industry precedents put short-form video inside social products:

- Instagram Reels is part of Instagram, not a separate service team.
- YouTube Shorts is part of YouTube, not a separate property.
- Twitter/X video is part of the timeline, not a side product.
- LinkedIn video is part of the feed, not a sibling product.

TikTok exists as a standalone product, but TikTok is itself a unified social
product, not a multi-service decomposition of social into "feed + shorts +
profile + DM".

ADR-0132 forbids new bundle and grouping µservices. Each new µservice is
single-concern and flat. A `shorts` µservice alongside `social` violates the
single-concern bar: both produce posts, both rank feeds, both carry follow
graphs, both moderate, both serve creators. The bounded contexts are the same;
only the media flavor differs.

Wave 15K (`network` → `community`) and Wave 15L (`cell` retired) established
the precedent for this session. Both pruned a service boundary while keeping
the underlying capabilities and absorbing them into the correct owner.

Therefore `shorts` retires as a service and the short-video capabilities live
inside `social` as the TikTok-style media flavor.

## Decision

D-1. `microservices/shorts/` is retired as a standalone µservice.

D-2. `microservices/shorts/` keeps only a `RETIRED.md` redirect marker.

D-3. Historical shorts service content is not the live authority after this ADR.

D-4. `microservices/social/` is the canonical owner of short-form video.

D-5. `microservices/social/` is the canonical owner of long-form video where
it appears in the social product.

D-6. Short-form video posts ride the same `Post` aggregate as text and image
posts.

D-7. Short-form video posts use a `media.kind = short_video` discriminator.

D-8. Short-form video posts use the same follow graph as other social posts.

D-9. Short-form video posts use the same content moderation pipeline as other
social posts, with media-specific classifiers chained in.

D-10. Short-form video posts use the same feed-timeline ranker as other social
posts, with media-aware features added to the ranker input vector.

D-11. The social feed-timeline kernel admits a chronological and an
algorithmic feed shape for short-video content, matching prior shorts product
behavior.

D-12. Stitch and Duet remix variants attach to the social `Post` aggregate via
a `derives_from = post_id` edge.

D-13. The audio-track library moves to social as a media composition
substrate.

D-14. Audio attribution moves to social as part of the same library.

D-15. Hashtag and trending logic uses the existing social hashtag and trending
BCs.

D-16. Sound-of-the-week trending becomes a social trending facet, not a
separate computation.

D-17. Watch-time tracking moves to social as a media-engagement signal.

D-18. Copyright-claim moves to social as a media-rights workflow.

D-19. DRM entitlement moves to social as a tenant-class gated media feature
per ADR-0330.

D-20. Content-ID-class fingerprint matching moves to social as a copyright
precheck step in post composition.

D-21. Age-gate moves to social as part of profile and post visibility policy.

D-22. Parental controls move to social as part of minor-protection policy.

D-23. Accessibility captions move to social as part of accessibility policy
already in scope.

D-24. Creator analytics move to social as part of analytics already in scope.

D-25. The DMCA Title II safe-harbor workflow moves to social.

D-26. The takedown, counter-notice, and repeat-infringer registry move to
social.

D-27. The HLS/DASH ABR transcode pipeline moves to social media composition.

D-28. Social already owns image and video transcode via ffmpeg and ImageMagick
adapters; the ABR ladder is added there.

D-29. The signed-URL CDN delivery path moves to social media delivery.

D-30. Pack-aware retention and residency for short video follow the social
pack pinning rules.

D-31. Cross-pack federation never crosses video boundaries; metadata-only
federation remains unchanged.

D-32. Workflow Studio events `VideoPublished`, `CopyrightClaimFiled`, and
`ModerationVerdictEmitted` move to the social event namespace and are
re-emitted with `media.kind = short_video` where appropriate.

D-33. The social Ontology object types `Person`, `Post`, and `Topic` cover
short-video posts; `Sound`, `Sticker`, and `Video` become projections on
`Post.media`.

D-34. The social SLO set is extended to cover short-video specific surfaces:
auto-caption latency, copyright-claim match latency, DRM license issuance
latency, video-start latency, transcode throughput, feed-load latency for
short-video feeds.

D-35. The social runbook set is extended with: transcode queue backup,
copyright-claim storm, CDN cache invalidation cascade, DRM key rotation,
moderation classifier rollback for media classifiers, and age-gate bypass
incident response.

D-36. The social Cedar policy set is extended with: short-video moderation
fragments, DRM entitlement fragments, and minor-protection fragments for
video.

D-37. The social manifest declares the absorbed scope in `bounded_contexts`
and `slos`.

D-38. The social PRD includes a short-video flavor section referencing this
ADR.

D-39. The social ARCHITECTURE walkthrough includes a short-video composition
and delivery section referencing this ADR.

D-40. New code must not import `microservices/shorts/` artifacts.

D-41. New code must not generate `oya-shorts-*` service crates.

D-42. Existing `oya-shorts-*` crate references are transition debt unless
explicitly retained as historical evidence.

D-43. Media-specific kernels (transcode, copyright-claim, feed-timeline for
video) live under the social workspace under `oya-community-social-*-kernel` and
`oya-community-social-*-adapter-*` names.

D-44. The old shorts contracts (OpenAPI, AsyncAPI, proto) are historical
after this ADR.

D-45. New contracts bind through social OpenAPI, AsyncAPI, and proto with
short-video paths and event topics added under the social umbrella.

D-46. The old shorts capability YAMLs are retired.

D-47. The old shorts runbooks are retired in place; successor runbooks live
under social runbooks.

D-48. The old shorts dashboards are retired in place; successor dashboards
live under social dashboards.

D-49. The old shorts SLOs are retired in place; successor SLOs live under
social slos.

D-50. The ADR-0138 strangler discipline applies.

D-51. Because the service retires before launch, the retirement uses the
zero-current-consumer variant.

D-52. The zero-current-consumer variant keeps a redirect marker and removes
live authority.

D-53. The redirect marker is enough because no production caller is being
migrated.

D-54. Cross-reference sweeps must route old paths to absorption targets.

D-55. Historical forensic mentions may survive only when clearly marked
historical.

D-56. Machine-readable specs must not list shorts as an active µservice after
this ADR.

D-57. Counts that included shorts as an active µservice must be corrected
when touched.

D-58. The `axis-shorts` ownership team folds into `axis-social`.

D-59. The HG-SHORTS hyperscaler-grade conformance gate folds into HG-SOCIAL.

D-60. `IP-015-drm-and-hg-shorts-registration` is retired; DRM registration
becomes part of the social HG conformance set.

D-61. Tenant-class entitlement gating for DRM follows ADR-0330 verbatim under
the social product surface.

## Absorption Map

| Retired responsibility | Successor owner | Successor authority |
|---|---|---|
| Video upload | social | `microservices/social/ARCHITECTURE.md#short-video-upload` |
| Video transcode (HLS/DASH ABR) | social | `microservices/social/ARCHITECTURE.md#short-video-transcode` |
| Video storage and CDN | social | `microservices/social/ARCHITECTURE.md#short-video-delivery` |
| Thumbnail generation | social | post-composition kernel media path |
| Audio-track library | social | `microservices/social/ARCHITECTURE.md#audio-track-library` |
| Audio attribution | social | same as audio-track library |
| Feed timeline (algorithmic + chronological short-video) | social | feed-timeline kernel, video-aware features |
| Watch-time tracking | social | feed-timeline engagement signal |
| Like / share / comment / repost | social | reactions and post-composition (already owned) |
| Stitch / Duet | social | post-composition with `derives_from` edge |
| Hashtag and trending sounds | social | hashtag and trending BCs (already owned) |
| Content moderation classifiers (NSFW, violence, minor-protection video) | social | content-moderation kernel, media-aware |
| Copyright claim (Content-ID-class) | social | `microservices/social/ARCHITECTURE.md#copyright-claim` |
| Age gate | social | age-verification BC (already owned) |
| Parental controls | social | minor-protection policy (already owned) |
| Accessibility captions | social | accessibility policy (already owned) |
| Notifications | social | notifications BC (already owned) |
| Creator analytics | social | analytics surface (already owned) |
| DRM substrate (Widevine, FairPlay, PlayReady) | social | tenant-class gated per ADR-0330 |
| HG-SHORTS conformance gate | social | HG-SOCIAL conformance gate |

## Successor Contract

C-1. Social is the post writer for short-video posts.

C-2. Social is the moderation owner for short-video posts.

C-3. Social is the copyright-claim owner for short-video posts.

C-4. Social is the DRM owner for short-video posts.

C-5. Social is the feed owner for short-video posts.

C-6. Social is the analytics owner for short-video posts.

C-7. Social is the notifications owner for short-video posts.

C-8. Identity carries the signed principal context for creators and viewers.

C-9. Tenancy persists tenant-class and pack pinning.

C-10. Cloud-iac provisions the social workload and its media storage.

C-11. Observability owns short-video SLO burn under the social labels.

C-12. Audit-chain seals short-video moderation, copyright, and DRM events.

C-13. Api-gateway routes short-video traffic to the social cell-aware routes.

C-14. Workload µservices consume short-video events from the social event
namespace, not a retired shorts namespace.

C-15. No workload µservice calls a retired shorts endpoint.

C-16. No workload µservice infers short-video ownership from a stale shorts
crate.

C-17. The only approved short-video kernel surface is under the social
workspace.

C-18. The only approved short-video contract surface is social OpenAPI,
AsyncAPI, and proto.

## Consequences

Merging `microservices/shorts/` into `microservices/social/` means short-form video is owned by social and rides the same `Post` aggregate with a `media.kind = short_video` discriminator rather than a separate service; the data-model, operational, and migration consequences are enumerated in the sections below.

## Data Model Consequences

M-1. `Post.media.kind` accepts `text`, `image`, `video`, and `short_video`.

M-2. `Post.media.short_video` carries variant_ladder, audio_track_id,
duration_ms, and copyright_claim_status.

M-3. `Post.media.short_video.derives_from` accepts a parent post id for
stitch and duet variants.

M-4. `Post.media.short_video.drm_policy` accepts platform, tenant-class
gated, and unrestricted.

M-5. `Post.audience` and `Post.visibility` remain unchanged for short-video
posts.

M-6. `FollowGraph` remains unchanged; short-video posts ride the same
follow edges.

M-7. `TrendingTopic` admits a sound-of-the-week facet.

M-8. `ModerationVerdict` admits short-video media facets (nsfw, violence,
minor-protection).

M-9. `CopyrightClaim` is a first-class social event class with takedown,
counter-notice, and repeat-infringer registry entries.

M-10. `WatchTime` is a per-(viewer, post) signal used by feed-timeline
ranking.

M-11. `MediaVariant` describes the HLS/DASH ladder per short-video post.

M-12. Audit evidence may carry short-video media identifiers.

M-13. Metrics aggregate by short-video features under social labels.

M-14. Dashboards include short-video feed health, transcode health, and
copyright-claim health under social.

M-15. Public APIs expose short-video posts through the same social post
endpoints with media discriminators.

M-16. OpenTofu modules for the social workload subsume the media storage
modules previously planned under shorts.

M-17. Cedar context includes `media.kind` for short-video moderation
decisions.

## Operational Consequences

O-1. Short-video transcoding is a social workload operation.

O-2. Short-video transcoding follows the social cloud-iac plan and apply path.

O-3. Short-video feed cache drain is a social runbook.

O-4. Short-video copyright-claim incident response is a social runbook.

O-5. Short-video moderation classifier rollback is a social runbook.

O-6. Short-video DRM key rotation is a social runbook.

O-7. Short-video age-gate bypass incident is a social runbook.

O-8. Short-video CDN cache invalidation cascade is a social runbook.

O-9. Short-video transcode queue backup is a social runbook.

O-10. Short-video SLO burn is observability-owned under social labels.

O-11. Short-video evidence drift is an audit-chain issue.

O-12. Short-video route drift is an api-gateway issue.

O-13. Short-video assignment drift is a tenancy issue.

## ADR-0132 Preservation

P-1. ADR-0132 remains active.

P-2. ADR-0132 forbids new bundle and grouping µservices.

P-3. Social as a hero product is single-concern; short-video is one media
flavor within social, not a sibling product.

P-4. Industry precedent puts short-video inside social products.

P-5. The pattern is stronger because ownership follows the natural product
boundary.

P-6. The service boundary is weaker when the same user, the same follow
graph, the same feed, the same moderation pipeline, and the same compliance
pack apply across both products.

## Rejected Alternatives

R-1. Keep `shorts` standalone.

R-2. Rejected because the bounded contexts overlap with social.

R-3. Rejected because Instagram, YouTube, X, and LinkedIn each put short
video inside the social product, not in a separate service.

R-4. Rejected because two parallel feed-timeline kernels duplicate ranking
infrastructure.

R-5. Rejected because two parallel moderation kernels duplicate compliance
infrastructure.

R-6. Convert `shorts` into a thin media-pipeline service for transcode only.

R-7. Rejected because the social composition kernel already owns image and
video transcode; carving short-video out splits a natural ownership boundary.

R-8. Convert `shorts` into a feed-only service that delegates moderation to
social.

R-9. Rejected because the For-You and chronological feed shapes are already
the social feed-timeline kernel's responsibility.

R-10. Keep old shorts docs as live references.

R-11. Rejected because live old docs preserve an incorrect service boundary.

R-12. Delete shorts evidence without a redirect.

R-13. Rejected because future agents need a deterministic retirement
pointer.

R-14. Migrate shorts into community instead of social.

R-15. Rejected because community is forum-shape (Reddit, Teamblind,
Handshake, LinkedIn jobs); short-form video is broadcast-shape, matching
social.

R-16. Migrate shorts into a hypothetical `video` µservice.

R-17. Rejected because there is no need for a media-only service; media
ownership follows the consuming product.

## Migration Plan

S-1. Author this ADR.

S-2. Replace active `microservices/shorts/` content with `RETIRED.md`.

S-3. Update `microservices/social/PRD.md` with a short-video flavor section
that cites this ADR.

S-4. Update `microservices/social/ARCHITECTURE.md` with absorbed sections
where appropriate.

S-5. Update `microservices/social/manifest.json` to declare the absorbed
scope.

S-6. Update `specs/master-plan-sequencing.json` to remove `shorts` from
the active µservice roster.

S-7. Update `specs/microservices/manifests-index.json` to remove the shorts
manifest pointer.

S-8. Update `specs/root-hub-pointers.json` historical entries to mark
shorts as retired.

S-9. Update memory note
`feedback_cell_standalone_network_merges_community_2026_05_21.md` with the
shorts → social merge.

S-10. Verify no active docs/specs references still route readers to
`microservices/shorts/`.

S-11. Report any remaining historical references or validation gaps.

## Verification

V-1. `microservices/shorts/RETIRED.md` exists.

V-2. `microservices/shorts/` has no live service artifacts after retirement.

V-3. `microservices/social/PRD.md` references this ADR for short-video.

V-4. `microservices/social/manifest.json` declares the absorbed scope.

V-5. `specs/microservices/manifests-index.json` does not list shorts as an
active manifest pointer.

V-6. `specs/master-plan-sequencing.json` does not list shorts in the active
µservice phase roster.

V-7. Docs and specs cross-reference sweep points to successor owner social.

V-8. ADR-0132 doctrine remains in force.

V-9. No commit is created by this wave.

## Completion Report

The completion report is embedded as an HTML comment so automated readers can
parse the ADR without changing the visible decision text.

<!--
wave: 15O
status: completed-locally
decision: shorts µservice retired; short-form video absorbed into social as TikTok-style media flavor
absorbing_microservice: microservices/social/
retired_marker: microservices/shorts/RETIRED.md
absorption_map_owner: microservices/social/ARCHITECTURE.md
prd_owner: microservices/social/PRD.md
manifest_owner: microservices/social/manifest.json
precedent_waves: Wave 15K network→community; Wave 15L cell retire
authority_adrs: ADR-0132 no-grouping policy; ADR-0238 dissolution; ADR-0330 tenant-class entitlement
commits: none
-->
