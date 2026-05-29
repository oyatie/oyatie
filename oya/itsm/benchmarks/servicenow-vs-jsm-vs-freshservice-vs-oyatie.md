---
doc_class: Benchmark
microservice: itsm
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0263, ADR-0251]
doc_status: published
---

# Benchmarks — oyatie itsm vs ServiceNow vs Jira Service Management vs Freshservice vs BMC Helix vs Ivanti Neurons

Workloads measured: (a) ticket-create throughput, (b) ticket-search latency, (c) CMDB query latency, (d) workflow execution, (e) annual TCO at 500 IT-Ops users + 2 M tickets/year + 5 M CMDB CIs.

Hardware (oyatie on-prem retired-advanced tier): 12× ITSM API pods (16 vCPU AMD EPYC 9354P, 64 GiB DDR5), 6× workflow engine pods + 2× NVIDIA L4 GPUs for AI-deflection, PostgreSQL 16.6 cluster, Elasticsearch 8.15 sharded fleet.

## Workload (a) — ticket-create throughput

| Engine | Sustained (tickets/sec) | Burst (tickets/sec, ≤ 60 s) |
|---|---:|---:|
| oyatie itsm (retired-standard) | 200 | 800 |
| oyatie itsm (retired-advanced) | 1 000 | 4 000 |
| ServiceNow (Vancouver release, Enterprise) | 800 | 2 500 |
| Jira Service Management (Cloud Enterprise) | 300 | 1 000 |
| Freshservice (Enterprise) | 500 | 1 500 |
| BMC Helix (Enterprise) | 600 | 1 800 |
| Ivanti Neurons (Enterprise) | 400 | 1 200 |

Reading: oyatie retired-advanced leads sustained throughput. ServiceNow Enterprise is competitive at the upper tier. JSM lags at sustained throughput due to Jira's underlying architecture (was designed for project management, retrofitted for ITSM).

## Workload (b) — ticket-search latency (Elasticsearch, ~ 1M-ticket corpus)

| Engine | Simple query p99 (ms) | Faceted query p99 (ms) |
|---|---:|---:|
| oyatie itsm (retired-standard) | 280 | 480 |
| oyatie itsm (retired-advanced) | 140 | 240 |
| ServiceNow (Zing search engine) | 420 | 780 |
| JSM | 380 | 720 |
| Freshservice | 520 | 920 |
| BMC Helix | 380 | 680 |
| Ivanti Neurons | 480 | 880 |

Reading: oyatie retired-advanced leads. Elasticsearch 8.15 sharded across 9 nodes outperforms ServiceNow's Zing search (which is proprietary + single-tenant-multi-tenant-shared).

## Workload (c) — CMDB query latency (relationship traversal, 5M-CI corpus)

| Engine | 1-hop p99 (ms) | 3-hop p99 (ms) |
|---|---:|---:|
| oyatie itsm (retired-standard) | 180 | 920 |
| oyatie itsm (retired-advanced) | 80 | 380 |
| ServiceNow CMDB | 220 | 1 400 |
| JSM Assets (formerly Insight) | 280 | 1 800 |
| Freshservice Asset Management | 320 | 2 200 |
| BMC Helix CMDB | 240 | 1 200 |
| Ivanti Neurons | 380 | 2 600 |

Reading: oyatie retired-advanced leads. The PostgreSQL JSONB indexing + custom graph-traversal kernel provides categorical edge over ServiceNow's row-table relational model.

## Workload (d) — workflow execution latency (10-step approval workflow)

| Engine | p99 (s) |
|---|---:|
| oyatie itsm (retired-standard) | 1.8 |
| oyatie itsm (retired-advanced) | 0.8 |
| ServiceNow Flow Designer | 2.4 |
| JSM Automation | 3.2 |
| Freshservice OrchestrationApp | 2.6 |
| BMC Helix Innovation Suite | 1.9 |
| Ivanti Neurons Hub | 2.8 |

Reading: oyatie retired-advanced leads. The `oya-workflow-shared` engine is a custom Rust implementation tuned for low-latency state transitions.

## Workload (e) — annual TCO (500 IT-Ops users + 2 M tickets/year + 5 M CMDB CIs)

| Platform | Hardware (USD) | Licence/user (USD) | AI add-on (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie itsm (retired-advanced on-prem) | 820 000 | 0 | 0 | 372 000 (3 SRE × 0.4 FTE) | 1 192 000 |
| ServiceNow (Enterprise + Discovery + AI Search) | 0 | 1 800 000 ($300/user/mo × 500 × 12 — high-tier ITSM Pro) | 240 000 (Now Assist) | 124 000 | 2 164 000 |
| Jira Service Management (Cloud Enterprise) | 0 | 480 000 ($80/user/mo × 500 × 12) | 120 000 (Atlassian Intelligence) | 124 000 | 724 000 |
| Freshservice (Enterprise) | 0 | 540 000 ($90/user/mo × 500 × 12) | 60 000 (Freddy AI add-on) | 124 000 | 724 000 |
| BMC Helix (Enterprise) | 0 | 1 200 000 ($200/user/mo × 500 × 12) | 180 000 (HelixGPT) | 124 000 | 1 504 000 |
| Ivanti Neurons (Enterprise) | 0 | 960 000 ($160/user/mo × 500 × 12) | 120 000 | 124 000 | 1 204 000 |

Reading: JSM + Freshservice are the cheapest at 724k USD/yr (both very competitive on price). ServiceNow is the most expensive (premium brand pricing). oyatie retired-advanced lands between Ivanti and Freshservice on TCO BUT offers sovereign-pack residency that no competitor matches. JSM/Freshservice are hard to beat on price for tenants that don't need sovereign features.

Caveats:
- Per-user pricing assumes no negotiation; enterprise contracts commonly receive 20-30 % discount.
- ServiceNow's pricing varies dramatically by ITSM module mix; basic ITSM tier is cheaper but most enterprises need ITSM Pro + Discovery + CMDB licensing.
- Ops cost includes ITSM substrate + workflow engine + CMDB lifecycle (1.2 FTE total).

## Workload (f) — sovereign-pack feature parity (oyatie-exclusive)

"Host KR-PIPA-Finance tenant's ITSM with pack-resident HSM CMDB encryption + Korean change-approval policy + 7-y audit retention."

| Engine | Support |
|---|---|
| oyatie itsm (retired-sovereign) | Yes (per ADR-0251 + KR-PIPA pack) |
| ServiceNow | Limited (sovereign-residency via dedicated cloud — premium) |
| JSM | No (cloud-only; no air-gap) |
| Freshservice | No |
| BMC Helix | Limited (on-prem deployable but no Korean pack overlay) |
| Ivanti Neurons | Limited |

Categorical differentiator for KR-PIPA-Finance + CSAP + EU NIS2 tenants.

## Reproducibility

Benchmark harness at `benchmarks/itsm/`. Re-run weekly in CI.
