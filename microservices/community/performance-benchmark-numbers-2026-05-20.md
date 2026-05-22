# community performance benchmark numbers — 2026-05-20

Audit owner: solo Codex audit lane.
Target µservice: `community`.
Counterparts: Discourse, Circle, Vanilla Forums.
Benchmark shape: single industry-leader target set with overlays.
Retired shape: no demo_trial/paid/paid advanced/paid compliance-pack rows, headings, or target ladders.
Local performance anchor 1: `microservices/community/PRD.md:864-879`.
Local performance anchor 2: `microservices/community/PRD.md:881-900`.
Local rate-limit anchor: `microservices/community/PRD.md:913-916`.
Local post-create SLO anchor: `microservices/community/slos/post-create-latency.openslo.yaml:1-44`.
Local search-backend anchor: `microservices/community/decisions/ADR-COMM-0004-content-search-backend.md:66-89`.
Local OpenTofu gap anchor: `microservices/community/coherence-audit-2026-05-20.md §3.6-§3.7`.
Canonical deployment-context anchor: `specs/master-plan-sequencing.json:704-746`.
Canonical OCI Always Free anchor: `specs/master-plan-sequencing.json:857-867`.
Canonical no-tier anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_customer_class_ladders_2026_05_20.md:1-45`.
Canonical tenant-class anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-142`.
Discourse feature source: https://discourse.org/features.
Discourse hosted performance source: https://www.discourse.org/meta.
Discourse enterprise source: https://www.discourse.org/enterprise.
Discourse install source: https://raw.githubusercontent.com/discourse/discourse/main/docs/INSTALL.md.
Circle platform source: https://circle.so/platform.
Circle developer limits source: https://api.circle.so/apis/admin-api/usage-and-limits.
Vanilla API limits source: https://success.vanillaforums.com/kb/articles/44-rate-limits.
Vanilla moderation source: https://success.vanillaforums.com/kb/articles/342-moderation-process-tools.
Vanilla analytics source: https://success.vanillaforums.com/kb/articles/1505-out-of-the-box-dashboards.
Core Web Vitals source: https://web.dev/articles/defining-core-web-vitals-thresholds.

## §1 Methodology

M-001 Public benchmark disclosure: the three counterparts publish product limits and scale claims, but not comprehensive p50/p95/p99 API latency for every community operation.
M-002 Source policy: every public number below is labeled `source: public` when it comes from an official page.
M-003 Estimate policy: every derived target is labeled `source: estimated` and derived from public limits, Core Web Vitals thresholds, or Oyatie's local PRD/SLO targets.
M-004 Local target policy: Oyatie numbers are target numbers, not measured implementation results.
M-005 Implementation caveat: no `src/` or `tests/` directory exists under `microservices/community/`, so measured implementation benchmarks cannot yet be produced.
M-006 Infrastructure caveat: six canonical OpenTofu context modules are missing, so context overlays are design targets, not deployed proof.
M-007 Counterpart workload: read-heavy forum feed workloads use 200:1 feed read to post create from local capacity model `microservices/community/capacity-model.md:28-32`.
M-008 Counterpart workload: voting workloads use `vote cast : vote read = 1:30` from local capacity model `microservices/community/capacity-model.md:30-32`.
M-009 Counterpart workload: KB workloads use `KB article view : edit = 300:1` from local capacity model `microservices/community/capacity-model.md:32`.
M-010 UI benchmark baseline: Core Web Vitals good thresholds are LCP <=2500 ms, INP <=200 ms, CLS <=0.1 at the 75th percentile, per official web.dev source.
M-011 API benchmark baseline: service-local post-create good threshold is <=250 ms at the SLO query bucket, per OpenSLO.
M-012 API benchmark baseline: local PRD targets feed render p50 <=80 ms, p99 <=300 ms, p999 <=800 ms.
M-013 API benchmark baseline: local PRD targets search query p50 <=120 ms, p99 <=500 ms, p999 <=1.2 s.
M-014 API benchmark baseline: local PRD targets vote cast p50 <=25 ms, p99 <=100 ms, p999 <=300 ms.
M-015 API benchmark baseline: local PRD targets post create p50 <=80 ms, p99 <=250 ms, p999 <=700 ms.
M-016 API benchmark baseline: local PRD targets moderation action seal p50 <=80 ms, p99 <=200 ms, p999 <=500 ms.
M-017 Scale baseline: local PRD targets 1M posts per tenant per month.
M-018 Scale baseline: local PRD targets 10K concurrent WebSocket connections per cell.
M-019 Scale baseline: local PRD targets 100K search queries per second per cell.
M-020 Scale baseline: local PRD targets 5TB search index per cell and 256 shards per cell.
M-021 OS disclosure: no service-local `supported-oses.json` exists, so OS-specific performance cannot be claimed.
M-022 Architecture disclosure: target CPU architectures follow the canonical matrix, but local service evidence is missing.
M-023 Deployment-context disclosure: overlays cover all six contexts from the canonical master-plan sequencing file.
M-024 Tenant-class disclosure: overlays use `demo_trial`, `paid`, and `revenue_share` per the user prompt for this audit.
M-025 Tenant-class caveat: local service artifacts do not yet implement tenant_class semantics.
M-026 Workload A: feed list with hot-cache hit, tenant-scoped policy filter, and 50 posts.
M-027 Workload B: post create with audit-chain enqueue, search fan-out, and notification enqueue.
M-028 Workload C: vote cast with idempotency key and async durable flush.
M-029 Workload D: cross-BC search across announcements, questions, KB, discussions, and tags.
M-030 Workload E: moderation action with Cedar decision, audit-chain seal, and queue update.
M-031 Workload F: KB article render from cached revision.
M-032 Workload G: WebSocket presence update.
M-033 Workload H: tenant export and migration import.
M-034 Metric dimensions: latency p50, p95, p99, p999, throughput, concurrency, rate limit, scale ceiling, availability, resource floor, and UI web vitals.
M-035 Stop condition: produce target numbers sufficient to compare to counterparts without inventing hidden vendor internals.

## §2 Counterpart numbers

### §2.1 Discourse numbers

D-N001 Source public: Discourse hosted page states 99.9% uptime SLA.
D-N002 Source public: Discourse hosted page states 3M+ monthly posts created across hosting.
D-N003 Source public: Discourse hosted page states 1B+ monthly page views served across hosting.
D-N004 Source public: Discourse hosted page states hosted individual sites with 100K+ monthly active users.
D-N005 Source public: Discourse hosted page states hosted individual sites with 45M+ monthly views.
D-N006 Source public: Discourse enterprise page states more than 22,000 communities.
D-N007 Source public: Discourse enterprise page states 99.9% uptime SLA.
D-N008 Source public: Discourse enterprise page states 80+ integrations.
D-N009 Source public: Discourse install doc requires modern single-core CPU, with dual-core recommended.
D-N010 Source public: Discourse install doc requires 1GB RAM minimum with swap.
D-N011 Source public: Discourse install doc requires 10GB disk minimum.
D-N012 Source public: Discourse install doc requires 64-bit Linux compatible with Docker.
D-N013 Source public: Discourse install doc requires PostgreSQL.
D-N014 Source public: Discourse install doc requires Valkey 7.
D-N015 Source public: Discourse install doc requires Ruby 3.2+.
D-N016 Source estimated: Discourse API/forum p75 UI target should satisfy Core Web Vitals LCP <=2500 ms for good UX; public Discourse-specific LCP not published.
D-N017 Source estimated: Discourse interaction target should satisfy INP <=200 ms at p75; public Discourse-specific INP not published.
D-N018 Source estimated: Discourse layout stability should satisfy CLS <=0.1 at p75; public Discourse-specific CLS not published.
D-N019 Comparison note: public Discourse scale numbers are whole-hosting aggregate and large-site examples, not per-tenant API latency.
D-N020 Comparison note: Discourse's public minimum resource floor is much smaller than Oyatie's target service-cell floor, but Oyatie owns broader compliance and multi-context goals.

### §2.2 Circle numbers

C-N001 Source public: Circle platform page states more than 15M members.
C-N002 Source public: Circle platform page states 20,000 communities.
C-N003 Source public: Circle platform page states $194M creator revenue in the last year.
C-N004 Source public: Circle platform page states 48K courses in the last year.
C-N005 Source public: Circle platform page advertises a 14-day free trial.
C-N006 Source public: Circle developer limits state Business plan Admin API allotment of 5,000 requests/month.
C-N007 Source public: Circle developer limits state Enterprise and Circle Plus Admin API allotment of 30,000 requests/month.
C-N008 Source public: Circle developer limits state Circle Plus Platform Admin API allotment of 250,000 requests/month.
C-N009 Source public: Circle developer limits state API rate limit of 2,000 requests per 5 minutes per IP.
C-N010 Source derived: Circle's public per-IP API rate is 400 requests/min/IP.
C-N011 Source derived: Circle's public per-IP API rate is about 6.67 requests/sec/IP sustained over the 5-minute window.
C-N012 Source public: Circle developer limits say usage counts are cached and can update around 5 minutes after calls.
C-N013 Source public: Circle developer limits count 200, 201, 204, 400, 401, 403, 404, 405, 422, and 429 responses against API usage.
C-N014 Source public: Circle developer overview says headless APIs cover discussions, feed, notifications, events, and more.
C-N015 Source estimated: Circle public UI should meet Core Web Vitals LCP <=2500 ms p75 for good UX; Circle-specific LCP not published.
C-N016 Source estimated: Circle public UI should meet INP <=200 ms p75 for good UX; Circle-specific INP not published.
C-N017 Source estimated: Circle public UI should meet CLS <=0.1 p75 for good UX; Circle-specific CLS not published.
C-N018 Comparison note: Circle publishes business-scale and API-limit numbers, but not community-operation p99 latency.
C-N019 Comparison note: Circle's product scope includes courses, events, live, payments, AI agents, website builder, and CRM, so throughput comparisons must separate community-core paths from sibling-owned Oyatie surfaces.
C-N020 Comparison note: Circle API monthly allotments are plan-based; Oyatie must translate its model into tenant_class and billing-component caps without capability profiles.

### §2.3 Vanilla Forums numbers

V-N001 Source public: Vanilla API GET requests are limited to 300 requests per 1 minute per IP.
V-N002 Source public: Vanilla API write requests are limited to 120 requests per 1 minute per IP.
V-N003 Source public: Vanilla hard limit is more than 250 requests within 10 seconds.
V-N004 Source public: Vanilla temporary block lifts automatically after 1 minute.
V-N005 Source public: Vanilla returns HTTP 429 Too Many Requests during rate-limit blocks.
V-N006 Source derived: Vanilla GET rate is 5 requests/sec/IP sustained.
V-N007 Source derived: Vanilla write rate is 2 requests/sec/IP sustained.
V-N008 Source derived: Vanilla hard burst threshold is 25 requests/sec/IP over 10 seconds.
V-N009 Source public: Vanilla moderation docs identify Spam Queue and Moderation Queue surfaces.
V-N010 Source public: Vanilla moderation docs identify Change Log for edited/deleted posts.
V-N011 Source public: Vanilla moderation docs identify inline moderation actions on single or multiple posts.
V-N012 Source public: Vanilla analytics docs identify out-of-the-box dashboards and custom dashboards.
V-N013 Source public: Vanilla gamification docs identify points, badges, ranks, and reactions.
V-N014 Source estimated: Vanilla dashboard UI should meet LCP <=2500 ms p75 for good UX; Vanilla-specific LCP not published.
V-N015 Source estimated: Vanilla dashboard UI should meet INP <=200 ms p75 for good UX; Vanilla-specific INP not published.
V-N016 Source estimated: Vanilla dashboard UI should meet CLS <=0.1 p75 for good UX; Vanilla-specific CLS not published.
V-N017 Source estimated: moderation queue list p95 should stay under 1000 ms for operator usability; vendor p95 not published.
V-N018 Source estimated: post/comment create p95 should stay under 500 ms for forum usability; vendor p95 not published.
V-N019 Comparison note: Vanilla publishes useful API limits but not whole-community installed-base or per-operation latency numbers.
V-N020 Comparison note: Vanilla's rate limits are stricter than Circle's per-IP limit but clearer for write throttling.

## §3 Oyatie target numbers — single industry-leader target set

T-001 Feed render p50: <=80 ms canonical target; source local PRD.
T-002 Feed render p95: <=200 ms canonical target; derived from p99 <=300 ms local target.
T-003 Feed render p99: <=300 ms canonical target; source `microservices/community/PRD.md:864-867`.
T-004 Feed render p999: <=800 ms canonical target; source local PRD.
T-005 Search query p50: <=120 ms canonical target; source local PRD.
T-006 Search query p95: <=350 ms canonical target; derived from p99 <=500 ms local target.
T-007 Search query p99: <=500 ms canonical target; source `microservices/community/PRD.md:867`.
T-008 Search query p999: <=1200 ms canonical target; source local PRD.
T-009 Vote cast p50: <=25 ms canonical target; source local PRD.
T-010 Vote cast p95: <=75 ms canonical target; derived from p99 <=100 ms local target.
T-011 Vote cast p99: <=100 ms canonical target; source `microservices/community/PRD.md:868`.
T-012 Vote cast p999: <=300 ms canonical target; source local PRD.
T-013 Post create p50: <=80 ms canonical target; source local PRD.
T-014 Post create p95: <=175 ms canonical target; derived from p99 <=250 ms local target.
T-015 Post create p99: <=250 ms canonical target; source local PRD and OpenSLO.
T-016 Post create p999: <=700 ms canonical target; source local PRD.
T-017 Post edit p50: <=80 ms canonical target; source local PRD.
T-018 Post edit p95: <=175 ms canonical target; derived from p99 <=250 ms local target.
T-019 Post edit p99: <=250 ms canonical target; source local PRD.
T-020 Post edit p999: <=700 ms canonical target; source local PRD.
T-021 KB article publish p50: <=200 ms canonical target; source local PRD.
T-022 KB article publish p95: <=350 ms canonical target; derived from p99 <=500 ms local target.
T-023 KB article publish p99: <=500 ms canonical target; source local PRD.
T-024 KB article publish p999: <=1500 ms canonical target; source local PRD.
T-025 KB cached render p50: <=50 ms canonical target; source local PRD.
T-026 KB cached render p95: <=125 ms canonical target; derived from p99 <=200 ms local target.
T-027 KB cached render p99: <=200 ms canonical target; source local PRD.
T-028 KB cached render p999: <=500 ms canonical target; source local PRD.
T-029 Moderation action seal p50: <=80 ms canonical target; source local PRD.
T-030 Moderation action seal p95: <=150 ms canonical target; derived from p99 <=200 ms local target.
T-031 Moderation action seal p99: <=200 ms canonical target; source local PRD.
T-032 Moderation action seal p999: <=500 ms canonical target; source local PRD.
T-033 Audit-chain seal latency p50: <=200 ms canonical target; source local PRD.
T-034 Audit-chain seal latency p95: <=600 ms canonical target; derived from p99 <=800 ms local target.
T-035 Audit-chain seal latency p99: <=800 ms canonical target; source local PRD.
T-036 Audit-chain seal latency p999: <=1500 ms canonical target; source local PRD.
T-037 Threaded reply render for 1000 nodes p50: <=100 ms canonical target; source local PRD.
T-038 Threaded reply render p95: <=250 ms canonical target; derived from p99 <=350 ms local target.
T-039 Threaded reply render p99: <=350 ms canonical target; source local PRD.
T-040 Threaded reply render p999: <=900 ms canonical target; source local PRD.
T-041 Notification fan-out p50: <=1000 ms canonical target; source local PRD.
T-042 Notification fan-out p95: <=3000 ms canonical target; derived from p99 <=5000 ms local target.
T-043 Notification fan-out p99: <=5000 ms canonical target; source local PRD.
T-044 Notification fan-out p999: <=15000 ms canonical target; source local PRD.
T-045 Mention resolution p50: <=30 ms canonical target; source local PRD.
T-046 Mention resolution p95: <=75 ms canonical target; derived from p99 <=100 ms local target.
T-047 Mention resolution p99: <=100 ms canonical target; source local PRD.
T-048 Mention resolution p999: <=300 ms canonical target; source local PRD.
T-049 Federation outbound p50: <=5 s canonical target; source local PRD.
T-050 Federation outbound p95: <=20 s canonical target; derived from p99 <=30 s local target.
T-051 Federation outbound p99: <=30 s canonical target; source local PRD.
T-052 Federation outbound p999: <=120 s canonical target; source local PRD.
T-053 Presence update p50: <=100 ms canonical target; source local PRD.
T-054 Presence update p95: <=350 ms canonical target; derived from p99 <=500 ms local target.
T-055 Presence update p99: <=500 ms canonical target; source local PRD.
T-056 Presence update p999: <=2000 ms canonical target; source local PRD.
T-057 Read availability: >=99.95% monthly canonical target; source local PRD.
T-058 Write availability: >=99.9% monthly canonical target; source local PRD.
T-059 Realtime availability: >=99.9% monthly canonical target; source local PRD.
T-060 Federation availability: >=99.5% monthly canonical target; source local PRD.
T-061 RTO: <=15 min canonical target; source local PRD.
T-062 RPO: <=30 s canonical target; source local PRD.
T-063 Search index ceiling: 5TB per cell canonical target; source local PRD.
T-064 Tenant fallback index ceiling: 100GB per tenant canonical target; source local PRD.
T-065 Sustained ingest: 1M posts per tenant per month canonical target; source local PRD.
T-066 WebSocket concurrency: 10K connections per cell canonical target; source local PRD.
T-067 Search read throughput: 100K qps per cell canonical target; source local PRD.
T-068 Shard ceiling: 256 shards per cell canonical target; source local PRD.
T-069 Per-member post create limit: <=60 per minute canonical guardrail; source local PRD.
T-070 Per-member vote limit: <=600 per minute canonical guardrail; source local PRD.
T-071 Per-member report limit: <=30 per minute canonical guardrail; source local PRD.
T-072 Web LCP target: <=2500 ms at p75; source Core Web Vitals.
T-073 Web INP target: <=200 ms at p75; source Core Web Vitals.
T-074 Web CLS target: <=0.1 at p75; source Core Web Vitals.
T-075 API abuse IP default target: <=300 read requests/min/IP for unauthenticated or weakly trusted clients; derived from Vanilla.
T-076 API abuse write default target: <=120 write requests/min/IP for unauthenticated or weakly trusted clients; derived from Vanilla.
T-077 API authenticated platform ceiling: >=2000 requests/5min/IP before abuse throttling for trusted integrations; derived from Circle.
T-078 API monthly allotment: tenant_class-owned, not service-local feature gating; target emitted to cloud-billing meters.
T-079 Import backfill target: >=10K posts/sec only where infrastructure profile can sustain it; inherited from local Discourse migration playbook but must be revalidated without retired tier language.
T-080 Export target: full tenant export must run within tenant contract window; exact throughput should be sized by data volume and deployment context.

## §3.1 Deployment-context overlays

DC-001 `oyatie-public-cloud`: target all canonical latency and throughput numbers with elastic cell scaling.
DC-002 `oyatie-public-cloud`: read availability target stays >=99.95% monthly.
DC-003 `oyatie-public-cloud`: write availability target stays >=99.9% monthly.
DC-004 `oyatie-public-cloud`: deployment blocker is missing `iac/oyatie-public-cloud/`.
DC-005 `guest-on-aws`: target canonical latency where AWS regional resources meet storage/network floor.
DC-006 `guest-on-aws`: throughput ceiling depends on customer account quotas and approved OpenTofu module shape.
DC-007 `guest-on-aws`: deployment blocker is missing `iac/guest-on-aws/`.
DC-008 `guest-on-oci`: target canonical latency for paid OCI shape.
DC-009 `guest-on-oci`: OCI Always Free profile caps aggregate demo_trial substrate to 4 OCPU + 24GB RAM total, 200GB block, 10GB object, 10GB archive, 2 Autonomous DB x20GB, 10Mbps LB, and 10TB egress/month.
DC-010 `guest-on-oci`: demo_trial OCI Always Free search throughput should start at <=250 qps/cell until measured.
DC-011 `guest-on-oci`: demo_trial OCI Always Free WebSocket concurrency should start at <=1000 connections/cell until measured.
DC-012 `guest-on-oci`: demo_trial OCI Always Free sustained ingest should start at <=50K posts/tenant/month until measured.
DC-013 `guest-on-oci`: deployment blocker is missing `iac/oci-guest/` and `iac/oci-guest/always-free/`.
DC-014 `on-prem`: target canonical latency only when customer hardware meets a published capacity class.
DC-015 `on-prem`: facility network, storage, and operator constraints may reduce throughput without changing feature quality.
DC-016 `on-prem`: deployment blocker is missing `iac/on-prem/`.
DC-017 `colo`: target canonical latency on dedicated hardware when power/network/storage redundancy meets the service-cell floor.
DC-018 `colo`: throughput ceiling is facility-specific and must be declared in the OpenTofu plan output.
DC-019 `colo`: deployment blocker is missing `iac/colo/`.
DC-020 `oyatie-as-cloud-provider`: target canonical latency with Oyatie-owned IaaS primitives.
DC-021 `oyatie-as-cloud-provider`: throughput should meet or exceed public-cloud context after cloud-* primitives mature.
DC-022 `oyatie-as-cloud-provider`: deployment blocker is missing `iac/oyatie-iaas/`.
DC-023 all contexts: telemetry must include deployment context, cell, region, tenant, and service labels.
DC-024 all contexts: no context is verified deployable until OpenTofu module evidence exists.
DC-025 all contexts: Terraform artifacts do not count as accepted deployment proof.

## §3.2 Tenant-class overlays

TC-001 `demo_trial`: quality bar remains industry-leader-grade.
TC-002 `demo_trial`: usage caps constrain total volume, not feature availability.
TC-003 `demo_trial`: default infrastructure profile is OCI Always Free where using OCI.
TC-004 `demo_trial`: target post-create p99 remains <=250 ms until cap breach; after cap breach, requests should fail fast with a policy/billing reason.
TC-005 `demo_trial`: target search p99 remains <=500 ms for allowed usage; search qps cap starts at <=250 qps/cell until measured.
TC-006 `demo_trial`: target WebSocket concurrency cap starts at <=1000/cell until measured.
TC-007 `demo_trial`: target sustained ingest cap starts at <=50K posts/tenant/month until measured.
TC-008 `demo_trial`: best-effort SLO disclosure applies; no contractual credit.
TC-009 `demo_trial`: compliance packs and BYOK are denied by tenant_class policy.
TC-010 `paid`: quality bar remains industry-leader-grade.
TC-011 `paid`: canonical latency and availability targets apply contractually where deployment context supports them.
TC-012 `paid`: usage scales with per-seat and usage-based billing.
TC-013 `paid`: compliance packs may be activated where pack prerequisites are satisfied.
TC-014 `paid`: BYOK may be activated where cloud-secrets and deployment context support it.
TC-015 `paid`: search qps target can scale to 100K qps/cell with payment and capacity.
TC-016 `paid`: WebSocket concurrency target can scale to 10K/cell with payment and capacity.
TC-017 `paid`: sustained ingest target can scale to 1M posts/tenant/month with payment and capacity.
TC-018 `revenue_share`: quality bar remains industry-leader-grade.
TC-019 `revenue_share`: infrastructure may run at cost or zero-margin substrate while Oyatie receives gross-revenue share.
TC-020 `revenue_share`: usage caps are contract-specific but should not feature-gate the service.
TC-021 `revenue_share`: marketplace seller and B2C operator workloads should emit revenue-share meter events.
TC-022 `revenue_share`: target latency remains canonical unless the contract pins a lower-cost deployment context.
TC-023 `revenue_share`: compliance and BYOK eligibility follow contract and regulatory posture, not a feature tier.
TC-024 all classes: local service currently lacks tenant_class evidence, so these overlays are target doctrine, not implemented proof.
TC-025 all classes: tenant_class should be enforced by IAM/gateway/Cedar and cloud-billing, not request parameters in community APIs.

## §4 Comparison narrative

CN-001 Feed latency: Oyatie target p99 <=300 ms is ahead of public counterpart evidence because counterparts do not publish per-feed p99; claim is target-only.
CN-002 Search latency: Oyatie target p99 <=500 ms is parity with local PRD and more explicit than public counterpart docs; implementation proof absent.
CN-003 Vote latency: Oyatie target p99 <=100 ms is ahead of public counterpart evidence; implementation proof absent.
CN-004 Post create latency: Oyatie target p99 <=250 ms is ahead of public counterpart evidence and bound by OpenSLO; implementation proof absent.
CN-005 Moderation seal latency: Oyatie target p99 <=200 ms is additive because counterparts do not expose audit-chain seal semantics.
CN-006 Audit seal latency: Oyatie target p99 <=800 ms is additive, but failure-mode behavior must be reconciled with P0 missing-seal policy.
CN-007 UI LCP: Oyatie target <=2500 ms p75 is parity with Core Web Vitals good threshold.
CN-008 UI INP: Oyatie target <=200 ms p75 is parity with Core Web Vitals good threshold.
CN-009 UI CLS: Oyatie target <=0.1 p75 is parity with Core Web Vitals good threshold.
CN-010 Read availability: Oyatie 99.95% read target is ahead of Discourse's public 99.9% hosted SLA, but only as design target.
CN-011 Write availability: Oyatie 99.9% write target is parity with Discourse public 99.9% SLA.
CN-012 Realtime availability: Oyatie 99.9% realtime target is parity with SaaS expectations, but meet/shorts handoff proof is absent.
CN-013 Federation availability: Oyatie 99.5% target is lower than read/write targets and explicitly best-effort.
CN-014 RTO/RPO: Oyatie RTO <=15 min and RPO <=30 s are explicit; counterparts do not publish equivalent forum-operation RTO/RPO in cited pages.
CN-015 Scale posts: Oyatie 1M posts/tenant/month is smaller than Discourse aggregate 3M+ hosted monthly posts only when comparing one tenant to all hosted Discourse; per-tenant target remains aggressive.
CN-016 Scale page views: Discourse public 1B+ monthly page views is a fleet-scale counterpart number; Oyatie has not declared page-view fleet target.
CN-017 Scale communities: Discourse 22,000+ communities and Circle 20,000 communities are installed-base numbers; Oyatie has no installed-base proof.
CN-018 Scale members: Circle 15M+ members is an installed-base number; Oyatie has no installed-base proof.
CN-019 API rate limits: Circle allows about 400 requests/min/IP and Vanilla allows 300 GET/min/IP and 120 write/min/IP; Oyatie member limits are explicit but IP/admin/API limits need definition.
CN-020 Monthly API allotments: Circle publishes 5,000/30,000/250,000 Admin API request allotments; Oyatie should express equivalent caps through tenant_class and billing components.
CN-021 Resource floor: Discourse can install on 1GB RAM minimum; Oyatie's multi-service, Rust, compliance, and multi-context posture requires different resource floors and cannot claim the same low-footprint install until measured.
CN-022 OCI Always Free: Oyatie must prove reduced demo_trial caps on 4 OCPU/24GB before claiming Always Free viability.
CN-023 On-prem/colo: Oyatie can exceed SaaS counterparts if OpenTofu modules and OS manifest land; current proof is absent.
CN-024 Revenue-share class: no counterpart in this set maps cleanly; Oyatie target is a business-model overlay, not a performance differentiator.
CN-025 Overall status: target numbers are coherent and industry-leader-grade, but they remain design targets until Rust implementation, tests, OpenTofu modules, and context-specific benchmarks exist.

## §5 Benchmark action list

BA-001 Add `benches/feed_render.rs` once Rust implementation exists.
BA-002 Add `benches/post_create.rs` to prove p50/p95/p99/p999 against the OpenSLO bucket.
BA-003 Add `benches/vote_cast.rs` for idempotency and tally drift under concurrency.
BA-004 Add `benches/search_query.rs` for Meilisearch and Tantivy profiles.
BA-005 Add `benches/moderation_action.rs` for Cedar and audit seal latency.
BA-006 Add load scenario `community_feed_read_200_to_1_write`.
BA-007 Add load scenario `community_vote_1_to_30_read`.
BA-008 Add load scenario `community_kb_view_300_to_1_edit`.
BA-009 Add deployment profile `oyatie-public-cloud`.
BA-010 Add deployment profile `guest-on-aws`.
BA-011 Add deployment profile `guest-on-oci`.
BA-012 Add deployment profile `guest-on-oci-always-free`.
BA-013 Add deployment profile `on-prem`.
BA-014 Add deployment profile `colo`.
BA-015 Add deployment profile `oyatie-iaas`.
BA-016 Emit tenant_class label in benchmark telemetry.
BA-017 Emit billing_component label for paid/revenue-share overlays.
BA-018 Emit deployment_context label in every benchmark result.
BA-019 Emit os_id and arch labels once `supported-oses.json` exists.
BA-020 Compare benchmark output to the single target set in §3, not to retired feature tiers.
