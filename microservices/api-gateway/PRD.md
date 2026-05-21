---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-api-gateway
microservice: api-gateway
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
related_adrs:
  - ADR-0001
  - ADR-0007
  - ADR-0009
  - ADR-0049
  - ADR-0114
  - ADR-0121
  - ADR-0128
  - ADR-0131
  - ADR-0145
  - ADR-0148
  - ADR-0157
  - ADR-0158
  - ADR-0163
  - ADR-0166
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs: [/specs/hyperscaler-architecture-invariants.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-18
owner_team: axis-network
doc_status: published
---

# PRD-api-gateway: Dedicated North-South Edge Tier

## Purpose

The `api-gateway` µservice is oyatie's canonical north-south entry tier — the single edge that every external (tenant browser, mobile, partner API, public-internet) HTTP / gRPC / WebSocket request transits before reaching any workload µservice. It is the runtime operationalization of ADR-0157.

It is shared substrate; not a hero product. It exists once per cell (ADR-0009) and is consumed by every other µservice that exposes an external surface. Its existence is the precondition for uniform TLS, JWT/mTLS/OAuth authentication, OWASP-CRS-class WAF, per-tenant token-bucket DDoS protection, global rate-limiting (first tier; the per-µservice INV-SHUFFLE-SHARDING is the second tier), W3C trace-context injection, OpenAPI 3.1 schema enforcement at edge, and per-cell residency enforcement.

## Scope

In:
- TLS 1.3 termination + cert rotation per ADR-0064 + ACME automation (cert-manager).
- AuthN: JWT bearer (tenancy-µservice-issued); mTLS partner cert; OAuth 2.1 + PAR (RFC 9126) for human flows.
- AuthZ (coarse): Cedar fragment for hostname / tenant_id scoping. Fine-grained auth stays at workload tier.
- WAF: Coraza + OWASP CRS 4.x rules; per-pack overlay tuning.
- DDoS: per-tenant token-bucket; L4 SYN-cookie; Envoy listener filters.
- Global rate-limit (first tier): Envoy ratelimit + Valkey backend.
- Request mutation: W3C traceparent / tracestate / x-oya-cell-id / x-oya-persona-tier headers.
- Schema enforcement: OpenAPI 3.1 fail-fast for malformed requests.
- Per-cell deployment + cross-cell traffic rejection (ADR-0049).
- API-key prefix routing (`sk_test_` / `sk_stage_` / `sk_live_` per ADR-0163).

Out:
- Domain-tier authorization (workload µservice + ADR-0148 AuthorizationPolicy).
- Per-resource Cedar evaluation (workload µservice + ADR-0007).
- Per-µservice shuffle-sharding (workload µservice ingress + ADR-0128).
- Business logic of any kind.

## Personas

- **Tenant developer.** Calls oyatie APIs; expects predictable 4xx on schema error, predictable 429 on rate-limit, predictable 401 on bad JWT.
- **Tenant admin.** Manages API key issuance (issued by tenancy µservice; gated here).
- **SRE / on-call.** Owns api-gateway availability; expects per-cell HA + < 5 ms p99 added latency.
- **Compliance auditor.** Reads the WAF rule pack, the JWT-verification posture, the per-tenant rate-limit policy, in one place.

## Functional requirements

1. **F-AGW-01** — Terminate TLS 1.3 at the edge with cert rotation < 24h.
2. **F-AGW-02** — Verify JWT bearer with JWKS rotation; reject expired / malformed; emit audit-chain seal on auth-failure spike.
3. **F-AGW-03** — Verify mTLS partner certs against the tenancy-µservice partner-cert registry.
4. **F-AGW-04** — Run Coraza WAF in line; emit audit-chain seal on rule trigger.
5. **F-AGW-05** — Apply per-tenant + per-route + per-IP-prefix global rate-limits via Envoy ratelimit + Valkey.
6. **F-AGW-06** — Inject W3C trace-context + x-oya-cell-id + x-oya-persona-tier headers.
7. **F-AGW-07** — Validate request body + path against OpenAPI 3.1 schema; fail-fast 4xx on violation.
8. **F-AGW-08** — Reject cross-cell traffic at edge (ADR-0049 residency invariant).
9. **F-AGW-09** — Route by API-key prefix to the env-tier-specific workload pool (ADR-0163).
10. **F-AGW-10** — Emit OpenTelemetry metrics + traces + logs to the observability µservice.

## Non-functional requirements

- **Availability** ≥ 99.99% per cell.
- **Added latency p99** ≤ 5 ms over the workload-tier baseline (TLS terminate + auth + WAF + rate-limit eval combined).
- **Throughput** ≥ 50k req/s per replica baseline (Envoy 1.30 LTS bench).
- **JWKS cache refresh** ≤ 60 s.
- **WAF rule update propagation** ≤ 5 min cell-wide.
- **Audit-chain emission** on every auth-failure spike, every WAF trigger, every rate-limit deny ≥ 99.9% success rate.

### DR posture

| Field | Value |
|---|---|
| ADR | ADR-0343 |
| Target | RTO 300 s and RPO 0 s for stateless edge admission, route-cache refresh, and cell evacuation, matching `manifest.json#dr`. |
| Compliance-pack floor | HIPAA floor RTO 3600 s / RPO 300 s, EU-AI-ACT high-risk floor RTO 1800 s / RPO 300 s, SOC2-T2 floor RTO 14400 s / RPO 900 s; api-gateway's manifest target is stricter at 300 s / 0 s. |
| Failover runbook | `runbooks/cell-evac.md`, `runbooks/blue-green-rollback.md`, and `runbooks/edge-admission-regression.md`. |
| Multi-region active-active | Yes. The PRD already declares `active_active` per cell; manifest `cell_eligibility=["tier-0"]` keeps the edge in Tier-0 cells. |
| WHY | External tenants see the edge first; DR must shed or reroute a bad cell without losing request identity, policy evidence, or audit correlation. |

### Capacity model

| Field | Value |
|---|---|
| ADR | ADR-0340, with pod runtime tier declared by ADR-0338. |
| Per-tenant baseline | `manifest.json#capacity_model`: 0.10 vCPU, 128 MiB RAM, 0 GB durable storage, and connections `{valkey: 6, postgres: 0, outbound_http: 16}` per tenant. `capacity-model.md` per Tier-0 edge cell remains 50K TLS handshakes/sec, 5M sustained connections, 250K HTTP req/sec, 500K Cedar evals/sec, and 1M Valkey lookups/sec. |
| Scaling dimension | `per_request` for admission, WAF, auth, rate-limit, and routing; `per_capability` for TLS/ECH/PQC rotation and canary/blue-green controls. |
| Cell placement class | Tier-4 per `manifest.json#capacity_model.cell_placement_class`, deployed only in Tier-0 edge cells per `cell_eligibility=["tier-0"]`; pod runtime tier is ADR-0338 Tier-3 because `manifest.json#pod_runtime_tier=3` and the data plane is perf-critical edge/Envoy. |
| Autoscaling boundaries | Four 64-vCPU/192 GiB nodes per cell baseline; autoscale at >50% utilization for >5 min or >70% for >30 s; scale-down only below 20% for >30 min with single-cell delta. |
| WHY | The model preserves north-south admission and DDoS headroom while bounding per-tenant hot keys through Valkey shuffle-sharding. |

### Sustainability + cost attribution

| Field | Value |
|---|---|
| ADR | ADR-0344 |
| Per-call emission claim | Every admitted, denied, WAF, rate-limit, TLS, canary, blue-green, cell depool/repool, DDoS, and cache-poisoning audit row must include `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region`. |
| Carbon-aware routing | No for realtime request admission, emergency access, HIPAA emergency, PCI realtime fraud, or DDoS mitigation. Yes for non-urgent WAF rule propagation, blue/green analysis, certificate inventory checks, and cold-path log backfills when policy allows. |
| Tenant transparency surface | Tenant admins see per-usage edge request, denial, WAF, rate-limit, and TLS rotation cost in the FinOps portal, keyed to the manifest `paid_billing_components_emitted=["per_usage"]`. |
| WHY | CSRD, SB-253, and SEC climate-disclosure posture require edge traffic cost and emissions to be attributable, but request routing must prioritize safety, residency, and latency. |

### API versioning posture

| Field | Value |
|---|---|
| ADR | ADR-0342 |
| Public API version model | Date carrier triplet: `Oyatie-Version: YYYY-MM-DD`, `/v/YYYY-MM-DD/...` for route/admission management, and proto3 `oyatie_version`. |
| SDK semver model | Api-gateway SDKs use `major.minor.patch`; OpenAPI/AsyncAPI/proto contracts remain date-carrier pinned. |
| Support window | Last N=3 public versions supported for >=180 days. |
| Per-tenant pinning | Yes for route-management APIs and partner integrations; no for emergency deny rules, WAF safety fixes, or TLS/PQC security posture updates. |
| Internal-mesh exemption | Yes. ADR-0145 direct gRPC and Envoy control-plane mesh traffic remain exempt from public URL date prefixes. |

## Architecture

- Layer 7 (per ADR-0105 13-layer enum) — pure adapter; zero domain logic.
- Data plane: Envoy 1.30 LTS in DaemonSet per cell.
- Control plane: Envoy Gateway 1.1 (Kubernetes Gateway API CRDs).
- Rate-limit backend: Valkey 8.1 (RESP wire-compatible) in cell-µservice cluster.
- WAF: Coraza loaded as Envoy HTTP filter.
- JWKS cache: in-process at each Envoy replica; refresh on rotation event.
- Multi-region disposition: `active_active` per-cell (ADR-0158).

## Threat model summary (full at threat-model.md)

- **STRIDE coverage** at the edge: spoofing (JWT/mTLS); tampering (TLS); repudiation (audit-chain seals); info-disclosure (Cedar coarse-scope); DoS (rate-limit + WAF); elevation (Cedar coarse-scope).
- **OWASP API Security Top-10 2023**: R3 / R5 / R6 / R8 / R10 covered structurally; remainder gated by workload-tier Cedar.

## Compliance

- SOC 2 CC6.1 (logical access) + CC6.6 (transmission) + CC7.x (operational monitoring).
- ISO 27001 A.13.1 (network security) + A.9 (access control).
- PCI DSS 1.x (boundary controls).
- KR PIPA Art 28-8 (cross-border transfer rejection at edge).
- GDPR Art 32 (technical and organisational measures).

## Failure modes (full at failure-modes.md)

- **JWKS unavailable** → cached JWKS used up to TTL; fail-open with `503` on stale cache.
- **Valkey unavailable** → rate-limit falls back to per-replica local token bucket (advisory; not global).
- **WAF rule pack update breaks** → previous rule pack pinned; ops-paged on rollback.
- **Envoy panic** → DaemonSet healthcheck triggers reschedule; > 10s outage triggers cell-wide alert.
- **Cert rotation fails** → SLO breach; on-call paged within 60 min of expiry runway.

## Observability + SLOs

Authored in `slos/api-gateway.openslo.yaml`:

- `request-success-rate` ≥ 99.99% rolling 30d.
- `request-duration-p99` ≤ 5 ms added latency.
- `auth-failure-rate` < 1% baseline; alert on > 5% over 5 min.
- `waf-block-rate` baselined per pack; alert on 10× baseline.
- `rate-limit-deny-rate` baselined; alert on 10× baseline.

## Status

Skeleton PRD shipped 2026-05-18 alongside ADR-0157. Full PRD body + IP pack land in stacked PR. Implementation Plans (IP-001 … IP-015) authored separately.
