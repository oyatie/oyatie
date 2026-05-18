---
doc_class: CAPACITY-MODEL
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: ops-sre-reliability + axis-foundry
related_adrs: [ADR-0136, ADR-0137]
---

# Capacity Model — foundry (consolidated)

## Scope

Cross-BC capacity envelope for foundry. Per-BC capacity models preserved at
`bc-sources/<bc>/capacity-model.md`.

## M01 launch envelope (pack-kr, XS tier, single cell)

| Dimension | Owner BC | Baseline | Max per cell | Scale-out trigger |
|---|---|---|---|---|
| Concurrent invocations | runtime | 5,000 | 50,000 | runtime-pool queue depth > 200/pod |
| Active sessions | runtime | 50,000 | 500,000 | Redis memory > 70% |
| Capabilities mirrored | runtime + supervisor | 10,000 | 100,000 | Postgres mirror > 1 GB |
| Dispatch throughput | runtime | 1,000 rps | 10,000 rps | executor CPU > 70% |
| Session-state ops/sec | runtime | 10,000 ops/s | 100,000 ops/s | Redis ops > 70% of cluster ceiling |
| Supervision commands/sec | supervisor | 100 cmd/s | 1,000 cmd/s | event-bus lag > 5s |
| Active fleets | supervisor | 1,000 | 10,000 | k8s operator queue > 100 |
| Eval-runs in flight | eval | 50 | 500 | GPU pool > 80% utilisation |
| Eval golden-output entries | eval | 100,000 | 1,000,000 | ClickHouse storage > 70% |
| Evidence packs per day | evidence | 100 | 5,000 | S3 PUT rate > 70% account ceiling |
| Evidence blob storage | evidence | 1 TB | 100 TB | S3 lifecycle policy archives ≥30d |
| Guardrail inline checks/sec | guardrails | 2,000/s | 20,000/s | classifier-serving CPU > 70% or queue > 100 |
| Cedar policy evaluations/sec | guardrails | 5,000/s | 50,000/s | Cedar engine CPU > 70% |
| Provider routing decisions/sec | providers | 1,000/s | 10,000/s | router CPU > 70% |
| Provider concurrent calls | providers | 500 | 5,000 | per-provider rate-limit hit |

## Scale-out policy

- **Kubernetes HPA**: every BC's stateless-tier deployment scales on CPU
  > 70%; min 3 replicas (HA quorum); max varies per BC (200 for runtime
  executor; 50 for supervisor; 32 for eval-runner; 50 for evidence-builder;
  100 for guardrails inline; 50 for providers router).
- **Redis cluster** (runtime + providers rate-limit): 6-shard primary +
  replica per pack; scale shards on memory > 70%.
- **Postgres** (supervisor fleet-state + runtime registry + guardrails
  rules + evidence pack-index + providers router config): read-replica
  fanout up to 8/pack; primary vertical scale until `db.standard.E4.16`.
- **ClickHouse** (eval parity store): 3-replica per pack; horizontal scale
  by shard.
- **S3** (eval golden store + evidence blob): scale-by-design; per-tenant
  bucket prefix + lifecycle policies.
- **GPU pool** (eval): 0–16 A100 slots per pack; HPA on queue depth.

## Pre-warming policies

- Runtime pod warm pool: per `bc-sources/runtime/capacity-model.md`; cold-
  start ≤500ms.
- Classifier model serving (guardrails): pre-load all active rulesets per
  pack on startup; hot-reload on `GuardrailRulesetUpdated`.
- Capability registry cache (runtime): warm on startup from supervisor's
  pull endpoint; refresh on `CapabilityRegistryUpdated` events.

## Cross-BC backpressure

- Provider rate-limit (providers BC) → runtime executor receives 429 →
  invocation parks for retry per backoff schedule → emits
  `InvocationParked` to evidence.
- Guardrail classifier overloaded (guardrails BC) → runtime fast-fails
  invocation per circuit-breaker → emits `InvocationFailed{reason=guardrail_overload}`.
- Evidence pack-builder backlog (evidence BC) → runtime invocations
  continue; recorder writes to local buffer + background drain.

## Per-BC capacity archives

- `bc-sources/runtime/capacity-model.md`
- `bc-sources/supervisor/capacity-model.md`
- `bc-sources/eval/capacity-model.md`
- `bc-sources/evidence/capacity-model.md`
- `bc-sources/guardrails/capacity-model.md`
- `bc-sources/providers/capacity-model.md`

## References

- ADR-0136 / ADR-0137: foundry topology.
- `microservices/foundry/cost-budget.md` — costs behind this envelope.
