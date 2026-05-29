# Workplace Integration Performance Benchmark Numbers Audit - 2026-05-20
1. Microservice: `workplace-integration`.
2. Deliverable: Wave 3 Batch 3.2 performance benchmark numbers.
3. Counterpart bar: Slack App Directory / Microsoft Teams App Store / Zapier Integrations.
4. Output rule: this document uses one industry-leader target set plus deployment-context and tenant_class overlays.
5. Output rule: this document does not define feature strata or commercial quality bands.
6. Current artifact anchor: `README.md:16` frames the service as workplace agreement, e-sign, roster, and regulated workforce integration substrate.
7. Current artifact anchor: `PRD.md:65-73` gives coarse p95 and p99 nonfunctional targets but not counterpart-specific benchmark evidence.
8. Current artifact anchor: `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:1-5` covers HRIS/e-sign vendors, not the assigned Slack/Teams/Zapier family.
9. Current artifact anchor: `contracts/openapi-v1.yaml:21-189` defines seven workplace mutation routes.
10. Current artifact anchor: `contracts/asyncapi-v1.yaml:18-52` defines seven workplace event channels.
11. Current artifact anchor: `manifest.json:47-58` lists dependencies but does not define marketplace, channel-adapter, workflow, billing, or tenant-class ownership.
12. Current artifact anchor: `specs/master-plan-sequencing.json:704-745` defines six deployable contexts.
13. Current artifact anchor: `specs/master-plan-sequencing.json:857-868` defines the OCI Always Free profile expectation.
14. Current artifact anchor: chat line 16311 assigns Slack App Directory, Microsoft Teams App Store, and Zapier Integrations as the target counterpart set.
15. Methodology disclosure: public counterpart documents expose limits, throttles, payload budgets, install surface, and ecosystem scale more often than full vendor latency tests.
16. Methodology disclosure: values labeled `source` are direct public numbers from counterpart documentation.
17. Methodology disclosure: values labeled `derived` are computed from a source number, such as events per hour divided by seconds.
18. Methodology disclosure: values labeled `target` are Oyatie engineering targets and not measured production results.
19. Methodology disclosure: values labeled `current artifact state` describe the present service documentation or contract state.
20. Methodology disclosure: no current load-test harness was found in `tests/`, because no `tests/` directory exists in the service inventory.

## §1 Methodology
1. Benchmark dimension: app-directory ecosystem scale, because Slack and Teams expose marketplace reach as part of product credibility.
2. Benchmark dimension: integration-catalog breadth, because Zapier's value is trigger/action/search coverage across many apps.
3. Benchmark dimension: workplace action API latency, because the PRD promises synchronous user-facing operations at `PRD.md:65-73`.
4. Benchmark dimension: event acknowledgement latency, because Slack requires event responses within three seconds and Zapier REST Hooks require dependable webhook handling.
5. Benchmark dimension: trigger freshness, because Zapier polling can check endpoints every one to fifteen minutes.
6. Benchmark dimension: message or event throughput, because Slack, Teams, and Zapier each publish throttling boundaries.
7. Benchmark dimension: payload size, because Zapier publishes concrete webhook, action, trigger, and file budgets.
8. Benchmark dimension: concurrent collaboration scale, because Teams publishes team, channel, member, and meeting ceilings.
9. Benchmark dimension: install and app governance scale, because Teams admin controls and Slack marketplace states affect enterprise readiness.
10. Benchmark dimension: retry and replay durability, because Slack retries failed event deliveries and Zapier supports replay mechanics.
11. Test workload W1: create e-sign session through `/workplace/esign/sessions`, anchored at `contracts/openapi-v1.yaml:21-31`.
12. Test workload W2: capture signature proof through `/workplace/esign/sessions/{session_id}/sign`, anchored at `contracts/openapi-v1.yaml:45-55`.
13. Test workload W3: generate offer letter through `/workplace/offer-letters`, anchored at `contracts/openapi-v1.yaml:69-79`.
14. Test workload W4: bind engagement agreement through `/workplace/engagement-agreements`, anchored at `contracts/openapi-v1.yaml:93-103`.
15. Test workload W5: bind roster through `/workplace/roster-bindings`, anchored at `contracts/openapi-v1.yaml:117-127`.
16. Test workload W6: record clock event through `/workplace/clock-events`, anchored at `contracts/openapi-v1.yaml:141-151`.
17. Test workload W7: record DLP trace through `/workplace/dlp-traces`, anchored at `contracts/openapi-v1.yaml:165-175`.
18. Test workload W8: publish `WorkplaceSignatureCaptured` as a near-real-time integration trigger, anchored at `contracts/asyncapi-v1.yaml:23-27`.
19. Test workload W9: publish `WorkplaceClockEventAttested` as a workflow and payroll-evidence trigger, anchored at `contracts/asyncapi-v1.yaml:43-47`.
20. Test workload W10: publish `WorkplaceDlpTraceSealed` as audit evidence, anchored at `contracts/asyncapi-v1.yaml:48-52`.
21. Test workload W11: install or enable a workplace integration app package for a tenant; current artifact state has no app package contract.
22. Test workload W12: subscribe an external REST Hook to workplace events; current artifact state has no subscribe or unsubscribe endpoint.
23. Test workload W13: poll for new workplace events; current artifact state has no polling cursor contract.
24. Test workload W14: run channel notification to Slack or Teams; current artifact state has no Slack app manifest or Teams package definition.
25. Test workload W15: export integration certification evidence; current artifact state has compliance docs but no app-store evidence package.
26. OS disclosure: canonical OS support comes from `specs/master-plan-sequencing.json:777-815`.
27. OS disclosure: benchmark harness should run Linux x86_64, Linux arm64, macOS Apple Silicon developer, Windows desktop-client integration, and the remaining canonical server/client OS set.
28. OS disclosure: this report defines required numbers, not observed cross-OS numbers, because no service-local harness exists.
29. Architecture disclosure: backend benchmark implementation must be Rust according to `specs/master-plan-sequencing.json:817-856`.
30. Architecture disclosure: generated non-Rust clients require generated-client provenance, because `contracts/workplace-integration-v1.proto:5-7` contains Java/Go package options.
31. Deployment context: `oyatie-public-cloud` should be elastic and multi-cell.
32. Deployment context: `guest-on-aws` should meet the same targets when tenant-provisioned capacity matches the reference shape.
33. Deployment context: `guest-on-oci` should include both paid-capacity and OCI Always Free profile overlays.
34. Deployment context: `on-prem` should meet targets only when the facility provides reference CPU, memory, disk, and network.
35. Deployment context: `colo` should meet targets only when the rack, uplink, and storage profile match the reference model.
36. Deployment context: `oyatie-as-cloud-provider` should meet public-cloud targets on Oyatie-owned substrate.
37. Tenant-class disclosure: `demo_trial` means free profile, usage caps, best-effort SLO, no compliance packs, and no BYOK.
38. Tenant-class disclosure: `paid` means per-seat license plus usage-based billing, contractual SLO, compliance packs allowed, and BYOK allowed.
39. Tenant-class disclosure: `revenue_share` means Oyatie takes a share of customer gross revenue with at-cost or zero-margin substrate.
40. Quality disclosure: all tenant classes share the same correctness, security, and product-quality bar.
41. Constraint disclosure: `demo_trial` caps constrain volume and retention, not feature correctness.
42. Constraint disclosure: `paid` scale is limited by contracted capacity, measured usage, and deployment context.
43. Constraint disclosure: `revenue_share` scale is limited by contract economics and substrate cost envelope.
44. Evidence disclosure: every current-state gap in this report cites service-local files or canonical direction.
45. Evidence disclosure: counterpart numbers cite public Slack, Microsoft, and Zapier documentation.
46. Measurement disclosure: this audit did not execute load tests.
47. Measurement disclosure: target numbers must be validated later with Rust benchmarks and production-like OpenTofu deployments.
48. Measurement disclosure: current SLO files cannot be trusted for route-specific numbers because metric queries are shifted, as cited in the coherence audit findings.
49. Scoring term: `ahead` means target exceeds the most demanding published counterpart limit or offers lower latency.
50. Scoring term: `parity` means target matches the relevant published limit or behavior.
51. Scoring term: `catch-up` means target is directionally correct but current artifacts lack required implementation surface.
52. Scoring term: `blocked-currently` means current artifacts contradict or omit the measurement path.

## §2 Counterpart numbers
1. Slack number S1: Slack docs say users can browse more than 2,000 apps; source: `https://slack.com/help/articles/360001537467-Guide-to-apps-in-Slack` lines 41-60.
2. Slack number S2: Slack app surfaces include App Home, Messages tabs, shortcuts, slash commands, messages, app DMs, and assistants; source: same Slack help page lines 67-86.
3. Slack number S3: distributed Slack apps use OAuth 2.0 and SSL request URLs; source: `https://docs.slack.dev/app-management/distribution/` lines 64-105.
4. Slack number S4: Slack manifests are JSON or YAML app configuration and support version control; source: `https://docs.slack.dev/app-manifests/` lines 61-79.
5. Slack number S5: Slack Web API low-rate class permits 1+ request per minute; source: `https://docs.slack.dev/apis/web-api/rate-limits` lines 78-92.
6. Slack number S6: Slack Web API second class permits 20+ requests per minute; source: Slack Web API rate-limit page lines 78-92.
7. Slack number S7: Slack Web API third class permits 50+ requests per minute; source: Slack Web API rate-limit page lines 78-92.
8. Slack number S8: Slack Web API fourth class permits 100+ requests per minute; source: Slack Web API rate-limit page lines 78-92.
9. Slack number S9: Slack Events API permits 30,000 event deliveries per workspace-team-app per 60 minutes; source: Slack rate-limit page lines 98-114 and Events API page lines 475-487.
10. Slack number S10: Slack Events API source limit derives to 500 events per minute and about 8.33 events per second per workspace-app.
11. Slack number S11: Slack event receivers must respond with HTTP 2xx within three seconds; source: `https://docs.slack.dev/apis/events-api/` lines 503-512.
12. Slack number S12: Slack retries failed event delivery three times, near immediate, one minute, and five minutes; source: Slack Events API page lines 695-704.
13. Slack number S13: Slack incoming webhooks permit one request per second; source: Slack Web API rate-limit page lines 98-114.
14. Slack number S14: Slack `chat.postMessage` allows one message per second per channel; source: Slack Web API rate-limit page lines 196-204.
15. Slack number S15: Slack profile updates permit ten updates per minute per user and thirty users per minute per token; source: Slack Web API rate-limit page lines 211-214.
16. Microsoft Teams number M1: Microsoft publishing page cites 145 million daily active users; source: `https://developer.microsoft.com/en-us/microsoft-teams/app-publishing` lines 84-87.
17. Microsoft Teams number M2: Microsoft publishing page says apps can reach millions of users and references thousands of apps; source: Teams publishing page lines 110-132.
18. Microsoft Teams number M3: Teams app capabilities include tabs, webhooks/connectors, messaging extensions, meeting extensions, bots, cards, task modules, and activity feeds; source: `https://learn.microsoft.com/pl-pl/microsoftTeams/apps-in-teams` lines 84-119.
19. Microsoft Teams number M4: Teams admin app details include certification, compliance, support, update, security, permissions, pricing, and setup metadata; source: `https://learn.microsoft.com/en-us/microsoftteams/manage-apps` lines 112-132.
20. Microsoft Teams number M5: a team can have 25,000 members; source: `https://learn.microsoft.com/en-us/microsoftteams/limits-specifications-teams` lines 61-86.
21. Microsoft Teams number M6: a team can have 100 owners; source: Teams limits page lines 61-86.
22. Microsoft Teams number M7: a team can have 30 private channels; source: Teams limits page lines 61-86.
23. Microsoft Teams number M8: a private channel can have 250 members; source: Teams limits page lines 61-86.
24. Microsoft Teams number M9: a team can have 30 shared channels; source: Teams limits page lines 61-86.
25. Microsoft Teams number M10: a shared channel can be shared with 50 teams; source: Teams limits page lines 61-86.
26. Microsoft Teams number M11: a shared channel can have 1,000 direct members; source: Teams limits page lines 61-86.
27. Microsoft Teams number M12: a team can have 1,000 channels including deleted channels; source: Teams limits page lines 97-101.
28. Microsoft Teams number M13: channel message post size is 100 KB and sent messages per user are capped at 10,000 per 24 hours; source: Teams limits page lines 116-128.
29. Microsoft Teams number M14: bot messages are capped at 50 requests per second per app per tenant and 50 requests per second per app across all tenants; source: `https://learn.microsoft.com/en-us/microsoftteams/platform/bots/how-to/rate-limit` lines 100-123.
30. Microsoft Teams number M15: bot thread limits include seven QPS per bot per thread and fourteen QPS for all bots per thread; source: Teams bot rate-limit page lines 65-77.
31. Zapier number Z1: Zapier docs define Zaps using authentication, triggers, and actions against publicly accessible APIs; source: `https://docs.zapier.com/integrations/quickstart/how-zapier-works` lines 130-147.
32. Zapier number Z2: Zapier recommends foundational triggers, actions, searches, and search-or-create patterns; source: `https://docs.zapier.com/integrations/quickstart/recommended-triggers-and-actions` lines 162-178.
33. Zapier number Z3: Zapier polling triggers check an API endpoint every one to fifteen minutes; source: `https://docs.zapier.com/integrations/build/trigger` lines 143-154.
34. Zapier number Z4: Zapier REST Hook triggers use subscribe and unsubscribe performs; source: Zapier trigger page lines 239-252.
35. Zapier number Z5: Zapier REST Hook subscribe performs receive a `target_url`; source: Zapier trigger page lines 274-287.
36. Zapier number Z6: Zapier webhook response includes `X-Hook-Secret`; source: Zapier trigger page lines 337-347.
37. Zapier number Z7: Zap runs and app extensions have a 30 second timeout; source: `https://docs.zapier.com/platform/reference/throttling` lines 145-158.
38. Zapier number Z8: webhook trigger payload budget is 10 MB; source: Zapier throttling page lines 162-189.
39. Zapier number Z9: action or trigger input budget is 35 MB for platform version 17.2.0 and later; source: Zapier throttling page lines 162-189.
40. Zapier number Z10: action response budget is 20 MB; source: Zapier throttling page lines 162-189.
41. Zapier number Z11: total file budget is 150 MB and file download budget is 120 MB; source: Zapier throttling page lines 162-189.
42. Zapier number Z12: private apps on Free and Pro plans are limited to 100 requests per 60 seconds; source: Zapier throttling page lines 196-207.
43. Zapier number Z13: private apps on Team and Enterprise plans are limited to 5,000 requests per 60 seconds; source: Zapier throttling page lines 196-207.
44. Zapier number Z14: Zapier global private-app limit is 15,000 requests per 60 seconds; source: Zapier throttling page lines 196-207.
45. Zapier number Z15: polling triggers return up to 100 new items per poll; source: Zapier throttling page lines 211-240.
46. Zapier number Z16: REST Hook triggers allow 10,000 requests per five minutes per Zap and 30 requests per second per Zap; source: Zapier throttling page lines 211-240.
47. Zapier number Z17: instant triggers allow 20,000 requests per five minutes per user and 30 requests per second per app-user pair; source: Zapier throttling page lines 211-240.
48. Zapier number Z18: Free and trial plan polling is limited to 200 requests per ten minutes per Zap; source: Zapier throttling page lines 248-251.
49. Zapier number Z19: replay webhooks run at one request per second and at most 100 replay requests; source: Zapier throttling page lines 257-265.

## §3 Oyatie target numbers - single industry-leader target set
1. Target T1: synchronous create e-sign session p50 <= 80 ms, p95 <= 250 ms, p99 <= 750 ms; type: target.
2. Target T2: synchronous signature proof capture p50 <= 100 ms, p95 <= 300 ms, p99 <= 900 ms; type: target.
3. Target T3: offer-letter create request accepted p50 <= 80 ms, p95 <= 250 ms, p99 <= 750 ms; heavy rendering may continue asynchronously.
4. Target T4: engagement-agreement bind p50 <= 100 ms, p95 <= 300 ms, p99 <= 900 ms.
5. Target T5: roster binding p50 <= 75 ms, p95 <= 200 ms, p99 <= 600 ms.
6. Target T6: clock event attestation p50 <= 90 ms, p95 <= 250 ms, p99 <= 750 ms.
7. Target T7: DLP trace seal p50 <= 120 ms, p95 <= 350 ms, p99 <= 1,000 ms.
8. Target T8: evidence lookup by audit-chain reference p50 <= 100 ms, p95 <= 300 ms, p99 <= 900 ms.
9. Target T9: external webhook acknowledgement p95 <= 500 ms and p99 <= 2,000 ms.
10. Target T10: external webhook acknowledgement hard ceiling <= 3,000 ms to satisfy Slack event receiver behavior.
11. Target T11: event publication from committed route mutation to internal bus p95 <= 250 ms and p99 <= 1,000 ms.
12. Target T12: Slack adapter sustained ingress target >= 30,000 events per workspace-app per 60 minutes, matching Slack's published event envelope.
13. Target T13: Slack adapter burst target absorbs 3 retry attempts without duplicate side effects.
14. Target T14: Slack channel message adapter respects one message per second per channel and queues excess without losing audit evidence.
15. Target T15: Teams bot adapter sustained target >= 50 requests per second per app per tenant when Microsoft allows that rate.
16. Target T16: Teams thread adapter respects seven QPS per bot per thread and fourteen QPS all bots per thread.
17. Target T17: Zapier REST Hook adapter sustained target >= 30 requests per second per Zap and >= 10,000 requests per five minutes per Zap.
18. Target T18: Zapier instant trigger adapter target >= 30 requests per second per app-user pair and >= 20,000 requests per five minutes per user.
19. Target T19: Zapier polling endpoint returns up to 100 new items per poll with stable cursor ordering.
20. Target T20: Zapier polling freshness supports one-minute poll plans without duplicate or missing workplace events.
21. Target T21: webhook payload body target <= 1 MB for normal workplace events to stay far below Zapier's 10 MB webhook ceiling.
22. Target T22: signed-document metadata payload target <= 256 KB, with file bodies stored by drive or document storage handoff.
23. Target T23: action response body target <= 2 MB, far below Zapier's 20 MB action response ceiling.
24. Target T24: file handoff target supports 120 MB external file downloads only through storage services, not route response bodies.
25. Target T25: idempotency key replay p95 <= 50 ms for duplicate detection on all mutation routes.
26. Target T26: Cedar authorization decision p95 <= 20 ms under warm cache and p99 <= 100 ms under cache miss.
27. Target T27: tenant-class cap check p95 <= 20 ms and p99 <= 100 ms.
28. Target T28: usage meter emission p95 <= 100 ms after committed mutation.
29. Target T29: audit-chain reference seal p95 <= 250 ms and p99 <= 1,000 ms.
30. Target T30: end-to-end route-to-event conformance mismatch count = 0 for OpenAPI, AsyncAPI, Cedar, SLO, and runbook tables.
31. Target T31: SLO metric-name mismatch count = 0 after remediation.
32. Target T32: duplicate external webhook side effects = 0 under Slack retry, Teams retry, and Zapier replay tests.
33. Target T33: app install enablement p95 <= 2 seconds for already-approved tenant app configurations.
34. Target T34: OAuth token rotation p95 <= 1 second for tenant-scoped app credentials.
35. Target T35: app uninstall and token revocation p95 <= 2 seconds for policy state propagation.
36. Target T36: app listing metadata generation p95 <= 1 second from validated service manifest and compliance evidence.
37. Target T37: certification evidence export p95 <= 5 seconds for normal evidence bundle and p99 <= 30 seconds for large tenant history.
38. Target T38: workflow trigger fanout p95 <= 500 ms to workflow-engine after local event commit.
39. Target T39: channel notification enqueue p95 <= 250 ms to messenger/mail/calendar/meet handoff.
40. Target T40: cross-service handoff timeout budget <= 2 seconds for synchronous path and async fallback for longer operations.
41. Target T41: per-tenant sustained workplace action throughput in public cloud >= 1,000 mutation requests per second when paid capacity is provisioned.
42. Target T42: per-cell public-cloud aggregate workplace action throughput >= 10,000 mutation requests per second.
43. Target T43: per-cell public-cloud event publication throughput >= 100,000 workplace events per minute.
44. Target T44: public-cloud active signing session ceiling >= 100,000 concurrent sessions per cell.
45. Target T45: public-cloud active clock-event ingest ceiling >= 50,000 workers per minute per cell.
46. Target T46: public-cloud DLP trace seal ceiling >= 20,000 traces per minute per cell.
47. Target T47: public-cloud availability target for paid and revenue_share production tenants >= 99.95 percent monthly for this service route family after SLO rewiring.
48. Target T48: demo_trial availability language remains best-effort while correctness and security remain identical.
49. Target T49: benchmark harness records p50, p95, p99, max, error count, duplicate count, and throttle count for every workload.
50. Target T50: benchmark harness records context, OS, architecture, tenant_class, worker count, data volume, and payload size for every run.

## §3.1 Deployment-context overlays
1. `oyatie-public-cloud` overlay: T1 through T50 are expected with elastic capacity and multi-cell routing.
2. `oyatie-public-cloud` overlay: default paid/revenue_share reference cell target is 10,000 mutation rps and 100,000 events per minute.
3. `oyatie-public-cloud` overlay: public ingress must queue or shed external adapter traffic without dropping committed local audit evidence.
4. `guest-on-aws` overlay: targets are expected when the tenant supplies equivalent compute, storage, queue, and network resources.
5. `guest-on-aws` overlay: if tenant infrastructure is undersized, published SLO must name the tenant-provided bottleneck, not lower the product-quality bar.
6. `guest-on-oci` overlay: paid-capacity OCI deployments should match the same reference shape as other guest-cloud contexts.
7. `guest-on-oci` OCI Always Free profile overlay: use four Arm OCPU and twenty-four GB memory as the ceiling described by canonical OCI guidance.
8. `guest-on-oci` OCI Always Free profile overlay: cap aggregate workplace mutation throughput at 250 rps until measured otherwise.
9. `guest-on-oci` OCI Always Free profile overlay: cap event publication at 15,000 events per minute until measured otherwise.
10. `guest-on-oci` OCI Always Free profile overlay: cap active signing sessions at 1,000 concurrent sessions.
11. `guest-on-oci` OCI Always Free profile overlay: cap DLP trace seal volume at 2,000 traces per minute.
12. `guest-on-oci` OCI Always Free profile overlay: enforce demo_trial usage caps before infrastructure saturation.
13. `guest-on-oci` OCI Always Free profile overlay: no compliance pack or BYOK claim is made for demo_trial tenants.
14. `on-prem` overlay: targets apply when customer facility meets reference CPU, memory, storage IOPS, network latency, and clock synchronization.
15. `on-prem` overlay: app-directory outbound integrations require customer firewall and egress allowlist readiness.
16. `on-prem` overlay: measured SLO report must separate Oyatie service latency from facility network latency.
17. `colo` overlay: targets apply when rack placement, uplink, storage, and time sync match the reference profile.
18. `colo` overlay: cross-region webhook fanout must account for facility egress and peering constraints.
19. `oyatie-as-cloud-provider` overlay: targets match public cloud on Oyatie-operated substrate.
20. `oyatie-as-cloud-provider` overlay: capacity planning must include marketplace, workflow-engine, messenger, mail, drive, audit-chain, payments, and tenancy dependencies.
21. All contexts overlay: OpenTofu modules must exist under canonical context paths before benchmark claims are production-credible.
22. All contexts overlay: current service state lacks those modules, because only a flat `iac/` root was found.
23. All contexts overlay: current `iac/terraform-main.tf:6` and `iac/terraform-variables.tf:6` use `null_resource`, so they cannot be used as benchmark substrate evidence.
24. All contexts overlay: current service state lacks `supported-oses.json`, so OS-specific performance remains unmeasured.
25. All contexts overlay: SLO numbers must be regenerated after route-to-metric rewiring.

## §3.2 Tenant-class overlays
1. `demo_trial` overlay: same correctness, security, audit, and UX quality as paid use.
2. `demo_trial` overlay: cap active tenant users at 10 unless a later billing policy sets a different cap.
3. `demo_trial` overlay: cap workplace mutation requests at 1,000 per tenant per day.
4. `demo_trial` overlay: cap e-sign sessions at 100 per tenant per day.
5. `demo_trial` overlay: cap clock events at 2,000 per tenant per day.
6. `demo_trial` overlay: cap DLP trace seals at 500 per tenant per day.
7. `demo_trial` overlay: cap external integration subscriptions at 5 active subscriptions.
8. `demo_trial` overlay: cap REST Hook deliveries at 10,000 per tenant per day.
9. `demo_trial` overlay: cap retention to the demo retention window defined by tenancy and data-retention policy.
10. `demo_trial` overlay: reject compliance pack and BYOK enablement requests with explicit policy responses.
11. `demo_trial` overlay: use best-effort SLO language while preserving no-data-loss behavior for committed events.
12. `paid` overlay: no artificial feature cap; scale follows purchased seats, usage budget, and deployment capacity.
13. `paid` overlay: contractual SLO applies after service-local SLO metric rewiring and production validation.
14. `paid` overlay: compliance packs are allowed when compliance, DPIA, and evidence exports pass review.
15. `paid` overlay: BYOK is allowed when key-management handoff and data-flow docs are complete.
16. `paid` overlay: per-seat and usage-based billing events must be emitted for every billable route and integration delivery.
17. `revenue_share` overlay: same quality target as paid production use.
18. `revenue_share` overlay: substrate should run at cost or zero-margin according to contract economics.
19. `revenue_share` overlay: gross-revenue share settlement requires usage and revenue events routed to payments/cloud-billing.
20. `revenue_share` overlay: marketplace seller, B2C operator, embedded SaaS reseller, and affiliate-partner scenarios need separate metering labels.
21. All tenant classes overlay: tenant_class must be represented in manifest or an adjacent service-local policy file.
22. All tenant classes overlay: current service state has no `tenant_class`, `demo_trial`, `paid`, or `revenue_share` search hits.
23. All tenant classes overlay: current docs that use retired commercial quality labels should be rewritten to tenant_class plus policy profile language.
24. All tenant classes overlay: tests must prove cap-hit behavior and scaling behavior without lowering correctness.
25. All tenant classes overlay: benchmark reports must include tenant_class dimension in every result row.

## §4 Comparison narrative
1. Slack ecosystem scale comparison: current workplace-integration is catch-up because no Slack app manifest or listing metadata exists.
2. Slack ecosystem scale target: parity requires app manifest generation, app directory metadata, OAuth scope model, and app Home/shortcut/command surfaces.
3. Slack event limit comparison: Oyatie target T12 matches Slack's 30,000 events per workspace-app per hour envelope.
4. Slack event acknowledgement comparison: Oyatie target T9 and T10 are ahead of the three-second hard behavior because p95 is 500 ms and p99 is 2,000 ms.
5. Slack retry comparison: Oyatie target T13 is parity only after duplicate side-effect tests exist.
6. Slack message throttling comparison: Oyatie target T14 is parity because it respects one message per second per channel.
7. Slack Web API class comparison: Oyatie must not assume more than each Slack method's class; adapter design is catch-up until method-specific budgets are documented.
8. Slack app governance comparison: current compliance docs are incomplete for Slack marketplace evidence, so the current state is blocked-currently.
9. Teams ecosystem scale comparison: current service is catch-up because no Teams app package, admin policy metadata, or task module/bot surface exists.
10. Teams tenant app governance comparison: current service is blocked-currently because certification, permissions, pricing, setup, and security metadata are not represented.
11. Teams collaboration scale comparison: Oyatie does not need to beat 25,000 members per team directly, but roster and channel mapping must tolerate that ceiling.
12. Teams channel scale comparison: Oyatie must design for 1,000 channels per team if roster bindings can target channel permissions.
13. Teams message size comparison: Oyatie target T21 and T22 stay below the 100 KB Teams channel message post size when message bodies are metadata-only.
14. Teams bot throughput comparison: Oyatie target T15 matches the 50 rps app-tenant rate when Microsoft permits that capacity.
15. Teams thread throughput comparison: Oyatie target T16 matches the thread-specific QPS limits by queueing per thread.
16. Teams meeting extension comparison: current service has no meeting extension surface, so the relevant state is catch-up if meetings remain in scope from chat line 1444.
17. Zapier app model comparison: current route/event vocabulary is partial because OpenAPI and AsyncAPI can map to actions and triggers.
18. Zapier trigger/action/search comparison: current state is catch-up because no explicit Zapier catalog or search-or-create semantics exist.
19. Zapier polling comparison: target T19 and T20 meet one-minute freshness and 100-new-item poll budgets, but current state lacks cursor endpoints.
20. Zapier REST Hook comparison: target T17 matches the 30 requests per second and 10,000 per five minutes per Zap budgets.
21. Zapier instant trigger comparison: target T18 matches the 30 requests per second app-user and 20,000 per five minutes user budgets.
22. Zapier timeout comparison: all synchronous route p99 targets are below 30 seconds, so target state is ahead of Zapier run timeout.
23. Zapier payload comparison: target T21 through T24 stay within webhook, action, and file limits by keeping bodies metadata-heavy.
24. Zapier replay comparison: target T32 is parity only after replay tests prove zero duplicate side effects.
25. Zapier hook-secret comparison: current security docs need explicit `X-Hook-Secret` verification and rotation to reach parity.
26. Industry-leader target comparison: the single target set is intentionally stricter than current `PRD.md:65-73` by defining route-level p50/p95/p99 and adapter limits.
27. Current contract comparison: current OpenAPI audit-event repetition blocks route-level benchmark credibility until corrected.
28. Current SLO comparison: shifted SLO queries block any claim that present SLOs validate target T1 through T8.
29. Current IaC comparison: absent context modules and forbidden `null_resource` scaffolds block production benchmark claims.
30. Current OS comparison: absent supported-OS manifest blocks cross-OS benchmark claims.
31. Current tenant-class comparison: absent tenant_class semantics block demo, paid, and revenue-share benchmark reporting.
32. Current counterpart comparison: existing HRIS benchmark docs block nothing by themselves, but they cannot be used as evidence for Slack/Teams/Zapier parity.
33. Remediation priority 1: fix route-event and SLO metric conformance before publishing any measured numbers.
34. Remediation priority 2: add app/integration catalog model for Slack, Teams, and Zapier surfaces.
35. Remediation priority 3: add OpenTofu context modules and OCI Always Free profile module.
36. Remediation priority 4: add supported OS manifest and Rust benchmark harness.
37. Remediation priority 5: add tenant_class policy, caps, usage events, and billing handoff.
38. Remediation priority 6: run benchmarks per context, OS, architecture, tenant_class, and workload.
39. Final benchmark verdict: target numbers are industry-leader-grade, but current artifacts are not yet benchmark-claim-ready.
40. Final benchmark verdict: no fourth deliverable is required, and this report uses no retired commercial quality scaffold.

## §5 Benchmark validation acceptance checks
1. Acceptance check A1: every benchmark run records the exact git commit and generated contract hash.
2. Acceptance check A2: every benchmark run records deployment context as one of the six canonical contexts.
3. Acceptance check A3: every benchmark run records tenant_class as `demo_trial`, `paid`, or `revenue_share`.
4. Acceptance check A4: every benchmark run records OS name, OS version, architecture, kernel, container runtime, and database mode.
5. Acceptance check A5: every benchmark run records workload W1 through W15 separately.
6. Acceptance check A6: every route benchmark records p50, p95, p99, max, error rate, timeout count, and retry count.
7. Acceptance check A7: every event benchmark records enqueue latency, publish latency, delivery latency, duplicate count, and replay count.
8. Acceptance check A8: every external-adapter benchmark records counterpart throttle responses and local queue depth.
9. Acceptance check A9: Slack adapter tests prove acknowledgement below the three-second ceiling under normal and retry load.
10. Acceptance check A10: Slack adapter tests prove per-channel `chat.postMessage` queueing honors one message per second per channel.
11. Acceptance check A11: Slack adapter tests prove 30,000 events per workspace-app-hour can be absorbed without duplicate side effects.
12. Acceptance check A12: Teams adapter tests prove app-tenant throughput up to 50 requests per second when the upstream allows it.
13. Acceptance check A13: Teams adapter tests prove thread queueing respects seven QPS per bot per thread.
14. Acceptance check A14: Zapier polling tests prove stable newest-first ordering and a maximum of 100 new items per poll.
15. Acceptance check A15: Zapier REST Hook tests prove subscribe, unsubscribe, target URL handshake, and hook-secret verification.
16. Acceptance check A16: Zapier replay tests prove one request per second replay can recover without duplicate workplace actions.
17. Acceptance check A17: payload tests prove normal workplace event bodies stay below 1 MB.
18. Acceptance check A18: file handoff tests prove large documents are delegated to storage rather than embedded in action responses.
19. Acceptance check A19: tenant cap tests prove `demo_trial` caps fail closed with explicit policy responses.
20. Acceptance check A20: paid capacity tests prove scale increases through purchased capacity, not by changing quality rules.
21. Acceptance check A21: revenue-share tests prove usage and revenue events are emitted for settlement without changing route correctness.
22. Acceptance check A22: OCI Always Free profile tests prove cap enforcement before the four-OCPU resource envelope saturates.
23. Acceptance check A23: public-cloud tests prove multi-cell routing can sustain the per-cell event and mutation targets.
24. Acceptance check A24: guest-cloud tests prove tenant-provided resource constraints are reported as substrate constraints.
25. Acceptance check A25: on-prem and colo tests separate service latency from facility network latency.
26. Acceptance check A26: app install tests prove install, uninstall, token rotation, and token revocation budgets.
27. Acceptance check A27: certification-export tests prove compliance, DPIA, security, and threat-model evidence can be assembled within target latency.
28. Acceptance check A28: cross-service handoff tests prove workflow-engine, marketplace, messenger, mail, drive, audit-chain, payments, and tenancy boundaries.
29. Acceptance check A29: route-event conformance tests prove OpenAPI `x-audit-event` values match AsyncAPI channels.
30. Acceptance check A30: SLO validation tests prove SLO queries measure the named operation.
31. Acceptance check A31: OS matrix tests prove benchmark harness runs on every supported service OS profile.
32. Acceptance check A32: OpenTofu validation proves context modules are real infrastructure definitions, not `null_resource` stand-ins.
33. Acceptance check A33: generated-client provenance checks prove non-Rust SDK metadata is generated output, not backend runtime code.
34. Acceptance check A34: no benchmark result may be published as current production evidence until A1 through A33 pass for the relevant context.
35. Acceptance check A35: until those checks pass, this document remains a target-number audit and not a measured production benchmark report.
