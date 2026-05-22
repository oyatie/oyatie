# Social Feature-Parity Matrix - 2026-05-20

Target microservice: `microservices/social/`.
Counterpart 1: TikTok.
Counterpart 2: Instagram.
Counterpart 3: Snapchat.
Audit boundary: visual and short-video social inside the unified mobile-app bundle.
Non-goal boundary: LinkedIn-style engagement feed, X/Threads text broadcast, follower-monetization-via-followers, sponsored-post-promotion, and algorithmic For-You-feed.
Current Oyatie artifact baseline: `microservices/social/PRD.md:22`, `microservices/social/README.md:18`, and `microservices/social/contracts/openapi/social.yaml:5`.
Current canonical correction: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:109-191`.
Chat-history correction: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17079`.
Tenant-class adoption boundary: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:10-43`.

## 1. Counterpart 1 Capability Surface - TikTok

1. TikTok surface family: short-video-first creation and consumption.
2. TikTok public source: For You is a personalized feed based on interests and engagement; source: `https://support.tiktok.com/en/getting-started/for-you?lang=en`, lines 182-198.
3. TikTok public source: content posting API supports media transfer by HTTP upload; source: `https://developers.tiktok.com/doc/content-posting-api-media-transfer-guide`, lines 265-272.
4. TikTok public source: chunked video uploads require 5 MB to 64 MB chunks except final chunks up to 128 MB; source: `https://developers.tiktok.com/doc/content-posting-api-media-transfer-guide`, lines 274-280.
5. TikTok public source: pull-from-URL ingest can reach 100 Mbps server ingress; source: `https://developers.tiktok.com/doc/content-posting-api-media-transfer-guide`, lines 500-504.
6. Capability: vertical short-video capture.
7. Capability: short-video edit flow.
8. Capability: music/sound attachment.
9. Capability: effects attachment.
10. Capability: captions and description.
11. Capability: creator profile.
12. Capability: follow graph.
13. Capability: like/reaction.
14. Capability: comment.
15. Capability: share.
16. Capability: repost-style amplification.
17. Capability: hashtag.
18. Capability: search and discovery.
19. Capability: personalized feed.
20. Capability: following/friends feed.
21. Capability: content eligibility and safety review.
22. Capability: reporting and moderation.
23. Capability: privacy controls.
24. Capability: direct messaging integration.
25. Capability: notification stream.
26. Capability: creator analytics.
27. Capability: content status after upload.
28. Capability: chunked upload reliability.
29. Capability: large-file media ingress.
30. Capability: public link/embed surface.
31. Capability: content recommendation explanation.
32. Capability: user control over recommendations.
33. Capability: keyword/topic filtering.
34. Capability: trend and topic discovery.
35. Capability: duet/stitch/remix-like interaction family.
36. Capability: content safety age gating.
37. Oyatie status: PRD owns post-composition, reactions, comments, hashtags, feed, moderation, notifications, and search; evidence: `microservices/social/PRD.md:41-70`.
38. Oyatie status: PRD owns images and video transcode only as media attached to posts; evidence: `microservices/social/PRD.md:76-90`.
39. Oyatie status: ADR-SOC-0006 supports images and short videos with HLS; evidence: `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:32-35`.
40. Oyatie gap: no first-class clip object in OpenAPI; evidence: `microservices/social/contracts/openapi/social.yaml:95-138`.
41. Oyatie gap: no duet/stitch/remix object in OpenAPI; evidence: `microservices/social/contracts/openapi/social.yaml:95-138`.
42. Oyatie gap: no effect/lens/sound attachment object in OpenAPI; evidence: `microservices/social/contracts/openapi/social.yaml:95-138`.
43. Oyatie gap: current feed exposes algorithmic mode, which conflicts with the current anti-pattern boundary; evidence: `microservices/social/contracts/openapi/social.yaml:450-479`.
44. Oyatie gap: PRD and ADR-SOC-0001 define algorithmic feed as core; evidence: `microservices/social/PRD.md:47` and `microservices/social/decisions/ADR-SOC-0001-feed-ranking-algorithm.md:44-60`.
45. Oyatie gap: current product target is Twitter/X-class, not TikTok-class; evidence: `microservices/social/PRD.md:22`.
46. Oyatie gap severity: P1 for product-family mismatch.
47. TikTok parity implication: preserve short-video creation, media ingest, creator safety, search/tag discovery, and share-to-message.
48. TikTok non-parity implication: do not copy engagement-optimized For-You as a product objective because current directive forbids it.
49. TikTok union contribution: strongest media-ingest and short-video creation bar.
50. TikTok union contribution: strongest creator workflow and clip status bar.
51. TikTok union contribution: strongest recommendation-control and safety-transparency pressure, but only as a constrained non-goal or explainability reference.

## 2. Counterpart 2 Capability Surface - Instagram

1. Instagram surface family: visual profile grid, feed images, reels, stories, explore, comments, DMs, creator profile.
2. Instagram public source: Instagram Explore serves hundreds of millions of people daily and ranks from billions of available options; source: `https://engineering.fb.com/2023/08/09/ml-applications/scaling-instagram-explore-recommendations-system/`, lines 41-45.
3. Instagram public source: large-scale recommenders use a funnel from thousands of candidates down to hundreds; source: `https://engineering.fb.com/2023/08/09/ml-applications/scaling-instagram-explore-recommendations-system/`, lines 57-61.
4. Instagram public source: Explore uses retrieval, first-stage ranking, second-stage ranking, and final reranking; source: `https://engineering.fb.com/2023/08/09/ml-applications/scaling-instagram-explore-recommendations-system/`, lines 47-51.
5. Instagram public source: Instagram help describes Reels recording and editing up to 20 minutes; source: `https://www.facebook.com/help/instagram/225190788256708`, search result snippet.
6. Instagram public source: Reels should have minimum 30 FPS and minimum 720 px resolution; source: `https://www.facebook.com/help/1038071743007909`, search result snippet.
7. Capability: photo post.
8. Capability: visual profile grid.
9. Capability: carousel/multi-asset post.
10. Capability: short-video reels.
11. Capability: story lifecycle.
12. Capability: highlights/archive.
13. Capability: creator profile controls.
14. Capability: follow graph.
15. Capability: likes.
16. Capability: comments.
17. Capability: saves/bookmarks.
18. Capability: shares to DMs.
19. Capability: mentions and tags.
20. Capability: hashtags.
21. Capability: location tagging.
22. Capability: Explore discovery.
23. Capability: audio/effect attachment.
24. Capability: sensitive content control.
25. Capability: privacy controls.
26. Capability: close-friends or audience scoping.
27. Capability: business/creator insights.
28. Capability: content moderation and reporting.
29. Capability: feed and profile media rendering.
30. Capability: web profile/read surface.
31. Capability: mobile-native camera import.
32. Capability: draft/edit flow.
33. Capability: notification stream.
34. Capability: content accessibility, including alt text and captions.
35. Oyatie status: PRD includes accessibility-alt-text; evidence: `microservices/social/PRD.md:22`.
36. Oyatie status: PRD includes bookmarks and lists; evidence: `microservices/social/PRD.md:22`.
37. Oyatie status: OpenAPI includes bookmarks/list concepts indirectly in high-level summary; evidence: `microservices/social/contracts/openapi/social.yaml:5`.
38. Oyatie status: current media variants include thumbnail, small, medium, large, hls_low, and hls_high; evidence: `microservices/social/contracts/openapi/social.yaml:150-167`.
39. Oyatie status: media upload max is 209,715,200 bytes; evidence: `microservices/social/contracts/openapi/social.yaml:545-572`.
40. Oyatie gap: no visual profile grid contract.
41. Oyatie gap: no story lifecycle contract.
42. Oyatie gap: no highlight/archive contract.
43. Oyatie gap: no location-tagging contract.
44. Oyatie gap: no close-friends/audience list contract separate from generic visibility.
45. Oyatie gap: no mobile camera/draft/edit product contract.
46. Oyatie gap: no creator insights contract.
47. Oyatie gap: no DM share contract tied to mobile bundle beyond messenger deep-link events.
48. Oyatie evidence for partial messenger link: `microservices/social/PRD.md:57`.
49. Oyatie evidence for missing full mobile bundle: `microservices/social/manifest.json:375-383`.
50. Instagram parity implication: visual object model must become primary.
51. Instagram parity implication: story, grid, carousel, save/share, and profile display must be first-class.
52. Instagram parity implication: Explore-style scale is a benchmark reference, but engagement-optimized ranking remains a non-goal unless reframed.
53. Instagram union contribution: strongest visual profile and story/product grammar bar.
54. Instagram union contribution: strongest consumer expectations for image/video quality, captions, alternate text, and creator account state.

## 3. Counterpart 3 Capability Surface - Snapchat

1. Snapchat surface family: camera-first ephemeral visual messaging, Stories, Spotlight, Snap Map, AR lenses, and chat-adjacent visual sharing.
2. Snapchat public source: Snap Q1 2026 reported 956 million MAU and 483 million DAU; source: `https://www.sec.gov/Archives/edgar/data/1564408/000156440826000024/snap-20260506xexx991pressr.htm`, lines 57-69.
3. Snapchat public source: Q1 2026 Spotlight posters grew about 74% year-over-year in the US and over 61% globally; source: same SEC release, lines 61-63.
4. Snapchat public source: Q1 2026 AR Lenses were used more than 9 billion times per day on average and 75% of Snapchatters engaged with AR daily on average; source: same SEC release, lines 65-67.
5. Snapchat public source: more than 400,000 Lenses were submitted in Q1 2026; source: same SEC release, lines 65-68.
6. Snapchat public source: Snap Map reached over 450 million global monthly active users in Q1 2026; source: same SEC release, line 69.
7. Snapchat public source: Spotlight posts are video Snaps and may appear in Search and Stories; source: `https://help.snapchat.com/hc/en-us/articles/7012288096532-How-do-I-post-Snaps-to-Spotlight`, lines 22-35.
8. Snapchat public source: delete is default; one-to-one and group Snaps delete after viewed, and Stories usually expire after 24 hours; source: `https://help.snapchat.com/hc/en-us/articles/7012334940948-When-does-Snapchat-delete-Snaps-and-Chats`, lines 21-67.
9. Capability: camera-first capture.
10. Capability: ephemeral snap lifecycle.
11. Capability: story expiry.
12. Capability: shared story scaling.
13. Capability: Spotlight-style public video.
14. Capability: AR lens/effect ecosystem.
15. Capability: map/location visual discovery.
16. Capability: remix/share mechanics.
17. Capability: chat-adjacent visual content.
18. Capability: notification stream.
19. Capability: public profile.
20. Capability: creator/developer lens submission.
21. Capability: safety and community guideline gating.
22. Capability: screenshot/save risk disclosure.
23. Capability: memories/archive where user saves content.
24. Capability: friend graph.
25. Capability: group chat association.
26. Capability: search.
27. Capability: topic chat/public content persistence differences.
28. Capability: age-sensitive content defaults.
29. Oyatie status: social has messenger bridge references but does not own chat; evidence: `microservices/social/IP-010-notifications-bc.md:20-23`.
30. Oyatie status: PRD cross-links posts to messenger and community; evidence: `microservices/social/PRD.md:34`.
31. Oyatie status: AsyncAPI includes messenger and mail consumption; evidence: `microservices/social/contracts/asyncapi/social-events.yaml:7-13`.
32. Oyatie status: moderation and minor-protection artifacts are present.
33. Oyatie gap: no ephemeral snap/story lifecycle.
34. Oyatie gap: no AR lens/effect lifecycle.
35. Oyatie gap: no Snap Map-style location discovery.
36. Oyatie gap: no Spotlight-like public video object.
37. Oyatie gap: no explicit save/screenshot/remix risk policy in contracts.
38. Oyatie gap: no unified mobile push coordination with messenger/mail/community.
39. Oyatie evidence for absent community dependency: `microservices/social/manifest.json:375-383`.
40. Snapchat parity implication: add ephemeral visual lifecycle and camera/effect concerns if union coverage requires Snapchat-class behavior.
41. Snapchat parity implication: integrate with messenger without moving chat ownership into social.
42. Snapchat parity implication: push and privacy defaults matter as much as feed rendering.
43. Snapchat union contribution: strongest ephemeral, AR/effects, camera-first, map, and chat-adjacent sharing bar.

## 4. Union-Coverage Matrix

| # | Capability | TikTok | Instagram | Snapchat | Current Oyatie Evidence | Status |
|---:|---|---|---|---|---|---|
| 1 | Visual post as first-class object | partial | core | partial | `microservices/social/contracts/openapi/social.yaml:95-138` | Gap: post object is text-feed shaped. |
| 2 | Short-video clip as first-class object | core | core | core | `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:32-35` | Partial: media exists, clip object absent. |
| 3 | Story/ephemeral lifecycle | adjacent | core | core | no story lifecycle in OpenAPI | Gap. |
| 4 | Public profile media grid | creator profile | core | public profile | `microservices/social/PRD.md:41-70` | Gap: profile exists but grid absent. |
| 5 | Follow graph | core | core | friend/public graph | `microservices/social/PRD.md:41-70` | Present as old model. |
| 6 | Likes/reactions | core | core | reactions/remix | `microservices/social/PRD.md:41-70` | Present. |
| 7 | Comments | core | core | limited/contextual | `microservices/social/PRD.md:41-70` | Present. |
| 8 | Saves/bookmarks | favorites | saves | memories | `microservices/social/PRD.md:22` | Partial; needs visual semantics. |
| 9 | Share to direct message | core | core | native chat | `microservices/social/PRD.md:57` | Partial; messenger gRPC handoff absent. |
| 10 | Mail action card | not native | notifications/email | not core | `microservices/social/contracts/asyncapi/social-events.yaml:317` | Partial. |
| 11 | Community discussion attach | not core | comments/groups adjacent | shared stories/groups | `microservices/social/PRD.md:34` | Partial; community dependency absent. |
| 12 | Unified mobile app session | app-specific | app-specific | app-specific | no cloud-iam session contract | Gap. |
| 13 | Unified push stream | app-specific | app-specific | app-specific | notifications BC exists | Gap: cross-bundle dedupe absent. |
| 14 | Media ingest upload | core | core | core | `microservices/social/contracts/openapi/social.yaml:545-572` | Partial. |
| 15 | Chunked upload / resumable ingest | core | platform-dependent | platform-dependent | no contract | Gap. |
| 16 | HLS playback variants | core | core | video | `microservices/social/contracts/openapi/social.yaml:150-167` | Partial. |
| 17 | Caption/alt text | captions | alt text/captions | captions | `microservices/social/PRD.md:22` | Partial. |
| 18 | Sound/music attachment | core | core | lens/audio | no sound object | Gap. |
| 19 | Effects/lenses | effects | effects | core AR Lens | no lens/effect object | Gap. |
| 20 | Remix/stitch/duet | core | remix | remix | no remix object | Gap. |
| 21 | Hashtag discovery | core | core | topic | `microservices/social/PRD.md:50` | Present but tied to trending. |
| 22 | Search | core | core | core | `microservices/social/contracts/openapi/social.yaml:5` | Present. |
| 23 | Location discovery | limited | location tags | Snap Map | no location object | Gap. |
| 24 | Personalized recommendation | core | core | Spotlight/discover | `microservices/social/contracts/openapi/social.yaml:452-479` | Drift: prohibited as For-You-style product direction. |
| 25 | Chronological/following feed | following | following/feed | friends/stories | `microservices/social/contracts/openapi/social.yaml:452-479` | Present. |
| 26 | Safe non-algorithmic discovery | topic/tag/search | search/profile | stories/map/search | no explicit safe-discovery doctrine | Gap. |
| 27 | Content moderation | core | core | core | `microservices/social/PRD.md:54`; SLOs present | Present but must pivot to visual-first. |
| 28 | CSAM/media safety | required | required | required | `microservices/social/runbooks/csam-detect-and-ncmec-report.md` | Present as artifact. |
| 29 | Minor protection | required | required | required | `microservices/social/slos/minor-protection-engagement-correctness.openslo.yaml` | Present as artifact. |
| 30 | Creator monetization | external to target | product feature | product feature | `microservices/social/ARCHITECTURE.md:612-614` | Drift: forbidden anti-pattern in Oyatie social. |
| 31 | Sponsored post promotion | external to target | product feature | product feature | `microservices/social/PRD.md:70` | Drift: forbidden anti-pattern. |
| 32 | Federation | not core | not core | not core | `microservices/social/decisions/ADR-SOC-0004-federation-posture.md:35-91` | Likely stale. |
| 33 | OpenTofu per context | not counterpart feature | not counterpart feature | not counterpart feature | `microservices/social/IP-001-iac-bootstrap.md:20-44` | Gap: Terraform text and no context dirs. |
| 34 | OS support matrix | not counterpart feature | not counterpart feature | not counterpart feature | no `supported-oses.json` | Gap. |
| 35 | OCI Always Free profile | not counterpart feature | not counterpart feature | not counterpart feature | no `iac/oci-guest/always-free/` | Gap. |
| 36 | Tenant classes | not counterpart feature | not counterpart feature | not counterpart feature | no `demo_trial`, `tenant_class`, or `revenue_share` | Gap. |
| 37 | Rust backend implementation | internal | internal | internal | no `src/` | Gap if readiness claimed. |
| 38 | Test evidence | internal | internal | internal | PRD names tests, directory absent | Gap. |
| 39 | Benchmark evidence | internal | internal | internal | empty `benchmarks/` | Gap. |
| 40 | Mobile bundle explicit dependency graph | app-level | app-level | app-level | `microservices/social/manifest.json:375-383` | Gap: missing mail/community/cloud-iam detail. |

## 5. Family Summary

1. The union family is visual and short-video social.
2. TikTok contributes the short-video creation, ingest, clip lifecycle, creator workflow, and media-scale pressure.
3. Instagram contributes visual profile, grid, reels, stories, saves, comments, share-to-DM, and Explore-scale pressure.
4. Snapchat contributes camera-first, ephemeral story/snap lifecycle, AR/effects, map/location, and chat-adjacent visual sharing.
5. Oyatie current social contributes safety, moderation, compliance, dual-context thinking, post/follow/comment primitives, contracts, and SLO scaffolding.
6. Oyatie current social does not yet contribute the correct visual object grammar.
7. Oyatie current social does not yet contribute mobile-bundle backend handoffs with messenger, mail, and community.
8. Oyatie current social does not yet contribute deployment-context readiness.
9. Oyatie current social does not yet contribute tenant-class semantics.
10. Oyatie current social does not yet contribute OCI Always Free demo_trial infrastructure.
11. Oyatie current social does not yet contribute OS matrix evidence.
12. Oyatie current social does not yet contribute source/test implementation evidence.
13. The most reusable current assets are moderation, safety, compliance, and media transcode doctrine.
14. The least reusable current assets are old competitor framing, old text-feed benchmarking, ActivityPub/federation emphasis, and sponsored/creator monetization stubs.
15. The strongest current artifact conflict is `PRD.md:22` versus the 2026-05-21 social directive.
16. The second strongest conflict is `contracts/openapi/social.yaml:452-479` exposing algorithmic feed mode against the no For-You anti-pattern.
17. The third strongest conflict is `ARCHITECTURE.md:612-614` exposing creator monetization and branded content templates.
18. The fourth strongest conflict is `IP-001-iac-bootstrap.md:44` naming Terraform-managed Grafana RBAC.
19. The fifth strongest conflict is `manifest.json:318-322` retaining capability-profile semantics after Wave 15J retirement.
20. The parity family cannot be satisfied by adding media fields to the old text-post model.
21. The parity family needs a first-class media product model.
22. The parity family needs a first-class lifecycle model for clips and ephemeral visual posts.
23. The parity family needs a first-class mobile-bundle handoff model.
24. The parity family needs a safe discovery model that is not an algorithmic For-You clone.
25. The parity family needs a quality target that is uniform across tenant classes.

## 6. Headline Gap Analysis

1. P1 gap: social purpose is currently wrong.
2. Evidence: `microservices/social/PRD.md:22` says Twitter/X-class; directive says visual/short-video social at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_cell_standalone_network_merges_community_2026_05_21.md:109-134`.
3. P1 gap: counterpart matrix is currently wrong.
4. Evidence: `microservices/social/competitor-parity-matrix.md:28-40` lists broad text/professional/forum competitors; dispatch says TikTok/Instagram/Snapchat at chat line 17079.
5. P1 gap: current contract lacks clip/story/effect/remix/lens/location primitives.
6. Evidence: `microservices/social/contracts/openapi/social.yaml:95-138` and `microservices/social/contracts/proto/social.proto:142-148`.
7. P1 gap: current contract exposes algorithmic feed mode.
8. Evidence: `microservices/social/contracts/openapi/social.yaml:452-479` and `microservices/social/contracts/proto/social.proto:399`.
9. P1 gap: unified mobile app backend seams are incomplete.
10. Evidence: `microservices/social/manifest.json:375-383` and `microservices/social/contracts/asyncapi/social-events.yaml:7-13`.
11. P1 gap: OpenTofu context IaC is absent.
12. Evidence: `microservices/social/IP-001-iac-bootstrap.md:20-44` and missing canonical context directories.
13. P1 gap: OCI Always Free profile is absent.
14. Evidence: no `microservices/social/iac/oci-guest/always-free/`; canonical source `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3493-3790`.
15. P1 gap: source/tests are absent despite PRD and IP references.
16. Evidence: `microservices/social/PRD.md:321-339`; no `src/`; no `tests/`.
17. P2 gap: tenant classes are absent.
18. Evidence: no `demo_trial`, no `tenant_class`, no `revenue_share`; `microservices/social/capacity-model.md:183-195`.
19. P2 gap: stale tier semantics remain.
20. Evidence: `microservices/social/manifest.json:318-322`; `microservices/social/PRD.md:8`.
21. P2 gap: empty support directories are scaffold, not substance.
22. Evidence: empty `benchmarks/`, `faqs/`, `onboarding/`, `migration-playbooks/`, `reference-implementations/`, and `tutorials/`.
23. P2 gap: performance targets are not benchmarked against TikTok/Instagram/Snapchat.
24. Evidence: `microservices/social/competitor-parity-matrix.md:116-139`.
25. P3 gap: Rust-strict file scan passes only because implementation is absent.
26. Evidence: no forbidden source extension hits and no `src/`.
27. P3 gap: `reference-set` is a false-positive metal-like term, not a retired feature tier.
28. Evidence: `microservices/social/decisions/ADR-SOC-0003-content-moderation-classifier-bounds.md:178`.
29. Net analysis: this is a product-direction correction before an implementation hardening exercise.
30. Net analysis: current social cannot honestly claim TikTok/Instagram/Snapchat union coverage.
31. Net analysis: current social can retain moderation, compliance, and some media-transcode assets after correction.
32. Net analysis: current social must retire old competitor, algorithmic-feed, sponsored-promotion, and capability-profile assumptions.
33. Net analysis: the mobile-bundle requirement is now part of social's product boundary.
34. Net analysis: deployment readiness is blocked independently of product shape because context IaC is absent.
35. Net analysis: tenant-class readiness is blocked independently of product shape because no active tenant-class terms are expressed.

## 7. Additive Surface Required For Union Coverage

1. Additive surface: `VisualPost` contract.
2. Evidence basis: current `Post` schema is text-post shaped at `microservices/social/contracts/openapi/social.yaml:95-138`.
3. Additive surface: `ShortVideoClip` contract.
4. Evidence basis: current media upload exists but clip lifecycle is absent at `microservices/social/contracts/openapi/social.yaml:545-572`.
5. Additive surface: `StoryItem` contract.
6. Evidence basis: Instagram and Snapchat parity requires story lifecycle; current contracts have no story path.
7. Additive surface: `ClipUploadSession` for chunked/resumable ingest.
8. Evidence basis: TikTok chunked upload constraints demonstrate media-scale expectation; source `https://developers.tiktok.com/doc/content-posting-api-media-transfer-guide`, lines 274-280.
9. Additive surface: `MediaProcessingStatus` with variants, safety scan, transcode, thumbnail, and playback-readiness states.
10. Evidence basis: ADR-SOC-0006 has transcode/storage but OpenAPI lacks full lifecycle; evidence `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:68-110`.
11. Additive surface: `RemixPolicy`.
12. Evidence basis: TikTok/Snapchat-style remix exists in counterpart family; current schema has no remix object.
13. Additive surface: `EffectReference` or `LensReference`.
14. Evidence basis: Snapchat AR Lens scale is public evidence; source SEC release lines 65-68.
15. Additive surface: `SoundReference`.
16. Evidence basis: TikTok/Instagram short-video creation expects audio attachment; current schema has no sound object.
17. Additive surface: `AudienceScope`.
18. Evidence basis: current docs use Personal/Professional and generic visibility, but mobile-bundle social needs audience controls without retired tier language.
19. Additive surface: `SafeDiscoveryQuery`.
20. Evidence basis: no algorithmic For-You-feed directive at memory lines 136-147.
21. Additive surface: `TagSearchResult`.
22. Evidence basis: hashtags currently exist but trending is overemphasized at `microservices/social/PRD.md:50`.
23. Additive surface: `LocationContext` only if Snap Map / Instagram location parity is accepted.
24. Evidence basis: Snapchat Snap Map scale source SEC release line 69.
25. Additive surface: `MobileBundleShareToMessenger`.
26. Evidence basis: current messenger deep-link FR is present at `microservices/social/PRD.md:57` but gRPC handoff is absent.
27. Additive surface: `MobileBundleMailActionCard`.
28. Evidence basis: AsyncAPI mail action-card event is present at `microservices/social/contracts/asyncapi/social-events.yaml:317`.
29. Additive surface: `MobileBundleCommunityAttach`.
30. Evidence basis: PRD claims community cross-links at `microservices/social/PRD.md:34`, but manifest dependencies omit community at `microservices/social/manifest.json:375-383`.
31. Additive surface: `CloudIamSessionContext`.
32. Evidence basis: mobile-bundle directive requires shared cloud-iam session at memory lines 165-191.
33. Additive surface: `UnifiedPushEvent`.
34. Evidence basis: social notification BC exists but cross-bundle dedupe/presentation is absent; current references at `microservices/social/IP-010-notifications-bc.md:20-23`.
35. Additive surface: `TenantClassOverlay`.
36. Evidence basis: no `tenant_class` terms exist under service path.
37. Additive surface: `DeploymentContextOverlay`.
38. Evidence basis: six context directories are absent and canonical requirement is ADR-0328 lines 1732-1994.
39. Additive surface: `OciAlwaysFreeBudgetProfile`.
40. Evidence basis: canonical OCI Always Free profile source is ADR-0328 lines 3493-3790.
41. Additive surface: `SupportedOsMatrix`.
42. Evidence basis: `supported-oses.json` is absent and OS matrix is required.
43. Additive surface: `VisualSafetyCase`.
44. Evidence basis: current moderation artifacts exist but are text-feed oriented in product framing.
45. Additive surface: `VisualIncidentRunbook` set for clip takedown, story expiry corruption, media-transcode degradation, and remix abuse.
46. Evidence basis: current runbooks cover abuse/trending/follow/cache but not visual-first lifecycle failures.
47. Additive surface: `VisualBenchmarkPlan`.
48. Evidence basis: benchmark directory is empty and competitor matrix benchmarks are old.
49. Additive surface: `NoSponsoredPromotionInvariant`.
50. Evidence basis: current architecture includes monetization/branded templates at `microservices/social/ARCHITECTURE.md:612-614`.
51. Additive surface: `NoForYouInvariant`.
52. Evidence basis: current feed algorithm ADR conflicts with memory lines 136-147.
53. Additive surface: `NoLinkedInStyleFeedInvariant`.
54. Evidence basis: current README lists LinkedIn as precedent at `microservices/social/README.md:18`.
55. Additive surface: `NoTextBroadcastPrimaryInvariant`.
56. Evidence basis: current PRD says Twitter/X-class at `microservices/social/PRD.md:22`.

## 8. Readiness Classification

1. TikTok parity readiness: red.
2. Reason: clip object, remix/sound/effects, upload session, and safe short-video discovery are absent.
3. Instagram parity readiness: red.
4. Reason: visual profile grid, stories, carousel semantics, creator insight, share-to-DM handoff, and visual discovery are absent or partial.
5. Snapchat parity readiness: red.
6. Reason: ephemeral lifecycle, camera/effect/lens object, map/location, Spotlight-style video, and chat-adjacent visual sharing are absent or partial.
7. Mobile-bundle readiness: red.
8. Reason: messenger is partial, mail is event-only, community is absent from manifest, cloud-iam session is absent, and unified push is absent.
9. Infrastructure readiness: red.
10. Reason: six context OpenTofu modules are absent.
11. Tenant-class readiness: red.
12. Reason: `demo_trial`, `paid`, and `revenue_share` are absent as tenant classes.
13. Tenant-class adoption readiness: yellow.
14. Reason: exact retired metal-label references are absent, but stale tier semantics remain.
15. Rust-policy readiness: yellow.
16. Reason: no forbidden source files are present, but no Rust implementation is present.
17. Evidence-readiness: red.
18. Reason: source/test/benchmark directories are absent or empty while docs name expected evidence.
19. Overall parity status: red.
20. Required next gate: product realignment before implementation.

## 9. Parity Remediation Sequence

1. Step 1: rewrite the social product statement before editing implementation plans.
2. Evidence: current product statement is Twitter/X-class at `microservices/social/PRD.md:22`.
3. Success condition: product statement names visual posts, short video, mobile-bundle handoffs, and TikTok/Instagram/Snapchat union coverage.
4. Step 2: retire old competitor framing from the active matrix.
5. Evidence: old matrix includes X, Bluesky, Mastodon, Threads, LinkedIn, Reddit, Tumblr, and Hive Social at `microservices/social/competitor-parity-matrix.md:28-40`.
6. Success condition: old competitors appear only in history or explicit non-goal notes.
7. Step 3: replace text-post contract center with visual and clip contracts.
8. Evidence: current OpenAPI `Post` schema is post/repost/quote/comment shaped at `microservices/social/contracts/openapi/social.yaml:95-138`.
9. Success condition: OpenAPI and proto both expose visual post, clip, story item, media processing status, and mobile share handoff objects.
10. Step 4: delete or quarantine algorithmic For-You-style feed semantics.
11. Evidence: current OpenAPI feed mode includes algorithmic at `microservices/social/contracts/openapi/social.yaml:452-479`.
12. Success condition: discovery contract is tag/search/relationship-guided and has no engagement-optimized For-You product invariant.
13. Step 5: remove sponsored and follower-monetization surfaces from the social product path.
14. Evidence: PRD names ads-substrate-stub at `microservices/social/PRD.md:70`; architecture names creator/branded templates at `microservices/social/ARCHITECTURE.md:612-614`.
15. Success condition: social supports safety and analytics without sponsored-post promotion or follower-monetization-via-followers.
16. Step 6: define messenger handoff as a gRPC/event contract.
17. Evidence: messenger deep-link intent exists at `microservices/social/PRD.md:57` but manifest only lists messenger as a generic dependency at `microservices/social/manifest.json:375-383`.
18. Success condition: share-to-DM has request, response, failure, auth, push, and audit behavior.
19. Step 7: define mail handoff as action-card and digest contract.
20. Evidence: mail action-card event appears at `microservices/social/contracts/asyncapi/social-events.yaml:317`.
21. Success condition: mail receives bounded action-card payloads without social importing mail internals.
22. Step 8: define community handoff as discussion-attach contract.
23. Evidence: PRD claims community cross-link at `microservices/social/PRD.md:34`, but manifest omits community at `microservices/social/manifest.json:375-383`.
24. Success condition: community can attach social visual posts without collapsing backend ownership.
25. Step 9: define shared cloud-iam session contract.
26. Evidence: current service links list identity-like dependencies but not the unified mobile-bundle session contract at `microservices/social/ARCHITECTURE.md:1121-1128`.
27. Success condition: social validates session claims from cloud-iam/identity and does not mint a separate app session.
28. Step 10: define unified push event shape.
29. Evidence: notification BC exists at `microservices/social/IP-010-notifications-bc.md:20-23`, but cross-bundle push dedupe is absent.
30. Success condition: push payloads include owner, dedupe key, priority, display policy, and mobile-bundle destination.
31. Step 11: add tenant-class overlay.
32. Evidence: `demo_trial`, `tenant_class`, and `revenue_share` are absent under the service path.
33. Success condition: contracts and capacity models express `demo_trial`, `paid`, and `revenue_share` without lowering feature quality.
34. Step 12: add OCI Always Free profile.
35. Evidence: no `microservices/social/iac/oci-guest/always-free/` directory exists.
36. Success condition: demo_trial infrastructure maps to OCI Always Free caps and outputs, not to an old feature tier.
37. Step 13: add six deployment-context OpenTofu modules.
38. Evidence: current `iac/` has Helm/Kustomize/YAML, while canonical context directories are absent.
39. Success condition: each context has OpenTofu files or a justified N/A field.
40. Step 14: add OS support matrix.
41. Evidence: no `microservices/social/supported-oses.json` exists.
42. Success condition: supported OS and architecture claims match canonical policy.
43. Step 15: create benchmark harness only after contract correction.
44. Evidence: `microservices/social/benchmarks/` is empty and current competitor numbers are stale.
45. Success condition: harness measures visual feed, profile grid, upload session, transcode readiness, story expiry, push, and mobile-bundle handoffs.
46. Step 16: create source and test evidence only after product correction is locked.
47. Evidence: no `microservices/social/src/` and no `microservices/social/tests/` exist, while PRD names expected tests at `microservices/social/PRD.md:321-339`.
48. Success condition: tests prove current product invariants instead of the retired text-feed model.
49. Step 17: preserve safety assets during realignment.
50. Evidence: current runbooks and SLOs include moderation, CSAM, minor protection, and policy correctness.
51. Success condition: safety artifacts are rewritten for visual clips, stories, remixes, and camera/effect surfaces.
52. Step 18: preserve useful media-transcode assets during realignment.
53. Evidence: ADR-SOC-0006 already covers image and short-video transcode at `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:32-35`.
54. Success condition: media transcode becomes a lifecycle contract with first-playable and full-variant readiness states.
55. Step 19: block parity claims until all red readiness rows move to green evidence.
56. Evidence: readiness classification in this report lists TikTok, Instagram, Snapchat, mobile-bundle, infra, tenant-class, and evidence readiness as red.
57. Success condition: every claim in future sales, architecture, and implementation plans cites executable or canonical evidence.
58. Step 20: keep the fourth retired-tier delta deliverable absent.
59. Evidence: this audit only authors the three requested reports.
60. Success condition: no new feature-tier matrix appears in social deliverables.
