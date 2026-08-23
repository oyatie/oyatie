---
purpose: Oyatie — SLO Catalog
doc_status: published
---

# Oyatie — SLO Catalog

> **Status:** Draft v0.1 — 2026-05-09. Per-surface SLOs across all 7 axes with error-budget policy and burn-rate gates per Google SRE workbook.
> **Owner:** `ops-sre-reliability`. Updates per [DOC-CATALOG.md `doc.slo_catalog`](DOC-CATALOG.md).
> **Companion:** [RUNBOOKS-INDEX.md](RUNBOOKS-INDEX.md), [INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md), [RELEASE-MANAGEMENT.md](RELEASE-MANAGEMENT.md).

## 1. Universal SLO classes

| Class | Target preview / stable / GA |
|---|---|
| **Availability (control-plane API)** | 99.9% / 99.95% / 99.99% over 30d |
| **Availability (data-plane API)** | 99.9% / 99.95% / 99.99% |
| **Availability (analytics-plane API)** | 99.5% / 99.9% / 99.95% |
| **Latency p95** | per-surface; declared per row |
| **Latency p99** | per-surface; declared per row |
| **Durability (storage)** | 99.999999999% (11 nines) for object storage; 99.99999% for block |
| **Correctness (semantic)** | 100% on the published contract; per-PR contract test gate |
| **Audit-chain emission completeness** | 100% (any miss is a P0 incident) |
| **DSR cascade SLA** | 30d preview / 14d stable / 7d GA |

## 2. Per-axis SLO targets (preview wave; tighter at stable + GA)

### 2.1 SaaS multi-tenant
| Surface | Availability | p95 latency | p99 latency |
|---|---|---|---|
| Tenant API control plane | 99.9% | 200ms | 500ms |
| Workflow engine data plane | 99.9% | 100ms | 300ms |
| Plugin runtime invocation | 99.9% | 200ms | 1s |
| Marketplace catalog read | 99.95% | 50ms | 150ms |

### 2.2 Workspace
| Surface | Availability | p95 | p99 |
|---|---|---|---|
| Mail SMTP receive | 99.9% deliverability | (per-stage) | (per-stage) |
| Calendar API | 99.95% | 100ms | 300ms |
| Doc edit propagation | 99.9% | 80ms intra-region | 200ms |
| Drive object download | 99.95% | 100ms TTFB | 500ms TTFB |
| Meet RTT (intra-region) | 99.9% | 150ms | 250ms |
| Meet recording playback start | 99.9% | 1s | 3s |

### 2.3 Vertical (per-vertical override possible)
| Surface | Default |
|---|---|
| Vertical control plane | 99.9% |
| Vertical data plane (e.g. clinical exchange) | 99.95% (HIPAA-driven) |

### 2.4 Foundry
| Surface | Availability | p95 | p99 |
|---|---|---|---|
| Capability invocation control plane | 99.9% | 100ms | 300ms |
| Provider adapter (per provider per mode) | 99.9% with failover | (provider-bound) | (provider-bound) |
| RAG endpoint | 99.9% | 200ms | 500ms |
| Eval harness | 99% | (batch) | (batch) |
| Sandbox spawn | 99.9% | 500ms cold; 100ms warm | 2s cold |
| Audit-chain emission | 100% emission completeness | 50ms | 200ms |

### 2.5 Cloud
| Surface | Availability | p95 | p99 |
|---|---|---|---|
| IAM control plane | 99.99% | 50ms | 200ms |
| Cell-routed compute control | 99.95% | 200ms | 1s |
| Object storage GET | 99.9% | 100ms TTFB | 300ms |
| Object storage PUT | 99.9% | 200ms | 1s |
| Block storage IOPS | 99.95% | (per-tier) | (per-tier) |
| KMS encrypt / decrypt | 99.99% | 10ms | 50ms |
| VPC control | 99.99% | 100ms | 500ms |
| Load balancer | 99.99% | 5ms added | 20ms added |
| DNS | 99.99% | 20ms | 80ms |
| Billing event ingest | 99.95% | 100ms | 300ms |

### 2.6 Search
| Surface | Availability | p95 | p99 |
|---|---|---|---|
| Query / SERP | 99.95% | 200ms | 500ms |
| Per-tenant index query | 99.9% | 100ms | 300ms |
| Crawl ingest rate | (per host) | (per host) | (per host) |
| Index update propagation | 99.9% | 5min | 15min |
| Vector ANN query | 99.9% | 50ms | 200ms |

### 2.7 Ads + Analytics
| Surface | Availability | p95 | p99 |
|---|---|---|---|
| Ad-request → render | 99.95% | 50ms | 100ms |
| Auction bid evaluation | 99.95% | 30ms | 80ms |
| Impression / click record | 99.95% | 50ms | 200ms |
| Attribution batch | 99.5% (batch) | (batch) | (batch) |
| Analytics dashboard query | 99.9% | 1s | 5s |

## 3. Error-budget policy

- **Budget = (1 − SLO) × window** (30d default)
- **Burn-rate alerts** (Google SRE 4-window method):
  - 1h window > 14.4× ⇒ page (consumes 2% of budget in 1h)
  - 6h window > 6× ⇒ page (consumes 5% in 6h)
  - 24h window > 3× ⇒ ticket (consumes 10% in 24h)
  - 3d window > 1× ⇒ informational (consumes 10% in 3d)
- **Budget exhaustion → release freeze.** Per surface, when 30d budget = 0, releases for that surface require explicit waiver from `ops-sre-reliability` + `council-architecture`.
- **Budget reset** at the start of each 30d window.
- **Multi-window alerting** prevents alert fatigue (require both short + long window to fire).

## 4. Per-surface SLO row source of truth

Each surface has a `slo:` field in its catalog record (`registry/catalog/<crate>.yaml`). This doc is generated from the catalog. CI lane `pipeline-slo-coverage` checks every public surface has a row.

## 5. Sources scanned
Google SRE workbook (4-window burn rate); per-product PRD §9 metrics; ADR-0050 (Argo Rollouts); ADR-0045 (VictoriaMetrics + Grafana); CLAUDE.md.
