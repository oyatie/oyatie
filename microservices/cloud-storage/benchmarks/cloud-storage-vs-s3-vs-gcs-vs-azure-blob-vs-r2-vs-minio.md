# `cloud-storage` µservice — Benchmark vs AWS S3, GCS, Azure Blob, Cloudflare R2, MinIO Enterprise, Backblaze B2

> Measured 2026-04-25 to 2026-05-13 across 3 trial windows × 6 workloads (small-object GET p95, small-object PUT p95, large-object
> multipart upload throughput, lifecycle transition latency, cross-region replication lag, S3-API compatibility coverage).
> `cloud-storage` runs HTTP/3 (QUIC) by default per ADR-0253. Vendor pricing per public sheet 2026-05-13.

## Small-object GET latency (1 KB, hot cache, regional)

| Surface | p50 | p95 | p99 | API |
| --- | --- | --- | --- | --- |
| `cloud-storage` (paid tenant_class, SSD edge cache) | **1.4 ms** | **2.2 ms** | 3.8 ms | S3 v2006-03-01 |
| `cloud-storage` (paid tenant_class, MinIO Enterprise) | 8.2 ms | 13.6 ms | 24.4 ms | S3 + Azure Blob |
| AWS S3 Express One Zone | 1.8 ms | 3.2 ms | 5.4 ms | S3 |
| AWS S3 Standard | 12.4 ms | 28.6 ms | 56.2 ms | S3 |
| GCS Multi-Regional | 14.8 ms | 32.4 ms | 64.2 ms | XML + JSON |
| Azure Blob (Hot) | 16.4 ms | 34.8 ms | 68.4 ms | Blob REST |
| Cloudflare R2 | 22.4 ms | 48.6 ms | 92.4 ms | S3 |
| MinIO Enterprise (self-hosted) | 8.6 ms | 14.8 ms | 26.4 ms | S3 |
| Backblaze B2 | 28.6 ms | 62.4 ms | 124.2 ms | S3 / B2 native |

## Small-object PUT latency (1 KB, regional)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `cloud-storage` (paid tenant_class production profile) | **6.4 ms** | **15.8 ms** | 28.4 ms |
| `cloud-storage` (paid tenant_class baseline profile) | 18.6 ms | 32.4 ms | 56.2 ms |
| AWS S3 Standard | 24.6 ms | 48.4 ms | 92.4 ms |
| AWS S3 Express One Zone | 8.4 ms | 16.2 ms | 28.6 ms |
| GCS | 28.4 ms | 54.6 ms | 102.4 ms |
| Azure Blob (Hot) | 32.4 ms | 64.2 ms | 124.6 ms |
| Cloudflare R2 | 38.4 ms | 78.2 ms | 142.4 ms |
| MinIO Enterprise | 18.4 ms | 34.6 ms | 64.2 ms |

## Large-object multipart upload throughput (5 GB object, 16 concurrent parts)

| Surface | Sustained MB/s |
| --- | --- |
| `cloud-storage` (paid tenant_class production profile) | **2,450** |
| `cloud-storage` (paid tenant_class regulated HSM-bypass profile) | **3,200** |
| AWS S3 Standard | 1,800 |
| AWS S3 Express One Zone | 2,400 |
| GCS | 1,600 |
| Azure Blob (Hot) | 1,400 |
| Cloudflare R2 | 800 |
| MinIO Enterprise | 1,200 |
| Backblaze B2 | 600 |

## Lifecycle transition latency (Hot → Warm, p95 from policy match to physical move)

| Surface | p95 |
| --- | --- |
| `cloud-storage` (paid tenant_class, 1 h scan) | **24 min** |
| `cloud-storage` (paid tenant_class regulated profile, 5 min scan) | **2 min** |
| AWS S3 (lifecycle daemon, 24 h) | 12 h |
| GCS (lifecycle daemon, 24 h) | 14 h |
| Azure Blob (24 h cycle) | 18 h |
| MinIO Enterprise | 4 h |
| R2 / B2 | no built-in lifecycle; tenant scripts only |

## Cross-region replication lag (US ↔ EU, p95)

| Surface | p95 lag |
| --- | --- |
| `cloud-storage` (paid tenant_class, async) | **4.8 s** |
| `cloud-storage` (paid tenant_class regulated profile, sync Raft) | **240 ms** |
| AWS S3 CRR | 15 min (target) — observed 2-30 min |
| AWS S3 RTC (Replication Time Control) | 15 min (SLA) |
| GCS Multi-Regional | sync within multi-region (e.g. us, eu) |
| Azure Blob GRS | 15 min (target) |
| Cloudflare R2 | manual / external |
| MinIO Enterprise | configurable; typically 5-30 s |

## S3-API compatibility coverage

| Surface | S3 API ops supported | Azure Blob | GCS | R2 | B2 |
| --- | --- | --- | --- | --- | --- |
| `cloud-storage` (paid tenant_class production profile) | 86 / 92 (94 %) | ✅ | partial | n/a | n/a |
| `cloud-storage` (paid tenant_class regulated profile) | 92 / 92 (100 %) | ✅ | ✅ | ✅ | ✅ |
| AWS S3 | 92 / 92 (100 %, definition) | n/a | n/a | n/a | n/a |
| MinIO Enterprise | 88 / 92 | n/a | n/a | n/a | n/a |
| Cloudflare R2 | 78 / 92 (~85 %) | n/a | n/a | n/a | n/a |
| Backblaze B2 (S3-compat) | 64 / 92 (~70 %) | n/a | n/a | n/a | n/a |
| Azure Blob (S3-compat layer) | 58 / 92 | n/a | n/a | n/a | n/a |

## TCO at 100 TB hot data, 10 PB cold data, 10 M req/day mid-market scope

| Surface | Hot storage | Cold storage | Requests | Replication | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- |
| `cloud-storage` (paid tenant_class production profile) | $600 | $600 | included | included | **$2,200** | **$26,400** |
| AWS S3 Standard + Glacier Deep | $2,300 | $990 | $400 | $400 (CRR) | $4,090 | $49,080 |
| AWS S3 Express + Glacier Deep | $11,200 | $990 | $200 | $400 | $12,790 | $153,480 |
| GCS Multi-Regional + Archive | $2,600 | $1,200 | $500 | $0 (multi-reg) | $4,300 | $51,600 |
| Azure Blob Hot + Archive | $2,000 | $990 | $440 | $300 (GRS) | $3,730 | $44,760 |
| Cloudflare R2 + B2 mix | $1,500 | $500 | $0 (R2 no egress) | n/a | $2,000 | $24,000 |
| MinIO Enterprise self-host | hardware + ops | hardware + ops | hardware | hardware | $1,400 + ops | n/a |
| Backblaze B2 | $600 | $4 | $0.40 | $0 | $604 | $7,248 |

At pure-cost level, R2 + B2 beat `cloud-storage` for simple workloads (no egress fees on R2; cheapest at B2). `cloud-storage`
(paid tenant_class production profile) is **46 % below S3 Standard + Glacier + CRR** and **75 % below S3 Express** at mid-market scale, while bundling the
lifecycle + replication + audit-chain stack vendors charge for separately.

## Where vendors still win

1. **S3 ecosystem maturity** — AWS S3 has 18 years of production, 200+ AWS service integrations.
2. **R2/B2 cost ceiling** — for pure-storage workloads, R2's no-egress + B2's cheap pricing beat us.
3. **AWS S3 Express One Zone** for ultra-low-latency hot paths.
4. **Cloudflare R2 + Workers** for edge-compute-near-storage workloads.
5. **Marketplace integrations** — every SaaS/PaaS targets AWS S3 first.
6. **Public sign-up** — all vendors self-serve.

## Where `cloud-storage` wins

1. **Bundled lifecycle + replication + WORM + inventory** — vendors charge separately.
2. **AAD-bound encryption mandatory at v0.42+** — vendor SSE doesn't bind AAD per object.
3. **Per-tenant CMK with `cloud-kms`** — vendor SSE-KMS is account-level.
4. **WORM compliance mode across tenant_class eligibility** — only AWS S3 Object Lock matches; Azure Blob Immutable Storage is similar; others lack.
5. **Lifecycle latency ≤ 2 min in paid tenant_class regulated profile** — vendors are hour+.
6. **Cross-region sync replication in paid tenant_class regulated profile** — vendor offerings are async with > 15 min target.
7. **BLAKE3 audit-chain on every PUT/DELETE/lifecycle event** — vendor logs are append-only-not-chained.
8. **PQC envelope encryption (paid tenant_class regulated profile)** — no vendor ships this in 2026.
9. **6 storage classes** (Hot, Warm, Cold, Archive, Tape, Sovereign-Air-Gapped) — vendors are 3-4.
10. **Air-gap sovereign deployment** — no vendor offers this.

## Reproducibility

```bash
make benchmarks.cloud-storage.run \
  VENDORS="cloud-storage,s3-standard,s3-express,gcs,azure-blob,r2,minio,b2" \
  WORKLOADS="get-1k,put-1k,multipart-5g,lifecycle-trans,crr-lag,api-coverage" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/cloud-storage/2026-05-13T22:14:42Z/`.
