---
contract: throttling-tiers
authored: 2026-05-18
canonical_authority: ADR-0178
related_specs:
  - /specs/throttling-tiers.json
related_adrs:
  - ADR-0021
  - ADR-0044
  - ADR-0148
  - ADR-0157
  - ADR-0177
  - ADR-0178
status: canonical-base
authorities_cited:
  - Cloudflare Engineering — per-key + per-IP rate limiting (2023)
  - AWS API Gateway — per-API-key throttling docs
  - Stripe Engineering — layered rate limiting (2018)
  - Twilio Engineering — edge throttling architecture (2022)
  - Shopify Engineering — per-shop request budgets (2023)
  - RFC 6585 — HTTP 429
---

# Layered throttling — per-IP / per-API-key / per-user / per-tenant

## Layers and evaluation order

```
Request → per-IP → per-API-key → per-user → per-tenant → handler
```

Any layer's denial short-circuits subsequent evaluation.

## Per-layer policy

| Layer | Counter store | Window | Default budget | Burst | Denial header |
| --- | --- | --- | --- | --- | --- |
| per-IP | Redis mesh-edge cache | rolling 1 min | 100 req/min (anon), 10000 req/min (mesh-internal) | 20 tokens | `throttle-class: ip-abuse` |
| per-API-key | Envoy ratelimit-service | rolling 1 min + 1 hour | 1000/min, 50000/hour | 200 tokens | `throttle-class: api-key` |
| per-user | Tenant-scope Redis per cell | rolling 1 min | 600 req/min | 60 tokens | `throttle-class: user` |
| per-tenant | Cell-level Postgres + Redis lookaside | rolling 1 min + 1 day | Free=1k, Pro=10k, Enterprise=negotiated | budget × 0.2 | `throttle-class: tenant` |

Denial: HTTP 429 with `Retry-After` per RFC 6585.

## Headroom headers

Every response emits four headroom headers (`0.0` = no headroom; `1.0`
= full headroom):

```
throttle-ip-headroom: 0.83
throttle-key-headroom: 0.62
throttle-user-headroom: 0.94
throttle-tenant-headroom: 0.71
```

Public-customer SDKs read these and self-bias their request patterns.
The µservice also publishes them as Prometheus gauges:

```
throttle_headroom{layer="ip|key|user|tenant", microservice="<name>"}
```

## Brown-out integration

When per-tenant headroom drops below 0.05, the µservice transitions to
`degraded` class per ADR-0176. The headroom signal is the canonical
input to the degradation classifier's resource-pressure dimension.

## Per-microservice declaration

In `microservices/<ms>/manifest.json`:

```yaml
throttle_policy:
  per_ip:
    steady_rate_per_min: 100
    burst_capacity: 20
  per_api_key:
    steady_rate_per_min: 1000
    burst_capacity: 200
  per_user:
    steady_rate_per_min: 600
    burst_capacity: 60
  per_tenant:
    steady_rate_per_min:
      free: 1000
      pro: 10000
      enterprise: negotiated
    burst_capacity_ratio: 0.2
```

Defaults from this standard; per-µservice overrides allowed when the
manifest declares a custom block.

## Internal traffic exception

Per ADR-0177 the internal API surface (`internal-api.oyatie.com`):

- per-IP: 10× public budget (mesh-internal traffic).
- per-API-key: N/A (mesh mTLS + SPIFFE id; no public keys).
- per-user: same budget.
- per-tenant: same budget.

## Token-bucket implementation

Token-bucket is implemented per cell (so a cell's tokens are
independent across cells — preserves cell-isolation per ADR-0009). The
leak rate = steady_rate / 60 per second. Burst capacity is the
bucket's maximum token count.

## Observability

Dashboards:

- `microservices/observability/dashboards/throttling.md` —
  per-layer denial rates + headroom percentiles.

Alerts:

- Sustained denial rate > 1% for any layer → SEV-3 page to
  ops-sre-reliability.
- Per-tenant tenant-budget-exhausted → SEV-2 to ops-finops (via
  ADR-0174).

## Anti-patterns

- Adaptive ML rate limiting as primary mechanism — banned (brittle
  under traffic shape changes). May augment as a secondary signal.
- Single per-tenant layer only — banned (lets a noisy user starve
  the tenant).
- Per-µservice ad-hoc throttle implementations — banned (must use the
  canonical layer policy).

## Coverage tracker

Per-µservice rollout in `registry/throttling/coverage-tracker.tsv`.
Validator lane `throttling-tiers` is advisory until rollout completes.
