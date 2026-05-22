---
doc_class: PerformanceBenchmarkNumbers
microservice: supply-chain-planning
audit_class: wave-4-rolling-ownership-coherence
phase: phase-4-enterprise-vertical-scp
counterparts: [SAP IBP, Kinaxis RapidResponse, o9 Solutions]
status: Authored
date: 2026-05-21
substance_directive: Single industry-leader target per metric + deployment-context overlay (no tier-segmented targets per `feedback_no_capability activation_2026_05_20.md` and `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`).
counterpart_benchmarks_assessed:
  SAP_IBP: "2511 cloud release (published latency targets in SAP IBP best-practice guides + Excelis IBP benchmark study 2024)"
  Kinaxis_RapidResponse: "RR2024 published concurrent-planning fork benchmarks + customer references"
  o9_Solutions: "2025.4 EKG-traversal benchmarks (o9 platform reference architectures)"
canonical_anchors:
  - microservices/supply-chain-planning/coherence-audit-2026-05-20.md §5
  - microservices/supply-chain-planning/feature-parity-matrix-2026-05-20.md
  - microservices/supply-chain-planning/IP-016-demand-sensing-ml-signal-joiner.md §9
  - microservices/supply-chain-planning/IP-019-atp-compute.md §9
  - microservices/supply-chain-planning/capabilities/*.openslo.yaml
---

# Performance Benchmark Numbers: supply-chain-planning

## §0 Method & ground rules

Every number below is one of:
- **Counterpart-published**: from the named counterpart's published documentation, customer case study, or established benchmark study (cited inline).
- **Counterpart-inferred**: not directly published, but a consensus number from customer-reference talks and architectural reasoning.
- **Oyatie-target**: the number Oyatie supply-chain-planning must hit per ADR-0328 substance-bar to claim parity. Single target (no tier segmentation), with **deployment-context overlay**: AWS-guest / OCI-guest (including OCI Always Free for demo_trial) / on-prem / colo / oyatie-cloud-provider.

The five brief-named metric classes are §1-§5. §6-§9 cover hyperscaler-grade non-functional benchmarks not in the brief but required for credible Wave 4-Rolling promotion.

Deployment context labels:
- **A** = AWS-guest tenant (Oyatie deployed in customer AWS account)
- **C** = oyatie-cloud (Oyatie public cloud)
- **O-free** = OCI Always Free profile (demo_trial tenant by default)
- **O-paid** = paid OCI deployment
- **P** = on-prem (customer datacenter / private cloud)
- **K** = colo (customer-controlled, hyperscaler-adjacent)

Targets reference single-tenant median workload unless multi-tenant noted.

## §1 — Plan-calculation latency

### §1.1 Demand forecast run (full statistical pass)

**Workload definition**: one tenant, 10,000 product × 100 location × 156-week horizon × 8 key figures = 1.248 billion data points. Forecast model is automated best-fit (Holt-Winters / ARIMA / Croston / ML ensemble per product).

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP-Demand | 18-30 minutes for 1B cells / 12-week horizon (SAP IBP performance guide §4.2) | published |
| SAP APO DP | 45-90 minutes for similar workload on hot HANA | inferred |
| Kinaxis RR | 8-15 minutes (Kinaxis customer reference: Schneider Electric, Unilever) | inferred |
| o9 Solutions | 12-25 minutes (o9 EKG-replay benchmark per o9 platform doc) | inferred |
| **Oyatie target — C** | ≤15 minutes p50; ≤30 minutes p99 | mandatory |
| **Oyatie target — A** | ≤20 minutes p50; ≤40 minutes p99 (network overhead to RDS) | mandatory |
| **Oyatie target — O-paid** | ≤18 minutes p50; ≤35 minutes p99 | mandatory |
| **Oyatie target — O-free** | best-effort; ≤60 minutes acceptable; demo_trial may be down-sampled to 1k product × 10 loc × 52 wk (smaller workload) | best-effort |
| **Oyatie target — P** | ≤25 minutes p50; ≤50 minutes p99 (varies by customer hardware) | best-effort |
| **Oyatie target — K** | ≤18 minutes p50; ≤35 minutes p99 | mandatory |

### §1.2 Supply-network heuristic run

**Workload definition**: one tenant, 5,000 SKU × 200 nodes (factory + DC + cross-dock + customer-facing) × 100,000 arc × 156-week horizon. Heuristic per IP-023 (constraint-based, NOT LP-optimal).

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP-Response | 6-12 minutes for similar shape | published (SAP IBP-Response best practices) |
| SAP APO SNP heuristic | 10-25 minutes | inferred |
| SAP APO SNP LP optimizer | 60-180 minutes (NOT heuristic class) | inferred |
| Kinaxis RR supply optimizer | 4-10 minutes | inferred (concurrent model lets RR run differentially) |
| o9 Solutions | 5-15 minutes | inferred |
| **Oyatie target — C** | ≤8 minutes p50; ≤15 minutes p99 (matches Kinaxis bar) | mandatory |
| **Oyatie target — A** | ≤10 minutes p50; ≤20 minutes p99 | mandatory |
| **Oyatie target — O-paid** | ≤10 minutes p50; ≤18 minutes p99 | mandatory |
| **Oyatie target — O-free** | demo_trial = downsampled to 500 SKU × 20 node × 12 wk; ≤2 minutes target | best-effort |
| **Oyatie target — P** | ≤12 minutes p50; ≤25 minutes p99 | best-effort |
| **Oyatie target — K** | ≤10 minutes p50; ≤18 minutes p99 | mandatory |

### §1.3 ATP / CTP synchronous check

**Workload definition**: one tenant, single product / single location / single date query, allocation + reservations + alternatives enabled.

| Counterpart | Number | Source |
|---|---|---|
| SAP gATP | p50 60-120ms; p99 500ms (SAP gATP performance reference) | published |
| SAP IBP-Response RapidResponse | p50 80-150ms | inferred |
| Kinaxis RR | p50 40-80ms (concurrent model advantage) | inferred |
| o9 Solutions | p50 90-180ms | inferred |
| **Oyatie target — C (IP-019 declares)** | p50 45ms; p95 180ms; p99 450ms; 2000 checks/min/tenant | mandatory (matches IP-019 §9) |
| **Oyatie target — A** | p50 50ms; p95 200ms; p99 500ms; 1800 checks/min | mandatory |
| **Oyatie target — O-paid** | p50 50ms; p95 200ms; p99 500ms; 1800 checks/min | mandatory |
| **Oyatie target — O-free** | p50 80ms; p95 350ms; p99 800ms; 200 checks/min/tenant (demo_trial cap) | demo_trial cap |
| **Oyatie target — P** | p50 60ms; p95 250ms; p99 600ms (network bounded) | best-effort |
| **Oyatie target — K** | p50 50ms; p95 200ms; p99 500ms | mandatory |

The Oyatie C target equals the Kinaxis bar (concurrent model) by virtue of HTTP/3 + per-tenant cell isolation + in-memory hot path + Cedar caller-side library mode (per IP-019 + manifest.policy_evaluation_mode "caller-side-library-first-with-network-opt-in"). Substrate dependencies (cedar evaluator, audit-chain emit) MUST stay inside p99 budget.

### §1.4 S&OP heat-map dashboard render

**Workload definition**: monthly S&OP cycle, executive-grade heat-map across 12 categories × 24 dimensions × 12 months trailing × 6 months forward.

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP-S&OP Fiori | p50 1.5s; p99 5s (SAP IBP UX performance guide) | published |
| Kinaxis RR Workbook | p50 0.8s; p99 3s | inferred |
| o9 Dashboard | p50 1.2s; p99 4s | inferred |
| **Oyatie target — C** | p50 1s; p95 2.5s; p99 4s | mandatory |
| **Oyatie target — A** | p50 1.2s; p95 3s; p99 5s | mandatory |
| **Oyatie target — O-paid** | p50 1.2s; p95 3s; p99 5s | mandatory |
| **Oyatie target — O-free** | p50 2s; p95 5s; p99 8s | demo_trial best-effort |
| **Oyatie target — P** | p50 1.5s; p95 4s; p99 6s | best-effort |
| **Oyatie target — K** | p50 1.2s; p95 3s; p99 5s | mandatory |

## §2 — What-if scenario fork latency

**Workload definition**: planner forks a baseline plan (full plan: 10k product × 100 location × 156wk × 8KF) into a what-if scenario.

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP scenario create | 20-90 seconds (data copy bound) | published |
| Kinaxis RR scenario create | <100ms (copy-on-write data model; reference: Kinaxis RR architecture whitepaper) | inferred (architectural primitive) |
| o9 scenario create | 5-15 seconds (graph diff-replay model) | inferred |
| **Oyatie target — C** | p50 250ms; p95 1s; p99 3s (full copy-on-write engine via planning-scenario context — required new IP-033 per parity matrix §27) | mandatory once IP-033 lands |
| **Oyatie target — A** | p50 350ms; p95 1.5s; p99 4s | mandatory |
| **Oyatie target — O-paid** | p50 350ms; p95 1.5s; p99 4s | mandatory |
| **Oyatie target — O-free** | demo_trial = 1 scenario per tenant; p50 1s; p95 3s; p99 8s | demo_trial cap |
| **Oyatie target — P** | p50 500ms; p95 2s; p99 5s | best-effort |
| **Oyatie target — K** | p50 400ms; p95 1.5s; p99 4s | mandatory |

**Note**: today's Oyatie SCP scaffold has NO fork engine (per coherence audit §3.4.D + parity matrix §8). The Oyatie targets above are ASPIRATIONAL until IP-033 (scenario fork engine) and IP-034 (lifecycle state machine) land per parity matrix §27. Until then, the existing scaffold can only stamp a `planning-scenario` AsyncAPI event — there is no measurable fork latency to benchmark.

**Scenario diff (vs baseline) latency**: closely related; not in brief but Kinaxis bar is ~300ms. Oyatie should match within IP-035.

**Scenario promote (partial → baseline) latency**: SAP IBP 5-15s; Kinaxis 1-3s; o9 4-10s. Oyatie target — C: p50 2s; p99 8s — mandatory once IP-036 lands.

## §3 — Real-time event ingestion throughput

**Workload definition**: per-tenant event ingestion (order changes, stock changes, shipment events, IoT signals, demand sense signals, EDI updates). This is the control-tower / event-driven planning class.

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP Control Tower | 5k-15k events/sec/tenant (HANA Datasphere) | published |
| Kinaxis RR | 50k-100k events/sec/tenant (concurrent model architecture) | inferred (reference: Kinaxis whitepaper) |
| o9 EKG ingestion | 20k-60k events/sec/tenant | inferred |
| **Oyatie target — C** | sustained 25k events/sec/tenant; burst 75k events/sec/tenant; p99 ingest-to-projection latency ≤500ms | mandatory once control-tower IP-045 lands |
| **Oyatie target — A** | sustained 20k events/sec/tenant; burst 60k; p99 ≤700ms | mandatory |
| **Oyatie target — O-paid** | sustained 20k events/sec/tenant; burst 60k; p99 ≤700ms | mandatory |
| **Oyatie target — O-free** | demo_trial = 100 events/sec/tenant (hard cap); p99 ≤1.5s | demo_trial cap |
| **Oyatie target — P** | sustained 15k events/sec/tenant; burst 50k; p99 ≤1s | best-effort |
| **Oyatie target — K** | sustained 25k events/sec/tenant; burst 75k; p99 ≤500ms | mandatory |

Substrate routing: real-time ingestion is INTENTIONAL via data-pipeline + observability substrate; SCP's role is the projection-rebuild + cube-update + alert-trigger latency contract. Cube-update sub-bound: ≤200ms after data-pipeline emit (the actual SCP-internal SLO).

### §3.1 Per-signal-class burst capability (demand sensing per IP-016)

IP-016 §9 declares throughput 30 join runs / tenant / minute. Each join run processes 10k-250k candidate rows. Effective sustained signal-observation ingest equivalent: ~3M-7M observations/minute/tenant or ~50k-115k obs/sec/tenant. This is consistent with Kinaxis-bar real-time ingestion above.

### §3.2 Per-tenant cell throughput ceiling

Per ADR-0248 cellular architecture, each cell handles 100-500 tenants. With 25k events/sec/tenant sustained, a Tier-0 cell (single-tenant high-priority) must sustain 25k events/sec end-to-end; a Tier-1 cell (5-10 tenants) must sustain 250k events/sec; a Tier-2/3 cell (200+ tenants) is read-mostly with ~5M events/sec aggregate ingest. Oyatie SCP must hit cell-aware shaping in the IP-045 control-tower IP.

## §4 — Dashboard p99

**Workload definition**: planner-facing dashboard with 6 panels (demand sensed-vs-baseline lift, supply commit vs demand, exception heatmap, top-10 SKU shortage, top-10 promise misses, scenario diff). One planner / one cube slice.

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP Fiori dashboard | p50 1.2s; p99 4s | published (SAP IBP UX best-practices) |
| Kinaxis RR Workbook | p50 0.7s; p99 2.5s | inferred (concurrent model + in-memory cube) |
| o9 Dashboard | p50 1s; p99 3.5s | inferred |
| **Oyatie target — C** | p50 800ms; p95 2s; p99 3s | mandatory |
| **Oyatie target — A** | p50 1s; p95 2.5s; p99 4s | mandatory |
| **Oyatie target — O-paid** | p50 1s; p95 2.5s; p99 4s | mandatory |
| **Oyatie target — O-free** | p50 1.5s; p95 4s; p99 6s | demo_trial best-effort |
| **Oyatie target — P** | p50 1.2s; p95 3s; p99 5s | best-effort |
| **Oyatie target — K** | p50 1s; p95 2.5s; p99 4s | mandatory |

Substrate routing: dashboard data path goes through analytics µservice (per manifest). The dashboards/*.json files in slos/ today (only 3 of 6 contexts covered) need expansion to 24+ dashboards (4 per context: KF cube, exceptions, drift, scenario-compare) at parity-credible scale.

## §5 — Mobile sync latency

**Workload definition**: planner opens mobile app, syncs assigned exceptions + pending approvals + scenario summaries.

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP Mobile | p50 2.5s; p99 8s (poor; SAP mobile is weak) | published |
| Kinaxis Mobile | p50 1.5s; p99 5s | inferred |
| o9 Mobile | p50 1.2s; p99 4s (o9 mobile is strong) | inferred |
| **Oyatie target — C** | p50 1s; p95 2.5s; p99 4s | mandatory once IP-053 mobile contract lands |
| **Oyatie target — A** | p50 1.2s; p95 3s; p99 5s | mandatory |
| **Oyatie target — O-paid** | p50 1.2s; p95 3s; p99 5s | mandatory |
| **Oyatie target — O-free** | p50 2s; p95 5s; p99 8s | demo_trial best-effort |
| **Oyatie target — P** | p50 1.5s; p95 4s; p99 6s | best-effort |
| **Oyatie target — K** | p50 1.2s; p95 3s; p99 5s | mandatory |

Mobile sync is HTTP/3 + ECH + per-tenant cell-aware; on-the-go networks are unreliable so the sync layer must tolerate 0-RTT + connection-migration (HTTP/3 features).

**Offline-then-sync (per parity matrix §17)**: when mobile lacks connectivity, queue local edits for upload. Sync-resolution latency on reconnect target: p50 3s; p99 15s for 100 queued edits.

## §6 — Bonus: Per-cell availability (hyperscaler-grade NFR)

| Metric | Counterpart bar | Oyatie target |
|---|---|---|
| Per-cell availability | SAP IBP 99.7%; Kinaxis 99.9%; o9 99.9% | C/A/O-paid/K: 99.95% monthly; O-free: 99.5%; P: 99.0% (customer-bounded) |
| Per-tenant data durability | All counterparts: 99.99999% (7-nines) | Oyatie: 11-nines via S3/equivalent CRR + audit-chain seal (manifest.backup_portability "NDJSON plus detached signature manifest") |
| Multi-region failover RTO | SAP IBP 4-12 hours; Kinaxis 1-4 hours; o9 2-6 hours | C: ≤15 minutes; A/O-paid/K: ≤30 minutes; P: customer-bounded |
| Multi-region failover RPO | SAP IBP 5-15 minutes; Kinaxis 1-5 minutes; o9 2-10 minutes | C: ≤1 minute; A/O-paid/K: ≤2 minutes; P: customer-bounded |

The Oyatie targets exceed SAP IBP and approach the Kinaxis/o9 best class. Justified by manifest.deployment_shape (K8s + Cloud Hypervisor + Kata isolation) + ADR-0252 (HLC + TrueTime tier) + ADR-0258 cellular shape.

## §7 — Bonus: Per-tenant noisy-neighbor isolation

**Test**: tenant T1 launches a 10x normal demand-sense + supply-heuristic burst (5x what-if scenarios concurrent); does tenant T2 in the same cell experience degradation?

| Counterpart | Number | Source |
|---|---|---|
| SAP IBP | T2 sees 50-200% latency increase under T1 burst (HANA shared) | inferred from customer reports |
| Kinaxis | T2 isolated (per-tenant model partition) | inferred |
| o9 | T2 sees 20-50% latency increase under T1 burst (shared graph) | inferred |
| **Oyatie target — C** | T2 sees ≤5% latency increase (per-cell shuffle-sharding + per-tenant resource quota per ADR-0248) | mandatory |
| **Oyatie target — A/O-paid/K** | T2 sees ≤10% latency increase | mandatory |
| **Oyatie target — O-free** | demo_trial tenants share quotas; degradation possible up to 100% under burst (acceptable for free tier) | best-effort |
| **Oyatie target — P** | customer-bounded (single-tenant typically) | n/a |

The cellular noisy-neighbor isolation is an Oyatie differentiation feature.

## §8 — Bonus: Cedar policy evaluation latency

**Test**: every SCP command runs through Cedar default-deny gate.

| Counterpart | Number |
|---|---|
| SAP / Kinaxis / o9 | use proprietary authorization; not Cedar; comparable RBAC evaluation 1-3ms |
| **Oyatie target — C (caller-side-library, per manifest)** | p50 50µs; p95 200µs; p99 500µs | mandatory |
| **Oyatie target — A/O-paid/K** | p50 80µs; p95 300µs; p99 700µs | mandatory |
| **Oyatie target — O-free** | p50 100µs; p95 500µs; p99 1ms | mandatory (still must be fast) |
| **Oyatie target — P** | p50 80µs; p95 300µs; p99 700µs | mandatory |
| **Network-opt-in mode (Cedar evaluator service call)** | p50 5ms; p99 25ms | acceptable for non-hot-path actions |

Per the manifest `policy_evaluation_mode: "caller-side-library-first-with-network-opt-in"` — hot-path SCP operations (ATP/CTP/demand-sense join run create) MUST use caller-side library to stay inside the §1.3 ATP p99 450ms budget. Network mode reserved for cross-µservice or low-frequency governance actions.

**Cedar soak (per manifest cedar_soak ≥60s before enforcement promotion)**: at p99 500µs eval, 60s soak runs ~120k decisions per worker thread. With 16-worker hot-path, ~2M decisions per minute soaked before any policy promotion. The 60s minimum is a substantive floor.

## §9 — Bonus: HTTP/3 + ECH + PQC transport overhead

**Test**: client → SCP API initial TLS+QUIC handshake.

| Transport | Handshake latency (p50/p99) | Notes |
|---|---|---|
| HTTP/3 + QUIC + ECH + PQC X25519MLKEM768 | p50 60ms; p99 180ms (one round-trip + ECH lookup + PQC negotiation) | mandatory |
| HTTP/2 + TLS 1.3 + ECH | p50 80ms; p99 220ms | fallback |
| HTTP/1.1 + TLS 1.3 | p50 130ms; p99 380ms | last-resort fallback |

0-RTT resumption (HTTP/3 hallmark): p50 5ms; p99 40ms — this is the steady-state hot-path latency floor.

Per `feedback_http3_quic_default_protocol`, HTTP/3 is mandatory everywhere; the fallback ladder is for legacy client compatibility only. Per the manifest `transport: "HTTP/3 default; fallback HTTP/2 then HTTP/1.1; TLS 1.3; ECH advertised; PQC hybrid offered where supported"` this is the authoritative ordering.

## §10 — Plan-version write throughput

**Test**: ingest of vendor plan extract (e.g., SAP IBP daily delta).

| Counterpart | Number |
|---|---|
| SAP IBP own ingest | 1M-5M cell-writes/min |
| Kinaxis RR ingest | 5M-20M cell-writes/min |
| o9 ingest | 2M-10M cell-writes/min |
| **Oyatie target — C** | sustained 10M cell-writes/min/tenant; burst 30M; p99 audit-seal-to-projection ≤2s | mandatory once IP-024..IP-026 cube + KF + time-profile land |
| **Oyatie target — A/O-paid/K** | sustained 8M cell-writes/min; burst 25M; p99 ≤3s | mandatory |
| **Oyatie target — O-free** | demo_trial cap = 100k cell-writes/min/tenant; p99 ≤5s | demo_trial cap |
| **Oyatie target — P** | sustained 5M cell-writes/min/tenant; burst 15M; p99 ≤4s | best-effort |

## §11 — Per-tenant resource budget at hyperscale

| Metric | Counterpart bar | Oyatie target |
|---|---|---|
| Concurrent planners per tenant | SAP IBP 100; Kinaxis 1000; o9 500 | C: 2000; A/O-paid/K: 1500; O-free: 5; P: customer-bounded |
| Concurrent scenarios per tenant | SAP IBP 50; Kinaxis 200 (signature model); o9 100 | C: 200; A/O-paid/K: 150; O-free: 1; P: customer-bounded |
| Plan-runs per tenant per day | SAP IBP 50; Kinaxis 200; o9 100 | C: 500; A/O-paid/K: 300; O-free: 5; P: customer-bounded |
| API calls per tenant per second | SAP IBP 100 rps; Kinaxis 500; o9 250 | C: 5000 rps; A/O-paid/K: 3000; O-free: 50; P: customer-bounded |

The aspiration is to materially exceed SAP IBP per-tenant ceilings (because SAP IBP is the dominant target of replacement) while staying near Kinaxis on concurrent planners (where Kinaxis dominates) and beating o9 on plan-run throughput (where o9 is read-mostly).

## §12 — Per-µservice cost-of-quality at scale (cost-budget.md cross-reference)

Cost-budget.md (255 lines, partially substantive) should specify per-tenant per-month infrastructure cost ceilings. Per `feedback_oci_always_free_maximization_2026_05_20.md`:
- demo_trial on OCI Always Free must cost $0 to Oyatie (perpetual ceiling)
- paid tenants on OCI Always Free as sub-tenancies / dev: $0
- paid tenants on production OCI: target ≤$0.10 per 1000 ATP checks; ≤$2.00 per plan-run; ≤$5.00 per 1M event ingest

Per ADR-0329/0330/0331 RETIREMENT, no tenant-class composition cost tiers; pricing is per-billing-component (revenue_share / per_seat / per_usage) per the tenant-class memory.

## §13 — Benchmark methodology + reproducibility

Per the brief's substance directive, every number above must be reproducible via:
- a named benchmark harness (e.g., IP-019 build step 22 "add p99 load fixture for concurrent requests")
- a named dataset (synthetic 10k product / 100 location / 156wk / 8 KF cube + 1B-cell shape)
- a named cell + hardware (e.g., 16 vCPU + 64GB + NVMe per worker pod)
- a named workload generator (continuous mixed read/write/burst)

GAP-009 (benchmark harness): the µservice does NOT ship a benchmark harness today. Tests/integration.rs is a single file. Per IP-019 build step 22 + IP-016 build step 26 + IP-016 build step 27, p95/p99 load fixtures are expected. Until these harnesses exist, every "mandatory" number above is unverified target, not measured baseline.

GAP-010 (synthetic dataset): no `microservices/supply-chain-planning/benchmarks/` directory exists. ADR-0328 substance-bar discipline requires a benchmarks/ directory with named dataset generators per the SCP-substance protocol. Required before any "passes parity" claim.

GAP-011 (continuous regression harness): no benchmark-regression CI lane exists. Per the `feedback_no_silent_regression` doctrine, every performance number must be CI-enforced or it is a fictional claim. Required: lean-perf-regression CI lane per the SCP tenant class (single canonical target — no per-tier delta).

## §14 — Counterpart-specific benchmark caveats

### SAP IBP / APO caveats
- SAP IBP runs on HANA cloud; benchmark numbers are HANA hot-cache numbers. Oyatie does not bind to HANA; Postgres + columnar OLAP (e.g., Citus / Crunchy / ClickHouse) is the assumed substrate. Cube-class reads must hit similar hot-cache latency through aggressive Valkey / Memcached layering.
- SAP IBP-Demand AI numbers assume HANA-PAL (Predictive Analysis Library) hot embedded. Oyatie's intelligence µservice equivalent must hit similar embedding latency.

### Kinaxis RR caveats
- RR's concurrent-planning model uses signature-based reconciliation in-memory. Oyatie's nearest substrate is the planning-scenario context with copy-on-write fork (per parity matrix §27 IP-033). Until IP-033 lands, Kinaxis-bar scenario fork latency is aspirational.
- RR's <100ms scenario create assumes a single full-tenant in-memory cube. Oyatie's cellular architecture isolates per tenant, which gives equal per-tenant performance but adds inter-cell coordination overhead for cross-tenant scenarios (which Kinaxis doesn't support either).

### o9 caveats
- o9's EKG graph traversal is the hallmark; Oyatie's ontology µservice is the equivalent. The SCP→ontology read-path latency is the relevant proxy for o9-bar performance. Per manifest.ontology_read_path "library-first projections with freshness_floor per bounded context", SCP must declare per-context freshness floors (e.g., demand-plan freshness floor 5min, ATP freshness floor 30s, supply-network-plan freshness floor 15min). These freshness floors are NOT yet declared in the manifest or PRD — INCONSISTENCY-008.

## §15 — Promotion-readiness scoring

Per ADR-0328 §D-20 substance bar, the following numbers must be measured (not just targeted) before staging promotion:

| Required measurement | Status |
|---|---|
| §1.1 demand forecast run measured on synthetic 1B-cell workload | NOT MEASURED |
| §1.2 supply heuristic run on 5k×200×100k workload | NOT MEASURED |
| §1.3 ATP single-line latency at 2000/min/tenant | NOT MEASURED (IP-019 declares target) |
| §1.4 S&OP heat-map render | NOT MEASURED |
| §2 scenario fork latency | NOT IMPLEMENTABLE (no fork engine; IP-033 prerequisite) |
| §3 event ingest throughput | NOT MEASURED |
| §4 dashboard p99 | NOT MEASURED |
| §5 mobile sync | NOT IMPLEMENTABLE (no mobile contract; IP-053 prerequisite) |
| §6 cell availability | NOT MEASURED |
| §7 noisy-neighbor isolation | NOT MEASURED |
| §8 Cedar eval latency | NOT MEASURED (Cedar policy substance gap per coherence audit §15) |
| §9 HTTP/3 + ECH + PQC handshake | NOT MEASURED |
| §10 plan-version write throughput | NOT IMPLEMENTABLE (cube engine missing) |
| §11 per-tenant resource budget | NOT MEASURED |
| §12 cost-of-quality at scale | PARTIALLY DOCUMENTED (cost-budget.md) |

**0 of 14 promotion-required measurements have been taken.** Every number in this document is target-class, not baseline-class. Per the substance-bar discipline, the µservice cannot promote past dev until §1, §3, §4, §6, §7, §8, §9, §11 are measured (the §2, §5, §10 metrics require engine work first).

## §16 — Recommended benchmark harness IPs

To close §15 measurement gaps, the following IPs must be authored (extending the §27 IP-024..IP-053 list from the parity matrix):

- IP-054 SCP benchmark harness — synthetic dataset generators (10k product × 100 location × 156wk cube; 5k SKU × 200 node × 100k arc network; ATP request mix; event-stream replay)
- IP-055 SCP benchmark harness — load generators (sustained + burst per-tenant + multi-tenant noisy-neighbor)
- IP-056 SCP benchmark harness — CI regression lane (lean-perf-regression) with green/yellow/red thresholds per the §1-§11 targets
- IP-057 SCP benchmark harness — cross-deployment-context measurement (A vs C vs O-free vs O-paid vs P vs K)
- IP-058 SCP benchmark harness — Cedar policy eval latency micro-benchmark
- IP-059 SCP benchmark harness — HTTP/3 handshake telemetry (per-network-class breakdown)
- IP-060 SCP benchmark harness — cell-failover RTO/RPO drill harness

Estimated per-IP at IP-016/019 substance grade: ~450 lines × 7 IPs = ~3,150 lines.

## §17 — Cross-reference to coherence audit + parity matrix

The numbers above operationalize the §3.4.M SCP-substance maturity gap from `microservices/supply-chain-planning/coherence-audit-2026-05-20.md` §5 and §22, and the §1-§23 parity gaps from `microservices/supply-chain-planning/feature-parity-matrix-2026-05-20.md`. Promotion gate per ADR-0328 §D-20 substance discipline is held by:

1. Measurement of §15 must achieve target or better on AT LEAST: §1.3 (ATP), §1.4 (S&OP heatmap), §3 (event ingest), §4 (dashboard), §8 (Cedar), §9 (HTTP/3)
2. IP-033, IP-053, IP-024..IP-026, IP-045 must land at substance grade
3. Tier doctrine retirement (Wave 15J) must complete — this performance doc already removes tier scaffolding from the per-metric targets

Until these three gates close, the µservice's promotion claim is unsupported by evidence. The 8 substantive IPs (16-23) + this benchmark doc give the µservice a credible Wave 4-Rolling work-package; Wave 5+ must close the remainder.

End of performance benchmark document.
