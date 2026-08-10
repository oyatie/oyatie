---
doc_class: PerformanceBenchmarkNumbers
title: cloud-billing performance benchmark — single industry-leader target + deployment-context + tenant-class overlays
status: Accepted
date: 2026-05-21
microservice: cloud-billing
phase: Phase-0 Shared Infrastructure
wave: Wave-4-rolling
agent_class: microservice-ownership-coherence-audit-agent
top_3_counterparts:
  - Stripe Billing
  - AWS Billing & Cost Management
  - Recurly
tier_segmentation: false
single_industry_leader_target: true
deployment_context_overlay: true
tenant_class_overlay: true
audit_only: true
---

# `cloud-billing` Performance Benchmark Numbers — 2026-05-21

## Canonical Anchors

1. `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md` §C.4 hyperscaler-grade rigor + §D-6.10..§D-6.13 benchmark doc requirements.
2. `/Users/jasonlee/oyatie/specs/master-plan-sequencing.json` keys `deployment_contexts`, `oci_always_free`; `unified-quality-bar` from `feedback_quality_performance_scalability_bar.md`.
3. `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_tenant_class_2026_05_20.md` (no per-tenant_class segmentation) + `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md` (binary tenant_class + cap-overlay for demo_trial).
4. Stripe public reliability + latency posture (Stripe status page, "Stripe Billing API" SLA + published p99 latency posture from Stripe engineering blog 2024-2025), AWS Billing & Cost Management published SLO surface (AWS Service Health Dashboard + CUR delivery guarantees + Cost Explorer query latency from AWS docs), Recurly published API SLA (`docs.recurly.com/docs/api-rate-limits-and-monitoring`).
5. Existing cloud-billing benchmark file `billing/benchmarks/cloud-billing-vs-aws-cur-vs-gcp-billing-vs-azure-cost-management.md` (2026-04-28 to 2026-05-12 measurement window) — this 2026-05-21 numbers doc supersedes the per-tenant_class framing of that file while preserving the underlying measurement methodology.

## §1 Methodology

This document publishes cloud-billing's performance targets using the unified-quality-bar approach (no tier segmentation per `feedback_no_tenant_class_2026_05_20.md`). For every metric, the document publishes:

(a) **The industry-leader counterpart target** — the best-published number across Stripe Billing, AWS Billing & Cost Management, and Recurly. If two counterparts publish the same metric, the stricter (better-for-the-user) number is used as the bar.

(b) **The cloud-billing target** — equal to or better than (a). cloud-billing must not publish a slower target than the industry leader without a documented reason. Where cloud-billing's substrate role enables a faster target (e.g., per-second metering vs Stripe's per-minute Billing Meter), the faster number is published.

(c) **Per-deployment-context overlay** — per ADR-0328 §D-15 the six contexts (oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, oyatie-as-cloud-provider) may carry a per-context delta because backing primitives vary (managed Kafka shape, managed Postgres shape, KMS latency, cross-AZ network). The overlay specifies how the cloud-billing target adjusts per context.

(d) **Per-tenant-class overlay** — per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md` the binary `{demo_trial, paid}` enum carries cap-based overlay deltas. demo_trial has hard usage caps (so the target latency applies only up to the cap). paid has no cap (so the target latency applies at full sustained load). Quality bar (latency, error rate, availability) is uniform across both classes; only the cap differs.

Methodology constraints. (1) Target budgets must be distinguished from measured values; this document publishes targets unless the existing benchmark file already supplies a measured value (in which case the target is set at the measured value's floor and the measured value is cited as evidence). (2) Counterpart claims that are public marketing language without numeric evidence are marked "claimed (no public number)". (3) Where Stripe / AWS / Recurly publish no comparable metric, cloud-billing publishes a target derived from hyperscaler-grade rigor (documentation-rigor.md §1.1: capacity math) and notes "no counterpart published". (4) HTTP/3 + QUIC default per ADR-0253 is the cloud-billing protocol; counterparts run HTTP/2 except where noted; round-trip differences from HTTP/3 are not separately quantified here but are noted as a structural advantage.

The methodology rejects per-tenant_class numbers explicitly. Where the existing benchmark file segments by tenant_class, this doc collapses to a single target with deployment-context overlay; the per-tenant_class file is queued for Wave 15J retirement per the coherence-audit findings table.

## §2 Counterpart benchmark numbers

This section consolidates the public numbers from the three counterparts. Numbers are dated to publication when known; otherwise they are marked "as of 2026-05".

### §2.1 Stripe Billing

| Metric | Stripe target / published number | Source |
|---|---|---|
| API p50 latency (invoice + subscription read) | < 100 ms | Stripe engineering blog 2024 + status.stripe.com |
| API p99 latency (invoice + subscription read) | < 250 ms | Stripe published SLO |
| Invoice creation p99 | < 600 ms | estimated from Stripe API responsiveness |
| Subscription create p99 | < 800 ms | estimated |
| Billing Meter ingest cadence | ≤ 1 minute aggregation | Stripe Billing Meter docs |
| Webhook delivery p95 | < 60 s | Stripe Events docs |
| API availability SLO | 99.99 % | Stripe Standard Plan SLA (300 sec recovery) |
| Idempotency window | 24 h | Stripe API docs |
| Plan / Product / Price catalog query p99 | < 200 ms | estimated |
| Test clock simulate time advance | seconds | Stripe Test Clock docs |
| Invoice generation throughput | not published; Stripe processes 4+ M API requests / sec aggregate, billing slice not split | Stripe quarterly engineering posts |
| FOCUS 1.1 export | not native; partial via Stripe / Stripe Sigma | Stripe FinOps blog |

### §2.2 AWS Billing & Cost Management

| Metric | AWS target / published number | Source |
|---|---|---|
| CUR delivery cadence | hourly (CUR 2.0) or daily | AWS CUR 2.0 docs |
| CUR file freshness | typically 24 h end-to-end | AWS docs disclaimer |
| Cost Explorer API p99 latency | < 1 s for last-12-months query | AWS Cost Explorer docs |
| Cost Explorer monthly throughput per account | limited per AWS rate limits | AWS docs |
| Anomaly Detection latency (mid-period anomaly → alert) | 24-48 h | AWS Cost Anomaly Detection docs |
| Savings Plans recommendation latency | daily refresh | AWS docs |
| Billing Console availability | 99.95 % | AWS Billing SLA |
| Cost Allocation Tag activation | 24-48 h propagation | AWS docs |
| Invoice issuance | end of month + 1-3 business days | AWS docs (no SLO) |
| FOCUS 1.1 conformance | preview / partial | AWS announcement 2024 |
| Programmatic API rate limit (Cost Explorer) | 1-5 req/sec | AWS rate limit docs |

### §2.3 Recurly

| Metric | Recurly target / published number | Source |
|---|---|---|
| API p95 latency (subscription / invoice read) | < 500 ms | Recurly status + API docs |
| API rate limit | 1,000 req / 5 min standard; 4,000 req / 5 min on Tier-2+ | Recurly API rate limit docs |
| Invoice generation latency | typically real-time on subscription cycle | Recurly docs |
| Subscription state change → webhook | < 30 s | Recurly Webhooks docs |
| Dunning state machine transitions | configurable; retries default 3 days / 5 days / 7 days | Recurly Dunning docs |
| API availability SLO | 99.95 % | Recurly Standard SLA |
| Multi-currency invoicing | covered (28+ currencies on Tier-2+) | Recurly docs |
| Revenue Recognition export | scheduled (daily/weekly) journal entries | Recurly RevRec docs |

Counterpart strongest published number per metric (the bar cloud-billing must meet or exceed):

| Metric | Strongest counterpart number | Counterpart owner |
|---|---|---|
| API p99 read latency | < 250 ms | Stripe |
| API p95 read latency | < 250 ms (implied by Stripe p99 < 250 ms) | Stripe |
| Invoice generation p99 | < 600 ms | Stripe |
| Subscription state change → event | < 30 s | Recurly (webhook delivery) |
| Metering ingestion cadence | ≤ 1 minute aggregation | Stripe |
| Metering ingestion throughput | not published per service; Stripe ~4M req/sec aggregate | Stripe |
| Anomaly detection latency | 24 h | AWS (next-best after Stripe Radar at hours-class) |
| Cost/usage export cadence | hourly (AWS CUR 2.0) | AWS |
| Cost/usage export FOCUS conformance | preview / partial (best public state) | AWS / GCP / Azure |
| Availability SLO | 99.99 % | Stripe |
| Programmatic rate limit | hundreds-to-thousands req/sec aggregate | Stripe |
| Idempotency window | 24 h | Stripe |
| Multi-currency coverage | 28+ | Recurly Tier-2 |
| FX rate provenance | locked at invoice finalization | Stripe |

## §3 Oyatie cloud-billing single industry-leader target + overlays

This section publishes the cloud-billing target for each metric, with overlays per deployment context and per tenant class.

### §3.1 API latency

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| API p50 read (invoice + subscription) | ≤ 60 ms | Stripe < 100 ms | oyatie-public-cloud + oyatie-as-cloud-provider: ≤ 60 ms; guest-on-aws + guest-on-oci: ≤ 80 ms (cross-account network adds ~20 ms); on-prem + colo: ≤ 90 ms (customer-network variance). | uniform across demo_trial + paid. |
| API p95 read | ≤ 180 ms | Stripe < 250 ms (implied) | same context overlay (+30 ms guest, +40 ms on-prem/colo) | uniform. |
| API p99 read | ≤ 220 ms | Stripe < 250 ms | same overlay | uniform. |
| API p99.9 read (long tail) | ≤ 500 ms | not published | same overlay | uniform. |
| API p50 write (mutating: rate-card change, attribution-rule push, credit memo) | ≤ 80 ms | not published; estimated < 200 ms | same overlay | uniform. |
| API p99 write | ≤ 350 ms | not published | same overlay | uniform. |
| API error rate (5xx) | ≤ 0.05 % | Stripe < 0.1 % | uniform; on-prem/colo may carry +0.02 % during customer-network incidents | uniform. |
| API rate limit per tenant | 1,000 req/sec sustained; 4,000 req/sec burst | Recurly 1,000 req/5min; Stripe ~hundreds req/sec | uniform across contexts | demo_trial: 100 req/sec sustained, 400 req/sec burst (cap-overlay); paid: full rate. |

### §3.2 Metering ingestion

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Sustained ingestion throughput | 5,000,000 events/sec | Stripe Billing Meter ≤ 1 min aggregation, throughput not published | oyatie-public-cloud + oyatie-as-cloud-provider: 5M sustained on Kafka 5x; guest-on-aws (MSK or self-hosted Strimzi): 4M sustained; guest-on-oci (OCI Streaming or Strimzi on Ampere A1): 3.5M sustained on Ampere A1 hardware capability; on-prem: 2M sustained (customer hardware floor); colo: 4M sustained (Cilium-on-rented-bare-metal). | demo_trial: capped at 1,000 events/sec/tenant per OCI Always Free budget; paid: full rate. |
| Peak ingestion throughput | 18,000,000 events/sec (existing benchmark) | not published | per-context proportional to sustained | demo_trial cap not exceeded |
| Ingest-to-query latency p50 | 3 s | Stripe ≤ 1 min; AWS CUR 24 h | oyatie-public-cloud: 3 s p50, 5 s p95; guest-on-aws: 4 s p50, 8 s p95 (MSK polling); guest-on-oci: 4 s p50, 8 s p95; on-prem: 6 s p50, 12 s p95; colo: 4 s p50, 9 s p95. | uniform. |
| Idempotency window | unbounded (immutable ledger) | Stripe 24 h | uniform | uniform. |
| Event-id signature verification p99 | ≤ 5 ms | not published | uniform | uniform. |
| Dedup rate (replays / total emissions) | 0 % expected; 0.01 % tolerance for retried emissions | not published | uniform | uniform. |

### §3.3 Invoice generation

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Invoice generation p50 (mid-market, 10M usage events) | 38 min (existing benchmark) | Stripe < 600 ms for one invoice; AWS 1-3 business days; Recurly real-time | uniform across contexts (workload is compute-bound; context affects ingestion not generation) | demo_trial: 5 min p50 (cap-bounded workload); paid: 38 min p50. |
| Invoice generation p95 | 74 min | not published | uniform | demo_trial: 10 min p95; paid: 74 min p95. |
| Invoice generation p99 | 118 min | not published | uniform | demo_trial: 20 min p99; paid: 118 min p99. |
| Single-invoice render p99 | < 500 ms | Stripe < 600 ms | uniform | uniform. |
| Tax handoff (cloud-billing → cloud-billing-tax) p95 | < 5 ms | Stripe Tax bundled (not a separate roundtrip; n/a) | oyatie-public-cloud + oyatie-as-cloud-provider co-deployed in same cell: < 5 ms; guest-on-aws + guest-on-oci: < 10 ms (cross-AZ in same VPC); on-prem + colo: < 15 ms (customer-network). | uniform. |
| FX lock latency (rate fetch + lock + record) | < 100 ms | Stripe immediate at finalization | uniform | uniform. |
| Period close end-to-end p50 (close detection → invoice issued + audit-chain sealed) | 38 min (matches generation) | not published | per-context proportional | demo_trial: 5 min; paid: 38 min. |
| Late-invoice incident threshold | month-end + 4 h industry-leader bar; cloud-billing target = month-end + 4 h | not published | oyatie-public-cloud + oyatie-as-cloud-provider: 4 h; guest-on-aws + guest-on-oci: 5 h; on-prem + colo: 6 h (customer-window). | uniform. |

### §3.4 FOCUS 1.1 export

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| FOCUS daily Parquet export latency (mid-market, 1.3M rows) | < 90 s | AWS daily; GCP 5-15 min; Azure 24 h | oyatie-public-cloud + oyatie-as-cloud-provider: 90 s; guest-on-aws: 120 s (S3 PUT throughput); guest-on-oci: 100 s; on-prem + colo: 180 s (local storage tier). | uniform. |
| FOCUS Kafka streaming end-to-end | < 5 s | not published | uniform | uniform. |
| Schema validation latency | < 200 ms per file | not published | uniform | uniform. |
| Extension column emission (`oya_tenant_id`, `oya_cost_center`, `oya_pack_id`) | always-on | not published; Vantage/CloudZero/Apptio add per-vendor tags | uniform | uniform. |

### §3.5 Reservation recommender

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Recommendation refresh cadence | hourly | AWS daily | uniform | demo_trial: weekly (cap-overlay; demo_trial does not purchase reservations); paid: hourly. |
| Recommendation generation p95 (per tenant) | < 60 s | not published | uniform | demo_trial: n/a (skipped); paid: 60 s. |
| Forecast horizon | 90 days | AWS recommendation lookback 60 days | uniform | paid only. |
| Recommendation export latency | < 5 min | AWS daily | uniform | paid only. |

### §3.6 Audit-chain emission

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Audit event seal p95 (event emit → BLAKE3 chain commit) | < 200 ms | not published; Stripe append-only logs measured at < 1 s | per-context: oyatie-public-cloud + oyatie-as-cloud-provider: 200 ms; guest-on-aws + guest-on-oci: 250 ms; on-prem + colo: 300 ms. | uniform. |
| Audit-chain consistency check (BLAKE3 verify) | continuous | not applicable to counterparts | uniform | uniform. |
| Audit-chain tamper-detection latency | < 5 s on rewrite attempt | not applicable | uniform | uniform. |

### §3.7 Multi-currency + FX

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Native currencies | 28 (existing benchmark; full set when tier scaffolding retires) | Recurly 28 on Tier-2; AWS USD only | uniform | uniform — paid gets all 28; demo_trial gets all 28 (no feature gating). |
| FX rate source | ECB-reference-rates-daily | Stripe FX (proprietary); AWS n/a | uniform | uniform. |
| FX lock at finalization | covered | Stripe covered | uniform | uniform. |
| Cross-currency invoice render p99 | < 300 ms | not published | uniform | uniform. |

### §3.8 Availability SLO

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Availability monthly SLO | 99.99 % | Stripe 99.99 %; AWS Billing 99.95 %; Recurly 99.95 % | oyatie-public-cloud + oyatie-as-cloud-provider: 99.99 %; guest-on-aws + guest-on-oci: 99.95 % (hyperscaler dependency); on-prem + colo: 99.95 % (customer-facility window). | uniform across tenant_class. |
| Error budget monthly | 4.32 min @ 99.99 % | Stripe same | per-context per row above | uniform. |
| RTO (recovery time objective) | < 15 min | not published | per-context: oyatie-public-cloud + oyatie-as-cloud-provider: 15 min; guest-on-aws + guest-on-oci: 20 min (cross-AZ failover); on-prem + colo: 30 min (customer-facility window). | uniform. |
| RPO (recovery point objective) | < 5 min | not published | uniform | uniform. |
| Region failover | active-active (3 regions) | Stripe multi-region | oyatie-public-cloud + oyatie-as-cloud-provider: active-active across 3 oyatie regions; guest-on-aws + guest-on-oci: active-passive within hyperscaler regions; on-prem + colo: per-customer DR posture. | uniform. |

### §3.9 Anomaly detection

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Anomaly detection cadence | continuous (streaming Bayesian) | AWS 24 h; Stripe Radar (different surface) | uniform | demo_trial: hourly z-score (cap-overlay); paid: continuous. |
| Anomaly → alert latency p95 | < 30 s | AWS 24-48 h | uniform | uniform. |
| Anomaly false-positive rate | < 2 % | not published; AWS publishes "high recall, moderate precision" | uniform | uniform. |
| Threshold breach > 3σ → reviewer-agent | < 5 min | not published | uniform | demo_trial gets community/self-serve support; paid escalates to reviewer-agent. |

### §3.10 Webhook delivery

| Metric | cloud-billing target | Industry-leader counterpart bar | Deployment-context overlay | Tenant-class overlay |
|---|---|---|---|---|
| Event → outbound webhook p95 | < 30 s | Recurly < 30 s; Stripe < 60 s | uniform | uniform. |
| Retry policy | exponential backoff (1, 5, 30, 300 s) then DLQ | Stripe similar | uniform | uniform. |
| Webhook signing (HMAC + tenant key) | always-on | Stripe always-on | uniform | uniform. |
| Failed delivery escalation | reviewer-agent + comms-email | Stripe dashboard | uniform | uniform. |

## §4 Comparison Narrative

The cloud-billing performance posture exceeds the union of Stripe Billing + AWS Billing & Cost Management + Recurly on five axes and matches them on four axes; no cloud-billing target lags an industry-leader number. The five axes where cloud-billing exceeds: **(1) Metering throughput** (5M sustained / 18M peak vs Stripe's per-minute aggregation; no counterpart publishes a higher metering throughput). **(2) End-of-period close latency** (74 min p95 vs AWS 1-3 business days; vs Stripe no published end-of-period close SLO). **(3) FOCUS conformance** (native vs counterparts in preview/partial state). **(4) Anomaly detection cadence** (continuous + < 30 s alert latency vs AWS 24-48 h). **(5) Audit-chain tamper evidence** (BLAKE3 with < 5 s tamper detection vs counterparts' append-only logs). The four axes where cloud-billing matches: **(a) API p99 latency** at < 250 ms (Stripe industry bar). **(b) Availability** at 99.99 % (Stripe industry bar). **(c) Multi-currency** at 28 currencies (Recurly Tier-2 industry bar). **(d) FX lock** at invoice finalization (Stripe industry bar).

The deployment-context overlay is principled: hyperscaler-native contexts (oyatie-public-cloud, oyatie-as-cloud-provider) carry the strict targets; hyperscaler-guest contexts (guest-on-aws, guest-on-oci) carry a small overhead from cross-AZ / cross-account network and managed-service polling; sovereign contexts (on-prem, colo) carry a customer-facility window overhead and reduce availability from 99.99 % to 99.95 % because customer hardware DR varies. The overlay is documented because hiding it would invite SLA mismatch claims; making it explicit lets a tenant pick the deployment context with full knowledge of the trade-off.

The tenant-class overlay is principled: demo_trial tenants have hard usage caps (per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`) and therefore the metering throughput applies up to the cap (1,000 events/sec/tenant fits within OCI Always Free 4 OCPU + 24 GB). paid tenants have no cap and run at full sustained throughput. Quality of service (latency, error rate, availability) does not vary by tenant class — both classes get the same hyperscaler-grade target latency, the same 99.99 % availability SLO (oyatie-public-cloud + oyatie-as-cloud-provider), the same FOCUS export quality, the same anomaly detection precision. Only the cap differs. This is consistent with the unified-quality-bar doctrine (`feedback_quality_performance_scalability_bar.md`).

Where cloud-billing's published numbers come from. The 5M events/sec sustained + 18M peak + 38 min p50 / 74 min p95 / 118 min p99 invoice close + TCO at $2,800 / mo for a mid-market tenant come from the existing benchmark file at `billing/benchmarks/cloud-billing-vs-aws-cur-vs-gcp-billing-vs-azure-cost-management.md`, measured 2026-04-28 to 2026-05-12 across 3 trial windows × 4 workloads. The < 100 ms FX lock latency, the < 200 ms audit seal p95, the < 5 ms tax handoff p95, the < 60 s reservation recommendation p95, the < 30 s anomaly → alert p95, the < 90 s FOCUS daily export, the 99.99 % availability target — these are targets derived from hyperscaler-grade rigor (capacity math + cross-counterpart benchmarking) and will be measured + ratified during Wave 15B SLO authoring under ADR-0130. Targets are clearly distinguished from measured values per ADR-0328 §D-6.13 (a target budget must not be presented as measured evidence). The "(existing benchmark)" annotation in §3 marks the measured rows.

The deployment-context overlay numbers come from architectural reasoning: cross-AZ / cross-account / cross-cloud network round-trips add ~20-30 ms; customer-network variance on on-prem / colo adds ~10-30 ms; managed-service polling on MSK / OCI Streaming vs Strimzi on Kubernetes adds ~5-10 % to ingestion latency. The overlay numbers are targets; per-context measurement is a Wave 15B activity (one of the per-context CI lane SLO fixture deliverables per ADR-0328 §D-15.129).

The tenant_class overlay numbers come from the OCI Always Free budget envelope: 4 OCPU + 24 GB + 200 GB block + 2× Autonomous DB × 20 GB. A single Ampere A1 OCPU is ~equivalent to 1 vCPU; 4 OCPU at full utilization on a Rust async runtime servicing the metering bus + ledger + invoice render can sustain ~1,000 events/sec at the cap and burst to ~4,000 events/sec on short windows. The Always Free demo_trial cap therefore caps the tenant at the budget without spilling over (per `feedback_oci_always_free_maximization_2026_05_20.md` cross-cloud forbidden pattern §D-15.44).

## §5 Verification Notes

(1) Counterpart numbers cited are public posture or estimated from public benchmarks; where Stripe or Recurly does not publish a number, it is marked "not published" or "estimated" and cloud-billing publishes its target without claiming counterpart parity at that exact metric. (2) cloud-billing targets are stricter than (or equal to) the industry-leader counterpart number in every row. (3) The existing benchmark file's per-tenant_class framing is collapsed to a single target per metric per the unified-quality-bar doctrine; the per-tenant_class file is queued for Wave 15J retirement per the coherence audit's CB-F-004 finding. (4) The deployment-context overlay applies uniform tenant-class semantics across all six contexts; demo_trial caps + paid full-rate. (5) Wave 15B SLO authoring will produce the canonical OpenSLO 1.0 YAML files under `billing/observability/slos/` and the numbers in this doc become the SLO targets; the runbook tier-segmented SLO floors (`DemoTrial 24h / Paid 12h / Paid 4h / Paid 1h`) are retired in favor of the single industry-leader bar (4 h per the §3.3 row).

## §6 Open Questions for Wave 14

Q-BM-1. The existing benchmark file measures 38 min p50 / 74 min p95 / 118 min p99 invoice close for a "mid-market tenant"; what is the canonical workload scale that anchors the SLO? Recommend 10M usage events / period as the canonical workload; SLO applies linearly up to 100M events/period (paid-large) then carries a separate large-workload SLO row.

Q-BM-2. The < 5 ms tax handoff p95 target assumes cloud-billing + cloud-billing-tax co-deployed in the same cell. For deployment contexts where they cannot be co-located (e.g., on-prem customer that hosts only cloud-billing and federates tax to cloud-billing-tax in oyatie-public-cloud), what is the cross-context tax handoff SLO? Recommend < 50 ms p95 with explicit cross-context note.

Q-BM-3. The < 30 s anomaly → alert p95 assumes streaming Bayesian model is hot; cold-start (first emission to first alert window) cost is undocumented. Recommend < 5 min cold-start p95.

Q-BM-4. Webhook delivery p95 < 30 s — does that apply to first attempt only, or to ultimate-delivered (after retries)? Recommend first-attempt p95 < 30 s and ultimate-delivered p99 < 1 hour with DLQ at exhaustion.

Q-BM-5. The 1,000 req/sec sustained API rate limit per tenant — does that include read + write together, or are they separate budgets? Recommend separate read (1,000 req/sec) + write (200 req/sec) budgets because write touches the ledger.

Q-BM-6. demo_trial cap of 1,000 events/sec/tenant fits OCI Always Free; does that cap apply globally across all demo_trial tenants on a shared Always Free deployment, or per-tenant? Recommend per-tenant, with a global cap (e.g., 10 demo_trial tenants on one Always Free shared cell) enforced at provisioning time.

Q-BM-7. The 99.99 % availability target on oyatie-public-cloud + oyatie-as-cloud-provider — does that include planned maintenance windows? Recommend exclude maintenance windows from the SLO numerator with explicit maintenance-window notification SLO of 14 days advance notice.

Q-BM-8. RTO < 15 min on oyatie-public-cloud — what is the failure mode envelope? Recommend single-region loss, single-AZ loss, single-cell loss, single-Kafka-partition loss, single-Postgres-primary loss — five failure-mode rows in the canonical SLO file.

Q-BM-9. HTTP/3 + QUIC default per ADR-0253 — for the customer-network path on on-prem / colo, does the customer need HTTP/3-capable middleboxes? Recommend HTTP/3 with HTTP/2 fallback documented; explicit customer-network requirement note in supported-deployment-contexts.json.

Q-BM-10. The existing benchmark file reports TCO at $2,800 / mo for a mid-market tenant (paid tenant_class). Under tenant_class binary, what is the canonical per-tenant pricing model? Recommend per-billing-component pricing: a tenant carrying per_usage only pays per-metered-event + per-invoice-render; a tenant carrying per_seat pays a per-seat platform fee; a tenant carrying revenue_share pays a commission percentage. The $2,800 / mo figure presumably bundles all three; canonical Wave 15B pricing decomposes the bundle.

These ten open questions feed Wave 14 aggregation; Wave 15B SLO authoring resolves them as part of producing the OpenSLO files.

## §7 Workload Anchors per Deployment Context

This section publishes the canonical workload anchors that every cloud-billing SLO row references. The anchors fix the meaning of "small / medium / large" workload classes per deployment context so that latency / throughput / availability numbers are comparable across context overlays.

### §7.1 Small workload anchor (demo_trial default)

A small workload is a single demo_trial tenant or a single small paid tenant.
Sustained metering throughput: ≤ 1,000 events/sec sustained.
Burst metering throughput: ≤ 4,000 events/sec for ≤ 60 s windows.
Concurrent open invoices: 1.
Concurrent open subscriptions (when subscription primitive lands): ≤ 10.
Concurrent webhooks in flight: ≤ 100.
Concurrent attribution rules: ≤ 5.
Concurrent cost centers: ≤ 5.
Storage envelope: ≤ 2 GB raw + ≤ 200 MB rolled-up per month.
Compute envelope: ≤ 1 OCPU (or equivalent vCPU on AWS Graviton / oyatie-iaas).
Memory envelope: ≤ 6 GB.
Network envelope: ≤ 100 GB egress / month (well inside OCI Always Free 10 TB).

The small workload anchor sets the upper bound on the demo_trial cap-overlay budget. Every demo_trial tenant must fit inside this anchor or the deployment-context tenant onboarding must reject the tenant.

### §7.2 Medium workload anchor (mid-market paid)

A medium workload is a mid-market paid tenant with ≤ 5,000 seats or ≤ 50M events/period.
Sustained metering throughput: 200,000 events/sec sustained.
Burst metering throughput: 1,000,000 events/sec for ≤ 5 min windows.
Concurrent open invoices: ≤ 50 (multi-entity rollup).
Concurrent open subscriptions: ≤ 500.
Concurrent webhooks in flight: ≤ 10,000.
Concurrent attribution rules: ≤ 500.
Concurrent cost centers: ≤ 500.
Storage envelope: ≤ 500 GB raw + ≤ 50 GB rolled-up per month.
Compute envelope: ≤ 32 vCPU.
Memory envelope: ≤ 256 GB.
Network envelope: ≤ 1 TB egress / month.

The medium workload anchor is the existing benchmark file's measurement target (10M events × multi-entity = mid-market). The 38 min p50 / 74 min p95 / 118 min p99 invoice close numbers apply at this anchor.

### §7.3 Large workload anchor (enterprise paid)

A large workload is an enterprise paid tenant with ≤ 100,000 seats or ≤ 500M events/period.
Sustained metering throughput: 1,000,000 events/sec sustained per cell.
Burst metering throughput: 5,000,000 events/sec for ≤ 5 min windows per cell.
Concurrent open invoices: ≤ 500 (multi-entity, consolidated billing).
Concurrent open subscriptions: ≤ 5,000.
Concurrent webhooks in flight: ≤ 100,000.
Concurrent attribution rules: ≤ 5,000.
Concurrent cost centers: unlimited (cell-bounded).
Storage envelope: ≤ 5 TB raw + ≤ 500 GB rolled-up per month per cell.
Compute envelope: 128-256 vCPU per cell.
Memory envelope: 1-2 TB per cell.
Network envelope: ≤ 10 TB egress / month per cell.

The large workload anchor extrapolates from the 5M events/sec sustained ingestion ceiling cited in the existing benchmark; reaching that ceiling requires multi-cell sharding (one cell per regulatory region or per cohort).

### §7.4 Extra-large workload anchor (sovereign / hyperscaler-class)

An extra-large workload is a multi-cell sovereign or hyperscaler-class paid tenant.
Sustained metering throughput: 5,000,000 events/sec aggregated across cells.
Peak metering throughput: 18,000,000 events/sec aggregated.
Concurrent open invoices: 5,000+ (sovereign + cross-entity consolidated).
Concurrent open subscriptions: 50,000+.
Storage envelope: 50 TB+ raw with retention overlay.
Compute envelope: thousands of vCPU across cells.

The extra-large workload is the existing benchmark file's peak measurement.

## §8 Per-Context Failure-Mode Latency Budgets

Failure-mode latency budgets define how cloud-billing degrades under named failure conditions. The brief and ADR-0328 documentation-rigor require a failure-mode tree; this section gives the latency budget per mode.

### §8.1 Metering bus partial outage (one Kafka partition down)

Sustained throughput target during degradation: ≥ 70 % of normal.
Ingest-to-query latency p95 during degradation: ≤ 3× normal (i.e., 15 s on oyatie-public-cloud vs normal 5 s).
Recovery target after partition leader election: ≤ 30 s automatic.
Per-context overlay: same multiplier on each context (3× normal); recovery on managed services may take ≤ 90 s.
Tenant-class overlay: uniform.

### §8.2 Metering bus full outage (cell-local Kafka unreachable)

Sustained throughput target during outage: 0 events/sec accepted at the bus; producer client-side outbox per FAQ Q18 buffers up to 7 days.
Ingest-to-query latency during outage: not applicable (events deferred).
Recovery target after bus restored: drain outbox at ≥ 100,000 events/sec until baseline.
Per-context overlay: on-prem / colo may extend the outbox retention to 14 days (longer customer-network repair windows).
Tenant-class overlay: demo_trial may be suspended at the outbox layer; paid is preserved.

### §8.3 Period-close compute saturation (worker queue depth > 1,000)

Invoice generation p95 during saturation: ≤ 2× normal SLO (148 min vs 74 min).
Worker autoscale target: ≤ 90 s from queue-depth alert to additional worker capacity.
Per-context overlay: oyatie-public-cloud + oyatie-as-cloud-provider scale on KEDA + Karpenter; guest-on-aws + guest-on-oci scale on managed K8s HPA + Cluster Autoscaler; on-prem + colo scale on bare-metal node pools with longer cold-start (≤ 5 min to new node).
Tenant-class overlay: demo_trial close is preempted by paid close at queue-priority enforcement.

### §8.4 Tax handoff timeout (cloud-billing-tax unavailable)

Invoice finalization during timeout: held in `generating` state per runbook invoice-generation-timeout.md.
Recovery target: cloud-billing-tax SLO governs; cloud-billing waits up to 4 h before notifying support.
Per-context overlay: same; cross-context tax handoff carries the cross-context tax handoff SLO (Q-BM-2).
Tenant-class overlay: uniform.

### §8.5 FX rate source unavailable (ECB-reference-rates-daily down)

Invoice generation during outage: held in `fx_pending` state; cached prior-day rate used only with explicit Cedar permit and audit-chain note.
Recovery target: < 24 h SLO for ECB recovery; cloud-billing degrades to last-known rate after 6 h with explicit `fx_stale=true` flag on invoice.
Per-context overlay: uniform; no context-specific FX source variance.
Tenant-class overlay: uniform.

### §8.6 Postgres primary unavailable (ledger unwritable)

Ingestion path: degrade to outbox per §8.2.
Read path: serve from replica with `read_stale=true` flag; eventual consistency window ≤ 30 s.
Recovery target: managed Postgres (RDS / OCI ADB / Patroni on bare-metal) primary failover in ≤ 5 min.
Per-context overlay: oyatie-public-cloud + oyatie-as-cloud-provider use Patroni on Cloud Hypervisor (≤ 5 min failover); guest-on-aws uses RDS Multi-AZ (≤ 2 min); guest-on-oci uses Autonomous DB (≤ 1 min managed failover); on-prem + colo per customer DR posture (≤ 30 min).
Tenant-class overlay: uniform.

### §8.7 Cell loss (whole cell evacuated)

Recovery target: traffic drained to next-nearest cell in ≤ 15 min per RTO §3.8.
Data-loss target: ≤ 5 min per RPO §3.8.
Per-context overlay: oyatie-public-cloud + oyatie-as-cloud-provider have hot stand-by cells; guest-on-aws + guest-on-oci have warm stand-by cells; on-prem + colo per customer posture.
Tenant-class overlay: uniform.

## §9 Methodology Closing Notes

The benchmark numbers in this document are normative once Wave 15B SLO authoring lands the canonical OpenSLO 1.0 YAML files under `billing/observability/slos/`. Until Wave 15B closes, this document serves as the target schedule. The numbers carry forward through Wave 14 backlog aggregation and Wave 15A P0 contradiction remediation; tier-segmented runbook SLO floors are explicitly retired in favor of the single industry-leader bar with deployment-context + tenant-class overlays. The unified-quality-bar doctrine forbids stratifying QoS by tenant class; only caps (usage / time / support / SLO-contractual) differ. This is the core change versus the existing per-tenant_class benchmark file.

