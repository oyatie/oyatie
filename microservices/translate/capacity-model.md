---
doc_class: CapacityModel
title: Capacity model + sizing
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-translate + ops-iac + ops-sre-reliability
related_adrs: [ADR-0117, ADR-0131, ADR-TRANSLATE-0001, ADR-TRANSLATE-0006]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/multi-region.md
  - microservices/translate/failure-modes.md
  - microservices/translate/cost-budget.md
review_cadence: quarterly + on every pack activation + on every vendor adapter add
doc_status: published
---

# Capacity Model — translate µservice

## Reference Workload Mix (per pack, steady-state)

Per `PRD.md` §"Tenant Value" and competitor benchmarks (Crowdin / Phrase / Smartling growth profile):

| Workload | Per-second rate (mean) | Per-second rate (peak; 95th-percentile hour) | Notes |
|---|---|---|---|
| Real-time translation request (≤ 500 chars) | 80 rps | 400 rps | Driven by mail / messenger / docs / sheets / slides / social / shorts in-editor translate |
| Batch translate (100 segments) | 4 rps | 20 rps | Driven by workflow-studio orchestration |
| Language detection | 100 rps | 500 rps | Driven by all in-editor surfaces + bulk-translate pre-flight |
| TM leverage query | 250 rps | 1 200 rps | Co-resident with translate request |
| QE score | 50 rps | 250 rps | Driven by ADR-TRANSLATE-0003 — sampled, not every call |
| Real-time caption stream chunks | 200 chunks/s | 1 000 chunks/s | Driven by `meet` (10 active meetings × 100 chunks/s) |
| Document translate (10-page DOCX) | 0.5/s | 3/s | Driven by docs/sheets/slides export |
| Bulk-translate XLIFF (10 k seg) | 0.1/s | 0.5/s | Driven by enterprise localization workflows |

## Per-Component Sizing (per pack, M01 baseline)

### translate-router (REST + worker + app)

| Component | Replicas | CPU req / lim | Memory req / lim | Notes |
|---|---|---|---|---|
| `translate-router-rest` | 4 | 1000m / 2000m | 2 Gi / 4 Gi | Stateless; HPA at 70 % CPU |
| `translate-router-worker` | 2 | 500m / 1500m | 1 Gi / 3 Gi | Engine-health monitor + cost roll-up |
| `translate-router-app` | 2 | 500m / 1500m | 1 Gi / 3 Gi | Composition root |

Decision latency budget: ≤ 5 ms p99 in-process. Drives router to be CPU-bound, not memory.

### TM stack

| Component | Replicas | CPU | Memory | Notes |
|---|---|---|---|---|
| `tm-rest` | 3 | 500m / 1000m | 1 Gi / 2 Gi | RLS via Postgres |
| `tm-worker` | 2 | 500m / 1500m | 2 Gi / 4 Gi | Minhash-LSH index maintenance + Meilisearch sync |
| Postgres TM (primary + 1 replica) | 1+1 | 4000m / 8000m | 16 Gi / 32 Gi | HA per pack; `BEHAVIORAL_TENANT_PRODUCT` + `PII_QUASI_IDENTIFIER` |
| Meilisearch | 2 | 2000m / 4000m | 8 Gi / 16 Gi | per pack; per-tenant index |

### Termbase stack

| Component | Replicas | CPU | Memory | Notes |
|---|---|---|---|---|
| `termbase-rest` | 2 | 500m / 1000m | 1 Gi / 2 Gi | |
| `termbase-worker` | 1 | 250m / 500m | 512 Mi / 1 Gi | TBX import/export |
| Postgres (shared with TM) | — | — | — | shared cluster; separate schema |

### QE + LangDetect

| Component | Replicas | CPU | Memory | Notes |
|---|---|---|---|---|
| `qe-rest` | 2 | 500m / 1000m | 1 Gi / 2 Gi | |
| `langdetect-rest` | 2 | 500m / 1000m | 512 Mi / 1 Gi | |
| (Models served on foundry-runtime; sized separately) | — | — | — | |

### Document translation

| Component | Replicas | CPU | Memory | Notes |
|---|---|---|---|---|
| `doc-translate-worker` (gVisor sandbox) | 4 | 1000m / 2000m | 2 Gi / 4 Gi | gVisor overhead ~10 % per `gvisor.dev/docs/architecture_guide/performance/` |
| `pandoc` sidecar | (in-pod) | 500m / 1000m | 512 Mi / 1 Gi | per worker |
| `libreoffice` sidecar | (in-pod) | 1000m / 2000m | 1 Gi / 2 Gi | per worker |

### Bulk translate

| Component | Replicas | CPU | Memory | Notes |
|---|---|---|---|---|
| `bulk-translate-worker` | 4 | 1000m / 2000m | 2 Gi / 4 Gi | Fan-out per-chunk |
| Redis | 1 (HA via sentinel) | 1000m / 2000m | 4 Gi / 8 Gi | job state + per-tenant token bucket |

### Real-time stream

| Component | Replicas | CPU | Memory | Notes |
|---|---|---|---|---|
| `stream-router` | 4 | 1000m / 2000m | 2 Gi / 4 Gi | WS termination + chunk dispatch |
| Redis (shared) | — | — | — | session state |

### Engine adapters

| Adapter | Replicas | CPU | Memory | Notes |
|---|---|---|---|---|
| `adapter-foundry-runtime` | 4 | 500m / 1500m | 1 Gi / 3 Gi | In-house path |
| `adapter-anthropic` | 2 | 500m / 1500m | 1 Gi / 3 Gi | Via foundry-providers |
| `adapter-openai` | 2 | 500m / 1500m | 1 Gi / 3 Gi | Via foundry-providers |
| `adapter-google-translate` | 2 | 500m / 1500m | 1 Gi / 3 Gi | Via foundry-providers |
| `adapter-deepl` | 2 | 500m / 1500m | 1 Gi / 3 Gi | Via foundry-providers |

## Aggregate per-pack resource budget (M01 baseline)

| Resource | Total request | Total limit |
|---|---|---|
| CPU | 28 000m (~ 28 cores) | 56 000m (~ 56 cores) |
| Memory | 70 Gi | 140 Gi |
| Storage (Postgres + Meilisearch + Redis + S3) | 500 Gi base + tenant growth | per-tenant |

Per-pack node-pool sizing: ≥ 4 worker nodes; each ≥ 16 vCPU + 64 GiB RAM; anti-affinity ensures every component spreads across nodes + AZs.

## Scaling Policies

### HPA

- All stateless components: HPA at 70 % CPU; min replicas as above; max = 4× min.
- `doc-translate-worker`: HPA at 50 % CPU (gVisor overhead + variable workload).
- `bulk-translate-worker`: HPA at queue depth > 100 jobs.
- `stream-router`: HPA at active sessions > 500 per replica.

### VPA (off by default for production)

- Only on `translate-router-worker` + `bulk-translate-worker` where workload variance is high; VPA off in production to avoid pod restarts.

### PDB (PodDisruptionBudget)

- `minAvailable: 50 %` for every stateless component.
- `maxUnavailable: 1` for Postgres + Meilisearch (stateful).

## Growth Forecasts

| Quarter | Tenant base estimate | rps multiplier | Action |
|---|---|---|---|
| M01 (Q3 2026) | 5 anchor tenants | 1× | Baseline as above |
| M01+90d | 20 tenants | 2× | Scale router-rest to 8 replicas |
| M01+180d | 50 tenants | 5× | Scale Postgres to 8000m / 32 Gi; consider read-replica fan-out |
| M01+365d | 150 tenants | 10× | Re-evaluate per-pack node-pool count; potential shard split |

## TM Storage Growth

Reference workload: each tenant accumulates ~ 10 k–1 M TM units depending on project scope.

| Metric | Estimate |
|---|---|
| Avg TM unit size (source + target + metadata) | ~ 2 KB |
| 100-tenant pack at 100 k TM units each | 20 GB |
| 1000-tenant pack at 200 k TM units each | 400 GB |

Postgres `tm_units` table + Meilisearch index size scale proportionally; partitioning by tenant-id hash + per-tenant index isolation per ADR-TRANSLATE-0002.

## Throughput Bottlenecks (anticipated)

1. **TM leverage Meilisearch index** — per-tenant index grows; mitigated via per-tenant index isolation; if a single tenant exceeds 5 M units, shard their index.
2. **Engine vendor rate-limits** — per-tenant + per-engine token bucket (Redis); router demote when bucket exhausted.
3. **Document translation throughput** — gVisor sandbox start-up ~ 200 ms; pre-warmed sandbox pool of 8 per worker pod mitigates.
4. **Real-time stream concurrency** — per-replica concurrent-session ceiling ~ 500; HPA at 500 sessions.
5. **Bulk-translate fan-out concurrency** — per-job concurrency cap of 16; per-tenant total fan-out budget of 64; prevents single tenant exhausting vendor quota.

## Cost-of-capacity model

See `cost-budget.md` for $-per-MAU and $-per-translation reference figures.

## Verification

- `tests/load/` directory contains per-component k6/wrk load drivers replaying the reference workload mix.
- `cargo run -p oya-dev-cli -- gate validate capacity --microservice translate` (when implemented) validates Helm requests/limits against this model.

## References

- ADR-0117 — pack residency model + per-pack sizing.
- ADR-TRANSLATE-0001 — engine routing.
- ADR-TRANSLATE-0006 — real-time stream.
- Crowdin / Phrase / Smartling published architecture references (growth profiles).
- gVisor performance docs.
- AWS Well-Architected Cost + Performance pillars.
- Google SRE Workbook ch. 13 (Capacity Planning).
