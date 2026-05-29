---
doc_class: Benchmark
microservice: audit-chain
benchmark_date: 2026-05-20
related_adrs: [ADR-0028, ADR-0263, ADR-0296, ADR-0329, ADR-0330, ADR-0331]
doc_status: published
---

# Benchmarks — oyatie audit-chain vs Splunk Audit vs Datadog Audit Logs vs AWS CloudTrail Lake

Workloads measured: (a) sustained emit throughput, (b) seal latency (event-emit → tamper-evidence-anchored), (c) verify-chain query latency (cryptographic re-verification), (d) regulator-evidence export wall-clock, (e) annual TCO at 100 k events/sec sustained + 1 PiB cold-tier retention + tenant-visible query.

Hardware (oyatie on-prem paid tenant_class): 5× sealer nodes (8 vCPU AMD EPYC 9354P, 32 GiB DDR5, 1 TiB NVMe), 3× HSM (Thales Luna 7 PCIe A790, FIPS 140-3 Level 2), PostgreSQL 16.6 primary + 2 replicas (16 vCPU, 64 GiB RAM, 3.84 TiB NVMe), SeaweedFS-S3 6 nodes × 7.68 TiB. Network: 25 GbE leaf-spine, MTU 9000.

Cloud comparators measured on equivalent service class; SaaS pricing taken from public 2026-Q2 list price unless noted.

## Workload (a) — sustained emit throughput (events/sec)

| Engine | Sustained (events/sec) | Burst (events/sec, ≤ 60 s) | Notes |
|---|---:|---:|---|
| oyatie audit-chain (paid, 3 HSMs) | 100 000 | 500 000 | Batch sealing every 1 s amortises HSM signs |
| oyatie audit-chain (paid, 5 HSMs) | 500 000 | 2 000 000 | Active-active sealer across 3 AZs |
| Splunk Audit (managed; "Audit Search" entitlement) | 30 000 | 100 000 | Splunk's HEC endpoint scales but search backpressure caps practical rate |
| Datadog Audit Logs (per-org enterprise) | 50 000 | 200 000 | Per Datadog 2026 SLA |
| AWS CloudTrail (data events; high-volume service class) | 1 000 000 | 5 000 000 | CloudTrail Lake scales massively but is read-rate-limited (~ 100 qps) |
| AWS CloudTrail Lake | (managed, ingest-only metric) | (managed) | Pricing per GiB ingested ($2.50/GiB Standard, $1/GiB Lake) |

Reading: CloudTrail's emit envelope is the largest by construction (it's a managed firehose). For multi-tenant SaaS where the chain must be tenant-visible (not just AWS-account-visible), oyatie's per-cell paid envelope is competitive and tenant-isolated. Splunk and Datadog backpressure under sustained > 50 k events/sec without dedicated entitlement upgrades.

## Workload (b) — seal latency (event-emit → tamper-evidence-anchored)

| Engine | p50 (s) | p99 (s) | Anchor mechanism |
|---|---:|---:|---|
| oyatie audit-chain (paid, batch-1s) | 0.4 | 1.0 | Merkle root + Ed25519 (HSM L2) |
| oyatie audit-chain (paid, batch-1s) | 0.3 | 1.0 | Merkle root + Ed25519 (HSM L2) cross-AZ |
| oyatie audit-chain (paid, batch-1s) | 0.5 | 1.0 | Merkle root + Ed25519 (HSM L3) + dual-control |
| Splunk Audit (HMAC-signed event) | 0.5 | 2.0 | Per-event HMAC (NOT non-repudiation; key shared) |
| Datadog Audit Logs | 1.0 | 3.5 | Sequence-numbered, no cryptographic anchor |
| AWS CloudTrail | 5.0 (data events) | 20.0 (data events) | CloudTrail digest (SHA-256, RSA-signed; per-trail-hourly) |
| AWS CloudTrail Lake | 5.0 (data events) | 20.0 (data events) | Same as CloudTrail (digest is upstream of Lake) |

Reading: oyatie's seal latency is competitive with CloudTrail; we anchor every 1 s while CloudTrail anchors hourly (digest). Splunk + Datadog have lower latency but don't provide cryptographic non-repudiation. For compliance-bound tenants this is a categorical difference: a non-cryptographic anchor cannot satisfy SEC 17a-4(f) WORM-equivalence.

## Workload (c) — verify-chain query latency (cryptographic re-verification of N events)

| Engine | 1 K events (ms) | 100 K events (ms) | 1 M events (ms) | 10 M events (s) |
|---|---:|---:|---:|---:|
| oyatie audit-chain (paid, in-region) | 22 | 184 | 920 | 12.4 |
| oyatie audit-chain (paid, in-region) | 18 | 148 | 612 | 7.8 |
| Splunk Audit "verify" (HMAC re-check) | 80 | 6 200 | (not supported at scale) | (not supported) |
| Datadog Audit Logs | (no verify) | (no verify) | (no verify) | (no verify) |
| AWS CloudTrail digest verification (per-hour digest) | 1 200 (1 digest) | 60 000 (~100 digests) | (not at single-event granularity) | (not at single-event granularity) |
| AWS CloudTrail Lake (SQL query, no crypto verify) | 200 | 1 800 | 12 000 | (out of envelope; per-query slot limit) |

Reading: oyatie supports cryptographic verification at single-event granularity over millions of events. CloudTrail's digest verification is per-hour-trail, not per-event; for single-event provenance you must reconstruct the hourly digest and find your event inside. Splunk + Datadog cannot offer cryptographic verification at all.

## Workload (d) — regulator-evidence export wall-clock (export 90 d of 1.6 M events with full Merkle proofs)

| Engine | Wall-clock | External-verify time |
|---|---:|---:|
| oyatie audit-chain (paid) | 12 min | 47 s (auditor's laptop, no oyatie tooling) |
| oyatie audit-chain (paid) | 8 min | 38 s |
| Splunk Audit export (CSV) | 4 min | (no cryptographic verify) |
| Datadog Audit Logs export | 6 min | (no cryptographic verify) |
| AWS CloudTrail Lake export (Athena-equivalent) | 18 min (Athena CTAS to S3) | (digests verified via separate AWS CLI walk) |

Reading: the wall-clock is comparable; the categorical difference is that oyatie's export carries everything needed for an auditor to verify independently. Splunk/Datadog exports are unverifiable by construction.

## Workload (e) — annual TCO at 100 k events/sec sustained + 1 PiB cold tier + 7-y retention

| Platform | Hardware / compute (USD) | Cold storage (USD) | Licence (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie audit-chain (paid on-prem, 5 sealers + 3 HSMs) | 312 000 | 92 000 (SeaweedFS @ 1 PiB) | 0 | 248 000 (2 SRE × 0.4 FTE) | 652 000 |
| oyatie audit-chain (paid, 9 sealers + 5 HSMs, multi-AZ) | 780 000 | 240 000 (cross-region SeaweedFS replica) | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 392 000 |
| Splunk Audit (managed, "Enterprise + Audit" entitlement) | 0 | 240 000 (S3) | 2 800 000 (Splunk per-GiB-indexed @ 100 k events/sec ~ 3 TiB/day) | 124 000 (1 SRE × 0.2 FTE) | 3 164 000 |
| Datadog Audit Logs (Enterprise service class, 100 k events/sec) | 0 | 240 000 | 1 900 000 | 124 000 | 2 264 000 |
| AWS CloudTrail data events (S3 destination) + CloudTrail Lake | 0 | 240 000 | 1 240 000 (data event @ $0.10/100k events + Lake @ $2.50/GiB) | 124 000 | 1 604 000 |

Reading: oyatie paid tenant_class is cost-competitive vs every managed alternative AND provides cryptographic non-repudiation that none of them match. The paid tenant_class premium goes to multi-AZ + cross-region — appropriate for revenue-bearing tenants.

Caveats:

- HSM hardware amortised over 5 years; refresh cycle assumed at year-5.
- Datadog/Splunk pricing assumes no negotiation; enterprise contracts commonly receive 30-50 % discount at scale. Even at 50 % off, Splunk + Datadog remain at ~ 2× oyatie paid TCO.
- Ops cost includes the audit-chain rotation + 1 SRE for HSM lifecycle + 0.2 SRE for cross-µservice emission-adapter incidents.

## Reproducibility

The benchmark harness is at `benchmarks/audit-chainbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks audit-chain \
    --workload sustained-emit-100k-events-sec \
    --tenant-class paid \
    --output ./benchmark-results.json
```

Cloud comparators (Splunk, Datadog, CloudTrail) require valid `--cloud-credentials` for the relevant SaaS. Results live at `benchmarks/results/audit-chain/<date>.csv` and are re-run weekly in CI to detect drift.
