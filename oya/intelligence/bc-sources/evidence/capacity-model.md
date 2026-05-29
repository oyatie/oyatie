---
doc_class: CapacityModel
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-evidence
related_artifacts:
  - microservices/intelligence-evidence/PRD.md
  - microservices/intelligence-evidence/cost-budget.md
  - microservices/audit-chain/capacity-model.md  (substrate; references load-shape)
doc_status: published
---

# foundry-evidence — capacity model

## Forecast (M01 → M03)

| Phase | Sustained record_invocation rate | Peak record_invocation rate | Active packs in hot tier |
|---|---|---|---|
| M01 launch | 5 k inv/s aggregate | 20 k inv/s aggregate (peak ×4) | 500 M packs (90d) |
| M02 6 mo | 15 k inv/s aggregate | 60 k inv/s aggregate | 1.5 B packs |
| M03 12 mo | 50 k inv/s aggregate | 200 k inv/s aggregate | 5 B packs |

Per-pack ingest sharding: each pack carries its own `audit-chain` substrate chain; cross-pack sharing is forbidden per ADR-0117 + `policy/data-residency.md`.

## Bottleneck analysis

| Component | Saturation point | Mitigation |
|---|---|---|
| record_invocation REST | ~ 5 k inv/s per pod | horizontal autoscale; HPA on per-pod RPS |
| Pack-builder worker | ~ 2 k pack/s per pod | horizontal autoscale; HPA on queue depth |
| Postgres primary INSERT | ~ 15 k inserts/s per primary | per-tenant_partition shard split when sustained > 70 % |
| Postgres replica read | ~ 50 k reads/s per replica | add replicas; query-router load-balance |
| audit-chain bridge | ~ 10 k emits/s per pod | horizontal autoscale; bounded by substrate-side seal throughput |
| Substrate sealing | ~ 50 k events/s per pack (Bominal ADR-0028) | substrate-owned; foundry-evidence respects substrate back-pressure |
| Workflow event bus | ~ 100 k events/s | shared substrate; foundry-evidence ~10 % share at M03 |

## Storage sizing

| Tier | M01 | M02 | M03 |
|---|---|---|---|
| Postgres evidence_pack (hot, 90d) | 500 GB / pack-major | 1.5 TB / pack-major | 5 TB / pack-major |
| Postgres evidence_pack_warm (1y) | 1 TB / pack-major | 3 TB / pack-major | 10 TB / pack-major |
| Postgres evidence_pack_cold_index (multi-year) | 200 GB / pack-major | 600 GB / pack-major | 2 TB / pack-major |
| WORM blob hot (substrate) | 5 TB / pack-major | 15 TB / pack-major | 50 TB / pack-major |
| WORM blob warm | 10 TB / pack-major | 30 TB / pack-major | 100 TB / pack-major |
| WORM blob cold | 20 TB / pack-major | 60 TB / pack-major | 200 TB / pack-major |

"pack-major" = pack-us, pack-eu, pack-kr individually; smaller packs scale by 1/3 to 1/10 of these numbers.

## Headroom contract

- 18-month forward headroom on every tier per ADR-0117.
- Capacity alert at < 180 days projected (Sev-3); < 90 days (Sev-2).
- Per-tenant cap: 10 k inv/s sustained per tenant; raise via tenancy DPA negotiation + capacity review.

## Burst behaviour

- record_invocation peak ×4 over sustained baseline tolerated for 1h with full SLO.
- > ×4 → 429 with `Retry-After`; tenant SDK auto-backs-off; foundry-runtime worker queue absorbs.
- Pack-builder back-pressures by slowing dead-letter drain; the receipt path stays at full SLO.

## Per-tenant + per-source rate limits

| Limit | Target | Source |
|---|---|---|
| per-tenant record_invocation | 10 k inv/s sustained; 40 k peak | tenancy DPA baseline |
| per-source-µservice global | 100 k inv/s | foundry-* operations bound |
| per-tenant evidence_query | 1 k qps | tenant DPA |
| per-engagement regulator-export | 10 concurrent bundles | council-privacy operational |

## Sharding strategy

- Per-pack physical isolation (data residency).
- Within a pack, per-tenant_partition Postgres sharding (default 16 shards per pack).
- Shard count doubles when sustained per-shard CPU > 70 % for 1h.
- Per-shard Postgres + WORM ratios stay invariant during reshards (substrate-coordinated).

## Test cadence

| Drill | Cadence | What it verifies |
|---|---|---|
| Load drill at 1.5× peak target | weekly | record_invocation + pack-builder + bridge stay in SLO |
| Chaos drill: substrate down 5 min | monthly | dead-letter drains within 10 min after substrate recovery; no pack loss |
| Capacity headroom dry-run | quarterly | 18-mo forward headroom holds at current growth |
| Per-tenant cap drill | monthly | 429 + Retry-After actually shed load; no cascade failure |

## ADR-0133 honest gap

Capacity targets are CI-asserted via the load-drill lane. If a drill fails the target, the capacity-model is rerun + a new contract negotiated before the next milestone. No aspirational numbers.

## References

- `microservices/audit-chain/capacity-model.md` (substrate-side load-shape).
- ADR-0117 (cloud-native infra capacity policy).
- `docs/standards/observability-slo.md`.
