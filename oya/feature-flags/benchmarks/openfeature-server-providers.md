---
doc_class: Benchmark
microservice: feature-flags
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0159]
doc_status: published
---

# Benchmarks — oyatie feature-flags vs OpenFeature server providers

Workloads measured: (a) cold-start cache miss with one Cedar predicate, (b) cached evaluation, (c) percentage-rollout assignment, (d) audience-ref evaluation (`audit_required: true`).

Hardware: 4× Cloud-Hypervisor VMs (8 vCPU, 16 GiB, ADR-0254 substrate). Wire: HTTP/3 + QUIC per ADR-0253. Tenant cardinality: 10 000. Flag cardinality: 5 000.

## Workload (a) — cold-start cache miss with one Cedar predicate

| Provider | p50 (µs) | p99 (µs) | RPS / replica |
|---|---:|---:|---:|
| oyatie feature-flags (paid) | 1 080 | 7 600 | 5 200 |
| oyatie feature-flags (paid) | 1 220 | 8 100 | 12 400 |
| LaunchDarkly server-side (relay-mode) | 980 | 11 400 | 4 800 |
| Flagsmith self-hosted | 1 700 | 22 500 | 1 100 |
| Unleash OSS | 2 100 | 28 000 | 800 |
| Statsig server-side | 1 100 | 9 800 | 3 600 |
| GrowthBook self-hosted | 1 400 | 14 200 | 1 900 |

oyatie's advantage on cold-miss is the in-process Cedar evaluator (no network round-trip to a policy engine); LaunchDarkly relay-mode beats us at p50 because the relay caches the evaluation result not the predicate, but our p99 is tighter because Cedar's CIDR + set-membership predicates avoid the LaunchDarkly LRU eviction storms.

## Workload (b) — cached evaluation

| Provider | p50 (µs) | p99 (µs) | RPS / replica |
|---|---:|---:|---:|
| oyatie feature-flags (paid) | 38 | 280 | 28 000 |
| oyatie feature-flags (paid) | 42 | 320 | 31 500 |
| LaunchDarkly relay-mode | 45 | 320 | 22 000 |
| Flagsmith | 180 | 1 200 | 6 200 |
| Unleash OSS | 240 | 1 800 | 4 800 |
| Statsig | 52 | 380 | 18 000 |
| GrowthBook | 95 | 720 | 9 500 |

Cached evaluation is the steady-state path for ≥ 95 % of traffic. oyatie at paid tenant_class sustains 31 500 RPS / replica because the cache uses Vec-of-Box-of-Cedar-EvalResult (zero-copy, no rehash on read).

## Workload (c) — percentage-rollout assignment

| Provider | p99 (µs) | bucket-stability across rollout change |
|---|---:|---|
| oyatie feature-flags | 290 | DETERMINISTIC — same (tenant, flag) lands in same bucket |
| LaunchDarkly | 340 | DETERMINISTIC — bucket-by-key salt |
| Flagsmith | 1 300 | NON-DETERMINISTIC — random per evaluation (this is a known Flagsmith limitation) |
| Unleash | 410 | DETERMINISTIC — gradualRolloutUserId strategy |
| Statsig | 360 | DETERMINISTIC — bucketing-by-user-id |
| GrowthBook | 880 | DETERMINISTIC — sticky-bucket-with-salt |

Bucket stability is the user-facing invariant; if it flickers, a tenant sees the feature flip on every page-load and you take a UX complaint. oyatie's xxHash3(tenant_id, flag_key, salt) → 0-99 has been audit-verified against 10 M tenant-flag pairs with zero flicker.

## Workload (d) — audience-ref evaluation (`audit_required: true`)

| Provider | p99 (ms) | audit emission overhead |
|---|---:|---|
| oyatie feature-flags (paid) | 1.3 | 820 µs (Ed25519 sign + audit-chain commit) |
| LaunchDarkly | n/a | LaunchDarkly does not emit per-evaluation audit; relies on log scraping |
| Flagsmith | 2.8 | 1 700 µs (asynchronous to a separate audit DB; durability gap of ≤ 5 s) |
| Unleash | n/a | No first-class audit-emission surface |
| Statsig | 1.9 | 950 µs (synchronous; signed JWT, not Ed25519) |
| GrowthBook | 3.4 | 2 100 µs (asynchronous) |

oyatie's audit-emission is **synchronous** with Ed25519 signing per ADR-0263 — the audit-chain row is committed before the evaluator returns. Statsig is closest (synchronous JWT, but no per-row signing; the JWT is per-batch). For compliance-class flags this distinction matters: SOC 2 §CC7.2 + KR-PIPA Art. 28 expect per-event signing, not per-batch.

## Cost benchmark

For a steady 100 k RPS evaluation workload at paid tenant_class on the substrate:

| Provider | Annualised cost (USD, all-in) |
|---:|---:|
| oyatie feature-flags (on cloud-k8s substrate) | 28 400 (4× Cloud-Hypervisor VMs + Cedar engine licence) |
| LaunchDarkly Cloud | 142 000 (Enterprise tier, > 1B events/month) |
| Flagsmith self-hosted | 38 200 (own infra + ops time) |
| Unleash self-hosted | 26 800 (own infra; lower throughput → more replicas) |
| Statsig Cloud | 96 000 (Pro tier) |
| GrowthBook Cloud | 84 000 (Pro tier) |

oyatie has the cost edge because the substrate is shared with `governance` (same Cedar engine) and `analytics` (same audience materialisation); LaunchDarkly + Statsig are stand-alone cloud spends.

## Caveats

These benchmarks measure server-side providers only. Client-side providers (LaunchDarkly JS client, GrowthBook JS, Unleash JS) are not comparable — they ship the entire ruleset to the browser and evaluate locally, which has different latency + cardinality tradeoffs and is out of scope per ADR-0159 §"Server-side-only OpenFeature surface".

Reproducibility: the benchmark harness lives at `benchmarks/ffbench/` and can be re-run against any provider with a Docker image via `oya benchmarks ff --provider <name>`. The harness is open-sourced and runs in CI weekly to detect drift; raw CSVs are committed under `benchmarks/results/`.
