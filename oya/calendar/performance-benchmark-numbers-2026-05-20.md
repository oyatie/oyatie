# Calendar Performance Benchmark Numbers

Audit date: 2026-05-20.
Target microservice: `calendar`.
Benchmark model: one industry-leader target set with deployment-context overlays and tenant_class overlays.
No customer capability ladder is used in this document.

Five-citation anchor block:
- Calendar PRD performance targets: `microservices/calendar/PRD.md:61-70`.
- Calendar PRD scalability targets: `microservices/calendar/PRD.md:248-283`.
- Calendar capacity envelope: `microservices/calendar/capacity-model.md:34-58`.
- Google Calendar API usage limits: `https://developers.google.com/workspace/calendar/api/guides/quota`.
- Microsoft Graph Outlook service limits: `https://learn.microsoft.com/en-us/graph/throttling-limits`.
- Cal.com API v2 rate limits and authentication behavior: `https://cal.com/docs/api-reference/v2/introduction`.

Methodology disclosure:
- Public counterpart latency distributions are not consistently published by Google, Microsoft, or Cal.com.
- Where counterpart vendors publish request limits, page sizes, expansion caps, or concurrency limits, this document cites those public numbers directly.
- Where counterpart vendors do not publish latency, the Oyatie target uses calendar-local PRD targets plus an industry-leader requirement to meet or beat the strongest disclosed counterpart constraint.
- Performance targets are not segmented by customer capability classes.
- Performance targets are adjusted only by deployment_context infrastructure caps and tenant_class usage policy.
- The tenant_class terms used for this deliverable are `demo_trial`, `paid`, and `revenue_share` because the batch directive requires those overlays.
- The coherence audit records the remaining canonical naming tension around whether `revenue_share` is a class or a billing component.

## §1 Methodology

Benchmark dimensions:
1. Event read latency p50.
2. Event read latency p95.
3. Event read latency p99.
4. Event write latency p50.
5. Event write latency p95.
6. Event write latency p99.
7. Availability/freebusy latency p50.
8. Availability/freebusy latency p95.
9. Availability/freebusy latency p99.
10. Recurrence expansion latency p50.
11. Recurrence expansion latency p95.
12. Recurrence expansion latency p99.
13. Room conflict check latency p99.
14. RSVP fanout latency p99.
15. `.ics` import throughput.
16. `.ics` export throughput.
17. CalDAV PROPFIND latency p99.
18. Event fetch throughput.
19. Event write throughput.
20. Availability lookup throughput.
21. Concurrent CalDAV sessions.
22. Calendar count per production cell.
23. Calendar count per compact demo cell.
24. API fairness per tenant.
25. Burst rejection or shaping behavior.

Test workloads:
26. Single-event read by tenant and event id.
27. Agenda read over a seven-day window.
28. Event create with organizer, attendees, location, reminders, and audit emission.
29. Event update with recurrence exception.
30. Event delete with RSVP and audit fanout.
31. Availability query for 10 attendees over seven days.
32. Availability query for 50 attendees over 30 days.
33. Availability query for 100 attendees over 30 days.
34. Cross-tenant freebusy projection with private fields redacted.
35. Room-booking conflict check with simultaneous contenders.
36. Recurrence expansion for weekly rules across one year.
37. Recurrence expansion for dense rules across 10,000 generated instances.
38. `.ics` import of 10,000 events.
39. `.ics` export of 10,000 events.
40. CalDAV PROPFIND for a 1,000-event collection page.
41. CalDAV sync loop with unchanged state.
42. RSVP fanout to 100 attendees.
43. Timezone database refresh and staleness probe.
44. Legal-hold protected event mutation.
45. Dashboard and SLO metric emission under load.

OS disclosure:
46. This document sets runtime targets independent of OS where Rust backend behavior should be portable.
47. Calendar currently lacks a `supported-oses.json` manifest, recorded as a P1 finding in the coherence audit.
48. OS-specific performance is therefore a required later validation lane rather than a proven current property.
49. The target OS matrix must eventually cover backend services, worker processes, admin tools, and client surfaces.
50. The current performance numbers assume Linux production cells until the OS manifest exists.

Architecture disclosure:
51. Production backend target: Rust services and workers.
52. Storage target: Postgres for durable event state.
53. Cache target: Valkey or equivalent cache layer for availability projections.
54. Queue/event target: Async event fanout through Oyatie workflow/event substrate.
55. Protocol target: OpenAPI, AsyncAPI, proto, `.ics`, and CalDAV.
56. Deployment target: all six canonical deployment contexts after OpenTofu modules exist.
57. Current IaC gap: only Helm/Kustomize exists in the calendar path.
58. Current source gap: no `src/` tree exists in the calendar path.
59. Current test gap: no `tests/` tree exists in the calendar path.
60. Therefore these numbers are targets and audit thresholds, not measured current production results.

Deployment-context disclosure:
61. `oyatie-public-cloud` is expected to scale elastically and preserve the canonical target set.
62. `guest-on-aws` is expected to preserve the canonical target set when the customer provisions enough substrate.
63. `guest-on-oci` is expected to preserve the canonical target set for paid capacity and apply a separate OCI Always Free profile for demo-trial use.
64. `on-prem` is expected to preserve the canonical target set subject to facility storage, network, and hardware constraints.
65. `colo` is expected to preserve the canonical target set subject to facility network and hardware constraints.
66. `oyatie-as-cloud-provider` is expected to preserve the canonical target set using Oyatie-owned cloud substrate.
67. No context may claim the target set without an OpenTofu module or explicit N/A decision.

Tenant_class disclosure:
68. `demo_trial` targets the same quality bar while applying hard usage caps and best-effort SLO.
69. `paid` targets the same quality bar and scales with purchased capacity plus usage.
70. `revenue_share` targets the same quality bar and runs at-cost or zero-margin substrate when commercially required.
71. Tenant_class overlays constrain allocation and admission, not feature quality.
72. The same latency SLO should hold until the tenant reaches its explicit cap.
73. Once a tenant reaches its cap, the system should reject, queue, or shape work with explicit errors instead of degrading correctness.

## §2 Counterpart Numbers

### §2.1 Google Calendar

Source summary:
1. Google Calendar usage-limit docs disclose two enforced quota dimensions.
2. The quota dimensions are per minute per project and per minute per user per project.
3. Google Calendar quotas use a sliding-window model.
4. Google Calendar docs disclose a daily billing threshold.
5. Google Calendar freebusy docs disclose group and calendar expansion maxima.
6. Google Calendar events docs disclose default and maximum event-list page sizes.
7. Google does not publish public p50/p95/p99 Calendar API latency distributions in the cited docs.
8. Google public numbers are therefore used as limit and scale comparators, not latency observations.

Google benchmark numbers:
9. Per-minute project quota: 10,000 requests per minute, source `https://developers.google.com/workspace/calendar/api/guides/quota`.
10. Per-minute per-user per-project quota: 600 requests per minute, same source.
11. Daily billing threshold: 1,000,000 requests per day per project, same source.
12. Freebusy group expansion maximum: 100 calendar identifiers, source `https://developers.google.com/workspace/calendar/api/v3/reference/freebusy/query`.
13. Freebusy calendar expansion maximum: 50 calendars, same source.
14. Events list default page size: 250 events, source `https://developers.google.com/workspace/calendar/api/v3/reference/events/list`.
15. Events list maximum page size: 2,500 events, same source.
16. Published official latency distribution count in cited docs: 0 p50/p95/p99 values.
17. Published official CalDAV latency count in cited docs: 0 values.
18. Published official recurrence expansion latency count in cited docs: 0 values.
19. Published official `.ics` import throughput count in cited docs: 0 values.
20. Public comparison interpretation: Google sets a high API-scale floor but does not provide latency targets to copy.

Google implications for Oyatie:
21. Oyatie should support at least 100 identities in a group availability query when policy permits.
22. Oyatie should support at least 50 explicit calendars in a single freebusy query when policy permits.
23. Oyatie should support event-list pages of at least 2,500 events for backend API consumers.
24. Oyatie should enforce tenant fairness at a finer grain than only project-level quotas.
25. Oyatie should publish its own latency SLOs because Google does not expose comparable public latency numbers.

### §2.2 Microsoft Outlook Calendar

Source summary:
26. Microsoft Graph Outlook service limits disclose mailbox-scoped request and concurrency constraints.
27. Microsoft Graph Outlook service limits apply per app id and mailbox combination.
28. Microsoft Graph getSchedule discloses availability slot interval bounds.
29. Microsoft Graph getSchedule discloses a 1,000-entry response failure threshold.
30. Microsoft Graph findMeetingTimes discloses confidence scoring inputs and defaults.
31. Microsoft Graph list-events docs disclose event listing routes and UTC/timezone behavior.
32. Microsoft public docs do not disclose p50/p95/p99 Outlook Calendar latency distributions.
33. Microsoft public numbers are therefore used as mailbox, concurrency, schedule, and suggestion comparators.

Microsoft benchmark numbers:
34. Outlook mailbox-scoped request limit: 10,000 API requests in a 10-minute period, source `https://learn.microsoft.com/en-us/graph/throttling-limits`.
35. Outlook mailbox-scoped concurrency limit: 4 concurrent requests, same source.
36. Outlook upload write volume limit: 150 MB in a 5-minute period for upload operations, same source.
37. JSON batching concurrency sent to Outlook service: up to 4 individual requests at a time, same source.
38. Places API throttle: 3 calls per second, source `https://learn.microsoft.com/en-us/graph/throttling-limits`.
39. Subscription write limit per app across tenants: 2,000 requests per 20 seconds, same source.
40. Subscription write limit per app per tenant: 500 requests per 20 seconds, same source.
41. getSchedule default availability interval: 30 minutes, source `https://learn.microsoft.com/en-us/graph/api/calendar-getschedule`.
42. getSchedule minimum availability interval: 5 minutes, same source.
43. getSchedule maximum availability interval: 1,440 minutes, same source.
44. getSchedule oversized slot response threshold: more than 1,000 entries returns a documented error response, same source.
45. findMeetingTimes confidence range: 0 percent to 100 percent, source `https://learn.microsoft.com/en-us/graph/api/user-findmeetingtimes`.
46. findMeetingTimes default minimum attendee confidence: 50 percent, same source.
47. findMeetingTimes example strict attendee confidence: 80 percent filter, same source.
48. Published official latency distribution count in cited docs: 0 p50/p95/p99 values.

Microsoft implications for Oyatie:
49. Oyatie should beat mailbox-scoped concurrency by sharding on tenant, calendar, and cell rather than using a four-request mailbox bottleneck.
50. Oyatie should support schedule slot widths from 5 minutes through full-day windows.
51. Oyatie should handle more than 1,000 busy entries by pagination, summarization, or explicit bounded errors rather than opaque failure.
52. Oyatie should expose scheduling confidence if it implements meeting-time suggestions.
53. Oyatie room/place targets should treat 3 calls per second as a lower external comparator, not an aspiration.

### §2.3 Cal.com

Source summary:
54. Cal.com API v2 docs disclose rate limits by authentication mode.
55. Cal.com API v2 docs disclose OAuth and API key authentication.
56. Cal.com booking docs disclose event type, team, organization, custom field, metadata, location, routing, and conflict-bypass fields.
57. Cal.com booking docs disclose metadata size limits.
58. Cal.com booking docs disclose response status and booking output shape.
59. Cal.com public docs do not disclose p50/p95/p99 latency distributions.
60. Cal.com public numbers are therefore used as booking API, metadata, and rate-limit comparators.

Cal.com benchmark numbers:
61. API key rate limit: 120 requests per minute, source `https://cal.com/docs/api-reference/v2/introduction`.
62. No-authentication default rate limit: 120 requests per minute, same source.
63. Reasonable support-increased limit example: 200 requests per minute, same source.
64. Higher support-increased limit example: 800 requests per minute, same source.
65. Managed-user access token validity: 60 minutes, same source.
66. Managed-user refresh token validity: 1 year, same source.
67. Booking metadata maximum keys: 50 keys, source `https://cal.com/docs/api-reference/v2/bookings/create-a-booking`.
68. Booking metadata key length maximum: 40 characters, same source.
69. Booking metadata string value length maximum: 500 characters, same source.
70. Create-booking success response code: 201, same source.
71. Out-of-bounds booking bypass support version date: 2026-02-25 API version, same source.
72. Published official latency distribution count in cited docs: 0 p50/p95/p99 values.

Cal.com implications for Oyatie:
73. Oyatie should exceed 120 requests per minute for paid scheduling use cases.
74. Oyatie demo-trial caps may deliberately sit closer to Cal.com's default API-key floor.
75. Oyatie paid and revenue-share deployments should exceed 800 booking-equivalent requests per minute when substrate is provisioned.
76. Oyatie should support booking metadata limits at least as expressive as Cal.com's 50-key model or explicitly bound metadata smaller for policy reasons.
77. Oyatie should add event-type and routing semantics if direct Cal.com displacement is expected.

## §3 Oyatie Target Numbers

Canonical latency target set:
1. Event fetch p50 target: 25 ms or lower.
2. Event fetch p95 target: 90 ms or lower.
3. Event fetch p99 target: 180 ms or lower.
4. Event fetch p999 target: 450 ms or lower.
5. Event write p50 target: 60 ms or lower.
6. Event write p95 target: 180 ms or lower.
7. Event write p99 target: 280 ms or lower.
8. Event write p999 target: 700 ms or lower.
9. Agenda render p50 target for seven-day window: 40 ms or lower.
10. Agenda render p95 target for seven-day window: 140 ms or lower.
11. Agenda render p99 target for seven-day window: 260 ms or lower.
12. Availability query p50 target for 10 attendees: 45 ms or lower.
13. Availability query p95 target for 10 attendees: 160 ms or lower.
14. Availability query p99 target for 10 attendees: 300 ms or lower.
15. Availability query p50 target for 50 attendees: 70 ms or lower.
16. Availability query p95 target for 50 attendees: 240 ms or lower.
17. Availability query p99 target for 50 attendees: 450 ms or lower.
18. Availability query p50 target for 100 attendees: 95 ms or lower.
19. Availability query p95 target for 100 attendees: 320 ms or lower.
20. Availability query p99 target for 100 attendees: 650 ms or lower.
21. Recurrence expansion p50 target for ordinary rules: 120 ms or lower.
22. Recurrence expansion p95 target for ordinary rules: 450 ms or lower.
23. Recurrence expansion p99 target for ordinary rules: 900 ms or lower.
24. Dense recurrence expansion p99 target with bounded output: 1,500 ms or lower.
25. Room conflict check p99 target: 90 ms or lower.
26. RSVP fanout p99 target for 100 attendees: 1,500 ms or lower.
27. Notification freshness p99 target: 60 seconds or lower.
28. Scheduling convergence p99 target: 30 seconds or lower.
29. Timezone database freshness bound: 24 hours or lower after upstream release approval.
30. CalDAV PROPFIND p99 target for 1,000-event collection page: 350 ms or lower.
31. `.ics` import target for 10,000 events: 55 seconds p99 or lower.
32. `.ics` export target for 10,000 events: 25 seconds p99 or lower.
33. Legal-hold mutation rejection target: 50 ms p99 or lower after policy cache warmup.
34. Cross-tenant private-field leakage target: 0 leaked private fields in conformance corpus.
35. Room double-book accepted-write target: 0 accepted conflicts under serializable conflict test.

Canonical throughput and scale target set:
36. Production cell event fetch throughput target: 50,000 requests per second.
37. Production cell event write throughput target: 10,000 requests per second.
38. Production cell availability lookup throughput target: 50,000 requests per second.
39. Production cell CalDAV concurrent session target: 100,000 sessions.
40. Production cell baseline calendar count: 100,000 active calendars.
41. Production cell max calendar count: 1,000,000 active calendars.
42. Production cell baseline daily event volume: 50,000,000 event reads.
43. Production cell write burst target: 100,000 writes over 60 seconds with shaped admission.
44. Freebusy explicit calendar cap target: 100 calendars where policy permits.
45. Freebusy group cap target: 100 principals where policy permits.
46. Event-list page target: 2,500 events per page.
47. Booking-equivalent API target for paid substrate: 10,000 requests per minute per tenant before negotiated scaling.
48. Booking-equivalent API target for revenue-share substrate: 10,000 requests per minute per tenant before negotiated scaling.
49. Demo-trial booking-equivalent cap target: 120 requests per minute per tenant.
50. Demo-trial freebusy cap target: 1,000 freebusy queries per day per tenant.
51. Demo-trial active calendar cap target: 2,000 calendars per tenant.
52. Demo-trial event write cap target: 10,000 writes per day per tenant.
53. Demo-trial `.ics` import cap target: 10,000 imported events per day per tenant.
54. Demo-trial CalDAV concurrent session cap target: 25 sessions per tenant.
55. Paid active calendar target: no fixed service cap below purchased capacity.
56. Revenue-share active calendar target: no fixed service cap below contract substrate envelope.

Deployment-context overlays:
57. `oyatie-public-cloud`: use canonical latency and throughput target set.
58. `oyatie-public-cloud`: elastic capacity should scale production cells horizontally before p95 exceeds target for 10 minutes.
59. `guest-on-aws`: use canonical target set when provisioned with the reference production cell.
60. `guest-on-aws`: smaller customer cells must publish an admission envelope before launch.
61. `guest-on-oci`: use canonical target set for paid OCI deployments with sufficient OCPU, memory, storage, and network.
62. `guest-on-oci` OCI Always Free profile: cap at 500 event fetch requests per second per compact cell.
63. `guest-on-oci` OCI Always Free profile: cap at 50 event writes per second per compact cell.
64. `guest-on-oci` OCI Always Free profile: cap at 100 availability lookups per second per compact cell.
65. `guest-on-oci` OCI Always Free profile: cap at 2,000 active calendars per compact cell.
66. `guest-on-oci` OCI Always Free profile: cap at 25 concurrent CalDAV sessions per compact cell.
67. `guest-on-oci` OCI Always Free profile: keep the same correctness and privacy targets under cap.
68. `on-prem`: use canonical target set when customer hardware meets the reference cell profile.
69. `on-prem`: publish facility-specific storage, network, and backup constraints before accepting paid production load.
70. `colo`: use canonical target set when network latency, storage IOPS, and hardware match the reference cell profile.
71. `colo`: publish facility-specific latency budgets for cross-region scheduling.
72. `oyatie-as-cloud-provider`: use canonical target set and Oyatie-owned admission control.
73. `oyatie-as-cloud-provider`: use the same public-cloud elasticity gates as `oyatie-public-cloud`.

Tenant_class overlays:
74. `demo_trial`: same latency SLO under cap, best-effort availability commitment.
75. `demo_trial`: hard cap admission rather than hidden quality degradation.
76. `demo_trial`: shape burst writes before storage or queue saturation.
77. `demo_trial`: prefer compact cells and OCI Always Free profile where feasible.
78. `demo_trial`: no compliance-pack expansion beyond the demo policy set.
79. `paid`: same latency SLO under purchased capacity.
80. `paid`: contractual availability and support commitment.
81. `paid`: compliance packs and BYOK allowed when canonical service dependencies support them.
82. `paid`: scale via additional cells, shards, and paid substrate.
83. `paid`: usage-based billing should reflect event writes, freebusy queries, CalDAV sessions, and import/export volume.
84. `revenue_share`: same latency SLO under contract substrate envelope.
85. `revenue_share`: substrate can run at cost or zero-margin while preserving correctness.
86. `revenue_share`: admission caps should be negotiated against gross-revenue model and expected customer traffic.
87. `revenue_share`: heavy booking or marketplace spikes should pre-reserve cell capacity.
88. `revenue_share`: privacy and audit guarantees do not change.

Validation targets:
89. Every latency target must be measured with warm-cache and cold-cache variants.
90. Every throughput target must be measured with a tenant-fairness check.
91. Every availability target must include private-field redaction assertions.
92. Every recurrence target must include recurrence-bomb defense.
93. Every room-conflict target must include simultaneous contender writes.
94. Every CalDAV target must include at least three independent client behaviors once the contract directory exists.
95. Every import/export target must include malformed and large file rejection.
96. Every deployment-context target must cite its OpenTofu module before being marked supported.
97. Every OS target must cite `supported-oses.json` before being marked supported.
98. Every tenant_class cap must be machine-readable before billing launch.

## §4 Comparison Narrative

Event read:
1. Google publishes quota and page-size numbers but not public latency distributions.
2. Microsoft publishes mailbox and concurrency limits but not public latency distributions.
3. Cal.com publishes request-rate limits but not public latency distributions.
4. Oyatie target of 25 ms p50 and 180 ms p99 is an industry-leader target, not a measured current result.
5. Current status: catch-up until source, tests, and measurement harness exist.

Event write:
6. Google project quota creates a 10,000-request-per-minute public comparator.
7. Microsoft mailbox limit creates a 10,000-request-per-10-minute per-mailbox comparator.
8. Cal.com default API-key limit creates a 120-request-per-minute booking comparator.
9. Oyatie production cell target of 10,000 writes per second is ahead of public API-limit comparators when properly provisioned.
10. Current status: target-ahead, evidence-catch-up.

Availability/freebusy:
11. Google freebusy public maxima are 100 group identifiers and 50 calendars.
12. Microsoft getSchedule supports users, distribution lists, rooms, and equipment and has a 1,000-entry oversize threshold.
13. Cal.com availability drives booking but public docs focus on booking flow rather than raw freebusy throughput.
14. Oyatie target supports 100 principals with privacy redaction and p99 under 650 ms.
15. Current status: parity-plus target because privacy semantics are stronger, but implementation proof is missing.

Recurrence:
16. Google and Microsoft have mature recurrence behavior but do not publish recurrence expansion latency.
17. Cal.com recurring booking support is not the same as full enterprise recurrence storage.
18. Oyatie target of ordinary recurrence p99 under 900 ms and dense bounded p99 under 1,500 ms is credible as an SLO target.
19. Current status: parity target, evidence-catch-up.

Room booking:
20. Microsoft is the strongest room/resource comparator.
21. Google Workspace resource calendars are a strong secondary comparator.
22. Cal.com location and routing support is adjacent.
23. Oyatie p99 room conflict check target of 90 ms and zero accepted double-book conflicts is an enterprise-grade target.
24. Current status: parity target, handoff-catch-up because room ownership needs closure.

CalDAV:
25. Google and Microsoft are not primarily CalDAV comparators in their current public API positioning.
26. Cal.com relies on connected calendars rather than a CalDAV-first public product surface.
27. Oyatie's CalDAV p99 target of 350 ms is a differentiator if the missing contract directory is added.
28. Current status: catch-up because the OpenAPI references a missing CalDAV contract directory.

Import/export:
29. Oyatie targets `.ics` import of 10,000 events in 55 seconds p99.
30. Oyatie targets `.ics` export of 10,000 events in 25 seconds p99.
31. Public counterpart throughput numbers for this exact workload are not disclosed in the cited docs.
32. Current status: target-defined, evidence-catch-up.

Scale ceiling:
33. Oyatie capacity docs target 100,000 active calendars baseline and 1,000,000 max per production cell.
34. Google and Microsoft public SaaS limits are not directly comparable because their internal cell shapes are not public.
35. Cal.com public API limits are lower for default API usage.
36. Current status: target-ahead for self-owned substrate, evidence-catch-up until measured.

Deployment overlays:
37. Public-cloud and Oyatie-as-provider contexts should meet the full target set after OpenTofu exists.
38. Guest AWS and paid guest OCI should meet the full target set when reference substrate exists.
39. OCI Always Free profile intentionally caps throughput but preserves correctness under cap.
40. On-prem and colo require facility-specific envelopes.
41. Current status: catch-up because the calendar path has no canonical OpenTofu modules.

Tenant_class overlays:
42. `demo_trial` is constrained by cap and best-effort availability, not lower feature quality.
43. `paid` is constrained by purchased capacity and contract.
44. `revenue_share` is constrained by at-cost or zero-margin substrate economics and negotiated traffic shape.
45. Current status: catch-up because calendar has no tenant_class semantics in its artifacts.

Final benchmark verdict:
46. The Oyatie target set is appropriately ambitious.
47. The target set meets or exceeds the disclosed public limits that are comparable.
48. The target set cannot be claimed as achieved because implementation, tests, OpenTofu modules, supported-OS manifest, and benchmark harness evidence are absent from the calendar path.
49. The next benchmark milestone is to turn this document into executable load profiles after source and deployment modules exist.
50. The next documentation milestone is to retire the older benchmark document or rewrite it around this single target model.
