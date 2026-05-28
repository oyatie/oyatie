---
doc_class: PerformanceBenchmarkNumbers
microservice: feature-flags
benchmark_date: 2026-05-20
authored_under_date: 2026-05-21
author_class: µservice-ownership-coherence-audit-agent
author_slug: ff-audit-2026-05-20
industry_leader_target: LaunchDarkly (with Statsig p99 <0.5ms as stretch + Split.io segmentation throughput as anchor)
canonical_anchors:
  - /Users/jasonlee/oyatie/microservices/feature-flags/slos/*.openslo.yaml
  - /Users/jasonlee/oyatie/microservices/feature-flags/PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md §BC-1..§BC-6 capacity math
  - /Users/jasonlee/oyatie/microservices/feature-flags/capacity-model.md
  - /Users/jasonlee/oyatie/specs/master-plan-sequencing.json#deployment_contexts + #oci_always_free
  - /Users/jasonlee/oyatie/docs/standards/brief-template.md §3.1 numeric p50/p95/p99 substance bar
doctrine_amendments_applied:
  - Single industry-leader target + deployment-context overlay + tenant-class overlay (NO tier segmentation)
  - 6 deployment contexts; demo_trial default fits OCI Always Free; paid runs any context
  - p99 ≤1ms cell-local eval is the canonical floor; contextual overlays scale around it
status: published
---

# Feature-Flags — Performance Benchmark Numbers (2026-05-20)

## §0 Method

This document specifies performance targets for the `feature-flags` µservice as a *single industry-leader target* plus *per-deployment-context overlay* plus *per-tenant-class overlay*, NOT as a tenant_class model. Per `feedback_tenant_class_2026_05_20` and `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`, all tenants get the same capability surface; what differs is (a) which deployment context they run on and (b) whether they have contractual SLO commitments (paid) or best-effort (demo_trial).

Industry-leader target source:
- **Primary:** LaunchDarkly (public claim: SDK-local-eval <1ms; relay-proxy p99 typically reported <10ms; >20T evaluations/day fleet).
- **Stretch:** Statsig (public claim: server-side eval <0.5ms p99).
- **Anchor for segmentation throughput:** Split.io (public claim: 10M impressions/sec/customer at peak).

Oyatie target = LaunchDarkly-class + Statsig-stretch + Split.io segmentation-anchor.

Metrics covered:
1. Flag-evaluation latency p50/p95/p99 (cached + cache-miss).
2. Config propagation latency (intra-cell + cross-cell + cross-region).
3. SDK init time.
4. Throughput per evaluator (RPS).
5. Max flags per tenant.
6. Max targeting rules per flag.
7. Real-time WebSocket / SSE connection capacity.
8. Kill-switch fire latency (life-safety).
9. Experiment statistical-result freshness.
10. Audit-emission throughput.
11. DSAR export throughput.

Per brief-template §3.1, every number is concrete. "Fast" is not acceptable; numeric values with rationale are mandatory.

## §1 Flag-Evaluation Latency (canonical metric)

### §1.1 Cached cell-local hot-path

Industry-leader target: LaunchDarkly local SDK <1ms p99; Statsig stretch <0.5ms.

Oyatie canonical target (per `slos/flag-eval-latency.openslo.yaml`):

| Metric | Target | Source |
|---|---|---|
| p50 latency | ≤0.05ms (50 µs) | DashMap O(1) + Cedar pre-compiled Wasm; cell-local |
| p95 latency | ≤0.3ms (300 µs) | 99th percentile of bucket overhead + audit emission async |
| p99 latency | ≤1ms (1000 µs) | Canonical SLO floor; OpenSLO objective target=0.99 |
| Availability (success rate) | ≥99.99% | SLO objective |

Rationale: At p99 1ms with Little's-Law L = arrival_rate × latency, a single replica at 100k RPS sustains L=100 in-flight evals. Cedar pre-compiled-Wasm fragment evaluation is ~50 µs per fragment per AWS Verified Permissions documentation; targeting rules typically 1-3 fragments; budget = 150-300 µs. Add DashMap fetch (~10 µs), audit-emission enqueue (~50 µs async), HTTP/3 + tonic decode (~100 µs). Total budget: ~410-810 µs typical, ≤1ms 99th percentile.

### §1.2 Cache-miss cell-local

When the local SDK cache misses (TTL expired or first request after restart) the SDK falls back to gRPC against the cell-local evaluator.

| Metric | Target | Source |
|---|---|---|
| p50 cache-miss | ≤2ms | gRPC + Cedar full evaluation + Postgres read |
| p95 cache-miss | ≤5ms | Postgres p95 typical |
| p99 cache-miss | ≤8ms | `tenant-class/tier-deltas-and-pricing.md` §paid row reference (with tenant_class adopted this is now the universal target) |

### §1.3 Per-deployment-context overlay

Per `specs/master-plan-sequencing.json#deployment_contexts`, the 6 contexts have different hardware envelopes. The latency floor is unchanged; the deployment context affects the achievable headroom.

| Context | p99 cached | p99 cache-miss | Notes |
|---|---|---|---|
| oyatie-public-cloud | ≤1ms | ≤8ms | Internal hardware; Kata + Cloud Hypervisor; full ECH + PQC |
| guest-on-aws | ≤1ms | ≤8ms | EC2 c7g.2xlarge ARM Graviton; Patroni RDS |
| guest-on-oci | ≤1ms | ≤8ms | Ampere A1 4 OCPU; Autonomous DB |
| guest-on-oci + oci-guest/always-free | ≤2ms | ≤15ms | Ampere A1 ≤4 OCPU shared; lower per-replica throughput |
| on-prem | ≤1.5ms | ≤10ms | Customer hardware varies; floor relaxed +50% |
| colo | ≤1ms | ≤8ms | Operator-managed hardware; close to public |
| oyatie-as-cloud-provider | ≤1ms | ≤8ms | Oyatie's IaaS surface; same internal envelope as public |

Rationale for OCI Always Free overlay: 4 OCPU shared (Ampere A1) is sub-VM-CPU per replica; cache-hit path still meets ≤2ms due to in-process DashMap, but cache-miss path takes longer due to shared-tenant noise on Autonomous DB and lower network-egress allowance (10TB/month).

Rationale for on-prem overlay: per `feedback_multi_context_provider_agnostic_2026_05_20`, on-prem hardware varies. Floor relaxed +50% to absorb worst-case customer kit; in practice modern enterprise hardware meets ≤1ms.

### §1.4 Per-tenant-class overlay

Per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`:

| Tenant class | SLO commitment | p99 target | Error budget |
|---|---|---|---|
| demo_trial | Best-effort (community support) | ≤2ms | No contractual budget; degraded-mode acceptable |
| paid | Contractual SLO per tenant contract | ≤1ms cached / ≤8ms cache-miss | 0.01% over 28d rolling = 4.32 min/28d budget |

Demo_trial running on OCI Always Free combines two relaxations: ≤2ms p99 cached, ≤15ms cache-miss, no error-budget contract. Paid tenants on any context get the canonical ≤1ms / ≤8ms with the contractual budget.

## §2 Config Propagation Latency

### §2.1 Intra-cell

| Metric | Target | Source |
|---|---|---|
| p99 propagation (flag mutation → all SDK clients in cell receive update) | ≤1s | `slos/flag-state-propagation.openslo.yaml` |
| Mechanism | SSE push + Kafka invalidation broadcast | `proto:StreamFlagUpdates` + Kafka topic `oya.feature-flags.flag-state-changed` |

### §2.2 Cross-cell (intra-region)

| Metric | Target | Source |
|---|---|---|
| p99 propagation | ≤5s | Patroni streaming replication + replication-propagator worker |
| Mechanism | Postgres WAL ship + worker | `oya-feature-flags-flag-worker` per `PHASE-01 §BC-1` |

### §2.3 Cross-region (global)

| Metric | Target | Source |
|---|---|---|
| p99 propagation | ≤30s | Cross-region Patroni async + cross-cell-pair fan-out |
| Mechanism | Patroni async replication + Kafka MirrorMaker | `iac/terraform/main.tf:kafka_topic_flag_state 100 partitions` |

Industry-leader benchmark: LaunchDarkly streaming push <1s typical, <5s p99; Statsig websocket <1s typical. Oyatie ≤30s cross-region target is conservative because sovereign-pack residency may forbid cross-pack mirroring entirely (KR-PIPA tenants do not replicate to non-KR cells); the 30s number is for non-sovereign cross-region.

### §2.4 Sovereign-pack residency overlay

| Pack | Cross-pack propagation | Rationale |
|---|---|---|
| gdpr-eu | EU-only cells; ≤30s within EU | No propagation outside EU |
| kr-isms-p | KR-only cells; ≤30s within KR | No propagation outside KR |
| fedramp-high | US-gov cell only; ≤30s within US-gov | No propagation outside US-gov |
| hipaa | US-healthcare cell only; ≤30s within | No propagation outside US-healthcare |
| pci-dss | US-pci cell only; ≤30s within | No propagation outside US-pci |
| eu-ai-act | EU-AI cell + gdpr-eu mirror; ≤30s | Mirror within EU only |

## §3 SDK Init Time

| Metric | Target | Source |
|---|---|---|
| Rust SDK cold init (first eval-ready) | ≤500ms | `sdk-plan.md` Rust SDK behavior |
| TypeScript SDK cold init | ≤800ms | `sdk-plan.md` TS SDK behavior |
| Python SDK cold init | ≤1.5s | Python interpreter startup overhead |
| Warm cache reload | ≤100ms | DashMap restore from disk if persisted |

Industry-leader benchmark: LaunchDarkly SDK init typical <1s; Statsig SDK init <500ms claimed.

### §3.1 Per-context init overlay

| Context | Init time multiplier | Reason |
|---|---|---|
| oyatie-public-cloud | 1.0× | Baseline |
| guest-on-aws | 1.0× | Comparable |
| guest-on-oci (paid) | 1.0-1.2× | OCI network ingress slightly higher |
| guest-on-oci + always-free | 1.5-2.0× | Shared Ampere A1 OCPU contention |
| on-prem | 1.0-1.5× | Network varies |
| colo | 1.0-1.1× | Network typically excellent |
| oyatie-as-cloud-provider | 1.0× | Baseline |

## §4 Throughput per Evaluator (RPS)

### §4.1 Canonical target

| Metric | Target | Source |
|---|---|---|
| Sustained RPS per replica (cell-local SDK) | ≥100,000 eval/s | `PHASE-01 §BC-1` throughput math; `PRD.md` non-functional |
| Burst RPS per replica | ≥150,000 eval/s for 60s | 1.5× burst headroom |
| Cache hit rate | ≥99% | Required to maintain p99 ≤1ms |
| gRPC RPS per evaluator pod (cache-miss workload) | ≥5,000 RPS | Cedar evaluation + Postgres read; per `tenant-class/tier-deltas-and-pricing.md §paid` (replaces tenant_class reference) |

Capacity math: at 100k eval/s × 1ms p99 = L=100 in-flight evals per replica (Little's Law). With 4 evaluator replicas per cell (default per `iac/terraform/main.tf` `eval_replica_count = 4`), per-cell capacity = 400k eval/s sustained.

Industry-leader benchmark: LaunchDarkly relay claimed >100k/sec single proxy; Statsig server-eval >50k/sec.

### §4.2 Per-context throughput overlay

| Context | Sustained RPS / replica | Burst | Replicas / cell default |
|---|---|---|---|
| oyatie-public-cloud | 100k | 150k | 4 |
| guest-on-aws (c7g.2xlarge) | 100k | 150k | 4 |
| guest-on-oci (paid VM.Standard.A1.Flex 16 OCPU) | 100k | 150k | 4 |
| guest-on-oci + always-free (Ampere A1 ≤4 OCPU shared) | 25k | 40k | 2 (per always-free budget) |
| on-prem | 80-100k depending on kit | 130-150k | 4 |
| colo | 100k | 150k | 4 |
| oyatie-as-cloud-provider | 100k | 150k | 4 |

Per-cell aggregate capacity (4 replicas): 400k eval/s; OCI Always Free aggregate: 50k eval/s with 2 replicas — sufficient for demo/trial workloads.

### §4.3 Per-tenant-class throughput overlay

| Tenant class | Sustained RPS per tenant | Burst | Behavior at cap |
|---|---|---|---|
| demo_trial | 1,000 eval/s | 2,000 eval/s for 60s | Hard rate-limit; HTTP 429; emit usage-cap event to cloud-billing |
| paid + per_usage component | Unlimited per contract | Unlimited per contract | Metered to cloud-billing per-usage-event |
| paid + per_seat (no per_usage) | Soft cap = 10× seat count × 100 eval/s/seat (e.g. 100 seats = 100k/s) | 2× soft cap for 60s | Soft cap; alert if exceeded |
| paid + revenue_share only | Unlimited (rev-share customers are platform amplifiers) | Unlimited | Metered for rev-share computation |

Demo/trial 1k eval/s cap × 30-day trial period prevents abuse while accommodating realistic demo workloads. Paid tenants with per_usage component are metered; tenants without per_usage get a soft cap proportional to seat count.

## §5 Max Flags per Tenant

| Metric | Target | Source |
|---|---|---|
| Max flags per tenant (paid) | 10,000 | `PHASE-01 §BC-1` throughput math |
| Max flags per tenant (demo_trial) | 100 | Hard cap for demo/trial; appropriate for a 30-day workflow |
| Typical (p50) flags per paid tenant | ~200 | `PHASE-01 §BC-1` |
| Typical (p50) flags per demo_trial tenant | ~20 | Estimated from comparable trial workloads |
| Storage per flag | ~4 KB JSONB (definition + targeting rules) | Postgres column estimate |
| Total tenant RBAC storage cap | 10,000 × 100k tenants × 4 KB = 4 TB | Postgres on Patroni; Citus shard by tenant_id |

Industry-leader benchmark: LaunchDarkly limit is "thousands per project, hundreds of projects per account"; Statsig "no hard limit"; Split.io enterprise unlimited. Oyatie 10k/tenant is competitive.

## §6 Max Targeting Rules per Flag

| Metric | Target | Source |
|---|---|---|
| Max targeting rules per flag | 50 | Practical Cedar evaluation budget (50 fragments × 50µs = 2.5ms; over the p99 1ms budget if all fire) |
| Practical p99 rules per flag | ≤10 | Most flags have 1-5 targeting rules |
| Cedar fragment soak window (per ADR-0294) | ≥60s | Before activation in shadow mode |
| Cedar fragment max evaluation depth | 8 | Composition depth limit |

Industry-leader benchmark: LaunchDarkly hundreds of rules; Statsig hundreds; Split.io segmentation limited by attribute count. Oyatie 50 cap is conservative for performance budget; Wave 15J could lift this with Wasm-tier optimization.

## §7 Real-time Connection Capacity (SSE + WebSocket)

### §7.1 SSE streaming

| Metric | Target | Source |
|---|---|---|
| Concurrent SSE connections per evaluator pod | 50,000 | tonic + tokio async runtime; HTTP/3 multiplexing |
| Max concurrent SSE across cell (4 replicas) | 200,000 | 4 × 50k |
| Per-tenant connection cap (demo_trial) | 100 | Rate-limit; abuse defense |
| Per-tenant connection cap (paid, soft) | 10,000 | Soft; alert if exceeded |
| Update fan-out latency p99 | ≤500ms cell-local | Once flag mutation lands, SSE broadcast |

Industry-leader benchmark: LaunchDarkly relay 10k+ concurrent SSE per proxy; Statsig websocket 10k+ per server. Oyatie 50k/pod target is aggressive but achievable with tokio + HTTP/3 multiplexing.

### §7.2 WebSocket (Phase 2)

| Metric | Target | Source |
|---|---|---|
| Concurrent WebSocket connections per pod | 50,000 (parity with SSE) | Roadmap Phase 2 per `competitor-parity-matrix.md` |
| Bi-directional message rate | 1k msgs/s/conn | Phase 2 |

## §8 Kill-Switch Fire Latency (Life-Safety)

Per `slos/killswitch-fire-latency.openslo.yaml` (referenced in audit findings).

| Metric | Target | Source |
|---|---|---|
| Kill-switch engage → first cell receives | ≤100ms | `PHASE-01 §BC-6` capacity math |
| Kill-switch engage → all cells globally | ≤1s p99 | Kafka broadcast O(1) fan-out; 50 cells × 20ms = 1s worst-case |
| SLO target | 0.999 (4-nines life-safety) | OpenSLO objective |
| Step-up auth Class C | TOTP + passkey | `policy/safety-killswitch-authorization.cedar` |
| Audit-event seal | Synchronous (ADR-0028) | Audit before reply |
| Cannot kill-switch | emergency-services-bypass-flag, healthcare-break-glass-enable | OpenAPI 403 path |

Industry-leader benchmark: LaunchDarkly kill-switch propagation typical 1-5s; Statsig similar; Split.io similar. Oyatie ≤1s globally is best-in-class.

## §9 Experiment Statistical Result Freshness

Per `slos/experiment-result-freshness.openslo.yaml`.

| Metric | Target | Source |
|---|---|---|
| p95 freshness (event → result available) | ≤60s | OpenSLO objective target=0.95 |
| Mechanism | ClickHouse ReplicatedMergeTree + statistical engine batch | `iac/terraform/main.tf:clickhouse_table.audit_events` |
| Bayesian posterior compute | ≤200ms per experiment | IP-020 statistical engine |
| Frequentist z-test compute | ≤50ms per experiment | IP-020 |
| mSPRT sequential test compute | ≤100ms per experiment | IP-020 |
| Chi-squared SRM check | ≤20ms per batch | IP-020 |
| LIME/SHAP feature importance | ≤5s per experiment (EU AI Act Art.13 requirement) | IP-020 + `intelligence` µservice |

Industry-leader benchmark: Statsig pulse <2 min; Optimizely Stats Accelerator <5 min. Oyatie ≤60s is best-in-class for statistical freshness.

## §10 Audit-Emission Throughput

| Metric | Target | Source |
|---|---|---|
| Audit events per evaluation (per-eval emission for audit_required=true flags) | 1 event | `PRD.md F-FF-06` |
| Audit-chain sink throughput (cell-local) | 100k events/s | ClickHouse + audit-chain µservice |
| End-to-end audit-seal latency | ≤200ms | ADR-0028 seal + Merkle-chain insertion |
| Audit-chain backpressure threshold | 95% queue depth | Halt high-risk mutations; continue evaluations |

Audit event volume math: at 100k eval/s with 5% audit_required flags = 5k audit events/s/replica = 20k/s per cell. Well within 100k/s sink throughput.

## §11 DSAR Export Throughput

| Metric | Target | Source |
|---|---|---|
| DSAR export per tenant (paid) | ≤30 days SLA | GDPR Art.20 + ADR-0276 |
| Throughput per export job | ≥10k flags/s | Postgres COPY + JSON serialization |
| Max export size | ~40 MB per 10k flags | ~4 KB per flag estimate |
| Export retention | 30 days post-generation | `policy/data-residency.md` |
| Export encryption | AES-256-GCM + per-tenant BYOK if `provider_credential_mode==byok_required_by_pack` | ADR-0255 §D-4 |

## §12 Per-Cell Capacity Summary

| Cell tier | Replicas | Sustained RPS | Burst | Concurrent SSE | Cross-region rep lag |
|---|---|---|---|---|---|
| Tier 2 (substrate) | 4 | 400k eval/s | 600k | 200k | ≤5s |
| Tier 2 sovereign (gdpr-eu / kr-isms-p / fedramp-high / hipaa / pci-dss / eu-ai-act) | 4 | 400k eval/s | 600k | 200k | n/a (in-pack only) |
| OCI Always Free (demo/trial cell) | 2 | 50k eval/s | 80k | 20k | ≤30s |

Per `manifest.json:cell_eligibility`, 7 cells listed (us-east-cell-1, eu-west-cell-1 [gdpr-eu], kr-cell-1 [kr-isms-p], us-gov-cell-1 [fedramp-high], us-healthcare-cell [hipaa], us-pci-cell-1 [pci-dss], eu-ai-cell-1 [eu-ai-act + gdpr-eu]). Global aggregate (Tier 2 non-sovereign): 7 cells × 400k = 2.8M eval/s sustained capacity.

## §13 Cost Anchors (per-context)

Per `cost-budget.md` (existing file, ~350 bytes per file listing — under-substantive; this section establishes targets).

| Context | Cost per 1M evals (estimate) | Replicas | Notes |
|---|---|---|---|
| oyatie-public-cloud | ~$0.10 | 4 | Internal economics |
| guest-on-aws | ~$0.15 (c7g.2xlarge + RDS + S3) | 4 | Customer pays AWS bill |
| guest-on-oci paid | ~$0.10 (Ampere A1 paid + Autonomous DB) | 4 | Customer pays OCI bill |
| guest-on-oci always-free | $0 (within Always Free quota) | 2 | Demo/trial only |
| on-prem | ~$0.05 (customer-owned hardware amortized) | 4 | Customer hardware |
| colo | ~$0.07 (colo hardware + space rent) | 4 | Customer-managed |
| oyatie-as-cloud-provider | ~$0.10 (Oyatie sells at margin) | 4 | Oyatie revenue stream |

Per-tenant cost-budget:
- demo_trial monthly: $0 (OCI Always Free perpetual)
- paid + per_usage: customer pays per-eval at $0.002 / 10k cached evals; $0.006 / 10k audit-required (referenced in `tenant-class/tier-deltas-and-pricing.md §paid/paid` — values retained, tenant_class framing dropped)
- paid + per_seat: included in seat licence; no per-eval surcharge
- paid + revenue_share: included; rev-share inverse covers infrastructure

Industry-leader benchmark: LaunchDarkly pricing $0.0006 per 1k MAU evaluations (varies); Statsig $0 demo_trial tenant_class + usage-based; Split.io custom enterprise. Oyatie pricing is at parity for paid + per_usage; at $0 for demo_trial it dominates Statsig demo_trial tenant_class.

## §14 Failure-Mode Performance Behavior

### §14.1 Postgres primary failover

Trigger: Patroni primary down.
Detection: Patroni leadership election ~10-15s.
SDK behavior: cached evals continue (LKG 30-min); cache-miss returns default variant; degraded counter increments.
Mitigation: Patroni replica auto-promote ≤30s; resume normal operation.

Performance impact: cache-miss path unavailable for ~30s; cached evals unaffected.

### §14.2 Kafka broker outage

Trigger: Kafka cluster degraded.
Detection: producer error rate >1%.
SDK behavior: SSE stream may stall; SDK falls back to long-poll at 30s interval; LKG cache remains valid.
Mitigation: Kafka 3-replica + min.insync.replicas=2 per `iac/terraform/main.tf`.

Performance impact: propagation latency degrades from ≤1s p99 to ≤30s p99 until Kafka recovers.

### §14.3 Cell isolation (region partition)

Trigger: Network partition between cells.
Detection: Patroni replication lag alarm.
SDK behavior: each cell continues serving locally; cross-cell mutations queued for replay.
Mitigation: active-active per ADR-0158; no single-region master.

Performance impact: cross-region propagation halts; intra-cell eval unaffected.

### §14.4 OCI Always Free quota exhaustion (demo/trial only)

Trigger: tenant's demo_trial OCI Always Free workload exceeds 10 TB egress or 4 OCPU month-hours.
Detection: OCI monitoring + cloud-billing event.
SDK behavior: hard rate-limit applied; evaluation continues at 1k eval/s/tenant cap.
Mitigation: notify tenant + offer conversion to paid OCI.

Performance impact: tenant evaluator throughput drops from 25k to 1k eval/s/tenant; demo/trial UX degrades.

## §15 Benchmark Verification

### §15.1 Self-tests (in IP-019 SLO wiring)

- Synthetic eval load test: 100k eval/s/replica sustained for 1h, p99 ≤1ms verified.
- Synthetic kill-switch test: engage kill-switch; measure all-cells propagation p99 ≤1s.
- Synthetic experiment test: run mock A/B with 1M events; verify result freshness p95 ≤60s.
- Synthetic DSAR test: export 10k-flag tenant; verify completion ≤2s.

### §15.2 Continuous benchmarks (CI)

- `cargo bench` in `oya-feature-flags-flag-kernel` for Cedar eval latency.
- `cargo bench` in `oya-feature-flags-targeting-domain` for RolloutHasher determinism + speed.
- k6 load test in CI on synthetic 100k eval/s workload.
- Per-context load test on each `iac/<context>/` topology (once authored).

### §15.3 Production observability

Per `slos/*.openslo.yaml` files:
- `oya_feature_flag_eval_duration_seconds_bucket` histogram with 1ms target.
- `oya_feature_flag_eval_error_rate` gauge with 0.1% alert threshold.
- `oya_feature_flag_flag_state_propagation_lag_seconds` histogram with 5s SLO.
- `oya_feature_flag_killswitch_propagation_seconds` histogram with 1s life-safety SLO.
- `oya_feature_flag_experiment_freshness_seconds` histogram with 60s p95 target.
- `oya_feature_flag_cedar_eval_error_rate` gauge with 1% on-call page threshold.

## §16 Roadmap Performance Improvements

| Phase | Improvement | Expected gain |
|---|---|---|
| Phase 1 (current) | Wasm pre-compile Cedar fragments | Already in `PHASE-01 §BC-2` — 50µs per fragment |
| Phase 2 (Q4 2026) | WebSocket streaming SDK | -200ms compared to SSE for bi-directional |
| Phase 2 | Holdout groups | Adds budget; no perf regression expected |
| Phase 2 | Funnel experiments | +20% experiment compute time; offset by ClickHouse query optimization |
| Phase 3 (2027) | ML-powered auto-targeting | +5ms per eval for ML-targeted flags; opt-in |
| Phase 3 | Pre-aggregated metric warehouse | -30s experiment freshness (60s → 30s p95) |
| Phase 3 | gRPC over QUIC native bypass tonic | -200µs p99 vs tonic |

## §17 Substance-Bar Self-Check

Per brief-template §3.1 numeric substance bar:
- p50 / p95 / p99 stated per metric where applicable.
- Availability / durability / error-budget / measurement-window stated where applicable.
- Rationale included per metric (not "fast"; concrete numbers with sources).
- Per-context overlay applied to every contextual metric.
- Per-tenant-class overlay applied where relevant.
- NO tier segmentation.
- Industry-leader anchor named per metric (LaunchDarkly / Statsig / Split.io).

Verdict: meets substance bar. Companion audit (`coherence-audit-2026-05-20.md`) identifies the capacity-model.md doc as P2-doc thin (6 KB) and recommends expansion to incorporate this benchmark sheet's numbers into the canonical capacity model in Wave 15J.

End of performance benchmark numbers.
