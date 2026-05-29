---
doc_class: CapacityModel
title: Capacity Sizing Model
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-application
deciders: ops-sre-reliability, axis-application, council-architecture
related_adrs: [ADR-0117, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/cost-budget.md
  - microservices/application/multi-region.md
  - microservices/application/PRD.md §"Horizontal Scalability"
  - microservices/application/policy/data-residency.md
review_cadence: quarterly + on every component-replica-set change
doc_status: published
---

# Capacity Sizing Model (application µservice)

## Purpose

Sizing formulae + reference baseline numbers for each Application Shell
component (shell-routing, tenant-context, auth-gateway, module-loader,
frontend-bundle-serve, Postgres + Citus, Valkey, CDN). Drives
`cost-budget.md` and `multi-region.md`. The Application Shell is the
front door — capacity headroom is set higher than product µservices
because shell outage = all products unreachable.

## Inputs

| Input | Variable | Source |
|---|---|---|
| Active tenants per pack | `N_tenants` | tenancy resolver |
| Avg concurrent sessions per tenant | `S_concurrent_per_tenant` | observed (M03: 5000) |
| Sign-in rate per second per tenant | `R_signin_per_sec_per_tenant` | observed (M03: 0.5) |
| Route resolves per session per minute | `R_routes_per_session_per_min` | observed (M03: 10) |
| Module load events per session per visit | `M_modules_per_session` | observed (M03: 6) |
| WASM bundle size (gzip) | `B_wasm_bytes` | build artifact (~1.5 MB) |
| Avg admin actions per tenant per day | `A_admin_per_day_per_tenant` | observed (M03: 20) |
| Audit seal latency target | `T_seal_seconds` | per `PRD.md` (≤1 s) |

## TTI Budget

Per PRD §"Performance":

```
TTI_warm_p99 ≤ 2000 ms
TTI_cold_p99 ≤ 3000 ms

Budget breakdown (warm path):
  DNS + TLS handshake (HTTP/3 0-RTT)     ≤  100 ms
  CDN cache hit (WASM bundle + CSS)      ≤  300 ms  (parallel)
  Auth cookie verify (server-side)       ≤   50 ms  (parallel)
  Shell HTML render (origin)             ≤  150 ms
  WASM instantiate                       ≤  400 ms
  Leptos hydrate                         ≤  300 ms
  First product-route resolve (Cedar)    ≤  100 ms
  First product module fetch (CDN warm)  ≤  500 ms
  ──────────────────────────────────────────────
  Sum (parallel-aware)                   ≤ 2000 ms p99
```

The bundle-size lane refuses any PR pushing WASM gzip > 2 MB.

## shell-routing Sizing

### Formulae

```
total_route_resolves_per_sec = N_tenants × S_concurrent_per_tenant × R_routes_per_session_per_min / 60
                             = N_tenants × S_concurrent_per_tenant × 0.167
```

Per Mimir + axum benchmarks (similar workload):
- Throughput per replica: 5 000 route-resolves/s (Cedar pre-compiled + cached).
- CPU per replica @ p99 100 ms: 4 vCPU.

```
replicas_routing = ceil(total_route_resolves_per_sec / 5000) + 1 (HA + headroom)
```

### Worked example (XS, pack-kr, M03)

```
N_tenants = 20, S_concurrent_per_tenant = 5000
total = 20 × 5000 × 0.167 = 16 700 resolves/s
replicas_routing = ceil(16700 / 5000) + 1 = 4 + 1 = 5  (round to 4 per cost-budget table; +1 HA spare)
```

## tenant-context Sizing

Throughput per replica: 20 000 resolves/s (cache-hit; OpenBao + tenancy
SDK). Light-weight middleware path.

```
replicas_tenant_context = ceil(total / 20000) + 1
```

Worked example: 16 700 / 20 000 ≈ 1 + 1 = 2 replicas.

## auth-gateway Sizing

```
total_signin_per_sec = N_tenants × R_signin_per_sec_per_tenant
                     = N_tenants × 0.5
```

Throughput per replica: 500 sign-ins/s (OIDC verify + cookie set;
constant-time path). CPU-bound on signature verify.

```
replicas_auth_rest = ceil(total_signin_per_sec / 500) + 1
replicas_auth_worker = 2 (session-rotation + revocation reaper; idle most cycles)
```

Worked example: 20 × 0.5 / 500 = 0.02 → floor 2 (HA min) + 1 = 3 → +1 burst headroom = 4 replicas.

## module-loader Sizing

```
total_module_loads_per_sec = N_tenants × S_concurrent_per_tenant × M_modules_per_session / 1800
                           = N_tenants × S_concurrent_per_tenant × 0.0033
```

(assuming each user visits once per 30 min)

Throughput per replica: 10 000 loads/s (signature verify amortized;
manifest cached). Signature verify is fast (Ed25519 ~ µs).

```
replicas_module_loader = ceil(total / 10000) + 1
```

Worked example: 20 × 5000 × 0.0033 = 330 → 1 + 1 = 2 replicas.

## frontend-bundle-serve Sizing

Mostly CDN-fronted; origin handles cache-miss + per-tenant shell HTML.

```
origin_qps = total_module_loads_per_sec × (1 - cdn_hit_ratio)  // typical 0.95
```

Worker (purge consumer): 2 replicas baseline (idle most cycles).

## Postgres + Citus Sizing

```
total_writes_per_sec  ≈ total_signin_per_sec × 2 (session insert + audit row)
                      + A_admin_per_day_per_tenant × N_tenants / 86400 × 3 (multi-row admin)
total_reads_per_sec   ≈ total_route_resolves_per_sec × 0.1 (cache miss)
                      + S_concurrent_per_tenant × N_tenants / 60 (audit list views)

storage_per_day_GB    ≈ N_tenants × S_concurrent_per_tenant × R_routes_per_session_per_min × 60 × 24 × audit_row_bytes / 1e9
```

(`audit_row_bytes ≈ 256 B` per row)

Worked example for XS:
- Writes: 10 + 0.005 ≈ 10/s; trivial.
- Reads: 1670 + 1667 ≈ 3 337/s.
- Storage: 20 × 5000 × 10 × 60 × 24 × 256 / 1e9 ≈ 37 GB/day; 30-day hot ~1.1 TB → too high; truncate route-audit to ≤ 1 sample/route + 90-day retention → 1 % of raw → ~11 GB/30d hot.

3-node HA Citus + RLS; tenant_id shard key per `policy/data-residency.md`.

## Valkey Sizing

```
sessions_in_flight = N_tenants × S_concurrent_per_tenant
bytes_per_session  ≈ 512 B (opaque token + binding)
memory_required    ≈ sessions_in_flight × bytes_per_session × 1.5 (headroom)
```

Worked example XS: 20 × 5000 × 512 × 1.5 ≈ 76 MB. Comfortable on 2 GB
Valkey nodes; 3-master + 3-replica cluster for HA.

## CDN Sizing

```
egress_per_day_GB = N_tenants × S_concurrent_per_tenant × M_modules_per_session × B_wasm_bytes / 1e9
                  × (1 - cdn_hit_ratio)  (origin)
                  + N_tenants × S_concurrent_per_tenant × M_modules_per_session × B_wasm_bytes / 1e9
                    × cdn_hit_ratio  (edge)
```

Worked example XS:
- Total: 20 × 5000 × 6 × 1.5 MB ≈ 900 GB/day.
- At hit ratio 0.95: 855 GB edge (POP egress) + 45 GB origin pull.
- Monthly: ~26 TB edge; per OCI CDN pricing ≈ $200 (pack-kr).

Cache-hit lane refuses deploys where rolling 7-day hit ratio < 90 %.

## Capacity envelope per cell

| Dimension | Baseline / cell | Max / cell | Scale-out trigger |
|---|---|---|---|
| Concurrent active sessions | 50 k | 5 M | Valkey memory > 80 % |
| Sign-ins / sec | 5 k | 100 k | Auth-gateway CPU > 70 % |
| Route resolves / sec | 50 k | 1 M | Shell-routing CPU > 70 % |
| Module loads / sec | 100 k | 5 M | CDN origin shield miss > 1 % |
| Tenants / cell | 1 k | 50 k | Postgres Citus shard > 80 % |
| TTI p99 | ≤2 s | breach @ >2.5 s for 5 min | Auto-rollback to prior bundle |

## Capacity drills

- Quarterly k6 ramp to 2× peak + 30 min sustain; verify HPA + p99 stays
  within budget.
- Annual chaos drill: kill one auth-gateway replica during ramp; verify
  zero new-sign-in failures.
- Monthly synthetic bundle-size + Lighthouse + WebPageTest probe.

## References

- ADR-0117 packs.
- `microservices/observability/capacity-model.md` (precedent + style).
- Grafana Mimir + axum + Citus published reference architectures.
