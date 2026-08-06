---
id: ADR-0178
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - ops-security
  - axis-cloud
  - platform-api-sdk
supersedes: []
superseded_by: [ADR-709]
related:
  - ADR-0021-intelligence-capability-registry-and-mcp-gateway.md
  - ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md
  - ADR-0148-service-mesh-cilium.md
  - ADR-0157-api-gateway-tier.md
  - ADR-0177-internal-external-api-surface-separation.md
doc_class: Architecture-Decision-Record
purpose: >
  Four throttle layers evaluated outermost-first — per-IP (anti-abuse),
  per-API-key (developer-facing), per-user (within tenant), per-tenant
  (cell-level). Each layer has its own counter store, denial semantics,
  and headroom gauge.
enforcement_status: advisory-until-public-rpc-coverage-complete
enforced_by: oya gate validate throttling-tiers
---

# ADR-0178: Layered throttling — per-tenant / per-user / per-IP / per-key

## Status

Accepted — 2026-05-18. Enforcement is advisory until every public RPC
declares its throttle policy per layer. Coverage tracker at
`registry/throttling/coverage-tracker.tsv`.

## Context

ADR-0021 mentions per-tenant rate limiting in the foundry MCP gateway.
ADR-0044 + ADR-0148 cover mesh-level throttles. ADR-0157 establishes
the API gateway tier and per-key rate limiting. But the portfolio has
no ADR establishing the *layered* throttle policy across:

1. **Per-IP** — anti-abuse layer. A single abusive IP must not be
   able to consume a meaningful slice of *any* downstream budget.
2. **Per-API-key** — developer-facing layer. A single key (typically
   one external customer's developer key) gets a documented budget.
3. **Per-user** — within-tenant layer. A single user inside a tenant
   must not be able to exhaust the tenant's budget.
4. **Per-tenant** — cell-level layer. A single tenant must not be
   able to exhaust a cell's compute or a downstream µservice's budget.

Without the layered policy, three failure modes become inevitable:

- A noisy user inside a tenant exhausts the tenant's budget, starving
  other users.
- A single abusive IP attacks the API and the per-tenant counter never
  trips (the abuser uses many tenant keys).
- A single API key burst-spikes and starves cross-µservice flows.

The Cloudflare model + AWS API Gateway model + Stripe + Twilio + Shopify
all converge on the same layered shape. This ADR adopts it.

## Decision

### D-1. Four layers, evaluated outermost-first

```
Request → [per-IP throttle]
              ↓ (allow)
          [per-API-key throttle]
              ↓ (allow)
          [per-user throttle]
              ↓ (allow)
          [per-tenant throttle]
              ↓ (allow)
          handler
```

Any layer's denial short-circuits subsequent evaluation.

### D-2. Per-layer policy

| Layer | Counter store | Window | Default budget | Denial code | Header emitted |
| --- | --- | --- | --- | --- | --- |
| **per-IP** | Redis (mesh-edge cache) | rolling 1 min | 100 req/min (anonymous), 10000 req/min (mesh-internal) | 429 + `oya-throttle-class: ip-abuse` | `oya-throttle-ip-headroom: <0..1>` |
| **per-API-key** | Gateway (Envoy) ratelimit-service | rolling 1 min + rolling 1 hour | 1000 req/min, 50000 req/hour (default; per-key override allowed) | 429 + `oya-throttle-class: api-key` | `oya-throttle-key-headroom: <0..1>` |
| **per-user** | Tenant-scope cache (Redis cluster per cell) | rolling 1 min | 600 req/min | 429 + `oya-throttle-class: user` | `oya-throttle-user-headroom: <0..1>` |
| **per-tenant** | Cell-level store (Postgres + Redis lookaside) | rolling 1 min + rolling 1 day | per tenant tier (Free=1k/min, Pro=10k/min, Enterprise=negotiated) | 429 + `oya-throttle-class: tenant` | `oya-throttle-tenant-headroom: <0..1>` |

Denial bodies include `Retry-After` per RFC 6585.

### D-3. Headroom gauge

Every public response emits the four headroom headers (`0.0` = no
headroom; `1.0` = full headroom). Upstream callers — including
public-customer code — observe headroom and *bias* their own request
patterns. The headroom signal is also the input to the brown-out
classifier (ADR-0176) at the cross-cutting carriers layer: when
per-tenant headroom drops below 0.05, the µservice transitions to
`degraded`.

### D-4. Burst handling

Per-layer leak rate uses a token-bucket model with burst capacity:

| Layer | Steady rate | Burst capacity |
| --- | --- | --- |
| per-IP | 100/min | 20 tokens |
| per-API-key | 1000/min | 200 tokens |
| per-user | 600/min | 60 tokens |
| per-tenant | per tier | per tier × 0.2 |

Token-bucket implementation per cell (so a cell's tokens are independent
of cross-cell traffic, preserving cell isolation per ADR-0009).

### D-5. Internal traffic exception

Per ADR-0177, internal-surface routes (`internal-api.oyatie.com`) have
10× public budget at the per-IP layer. The per-API-key layer is
inapplicable (internal traffic uses mesh mTLS / SPIFFE id, not API
keys). The per-user + per-tenant layers still apply at the same budget.

### D-6. Per-µservice declaration

Each µservice manifest declares its layered policy:

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
    steady_rate_per_min: # per tier
      free: 1000
      pro: 10000
      enterprise: negotiated
    burst_capacity_ratio: 0.2
```

Defaults come from this ADR; per-µservice overrides allowed.

### D-7. Observability

Per-layer denial rate + per-layer headroom percentile (p50/p95/p99)
plotted on `microservices/observability/dashboards/throttling.md`.
Anomaly: sustained denial rate > 1% pages ops-sre-reliability (SEV-3).

## Alternatives considered

### Alt-1. Per-tenant only (single layer)

Use only the per-tenant throttle. **Rejected.** Lets a single noisy
user starve other users in the same tenant; lets a single abusive IP
hit the tenant's key from multiple geographic IPs and exhaust the
tenant budget while remaining invisible per-IP.

### Alt-2. Per-user only

Use only the per-user throttle. **Rejected.** Pre-authentication
traffic (failed-auth attempts, sign-up spam, anonymous documentation
reads) has no user; defeats anti-abuse posture.

### Alt-3. Provider-native rate limiting (Envoy, Istio) without
        per-layer declaration

Configure the mesh's built-in rate limiter without a per-µservice
declaration. **Rejected.** Mesh configuration drifts; per-µservice
budget tuning is unobservable; the four layers cannot be coordinated
without an explicit declaration.

### Alt-4. ML-driven adaptive rate limiting

Drop the explicit budgets and use an adaptive limiter that learns
per-tenant. **Rejected.** Adaptive limiting is brittle under traffic
shape changes (Black Friday, regulator-driven spikes) and produces
unobservable denial patterns. We may *augment* with adaptive signals
later but the explicit layered model is the canonical default.

## Consequences

### C-1. Positive

- **Noisy-user containment.** Per-user layer prevents a tenant's user
  from exhausting the tenant budget.
- **Anti-abuse posture.** Per-IP layer absorbs DDoS-class abuse.
- **Hyperscaler-grade.** Matches Cloudflare + AWS API Gateway + Stripe
  + Twilio + Shopify layered models.
- **Brown-out signal input.** Headroom gauges feed the ADR-0176
  brown-out classifier.
- **Per-cell isolation preserved.** Token buckets are per cell.

### C-2. Negative

- **Four counter stores per request adds latency.** Mitigation: per-IP
  + per-key counters are local to the gateway; per-user + per-tenant
  are local to the cell; total added latency ≤ 2 ms at p99 (measured
  in the existing observability µservice latency budget).
- **Per-layer budget tuning is harder than single-layer.** Mitigation:
  defaults are conservative; per-µservice overrides are advisory-doc'd.
- **Denial response format must be uniform.** Mitigation: this ADR
  pins the format.

### C-3. Sustainability

- Headroom signals biasable load shedding when a cell's PUE exceeds
  a threshold: shed at the per-user layer first (least-impactful),
  then per-tenant, then per-key, then per-IP (most-impactful).

## Implementation surface

- `specs/throttling-tiers.json` — canonical layer enum + per-layer
  policy schema.
- `docs/standards/throttling-tiers.md` — full standards doc.
- `microservices/observability/dashboards/throttling.md` — dashboard
  schema.
- `registry/throttling/coverage-tracker.tsv` — per-µservice rollout.
- Validator lane `throttling-tiers` added to
  `AGGREGATED_VALIDATE_LANES` (advisory).
- Implementation crates (existing or planned):
  - `oya-cloud-rate-limit-domain` (kernel-tier counter logic;
    existing crate per ADR-0028 catalog).
  - Mesh policy in `microservices/cloud-iac/iac/k8s/cilium/`.
  - Gateway ratelimit-service config in
    `microservices/cloud-iac/iac/k8s/envoy/`.

## References

- Cloudflare Engineering — *Rate limiting: per-key + per-IP* (2023).
- AWS API Gateway — *Per-API-key throttling* (AWS docs).
- Stripe Engineering — *Layered rate limiting at Stripe* (public blog
  2018, still canonical).
- Twilio Engineering — *Edge throttling architecture* (2022).
- Shopify Engineering — *Per-shop request budgets* (public engineering
  blog 2023).
- RFC 6585 — *Additional HTTP Status Codes* (429).
- ADR-0021 (this portfolio) — foundry MCP gateway per-tenant rate limit.
- ADR-0157 (this portfolio) — API gateway tier.
- ADR-0177 (this portfolio) — internal vs external API surface
  separation.
