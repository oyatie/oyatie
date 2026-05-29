---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry
deciders: ops-sre-reliability, axis-foundry, council-architecture
related_adrs: [ADR-0025, ADR-0026, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence-providers/cost-budget.md
  - microservices/intelligence-providers/multi-region.md
  - microservices/intelligence-providers/policy/data-residency.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (foundry-providers µservice)

## Purpose

Sizing formulas + baseline numbers for every Layer-A component (Postgres / Valkey / OpenBao agent) and Layer-B component (`oya-foundry-providers-router-*`, `oya-foundry-providers-adapter-*`). Drives `cost-budget.md` and `multi-region.md`.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants | `N_tenants` | tenant-resolver |
| Calls per tenant per second (peak) | `R_calls_peak` | tenant SLA + capability_profile |
| Calls per tenant per day (average) | `R_calls_avg` | tenant SLA |
| Average prompt size (tokens) | `T_prompt_avg` | tenant workload |
| Average completion size (tokens) | `T_completion_avg` | tenant workload |
| Per-vendor split (anthropic / openai / gemini / in-house) | `S_<vendor>` | tenant config + router decisions |
| Token-bucket refill rate per (tenant, vendor) | `B_refill_per_sec` | per-vendor rate-limit policy |

## Provider-router Sizing

### Formulae

```
total_router_qps_peak = sum_over_tenants(R_calls_peak)
total_router_qps_avg  = sum_over_tenants(R_calls_avg)

router_pod_throughput  = 5000 qps per pod (single core; ≤ 5 ms p99 in-process decision)
router_pod_count_peak  = ceil(total_router_qps_peak / router_pod_throughput * 1.5)   # 50% headroom
router_pod_count_avg   = ceil(total_router_qps_avg  / router_pod_throughput * 1.2)   # 20% headroom

router_rest_replicas   = max(router_pod_count_peak, 2)     # min 2 for HA
router_worker_replicas = max(2, ceil(N_tenants / 1000))     # health monitor + cost roll-up
router_app_replicas    = max(router_pod_count_avg, 2)
```

### Per-component baseline (XS tier; 20 tenants; peak 100 qps total)

| Component | Replicas | Per-pod resources |
|---|---|---|
| `oya-foundry-providers-router-rest` | 4 | 2 core / 2 Gi |
| `oya-foundry-providers-router-worker` | 2 | 2 core / 2 Gi |
| `oya-foundry-providers-router-app` | 2 | 2 core / 2 Gi |

### Per-tier scaling

| Tier | N_tenants | Peak qps | router-rest replicas | router-worker replicas |
|---|---|---|---|---|
| XS | 20 | 100 | 4 | 2 |
| S | 100 | 500 | 6 | 2 |
| M | 1000 | 5000 | 12 | 3 |
| L | 10000 | 50000 | 60 | 12 |

## Adapter Sizing

Adapter pods are largely network-bound (upstream HTTP). Throughput depends on upstream RTT.

```
adapter_pod_throughput = (1 / upstream_p99_rtt_seconds) * concurrent_requests_per_pod
                       ≈ 200 req/s per pod for 1s p99 RTT and 200 concurrent
adapter_pod_count_per_vendor = ceil(per_vendor_qps_peak / adapter_pod_throughput * 1.5)
```

### Per-vendor baseline (XS tier)

| Vendor | Per-vendor qps peak | Adapter replicas | Notes |
|---|---|---|---|
| anthropic-api | 60 | 2 | min HA |
| anthropic-subscription | 10 | 2 | min HA; FRAGILE — extra monitoring |
| openai-api | 25 | 2 | min HA |
| openai-subscription | 5 | 2 | min HA |
| gemini-api | 10 | 2 | min HA |
| gemini-subscription | 5 | 2 | min HA |
| in-house | 5 | 2 | per ADR-0026 phase-1 |
| openbao-bridge (credential resolver) | matches all of above | 2 | sidecar pattern; mTLS to OpenBao agent |

## Postgres Sizing (provider-config)

```
config_rows ≈ N_tenants × N_vendors × 5  # config + capability profile + ceilings + pins + per-pack overrides
storage_per_day ≈ N_tenants × 100 KB  # config audit + change log

postgres_primary_resources = 2 core / 4 Gi RAM / 50 GB SSD (XS)
postgres_replica_resources = 2 core / 4 Gi RAM / 50 GB SSD (XS)
```

### Per-tier sizing

| Tier | Postgres CPU | Postgres RAM | Postgres SSD |
|---|---|---|---|
| XS | 2 core | 4 Gi | 50 GB |
| S | 4 core | 8 Gi | 100 GB |
| M | 8 core | 16 Gi | 200 GB |
| L | 16 core | 32 Gi | 500 GB |

## Valkey Sizing (rate-limit / token-bucket state)

```
buckets ≈ N_tenants × N_vendors × N_transports  # ~ 20 × 4 × 2 = 160 buckets for XS
bytes_per_bucket ≈ 200 bytes
total_redis_state ≈ buckets × bytes_per_bucket  # ≈ 32 KB for XS
```

Valkey is small-state — sentinel HA with 3 replicas (each 1 core, 1 Gi RAM) is sufficient through L tier.

## OpenBao Agent Sizing

OpenBao agent runs as sidecar per adapter pod. Agent provides credential resolution from local socket — ~1 ms p99 cached, ~10 ms p99 on cache miss.

```
agent_qps_per_pod ≈ adapter_qps_per_pod (1:1 ratio with adapter calls)
agent_resources ≈ 0.5 core / 256 Mi RAM per sidecar
```

## Cost Per Tier (substrate only; vendor cost separate per cost-budget.md)

See `cost-budget.md` for the full tier-by-tier cost table.

## Verify-against

- Reference vLLM/TGI throughput benchmarks for in-house GPU sizing (per ADR-0026).
- Vendor SDK guidance for connection-pool sizing (Anthropic / OpenAI / Google).
- Cloudflare AI Gateway + LiteLLM published throughput benchmarks for cross-reference.

## Verification

- `cargo run -p oya-dev-cli -- gate validate capacity-model --microservice foundry-providers` exits 0.
- Quarterly capacity refresh: actual qps + p99 latency vs forecast.
- Load-test (`tests/load/router_decision.rs`) demonstrates router throughput per replica matches the formula.

## References

- `microservices/intelligence-providers/cost-budget.md`.
- `microservices/intelligence-providers/multi-region.md`.
- ADR-0026 (in-house substrate).
- vLLM scaling guide — `vllm.ai`.
- Anthropic SDK docs — `anthropic.com/docs`.
- OpenAI rate-limit docs — `openai.com/docs/rate-limits`.
