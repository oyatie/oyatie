# Social Performance Benchmark Numbers - 2026-05-20

Target microservice: `microservices/social/`.
Benchmark family: visual and short-video social.
Counterparts: TikTok / Instagram / Snapchat.
Target model: one industry-leader target set with deployment-context overlays and tenant-class overlays.
Tenant classes used in overlays: `demo_trial`, `paid`, `revenue_share`.
Deployment contexts used in overlays: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.
Retired deliverable note: no fourth retired-tier delta document is authored for this audit.
Methodology disclosure: public counterpart latency and internal SLO numbers are incomplete, so this document separates sourced numbers, estimates from public engineering scale signals, current Oyatie artifact targets, and proposed Oyatie targets.

## Citation Anchor Block

1. Canonical Oyatie benchmark target shape must use the six deployment contexts; source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1994`.
2. Canonical OCI Always Free profile includes 4 Ampere A1 OCPU, 24 GB memory, 200 GB block storage, and 10 Mbps load balancer constraints; source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514-3571`.
3. Current Oyatie social PRD has old feed/post/media targets; source: `microservices/social/PRD.md:76-90`.
4. TikTok media transfer guide publishes upload chunk and ingress constraints; source: `https://developers.tiktok.com/doc/content-posting-api-media-transfer-guide`, lines 274-280 and 500-504.
5. Snap Q1 2026 SEC release publishes MAU, DAU, Spotlight, AR Lens, Lens submissions, Snap Map, revenue, and ARPU numbers; source: `https://www.sec.gov/Archives/edgar/data/1564408/000156440826000024/snap-20260506xexx991pressr.htm`, lines 57-79 and 104-112.

## 1. Methodology

1. Benchmark dimension: read latency for home/following visual feed.
2. Benchmark dimension: safe discovery/search latency.
3. Benchmark dimension: profile/grid render latency.
4. Benchmark dimension: short-video metadata render latency.
5. Benchmark dimension: media upload initialization latency.
6. Benchmark dimension: media upload sustained ingest throughput.
7. Benchmark dimension: media transcode queue latency.
8. Benchmark dimension: playback manifest readiness.
9. Benchmark dimension: push notification fanout latency.
10. Benchmark dimension: comment/reaction write latency.
11. Benchmark dimension: share-to-messenger handoff latency.
12. Benchmark dimension: mail action-card handoff latency.
13. Benchmark dimension: community attach handoff latency.
14. Benchmark dimension: moderation pre-publish safety decision latency.
15. Benchmark dimension: post-publication abuse-report decision latency.
16. Benchmark dimension: mobile-session validation overhead.
17. Benchmark dimension: per-tenant usage cap enforcement overhead.
18. Benchmark dimension: deployment-context infrastructure ceiling.
19. Benchmark dimension: tenant-class economics ceiling.
20. Benchmark dimension: OS and architecture coverage.
21. Test workload: image post creation with one 4 MB image.
22. Test workload: carousel visual post with 10 images.
23. Test workload: short-video upload with 50 MB clip.
24. Test workload: short-video upload with 200 MB clip, matching current Oyatie OpenAPI max evidence at `microservices/social/contracts/openapi/social.yaml:545-572`.
25. Test workload: story creation and expiry event.
26. Test workload: safe tag search over visual corpus.
27. Test workload: share visual post to messenger.
28. Test workload: send social digest/action card to mail.
29. Test workload: attach visual post to community discussion.
30. Test workload: moderation classifier pre-publish check.
31. Test workload: minor-protection gate for public visual post.
32. Test workload: push notification fanout to mobile app bundle.
33. OS disclosure: target matrix must cover canonical supported OSes, but social currently lacks `supported-oses.json`.
34. Architecture disclosure: target matrix must cover CPU architectures named by canonical OS policy, but current social artifacts do not express that matrix.
35. Deployment context disclosure: all six contexts are evaluated as overlays, but current social lacks the required context directories.
36. Tenant-class disclosure: this benchmark uses `demo_trial`, `paid`, and `revenue_share` overlays, but current social artifacts do not express those tenant classes.
37. Source disclosure: TikTok and Instagram do not publish all latency targets used internally.
38. Source disclosure: Snap publishes user, engagement, and business scale numbers, not all low-level latency targets.
39. Estimate disclosure: where counterpart latency is not public, Oyatie target selection uses current PRD targets, media workload expectations, and industry-leader user-scale signals.
40. Non-goal disclosure: this report does not set algorithmic For-You-feed targets because that product behavior is forbidden by current directive.
41. Non-goal disclosure: this report does not set sponsored-post-promotion targets because that product behavior is forbidden by current directive.
42. Non-goal disclosure: this report does not use old capability-tier headings or rows.
43. Validation disclosure: current social has no source implementation, no tests directory, and no benchmark directory.
44. Validation disclosure: numbers here are target-setting and gap analysis, not measured Oyatie runtime evidence.
45. Stop condition for this document: enough benchmark specificity to guide the corrective PRD/contract/performance plan without inventing measured implementation results.

## 2. Counterpart Numbers

### 2.1 TikTok Numbers

1. TikTok user scale number: more than 1 billion monthly active users was publicly announced in 2021; source: TikTok newsroom search result and `https://newsroom.tiktok.com/tiktok-rolls-out-new-solutions-and-showcases-the-power-of-joy-at-tiktok-the-stage-my?lang=en-MY`.
2. TikTok media transfer number: video uploads less than 5 MB must upload as one whole request; source: media transfer guide lines 274-280.
3. TikTok media transfer number: chunked uploads use chunks at least 5 MB; source: media transfer guide lines 274-280.
4. TikTok media transfer number: chunked uploads use chunks no greater than 64 MB except final chunks; source: media transfer guide lines 274-280.
5. TikTok media transfer number: final trailing chunk may be up to 128 MB; source: media transfer guide lines 274-280.
6. TikTok media transfer number: maximum chunk count is 1000; source: media transfer guide lines 274-280.
7. TikTok media transfer number: file chunks upload sequentially; source: media transfer guide lines 274-280.
8. TikTok media transfer number: a 50,000,123 byte example uploads in five chunks; source: media transfer guide lines 291-398.
9. TikTok media transfer number: whole upload example uses 4,194,304 bytes and returns success in one request; source: media transfer guide lines 399-445.
10. TikTok media transfer number: chunked upload returns 206 for partial content and 201 when complete; source: media transfer guide lines 380-389 and 446-466.
11. TikTok media transfer number: pull-from-URL server ingress can reach 100 Mbps; source: media transfer guide lines 500-504.
12. TikTok feed number: For You is the first feed opened in the app; source: `https://support.tiktok.com/en/getting-started/for-you?lang=en`, lines 182-185.
13. TikTok recommendation number: displayed reasons include liked/commented/shared/watched similar posts, country popularity, recency, longer-video preference, and followed creator; source: same TikTok support page lines 187-199.
14. TikTok control number: users can refresh feed, filter keywords, and manage topics; source: same TikTok support page lines 202-208.
15. Benchmark implication: Oyatie must match short-video ingest reliability and responsiveness while explicitly avoiding For-You-style engagement optimization.

### 2.2 Instagram Numbers

1. Instagram Explore scale number: hundreds of millions of people visit Explore every day; source: `https://engineering.fb.com/2023/08/09/ml-applications/scaling-instagram-explore-recommendations-system/`, lines 41-45.
2. Instagram candidate scale number: Explore recommends in real time out of billions of available options; source: same Meta Engineering article lines 44-45.
3. Instagram ranking pipeline number: Explore uses four stages: retrieval, first-stage ranking, second-stage ranking, and final reranking; source: same article lines 47-51.
4. Instagram retrieval funnel number: large-scale recommenders start from thousands of candidates and narrow to hundreds; source: same article lines 57-61.
5. Instagram retrieval source number: each source selects hundreds of relevant items from a media pool of billions; source: same article lines 59-61.
6. Instagram retrieval type number: sources can be heuristic, ML, real-time, or pre-generated; source: same article lines 61-67.
7. Instagram Reels duration number: public help states Reels can be recorded and edited up to 20 minutes; source: `https://www.facebook.com/help/instagram/225190788256708`, search result snippet.
8. Instagram Reels quality number: public help states Reels should use at least 30 FPS; source: `https://www.facebook.com/help/1038071743007909`, search result snippet.
9. Instagram Reels quality number: public help states Reels should use minimum 720 px resolution; source: same help result snippet.
10. Instagram Story retention number: public help states story photos/videos disappear after 24 hours unless highlighted or saved; source: `https://www.facebook.com/help/1729008150678239/`, search result snippet.
11. Instagram media-family estimate: visual profile grid should render enough media metadata for first viewport under 200 ms p95 on warm cache; source type: engineering target inferred from current Oyatie PRD feed p95 200 ms at `microservices/social/PRD.md:76-83`.
12. Instagram media-family estimate: Explore/tag/search request should cap candidate retrieval to bounded hundreds before final render; source type: estimated from Meta Engineering lines 57-61.
13. Instagram media-family estimate: short-video readiness should expose manifest/thumbnail state separately from final full transcode; source type: estimated from visual app product expectations and current Oyatie ADR-SOC-0006 media pipeline at `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:68-110`.
14. Benchmark implication: Oyatie must build a visual media graph and safe discovery funnel rather than an old text timeline benchmark.

### 2.3 Snapchat Numbers

1. Snapchat user scale number: 956 million global MAU in Q1 2026; source: Snap SEC release lines 61-69.
2. Snapchat user scale number: 483 million global DAU in Q1 2026; source: Snap SEC release lines 61-69.
3. Snapchat growth number: MAU increased 43 million, or 5% year-over-year; source: Snap SEC release lines 61-69.
4. Snapchat growth number: DAU grew 5% year-over-year; source: Snap SEC release lines 61-69.
5. Snapchat Spotlight number: Spotlight posters grew nearly 74% year-over-year in the US; source: Snap SEC release lines 61-63.
6. Snapchat Spotlight number: Spotlight posters grew over 61% globally; source: Snap SEC release lines 61-63.
7. Snapchat real-time group chat number: March Madness Topic Chat had more than 90,000 chats; source: Snap SEC release lines 64-65.
8. Snapchat real-time concurrency number: March Madness Topic Chat had over 40,000 simultaneous active people at peak; source: Snap SEC release lines 64-65.
9. Snapchat AR number: Snapchat camera used AR Lenses more than 9 billion times per day on average in Q1; source: Snap SEC release lines 65-67.
10. Snapchat AR engagement number: 75% of Snapchatters engaged with AR every day on average; source: Snap SEC release lines 65-67.
11. Snapchat creator ecosystem number: more than 400,000 Lenses submitted in Q1; source: Snap SEC release lines 65-68.
12. Snapchat creator ecosystem growth number: Lens submissions increased more than 150% year-over-year; source: Snap SEC release lines 65-68.
13. Snapchat map number: Snap Map reached over 450 million global MAU in Q1; source: Snap SEC release line 69.
14. Snapchat revenue number: Q1 2026 revenue was $1.529 billion, up 12% year-over-year; source: Snap SEC release lines 15-24 and 35-47.
15. Snapchat retention number: most Stories delete after 24 hours, Shared Stories hold up to 1,000 Snaps, and one-to-one Snaps delete after viewed by recipients; source: Snapchat Support lines 21-67.
16. Benchmark implication: Oyatie must budget for ephemeral lifecycle, visual camera/effect throughput, real-time chat-adjacent handoffs, and push surface coordination.

## 3. Oyatie Target Numbers

### 3.1 Single Industry-Leader Target Set

1. Target: visual home feed warm-cache p50 <= 50 ms.
2. Target: visual home feed warm-cache p95 <= 180 ms.
3. Target: visual home feed warm-cache p99 <= 350 ms.
4. Basis: improves current PRD feed target p95 200 ms and p99 400 ms; evidence `microservices/social/PRD.md:76-83`.
5. Target: visual home feed cold-cache p95 <= 450 ms.
6. Target: profile/grid render warm-cache p50 <= 45 ms.
7. Target: profile/grid render warm-cache p95 <= 160 ms.
8. Target: profile/grid render warm-cache p99 <= 325 ms.
9. Target: short-video metadata render p50 <= 60 ms.
10. Target: short-video metadata render p95 <= 200 ms.
11. Target: short-video metadata render p99 <= 400 ms.
12. Target: tag/search discovery p50 <= 80 ms.
13. Target: tag/search discovery p95 <= 250 ms.
14. Target: tag/search discovery p99 <= 500 ms.
15. Target: location discovery p95 <= 300 ms when location feature is enabled.
16. Target: story tray render p50 <= 50 ms.
17. Target: story tray render p95 <= 180 ms.
18. Target: story tray render p99 <= 350 ms.
19. Target: image post create p50 <= 35 ms excluding binary upload.
20. Target: image post create p95 <= 110 ms excluding binary upload.
21. Target: image post create p99 <= 250 ms excluding binary upload.
22. Target: short-video publish metadata commit p50 <= 45 ms excluding binary upload and transcode.
23. Target: short-video publish metadata commit p95 <= 130 ms excluding binary upload and transcode.
24. Target: short-video publish metadata commit p99 <= 300 ms excluding binary upload and transcode.
25. Target: media upload session init p50 <= 40 ms.
26. Target: media upload session init p95 <= 125 ms.
27. Target: media upload session init p99 <= 275 ms.
28. Target: sustained media ingest per upload session >= 100 Mbps where deployment context provides edge capacity.
29. Basis: TikTok pull-from-URL ingress can reach 100 Mbps; source: TikTok media transfer guide lines 500-504.
30. Target: 50 MB clip upload control-plane overhead <= 250 ms p95 excluding client network transfer.
31. Target: 200 MB clip upload control-plane overhead <= 400 ms p95 excluding client network transfer.
32. Target: 50 MB short-video transcode first-playable manifest p50 <= 8 seconds.
33. Target: 50 MB short-video transcode first-playable manifest p95 <= 25 seconds.
34. Target: 200 MB short-video transcode first-playable manifest p50 <= 20 seconds.
35. Target: 200 MB short-video transcode first-playable manifest p95 <= 75 seconds.
36. Basis: improves current PRD video transcode p95 90 seconds; evidence `microservices/social/PRD.md:76-90`.
37. Target: image derivative generation p50 <= 450 ms.
38. Target: image derivative generation p95 <= 1.5 seconds.
39. Basis: improves current PRD image transcode p95 2 seconds; evidence `microservices/social/PRD.md:76-90`.
40. Target: reaction write p50 <= 20 ms.
41. Target: reaction write p95 <= 75 ms.
42. Target: reaction write p99 <= 180 ms.
43. Target: comment write p50 <= 35 ms.
44. Target: comment write p95 <= 110 ms.
45. Target: comment write p99 <= 250 ms.
46. Target: share-to-messenger handoff p50 <= 50 ms.
47. Target: share-to-messenger handoff p95 <= 175 ms.
48. Target: mail action-card handoff p50 <= 80 ms.
49. Target: mail action-card handoff p95 <= 250 ms.
50. Target: community attach handoff p50 <= 80 ms.
51. Target: community attach handoff p95 <= 250 ms.
52. Target: unified push fanout p50 <= 250 ms.
53. Target: unified push fanout p95 <= 1.5 seconds.
54. Target: unified push fanout p99 <= 4 seconds.
55. Basis: current PRD notification fanout p95 2 seconds and p99 5 seconds; evidence `microservices/social/PRD.md:76-90`.
56. Target: moderation pre-publish synchronous decision p50 <= 80 ms.
57. Target: moderation pre-publish synchronous decision p95 <= 300 ms.
58. Target: moderation pre-publish synchronous decision p99 <= 750 ms.
59. Target: asynchronous human/moderation escalation SLA <= 15 minutes for severe abuse queue.
60. Target: story expiry event accuracy >= 99.99% within 60 seconds of configured expiry.
61. Target: story expiry backlog recovery <= 10 minutes after worker restart.
62. Target: feed/discovery cache rebuild for one tenant <= 30 minutes for 10 million visual objects.
63. Target: public media signed URL generation p95 <= 75 ms.
64. Target: Cedar authorization overhead p95 <= 10 ms per social request.
65. Target: cloud-iam session validation overhead p95 <= 15 ms per request.
66. Target: per-tenant usage cap check overhead p95 <= 5 ms.
67. Target: measured availability for visual read APIs >= 99.95% monthly for paid and revenue_share production tenants.
68. Target: measured availability for demo_trial follows best-effort SLO with hard usage caps.
69. Target: error budget burn alerts fire within 5 minutes.
70. Target: media malware quarantine event delivery p95 <= 2 seconds after scan verdict.

### 3.2 Deployment-Context Overlay

1. `oyatie-public-cloud`: elastic target; all canonical targets apply.
2. `oyatie-public-cloud`: media ingest target is >= 100 Mbps per upload session where regional edge policy permits.
3. `oyatie-public-cloud`: visual feed p95 target remains <= 180 ms warm-cache.
4. `oyatie-public-cloud`: push fanout p95 target remains <= 1.5 seconds.
5. `guest-on-aws`: target same API latencies if the customer-paid substrate supplies equivalent edge, storage, queue, and compute capacity.
6. `guest-on-aws`: deployment-specific storage and edge bottlenecks must be exposed as capacity variables, not feature downgrades.
7. `guest-on-oci`: target same API latencies for paid/revenue_share substrate outside Always Free resource caps.
8. `guest-on-oci`: OCI Always Free profile is separately capped by 4 OCPU, 24 GB memory, 200 GB block storage, 10 Mbps load balancer, and 10 TB monthly egress doctrine.
9. `guest-on-oci`: demo_trial under OCI Always Free caps sustained media ingest below the 100 Mbps target when fronted by the Always Free load-balancer ceiling.
10. `guest-on-oci`: demo_trial must cap concurrent uploads, concurrent feed sessions, media retention, and transcode queue depth.
11. `on-prem`: target API latencies apply only when facility compute, storage, edge cache, and network are sized to target.
12. `on-prem`: facility-specific constraints must surface as capacity warnings and tenant onboarding checks.
13. `colo`: target API latencies apply when cross-connect and storage are sized to target.
14. `colo`: regional media egress bottlenecks must become deployment context variables.
15. `oyatie-as-cloud-provider`: target matches public cloud for regions where Oyatie owns enough substrate.
16. `oyatie-as-cloud-provider`: target must include internal capacity reservation for revenue_share tenants if at-cost substrate is promised.
17. Overlay invariant: no deployment context changes the product-quality target.
18. Overlay invariant: constrained contexts change quotas, concurrency, and scale ceilings, not the feature quality bar.
19. Overlay invariant: every context requires OpenTofu-owned modules before readiness claim.
20. Overlay gap: current social has no context modules, so all overlay targets are plan targets, not measured evidence.

### 3.3 Tenant-Class Overlay

1. `demo_trial`: free tenant class.
2. `demo_trial`: uses OCI Always Free profile where possible.
3. `demo_trial`: hard cap concurrent active users per tenant to 100 by default until capacity model proves higher.
4. `demo_trial`: hard cap sustained media ingest to context ceiling, with 10 Mbps load-balancer ceiling in OCI Always Free profile.
5. `demo_trial`: hard cap stored media to a bounded quota that fits the 200 GB block and 10 GB object/archive free resource doctrine when colocated with other trial services.
6. `demo_trial`: transcode queue depth cap defaults to 2 concurrent clips per tenant.
7. `demo_trial`: short-video retention cap defaults to 7 days unless explicitly paid or converted.
8. `demo_trial`: best-effort SLO, but same correctness and safety quality.
9. `demo_trial`: no compliance packs, no BYOK, no private edge peering.
10. `paid`: per-seat plus usage billing.
11. `paid`: all industry-leader API latency targets apply when paid substrate is provisioned.
12. `paid`: media ingest can scale beyond 100 Mbps per upload session only with explicit edge/storage sizing.
13. `paid`: compliance packs allowed.
14. `paid`: BYOK allowed.
15. `paid`: contractual SLO applies.
16. `revenue_share`: Oyatie takes gross-revenue share.
17. `revenue_share`: substrate runs at-cost or zero-margin by doctrine.
18. `revenue_share`: same quality target as paid.
19. `revenue_share`: capacity scales with expected gross merchandise, creator, or B2C operator volume and must include a cost-recovery guardrail.
20. `revenue_share`: compliance and BYOK availability follow contract, not a quality stratum.
21. Tenant-class invariant: no class receives a lower-quality feature implementation.
22. Tenant-class invariant: caps are quota and economics controls, not capability tiers.
23. Tenant-class gap: current social artifacts do not express `demo_trial`, `paid`, or `revenue_share`.
24. Tenant-class gap evidence: no `tenant_class` string under service path; `microservices/social/capacity-model.md:183-195` uses older tenant-limit labels.
25. Tenant-class required action: performance targets must become measurable overlays after tenant-class contract adoption.

### 3.4 Current Oyatie Artifact Numbers To Retire Or Rebase

1. Current PRD feed render p50 60 ms should be rebased to visual-feed p50 50 ms warm-cache; evidence `microservices/social/PRD.md:76-83`.
2. Current PRD feed render p95 200 ms should be rebased to visual-feed p95 180 ms warm-cache; evidence `microservices/social/PRD.md:76-83`.
3. Current PRD feed render p99 400 ms should be rebased to visual-feed p99 350 ms warm-cache; evidence `microservices/social/PRD.md:76-83`.
4. Current PRD post-create p50 30 ms remains plausible for metadata-only image post creation; evidence `microservices/social/PRD.md:76-83`.
5. Current PRD post-create p95 100 ms remains plausible for metadata-only image post creation; evidence `microservices/social/PRD.md:76-83`.
6. Current PRD post-create p99 250 ms remains plausible for metadata-only image post creation; evidence `microservices/social/PRD.md:76-83`.
7. Current PRD image transcode p95 2 seconds should improve to 1.5 seconds for first derivative readiness; evidence `microservices/social/PRD.md:76-90`.
8. Current PRD video transcode p95 90 seconds should split into first-playable manifest and full variant readiness; evidence `microservices/social/PRD.md:76-90`.
9. Current PRD 500k active users per cell is not enough for counterpart parity claim without per-context sizing; evidence `microservices/social/PRD.md:297-307`.
10. Current PRD 5M active users in large public pack is not enough for TikTok/Instagram/Snapchat parity but may be a first production cell scale target; evidence `microservices/social/PRD.md:297-307`.
11. Current PRD 1k post writes/sec per cell should become visual object writes/sec and media upload session starts/sec; evidence `microservices/social/PRD.md:297-307`.
12. Current PRD 25k posts/sec large public pack should not be claimed until source, tests, and benchmark harness exist; evidence `microservices/social/PRD.md:297-307`.
13. Current PRD 100k media uploads/day per cell should be split by image, carousel, story, and short-video workload; evidence `microservices/social/PRD.md:297-307`.
14. Current PRD 5M media uploads/day large public pack should include transcode, storage, moderation, and push budgets; evidence `microservices/social/PRD.md:297-307`.
15. Current competitor matrix X/Bluesky/Mastodon/Threads numbers should be retired from social benchmark baseline; evidence `microservices/social/competitor-parity-matrix.md:116-127`.

## 4. Comparison Narrative

1. Headline: monthly active user scale.
2. Counterpart reference: TikTok exceeds 1B MAU; Snap reported 956M MAU in Q1 2026.
3. Oyatie target posture: catch-up; current PRD large public pack target of 5M active users is not platform-scale parity.
4. Evidence: `microservices/social/PRD.md:297-307`; Snap SEC release lines 61-69.
5. Headline: daily active user scale.
6. Counterpart reference: Snap reported 483M DAU in Q1 2026.
7. Oyatie target posture: catch-up; no social artifact expresses DAU target at hundreds-of-millions scale.
8. Headline: media ingest control plane.
9. Counterpart reference: TikTok documents 5 MB to 64 MB chunks, final chunk up to 128 MB, and 1000 chunk max.
10. Oyatie target posture: catch-up; current OpenAPI has a 200 MB media cap but no chunked/resumable upload contract.
11. Evidence: `microservices/social/contracts/openapi/social.yaml:545-572`.
12. Headline: pull-from-URL ingest.
13. Counterpart reference: TikTok states pull-from-URL server ingress can reach 100 Mbps.
14. Oyatie target posture: parity target where deployment context provides edge capacity; constrained for OCI Always Free profile.
15. Headline: visual discovery scale.
16. Counterpart reference: Instagram Explore serves hundreds of millions daily from billions of options through staged retrieval and ranking.
17. Oyatie target posture: catch-up in scale, but target semantics differ because For-You-style engagement optimization is forbidden.
18. Evidence: Meta Engineering lines 41-61 and mobile-bundle directive lines 136-147.
19. Headline: story/ephemeral lifecycle.
20. Counterpart reference: Snapchat Stories usually expire after 24 hours and Shared Stories hold up to 1,000 Snaps.
21. Oyatie target posture: catch-up; no story lifecycle exists in current OpenAPI/proto.
22. Headline: AR/effect ecosystem.
23. Counterpart reference: Snap reports more than 9B AR Lens uses per day and more than 400k Lens submissions in Q1 2026.
24. Oyatie target posture: catch-up or explicit scope deferral; no lens/effect object exists.
25. Headline: visual profile and grid.
26. Counterpart reference: Instagram visual profile expectations require grid/media-first render.
27. Oyatie target posture: catch-up; current profile contract is not grid-first.
28. Headline: short-video readiness.
29. Counterpart reference: TikTok and Instagram both anchor short-video workflows; Snap Spotlight anchors public video Snaps.
30. Oyatie target posture: catch-up; ADR-SOC-0006 gives media pipeline basics but no clip lifecycle.
31. Evidence: `microservices/social/decisions/ADR-SOC-0006-media-transcode-and-storage.md:32-35`.
32. Headline: push fanout.
33. Counterpart reference: all three mobile products depend on push and notification responsiveness, but public exact latency targets are not disclosed.
34. Oyatie target posture: parity target; current PRD notification fanout p95 2 seconds should be tightened to p95 1.5 seconds for unified mobile bundle.
35. Evidence: `microservices/social/PRD.md:76-90`.
36. Headline: mobile-bundle handoff.
37. Counterpart reference: the Oyatie directive, not external counterpart docs, requires one binary per platform with distinct messenger/mail/social/community backends.
38. Oyatie target posture: catch-up; current artifacts show partial messenger/mail events and no community/cloud-iam/unified-push contract.
39. Evidence: `microservices/social/contracts/asyncapi/social-events.yaml:7-13` and `microservices/social/manifest.json:375-383`.
40. Headline: deployment context capacity.
41. Counterpart reference: external counterparts operate centralized hyperscale infrastructure; Oyatie must support six deployment contexts.
42. Oyatie target posture: catch-up; current social has no six-context OpenTofu modules.
43. Headline: tenant-class economics.
44. Counterpart reference: external public products monetize differently; Oyatie must use `demo_trial`, `paid`, and `revenue_share`.
45. Oyatie target posture: catch-up; no tenant-class semantics exist in social artifacts.
46. Headline: source/test benchmark validity.
47. Counterpart reference: public counterpart numbers are production-scale disclosures.
48. Oyatie target posture: no measured evidence yet; source, tests, and benchmarks are absent.
49. Evidence: no `microservices/social/src/`, no `microservices/social/tests/`, and empty `microservices/social/benchmarks/`.
50. Final comparison: the proposed Oyatie numbers are credible targets only after product realignment, OpenTofu context modules, tenant-class contract adoption, and executable benchmark harness creation.
