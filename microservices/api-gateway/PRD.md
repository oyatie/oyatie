---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-api-gateway
microservice: api-gateway
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
related_adrs: [ADR-0001, ADR-0007, ADR-0009, ADR-0049, ADR-0114, ADR-0121, ADR-0128, ADR-0131, ADR-0145, ADR-0148, ADR-0157, ADR-0158, ADR-0163, ADR-0166]
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

## Architecture

- Layer 7 (per ADR-0105 13-layer enum) — pure adapter; zero domain logic.
- Data plane: Envoy 1.30 LTS in DaemonSet per cell.
- Control plane: Envoy Gateway 1.1 (Kubernetes Gateway API CRDs).
- Rate-limit backend: Valkey 8.1 (Redis wire-compat) in cell-µservice cluster.
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
