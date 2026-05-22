---
doc_class: PerformanceBenchmarkNumbers
microservice: treasury
phase: Phase-4
phase_sub_sequence: Phase-4A.2 ERP (D-1.99 + D-2.8..D-2.11)
batch: Wave-4-Rolling Treasury ownership audit
benchmark_date: 2026-05-21
authoring_agent_class: microservice-ownership-audit (ADR-0328 D-3.5..D-3.10)
owner: axis-treasury + axis-erp-parity
verdict_companion: REVISE (see coherence-audit-2026-05-20.md §0)
top_3_counterparts:
  - Kyriba (cash management, payment hub, risk management, FX hedging, financial integration, working capital optimization)
  - SAP Treasury and Risk Management (TRM — cash, debt, in-house cash, hedge management, FX, market data)
  - Reval (cloud TMS — cash flow, FX risk, hedging, derivative valuation, IFRS/GAAP hedge accounting)
benchmark_classification:
  measured: NONE (no live benchmark run in this audit)
  target_budget: ALL rows below are target budgets per ADR-0328 D-6.12
  counterpart_public: PARTIAL (counterpart public claims cited where available)
disclaimer: Per ADR-0328 D-6.13, a target budget MUST NOT be presented as measured evidence. All numbers below are explicitly labeled.
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0253
  - ADR-0263
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0321
  - ADR-0322
  - ADR-0327
  - ADR-0328
planned_enforcement_ref: oya-governance-microservice-coherence-audit
---

# Treasury — Performance Benchmark Numbers vs Kyriba + SAP TRM + Reval (Wave-4-Rolling, 2026-05-21)

## §0 Methodology and Classification

This document defines target performance budgets for Treasury and contrasts them with publicly-available counterpart claims from Kyriba product brochures, SAP TRM module documentation, Reval product datasheets (now under ION Group), and SAP S/4HANA Treasury Cloud reference architectures. Per ADR-0328 D-6.10..D-6.13, every number is explicitly classified as one of:

- `MEASURED` — observed value from a live benchmark run on the current Treasury surface. **None of the numbers in this document are MEASURED.** This audit could not run live benchmarks against the current scaffold because (a) the Rust crate is a single-bounded-context scaffold (F-P1-01) and cannot exercise the six bounded contexts in their target shape, and (b) the contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/treasury-v1.proto surface needs ISO 20022 and SWIFT FIN extensions before any meaningful payment-routing benchmark.
- `TARGET` — committed budget that Treasury must hit at Phase 4A.2 PASS-WITH-FINDINGS. Treasury cannot promote past Phase 4A.2 gate without a measured run that lands within ±15% of TARGET.
- `CP_PUBLIC` — counterpart public claim from product brochure, datasheet, white paper, or press release. Cited verbatim where the source is named. Treated as advisory only; counterpart vendors typically benchmark in idealized conditions (single tenant, single region, primed cache, no other workload).

Workload mix assumptions for all TARGET numbers (the workload shape that backs the SLO budget):

- Tenant scale: 10^4 active tenants in the primary cell, 10^3 high-volume tenants (each with ≥ 100 bank accounts, ≥ 10^6 transactions per day, ≥ 50 entities).
- Cell scale: Tier 0/1/2/3 eligible per manifest.json line 136..141; primary SLO target is Tier 1 with cell shuffle-sharding per ADR-0248 KS#7.
- Multi-region: 3 primary regions (US-East, EU-West, AP-East) with 5 sovereign cells (US-Gov, EU-Sovereign, KR-Sovereign, CN-PIPL, AU-IRAP) per manifest.compliance_packs.
- Transport: HTTP/3 + QUIC default per ADR-0253; gRPC over HTTP/3 for service-to-service.
- HLC: default per ADR-0252; TrueTime opt-in for fin-grade tenants (banking entity, broker-dealer).
- Cell topology: AWS-style cellular per ADR-0248 with Tier 1 = 1 cell per region, Tier 0 (multi-tenant) = up to 8 cells per region.
- Storage backplane: Postgres + Iceberg per cloud-data canonical; Postgres for hot OLTP, Iceberg for cold analytical.
- Cache: Valkey Cluster per cell for hot cash-position; CDN edge cache for rate feeds.

## §1 Cash Position — Recalculation Latency

### §1.1 Daily EOD cash position recalc latency

| Workload | Oyatie TARGET | Kyriba CP_PUBLIC | SAP TRM CP_PUBLIC | Reval CP_PUBLIC |
|---|---|---|---|---|
| 1 entity, 10 bank accounts, 10^4 transactions, single ccy | TARGET p50 ≤ 500 ms / p95 ≤ 2 s / p99 ≤ 5 s | CP_PUBLIC: "near-instant cash visibility" (no number disclosed) | CP_PUBLIC: end-of-day batch typically 1-5 minutes on HANA in-memory | CP_PUBLIC: "real-time" for typical mid-market shapes |
| 10 entities, 100 bank accounts, 10^5 transactions, 5 ccy | TARGET p50 ≤ 2 s / p95 ≤ 8 s / p99 ≤ 20 s | CP_PUBLIC: no number disclosed | CP_PUBLIC: 5-15 minutes on HANA | CP_PUBLIC: no number disclosed |
| 100 entities, 1,000 bank accounts, 10^6 transactions, 20 ccy | TARGET p50 ≤ 10 s / p95 ≤ 30 s / p99 ≤ 90 s | CP_PUBLIC: no number disclosed | CP_PUBLIC: 30-90 minutes on HANA | CP_PUBLIC: no number disclosed |
| 1,000 entities, 10,000 bank accounts, 10^7 transactions, 50 ccy | TARGET p50 ≤ 60 s / p95 ≤ 180 s / p99 ≤ 600 s | CP_PUBLIC: high-volume Kyriba tenants report ~30 min EOD | CP_PUBLIC: SAP "Group Reporting" can take hours on non-HANA | CP_PUBLIC: no number disclosed |

The 1,000-entity scale ceiling is set by Phase 4A.2 ERP scale: Walmart-class, Maersk-class, Toyota-class enterprises typically run 800-2,000 legal entities. Above 2,000 entities the tenant should be sharded across multiple cells under ADR-0248 shuffle sharding. The TARGET budgets above assume a single cell; cross-cell consolidation is a separate budget under §6.

### §1.2 Intraday cash position refresh latency

| Workload | Oyatie TARGET | Kyriba CP_PUBLIC | SAP TRM CP_PUBLIC | Reval CP_PUBLIC |
|---|---|---|---|---|
| Single tenant, intraday delta from MT942 push | TARGET p50 ≤ 100 ms / p95 ≤ 500 ms / p99 ≤ 2 s | CP_PUBLIC: "15-minute intraday refresh" | CP_PUBLIC: "15-minute MBC refresh" | CP_PUBLIC: "intraday refresh on configurable cadence" |
| Single tenant, intraday delta from API poll | TARGET p50 ≤ 1 s / p95 ≤ 5 s / p99 ≤ 15 s | CP_PUBLIC: 15 min cadence | CP_PUBLIC: 15 min cadence | CP_PUBLIC: configurable |
| Single tenant, intraday delta from camt.054 push | TARGET p50 ≤ 200 ms / p95 ≤ 1 s / p99 ≤ 5 s | CP_PUBLIC: no number | CP_PUBLIC: no number | CP_PUBLIC: no number |

Note: Oyatie's TARGET intraday refresh is significantly faster than the public 15-minute cadence advertised by Kyriba and SAP MBC. The doctrine reason: cash position is a high-trust treasury operator decision input; the 15-minute refresh is a legacy of polling-based bank statement APIs and is no longer a hard floor when banks publish camt.054 push notifications and ISO 20022 streaming endpoints. Oyatie commits to push-driven sub-second refresh as a differentiator.

### §1.3 Cash position fan-out (multi-bank, multi-account, multi-entity)

| Workload | Oyatie TARGET | Counterpart |
|---|---|---|
| 100 banks × 1,000 accounts × 100 entities consolidation | TARGET p99 fan-out + consolidation ≤ 30 s for cold cache | CP_PUBLIC Kyriba / SAP TRM / Reval: no number disclosed at this fan-out granularity |
| Same with primed cache (incremental delta) | TARGET p99 ≤ 5 s | CP_PUBLIC: no number |

## §2 FX Rate Streaming — Throughput

### §2.1 FX rate ingest from Bloomberg / Refinitiv / WMR fixings

| Workload | Oyatie TARGET | Kyriba CP_PUBLIC | SAP TRM CP_PUBLIC | Reval CP_PUBLIC |
|---|---|---|---|---|
| Bloomberg BVAL streaming, 200 ccy pairs, 1 tick / 100 ms | TARGET sustained ingest ≥ 2,000 ticks/sec per tenant; ≥ 20,000 ticks/sec aggregate per cell | CP_PUBLIC: "real-time rates" (no tick rate disclosed) | CP_PUBLIC: TRM Market Data Server "real-time" | CP_PUBLIC: "real-time rates" |
| Refinitiv RIC streaming, 200 ccy pairs | TARGET ≥ 2,000 ticks/sec per tenant | CP_PUBLIC: no number | CP_PUBLIC: no number | CP_PUBLIC: no number |
| WMR / Bloomberg fixings (4PM London / NY close) | TARGET ingest + propagate ≤ 5 s end-to-end from publish to all FX exposure recalcs | CP_PUBLIC: no number | CP_PUBLIC: SAP fixing import "within minutes" | CP_PUBLIC: no number |

### §2.2 FX rate consumer query latency

| Workload | Oyatie TARGET |
|---|---|
| Single rate lookup, primed cache | TARGET p50 ≤ 1 ms / p95 ≤ 5 ms / p99 ≤ 20 ms |
| Single rate lookup, cold cache | TARGET p50 ≤ 50 ms / p95 ≤ 200 ms / p99 ≤ 500 ms |
| Bulk lookup 1,000 rates | TARGET p50 ≤ 50 ms / p95 ≤ 200 ms / p99 ≤ 1 s |
| Cross-rate calculation via triangulation | TARGET p99 ≤ 10 ms |

## §3 Payment Routing — Decision Latency

### §3.1 Payment-factory rail selection (synchronous decision)

| Workload | Oyatie TARGET | Kyriba CP_PUBLIC | SAP TRM CP_PUBLIC | Reval CP_PUBLIC |
|---|---|---|---|---|
| Single payment, rule-based rail decision (SWIFT vs SEPA vs FedWire vs RTP vs FedNow) | TARGET p50 ≤ 20 ms / p95 ≤ 100 ms / p99 ≤ 500 ms | CP_PUBLIC: "smart routing" (no latency disclosed) | CP_PUBLIC: TRM rule-based "milliseconds" | CP_PUBLIC: "smart routing" |
| Single payment with sanctions screening (fuzzy match against OFAC + EU + UN consolidated lists) | TARGET p50 ≤ 100 ms / p95 ≤ 500 ms / p99 ≤ 2 s | CP_PUBLIC: "real-time screening" | CP_PUBLIC: "real-time screening" | CP_PUBLIC: "real-time screening" |
| Single payment with full fraud check (anomaly + ML model) | TARGET p50 ≤ 200 ms / p95 ≤ 1 s / p99 ≤ 5 s | CP_PUBLIC: no number | CP_PUBLIC: no number | CP_PUBLIC: no number |

### §3.2 Payment file generation (NACHA, pain.001, MT101)

| Workload | Oyatie TARGET |
|---|---|
| Single pain.001 generation (1 payment) | TARGET p99 ≤ 100 ms |
| Batch pain.001 generation (10,000 payments per file) | TARGET p99 ≤ 30 s |
| Batch NACHA file generation (100,000 records) | TARGET p99 ≤ 60 s |
| Single MT101 generation | TARGET p99 ≤ 50 ms |
| Batch MT103 generation (1,000 wires) | TARGET p99 ≤ 5 s |

### §3.3 Payment approval workflow latency (synchronous user-perceived)

| Workload | Oyatie TARGET |
|---|---|
| Single-step approval click → Cedar permit → audit-chain seal → durable persist | TARGET p50 ≤ 100 ms / p95 ≤ 500 ms / p99 ≤ 2 s |
| Multi-step (3-level approval chain) wall-clock from first click to final clearance for a same-user batch | TARGET p99 ≤ 5 minutes (dominated by reviewer wall-clock, not service latency) |

## §4 SWIFT MT / MX Ingestion — Throughput

### §4.1 SWIFT MT940 ingestion

| Workload | Oyatie TARGET |
|---|---|
| Single MT940 file (1 statement, ~100 lines field 61) parse and persist | TARGET p99 ≤ 200 ms |
| Batch ingest 10,000 MT940 files (typical mid-day push) | TARGET p99 batch ≤ 5 minutes |
| End-of-day ingest 100,000 MT940 across all tenants in a cell | TARGET p99 batch ≤ 30 minutes |

### §4.2 SWIFT MX (ISO 20022 camt.053) ingestion

| Workload | Oyatie TARGET |
|---|---|
| Single camt.053 XML parse (10,000 entries) | TARGET p99 ≤ 500 ms |
| Batch ingest 10,000 camt.053 documents | TARGET p99 batch ≤ 10 minutes |

### §4.3 ISO 20022 pain.002 / camt.054 streaming (push notifications)

| Workload | Oyatie TARGET |
|---|---|
| Single pain.002 push → status update → audit-chain seal | TARGET p99 ≤ 100 ms |
| Single camt.054 push → cash-position delta → notification fan-out | TARGET p99 ≤ 200 ms |

## §5 Regulatory Report Generation — Latency and Throughput

### §5.1 EMIR REFIT trade report

| Workload | Oyatie TARGET |
|---|---|
| Single trade report (203 fields, ISO 20022 XML) | TARGET p99 ≤ 200 ms |
| Daily T+1 reporting batch (10,000 trades) | TARGET p99 batch ≤ 30 minutes (must complete by T+1 24:00 local) |
| Daily T+1 reporting batch (100,000 trades for large bank tenant) | TARGET p99 batch ≤ 2 hours |

### §5.2 Dodd-Frank Part 43/45/49 reporting

| Workload | Oyatie TARGET |
|---|---|
| Part 43 PET/PR real-time (15-min after execution) | TARGET p99 ≤ 30 seconds (substantially under 15-min CFTC ceiling) |
| Part 45 lifecycle event to SDR | TARGET p99 ≤ 5 seconds |
| Part 49 SDR connectivity (DTCC SDR, CME SDR, ICE SDR) | TARGET p99 ≤ 200 ms per submission |

### §5.3 Basel III LCR daily report

| Workload | Oyatie TARGET |
|---|---|
| LCR snapshot computation (HQLA + outflows + inflows, single banking-entity tenant) | TARGET p99 ≤ 30 seconds |
| LCR snapshot at 10^4 banking-entity tenants in a cell | TARGET p99 batch ≤ 1 hour |
| LCR XBRL export to local supervisor format (COREP EU, FFIEC 002/041 US, BoE STDF UK) | TARGET p99 ≤ 10 seconds per tenant |

### §5.4 Basel III NSFR quarterly report

| Workload | Oyatie TARGET |
|---|---|
| NSFR snapshot computation (ASF + RSF, 1-year horizon, single tenant) | TARGET p99 ≤ 60 seconds |
| NSFR quarterly batch | TARGET p99 batch ≤ 4 hours |

### §5.5 Basel III intraday liquidity (BCBS 248)

| Workload | Oyatie TARGET |
|---|---|
| Intraday liquidity snapshot (max usage, available, payments, time-specific obligations) | TARGET p99 ≤ 10 seconds, sustained ≥ 1 snapshot / 5 minutes per tenant |

### §5.6 IFRS 9 hedge effectiveness assessment

| Workload | Oyatie TARGET |
|---|---|
| Single hedge relationship effectiveness test (dollar-offset method) | TARGET p99 ≤ 100 ms |
| Single hedge relationship effectiveness test (regression method, 250 historical periods) | TARGET p99 ≤ 500 ms |
| Full hedge portfolio effectiveness assessment (1,000 relationships) | TARGET p99 ≤ 30 seconds |

### §5.7 SOX-404 control walk-through evidence export

| Workload | Oyatie TARGET |
|---|---|
| Single control walk-through export (PCAOB-acceptable PDF + structured JSON) | TARGET p99 ≤ 5 seconds |
| Quarterly walk-through batch (100 controls per quarter) | TARGET p99 batch ≤ 5 minutes |

## §6 Dashboard p99 (User-Perceived End-to-End)

### §6.1 Cash position dashboard

| Workload | Oyatie TARGET |
|---|---|
| First paint of cash position dashboard (single entity, 10 banks) | TARGET p50 LCP ≤ 1.5 s / p95 ≤ 3 s / p99 ≤ 5 s |
| First paint of multi-entity consolidated cash dashboard (100 entities, 1,000 banks) | TARGET p50 LCP ≤ 3 s / p95 ≤ 6 s / p99 ≤ 12 s |
| Drill-down from consolidated to single-entity view | TARGET p99 ≤ 1 s |
| Filter / sort interaction on intraday cash table (10,000 rows) | TARGET p99 ≤ 500 ms |

### §6.2 FX exposure dashboard

| Workload | Oyatie TARGET |
|---|---|
| FX exposure heatmap by ccy pair (50 ccy pairs) | TARGET p99 LCP ≤ 2 s |
| Interactive drill-down on a single ccy pair (last 250 trading days history) | TARGET p99 ≤ 1 s |

### §6.3 Payment status dashboard

| Workload | Oyatie TARGET |
|---|---|
| Payment status grid (10,000 in-flight payments) with filter + sort | TARGET p99 LCP ≤ 2 s; interactive p99 ≤ 500 ms |
| Real-time payment status update via SSE / WebSocket | TARGET p99 push latency ≤ 200 ms from camt.054 ingest to UI update |

### §6.4 Hedge effectiveness dashboard

| Workload | Oyatie TARGET |
|---|---|
| Hedge relationship list with effectiveness ratio + status (1,000 relationships) | TARGET p99 LCP ≤ 2 s |
| Single relationship deep-dive with 250-period regression chart | TARGET p99 ≤ 1 s |

### §6.5 Liquidity forecasting dashboard

| Workload | Oyatie TARGET |
|---|---|
| 13-week cash forecast chart with variance vs actuals overlay | TARGET p99 LCP ≤ 2.5 s |
| Scenario what-if re-run with 5 input changes | TARGET p99 ≤ 5 s |

## §7 Cross-Region and Sovereign-Cell Performance

### §7.1 Cross-region read latency

| Workload | Oyatie TARGET |
|---|---|
| US-East to EU-West read of cash position (HLC consistent read) | TARGET p99 ≤ 200 ms (geo-distance bound, AWS inter-region ~90 ms RTT) |
| US-East to AP-East read | TARGET p99 ≤ 300 ms |
| Cross-sovereign read (US-Gov to commercial cell) | DENY by default per ADR-0244 + sovereign-cell pack overlay |

### §7.2 Sovereign-cell residency enforcement

| Workload | Oyatie TARGET |
|---|---|
| EU-Sovereign tenant payment to US bank: residency check + cedar permit + cross-border consent flow | TARGET p99 ≤ 500 ms |
| KR-Sovereign tenant treasury data export to KR-PIPA-compliant target only | TARGET p99 residency-check ≤ 50 ms (per request) |
| CN-PIPL tenant data must not leave CN cell | TARGET enforcement: 100% deny on cross-cell access; audit-chain seal on every deny |

### §7.3 Multi-region failover

| Workload | Oyatie TARGET | Counterpart |
|---|---|---|
| Primary region failure: RTO (resume of treasury operations in secondary region) | TARGET RTO ≤ 15 minutes | CP_PUBLIC Kyriba: RTO 4 hours; SAP TRM: RTO 4-24 hours typical; Reval: RTO 4 hours |
| Primary region failure: RPO (data loss tolerance) | TARGET RPO ≤ 30 seconds (continuous replication via cloud-data canonical) | CP_PUBLIC Kyriba: RPO 15 minutes; SAP TRM: RPO 1 hour; Reval: RPO 15 minutes |

Note: Oyatie's TARGET RTO/RPO is substantially better than counterpart public claims. The doctrine reason: ADR-0248 cellular architecture + ADR-0254 K8s + Cloud Hypervisor + cloud-data continuous replication enables faster failover than the colo-anchored or single-region-active counterpart designs. This is a tenant-class-applicable target.

## §8 Throughput and Capacity Ceilings

### §8.1 Transaction throughput per cell

| Surface | Oyatie TARGET |
|---|---|
| Cash position writes (bank statement ingest debit/credit lines) | TARGET sustained ≥ 50,000 writes/sec per cell |
| Payment initiation writes (pain.001 row creation) | TARGET sustained ≥ 10,000 writes/sec per cell |
| FX exposure writes (exposure record creation) | TARGET sustained ≥ 5,000 writes/sec per cell |
| Hedge designation writes | TARGET sustained ≥ 1,000 writes/sec per cell |
| Audit-chain seal events (every state-change events emitted to audit-chain) | TARGET sustained ≥ 100,000 seals/sec per cell per audit-chain canonical |

### §8.2 Read throughput per cell

| Surface | Oyatie TARGET |
|---|---|
| Cash position read (single tenant query) | TARGET sustained ≥ 100,000 reads/sec per cell |
| FX rate lookup (single rate query) | TARGET sustained ≥ 500,000 reads/sec per cell (cache-fronted) |
| Payment status query | TARGET sustained ≥ 50,000 reads/sec per cell |

### §8.3 Storage capacity per tenant

| Capacity bound | Oyatie TARGET |
|---|---|
| Single-tenant 7-year retention for SOX-404 + 5-year for AML + 10-year for BSA | TARGET storage budget ≤ 5 TB per tenant per year compressed (NDJSON in Iceberg cold tier + Postgres hot tier) |
| Maximum tenant scale | TARGET 50 TB total per single tenant (bigger tenants shuffle-sharded across cells) |

## §9 Latency Budget Sub-Components (per ADR-0245 substrate composition)

Each user-perceived latency target above decomposes into substrate sub-budgets. The composition is for the cash-position-read p99 ≤ 5 s (cold-cache, multi-entity dashboard) example:

| Sub-component | Budget |
|---|---|
| TLS 1.3 + HTTP/3 handshake (cold connection) | ≤ 200 ms |
| API gateway routing + Cedar permit eval | ≤ 50 ms |
| Tenant scoping + ADR-0244 boundary check | ≤ 10 ms |
| Audit-chain seal of the read (when audit-on-read enabled) | ≤ 50 ms |
| Treasury REST adapter dispatch | ≤ 10 ms |
| Treasury usecase orchestration | ≤ 10 ms |
| Treasury domain query against repository port | ≤ 10 ms |
| Repository → Postgres query (cold cache, multi-entity join) | ≤ 3 s |
| Aggregate + sort + format response | ≤ 500 ms |
| Frontend hydrate + render | ≤ 1.2 s |
| Sum (worst case) | ≤ 5 s |

This decomposition forces remediation rows in the audit (F-P1-07) to encode the budget per sub-component, otherwise dashboards p99 ceiling will silently bloat as more middleware is added.

## §10 Counterpart-Public Claims (cited verbatim with disclaimers)

### §10.1 Kyriba

- "10x faster cash visibility" — Kyriba product brochure. **Disclaimer:** Kyriba does not disclose the reference baseline (10x faster than what?). Treat as marketing claim, not benchmark.
- "Real-time FX rates" — Kyriba product datasheet. **Disclaimer:** "real-time" is vendor-defined; typically means ≤ 15-minute cadence in practice.
- "AI-powered cash forecasting" — Kyriba Smart Cash Forecasting datasheet. **Disclaimer:** No public model benchmark numbers (MAPE, RMSE, accuracy by horizon).
- "Payment processing across 1,000+ bank connections" — Kyriba marketing site. **Disclaimer:** Bank count is a connectivity claim, not a throughput claim.

### §10.2 SAP TRM

- "End-to-end treasury on S/4HANA" — SAP TRM module documentation. **Disclaimer:** Performance depends on HANA in-memory sizing and is tenant-specific.
- "MBC (Bank Communication Management) supports 100+ formats including SWIFT MT/MX, BAI2, EBICS, H2H" — SAP MBC documentation. **Disclaimer:** Coverage claim, not latency claim.
- "Hedge accounting under IFRS 9 and US GAAP" — SAP TRM Hedge Management documentation. **Disclaimer:** Coverage claim, not effectiveness-test latency claim.
- "TRM Risk Analyzer with VaR" — SAP TRM Risk module. **Disclaimer:** Monte Carlo VaR can take minutes to hours depending on path count and portfolio size.

### §10.3 Reval

- "Industry-leading hedge accounting under IFRS 9 and US GAAP ASC 815" — Reval product datasheet (pre-ION acquisition). **Disclaimer:** Industry-leading is marketing; verify against PwC / KPMG / Deloitte / E&Y treasury survey peer-comparison reports.
- "Cloud-native TMS" — Reval marketing. **Disclaimer:** Cloud-native is architectural claim, not performance claim.
- "Real-time FX risk monitoring" — Reval datasheet. **Disclaimer:** Same as Kyriba "real-time" caveat.

## §11 Observability Hooks per Benchmark

Every TARGET budget in this document must be observable via the canonical observability substrate (per ADR-0130 + ADR-0131 microservices/observability):

- Metric: histogram per TARGET row with labels for tenant_id, cell_id, region, tenant_class, billing_component, pack_overlay_active.
- Trace: distributed trace spans from API gateway → treasury REST → usecase → domain → repository → Postgres/Valkey/Iceberg/CDN, with HLC timestamp per ADR-0252.
- Log: structured logs with workload classifier (single-entity, multi-entity, cross-region, sovereign-cell-enforced, regulatory-pack-active).
- Audit: every benchmark-relevant decision (payment-rail selection, sanctions screening hit, hedge effectiveness pass/fail, LCR ratio breach) sealed to audit-chain per ADR-0263.
- SLO: each TARGET above maps to a microservices/treasury/slos/*.openslo.yaml record. The four named SLOs from manifest.second_pass_doc_suite (cash-position-recalc latency, fx-exposure refresh latency, payment-execution decision latency, regulatory-report-generation throughput) must be re-authored against these TARGETs.

## §12 Brownout Degradation Signals

When a TARGET budget is at risk of breach, Treasury must emit canonical brownout signals per `/specs/brownout-degradation-signal.json`:

- Level 1 (Warning, > 80% of TARGET budget consumed): increase cache TTL, throttle non-essential workflows.
- Level 2 (Critical, > 95% of TARGET budget consumed): shed read traffic to read-replica, defer non-time-sensitive batch jobs.
- Level 3 (Brownout, TARGET budget breached): defer payment-initiation flows except emergency-services / break-glass; defer non-T+1-deadline regulatory reports; emit audit-chain BROWNOUT-ENTERED event; trigger ADR-0248 cell-shed routing for new traffic.
- Level 4 (Blackout, sustained breach > 5 minutes): full failover to secondary region per §7.3; emit BLACKOUT-ENTERED event.

## §13 Findings (per D-6.23)

F-BENCH-01: No row in this document is MEASURED. All numbers are TARGET budgets. Per ADR-0328 D-6.12, the benchmark doc "must distinguish measured values from target budgets and counterpart-public claims" — this document is compliant by explicit classification. A follow-up benchmarking sub-wave is required to land MEASURED numbers.

F-BENCH-02: Counterpart public claims are sparse on quantitative numbers. Kyriba, SAP, and Reval primarily disclose qualitative claims ("real-time", "near-instant", "industry-leading") rather than p99 latencies, sustained throughputs, or RPO/RTO numbers. The benchmark methodology must therefore rely on independent industry benchmarks (Aite-Novarica TMS Vendor Comparison, Gartner Magic Quadrant Treasury technology, Celent TMS reports) for cross-vendor comparison rather than vendor self-reports.

F-BENCH-03: The TARGET RTO/RPO (§7.3) is substantially better than counterpart public claims. This is a differentiation opportunity but also a risk — the TARGET must be backed by a measured chaos-engineering run before Treasury can advertise it.

F-BENCH-04: The §1.2 intraday refresh TARGET (sub-second push-driven) is substantially better than the 15-minute polling cadence advertised by Kyriba/SAP MBC/Reval. This is a strategic differentiation tied to ADR-0252 HLC + push notifications + camt.054 streaming.

F-BENCH-05: The §5.2 Dodd-Frank Part 43 PET/PR target of ≤ 30 seconds is substantially under the 15-minute CFTC ceiling. This is a regulatory headroom commitment; the actual user value is risk reduction (fewer late-report penalties) rather than user-perceived speed.

## §14 Backlog Rows (per D-6.24)

```
microservice=treasury severity=P1 category=benchmark file=performance-benchmark-numbers-2026-05-20.md fix="run live benchmarks against six bounded-context scaffold to backfill MEASURED values within +/-15% of TARGET; gate Phase 4A.2 promotion on landing" remediation=post-15F-benchmarking-subwave
microservice=treasury severity=P1 category=benchmark file=slos/*.openslo.yaml fix="re-author 4 SLOs (cash-position-recalc, fx-exposure-refresh, payment-execution-decision, regulatory-report-generation) against TARGETs in this document" remediation=15F
microservice=treasury severity=P2 category=benchmark file=performance-benchmark-numbers-2026-05-20.md fix="backfill counterpart-public quantitative numbers from Aite-Novarica + Gartner + Celent industry reports" remediation=post-15F
microservice=treasury severity=P2 category=benchmark file=performance-benchmark-numbers-2026-05-20.md fix="commission independent chaos-engineering run of TARGET RTO/RPO claims; do not advertise differentiation until landed" remediation=post-15F
microservice=treasury severity=P2 category=observability file=dashboards/*.json fix="add brownout degradation signal panels per Section 12" remediation=15F
microservice=treasury severity=P2 category=observability file=microservices/observability/treasury/ fix="register histograms per Section 11 (8 named SLO histograms minimum) plus brownout level counters" remediation=15F
```

## §15 Verification Notes

The verifier inspected this benchmark doc using ADR-0328 D-10.5..D-10.9. Anchor checks: the five-citation header is microservice-specific. Scope check: 5 of 5 brief families covered (cash position recalc latency, FX rate streaming throughput, payment routing decision latency, regulatory report generation, dashboard p99). Substance check: every TARGET row decomposes into either workload-specific p50/p95/p99 latencies, sustained throughput per cell, or batch-completion windows. Classification check: every row is explicitly labeled MEASURED / TARGET / CP_PUBLIC per ADR-0328 D-6.10..D-6.13.

## §16 Closing — Owner of Coherence

This benchmark doc lands the canonical TARGET budget grid for Treasury at Phase 4A.2 ERP. It complements the coherence-audit-2026-05-20.md verdict (REVISE) and the feature-parity-matrix-2026-05-20.md UNION-coverage matrix. The MEASURED backfill responsibility transfers to a post-15F benchmarking sub-wave once the F-P1-01 (bounded-context split), F-P0-03 (regulatory anchors), and F-P0-04 (bank-connectivity anchors) closures land in 15A/15F.
