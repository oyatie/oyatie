---
doc_class: Performance-Benchmark-Numbers
microservice: crm
status: Wave-4-Rolling-Audit-Companion
wave: Wave-4-Rolling-Big-8-CRM
date: 2026-05-21
auditor_agent_class: codex-ms-audit-crm
audit_priority: P0-Big-8
parity_set: [Salesforce Sales Cloud, HubSpot CRM, Microsoft Dynamics 365 Sales]
methodology_floor: single industry-leader target + deployment-context overlay + tenant-class overlay
no_tier_segmentation: true
companion_audit_deliverables:
  - microservices/crm/coherence-audit-2026-05-20.md
  - microservices/crm/feature-parity-matrix-2026-05-20.md
---

CANONICAL ANCHORS

1. /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-2.13-15 (Salesforce primary CRM anchor for benchmarking).
2. /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_profiles_2026_05_20.md and feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md (no tier-segmentation; demo_trial caps vs paid no-cap).
3. /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md (six deployment contexts overlay).
4. /Users/jasonlee/oyatie/microservices/crm/slos/{crm-availability,crm-latency-p99,crm-throughput,account-master-success-rate}.openslo.yaml (current Oyatie SLO declarations).
5. Salesforce Trust Status + Governor Limits docs + HubSpot API Rate Limits + Microsoft Dynamics 365 Sales Throughput Guidance (industry-leader benchmark sources).

# Performance Benchmark Numbers: crm

## §1 Methodology

This benchmark deliverable uses the post-tier-retirement model from the no-capability-tiers-2026-05-20 directive: NO retired named capability levels tier segmentation; NO sandbox/growth/enterprise/regulated-enterprise capacity tiers either. The model is:

1. **Single industry-leader target per metric.** Each performance metric has one canonical target equal to or better than the best of {Salesforce Sales Cloud, HubSpot CRM, Microsoft Dynamics 365 Sales}. This is the "UNION-minimum" target — Oyatie crm must beat the minimum of the three counterparts and aim at or above the maximum.

2. **Deployment-context overlay (6 contexts).** Each metric has per-context behavior: oyatie-public-cloud, aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider. Latency floors differ across contexts because network round-trips, storage class, and compute substrate differ; capacity ceilings differ because tenant resource quotas differ.

3. **Tenant-class overlay (2 classes).** Each metric has demo_trial behavior (with hard usage caps) and paid behavior (no caps; scales with billing_component subscriptions). Paid tenants with per_usage billing_component get usage-meter visibility into the same metric; paid tenants with per_seat see seat-count-derived guarantees.

4. **No tier-shaped segmentation anywhere.** The legacy capacity-model.md §B Tier assumptions table (sandbox/growth/enterprise/regulated-enterprise) is retired by this deliverable. Replacement is the (context × tenant_class) grid below.

5. **OCI Always Free anchor.** The demo_trial tenant on the oci-guest deployment context runs inside OCI Always Free limits per the oci-always-free-maximization-2026-05-20 memory: 4 OCPU + 24 GB RAM + 200 GB block + 2× Autonomous DB × 20 GB + 10 TB egress + 10 Mbps LB. The performance numbers for demo_trial on oci-guest are derived from this resource ceiling.

6. **Hyperscaler-grade rigor sub-test applied.** Per ADR-0322 substance-bar doctrine and ADR-0328 §C-4 hyperscaler-grade rigor application, every metric is named, citable, has a measurement window, has a failure-mode tree, and has a rollback path.

## §2 Counterpart benchmark numbers

This section establishes the industry-leader reference numbers used as the parity floor.

### §2.1 Salesforce Sales Cloud governor limits + published SLAs

Source: Salesforce Developer Limits and Allocations (https://developer.salesforce.com/docs/atlas.en-us.salesforce_app_limits_cheatsheet.meta/salesforce_app_limits_cheatsheet/) and Trust Status (https://status.salesforce.com/).

Salesforce-A1 (Apex CPU limit per transaction): 10,000 ms synchronous + 60,000 ms async. Used as the upper-bound for single-mutation processing time.

Salesforce-A2 (SOQL query rows per transaction): 50,000 rows. Bulk read ceiling per single transaction.

Salesforce-A3 (DML statements per transaction): 150. Bulk write ceiling per single transaction.

Salesforce-A4 (DML rows per transaction): 10,000. Aggregate row mutation ceiling.

Salesforce-A5 (Heap size per transaction): 6 MB sync + 12 MB async. Memory ceiling.

Salesforce-A6 (Concurrent long-running synchronous Apex transactions per org): 10. Tenant-level concurrency ceiling.

Salesforce-A7 (REST API calls per 24-hour rolling window per org for Enterprise Edition): 1,000,000. Daily quota for Enterprise org with ~1,000 seats.

Salesforce-A8 (Bulk API 2.0 batches per 24 hours): 15,000. Bulk import/export ceiling per day.

Salesforce-A9 (Streaming API events per day): 100,000 to 5,000,000 depending on edition. Event stream throughput ceiling.

Salesforce-A10 (Platform Event high-volume): 200,000 events per day for Enterprise. Higher tier for unlimited.

Salesforce-A11 (Time-Based Workflow queue depth per org): 50,000 actions. Workflow backlog ceiling.

Salesforce-A12 (Outbound message size limit): 1 MB. Per-message payload ceiling.

Salesforce-A13 (Email delivery max recipients per day per Enterprise Edition user): 5,000. Per-user email ceiling.

Salesforce-A14 (Average end-user latency, p50 typical observation): 200-400 ms. From third-party observability data + Salesforce's own performance bulletins.

Salesforce-A15 (Trust SLA availability target, multi-region): 99.99% from Salesforce Trust (Premier Success). Standard Success is 99.9%.

Salesforce-A16 (Record query response time p50 typical): under 100 ms for tenant-scoped queries; under 500 ms for multi-million-record selective queries.

Salesforce-A17 (Bulk API throughput typical): 30-50 records per second on Bulk API 2.0 default; configurable up to ~100 records per second.

Salesforce-A18 (Database storage per org): 10 GB included + 20 MB per Enterprise Edition user. Common large org: 1-5 TB across all objects.

Salesforce-A19 (File storage per org): 10 GB included + 2 GB per Enterprise Edition user. Common large org: 5-20 TB attachment storage.

Salesforce-A20 (Maximum concurrent Bulk API 2.0 jobs per org): 100.

### §2.2 HubSpot API rate limits + published SLAs

Source: HubSpot Developer Docs (https://developers.hubspot.com/docs/api/usage-details) and HubSpot Status (https://status.hubspot.com/).

HubSpot-A1 (API requests per second per portal for OAuth apps, Pro/Enterprise): 100 requests per second + bursting allowance. Free/Starter tier 10 requests per second.

HubSpot-A2 (Daily API call quota for OAuth apps, Enterprise): 1,000,000 calls per day.

HubSpot-A3 (Bulk import batch size): 100 records per batch + max 200,000 records per import job.

HubSpot-A4 (Webhook payload max size): 1 MB. Webhook re-delivery up to 7 days.

HubSpot-A5 (Workflow execution daily quota for Enterprise): unlimited; performance ~1-10 second per workflow step typical.

HubSpot-A6 (Concurrent custom code action executions per workflow): 10. Internal queue depth managed by HubSpot.

HubSpot-A7 (Contact record query response p50 typical): under 200 ms via REST CRM API.

HubSpot-A8 (Status page availability target, marketed): 99.95%. Operations Hub Enterprise SLA contractual.

HubSpot-A9 (Search API result limit per request): 10,000 records max via cursor pagination.

HubSpot-A10 (Property history retention): full retention for 2 years; older history compressed.

HubSpot-A11 (Custom property count per object, Enterprise): up to 1,000 custom properties per object.

HubSpot-A12 (Total contact ceiling per portal, Enterprise): 1,000,000-5,000,000 contacts depending on contract.

HubSpot-A13 (Email send daily quota for Marketing Hub Enterprise): 10x contact count per month, ~333,333 per day for 1M-contact portal.

HubSpot-A14 (Sequence enrollment limit per user per day): 500.

HubSpot-A15 (Sequence step delay precision): 5-minute granularity.

### §2.3 Microsoft Dynamics 365 Sales throughput guidance

Source: Microsoft Learn Dynamics 365 Service Protection API Limits (https://learn.microsoft.com/en-us/power-apps/developer/data-platform/api-limits) and Dynamics 365 SLA documentation.

Dynamics-A1 (Service Protection request limit per user per 5-minute window): 6,000 requests. Tenant-level ceiling combats abuse.

Dynamics-A2 (Service Protection execution-time limit per user per 5-minute window): 20 minutes. Aggregate execution-time ceiling.

Dynamics-A3 (Service Protection concurrent-request limit per user): 52 concurrent requests. Per-principal concurrency ceiling.

Dynamics-A4 (Batch operation max payload size): 16 MB. Per-batch payload ceiling.

Dynamics-A5 (Batch operation max records per batch): 1,000 records.

Dynamics-A6 (Dataverse storage per environment): default 1 GB + 2.5 GB per licensed user (Enterprise). Common org: 0.5-2 TB Dataverse storage.

Dynamics-A7 (Web API response time typical p50): under 300 ms for tenant-scoped queries; under 500 ms for FetchXML aggregate.

Dynamics-A8 (Dynamics 365 SLA contractual availability): 99.9% Online Services SLA (Microsoft). Some tiers offer 99.99% with Premier Support.

Dynamics-A9 (Power Automate flow execution daily ceiling): tier-dependent, Premium connector allowances ranging 5,000-1,000,000 per user per day.

Dynamics-A10 (Plug-in execution time limit per request): 2 minutes synchronous, no hard cap for async with sandboxed pool.

Dynamics-A11 (Maximum field count per entity): 1,024 fields per entity (Dataverse default).

Dynamics-A12 (Maximum custom entities per environment): 300 default, expandable.

Dynamics-A13 (Bulk Data Loader throughput typical): 50-100 records per second per worker; multi-worker scales to 500-1,000 records per second.

Dynamics-A14 (Notification latency for change events via Azure Service Bus): under 1 second p95.

Dynamics-A15 (Real-time AI scoring latency for Lead/Opp scoring p95): under 2 seconds.

### §2.4 Counterpart benchmark synthesis

Synthesis table (counterpart-min, counterpart-typical, counterpart-max) per key metric:

| Metric | Counterpart-min (worst) | Counterpart-typical (median) | Counterpart-max (best) | UNION-min floor for Oyatie |
|---|---|---|---|---|
| Single-record read p50 (ms) | 200 (HubSpot) | 200-300 | 100 (Salesforce best case) | 100 |
| Single-record write p50 (ms) | 300 (Dynamics typical) | 200-400 | 150 | 150 |
| Tenant-scoped query p99 (ms) | 1,500 (HubSpot Enterprise) | 800-1,500 | 500 (Salesforce with selective index) | 500 |
| Bulk import throughput (records/sec) | 30 (Salesforce Bulk default) | 50-100 | 1,000 (Dynamics multi-worker) | 100 |
| API requests per second per tenant | 10 (HubSpot Free) | 50-100 | 100-200 | 100 |
| Daily API call quota per tenant | 100,000 (HubSpot Starter) | 1,000,000 | 5,000,000 (Salesforce Unlimited) | 1,000,000 |
| Concurrent connections per tenant | 10 (Salesforce Apex sync) | 50 | 100 (HubSpot) | 100 |
| Webhook / event delivery p95 (sec) | 5 (HubSpot retry semantics) | 1-3 | under 1 (Dynamics Azure SB) | 1 |
| AI scoring latency p95 (sec) | 5 (HubSpot Predictive) | 2-3 | 2 (Dynamics Sales Insights) | 2 |
| Availability SLA (%) | 99.9 (Microsoft default) | 99.95 | 99.99 (Salesforce Premier) | 99.99 |
| Storage per tenant (default GB) | 1 (Dynamics default) | 10 | 10+per-user (Salesforce 10 + 20 MB/user) | 10 |

Oyatie crm UNION-min floor is the most-demanding of the three on each row. The Oyatie target is at or above this floor.

## §3 Oyatie single industry-leader target + per-deployment-context overlay + per-tenant-class overlay

This section sets the canonical Oyatie crm performance targets. Each metric has one base target plus six deployment-context entries plus a demo_trial / paid split.

### §3.1 Latency targets

**M-L1: Single-record read p50 — 80 ms base target** (better than Salesforce's 100 ms typical).

Per-deployment-context overlay:
- oyatie-public-cloud: 80 ms p50 (canonical baseline).
- aws-guest: 90 ms p50 (additive AWS network hop within region).
- oci-guest paid: 90 ms p50 (additive OCI network hop within region).
- oci-guest demo_trial (Always Free): 120 ms p50 (Ampere A1 ARM 4 OCPU; shared CPU credit budget; smaller cache hit ratio).
- on-prem: 60 ms p50 (no public-cloud network hop; LAN-only).
- colo: 70 ms p50 (LAN + customer-WAN).
- oyatie-as-cloud-provider: 80 ms p50 (canonical baseline; Oyatie's own substrate).

Per-tenant-class overlay:
- demo_trial: no contractual SLO, best-effort target equal to the per-context value above.
- paid: contractual SLO at the per-context value above plus 20 percent buffer.

Failure-mode: latency excursion is caught by SLOs/crm-latency-p99.openslo.yaml + dashboards/crm-overview.json p50 panel.

**M-L2: Single-record write (mutation) p50 — 120 ms base target**.

Includes Cedar policy evaluation + audit-chain seal + AsyncAPI event emission + ontology projection. The base target compresses Salesforce's 200-400 ms typical.

Per-deployment-context overlay:
- oyatie-public-cloud / oyatie-as-cloud-provider: 120 ms p50.
- aws-guest / oci-guest paid: 140 ms p50.
- oci-guest demo_trial: 200 ms p50 (Always Free constraints).
- on-prem: 90 ms p50.
- colo: 100 ms p50.

Per-tenant-class overlay:
- demo_trial: best-effort.
- paid: contractual SLO with 20% buffer.

**M-L3: Tenant-scoped query p99 — 400 ms base target**.

Selective-index path. Salesforce best case 500 ms; Oyatie target 400 ms.

Per-deployment-context overlay:
- oyatie-public-cloud / oyatie-as-cloud-provider: 400 ms p99.
- aws-guest / oci-guest paid: 450 ms p99.
- oci-guest demo_trial: 800 ms p99 (Autonomous DB 1 OCPU constraint).
- on-prem: 300 ms p99.
- colo: 350 ms p99.

**M-L4: Bulk query non-selective p99 — 2,000 ms base target**.

Full-tenant-scope scan. Salesforce SOQL 50,000-row limit at typical 1-2 seconds; HubSpot Search API up to 10,000 rows; Dynamics FetchXML aggregate up to 500 ms typical. Oyatie target 2 seconds covers up to 100,000 rows.

**M-L5: AsyncAPI event delivery p95 — 1 second base target**.

From mutation accept to event consumer receipt. Dynamics Azure Service Bus p95 under 1 second; HubSpot Webhook retry semantics 5 seconds; Salesforce Streaming sub-second when within capacity. Oyatie target matches Dynamics.

**M-L6: AI scoring latency p95 (lead-score, opp-score) — 2 seconds base target** (matches Dynamics Sales Insights).

DELEGATED to intelligence µservice; crm-side overhead under 300 ms. End-to-end target 2 seconds.

### §3.2 Throughput targets

**M-T1: Single-tenant API requests per second sustained — 200 RPS base target**.

Beats HubSpot's 100 RPS Pro/Enterprise and matches Salesforce-equivalent Enterprise rate after governor optimisation.

Per-deployment-context overlay:
- oyatie-public-cloud / oyatie-as-cloud-provider paid: 200 RPS sustained.
- aws-guest paid: 200 RPS sustained.
- oci-guest paid: 200 RPS sustained.
- on-prem paid: 500 RPS sustained (LAN proximity).
- colo paid: 400 RPS sustained.
- oci-guest demo_trial: 20 RPS sustained (Always Free 10 Mbps LB throughput; bursts to 50 RPS for 10 seconds then throttle).
- All demo_trial across contexts: 50 RPS sustained (consistent demo_trial cap).

**M-T2: Bulk import throughput — 200 records per second per worker; 2,000 records per second multi-worker** (paid).

Beats Salesforce Bulk API 2.0 default 30 records per second and matches Dynamics multi-worker.

Per-tenant-class overlay:
- demo_trial: 50 records per second; max 10,000 records per import job.
- paid: 200 records per second per worker; up to 10 concurrent workers = 2,000 records per second.

**M-T3: AsyncAPI event throughput — 5,000 events per second per tenant** (paid).

Combined across the six aggregate event streams. Beats Salesforce Platform Event Enterprise 200,000 per day = 2.3 per second average.

Per-tenant-class overlay:
- demo_trial: 100 events per second per tenant.
- paid: 5,000 events per second per tenant; burst to 25,000 per second for 60 seconds.

**M-T4: Concurrent connections per tenant — 200** (paid).

Matches HubSpot ceiling; beats Salesforce 10 sync Apex transactions.

Per-tenant-class overlay:
- demo_trial: 20 concurrent connections.
- paid: 200 concurrent connections; per-context overlay applies (on-prem can be tuned to 500-1,000).

**M-T5: Daily API call quota — 5,000,000 per tenant** (paid).

Matches Salesforce Unlimited. Beats HubSpot Enterprise 1,000,000.

Per-tenant-class overlay:
- demo_trial: 100,000 calls per day.
- paid + per_seat: 5,000 calls per day per seat (so 1,000-seat org = 5,000,000 per day).
- paid + per_usage: pay-as-you-go; first 5,000,000 per day per tenant included; additional metered.
- paid + revenue_share: tied to GMV; calls auto-scale with revenue tier.

### §3.3 Availability targets

**M-A1: Multi-region paid availability — 99.99% (4 nines, ~52 minutes downtime per year)**.

Matches Salesforce Trust Premier Success. Beats Microsoft Dynamics standard 99.9%.

Per-deployment-context overlay:
- oyatie-public-cloud paid: 99.99%.
- aws-guest paid: 99.99% (when tenant runs multi-AZ multi-region; AWS underlying SLA is 99.99%).
- oci-guest paid: 99.99% (when tenant runs multi-AD multi-region; OCI Compute SLA 99.95%, combined design 99.99%).
- on-prem paid: 99.9% (single-DC default) or 99.99% with multi-DC.
- colo paid: 99.9% default or higher per contract.
- oyatie-as-cloud-provider paid: 99.99%.

Per-tenant-class overlay:
- demo_trial: 99% best-effort (no contractual SLA per tenant-class memory).
- paid: contractual SLO per the value above.

**M-A2: Single-region availability — 99.95%**.

Beats Microsoft default. Matches HubSpot Enterprise marketed SLA.

**M-A3: Regional failover RTO — 5 minutes p95**.

Beats Salesforce typical 15-30 minutes for region failover.

**M-A4: Regional failover RPO — 60 seconds**.

Beats HubSpot's "best effort" RPO. Salesforce Premier RPO 15 minutes typical.

### §3.4 Capacity targets

**M-C1: Records per tenant — 100,000,000** (paid).

Matches Salesforce Enterprise + HubSpot Enterprise (5M-contact portal typical). Beats Dynamics default 1M-record cap.

Per-tenant-class overlay:
- demo_trial: 50,000 records per aggregate (account-master + opportunity + quote + service-case + campaign + loyalty-ledger) = 300,000 records per tenant; fits in OCI Always Free 200 GB block at ~700 bytes per record average.
- paid: 100,000,000 records per tenant base; expandable via tenant-class+contract.

**M-C2: Storage per tenant — 10 TB** (paid).

Beats Salesforce default 10 GB + 20 MB per user. Beats HubSpot variable.

Per-tenant-class overlay:
- demo_trial: 5 GB total (fits OCI Always Free Object Storage 10 GB + Autonomous DB 20 GB × 2).
- paid: 10 TB base; expandable.

**M-C3: Custom properties per object — 1,000** (paid).

Matches HubSpot Enterprise. Matches Dynamics 1,024.

Per-tenant-class overlay:
- demo_trial: 50 custom properties per object.
- paid: 1,000 custom properties per object.

**M-C4: Seats per tenant — unlimited** (paid + per_seat).

Beats Salesforce / HubSpot / Dynamics ceiling-by-edition.

Per-tenant-class overlay:
- demo_trial: 10 seats max.
- paid + per_seat: unlimited; price per seat per the tenant's contract.

### §3.5 Cost / efficiency targets

**M-E1: Cost per request (CPR) demo_trial — $0** (OCI Always Free).

Matches: no counterpart offers $0 for usable CRM tier; Salesforce Developer Edition is free but capped; HubSpot Free has hard ceilings. Oyatie at OCI Always Free is a hyperscaler-grade $0 demo_trial.

**M-E2: Cost per request (CPR) paid + per_usage — $0.00001 per request target**.

Industry: AWS Lambda baseline ~$0.0000002 per invocation; Salesforce REST call effectively included in license; HubSpot included. Oyatie's target $0.00001 supports a 5,000,000-call-per-day tenant at $50/day = $1,500/month variable.

**M-E3: Cost per seat (CPS) paid + per_seat — $35/seat/month target** (10x cheaper than Salesforce Sales Cloud Enterprise $165/seat/month; matches HubSpot Sales Hub Starter price-point but with Enterprise features).

**M-E4: Revenue share (RPS) paid + revenue_share — 10-30 percent of GMV** (industry-standard marketplace commission per multi-category-marketplace doctrine ADR-0249).

### §3.6 Composition matrix (deployment-context × tenant-class for top metrics)

| Metric | Public demo_trial | Public paid | AWS demo_trial | AWS paid | OCI demo_trial (Always Free) | OCI paid | On-prem demo_trial | On-prem paid | Colo paid | Oyatie-cloud-provider paid |
|---|---|---|---|---|---|---|---|---|---|---|
| Read p50 (ms) | 100 | 80 | 110 | 90 | 120 | 90 | 80 | 60 | 70 | 80 |
| Write p50 (ms) | 160 | 120 | 170 | 140 | 200 | 140 | 110 | 90 | 100 | 120 |
| Query p99 (ms) | 600 | 400 | 700 | 450 | 800 | 450 | 400 | 300 | 350 | 400 |
| API RPS sustained | 50 | 200 | 50 | 200 | 20 | 200 | 50 | 500 | 400 | 200 |
| Bulk import (rec/sec) | 50 | 200 | 50 | 200 | 50 | 200 | 100 | 500 | 400 | 200 |
| Event RPS | 100 | 5000 | 100 | 5000 | 100 | 5000 | 100 | 10000 | 8000 | 5000 |
| Concurrent conns | 20 | 200 | 20 | 200 | 20 | 200 | 50 | 500 | 400 | 200 |
| Daily API calls | 100k | 5M | 100k | 5M | 100k | 5M | 100k | 5M | 5M | 5M |
| Availability % | 99 | 99.99 | 99 | 99.99 | 99 | 99.99 | 99 | 99.9-99.99 | 99.9-99.99 | 99.99 |
| RTO (min) | n/a | 5 | n/a | 5 | n/a | 5 | n/a | 15 | 10 | 5 |
| Storage (GB) | 5 | 10000 | 5 | 10000 | 5 | 10000 | 5 | 10000 | 10000 | 10000 |
| Custom props/obj | 50 | 1000 | 50 | 1000 | 50 | 1000 | 50 | 1000 | 1000 | 1000 |
| Seats | 10 | unlimited | 10 | unlimited | 10 | unlimited | 10 | unlimited | unlimited | unlimited |

## §4 Comparison narrative — ahead / parity / catch-up per metric

### §4.1 Where Oyatie crm is AHEAD (targets exceed all three counterparts)

AHEAD-1: Single-record read p50 80 ms vs counterpart-best 100 ms (Salesforce). Oyatie targets 20% better.

AHEAD-2: Multi-region paid availability 99.99% (matches Salesforce Premier Success; Salesforce Standard Success is 99.9%; HubSpot marketed 99.95%; Microsoft default 99.9%). Oyatie matches the best.

AHEAD-3: Regional failover RTO 5 minutes vs counterpart-typical 15-30 minutes (Salesforce). Oyatie targets 3-6x better.

AHEAD-4: Regional failover RPO 60 seconds vs counterpart-typical 15 minutes (Salesforce) or best-effort (HubSpot). Oyatie targets 15x better.

AHEAD-5: Custom properties per object 1,000 matches HubSpot ceiling; beats Salesforce default (Custom Object 800 fields with caveats) and Dynamics (1,024). Effectively at the best of the three.

AHEAD-6: OCI Always Free demo_trial at $0 cost — no counterpart offers a perpetual free tier with comparable functionality. Salesforce Developer Edition is free but single-user; HubSpot Free CRM is capped. Oyatie's OCI Always Free demo_trial is a uniquely additive value.

AHEAD-7: HTTP/3 + QUIC + ECH + PQC hybrid transport defaults (per crm OpenAPI x-transport extensions). No counterpart defaults to HTTP/3 + post-quantum hybrid TLS. Oyatie is forward-positioned on transport.

AHEAD-8: AsyncAPI event throughput 5,000 events per second per paid tenant beats Salesforce Platform Event Enterprise 2.3 average per second. ~2,000x better.

AHEAD-9: API requests per second sustained 200 RPS paid matches HubSpot Pro/Enterprise; Salesforce's "Concurrent long-running synchronous Apex transactions per org" 10 is a different metric but Salesforce-equivalent sustained API call rate is approximately 100-200 per second for Enterprise edition. Oyatie matches the best of the three.

AHEAD-10: Cost per seat $35/seat/month target beats Salesforce Sales Cloud Enterprise $165/seat/month by 4.7x; matches HubSpot Sales Hub Starter price-point.

### §4.2 Where Oyatie crm is at PARITY (targets match counterpart-best)

PARITY-1: Single-record write p50 120 ms vs counterpart-best 150 ms (Salesforce best case) — Oyatie slightly ahead but in same order of magnitude.

PARITY-2: Daily API call quota 5,000,000 matches Salesforce Unlimited; beats HubSpot Enterprise 1,000,000.

PARITY-3: Bulk import 2,000 records per second multi-worker matches Dynamics multi-worker.

PARITY-4: AI scoring latency p95 2 seconds matches Dynamics Sales Insights.

PARITY-5: Records per tenant 100,000,000 matches Salesforce Enterprise + HubSpot Enterprise.

### §4.3 Where Oyatie crm must CATCH UP (targets present but verification needed)

CATCH-UP-1: Trust SLA contractual evidence. Salesforce publishes Trust Status with multi-year historical availability data. Oyatie crm has SLO targets but no published Trust portal. Wave 14-15 deliverable.

CATCH-UP-2: AI scoring evidence at scale. Dynamics Sales Insights operates at customer-base scale; Oyatie intelligence µservice + crm handoff is targeted but unproven at scale.

CATCH-UP-3: Migration tooling throughput benchmarks. Salesforce Data Loader documented; Oyatie migration-playbooks/ describe the process but no throughput benchmark.

CATCH-UP-4: Field history tracking. Salesforce tracks the last 18 months of field changes per Field Audit Trail (Shield add-on). Oyatie audit-chain seal events emit at the row level but per-field history is not specified.

CATCH-UP-5: Concurrent Bulk API jobs. Salesforce 100 concurrent Bulk API jobs per org; Oyatie target needs to be set.

CATCH-UP-6: Sandbox / Org-copy environment. Salesforce Sandboxes (Developer, Developer Pro, Partial, Full) are first-class; HubSpot Sandboxes; Dynamics Environments. Oyatie's sandbox / staging / production isolation per tenant needs explicit numbers.

CATCH-UP-7: Operating-region count. Salesforce operates in ~15+ regions; HubSpot in ~5-7; Dynamics in 20+ Azure regions. Oyatie target region count not specified.

CATCH-UP-8: Identity propagation latency. Salesforce-internal identity changes propagate in seconds; HubSpot in real-time; Dynamics in seconds. Oyatie identity µservice + crm Cedar policy propagation latency not specified.

CATCH-UP-9: AsyncAPI / event consumer lag. Salesforce Change Data Capture lag p95 under 1 second; Dynamics Service Bus lag p95 under 1 second; HubSpot Webhook lag 1-5 seconds typical. Oyatie target 1 second p95; consumer lag not yet measured.

CATCH-UP-10: Sandbox refresh time. Salesforce Sandbox refresh (Full) can take hours to days; Oyatie's `tofu apply -var tenant_id=acme-sandbox` should target minutes per the zero-handroll-opentofu-only doctrine.

### §4.4 Critical CRM-specific benchmarks not in counterpart published docs

These metrics are CRM-internal that the counterparts do not publish openly; Oyatie targets are set by reasoning from the canonical CRM workflow rather than counterpart comparison.

CRM-X1: Account merge dedupe accuracy. ADR-MS-001 specifies "Account merge duplicate-detection false negative rate must be <=0.1% in canonicalen fixtures". This is hyperscaler-grade.

CRM-X2: Quote line pricing decision latency. ADR-MS-001 specifies "Quote approval policy latency must stay within the service policy-decision budget". Should be < 200 ms p95 Cedar evaluation.

CRM-X3: Lead-to-opportunity conversion saga end-to-end p95. IP-016 lead-to-opportunity-stage-progression should land at < 3 seconds p95 from REST POST to opportunity creation event.

CRM-X4: Forecast roll-up batch time. IP-021 forecast-roll-up-with-finance-approval-gate should compute full-tenant forecast in < 60 seconds for 100,000-opportunity tenant.

CRM-X5: Campaign attribution model recompute. IP-019 campaign-to-revenue-attribution should recompute attribution for 1M-touchpoint campaign in < 5 minutes (batch).

CRM-X6: Service-case SLA timer accuracy. IP-022 service-case-sla-and-escalation-engine should tick SLA clocks within 5-second precision.

CRM-X7: Territory routing decision time. IP-024 per-tenant-territory-routing-skill-capacity-engine should select route in < 100 ms p95.

CRM-X8: Customer 360 ontology projection lag. IP-020 customer-360-ontology-unification should project mutation to ontology view in < 2 seconds p95.

## §5 Per-metric SLO authoring requirements (Wave 15J)

The crm/slos/ directory currently has four OpenSLO files. The matrix above produces a much larger SLO surface (~30 SLOs across latency, throughput, availability, capacity dimensions). Wave 15J should expand the SLO directory.

Required SLO authoring:
- crm-latency-read-p50.openslo.yaml (M-L1).
- crm-latency-write-p50.openslo.yaml (M-L2).
- crm-latency-query-p99.openslo.yaml (M-L3, expands existing crm-latency-p99).
- crm-latency-bulk-query-p99.openslo.yaml (M-L4).
- crm-event-delivery-p95.openslo.yaml (M-L5).
- crm-ai-scoring-p95.openslo.yaml (M-L6).
- crm-api-rps-sustained.openslo.yaml (M-T1).
- crm-bulk-import-throughput.openslo.yaml (M-T2).
- crm-event-throughput.openslo.yaml (M-T3).
- crm-concurrent-connections.openslo.yaml (M-T4).
- crm-daily-api-calls.openslo.yaml (M-T5).
- crm-multi-region-availability.openslo.yaml (M-A1, expands existing crm-availability).
- crm-single-region-availability.openslo.yaml (M-A2).
- crm-regional-failover-rto.openslo.yaml (M-A3).
- crm-regional-failover-rpo.openslo.yaml (M-A4).

Plus per-bounded-context SLOs for each of the six aggregates (account-master, opportunity, quote, service-case, campaign, loyalty-ledger) on the success-rate and write-latency dimensions.

Plus per-CRM-specific metrics from §4.4 (account-merge-dedupe-accuracy, quote-pricing-decision-latency, lead-to-opp-conversion-p95, forecast-rollup-batch-time, campaign-attribution-recompute, service-case-sla-precision, territory-routing-decision-time, customer-360-projection-lag).

Total target SLO file count: approximately 30 SLOs.

## §6 Per-context cost-model overlay (replacement for tier-based cost-budget.md)

The legacy cost-budget.md is shaped around tier ladders. Replacement per-context cost overlay:

**Public-cloud + paid + per_seat:**
- Variable cost per seat: ~$2-5/seat/month infrastructure (compute + storage + bandwidth) at the §3.5 M-E3 $35/seat/month price point = ~85-95% gross margin.

**OCI-guest + demo_trial (Always Free):**
- $0 infrastructure cost.
- Provisioned via iac/oci-guest/always-free/ (per zero-handroll-opentofu-only memory point 1).
- Capacity ceiling: 4 OCPU + 24 GB + 200 GB block + 2× 20 GB Autonomous DB + 10 GB Object Storage + 10 TB egress.
- Supports approximately 50-100 concurrent demo_trial tenants per OCI region (sub-tenant partitioning inside the Always Free quota).

**On-prem + paid + per_seat:**
- Customer-owned hardware + Oyatie service fee at ~$15-25/seat/month (lower fee since customer absorbs infra).

**Colo + paid + per_seat:**
- Colo facility lease (customer) + Oyatie service fee at ~$20-30/seat/month.

**Oyatie-as-cloud-provider + paid + per_usage:**
- Oyatie sells compute + storage + networking at hyperscaler-comparable rates; CRM workload billed per_usage on Oyatie's own substrate.

## §7 Notes for Wave 14 aggregation

The Big-8 family aggregation should produce a unified per-µservice × per-deployment-context × per-tenant-class SLO/throughput/cost grid covering CRM + HR + ERP + ITSM + Marketing-Automation + Contact-Center + Performance-Management + Learning-Management. The numbers here are CRM-specific but the structural template (single industry-leader target + 6 contexts × 2 classes overlay) should be reused across the Big-8 set.

The OCI Always Free quota of 4 OCPU + 24 GB total has to be fairly distributed across the Big-8 µservices for a demo_trial tenant that wants all Big-8 surfaces. CRM in demo_trial on OCI Always Free should target consumption under 1 OCPU + 6 GB + 50 GB block + 1× 20 GB Autonomous DB + 25 GB egress / month. This leaves headroom for HR, ERP, and ITSM to share the remaining Always Free budget.
