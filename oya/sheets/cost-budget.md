---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: sheets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-sheets + ops-sre-reliability
deciders: ops-finops, axis-sheets, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/sheets/capacity-model.md
  - microservices/sheets/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (sheets µservice)

## Purpose

Track the sheets µservice's monthly cloud cost across infrastructure (CDN + WAF + compute + storage + Arrow/Parquet object storage + AI-formula) per Layer-A + Layer-B component per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| CDN (egress + edge cache) | WASM bundles, design-system assets, spec schema | `oracle.com/cloud/cdn/pricing/` |
| WAF | Ingress in front of CDN + editor REST | `oracle.com/cloud/security/waf/pricing/` |
| Compute (VM.Standard / OKE node) | Editor REST + WebSocket gateway + recalc worker + XLSX export worker + license-gate-cedar + composition-root pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres editor + cell store; Valkey AOF | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage | Arrow/Parquet large-sheet blocks + workbook snapshots + version-history binaries + XLSX export jobs + XLSX upload quarantine | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | WebSocket traffic + XLSX export download + cross-region replication | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack signing keys (audit chain + KMS-SSE for storage) | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ingress + WebSocket pool | `oracle.com/cloud/networking/load-balancing/pricing/` |
| AI-formula pass-through | foundry-runtime LLM invocation cost (varies per provider; passed through to tenant) | per-provider rate card |
| ClamAV + OPSWAT MetaDefender | AV scan sidecars for XLSX uploads | vendor rate card |
| gVisor user-mode sandbox | overhead on XLSX import/export worker compute (~15% CPU tax) | included in compute |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M03 preview launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M03 launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| CDN (per-pack edge cache) | – | – | $30 egress + $20 edge cache | $50 |
| WAF | – | $40 (base + per-request) | – | $40 |
| Postgres + Citus coordinator | 2 × VM.Standard.E4 4-core | $290 | $100 PV | $390 |
| Postgres workers (Citus shards) | 4 × VM.Standard.E4 4-core | $580 | $400 PV | $980 |
| Postgres read-replica | 4 × VM.Standard.E4 2-core | $290 | $200 PV | $490 |
| Valkey Sentinel HA (ephemeral CRDT) | 3 × VM.Standard.E4 2-core | $108 | $20 PV (AOF) | $128 |
| `cell-grid-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `collab-crdt-worker` (WebSocket gateway) | 3 × VM.Standard.E4 4-core | $217 | – | $217 |
| `recalc-engine-worker` | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| `xlsx-export-worker` (gVisor sandboxed) | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| `license-gate-cedar` | 2 × VM.Standard.E4 1-core | $36 | – | $36 |
| `cell-grid-app` (composition root) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Object storage (Arrow/Parquet + snapshots + version-history + XLSX) | – | – | $50 hot (2 TB) | $50 |
| KMS keyring | – | $5 | – | $5 |
| Load balancer | – | $20 | – | $20 |
| ClamAV + OPSWAT sidecars | 2 × VM.Standard.E4 1-core | $40 | – | $40 |
| **XS tier total per pack region** | | **~$2060** | **~$820** | **~$2880 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm at deploy time. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## AI-Formula Cost (Pass-Through)

| Tier | AI-formula requests/month | Avg cost per request | Estimated monthly cost |
|---|---|---|---|
| XS (M03 preview; 20 tenants) | 1,000 | $0.02 | $20 |
| S (~100 tenants) | 50,000 | $0.02 | $1000 |
| M (~1000 tenants) | 1,000,000 | $0.02 | $20,000 |
| L (~10000 tenants) | 50,000,000 | $0.02 | $1,000,000 |

AI-formula cost is per-tenant billed pass-through.

## Per-Scale-Tier Forecast (Sheets Infrastructure)

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M03 preview; 20 tenants) | 20 | ~$2880 | active: pack-kr |
| S (~100 tenants) | 100 | ~$11k | active: pack-kr + pack-eu + pack-us |
| M (~1000 tenants) | 1000 | ~$55k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$550k | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs**: 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention 6y; HIPAA-eligible compute slight premium).
- **KR-FSS-regulated** tenants in pack-kr: 1.2× base (retention 5y per KR commercial code).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base.

## Per-Seat Unit-Economics Target

Per `/specs/microservices/sheets.json` §goals.efficiency: ≤ $5 per-seat per-month at GA.

| Tier | N_seats | Sheets infra cost per seat-month | AI-formula cost per seat-month | Total per seat-month |
|---|---|---|---|---|
| XS (M03 preview) | 200 (10 seats avg/tenant) | $14.40 | $0.10 | $14.50 |
| S | 2,000 | $5.50 | $0.50 | $6.00 |
| M | 20,000 | $2.75 | $1.00 | $3.75 |
| L | 200,000 | $2.75 | $5.00 | $7.75 |

XS tier is uneconomic on per-seat basis (HA minimums dominate); GA target ≤ $5/seat met at S+ tier.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review |
| cost > 130% | red alert; budget breach incident | engage ops-finops |
| Per-tenant cost projection (highest spender) | within 5× median | normal |
| Per-tenant AI-formula cost > 10× median | yellow; engage tenant on AI usage | tenant-facing dashboard surfaces self-overage |
| Per-tenant XLSX export cost > 10× median | yellow; check for runaway export jobs | tenant-facing dashboard |
| Per-tenant connected-sheets cost > 10× median | yellow; check for runaway external-source queries | tenant-facing dashboard |
| Per-seat cost > $7.50 (GA target $5) | yellow if XS/S tier; red if M/L tier | FinOps review |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Sheets infra cost / N_seats | within 5% of forecast | 6× burn over 6h |
| AI-formula cost / N_invocations | within forecast | 14.4× burn over 1h |
| CDN egress / N_sessions | within 5% | informational |
| Spot-vs-on-demand ratio | ≥ 60% spot for non-WS workloads | informational |
| Object-storage growth rate (Arrow/Parquet) | within 10% of forecast | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| CDN cache TTL extension (1h → 24h for static assets) | 10% CDN cost | Slower invalidation on release |
| Spot-instance fleet for cell-grid-rest (stateless) | 30-50% compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20-40% compute | Vendor lock-in window |
| AI-formula zero-retention provider mandate | 0% (cost-neutral; risk reduction) | Tenant choice constraint |
| Per-tenant editor-session budget enforcement | 10-20% compute | Tenant disruption if too aggressive |
| Valkey memory cap per tenant | 5% Valkey cost | Tenant disruption on overage |
| Object-storage lifecycle: workbook snapshots → archive after 30d | 15% object storage | Slower restore for older versions |
| Arrow/Parquet hot↔cold tiering (>30d cold) | 15% large-sheet storage | Slower analytical recalc on cold blocks |
| XLSX upload quarantine retention 7d → 3d | 5% object storage | Less forensic window |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=cost-budget --microservice sheets` — exit 0; current spend within 110%.
- Monthly FinOps review.
- Quarterly: capacity-model + cost-budget refresh.

## References

- `microservices/sheets/capacity-model.md`.
- `microservices/sheets/multi-region.md`.
- `microservices/sheets/policy/data-residency.md`.
- `/specs/microservices/sheets.json` §goals.efficiency.
- OCI pricing — `oracle.com/cloud/pricing/`.
- FinOps Foundation framework — `finops.org`.
- ClamAV pricing — `clamav.net`.
- OPSWAT MetaDefender pricing — `opswat.com/products/metadefender`.
