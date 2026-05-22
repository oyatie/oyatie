---
doc_class: PerformanceBenchmarkNumbers
microservice: production-planning
audit_wave: wave-4-rolling
audit_date: 2026-05-20
authored_at: 2026-05-21
benchmarking_doctrine: single industry-leader target + deployment-context overlay + tenant-class overlay; NO tier deltas
companion_docs:
  - microservices/production-planning/coherence-audit-2026-05-20.md
  - microservices/production-planning/feature-parity-matrix-2026-05-20.md
related_adrs:
  - ADR-0252 HLC default
  - ADR-0253 HTTP/3 + QUIC default
  - ADR-0263 observability emission contract
  - ADR-0328 substance bar
target_workloads:
  - mrp_run_completion: MRP run end-to-end completion latency
  - schedule_recalc: finite-schedule recalculation latency
  - what_if_fork: scenario fork latency
  - shop_floor_ingest: shop-floor event ingestion sustained rate
  - dashboard_p99: dashboard render p99 latency
industry_leader_target_sources:
  - SAP S/4HANA PP/DS benchmark documentation (SAP Note 2825176)
  - Oracle Supply Chain Planning Cloud published response targets (Oracle Doc ID 2456773.1)
  - Kinaxis Maestro published concurrent-planning targets (Kinaxis whitepaper)
  - SCOR Reference Model performance metrics
---

# Performance Benchmark Numbers: production-planning

## §1 Doctrine

Per the 2026-05-20 directive, performance targets are expressed as **one industry-leader target + deployment-context overlay + tenant-class overlay**. The pre-2026-05-20 "tier delta" model (declaring different numbers for T0/T1/T2/T3) is **retired**. Performance budgets are composed at runtime by `(base_target, deployment_context, tenant_class)` triplet, not pre-computed per tier.

Composition rules:
1. **Base target** = the industry-leader published number adjusted for Oyatie's substrate (HTTP/3 + Cloud Hypervisor + Cedar + audit-chain seal). This is **one number**.
2. **Deployment-context overlay** = a multiplicative factor on the base. `oyatie-public-cloud` = ×1.0, `guest-on-aws` = ×1.10 (sibling-service latency budget for AWS-anchored adapters), `guest-on-oci` = ×1.10, `on-prem` = ×1.30 (customer hardware variance), `colo` = ×1.20, `oyatie-as-cloud-provider` = ×1.0.
3. **Tenant-class overlay** = a multiplicative or absolute floor. `multi-tenant` = ×1.0 noisy-neighbor budget; `paid` = ×1.0 with priority; `byo-cloud` = ×1.10; `self-hosted` = ×1.30; `demo`/`sandbox`/`trial`/`dev` = ×3.0 with absolute throughput cap (per OCI Always Free §D-19).

Each workload below names: base industry-leader target, overlay-composed effective target per (context, tenant_class), measurement window, evidence source, regression-gate threshold, and SLO-burn alert thresholds.

## §2 Workload 1 — MRP Run Completion Latency

### §2.1 Definition
The MRP run completion latency is the **end-to-end wall-clock** from the moment a `POST /v1/production-planning/mrp-run` request is accepted to the moment the corresponding ontology projection is published (per ADR-0263 emission contract). It is **not** the API-acknowledgment latency, which is a separate sub-SLO.

### §2.2 Industry-leader benchmarks
- **SAP S/4HANA PP/DS** (SAP Note 2825176, "Performance of MRP and Live Planning"):
  - 10k material masters single-plant: 30-90 seconds typical, 3 minutes p99 worst-case
  - 100k material masters single-plant: 5-15 minutes typical, 30 minutes p99 worst-case
  - 1M material masters multi-plant: 30-90 minutes typical, 4 hours p99 worst-case
- **Oracle Supply Chain Planning Cloud** (Oracle Doc ID 2456773.1):
  - 10k items: 1-3 minutes; 100k items: 10-25 minutes; 1M items: 60-180 minutes
- **Kinaxis Maestro** (Kinaxis "Concurrent Planning" whitepaper):
  - Concurrent planning recomputes incrementally; full-recompute baseline ~similar to SAP

### §2.3 Oyatie base target (industry-leader anchor: median of SAP / Oracle / Kinaxis full-recompute at each scale)

| SKU scale | Base p50 | Base p95 | Base p99 |
|---|---|---|---|
| `<10k` materials, single plant | 30 s | 60 s | 120 s |
| `10k-100k` materials, single plant | 4 min | 9 min | 18 min |
| `100k-1M` materials, single plant | 25 min | 60 min | 110 min |
| `>1M` materials, multi-plant | 90 min | 180 min | 240 min |

### §2.4 Overlay-composed effective targets

For `oyatie-public-cloud × multi-tenant`:
- `<10k` p99 = 120s × 1.0 × 1.0 = **120s**
- `100k` p99 = 18min × 1.0 × 1.0 = **18min**

For `guest-on-aws × paid` (single-tenant dedicated):
- `<10k` p99 = 120s × 1.10 × 1.0 = **132s**
- `100k` p99 = 18min × 1.10 × 1.0 = **20min**

For `on-prem × self-hosted`:
- `<10k` p99 = 120s × 1.30 × 1.30 = **203s**
- `100k` p99 = 18min × 1.30 × 1.30 = **30min**

For `guest-on-oci × demo (Always Free)`:
- absolute throughput cap: ≤1 MRP run per hour per tenant, hard-capped at 10k materials, no overlay (cap dominates)

### §2.5 API-acknowledgment latency (separate sub-SLO)
The `POST /v1/production-planning/mrp-run` request returns a 202 Accepted with run id. This response latency is the user-visible latency:
- Base p50 < 80 ms; base p95 < 200 ms; base p99 < 500 ms
- Effective per (context, tenant_class) using §2.4 overlays.

### §2.6 SLO file (proposed authoring)
`slos/mrp-run-completion-latency.openslo.yaml` should declare:
```
slo:
  operation: production_planning.mrp_run.completion_e2e
  scale_bucket: "<10k|10k-100k|100k-1M|>1M"
  availability_30d: "99.5%"
  base_latency_p50_seconds: { <10k: 30, 10k-100k: 240, 100k-1M: 1500, >1M: 5400 }
  base_latency_p95_seconds: { <10k: 60, 10k-100k: 540, 100k-1M: 3600, >1M: 10800 }
  base_latency_p99_seconds: { <10k: 120, 10k-100k: 1080, 100k-1M: 6600, >1M: 14400 }
  context_overlay: { oyatie-public-cloud: 1.0, guest-on-aws: 1.10, guest-on-oci: 1.10, on-prem: 1.30, colo: 1.20, oyatie-as-cloud-provider: 1.0 }
  tenant_class_overlay: { multi-tenant: 1.0, paid: 1.0, byo-cloud: 1.10, self-hosted: 1.30, demo: 3.0, sandbox: 3.0, trial: 3.0, dev: 3.0 }
  measurement_window_minutes: 30
  error_budget_30d_minutes_per_scale_bucket: { <10k: 217, 10k-100k: 217, 100k-1M: 217, >1M: 217 }
  rationale: "Net-requirements + lot-sizing + pegging are NP-complete pegging-multi (B.13); algorithmic complexity dominates wall-clock; deviation > base × overlay means BOM-explosion fan-out exploded or net-change is broken"
```

### §2.7 Regression gate
A CI lane `ci-mrp-run-perf` runs a synthetic 10k-material MRP run every PR; threshold = base × 1.30 (allows 30% headroom for noise). Failure blocks merge.

### §2.8 SLO-burn alert thresholds
- **Warning**: 30-day error budget consumed >50%.
- **Critical**: 30-day error budget consumed >80%.
- **Page**: 30-day error budget consumed >100% (exhausted).

## §3 Workload 2 — Schedule Recalc Latency

### §3.1 Definition
The schedule recalc latency is the wall-clock from the moment a disruption event arrives (machine breakdown, urgent customer order, expedite request) to the moment the new schedule is published. This is the **headline user-facing APS metric**.

### §3.2 Industry-leader benchmarks
- **SAP APO PP/DS interactive planning board `/SAPAPO/CDPSC`**: 1-5 seconds for ≤500 operations on a single planning board; 30 seconds for 5k operations.
- **Oracle Production Scheduling Cloud**: 2-10 seconds for ≤1k operations; published target "sub-minute for interactive replan".
- **Kinaxis Maestro concurrent planning**: published target "sub-second for incremental updates" via concurrent-planning engine; full recompute on 10k ops in ~5 seconds.

### §3.3 Oyatie base target

| Operation scale | Base p50 | Base p95 | Base p99 |
|---|---|---|---|
| `<100` ops (single work-center disruption) | 200 ms | 500 ms | 1.5 s |
| `100-1k` ops (single plant disruption) | 1 s | 2.5 s | 5 s |
| `1k-10k` ops (multi-plant) | 5 s | 12 s | 25 s |
| `>10k` ops (enterprise replan) | 30 s | 60 s | 120 s |

IP-021 benchmark table (existing) is consistent with this for the forward heuristic:
- 100 ops forward: 50ms p50, 110ms p95, 220ms p99 — **better than base** (in-process heuristic).
- 1k ops forward: 480ms p50, 1.1s p95, 2.2s p99 — within base.
- 10k ops bottleneck-anchor (with 50 work centers): 8s p50, 18s p95, 35s p99 — slightly **above** base p99 (35s > 25s); acceptable for the heaviest scenario with DBR + AC-3.

### §3.4 Overlay-composed effective targets

For `oyatie-public-cloud × multi-tenant`:
- `<100` ops p99 = 1.5 s × 1.0 × 1.0 = **1.5 s**
- `1k-10k` ops p99 = 25 s × 1.0 × 1.0 = **25 s**

For `on-prem × self-hosted`:
- `<100` ops p99 = 1.5 s × 1.30 × 1.30 = **2.54 s**
- `1k-10k` ops p99 = 25 s × 1.30 × 1.30 = **42 s**

### §3.5 SLO file (proposed authoring)
`slos/schedule-recalc-latency.openslo.yaml` should declare per-scale-bucket SLOs identical in shape to §2.6.

### §3.6 Strategy-specific deltas
The IP-021 benchmark distinguishes strategies; the SLO should track each:
- forward: heuristic, fastest
- backward: heuristic + due-date reverse traversal, ~10% slower than forward
- bottleneck-anchor (DBR): identifies bottleneck + AC-3 propagation + drum/rope/buffer placement, ~2-3× slower than forward but produces higher-utilization schedules

The SLO should bound bottleneck-anchor at ≤3× the forward latency.

### §3.7 Regression gate
CI lane `ci-schedule-recalc-perf` runs a synthetic 1k-operation replan via each of the three strategies. Threshold: base × 1.30.

### §3.8 Why schedule recalc latency dominates buyer evaluation
A machine breakdown at 09:14 on a 24-hour automotive production line costs ~$10k/minute in lost output for a Tier-1 OEM. The buyer evaluates: "How fast can my dispatcher get a new schedule on the screen?" Kinaxis Maestro wins this evaluation today with sub-second incremental updates. Oyatie's 25s p99 for 10k ops is acceptable for Phase-4 ship; for Phase-5 differentiation, IP-30 (concurrent planning) must close the gap to ≤2s p99 incremental.

## §4 Workload 3 — What-If Scenario Fork Latency

### §4.1 Definition
The what-if scenario fork latency is the wall-clock from the moment a scenario-fork command arrives to the moment the forked scenario is queryable (separate read view, separate Cedar context, separate audit-chain anchor). This is **NOT YET IMPLEMENTED** per §3.10 of the coherence audit. The targets below are **forward-looking** for IP-30.

### §4.2 Industry-leader benchmarks
- **SAP APO planning version copy** `/SAPAPO/MVM`: 30 s - 5 min depending on planning-version data size; not interactive.
- **Oracle Plan Inputs Diagnostics + Scenario Copy**: 1-3 min typical; not interactive.
- **Kinaxis Maestro `CreateScenario`**: published target "sub-second" via copy-on-write dataset overlay — **the headline differentiator**.

### §4.3 Oyatie target (forward-looking; binds IP-30 acceptance criteria)

| Scenario dataset size | Base p50 | Base p95 | Base p99 |
|---|---|---|---|
| `<1M` rows (per-plant slice) | 500 ms | 1 s | 2 s |
| `1M-10M` rows (multi-plant) | 2 s | 5 s | 10 s |
| `10M-100M` rows (enterprise) | 10 s | 25 s | 60 s |

Realization model: **copy-on-write** scenario overlay (no full dataset copy). Acceptance: scenario forking must NOT materialize the dataset; it must materialize only the divergence-input delta and lazily compute downstream projections via the concurrent-planning engine (when IP-30-CONCURRENT-PLANNING lands).

### §4.4 Overlay-composed effective targets

For `oyatie-public-cloud × multi-tenant`:
- `<1M` rows p99 = 2 s × 1.0 × 1.0 = **2 s** (matches Kinaxis Maestro sub-second to 2s window).

For `on-prem × self-hosted`:
- `<1M` rows p99 = 2 s × 1.30 × 1.30 = **3.4 s**.

### §4.5 SLO file (proposed authoring)
`slos/scenario-fork-latency.openslo.yaml` — initially as a forecast SLO; promoted to live SLO upon IP-30 GA.

### §4.6 Regression gate
N/A until IP-30 lands.

## §5 Workload 4 — Shop-Floor Event Ingestion (sustained rate)

### §5.1 Definition
Shop-floor event ingestion is the sustained per-tenant rate of inbound shop-floor signals (MES B2MML messages per IP-024) the service can accept without lag exceeding the audit-chain seal latency target.

### §5.2 Industry-leader benchmarks
- **SAP DMC** (SAP Digital Manufacturing Cloud): 10k events/sec/instance documented; bursts to 50k/sec.
- **Oracle MES Cloud**: 5-15k events/sec/instance.
- **AWS IoT Core + Industrial IoT**: 100k+ messages/sec/account (raw IoT; not domain-aware).
- **OPC-UA over standard server**: 5-20k tags/sec per gateway typical.

### §5.3 Oyatie base target

| Tenant scale | Sustained ingest rate | Burst (1-min) | Audit-seal lag p99 |
|---|---|---|---|
| Small (<10 work centers) | 100 events/sec | 500 events/sec | 200 ms |
| Medium (10-100 work centers) | 1k events/sec | 5k events/sec | 500 ms |
| Large (100-1k work centers) | 10k events/sec | 50k events/sec | 1.5 s |
| Enterprise (>1k work centers) | 50k events/sec | 200k events/sec | 3 s |

Rationale: per IP-024 §I event classes (`mes.production-performance.v1`, `mes.production-response.v1`, `mes.state-drift.v1`), each event triggers a Cedar gate, a state-machine transition check, an ontology projection upsert, and an audit-chain seal. The seal latency must remain p99 ≤ 3s under the highest sustained rate.

### §5.4 Overlay-composed effective targets

For `oyatie-public-cloud × multi-tenant`:
- Medium sustained = 1k events/sec × 1.0 × 1.0 = **1k events/sec**.

For `guest-on-aws × paid`:
- Medium sustained = 1k events/sec × 1.10 × 1.0 = **909 events/sec** (overlay >1 means *budget loosens*, so effective allowed maximum lowers proportionally).

For `on-prem × self-hosted`:
- Medium sustained = 1k events/sec × 1.30 × 1.30 = **591 events/sec** under customer hardware budget.

For `guest-on-oci × demo (Always Free)`:
- Absolute cap: 10 events/sec/tenant; OCI Streaming Always Free budget dominates.

### §5.5 SLO file (proposed authoring)
`slos/shop-floor-event-ingest.openslo.yaml`:
```
slo:
  operation: production_planning.mes_handshake.event_ingest
  scale_bucket: small|medium|large|enterprise
  sustained_throughput_events_per_second: { small: 100, medium: 1000, large: 10000, enterprise: 50000 }
  burst_throughput_events_per_second: { small: 500, medium: 5000, large: 50000, enterprise: 200000 }
  audit_seal_lag_p99_ms: { small: 200, medium: 500, large: 1500, enterprise: 3000 }
  availability_30d: "99.9%"
  measurement_window_minutes: 5
  rationale: "Per IP-024 ISA-95 / B2MML; each event triggers Cedar gate + state-machine transition + ontology projection + audit seal; throughput floor protects MES handshake from back-pressure that would cause state drift > 5min reconcile cadence"
```

### §5.6 Regression gate
CI lane `ci-mes-ingest-perf` runs a synthetic 1k events/sec sustained for 60 seconds; threshold: audit-seal-lag p99 ≤ 500 ms.

### §5.7 Failure mode: ingest back-pressure
If ingest rate exceeds the bucket budget, the queue at the AsyncAPI consumer fills. Per IP-024 §D-2.5, drift detection runs every 5 minutes; if back-pressure pushes drift > 0 for a sustained window, the reconcile UC fires. The reconcile UC carries its own throttle (reconcile-rate-limit Cedar policy) to prevent reconcile storms. SLO-burn threshold (warning) at 50% burn triggers an autoscale-up of the worker pool; SLO-burn threshold (critical) at 80% triggers a chaos-isolate of the noisiest tenant per ADR-0248 cellular shuffle sharding.

## §6 Workload 5 — Dashboard Render p99 Latency

### §6.1 Definition
Dashboard render p99 is the time from a dashboard GET request landing at the api-gateway µservice to the first-byte response from the production-planning dashboard endpoint, measured at the dashboard JSON contract (`dashboards/production-planning-overview.json`).

### §6.2 Industry-leader benchmarks
- **SAP Fiori dashboard**: p95 ~1-2 s, p99 ~3-5 s.
- **Oracle Fusion Analytics**: p95 ~1.5 s, p99 ~4 s.
- **Kinaxis Maestro dashboard**: p95 sub-second for default views; p99 ~2 s.
- **Grafana with Prometheus backend** (Oyatie's observability substrate): p95 100-500 ms, p99 ~1 s for cached panels; ~3 s for cold-cache wide-range panels.

### §6.3 Oyatie base target

| Dashboard class | Base p50 | Base p95 | Base p99 |
|---|---|---|---|
| Single-bounded-context summary (BomRevisionHealth) | 100 ms | 300 ms | 800 ms |
| Cross-bounded-context overview (production-planning-overview) | 200 ms | 600 ms | 1.5 s |
| Wide-range / multi-tenant drill (incident-response) | 500 ms | 1.5 s | 3 s |

### §6.4 Overlay-composed effective targets

For `oyatie-public-cloud × multi-tenant`:
- Cross-bounded-context p99 = 1.5 s × 1.0 × 1.0 = **1.5 s**.

For `on-prem × self-hosted`:
- Cross-bounded-context p99 = 1.5 s × 1.30 × 1.30 = **2.54 s**.

### §6.5 SLO file (proposed authoring)
`slos/dashboard-render-latency.openslo.yaml`.

### §6.6 Regression gate
CI lane `ci-dashboard-render-perf` renders 5 representative dashboards; threshold p99 ≤ 2 s.

## §7 Cross-Workload Composition Rules

### §7.1 Composing performance across workloads
When a user-facing operation spans multiple workloads (e.g., "click → fork scenario → recalc schedule → render dashboard"), the composed p99 is the **sum** of the per-workload p99s, not the **max**. A user clicking through scenario-fork (2s) + recalc (5s, 1k ops) + dashboard render (1.5s) sees up to 8.5s p99 user-facing latency.

This is acceptable for Phase-4 (typical SAP equivalent: 30s+). Phase-5 target: reduce to <3s composed via IP-30 concurrent planning.

### §7.2 Cellular sharding compositeness
Per ADR-0248, each cell carries its own performance envelope. A tenant pinned to a single cell sees the per-cell envelope. A tenant in shuffle-shard (across N cells) sees the median cell envelope. Cell migration latency (Tier 0 → Tier 1) is published separately and is NOT in this document's scope.

### §7.3 HTTP/3 transport overhead
Per ADR-0253, HTTP/3 + QUIC default. QUIC's 0-RTT handshake reduces per-call latency by ~80 ms vs HTTP/2 TLS 1.3 1-RTT. Base targets include this savings. Fallback to HTTP/1.1 adds 80 ms p99 (TCP+TLS handshake); fallback rate should be monitored as a separate SLO.

### §7.4 Audit-chain seal cost
Per ADR-0263, every state transition seals to the audit chain. Seal latency adds 10-30 ms p99 per operation under normal load. The MRP-run completion latency (§2) does NOT include the audit-seal cost; it's captured by the per-event audit-seal-lag SLO (§5.3).

### §7.5 Cedar evaluation cost
Per ADR-0243, every mutation evaluates a Cedar permit. Cedar evaluation latency is p99 ≤ 5 ms for the production-planning policy set (6 policies, ~200 lines each total). The mutation latency includes Cedar; the per-Cedar-evaluation latency is published as a separate `oya_cedar_evaluation_latency_ms` metric per ADR-0263.

### §7.6 Cell-Hypervisor isolation overhead
Per ADR-0254 Cloud Hypervisor + Kata pods, cell isolation adds <2% CPU overhead and <5 ms p99 syscall overhead vs bare-metal container. This is included in base targets.

## §8 Tenant-Class Specific Notes

### §8.1 `multi-tenant` baseline
- Noisy-neighbor isolation via Cell topology (per ADR-0248). Multi-tenant tenants share a cell with N other tenants.
- Per-tenant rate-limit governed by tenant quota service.
- Burst tolerance bounded by cell-level admission control.

### §8.2 `paid` tenant
- Same as multi-tenant unless the deal terms include dedicated cell.
- Priority in admission control = HIGH (vs multi-tenant = NORMAL).

### §8.3 `byo-cloud` tenant
- Tenant brings their own AWS / OCI account; Oyatie runs in customer VPC / VCN.
- Customer hardware variance is bounded by tenant SLA in marketplace deal (per ADR-0314 settlement evidence).
- Overlay ×1.10 reflects typical sibling-cloud latency.

### §8.4 `self-hosted` tenant
- Tenant owns hardware. Performance is bounded by customer hardware.
- Overlay ×1.30 reflects industry-typical variance between hyperscaler and customer-managed hardware (per various Gartner / IDC studies).

### §8.5 `demo` / `sandbox` / `trial` / `dev` (OCI Always Free per ADR-0328 §D-19)
- Resource cap: 4 OCPU Ampere A1 + 24 GB RAM per tenant.
- Throughput cap absolute, not multiplicative.
- Hard caps:
  - MRP run: ≤1 per hour, ≤10k materials.
  - Schedule recalc: ≤10 per hour, ≤500 operations.
  - Scenario fork: not available (defer to paid tier).
  - Shop-floor ingest: ≤10 events/sec.
  - Dashboard render: ≤5 concurrent users/tenant.

## §9 Deployment-Context Specific Notes

### §9.1 `oyatie-public-cloud` (Context 1)
- Reference deployment; ×1.0 overlay.
- Cell topology Tier 0/1/2/3 per ADR-0248.
- HTTP/3 + QUIC default.
- All SLOs measured here are the baseline for comparison.

### §9.2 `guest-on-aws` (Context 2)
- Tenant supplies AWS account (per ADR-0328 §D-15.29).
- Latency adders: AWS-VPC east-west hops vs Oyatie-cell internal mesh (~5-10 ms), AWS-managed S3 round-trip vs cloud-storage cell (~5-15 ms).
- Overlay ×1.10.

### §9.3 `guest-on-oci` (Context 3)
- Same architecture as guest-on-aws on OCI primitives.
- Ampere A1 arm64 default per ADR-0328 §D-19.12.
- Overlay ×1.10.
- Always-Free sub-profile: see §8.5.

### §9.4 `on-prem` (Context 4)
- Customer hardware. Performance bounded by customer SLA.
- Overlay ×1.30.
- Required by Phase-4 manufacturing buyers per coherence audit §5.4.

### §9.5 `colo` (Context 5)
- Equinix Metal / Cyxtera / similar.
- Overlay ×1.20 (between hyperscaler and on-prem).

### §9.6 `oyatie-as-cloud-provider` (Context 6)
- Oyatie operates its own cloud cells.
- Overlay ×1.0 (same as oyatie-public-cloud, since Oyatie owns the substrate).

## §10 Numerical Worked Examples

### §10.1 Example A: Mid-market discrete-manufacturing buyer on multi-tenant SaaS
- Tenant: 25 work centers, 8k materials, 800 work orders/day.
- Context: `oyatie-public-cloud`.
- Tenant_class: `multi-tenant`.
- MRP-run p99 = base 120s × 1.0 × 1.0 = **120s** → acceptable; SAP S/4HANA Cloud equivalent is 60-180s for this scale.
- Schedule-recalc p99 (single work-center disruption, ~80 ops): base 1.5s × 1.0 × 1.0 = **1.5s** → competitive with SAP `/SAPAPO/CDPSC` 1-5s.
- Scenario-fork (Phase-5): N/A until IP-30.
- Shop-floor ingest sustained: base 100 events/sec × 1.0 × 1.0 = **100 events/sec** → covers 25 WC × ~4 events/sec/WC.
- Dashboard render p99 (overview): base 1.5s × 1.0 × 1.0 = **1.5s** → competitive with Kinaxis Maestro.

### §10.2 Example B: Aerospace OEM on-prem + colo, paid deal
- Tenant: 800 work centers, 240k materials, 5k production orders/day, AS9100 + IATF.
- Context: `on-prem` (data residency for IP).
- Tenant_class: `self-hosted`.
- MRP-run p99 (100k+ scale bucket) = base 18min × 1.30 × 1.30 = **30 min** → above the 25-min cell-target but within the 30-min worst-case bound; SAP S/4HANA on-prem equivalent at this scale is 30-60 min, so competitive.
- Schedule-recalc p99 (multi-plant 1k-10k ops) = base 25s × 1.30 × 1.30 = **42 s** → slightly above Kinaxis Maestro but well within SAP APO PP/DS interactive (5k ops at 30s).
- Shop-floor ingest sustained = base 10k events/sec × 1.30 × 1.30 = **5.9k events/sec** → above target floor.
- Dashboard render p99 (wide drill) = base 3s × 1.30 × 1.30 = **5.1s** → SAP Fiori p99 is ~5s; competitive.

### §10.3 Example C: Demo tenant on OCI Always Free
- Tenant: demo. Hard caps apply (§8.5).
- MRP-run: 1/hour, ≤10k materials → no SLO bind, hard cap.
- Schedule-recalc: 10/hour, ≤500 ops → no SLO bind, hard cap.
- Throughput cap dominates all overlays.

### §10.4 Example D: Pharma migrating from SAP, byo-cloud on AWS, FDA 21 CFR Part 11
- Tenant: 60k materials, 250 work centers, GxP-compliant.
- Context: `guest-on-aws`.
- Tenant_class: `byo-cloud`.
- MRP-run p99 (10k-100k bucket) = base 18min × 1.10 × 1.10 = **22 min** → within SAP S/4HANA cloud equivalent.
- Schedule-recalc p99 (1k-10k bucket) = base 25s × 1.10 × 1.10 = **30s** → within SAP `/SAPAPO/CDPSC`.
- Shop-floor ingest = base 1k × 1.10 × 1.10 = **826 events/sec**.
- FDA 21 CFR Part 11 audit-trail seal latency must remain p99 ≤ 1.5s (medium bucket); enforced by `slos/shop-floor-event-ingest.openslo.yaml`.

## §11 SLO Burn and Page Policy

For each workload's SLO file, the burn alert configuration is identical in shape:
- **Multi-window multi-burn-rate alerts** per Google SRE workbook:
  - Fast burn: 1h window, burn-rate ≥14.4 (2% of 30-day budget in 1h) → page
  - Medium burn: 6h window, burn-rate ≥6 (5% of 30-day budget in 6h) → page
  - Slow burn: 3d window, burn-rate ≥1 (10% of 30-day budget in 3d) → ticket
- **Per-workload escalation matrix**:
  - MRP-run completion: page on-call within 5 min on fast burn (manufacturing P0 since plan never produces).
  - Schedule recalc: page on-call within 2 min on fast burn (shop floor P0).
  - Scenario fork (post-IP-30): page within 15 min on fast burn (back-office UX).
  - Shop-floor ingest: page within 1 min on fast burn (real-time MES feedback critical).
  - Dashboard render: ticket within 4h on slow burn (UX latency, not P0).

## §12 Cross-Counterpart Performance Comparison Roll-Up

| Workload | Oyatie target (multi-tenant × oyatie-public-cloud) | SAP S/4HANA PP/DS | Oracle SCP Cloud | Kinaxis Maestro |
|---|---|---|---|---|
| MRP-run 10k materials p99 | 120s | 90s-180s | 60s-180s | not directly comparable (concurrent) |
| MRP-run 100k materials p99 | 18min | 5-30min | 10-25min | sub-minute for incremental |
| Schedule recalc 1k ops p99 | 5s | 1-5s | 2-10s | sub-second incremental, 1-5s full |
| Schedule recalc 10k ops p99 | 25s | 30s | 30s-60s | 5s full recompute |
| Scenario fork 1M rows p99 | 2s (post-IP-30) | 30s-5min | 1-3min | sub-second |
| Shop-floor ingest sustained (medium) | 1k events/sec | 10k events/sec (DMC) | 5-15k events/sec | via custom integration |
| Dashboard render overview p99 | 1.5s | 3-5s | 1.5s-4s | sub-second to 2s |

### §12.1 Where Oyatie wins
- Dashboard render parity with Kinaxis; better than SAP/Oracle.
- Schedule recalc parity for small ops; competitive for medium; slightly behind for large until IP-30.
- HTTP/3 + QUIC default saves ~80ms p99 per call vs all three counterparts.

### §12.2 Where Oyatie loses (and the closure path)
- **MRP-run at >100k materials**: 18min target is industry-median; not headline-fast. Closure: IP-43-MRP-MULTI + incremental net-change MRP (IP-43-NET-CHANGE).
- **Scenario fork**: IP-30 not yet implemented; ~2s target competitive when implemented but Kinaxis is sub-second today.
- **Shop-floor ingest large/enterprise**: 10k-50k events/sec is below SAP DMC's 10k+. Closure: IP-50-OPC-UA native ingest + parallel worker scaling.

### §12.3 Where parity is uncomparable
- Kinaxis Maestro's concurrent-planning model produces incremental recalc latencies that are not directly comparable to batch MRP / batch recalc. Phase-5 Oyatie target: implement concurrent planning (IP-30-CONCURRENT-PLANNING) to compete head-to-head.

## §13 Performance-Test Harness Roster (proposed)

These benchmarks should land as CI lanes per ADR-0328 §D-15.14:

1. `ci-mrp-run-perf` — synthetic 10k-material MRP run; threshold p99 ≤ 156s (base 120s × 1.30 regression headroom).
2. `ci-schedule-recalc-perf` — synthetic 1k-op recalc per strategy; threshold p99 ≤ 6.5s.
3. `ci-mes-ingest-perf` — sustained 1k events/sec for 60s; threshold seal-lag p99 ≤ 650ms.
4. `ci-dashboard-render-perf` — render 5 dashboards; threshold p99 ≤ 1.95s.
5. `ci-cedar-eval-perf` — 1M Cedar evaluations; threshold p99 ≤ 6.5ms.
6. `ci-audit-seal-perf` — 100k seal operations; threshold p99 ≤ 39ms.
7. `ci-cell-isolation-overhead` — bare-metal vs Cloud Hypervisor; threshold delta ≤ 7ms p99 (5ms base + 30% headroom).

Each CI lane uploads numerical evidence to the marketplace settlement contract per ADR-0314 so customers can audit performance claims independently.

## §14 Failure-Mode Performance Budget Erosion

When a failure mode fires, performance degrades by a known delta:

| Failure mode | MRP-run delta | Recalc delta | Ingest delta | Dashboard delta |
|---|---|---|---|---|
| Source-system import drift (PRD §F.failure-modes) | +50% | 0% | 0% | 0% |
| Cross-tenant denial (Cedar) | 0% | 0% | 0% | 0% (Cedar denial is fast-fail) |
| Duplicate command (idempotency) | 0% | 0% | 0% | 0% (idempotency cache hit) |
| Regional outage | +100% (write-queue) | +50% | +30% | +20% |
| Audit-chain outage | +30% (critical transitions pause) | +30% | +50% (seal-lag dominant) | +10% |
| MES drift > 5min | 0% | +20% (reconcile UC fires) | +100% (back-pressure) | +5% |

### §14.1 Composed degradation cap
The SLO file should bound composed degradation to ≤2.5× the base target before promotion to "service-impaired" status. Beyond 2.5×, an incident is declared per `runbooks/regional-failover.md` or equivalent.

## §15 Closing Note

Performance is the second hardest dimension of this microservice (after APS substance itself). The pre-2026-05-20 PRD declared a single flat target (p50<120ms / p95<300ms / p99<750ms) for every command in every bounded context; that target was physically false for MRP-run (which is ALWAYS measured in seconds-to-minutes, not milliseconds). This document supersedes that flat target with workload-specific industry-leader-anchored numbers + deployment-context overlay + tenant-class overlay.

When IP-30 (concurrent planning), IP-43 (MRP detail), and IP-50 (OPC-UA) land, the targets in this document tighten by ~2-5× on scenario-fork and shop-floor ingest. Until then, the targets here represent the floor a Phase-4 GA build must meet to be credible against SAP S/4HANA PP/DS, Oracle SCP Cloud, and Kinaxis Maestro.

End of performance benchmark numbers.
