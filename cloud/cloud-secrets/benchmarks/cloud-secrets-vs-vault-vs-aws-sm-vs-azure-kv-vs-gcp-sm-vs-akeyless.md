# `cloud-secrets` µservice — Benchmark vs HashiCorp Vault, AWS Secrets Manager, Azure Key Vault, GCP Secret Manager, Akeyless

> Measured 2026-04-22 to 2026-05-14. 6 vendors × 4 workloads × 3 trial windows of 10 minutes each. Reads from a warmed cache,
> writes against a fresh-version path. mTLS to all surfaces (where available). HTTP/3 to `cloud-secrets`; HTTP/2 to vendors.

## Latency — single-secret read (warm cache, ≤ 4 KiB)

| Surface | p50 | p95 | p99 | Region |
| --- | --- | --- | --- | --- |
| `cloud-secrets` (paid) | **3.8 ms** | **9.2 ms** | **18 ms** | us-east-2 |
| HashiCorp Vault Enterprise (Consul backend) | 7.4 ms | 22 ms | 51 ms | us-east-2 |
| AWS Secrets Manager (DescribeSecret + GetSecretValue) | 24 ms | 58 ms | 110 ms | us-east-2 |
| Azure Key Vault (Standard) | 38 ms | 95 ms | 190 ms | eastus2 |
| GCP Secret Manager | 28 ms | 71 ms | 140 ms | us-east4 |
| Akeyless SaaS | 19 ms | 47 ms | 92 ms | us-east-2 (vendor) |

## Latency — write (new version, ≤ 4 KiB)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `cloud-secrets` (paid) | **11 ms** | **24 ms** | **48 ms** |
| Vault Enterprise | 19 ms | 52 ms | 110 ms |
| AWS Secrets Manager (UpdateSecret) | 86 ms | 220 ms | 410 ms |
| Azure Key Vault | 140 ms | 340 ms | 690 ms |
| GCP Secret Manager (AddSecretVersion) | 92 ms | 220 ms | 420 ms |
| Akeyless | 58 ms | 130 ms | 250 ms |

## Latency — dynamic credential issuance (Postgres user)

| Surface | p50 | p95 | p99 | Native dynamic? |
| --- | --- | --- | --- | --- |
| `cloud-secrets` (paid) | **180 ms** | **310 ms** | **520 ms** | ✅ |
| Vault Enterprise (Postgres secret engine) | 240 ms | 390 ms | 680 ms | ✅ |
| AWS Secrets Manager (Lambda rotation) | 4.8 s | 9.2 s | 18 s | ❌ (Lambda-based) |
| Azure Key Vault | n/a | n/a | n/a | ❌ |
| GCP Secret Manager | n/a | n/a | n/a | ❌ |
| Akeyless | 320 ms | 540 ms | 950 ms | ✅ |

## Throughput — sustained reads/s per tenant

| Surface | Throughput (sustained) | Burst (5 s) |
| --- | --- | --- |
| `cloud-secrets` (paid) | **5,000** | **15,000** |
| `cloud-secrets` (paid) | **50,000** | **150,000** |
| Vault Enterprise (HA cluster) | 2,800 | 8,400 |
| AWS Secrets Manager | 5,000 (account-wide soft limit; throttled per-secret) | 10,000 (with retry) |
| Azure Key Vault | 2,000 (subscription-wide) | 4,000 (with retry, throttled) |
| GCP Secret Manager | 6,000 (project-wide) | n/a |
| Akeyless SaaS | 1,500 (tier-dependent) | 5,000 |

## Cost — 10,000 secrets, 10 M reads/mo, 50 k writes/mo, 100 k dynamic issuances/mo

| Surface | Storage | Reads | Writes | Dynamic | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- |
| `cloud-secrets` (paid) | included | included | included | included | **$2,400** | **$28,800** |
| Vault Enterprise (self-hosted on AWS) | $0 | $0 | $0 | $0 | $9,200 (compute+ops) | $110,400 |
| AWS Secrets Manager | $4,000 ($0.40/secret) | $500 ($0.05 / 10k) | $25 | $20 | $4,545 | $54,540 |
| Azure Key Vault | $30 (standard) | $250 ($0.025 / 10k) | $13 | n/a | $293 + ops | $3,516 + ops |
| GCP Secret Manager | $60 ($0.06/secret/active version) | $300 ($0.03 / 10k) | $1.50 | n/a | $361 + ops | $4,332 + ops |
| Akeyless SaaS (Enterprise) | tenant_class-bundled | bundled | bundled | bundled | $5,400 | $64,800 |

Notes:
1. AWS SM is cheaper for storage at 10k secrets but loses on throughput ceilings + dynamic credential latency.
2. Azure KV / GCP SM look cheap on the sheet but require building dynamic credential automation (Lambda / Cloud Function) yourself
   — TCO including that engineering load is roughly 3-5x sheet price.
3. Vault Enterprise's $0 list price hides ~$110k/yr in compute + ops + license — the comparison row is a representative cost.

## Governance / compliance surface

| Surface | ABAC | Audit chain | Tamper-evidence | HSM | encryption-key BYOK (ADR-0251 §D-10) | Per-tenant pack overlays |
| --- | --- | --- | --- | --- | --- | --- |
| `cloud-secrets` | Cedar (in-process) | BLAKE3 chain | ✅ (optional blockchain anchor) | ✅ Level 3 | ✅ KEK ceremony | ✅ |
| Vault Enterprise | identity policies | audit devices | append-only | ✅ | ✅ | ❌ |
| AWS SM | IAM (round-trip) | CloudTrail | append-only | ✅ via CloudHSM | partial | ❌ |
| Azure KV | RBAC + access policies | Azure Monitor | append-only | ✅ Dedicated HSM | ✅ | ❌ |
| GCP SM | IAM (round-trip) | Cloud Audit Logs | append-only | partial | partial | ❌ |
| Akeyless | RBAC | activity log | append-only | ✅ | ✅ | partial |

## Where `cloud-secrets` wins

1. Lowest p50/p95 across reads + writes — by 2-7x at the p95.
2. Dynamic credential issuance is sub-second at p95 vs 9.2 s for AWS SM via Lambda.
3. Cedar in-process eliminates the IAM round-trip cost.
4. Per-tenant pack overlays flip compliance posture without code changes.
5. BLAKE3 audit chain is verifiable client-side.
6. HTTP/3 default.

## Where vendors still win

1. **Self-service onboarding** — AWS SM / Azure KV / GCP SM are 1-click in their consoles; `cloud-secrets` needs tenant provisioning.
2. **Native KMS integration** — AWS KMS in particular is more tightly coupled with AWS services; `cloud-secrets` requires explicit
   adapter use.
3. **Vault's secret engine ecosystem** — 50+ secret engines vs `cloud-secrets`'s 16 first-party types + extension framework.

## Reproducibility

```bash
make benchmarks.cloud-secrets.run \
  VENDORS="cloud-secrets,vault,aws-sm,azure-kv,gcp-sm,akeyless" \
  WORKLOADS="read-warm,read-cold,write,dynamic-postgres" \
  TRIALS=3 \
  DURATION=10m
```

Evidence: `.foundry/evidence/benchmarks/cloud-secrets/2026-05-14T16:02:11Z/`.
