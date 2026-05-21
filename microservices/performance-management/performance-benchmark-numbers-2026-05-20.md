---
doc_class: performance-benchmark-numbers
microservice: performance-management
audit_wave: wave-4-rolling
audit_date: 2026-05-21
benchmark_anchor: Workday Performance (industry-leader)
secondary_anchors: [Lattice, 15Five]
big_8_family: HR/Payroll
big_8_priority: P0
governing_adr: ADR-0328
related_adrs: [ADR-0316, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0248]
companion_docs:
  - microservices/performance-management/coherence-audit-2026-05-20.md
  - microservices/performance-management/feature-parity-matrix-2026-05-20.md
deployment_contexts: [oyatie-public-cloud, guest-on-aws, oci-guest, on-prem, colo, oyatie-iaas]
tenant_classes: [demo_trial, paid]
metric_axes:
  - review-cycle-completion-latency
  - manager-dashboard-p99
  - mobile-sync-latency
  - feedback-delivery-latency
  - calibration-calculation-throughput
---

# performance-management — performance benchmark numbers (2026-05-21)

## 0. Benchmark envelope

### 0.1 Industry-leader anchor

Workday Performance is the chosen industry-leader anchor for the µservice's performance
targets. Reasoning:

- Workday Performance is the deepest-coverage counterpart in the feature-parity matrix
  (§5.4 of feature-parity-matrix-2026-05-20.md: 86% coverage of the 154-primitive union vs
  Lattice 71% and 15Five 68%)
- Workday Performance serves the largest enterprise tenants by employee count (publicly
  cited >10,000-employee deployments) and therefore its operational scale envelope is the
  tightest reasonable target for an oyatie HR/Payroll Big-8 service
- Workday Performance publishes performance commitments in its SLA and trust-center docs
  that are directly transferable to oyatie's target shape

Lattice and 15Five are secondary anchors. Where Lattice/15Five publish a stricter latency
target than Workday Performance (typically true for the consumer-grade interactive flows
like manager dashboard and mobile), the stricter target wins.

### 0.2 Single anchor rationale

The user directive specifies "single industry-leader target." Workday Performance is the
anchor. Where Lattice or 15Five publishes a tighter consumer-grade interactive target,
the tighter value is adopted as the consumer-grade overlay on Workday Performance's
enterprise base.

### 0.3 Deployment-context overlay

Targets are expressed for each of the six deployment contexts (oyatie-public-cloud,
guest-on-aws, oci-guest, on-prem, colo, oyatie-iaas) per ADR-0328 §D-20.13. Targets that
are dominated by network round-trip (mobile sync, manager dashboard p99) have stricter
in-region oyatie-public-cloud targets and relaxed cross-region targets.

### 0.4 Tenant-class overlay

Targets are expressed per tenant_class (demo_trial, paid). Demo_trial may run on
OCI-Always-Free shape (2× Ampere A1 ARM 4 OCPU / 24 GB total, see
`feedback_oci_always_free_maximization_2026_05_20`) and therefore inherits a relaxed target
appropriate to that footprint. Paid runs the full provisioned envelope.

Paid further decomposes by billing-component (bc-performance-management-core,
bc-performance-management-calibration, bc-performance-management-succession,
bc-performance-management-engagement, bc-performance-management-manager-toolbox) per
feature-parity-matrix-2026-05-20.md §13.2.

### 0.5 Metric envelope

Five required metrics per the audit deliverable:
- M1: Review-cycle-completion latency
- M2: Manager-dashboard p99
- M3: Mobile-sync latency
- M4: Feedback-delivery latency
- M5: Calibration-calculation throughput

Each metric is decomposed into the operational sub-metrics that the SLO files must encode.

## 1. M1 — review-cycle-completion latency

### 1.1 Definition

Review-cycle-completion latency = time from `review-cycle.create` event to the terminal
`review-cycle.evidence-sealed` event for every employee in the cycle. Measured end-to-end
across the cycle's lifetime, not per-step.

Sub-metrics:
- M1.a — review-form-render p95 (single review form load)
- M1.b — review-section-save p95 (write-through of a review section)
- M1.c — review-cycle-evidence-seal p99 (batch seal of all review evidence at cycle close)
- M1.d — review-cycle-rollover-to-archival p99 (move sealed cycle to cold storage)

### 1.2 Industry-leader anchor target (Workday Performance)

- M1.a review-form-render p95: 800 ms (Workday Performance trust-center cite for HCM
  Performance module interactive load on 5,000-employee tenant)
- M1.b review-section-save p95: 600 ms (Workday HCM general write-back commit envelope)
- M1.c review-cycle-evidence-seal p99 (per-1000-employee batch): 90 seconds (Workday batch
  job envelope for cycle-close)
- M1.d archival rollover p99 (per-1000-employee): 5 minutes

### 1.3 Lattice/15Five consumer-grade overlay

Lattice's mid-market interactive cite is review-form-render p95 ≈ 400 ms (single-region).
15Five's similar cite is ≈ 350 ms. The µservice adopts 400 ms as the consumer-grade overlay
when serving the bc-performance-management-core billing-component on a Tier-1 cell.

### 1.4 Tenant-class overlay

| Sub-metric | Demo_trial target | Paid target | Paid (Workday-class) target |
|---|---|---|---|
| M1.a render p95 | 2,000 ms | 400 ms (Lattice-class) | 800 ms (Workday-class) |
| M1.b save p95 | 1,500 ms | 500 ms | 600 ms |
| M1.c seal p99 / 1000 | 180 sec | 60 sec | 90 sec |
| M1.d archival p99 / 1000 | n/a (no archive) | 5 min | 5 min |

### 1.5 Deployment-context overlay

| Context | M1.a render p95 (paid) | Notes |
|---|---|---|
| oyatie-public-cloud (US, EU, KR multi-region) | 400 ms in-region; 700 ms cross-region | HTTP/3 + QUIC + ECH baseline |
| guest-on-aws | 500 ms in-region | AWS-native ALB + ACM-PCA + CloudFront added latency |
| oci-guest | 450 ms in-region | OCI Flexible Load Balancer + Bastion overhead |
| on-prem | 1,000 ms (single-DC, often no CDN) | Customer-controlled network |
| colo | 600 ms | Colo-network typically partial-CDN |
| oyatie-iaas | 400 ms | oyatie-owned full-stack, parity with oyatie-public-cloud |

### 1.6 Cell-tier overlay (T0..T4 per ADR-0248)

- T0 (substrate-only): N/A
- T1 (small): M1.a paid target ≤ 400 ms p95
- T2 (medium): M1.a paid target ≤ 500 ms p95 (more cross-cell traffic)
- T3 (large, geographic residency overlay): M1.a paid target ≤ 600 ms p95
- T4 (sovereign-pack): M1.a paid target ≤ 800 ms p95 (sovereign-cloud overhead)

### 1.7 SLO file binding

The existing `slos/local-review-form-latency.openslo.yaml` must be parameterized by tenant_class
and deployment_context. Author also `slos/review-cycle-evidence-seal.openslo.yaml` and
`slos/review-cycle-archival.openslo.yaml`.

### 1.8 Burn-rate alerting

p95 burn-rate: fast 5-min burn at 14.4× error-budget consumption rate → page.
Slow 1-hour burn at 6× → ticket.

## 2. M2 — manager-dashboard p99

### 2.1 Definition

Manager-dashboard p99 = the time from manager-user navigation event to first contentful
paint of the manager's team-status panel including team goal roll-up, team review-cycle
status, team feedback volume, team engagement score.

Sub-metrics:
- M2.a manager-dashboard-fcp p99 (first contentful paint)
- M2.b manager-dashboard-tti p99 (time to interactive)
- M2.c manager-dashboard-team-rollup-aggregate p99 (the underlying query that powers the
  dashboard)
- M2.d manager-dashboard-realtime-update lag p99 (WebSocket / HTTP/3 server-push update
  after a downstream event)

### 2.2 Industry-leader anchor (Workday Performance)

- M2.a FCP p99: 1,500 ms (Workday HCM trust-center)
- M2.b TTI p99: 2,500 ms
- M2.c team-rollup query p99 (50-direct-report team): 800 ms
- M2.d realtime update lag p99: 5 seconds

### 2.3 Consumer-grade overlay

Lattice manager-dashboard FCP cite ≈ 700 ms; 15Five ≈ 800 ms. The µservice adopts 800 ms
as the consumer-grade FCP overlay for paid Tier-1 cells.

### 2.4 Tenant-class overlay

| Sub-metric | Demo_trial | Paid Lattice-class | Paid Workday-class |
|---|---|---|---|
| M2.a FCP p99 | 3,000 ms | 800 ms | 1,500 ms |
| M2.b TTI p99 | 5,000 ms | 1,500 ms | 2,500 ms |
| M2.c rollup p99 | 2,500 ms | 500 ms | 800 ms |
| M2.d realtime lag p99 | 30 sec | 3 sec | 5 sec |

### 2.5 Deployment-context overlay

| Context | M2.a FCP p99 (paid) | Notes |
|---|---|---|
| oyatie-public-cloud | 800 ms | CDN + HTTP/3 priority hints |
| guest-on-aws | 900 ms | CloudFront priority hints; some pessimistic worst-case |
| oci-guest | 850 ms | OCI Edge CDN |
| on-prem | 2,000 ms | No CDN typical |
| colo | 1,200 ms | Partial CDN |
| oyatie-iaas | 800 ms | parity |

### 2.6 Team-size sensitivity

| Team size | M2.c rollup query p99 |
|---|---|
| 1-9 (small team) | 200 ms |
| 10-49 (medium team) | 400 ms |
| 50-199 (large team) | 800 ms |
| 200-999 (executive) | 2,000 ms |
| 1,000+ (CEO/CHRO) | async + cached projection |

The 1,000+ size invokes an async-projection pattern: precompute the rollup nightly into a
read-optimized projection store (CockroachDB or ScyllaDB per substrate decisions) and serve
from cache. Per-event invalidation via the ontology-projection bus.

### 2.7 SLO file binding

Author `slos/manager-dashboard-fcp.openslo.yaml`, `slos/manager-dashboard-team-rollup.openslo.yaml`,
`slos/manager-dashboard-realtime-lag.openslo.yaml`.

## 3. M3 — mobile-sync latency

### 3.1 Definition

Mobile-sync latency = time from mobile-client opens app to first usable view of the
employee's current goals, pending feedback, pending review-cycle tasks, and any
notifications. Covers initial sync after a cold-start and delta-sync after a
warm-resume.

Sub-metrics:
- M3.a cold-start sync p95 (first-time-this-session sync, full data set)
- M3.b warm-resume sync p95 (delta-only)
- M3.c push-to-device-display lag p95 (push notification originated → device shows it)
- M3.d offline-write-replay p95 (offline-mode write replays when network returns)
- M3.e bandwidth budget per sync (KB)

### 3.2 Industry-leader anchor (Workday Performance mobile)

- M3.a cold-start p95: 4 seconds (Workday Mobile trust-center, Workday Anywhere)
- M3.b warm-resume p95: 1.5 seconds
- M3.c push delivery lag p95: 8 seconds (vendor-end; rest is OS APN/FCM)
- M3.d offline-replay p95: 6 seconds for 10 pending writes
- M3.e bandwidth: 150 KB per warm-resume; 800 KB per cold-start

### 3.3 Consumer-grade overlay

Lattice + 15Five mobile cold-start cite ≈ 2.5 seconds. The µservice adopts that as the
consumer-grade overlay.

### 3.4 Tenant-class overlay

| Sub-metric | Demo_trial | Paid Lattice-class | Paid Workday-class |
|---|---|---|---|
| M3.a cold-start p95 | 8 sec | 2.5 sec | 4 sec |
| M3.b warm-resume p95 | 4 sec | 1 sec | 1.5 sec |
| M3.c push lag p95 | 30 sec | 5 sec | 8 sec |
| M3.d offline-replay p95 (10 writes) | n/a | 4 sec | 6 sec |
| M3.e bandwidth cold | 1.5 MB | 500 KB | 800 KB |
| M3.e bandwidth warm | 300 KB | 100 KB | 150 KB |

### 3.5 Deployment-context overlay

Mobile clients are not deployment-context-specific in the µservice tree (the apps run on
employee devices). The relevant axis is which oyatie API endpoint serves them. Mobile
endpoints are:

| Endpoint context | M3.a cold-start p95 (paid) |
|---|---|
| oyatie-public-cloud | 2.5 sec |
| guest-on-aws | 3.0 sec |
| oci-guest | 2.8 sec |
| on-prem | 6.0 sec (often LAN-only + VPN tunneled) |
| colo | 3.5 sec |
| oyatie-iaas | 2.5 sec |

### 3.6 Cellular and 5G overlay

Mobile-sync targets are network-conditioned. The reported p95s assume Wi-Fi or 5G. On
LTE the targets relax by 1.5×; on 3G-fallback the µservice's mobile contract must
gracefully degrade to text-only.

### 3.7 SLO file binding

Author `slos/mobile-cold-start.openslo.yaml`, `slos/mobile-warm-resume.openslo.yaml`,
`slos/mobile-push-lag.openslo.yaml`, `slos/mobile-offline-replay.openslo.yaml`.

## 4. M4 — feedback-delivery latency

### 4.1 Definition

Feedback-delivery latency = time from `feedback.create` to the recipient (manager,
employee, peer) seeing the new feedback in their inbox (in-app + email digest + push).

Sub-metrics:
- M4.a in-app delivery p95 (recipient logged in, sees feedback)
- M4.b email digest delivery p95 (digest cadence: 4 hours batched by default)
- M4.c push notification p95 (recipient device receives notification)
- M4.d Slack / Teams notification p95 (workplace-integration handoff)
- M4.e abuse-flag interception (when `feedback.flagged-abuse` event fires, hold delivery
  until reviewer-clears)

### 4.2 Industry-leader anchor (Workday Performance)

- M4.a in-app p95: 2 seconds (write → projection → recipient inbox)
- M4.b email digest p95: 4 hours (default cadence)
- M4.c push p95: 10 seconds (vendor end)
- M4.d Slack p95: 5 seconds
- M4.e abuse-flag hold: 5 seconds detection + reviewer-cleared (open-ended)

### 4.3 Consumer-grade overlay

Lattice in-app delivery cite ≈ 1 second; 15Five ≈ 1.5 seconds. The µservice adopts 1 second
as the consumer-grade target.

### 4.4 Tenant-class overlay

| Sub-metric | Demo_trial | Paid Lattice-class | Paid Workday-class |
|---|---|---|---|
| M4.a in-app p95 | 10 sec | 1 sec | 2 sec |
| M4.b email digest p95 | 12 hours | 1 hour | 4 hours |
| M4.c push p95 | 60 sec | 5 sec | 10 sec |
| M4.d Slack p95 | n/a (Slack disabled) | 3 sec | 5 sec |
| M4.e abuse-flag hold | 60 sec detection | 5 sec | 5 sec |

### 4.5 Deployment-context overlay

| Context | M4.a in-app p95 (paid) | Notes |
|---|---|---|
| oyatie-public-cloud | 1 sec | Kafka or NATS event bus + projection |
| guest-on-aws | 1.5 sec | MSK or EventBridge added latency |
| oci-guest | 1.3 sec | OCI Streaming |
| on-prem | 3 sec | Customer-controlled event broker (often slower) |
| colo | 2 sec | mixed |
| oyatie-iaas | 1 sec | parity |

### 4.6 Continuous-feedback ingestion (IP-028)

IP-028 names continuous feedback. Target ingestion rate: 1,000 feedback events/sec per
cell. Backpressure shed-policy: prefer-newer with audit-chain entry per shed.

### 4.7 SLO file binding

Existing `slos/local-feedback-submit-success.openslo.yaml` covers ingest success. Author
also `slos/feedback-delivery-in-app.openslo.yaml`, `slos/feedback-delivery-email-digest.openslo.yaml`,
`slos/feedback-delivery-push.openslo.yaml`.

## 5. M5 — calibration-calculation throughput

### 5.1 Definition

Calibration-calculation throughput = events/sec processed during a calibration run when
the system computes ratings + distribution + fairness analytics + 9-box grid placements +
compensation-handoff projection.

Sub-metrics:
- M5.a calibration-bucket-recompute throughput (events/sec)
- M5.b calibration-distribution-recompute throughput (employees/sec)
- M5.c fairness-analytics-recompute throughput (employees × dimensions/sec)
- M5.d 9-box-grid-placement throughput (employees/sec)
- M5.e calibration-publish latency (from calibration.approve to downstream events emitted)

### 5.2 Industry-leader anchor (Workday Performance)

Workday Performance calibration cite (Workday Talent Reviews module): for a tenant with
5,000 employees in a calibration session,

- M5.a bucket recompute throughput: 500 events/sec
- M5.b distribution recompute throughput: 2,000 employees/sec
- M5.c fairness analytics throughput: 1,000 employees × dimensions/sec (typical 6
  dimensions = 6,000 cell-evaluations/sec)
- M5.d 9-box placement throughput: 2,000 employees/sec
- M5.e publish latency: 30 seconds (settle + emit downstream events)

### 5.3 Consumer-grade overlay

Lattice / 15Five calibration is shallower than Workday (rows 5.3, 5.7-5.9 in feature-
parity-matrix per the matrix). Where Lattice/15Five tighter latency exists for sub-metric
5.b distribution recompute (≈ 5,000 employees/sec), the µservice adopts the tighter target
as the consumer-grade overlay.

### 5.4 Tenant-class overlay

| Sub-metric | Demo_trial | Paid Lattice-class | Paid Workday-class |
|---|---|---|---|
| M5.a bucket recompute | n/a | 1,000/sec | 500/sec |
| M5.b distribution | n/a | 5,000 emp/sec | 2,000 emp/sec |
| M5.c fairness | n/a | 2,000 emp×dim/sec | 1,000 emp×dim/sec |
| M5.d 9-box | n/a | 4,000 emp/sec | 2,000 emp/sec |
| M5.e publish latency | n/a | 15 sec | 30 sec |

Demo_trial does not provision calibration (it is a paid-with-calibration-component-only
capability per feature-parity-matrix §13.2 billing-component decomposition).

### 5.5 Deployment-context overlay

Calibration is a compute-heavy batch operation. The deployment context governs the
compute envelope (CPU + memory + GPU if AI-assisted summary):

| Context | M5.b distribution (paid) | Notes |
|---|---|---|
| oyatie-public-cloud | 5,000 emp/sec | Autoscaling worker pool |
| guest-on-aws | 4,000 emp/sec | EC2 spot + ASG; some warm-up cost |
| oci-guest | 4,500 emp/sec | OCI Functions + Container Instances |
| on-prem | 1,500 emp/sec | Customer-provisioned fixed-size cluster |
| colo | 3,000 emp/sec | mixed |
| oyatie-iaas | 5,000 emp/sec | parity |

### 5.6 Cell-tier overlay

| Cell tier | Max calibration session size (employees) |
|---|---|
| T0 | n/a |
| T1 | 500 |
| T2 | 5,000 |
| T3 | 50,000 |
| T4 | 100,000 |

Sessions exceeding the cell-tier max must split across multiple session-runs.

### 5.7 Fairness analytics throughput

Fairness analytics is the privacy-pack-sensitive sub-metric. EU-worker-council pack and
US-labor pack may require additional dimensions (protected-class breakdowns). Throughput
budget must absorb up to 12 dimensions before degrading.

| Pack overlay active | Required dimensions | Throughput at Workday-class |
|---|---|---|
| baseline | 4 (manager, department, level, tenure-bucket) | 1,000 emp×dim/sec |
| EU-worker-council | 6 (+ contract-type, age-band redacted) | 800 emp×dim/sec |
| US-labor | 8 (+ ethnicity-bucket, gender, disability-status; redacted under k-anon) | 600 emp×dim/sec |
| KR-PIPA strict mode | 4 (no protected-class — must derive from external) | 1,000 emp×dim/sec |

### 5.8 SLO file binding

Author `slos/calibration-bucket-recompute.openslo.yaml`, `slos/calibration-distribution.openslo.yaml`,
`slos/calibration-fairness-analytics.openslo.yaml`, `slos/calibration-9-box-throughput.openslo.yaml`,
`slos/calibration-publish-latency.openslo.yaml`.

## 6. Cross-metric envelope summary

### 6.1 Tier-1 paid Lattice-class target envelope (most-aggressive)

The µservice's "leadership" envelope, the one the µservice must hit to advertise itself
as Lattice-class for the bc-performance-management-core billing component:

| Metric | Target |
|---|---|
| M1.a review-form-render p95 | 400 ms |
| M1.b review-section-save p95 | 500 ms |
| M2.a manager-dashboard FCP p99 | 800 ms |
| M2.c team-rollup query p99 (50 reports) | 500 ms |
| M3.a mobile cold-start p95 | 2.5 sec |
| M3.b mobile warm-resume p95 | 1 sec |
| M4.a feedback in-app delivery p95 | 1 sec |
| M5.b calibration distribution recompute | 5,000 emp/sec |

### 6.2 Tier-2 paid Workday-class enterprise envelope (most-conservative for paid)

| Metric | Target |
|---|---|
| M1.a review-form-render p95 | 800 ms |
| M1.b review-section-save p95 | 600 ms |
| M2.a manager-dashboard FCP p99 | 1,500 ms |
| M2.c team-rollup query p99 (50 reports) | 800 ms |
| M3.a mobile cold-start p95 | 4 sec |
| M3.b mobile warm-resume p95 | 1.5 sec |
| M4.a feedback in-app delivery p95 | 2 sec |
| M5.b calibration distribution recompute | 2,000 emp/sec |

### 6.3 Demo_trial envelope (OCI Always Free shape)

| Metric | Target |
|---|---|
| M1.a review-form-render p95 | 2,000 ms |
| M1.b review-section-save p95 | 1,500 ms |
| M2.a manager-dashboard FCP p99 | 3,000 ms |
| M2.c team-rollup query p99 (50 reports) | 2,500 ms |
| M3.a mobile cold-start p95 | 8 sec |
| M3.b mobile warm-resume p95 | 4 sec |
| M4.a feedback in-app delivery p95 | 10 sec |
| M5 calibration | n/a (no calibration in demo_trial) |

### 6.4 Cross-context overlay summary

Per-context overlay multipliers (apply to paid Lattice-class target):

| Context | Multiplier |
|---|---|
| oyatie-public-cloud | 1.0× (baseline) |
| guest-on-aws | 1.15× |
| oci-guest | 1.10× |
| on-prem | 2.0× to 3.0× (highly variable) |
| colo | 1.5× |
| oyatie-iaas | 1.0× |

## 7. Capacity model

### 7.1 Concurrent active tenants per cell

- T1 cell: 200 paid tenants + 1,000 demo_trial tenants
- T2 cell: 2,000 paid tenants + 5,000 demo_trial tenants
- T3 cell: 20,000 paid tenants + 50,000 demo_trial tenants
- T4 cell: 50,000 paid tenants (sovereign — no demo_trial)

### 7.2 Per-tenant compute envelope

- Demo_trial tenant: 0.05 OCPU avg + 0.2 OCPU burst; 100 MB memory; 1 GB storage. Shared
  on OCI Always Free shape across many tenants.
- Paid bc-performance-management-core (≤500 employees): 0.5 OCPU avg + 2 OCPU burst;
  1 GB memory; 50 GB storage.
- Paid bc-performance-management-core (501-5,000 employees): 2 OCPU avg + 8 OCPU burst;
  4 GB memory; 200 GB storage.
- Paid bc-performance-management-core (5,001-50,000 employees): 8 OCPU avg + 32 OCPU
  burst; 16 GB memory; 1 TB storage.
- Paid full-bundle (calibration + succession + engagement + manager-toolbox, ≤50,000
  employees): 32 OCPU avg + 128 OCPU burst; 64 GB memory; 5 TB storage.

### 7.3 Storage growth rate

- Goal records: ~10 KB/goal × 4 goals/employee/year = 40 KB/employee/year
- Review records: ~50 KB/review × 1 cycle/employee/year = 50 KB/employee/year
- Feedback records: ~5 KB/feedback × 24 feedback/employee/year (continuous) = 120 KB/employee/year
- Engagement-pulse responses: ~2 KB/response × 12 pulses/employee/year = 24 KB/employee/year
- Calibration evidence: ~30 KB/cycle/employee = 30 KB/employee/year
- Audit-chain entries: ~1 KB/event × 200 events/employee/year = 200 KB/employee/year

Total: ~470 KB/employee/year hot. Cold-archive after 2 years: ~30 KB/employee/year (most
compressible).

### 7.4 Network bandwidth

- Review-form-render typical payload: 80 KB (form + competencies + history)
- Manager-dashboard typical payload: 200 KB (team panel + rollups + recent activity)
- Mobile cold-start typical payload: 800 KB (employee profile + goals + pending tasks + cache)
- Mobile warm-resume typical payload: 150 KB (delta only)
- Feedback ingest per-event: 5 KB
- Engagement-pulse response per-event: 2 KB

## 8. Cost-budget linkage

### 8.1 Per-tenant cost envelope (paid Lattice-class, public cloud, T2 cell, 5,000-employee)

- Compute: ~$200/month (8 OCPU avg, modest oyatie-public-cloud rates)
- Storage: ~$60/month (200 GB at $0.30/GB-month tiered)
- Database (Postgres + projection store): ~$150/month
- Egress: ~$30/month (assuming 100 GB/month outbound to mobile clients + integrations)
- Observability (metrics + logs + traces): ~$50/month
- Total: ~$490/month per paid 5,000-employee tenant

Workday Performance / Lattice / 15Five list-pricing (approximate, 2026-Q2 public sources):

- Lattice: $11/user/month (Standard) → $55,000/month for 5,000 users
- 15Five: $10/user/month (Engage + Perform Standard) → $50,000/month
- Workday Performance: $20/user/month effective → $100,000/month

oyatie cost: $490/month for 5,000 employees → ~$0.10/user/month infrastructure cost.
Gross margin envelope at $5/user/month list-price: 98%. At $1/user/month: 90%. Both are
hyperscaler-grade margins.

### 8.2 Demo_trial cost envelope

Per the OCI Always Free maximization directive: demo_trial tenants run on shared OCI
Always Free shape (~$0/month real cost). Per-tenant infra cost: ~$0/month.

### 8.3 Cost overlay by deployment context

| Context | Cost relative to oyatie-public-cloud |
|---|---|
| oyatie-public-cloud | 1.0× (baseline) |
| guest-on-aws | 1.4× (AWS list-rates higher than oyatie internal) |
| oci-guest | 0.9× (OCI cheaper at our scale) |
| on-prem | 0.6× direct + customer's TCO ~3-5× overall |
| colo | 0.7× direct + customer's TCO ~2-3× overall |
| oyatie-iaas | 1.1× (we eat the full IaaS layer + margin) |

## 9. Promotion gates

### 9.1 Dev-to-staging promotion gate

Must demonstrate Tier-1 paid Lattice-class targets §6.1 in a synthetic load test of:
- 100 tenants × 500 employees each (50,000-employee total load)
- 10-minute steady-state with 4× spike at minute 5
- All p95/p99 targets must hold throughout the spike

### 9.2 Staging-to-production promotion gate

Must demonstrate Tier-2 paid Workday-class targets §6.2 at:
- 10 tenants × 5,000 employees each (50,000-employee total) on T2 cell shape
- 1-hour steady-state with 2× spike at minute 30
- Plus 1,000 demo_trial tenants on OCI Always Free in parallel (cost-budget verification)

### 9.3 Production-to-multi-region promotion gate

Must demonstrate cross-region p95 ≤700 ms (§1.5 overlay) and 99.99% availability over a
7-day window.

## 10. Observability binding

### 10.1 Metric names

All metrics follow oyatie naming convention `oya.performance_management.<sub_metric>.histogram`
emitted to Prometheus with labels:
- tenant_class ∈ {demo_trial, paid}
- billing_component ∈ {core, calibration, succession, engagement, manager-toolbox}
- deployment_context ∈ {oyatie-public-cloud, guest-on-aws, oci-guest, on-prem, colo, oyatie-iaas}
- cell_tier ∈ {T1, T2, T3, T4}
- bounded_context ∈ {goal-cycle, review-cycle, feedback, engagement-survey, calibration,
  1-on-1, weekly-check-in, recognition, succession, manager-tools}

### 10.2 Trace propagation

All sub-metrics correlate via W3C trace-context per ADR-0253. Trace IDs flow through:
edge HTTP/3 → REST → application → usecase → domain → kernel → adapter.

### 10.3 Audit-chain emission

Every metric breach (burn-rate fast or slow) emits an audit-chain event with full
labels. The `slos/local-slo-alerts.yaml` file must list every metric × tenant_class ×
context combination as an explicit alert row.

## 11. Closure

This benchmark numbers document binds:
- Workday Performance as single industry-leader anchor with Lattice/15Five consumer-grade
  overlays where stricter
- Six deployment contexts × two tenant_classes × four billing-components × four cell-tiers
  matrix of targets
- Five required metrics (M1..M5) decomposed into sub-metrics
- Capacity envelope per tenant size
- Cost envelope and margin verification
- Promotion gates at dev→staging, staging→prod, prod→multi-region

The SLO files in `slos/` must be augmented per §1.7, §2.7, §3.7, §4.7, §5.8 — net ~17
new OpenSLO files. The existing 12 OpenSLO files cover ~30% of the required surface.

Companion artifacts:
- coherence-audit-2026-05-20.md (ownership audit + 27 P0 findings)
- feature-parity-matrix-2026-05-20.md (Lattice + 15Five + Workday Performance union)

End of performance benchmark numbers.
