# consent-graph performance benchmark numbers — 2026-05-20

Audit owner: Wave 3 Batch 3.2 consent-graph audit.
Target microservice: `microservices/consent-graph/`.
Counterparts: OneTrust / TrustArc / Cookiebot.
Benchmark posture: published numbers are separated from audit estimates and Oyatie targets.
Tenant-class retirement migration posture: one industry-leader target set, with deployment-context and tenant_class overlays.
No retired feature-ladder rows are defined in this document.

## Five-citation anchor block

1. Local PRD success metrics anchor: grant p95 <= 2s, projection freshness p95 <= 500ms, revocation p99 <= 1s, Cedar p99 <= 10ms, audit coverage 1.0, divergence 0, sovereignty violations 0, partner handshake p95 <= 30s: `microservices/consent-graph/PRD.md:77-87`.
2. Local scale anchor: 10M active agreements, 100K new agreements/day, 1M revocations/day, 100B projection events/day, 10K peers, 100K enforcement req/s, availability 99.99 percent, fail closed: `microservices/consent-graph/PRD.md:193-202`.
3. Local capacity anchor: capacity model repeats 10M active agreements, 100K new/day, 1M revocations/day, 100B events/day, 100K enforcement evaluations/s peak, and 10K peers: `microservices/consent-graph/capacity-model.md:7-18`.
4. Public OneTrust anchor: OneTrust developer docs publish UCPM API service objectives and selected API rate limits: https://developer.onetrust.com/onetrust/reference/consent-preference-management-api-service-level-objectives and https://developer.onetrust.com/onetrust/reference/rate-limits-overview.
5. Public Cookiebot anchor: Cookiebot support docs publish consent-log, scanner, API, TCF, and Google Consent Mode checker facts: https://support.cookiebot.com/hc/en-us/articles/14455846346652-Data-processed-when-using-Cookiebot-CMP and https://support.cookiebot.com/hc/en-us/articles/15485609486492-Google-Consent-Mode-Checker.

## Explicit methodology disclosure

1. This document does not claim private benchmark access to OneTrust, TrustArc, or Cookiebot.
2. Publicly disclosed vendor numbers are marked `source: public vendor documentation`.
3. Numbers inferred from public product class and system shape are marked `source: audit estimate`.
4. Oyatie current documented numbers are marked `source: local service artifact`.
5. Oyatie target numbers are marked `source: audit target`.
6. Legacy consent-graph benchmark numbers exist but are not reused as final targets because the prior benchmark used retired feature segmentation and an expanded counterpart set.
7. When a public vendor does not disclose latency, throughput, or capacity, this document says `not publicly disclosed` rather than inventing a measured fact.
8. Audit estimates are directional planning figures, not contractual vendor claims.
9. The Oyatie target set is single and industry-leader-grade.
10. Deployment overlays reduce or qualify throughput where infrastructure constrains capacity.
11. Tenant-class overlays describe usage caps or economic controls; they do not lower feature quality.
12. Time units are milliseconds unless stated otherwise.
13. Throughput units are requests per second, events per second, or calls per minute as noted.
14. Availability numbers are monthly or rolling-window objectives where source docs describe them that way.
15. Error-rate numbers are copied only when public docs provide them.

## §1 Methodology

1. Benchmark dimension: consent grant lifecycle latency.
2. Benchmark dimension: preference or consent-record write latency.
3. Benchmark dimension: revocation propagation latency.
4. Benchmark dimension: enforcement decision latency.
5. Benchmark dimension: projection freshness.
6. Benchmark dimension: partner handshake latency.
7. Benchmark dimension: consent receipt or consent-log retrieval.
8. Benchmark dimension: API rate limit or throughput.
9. Benchmark dimension: active agreement or subject scale ceiling.
10. Benchmark dimension: event volume ceiling.
11. Benchmark dimension: cookie/tracker scan scale.
12. Benchmark dimension: availability objective.
13. Benchmark dimension: 5XX or failed-request objective.
14. Benchmark dimension: audit coverage or evidence retention.
15. Benchmark dimension: deployment-context capacity overlay.
16. Test workload A: draft -> offer -> accept -> active agreement.
17. Test workload B: grantor commit -> grantee-visible projection.
18. Test workload C: revoke -> all enforcement caches deny.
19. Test workload D: single enforcement evaluation on hot cache.
20. Test workload E: partner handshake initiate -> respond -> finalize -> audit-root proof.
21. Test workload F: consent receipt lookup or consent-log extraction.
22. Test workload G: cookie/tracker scan and consent-mode check for web-CMP comparators.
23. OS disclosure: current service path has no `supported-oses.json`; OS-specific benchmark evidence is therefore absent.
24. Architecture disclosure: current service path has no `src/` or `tests/`; local target numbers are design targets and documented capacity estimates, not fresh executable results.
25. Deployment context disclosure: all six contexts are expected by dispatch, but service-local context modules are absent.
26. Tenant_class disclosure: service-local tenant_class semantics are absent; overlays below describe required benchmark framing for future implementation.
27. Baseline hardware for canonical Oyatie target: elastic production service cell sized to PRD and capacity-model targets.
28. Baseline data shape: agreement rows, projection events, revocation events, Cedar decisions, audit-chain cross-pointers, and partner-directory records.
29. Baseline consistency model: fail closed on stale policy, identity outage, and revocation uncertainty.
30. Baseline privacy model: no cross-border failover unless agreement and pack allow it.

## §2 Counterpart numbers

### §2.1 OneTrust public and estimated numbers

1. API availability objective: 99 percent for selected UCPM APIs; source: public OneTrust developer SLO documentation.
2. API P99 satisfactory threshold: 500 ms for selected data subject, preference, and receipt APIs; source: public OneTrust developer SLO documentation.
3. API 5XX objective: less than 0.5 percent of requests; source: public OneTrust developer SLO documentation.
4. Account default rate limit: 200,000 calls/hour; source: public OneTrust developer rate-limit documentation.
5. Account default per-minute limit: 20,000 calls/minute; source: public OneTrust developer rate-limit documentation.
6. Sandbox default rate limit: 50,000 calls/hour; source: public OneTrust developer rate-limit documentation.
7. Sandbox default per-minute limit: 5,000 calls/minute; source: public OneTrust developer rate-limit documentation.
8. Consent receipt endpoint limit: 2,000 calls/minute for one published UCPM endpoint; source: public OneTrust developer rate-limit documentation.
9. Preference endpoint limit: 3,000 calls/minute for one published UCPM endpoint; source: public OneTrust developer rate-limit documentation.
10. Consent receipt bulk-ingestion endpoint limit: 3,000 calls/minute; source: public OneTrust developer rate-limit documentation.
11. Consent profile endpoint example limit: 300 calls/minute to 1,000 calls/minute depending on endpoint; source: public OneTrust developer rate-limit documentation.
12. Public customer count signal: 14,000+ customers rely on OneTrust; source: public OneTrust product catalog.
13. Consent grant lifecycle p95: not publicly disclosed as an end-to-end DSA lifecycle; source: absence in inspected public OneTrust docs.
14. Revocation propagation p99 to downstream systems: not publicly disclosed; source: absence in inspected public OneTrust docs.
15. Cross-tenant projection freshness: not a public OneTrust UCPM product metric; source: product-surface mismatch.
16. Audit estimate for consent-record write p95: 500-1500 ms for selected APIs under documented P99 objective and rate limit class; source: audit estimate.
17. Audit estimate for end-to-end workflow revocation propagation: minutes to hours when downstream integrations and human workflow queues are involved; source: audit estimate from workflow-oriented product class.
18. Audit estimate for maximum sustained receipt reads under default account cap: 333 calls/s account-level if evenly distributed from 20,000 calls/minute; source: arithmetic from public rate limit.
19. Audit estimate for consent receipt endpoint cap: 33.3 calls/s for one endpoint at 2,000 calls/minute; source: arithmetic from public rate limit.
20. Audit estimate for preference endpoint cap: 50 calls/s for one endpoint at 3,000 calls/minute; source: arithmetic from public rate limit.

### §2.2 TrustArc public and estimated numbers

1. Connector count: 300+ prebuilt connectors; source: public TrustArc integrations page.
2. Language support: 60+ languages for consent experiences; source: public TrustArc CPM page.
3. Public-facing accessibility target: WCAG 2.2 Level AA and ADA standards for public-facing web elements; source: public TrustArc CPM page.
4. Consent sync timing: real-time syncing is claimed for consent data; source: public TrustArc CPM page.
5. Marketing/CRM integration examples: Salesforce, Marketo, HubSpot, Adobe Experience Platform, Mailchimp, Iterable, Twilio, BigQuery, and custom apps; source: public TrustArc CPM FAQ.
6. Mobile SDK platform count: Android, iOS, React Native, and Flutter are named; source: public TrustArc mobile app consent page.
7. Jurisdiction coverage examples: GDPR, CCPA, LGPD, PIPEDA, UK DPA, ePrivacy, and GPC are named; source: public TrustArc CPM page.
8. Consent repository availability objective: not publicly disclosed in inspected TrustArc pages; source: absence in inspected docs.
9. API P95/P99 latency: not publicly disclosed in inspected TrustArc pages; source: absence in inspected docs.
10. API rate limit: not publicly disclosed in inspected TrustArc pages; source: absence in inspected docs.
11. Consent revocation propagation p99: not publicly disclosed in inspected TrustArc pages; source: absence in inspected docs.
12. Cookie scan frequency: not publicly disclosed in inspected TrustArc pages; source: absence in inspected docs.
13. Audit estimate for consent repository write p95: 500-2000 ms under enterprise SaaS control-plane assumptions; source: audit estimate.
14. Audit estimate for downstream connector propagation: seconds to minutes depending on connector; source: audit estimate from integration workflow class.
15. Audit estimate for preference-center user update completion: 1-3 seconds for ordinary web form save; source: audit estimate.
16. Audit estimate for mobile SDK consent update local path: sub-second local capture plus server sync; source: audit estimate.
17. Audit estimate for cross-system sync at 300+ connector breadth: connector-dependent, with no uniform p99 public guarantee; source: audit estimate.
18. Audit estimate for DSAR workflow completion: hours to statutory-day windows, not sub-second; source: audit estimate from DSR workflow domain.

### §2.3 Cookiebot public and estimated numbers

1. Consent Mode checker runtime: approximately 10-15 seconds per check; source: public Cookiebot support page.
2. Consent Mode checker report retention: reports remain available for 30 days; source: public Cookiebot support page.
3. Scanner URL list visibility: up to 10,000 found subpages for subscribed accounts; source: public Cookiebot scanner support page.
4. Cookiebot CMP scan cadence: newly added domains are automatically scanned monthly; source: public Cookiebot support scan-report page.
5. Consent cookie persistence: first-party consent cookie can persist up to 12 months; source: public Cookiebot data-processed support page.
6. Consent-log fields: anonymized IP, date/time, user agent, URL, encrypted key value, and consent state; source: public Cookiebot consent-log support page.
7. Consent-data extraction API: returns consent statistics by domain and date range; source: public Cookiebot extraction API support page.
8. IAB TCF support: Cookiebot supports TCF and exposes the standard CMP API; source: public Cookiebot TCF support page.
9. IAB consent-string storage: TCF consent string is stored in the existing consent cookie and consent-log downloads; source: public Cookiebot TCF support page.
10. Page-view charge metric: public Cookiebot page says it does not charge based on page views or usage; source: public Cookiebot Google Consent Mode page.
11. Free-plan subpage ceiling: public Cookiebot page says free plan applies to fewer than 50 subpages; source: public Cookiebot Google Consent Mode page.
12. Premium plan subpage ceiling: public Cookiebot page describes very large plans for more than 5,000 subpages; source: public Cookiebot Google Consent Mode page.
13. Language support: 47+ languages; source: public Cookiebot Google Consent Mode page.
14. Cookie banner load latency: not publicly disclosed in inspected Cookiebot pages; source: absence in inspected docs.
15. Consent-log API latency: not publicly disclosed in inspected Cookiebot pages; source: absence in inspected docs.
16. Consent withdrawal p99 to tag blocking: not publicly disclosed in inspected Cookiebot pages; source: absence in inspected docs.
17. Audit estimate for banner decision local effect: immediate on page after cookie/script state changes; source: audit estimate from browser-side CMP behavior.
18. Audit estimate for consent-log extraction: seconds to minutes depending on date range and domain volume; source: audit estimate.
19. Audit estimate for scan completion: minutes to hours for large sites; source: audit estimate from crawler workload class.
20. Audit estimate for cross-domain enterprise synchronization: not applicable unless configured through domain groups and scripts; source: audit estimate.

## §3 Oyatie target numbers — single industry-leader target set

### §3.1 Canonical targets

1. Consent grant lifecycle p50 target: <= 250 ms; source: audit target.
2. Consent grant lifecycle p95 target: <= 1,200 ms; source: audit target, tighter than local SLO p95 <= 2s.
3. Consent grant lifecycle p99 target: <= 2,000 ms; source: audit target.
4. Consent grant lifecycle hard-tail target: <= 3,000 ms for non-manual paths; source: audit target aligned with PRD revocation tail language.
5. Projection freshness p50 target: <= 75 ms; source: audit target.
6. Projection freshness p95 target: <= 500 ms; source: local SLO and audit target.
7. Projection freshness p99 target: <= 1,000 ms; source: audit target.
8. Revocation propagation p50 target: <= 100 ms; source: audit target.
9. Revocation propagation p95 target: <= 500 ms; source: audit target aligned with ADR-SVC-CG-002 same-region target.
10. Revocation propagation p99 target: <= 1,000 ms; source: local SLO and audit target.
11. Revocation hard-tail target: <= 3,000 ms for all subscribed enforcement caches before emergency alert; source: audit target.
12. Cedar evaluation p50 target: <= 1 ms for hot cache; source: audit target.
13. Cedar evaluation p95 target: <= 5 ms for hot cache; source: audit target.
14. Cedar evaluation p99 target: <= 10 ms; source: local SLO and audit target.
15. Partner handshake p50 target: <= 5 seconds; source: audit target.
16. Partner handshake p95 target: <= 30 seconds; source: local SLO and audit target.
17. Partner handshake p99 target: <= 60 seconds; source: audit target.
18. Enforcement throughput target: >= 100,000 evaluations/s globally for year-one peak; source: local PRD.
19. Enforcement aggregate capacity target: >= 3,300,000 evaluations/s across 11 regions; source: `microservices/consent-graph/capacity-model.md:33-42`.
20. Projection event target: 100B events/day year one; source: local PRD and capacity model.
21. Projection aggregate capacity target: >= 11M project events/s global design capacity; source: `microservices/consent-graph/capacity-model.md:54-61`.
22. Active agreement target: 10M year-one active agreements; source: local PRD and capacity model.
23. New agreement target: 100K/day year one; source: local PRD and capacity model.
24. Revocation target: 1M/day year one; source: local PRD and capacity model.
25. Partner peer target: 10K concurrent partner-directory peers year one; source: local PRD and capacity model.
26. Bilateral audit event target: 50B/day year one; source: `microservices/consent-graph/capacity-model.md:9-18`.
27. Availability target: 99.99 percent for service availability; source: local PRD.
28. Audit-chain coverage target: 1.0 sealed event coverage; source: local PRD and OpenSLO.
29. Agreement-state divergence target: 0 divergence; source: local PRD and OpenSLO.
30. Sovereignty violation target: 0 violations; source: local PRD and OpenSLO.
31. Bilateral-chain integrity target: 1.0 paired link integrity; source: local PRD and OpenSLO.
32. API 5XX target: <= 0.1 percent for consent/enforcement APIs; source: audit target, stricter than OneTrust public 0.5 percent selected objective.
33. Consent receipt or revocation receipt lookup p95 target: <= 200 ms from indexed store; source: audit target.
34. Consent-source import throughput target: >= 10,000 records/minute per tenant migration worker; source: audit target.
35. Partner offboarding cascade target: revoke active peer agreements within 60 seconds for ordinary scale and alert if batch exceeds deadline; source: audit target.

### §3.2 Deployment-context overlays

1. `oyatie-public-cloud` overlay: canonical targets apply with elastic capacity and regional admission control.
2. `oyatie-public-cloud` overlay: scale beyond year-one targets requires adding regions, pods, brokers, and Citus workers according to capacity model ratios.
3. `guest-on-aws` overlay: canonical latency targets apply when customer provisions required compute, storage, Pulsar, Postgres, Valkey, OpenBao, and network capacity.
4. `guest-on-aws` overlay: throughput is capped by customer account quotas, region count, and private-link design until verified by context module tests.
5. `guest-on-oci` overlay: canonical targets apply for paid or revenue-share tenants when OCI resources are provisioned beyond the free profile.
6. `guest-on-oci` overlay: demo_trial tenants use the OCI Always Free profile and receive usage caps, not degraded feature quality.
7. `on-prem` overlay: canonical correctness targets apply, but throughput and p99 latency require facility-specific hardware and WAN certification.
8. `on-prem` overlay: failover cannot cross residency boundaries unless agreement and pack allow it.
9. `colo` overlay: canonical correctness targets apply, with throughput constrained by purchased racks, cross-connects, storage, and operator procedures.
10. `colo` overlay: partner handshake p99 may depend on external network paths and must be measured per facility.
11. `oyatie-as-cloud-provider` overlay: canonical targets apply as provider-grade SLOs with stronger admission control and capacity reservation.
12. `oyatie-as-cloud-provider` overlay: revenue-share tenants may receive at-cost substrate but still use the same correctness and privacy targets.
13. OCI Always Free profile overlay: cap infrastructure around 4 OCPU and 24GB Ampere plus Always Free platform quotas; source: OCI memory `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:15-32`.
14. OCI Always Free profile overlay: load balancer bandwidth and egress constraints require demo_trial caps; source: same memory lines `31-32`.
15. OCI Always Free profile target: enforcement throughput budget 500-1,000 evaluations/s per demo tenant cell, depending on co-location and cache hit rate; source: audit target.
16. OCI Always Free profile target: agreement writes capped at 1,000/day per demo tenant; source: audit target.
17. OCI Always Free profile target: revocations capped at 10,000/day per demo tenant with emergency override; source: audit target.
18. OCI Always Free profile target: projection events capped at 10M/day per demo tenant; source: audit target.
19. OCI Always Free profile target: partner peers capped at 25 active peers per demo tenant; source: audit target.
20. OCI Always Free profile target: best-effort SLO disclosure while retaining fail-closed, audit, and sovereignty invariants; source: audit target.

### §3.3 Tenant_class overlays

1. `demo_trial` overlay: free, OCI Always Free profile by default, time caps, usage caps, best-effort SLO, no compliance packs, no BYOK.
2. `demo_trial` overlay: same code path and same privacy/security invariants as paid tenants.
3. `demo_trial` overlay: rate caps protect the free substrate, not feature quality.
4. `demo_trial` overlay: benchmark reports must separate free-profile caps from canonical service capability.
5. `demo_trial` overlay: expected public benchmark slice is small-cell smoke proof, not industry-scale throughput.
6. `paid` overlay: per-seat license plus usage-based billing, any deployment context, contractual SLO, compliance packs allowed, BYOK allowed.
7. `paid` overlay: canonical target numbers apply when paid capacity is provisioned.
8. `paid` overlay: rate limits scale with purchased capacity and contractual terms.
9. `paid` overlay: benchmark reports should include customer-specific deployment capacity and contractual SLO class.
10. `paid` overlay: no feature is held back because the tenant paid a lower feature level; only capacity and contract terms vary.
11. `revenue_share` overlay: Oyatie takes a percent of customer's gross revenue for marketplace sellers, B2C operators, embedded SaaS resellers, and affiliate partners.
12. `revenue_share` overlay: substrate runs at-cost or zero-margin but retains industry-leader quality.
13. `revenue_share` overlay: capacity gates protect gross-margin and settlement controls, not feature access.
14. `revenue_share` overlay: benchmark reports should include settlement-linked cost-per-transaction and capacity reserve metrics.
15. `revenue_share` overlay: compliance packs and BYOK are allowed when the contract and deployment context support them.
16. All tenant_class overlays: audit coverage, sovereignty, revocation correctness, and Cedar fail-closed behavior remain invariant.
17. All tenant_class overlays: service docs need a concrete contract field; current service path lacks one.
18. All tenant_class overlays: rate-limit and cost-budget docs need updates to prevent old feature-ladder vocabulary from re-entering the model.
19. All tenant_class overlays: performance dashboards need labels for tenant_class and deployment_context.
20. All tenant_class overlays: public claims must disclose whether numbers are canonical service capacity, free-profile capacity, or customer-provisioned capacity.

## §4 Comparison narrative

1. Consent API latency: OneTrust publishes selected UCPM P99 satisfactory threshold of 500 ms, while Oyatie targets Cedar evaluation p99 <= 10 ms and consent grant p95 <= 1,200 ms for an entire DSA lifecycle.
2. Consent API latency classification: Oyatie is ahead for enforcement hot path, at parity or catch-up for generic consent-record CRUD until executable tests exist.
3. Preference center update: OneTrust and TrustArc are ahead because consent-graph has no preference-center UI.
4. Preference center classification: gap, with likely ownership outside consent-graph unless scope changes.
5. Cookie banner latency: Cookiebot is ahead because consent-graph has no banner or script-blocking surface.
6. Cookie banner classification: gap, not a benchmark failure of enforcement substrate.
7. Cookie/tracker scan scale: Cookiebot publishes URL-list visibility up to 10,000 found subpages for subscribed accounts and monthly domain scans.
8. Cookie/tracker scan classification: gap, because consent-graph has no scanner workload.
9. Google Consent Mode check: Cookiebot publishes approximately 10-15 seconds per check and 30-day report retention.
10. Google Consent Mode classification: gap, because consent-graph has no GCM checker or schema.
11. API rate limits: OneTrust publishes 20,000 calls/minute account default and lower endpoint-specific UCPM caps.
12. API rate-limit classification: Oyatie target is ahead on enforcement throughput, but public rate-limit policy is not documented for tenant_class.
13. Revocation propagation: Oyatie targets p99 <= 1 second; public counterpart docs inspected do not disclose comparable downstream revocation p99.
14. Revocation propagation classification: Oyatie is ahead if implemented and tested.
15. Projection freshness: Oyatie targets p95 <= 500 ms and p99 <= 1,000 ms; top-three CMP counterparts do not expose a comparable cross-tenant projection product.
16. Projection freshness classification: Oyatie is ahead in its unique product surface.
17. Partner handshake: Oyatie targets p95 <= 30 seconds; counterparts provide integrations and connectors rather than bilateral audit-root handshake.
18. Partner handshake classification: Oyatie is ahead for audit-root proof, behind TrustArc for named connector breadth.
19. Connector breadth: TrustArc publishes 300+ prebuilt connectors; Oyatie consent-graph has internal catalogs but no named marketing/CRM connector set.
20. Connector breadth classification: catch-up or external dependency required.
21. Mobile consent SDK: TrustArc names Android, iOS, React Native, and Flutter; OneTrust also covers mobile/app consent in public product material.
22. Mobile consent SDK classification: gap for consent-graph.
23. Consent logs: Cookiebot publishes consent-log fields and extraction API; consent-graph has audit-chain and revocation receipts but no web consent-log export.
24. Consent logs classification: partial parity for enforcement evidence, gap for CMP-style logs.
25. Availability: OneTrust publishes 99 percent for selected UCPM APIs; Oyatie PRD targets 99.99 percent service availability.
26. Availability classification: Oyatie target is ahead, but not proven by live tests under this service path.
27. Error rate: OneTrust publishes less than 0.5 percent 5XX objective; Oyatie audit target is <= 0.1 percent.
28. Error-rate classification: Oyatie target is ahead, but needs implementation evidence.
29. Scale ceiling: Oyatie local capacity model targets 10M active agreements and 100B projection events/day; counterparts do not publish comparable cross-tenant DSA scale.
30. Scale ceiling classification: Oyatie is ahead in documented DSA design, unproven in executable artifacts.
31. Tenant-class performance: counterparts publish SaaS plan or account limits; Oyatie tenant_class model is absent in current service artifacts.
32. Tenant-class classification: gap until contract, policy, dashboard, and benchmark labels exist.
33. Deployment-context overlay: counterparts are SaaS-hosted products; Oyatie must prove six contexts.
34. Deployment-context classification: gap because service has no OpenTofu context modules.
35. OCI Always Free profile: Cookiebot has a small/free web plan analogy, but Oyatie needs demo_trial infrastructure constraints.
36. OCI Always Free profile classification: gap because `iac/oci-guest/always-free/` is absent.
37. OS support: counterparts do not publish a service OS matrix; Oyatie canonical requirements demand one.
38. OS support classification: gap because no `supported-oses.json` exists.
39. Rust/source evidence: counterparts are products; Oyatie service directory currently has documentation but no code or tests.
40. Rust/source classification: gap for implementation proof, despite passing forbidden-extension scan.
41. Audit evidence: Oyatie has strong local SLO, runbook, and audit-chain design docs.
42. Audit evidence classification: ahead in design, needs executable evidence bundle.
43. Compliance packs: Oyatie has broad compliance map and runbooks; OneTrust and TrustArc are mature privacy platforms.
44. Compliance packs classification: partial parity, with maturity depending on generated-content cleanup and tenant_class integration.
45. Final benchmark stance: consent-graph has industry-leading target numbers for enforcement, revocation, projection, and audit-chain integrity.
46. Final benchmark caveat: user-facing CMP, preference, cookie, mobile, and connector benchmarks require external owner or product expansion.
47. Final benchmark stop condition: future updates must separate public vendor facts, audit estimates, local documented targets, and executable measurements.
48. Final benchmark stop condition: future updates must avoid feature-ladder segmentation and use deployment_context plus tenant_class overlays.
49. Final benchmark stop condition: once code/tests exist, this document should be refreshed with fresh `cargo` benchmark output and context-module load tests.
50. Final benchmark stop condition: until then, this report is a target and counterpart audit, not a live performance certification.

## §5 Verification evidence needed before certification

1. Evidence needed: Rust benchmark harness for consent grant lifecycle.
2. Acceptance target: p50 <= 250 ms, p95 <= 1,200 ms, p99 <= 2,000 ms on canonical service-cell hardware.
3. Evidence needed: Rust benchmark harness for projection freshness.
4. Acceptance target: p50 <= 75 ms, p95 <= 500 ms, p99 <= 1,000 ms under representative Pulsar topic count.
5. Evidence needed: revocation storm benchmark across enforcement caches.
6. Acceptance target: p50 <= 100 ms, p95 <= 500 ms, p99 <= 1,000 ms, hard-tail <= 3,000 ms.
7. Evidence needed: Cedar hot-cache benchmark.
8. Acceptance target: p50 <= 1 ms, p95 <= 5 ms, p99 <= 10 ms.
9. Evidence needed: partner handshake benchmark with audit-root proof.
10. Acceptance target: p50 <= 5 seconds, p95 <= 30 seconds, p99 <= 60 seconds.
11. Evidence needed: audit-chain seal throughput benchmark.
12. Acceptance target: 50B/day design envelope with p99 seal latency <= 500 ms for service-local bridge.
13. Evidence needed: projection-gateway worker benchmark.
14. Acceptance target: 100B/day design envelope and 11M events/s global design capacity.
15. Evidence needed: rate-limit benchmark tied to tenant_class.
16. Acceptance target: demo_trial caps enforced without changing safety invariants.
17. Evidence needed: paid tenant benchmark under purchased-capacity profile.
18. Acceptance target: canonical target set met when capacity is provisioned.
19. Evidence needed: revenue_share benchmark under at-cost substrate and settlement guardrails.
20. Acceptance target: canonical correctness targets met while economic caps prevent runaway subsidy.
21. Evidence needed: `oyatie-public-cloud` OpenTofu context load test.
22. Acceptance target: canonical targets met in provider-operated public cloud context.
23. Evidence needed: `guest-on-aws` OpenTofu context load test.
24. Acceptance target: canonical targets met when customer quotas meet documented minimums.
25. Evidence needed: `guest-on-oci` paid-profile load test.
26. Acceptance target: canonical targets met outside the free profile.
27. Evidence needed: OCI Always Free profile smoke and cap test.
28. Acceptance target: demo_trial caps met with fail-closed behavior and complete audit evidence.
29. Evidence needed: on-prem hardware certification profile.
30. Acceptance target: correctness invariants met and throughput documented per certified facility.
31. Evidence needed: colo hardware and cross-connect certification profile.
32. Acceptance target: correctness invariants met and latency documented per facility.
33. Evidence needed: `oyatie-as-cloud-provider` provider-grade admission-control test.
34. Acceptance target: canonical targets met with reserved capacity and provider SLO labels.
35. Evidence needed: supported-OS matrix benchmark entries.
36. Acceptance target: Tier-1 server OS support documented by service manifest and CI evidence.
37. Evidence needed: counterpart benchmark evidence archive.
38. Acceptance target: every external number is marked public, estimated, or measured by allowed test rig.
39. Evidence needed: no-retired-feature-ladder benchmark lint.
40. Acceptance target: benchmark docs use deployment_context and tenant_class only.
41. Evidence needed: web-CMP workload owner decision.
42. Acceptance target: Cookiebot-style scan, banner, TCF, and Google Consent Mode workloads are either implemented or handed off.
43. Evidence needed: OneTrust/TrustArc migration import benchmark.
44. Acceptance target: 10,000 consent records/minute per tenant migration worker without audit gaps.
45. Evidence needed: DSR cascade benchmark.
46. Acceptance target: cross-tenant tombstone and revocation evidence meets statutory and service deadlines.
47. Evidence needed: dashboard-label verification.
48. Acceptance target: benchmark dashboards include `deployment_context`, `tenant_class`, `region`, `pack_id`, and `workload`.
49. Evidence needed: final certification gate.
50. Acceptance target: local executable evidence matches or revises every audit target above before public performance claims.
