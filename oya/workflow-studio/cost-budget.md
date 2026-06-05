---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-workflow + ops-sre-reliability
deciders: ops-finops, axis-workflow, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-studio/capacity-model.md
  - microservices/workflow-studio/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (workflow-studio µservice)

## Purpose

Track the workflow-studio µservice's monthly cloud cost across infrastructure (CDN + WAF + compute + storage + LLM-assist) per Layer-A + Layer-B component per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers called out.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| CDN (egress + edge cache) | WASM bundles, node library descriptors, design-system assets | `oracle.com/cloud/cdn/pricing/` |
| WAF | Ingress in front of CDN + editor REST | `oracle.com/cloud/security/waf/pricing/` |
| Compute (VM.Standard / OKE node) | Editor REST + WebSocket gateway + node-library-registry-rest + composition-root pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres editor session store; Valkey AOF | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage | Signed per-pack node library binaries | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | WebSocket traffic + cross-region replication | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack signing keys (node library + audit chain) | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ingress + WebSocket pool | `oracle.com/cloud/networking/load-balancing/pricing/` |
| LLM-assist pass-through | foundry-providers LLM invocation cost (varies per provider; passed through to tenant) | per-provider rate card |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M03 preview launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M03 launch; 20 tenants pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| CDN (per-pack edge cache) | – | – | $30 egress + $20 edge cache | $50 |
| WAF | – | $40 (base + per-request) | – | $40 |
| Postgres + Citus coordinator (editor sessions) | 2 × VM.Standard.E4 4-core | $290 | $100 PV | $390 |
| Postgres workers (Citus shards) | 4 × VM.Standard.E4 4-core | $580 | $400 PV | $980 |
| Postgres read-replica | 4 × VM.Standard.E4 2-core | $290 | $200 PV | $490 |
| Valkey Sentinel HA (ephemeral CRDT) | 3 × VM.Standard.E4 2-core | $108 | $20 PV (AOF) | $128 |
| `visual-canvas-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `collab-crdt-worker` (WebSocket gateway) | 3 × VM.Standard.E4 4-core | $217 | – | $217 |
| `node-library-registry-rest` | 2 × VM.Standard.E4 1-core | $36 | – | $36 |
| `node-library-registry-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `visual-canvas-app` (composition root) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Object storage (node library binaries) | – | – | $30 hot (1 TB) | $30 |
| KMS keyring | – | $5 | – | $5 |
| Load balancer | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$1800** | **~$770** | **~$2600 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm at deploy time. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## LLM-Assist Cost (Pass-Through)

LLM-assist cost varies per provider and per usage. Tracked separately because it is pass-through:

| Tier | LLM-assist requests/month | Avg cost per request | Estimated monthly cost |
|---|---|---|---|
| XS (M03 preview; 20 tenants) | 1,000 | $0.02 | $20 |
| S (~100 tenants) | 50,000 | $0.02 | $1000 |
| M (~1000 tenants) | 1,000,000 | $0.02 | $20,000 |
| L (~10000 tenants) | 50,000,000 | $0.02 | $1,000,000 |

LLM-assist cost is per-tenant billed pass-through; tenant chooses provider; oyatie does not pay margin on LLM cost.

## Per-Scale-Tier Forecast (Studio Infrastructure)

| Scale tier | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M03 preview; 20 tenants) | 20 | ~$2600 | active: pack-kr |
| S (~100 tenants) | 100 | ~$10k | active: pack-kr + pack-eu + pack-us |
| M (~1000 tenants) | 1000 | ~$50k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$500k | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention 6y).
- **KR-FSS-regulated** tenants in pack-kr: 1.2× base (retention 5y per KR commercial code).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base.

## Per-Seat Unit-Economics Target

Per `/specs/microservices/workflow-studio.json` §goals.efficiency: ≤ $5 per-seat per-month at GA.

| Tier | N_seats | Studio infra cost per seat-month | LLM-assist cost per seat-month | Total per seat-month |
|---|---|---|---|---|
| XS (M03 preview) | 200 (10 seats avg/tenant) | $13.00 | $0.10 | $13.10 |
| S | 2,000 | $5.00 | $0.50 | $5.50 |
| M | 20,000 | $2.50 | $1.00 | $3.50 |
| L | 200,000 | $2.50 | $5.00 | $7.50 |

XS tier is uneconomic on per-seat basis (HA minimums dominate); GA target ≤ $5/seat met at S+ tier. Reconciles with `/specs/microservices/workflow-studio.json` §metrics target.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review |
| cost > 130% | red alert; budget breach incident | engage ops-finops |
| Per-tenant cost projection (highest spender) | within 5× median | normal |
| Per-tenant LLM-assist cost > 10× median | yellow; engage tenant on LLM usage | tenant-facing dashboard surfaces self-overage |
| Per-seat cost > $7.50 (GA target $5) | yellow if XS/S tier; red if M/L tier | FinOps review |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Studio infra cost / N_seats (unit-economic) | within 5% of forecast | 6× burn over 6h |
| LLM-assist cost / N_invocations | within forecast | 14.4× burn over 1h |
| CDN egress / N_sessions | within 5% | informational |
| Spot-vs-on-demand ratio | ≥ 60% spot for non-WS workloads | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| CDN cache TTL extension (1h → 24h for static assets) | 10% CDN cost | Slower invalidation on release |
| Spot-instance fleet for visual-canvas-rest (stateless) | 30-50% compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20-40% compute | Vendor lock-in window |
| LLM-assist zero-retention provider mandate | 0% (cost-neutral; risk reduction) | Tenant choice constraint |
| Per-tenant editor-session budget enforcement | 10-20% compute | Tenant disruption if too aggressive |
| Valkey memory cap per tenant | 5% Valkey cost | Tenant disruption on overage |
| Object-storage lifecycle: node library binaries → archive after 30d | 10% object storage | Slower library hot-reload after archive |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=cost-budget --microservice workflow-studio` — exit 0; current spend within 110%.
- Monthly FinOps review.
- Quarterly: capacity-model + cost-budget refresh.

## References

- `microservices/workflow-studio/capacity-model.md`.
- `microservices/workflow-studio/multi-region.md`.
- `microservices/workflow-studio/policy/data-residency.md`.
- `/specs/microservices/workflow-studio.json` §goals.efficiency.
- OCI pricing — `oracle.com/cloud/pricing/`.
- FinOps Foundation framework — `finops.org`.
