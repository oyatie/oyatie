---
doc_class: MULTI-REGION
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: ops-sre-reliability + axis-foundry
related_adrs: [ADR-0117, ADR-0136, ADR-0137]
---

# Multi-Region Plan — foundry (consolidated)

## Scope

Cross-BC, cross-pack regional topology for foundry. Per-BC multi-region
docs preserved at `bc-sources/<bc>/multi-region.md`.

## M01 Launch (2026-Q3)

- **Single pack: pack-kr** on OCI ap-seoul-1.
- All 6 BCs deployed in single Kubernetes cluster.
- HA: per-BC Helm subchart declares `minReplicas: 3` for stateless tiers;
  6-shard Redis cluster (runtime session-state); 3-replica Postgres
  (per-BC); 3-replica ClickHouse (eval); 3-region S3 (evidence blob).

## Post-M01 expansion sequence

| Wave | Pack | Trigger | Notes |
|---|---|---|---|
| 1 | pack-eu (Frankfurt) | first EU tenant signed; GDPR + EU AI Act conformity | per-pack overlay applied across 6 BCs |
| 2 | pack-us (us-east + us-west) | first US tenant signed | CCPA overlay |
| 3 | pack-us-healthcare | first BAA signed | HIPAA overlay; 6y retention |
| 4 | pack-jp (Tokyo) | first JP tenant; APPI overlay | |
| 5 | pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | per-tenant trigger | per-jurisdiction overlay |

## Cross-pack invariants

- **Cross-pack data flow is forbidden by default.** Per ADR-0117 + per-pack
  Cedar fragments. Per-BC adapters refuse cross-pack reads/writes; CI lane
  `per-pack-residency` blocks misconfiguration at PR-time.
- **Per-pack independence**: each pack ships a complete foundry stack;
  outage in one pack does not degrade another.
- **Per-pack KMS keys**: never shared across packs; rotation per pack;
  audit-chain seal binds pack identity.

## Per-BC multi-region notes

| BC | Pack-local state | Cross-pack flow | Latency sensitivity |
|---|---|---|---|
| runtime | session-state Redis + capability-cache Postgres | none | dispatch p99 ≤50ms — co-locate with caller |
| supervisor | fleet-state Postgres | none | command propagation ≤5s — same-pack |
| eval | golden-store S3 + ClickHouse | optional cross-pack read of golden store (signed, content-addressed) | scheduling ≤500ms |
| evidence | pack-builder Postgres + blob S3 | regulator-export across packs (signed envelope only) | pack assembly ≤2s/100MB |
| guardrails | rule-store Postgres + ONNX serving | none | inline ≤20ms |
| providers | router Postgres + Redis rate-limit + OpenBao | provider call goes to provider-side endpoint (provider's own region) | router ≤5ms |

## DR / failover

- **RPO**: ≤30s per BC (sync-replicated state in adapters).
- **RTO**: ≤5 min per BC (HA failover; Kubernetes deployment-controller).
- **Pack-level outage**: declare in incident-response.md; tenant SLA carve-
  out per `cost-budget.md` and tenant contract; cross-pack failover is
  manual + tenant-approved (per ADR-0117 cross-pack-forbidden).

## Per-BC multi-region archives

- `bc-sources/runtime/multi-region.md`
- `bc-sources/supervisor/multi-region.md`
- `bc-sources/eval/multi-region.md`
- `bc-sources/evidence/multi-region.md`
- `bc-sources/guardrails/multi-region.md`
- `bc-sources/providers/multi-region.md`

## References

- ADR-0117: Data-residency + jurisdiction codes.
- ADR-0136 / ADR-0137: foundry topology.
