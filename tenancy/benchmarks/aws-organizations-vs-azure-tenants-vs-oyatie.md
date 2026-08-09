---
doc_class: Benchmark
microservice: tenancy
benchmark_date: 2026-05-20
related_adrs: [ADR-0329, ADR-0330, ADR-0331, ADR-0244, ADR-0251]
doc_status: published
---

# Benchmarks — oyatie tenancy vs AWS Organizations vs Azure Tenants vs GCP Organizations vs Auth0 Organizations vs Okta Tenants vs Salesforce Multi-Org

Workloads measured: (a) tenant-provision latency, (b) tenant-read latency at scale, (c) sub-scope traversal, (d) DR failover RTO, (e) annual TCO at 10M tenants + 100M sub-scopes.

Hardware (oyatie on-prem paid tenant_class expanded deployment): 12× tenancy API pods (16 vCPU AMD EPYC 9354P, 64 GiB DDR5), 6× lifecycle workers, 6× isolation enforcers, 21-node PostgreSQL Citus 12.1 cluster (3 AZs × 7 nodes).

## Workload (a) — tenant provisioning latency

| Engine | p50 (s) | p99 (s) |
|---|---:|---:|
| oyatie tenancy (paid tenant_class baseline) | 8.4 | 14.6 |
| oyatie tenancy (paid tenant_class expanded deployment) | 3.2 | 5.8 |
| AWS Organizations (Create Account) | 60-180 | 300 (often slower) |
| Azure Tenants (Create Tenant) | 120-300 | 600 |
| GCP Organizations | 90-240 | 480 |
| Auth0 Organizations | 1.8 | 4.2 |
| Okta Tenants | 30-90 | 180 |
| Salesforce Multi-Org | 600+ | 3 600 (1 h for full org provisioning) |

Reading: oyatie paid tenant_class expanded deployment provisions in seconds. Cloud-provider tenant provisioning is slow (per-account heavy resources). Auth0 is comparable for lightweight OAuth-org provisioning; Salesforce is the slowest by 1-3 orders of magnitude (full Salesforce org has extensive default metadata + sandboxes).

## Workload (b) — tenant-read latency at scale (10M-tenant corpus)

| Engine | p99 (ms) |
|---|---:|
| oyatie tenancy (paid tenant_class baseline) | 40 |
| oyatie tenancy (paid tenant_class expanded deployment) | 15 |
| AWS Organizations DescribeAccount | 240 |
| Azure Tenants Get-Tenant | 280 |
| GCP Organizations Get | 220 |
| Auth0 GetOrganization | 80 |
| Okta GetTenant | 140 |
| Salesforce Org metadata | 480 |

Reading: oyatie paid tenant_class expanded deployment leads. Citus 12.1 + PostgreSQL JSONB partitioning provides sub-20-ms lookup at scale. Cloud-provider APIs have per-tenant rate limits + multi-tenant query queue contention.

## Workload (c) — sub-scope traversal (100M-sub-scope corpus, 5-level hierarchy)

| Engine | 1-hop p99 (ms) | Full-tree-traversal p99 (ms) |
|---|---:|---:|
| oyatie tenancy (paid tenant_class baseline) | 80 | 1 800 |
| oyatie tenancy (paid tenant_class expanded deployment) | 30 | 720 |
| AWS Organizations OU listing | 280 | 6 200 |
| Azure Management Groups | 320 | 7 400 |
| GCP Folders | 240 | 5 800 |
| Auth0 Organizations Sub-Orgs | 120 | 2 400 |
| Salesforce Account Hierarchy | 480 | 12 000 |

Reading: oyatie paid tenant_class expanded deployment leads on sub-scope traversal. The custom graph-traversal kernel optimised for tenancy hierarchies provides categorical edge.

## Workload (d) — DR failover RTO (regional outage simulation)

| Engine | RTO (min) | RPO (s) |
|---|---:|---:|
| oyatie tenancy (paid tenant_class expanded deployment) | 12 | 38 |
| AWS Organizations (us-east-1 to us-west-2 manual) | 60-180 | varies |
| Azure Tenants (failover via management group) | 90-240 | varies |
| GCP Organizations | manual | manual |
| Auth0 Organizations | 30-60 | 300 |
| Okta Tenants | 60-180 | varies |
| Salesforce Multi-Org Disaster Recovery service | 4 h | 60 min |

Reading: oyatie paid tenant_class expanded deployment's RTO ≤ 15 min is competitive with the fastest cloud comparator (Auth0). Most others are slower or manual.

## Workload (e) — annual TCO at 10M tenants + 100M sub-scopes

| Platform | Hardware (USD) | Licence (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|
| oyatie tenancy (paid tenant_class expanded deployment on-prem) | 1 200 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 572 000 |
| AWS Organizations (10M AWS accounts) | 0 | (per-account services AWS doesn't directly charge for Organizations itself; downstream costs ~ $5/account/yr for IAM + CloudTrail at minimum) | n/a | ~ 50 000 000 (downstream costs) |
| Azure Tenants (10M Azure tenants) | 0 | (per-tenant licenses ~ $4-12/user; depends on entitlements) | n/a | varies |
| GCP Organizations | 0 | (similar to AWS; downstream costs) | n/a | varies |
| Auth0 Organizations | 0 | 4 800 000 ($40/MAU × 10M ÷ ~ 100 active per tenant ~ 100k MAU × $40 × 12) | 124 000 | 4 924 000 |
| Okta Tenants | 0 | 8 400 000 ($70/MAU × 100k MAU × 12) | 124 000 | 8 524 000 |
| Salesforce Multi-Org | 0 | 240 000 000+ ($200/user/mo × 10M × 12 — orders of magnitude more) | n/a | impractical at 10M scale |

Reading: cloud-provider tenant-models are designed for hundreds-to-thousands of tenants, not 10M. For oyatie-scale multi-tenancy (10M+), self-hosted is the only viable path. oyatie paid tenant_class expanded deployment at $1.6M/yr is 30× cheaper than Auth0 at the same scale, 5× cheaper than Okta, and Salesforce becomes impractical.

For the typical SaaS-platform-operator running 10-100k tenants, Auth0 or oyatie paid tenant_class baseline are competitive on TCO; oyatie wins on per-tenant database isolation + sovereign-pack features.

## Workload (f) — sovereign-pack tenant features (oyatie-exclusive)

"Provision KR-PIPA-Finance tenant with 신용평가회사 KYB + pack-resident HSM-encrypted database + dual-control admin."

| Engine | Support |
|---|---|
| oyatie tenancy (paid tenant_class regulated-pack overlay) | Yes |
| AWS / Azure / GCP / Auth0 / Okta / Salesforce | No (no Korean sovereign-residency tenancy product) |

Categorical differentiator.

## Reproducibility

Benchmark harness at `benchmarks/tenancy/`. Re-run weekly.
