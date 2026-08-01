---
id: ADR-0157
status: Accepted
deciders: council-architecture, axis-network, axis-tenancy, axis-cloud-k8s, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: []
related: [ADR-0009, ADR-0049, ADR-0114, ADR-0121, ADR-0128, ADR-0145, ADR-0148, ADR-0182, ADR-0203, ADR-0258]
architectural_authority: ADR-0182 (gateway-vs-mesh separation principle; this ADR picks the implementation)
last_reconciled: 2026-08-01
reconciled_with: [ADR-0203, ADR-0258]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
---

# ADR-0157 — Dedicated API Gateway Tier (separate from per-µservice rate-limit)

## Status

Accepted (2026-05-18). Promotes a dedicated `api-gateway` µservice as the canonical north-south entry point for every external (tenant-facing, partner-facing, public-internet) REST, webhook, event, and streaming call into the oyatie hyperscaler shape.

### Public-contract reconciliation

Per ADR-0203 and ADR-0258, public contract carriers are REST documented by OpenAPI 3.2 plus
webhooks, events, and streams documented by AsyncAPI 3.1. gRPC over HTTP/2 (H2) with proto3 is
internal-only service-to-service traffic under mTLS; it is not a public API contract.

## Context

ADR-0128 named INV-SHUFFLE-SHARDING (per-tenant shuffle-sharding rate-limits) and INV-CELL-ISOLATION as hyperscaler invariants. ADR-0145 fixed inter-µservice east-west communication onto a service-mesh substrate + direct gRPC. ADR-0148 pins the mesh to Cilium Service Mesh (primary, sidecarless eBPF) with Istio Ambient waypoint as an opt-in Tier-2 per-namespace L7 overlay for advanced traffic management.

What has *not* been fixed is the **north-south boundary**:

- Tenant browser / mobile / partner-API traffic enters how?
- Where is global TLS terminated?
- Where is per-tenant authentication (JWT validation, mTLS-cert-based partner auth) enforced before any application µservice sees the request?
- Where is the WAF that catches OWASP Top-10 attack patterns before they reach domain logic?
- Where is the global rate-limit + DDoS protection (the per-µservice INV-SHUFFLE-SHARDING is the *second* tier, not the first)?

Without an explicit decision each µservice would re-implement these concerns inside its `surface/rest/` adapter. That violates ADR-0145 Invariant 5 (a single canonical surface) and the cohesion thesis (ADR-0001), inflates the attack surface (33 WAF implementations is 33 ways to misconfigure), and makes per-tenant rate-limit policy non-uniform. The hyperscaler precedent is unambiguous: every Tier-1 SaaS ships a dedicated edge tier:

- **AWS API Gateway / CloudFront** front-of-house for AWS-shipped APIs.
- **Stripe edge** (Cloudflare-fronted) terminates TLS, enforces per-account auth, runs WAF, and applies global rate-limits BEFORE the request ever reaches `api.stripe.com` regional pods.
- **Cloudflare**, **Google Cloud Armor + Cloud Load Balancing**, **Azure Front Door + WAF** all encode the same shape.

ADR-0157 promotes this from "implied by ADR-0121" to a first-class µservice decision so the boundary is auditable, ownable, and uniform.

## Decision

Oyatie adopts a dedicated **`api-gateway` µservice** as the canonical north-south entry tier. Every external REST, webhook, event, or streaming request transits the api-gateway tier before the cell-µservice tenant-routing layer hands it to a workload µservice.

### Operational shape

1. **Termination.** TLS 1.3 termination at the api-gateway edge (cert rotation per ADR-0064 canonical-base + per-pack overlay). Public REST over HTTP/1.1 or HTTP/2, WebSocket, and SSE are supported; internal gRPC/proto3 remains on the east-west H2+mTLS path and is not exposed by the public listener.
2. **AuthN.** JWT bearer (tenant-issued, signed by tenancy µservice JWKS) verified at the gateway; mTLS partner certificates verified at the gateway; OAuth 2.1 + PAR per RFC 9126 (Pushed Authorization Requests) for human flows.
3. **AuthZ at the gateway.** Cedar fragment for *coarse* tenant scoping ("is this JWT's tenant_id allowed to talk to this hostname?"). Fine-grained per-resource Cedar evaluation remains at the workload µservice (per ADR-0145 + ADR-0148 — AuthorizationPolicy on the mesh).
4. **WAF.** Coraza (OWASP open-source) running in the gateway data path. Default rule pack = OWASP CRS 4.x; per-pack overlays in `iac/kustomize/components/waf/`. Rules covering OWASP API Security Top-10 (2023) — broken object-level auth, broken authentication, broken object-property-level auth, unrestricted resource consumption, broken function-level auth, server-side request forgery, security misconfiguration, lack of inventory, unsafe consumption.
5. **DDoS.** Per-tenant token-bucket limits at the gateway tier (e.g. 10k req/min per tenant default; per-pack overlays raise/lower). Layer 4 SYN-cookie + connection-rate limits enforced by Envoy listener filters. Anycast IP fronting + traffic engineering handled at the cloud provider's L4 LB (cloud-k8s pack overlay).
6. **Global rate-limit (first tier).** Envoy global rate-limit service (gRPC) backed by Redis (cell µservice's regional Redis cluster). Per-tenant + per-route + per-IP-prefix dimensions.
7. **Per-µservice rate-limit (second tier).** INV-SHUFFLE-SHARDING remains at the workload-µservice ingress as authored in ADR-0128. The gateway tier is *first* defense; per-µservice shuffle-sharding is *second*. Both tiers required.
8. **Request mutation.** The gateway injects W3C trace-context headers, the tenant-routing header `x-oya-cell-id`, and the persona-tier header `x-oya-persona-tier` (computed from the JWT) so downstream µservices read consistent context.
9. **Schema enforcement.** OpenAPI 3.1 schemas registered with the api-gateway tier (sourced from each workload µservice's `contracts/openapi-v1.yaml`). Requests failing schema validation fail-fast at the edge (4xx) without reaching the workload tier.
10. **Per-cell deployment.** The api-gateway tier deploys per-cell (per ADR-0009). A KR cell has its own api-gateway pool; a EU cell has its own pool. Cross-cell traffic is rejected at the gateway (ADR-0049 residency invariant).

### Tech selection

- **Data plane: Envoy 1.30 LTS.** Envoy is the de-facto hyperscaler edge proxy; integrates with Istio (already adopted in ADR-0148) so the gateway tier is "Envoy as an edge listener, same control-plane primitives we use east-west".
- **Control plane: Envoy Gateway 1.1 (Kubernetes Gateway API).** Standardizes on the Kubernetes Gateway API (graduated 2024) — cross-cloud-portable, replaces the legacy `Ingress` resource. Cite Envoy Gateway project (envoyproxy/gateway), graduated CNCF.
- **Rate-limit service: Envoy ratelimit + Redis backend.** The cell-µservice provisions the Redis cluster.
- **WAF: Coraza ModSecurity-compatible engine.** Loaded as an Envoy HTTP filter.

### What lives in api-gateway, what does NOT

In api-gateway:

- TLS termination, JWT/mTLS/OAuth verification, coarse Cedar tenant-scoping, WAF, DDoS protection, global rate-limit (first tier), trace-context injection, OpenAPI schema enforcement, per-cell traffic rejection.

NOT in api-gateway:

- Domain-tier authorization (stays in workload µservice's `AuthorizationPolicy` per ADR-0148).
- Per-resource Cedar evaluation (stays in workload µservice per ADR-0007).
- Per-µservice shuffle-sharding (stays at workload-µservice ingress per ADR-0128).
- Business logic of any kind.

The api-gateway µservice has zero domain logic; it is pure adapter (Layer 7 in the 13-layer enum per ADR-0105).

## Alternatives considered

### Alternative A — Per-µservice gateway (each µservice owns its own edge)

- **Pros:** maximum µservice autonomy; no shared single-point-of-failure tier.
- **Cons:** 33 separate TLS-termination implementations; 33 separate JWT verification flows; 33 separate WAF policies; 33 separate per-tenant rate-limits → impossible to audit uniformly; OWASP API Security Top-10 R10 ("unsafe consumption") fails by construction because per-µservice WAF coverage drifts.
- **Rejected because:** the cohesion thesis (ADR-0001) plus the hyperscaler precedent (Stripe / AWS / Google) both forbid per-µservice edge tiers. The audit cost alone (33 separate compliance attestations vs. 1) is dispositive.

### Alternative B — Service-mesh ingress gateway only (Istio IngressGateway, no dedicated api-gateway µservice)

- **Pros:** reuses existing Istio control plane; zero new µservice.
- **Cons:** Istio IngressGateway does NOT include WAF (Coraza is not a default Istio component); JWT-level auth at IngressGateway exists but lacks the per-tenant policy surface this decision needs; OWASP CRS rule pack is not Istio-native; per-tenant token-bucket DDoS requires a Redis tier that does not exist in vanilla Istio.
- **Rejected because:** Istio IngressGateway is the L7 routing layer; the api-gateway tier is the *defense-in-depth* tier in front of it. They are different concerns; collapsing them violates ADR-0145 Invariant 5 (canonical surface separation).

### Alternative C — Dedicated api-gateway µservice (this ADR's choice)

- **Pros:** uniform tenant-facing edge; one place to audit auth/WAF/DDoS/rate-limit; clean separation from east-west mesh; aligns with AWS/Stripe/Cloudflare precedent; ADR-0009 per-cell deployment fits naturally; OWASP API Security Top-10 R8 ("security misconfiguration") becomes auditable in one place.
- **Cons:** new µservice to own + operate; requires the `cloud-k8s` pack to provision Envoy Gateway + Coraza per cell; one more tier in the request path (~2-5 ms added latency budget per ADR-0145 latency budgets).
- **Accepted.**

### Alternative D — Cloud-provider managed gateway only (AWS API Gateway native, GCP API Gateway native)

- **Pros:** zero ops cost.
- **Cons:** AWS-API-Gateway-specific request/response shape; vendor lock-in; cannot ship to EU sovereign cells (Schrems-class concerns; ADR-0049); cannot ship on-prem (ADR-0121); WAF rules cannot be expressed identically across providers.
- **Rejected because:** ADR-0121 requires cluster-portable substrate; vendor-managed gateway breaks portability invariant.

### Alternative E — Kong / Traefik / NGINX-based commercial gateway

- **Pros:** mature product, ecosystem of plugins.
- **Cons:** licensing posture (Kong Enterprise / commercial NGINX) drives per-cell cost; plugin ecosystem doesn't compose with Envoy/Istio that we already own; introduces a second L7 stack to maintain.
- **Rejected because:** Envoy already in the building (ADR-0148); a second L7 stack is duplicate cost.

## Consequences

### Positive

1. **Single auditable edge.** SOC 2 CC6.1 + ISO 27001 A.13.1 (network controls) + PCI DSS 1.x evidence rolls up at one tier. One WAF, one rate-limit policy, one JWT verification implementation.
2. **OWASP API Security Top-10 (2023) covered structurally.** R3 broken-object-property-auth, R5 broken-function-level-auth, R6 unrestricted-resource-consumption, R8 security-misconfiguration, R10 unsafe-consumption all become single-tier gates.
3. **Per-cell traffic enforcement.** A KR cell's api-gateway rejects traffic destined for an EU tenant (ADR-0049 residency invariant); the workload tier never sees the request.
4. **Layered defense.** Edge tier (this ADR) + mesh tier (ADR-0148) + per-µservice ingress tier (ADR-0128 INV-SHUFFLE-SHARDING) form a defense-in-depth ladder.
5. **Schema enforcement at the edge.** OpenAPI 3.1 schema-fail-fast prevents malformed payloads from reaching domain logic; reduces fuzzing surface for workload tiers.
6. **Trace-context propagation closed.** W3C trace headers injected at the edge → preserved through Istio → received at workload µservice. ADR-0145 Invariant 2 closed end-to-end.

### Negative

1. **One more µservice to own.** Adds `microservices/api-gateway/` to the inventory (now 34 µservices).
2. **Added latency.** ~2-5 ms per request for TLS termination + auth + WAF + rate-limit evaluation. ADR-0145 latency budget revised accordingly.
3. **Per-cell HA pool required.** The api-gateway tier itself must be highly-available; minimum 3 replicas per cell with anti-affinity. Adds cell capacity cost.
4. **WAF false-positive risk.** OWASP CRS rule pack out-of-the-box has known false-positive cases; per-pack tuning required during onboarding.
5. **Schema-fail-fast can break legacy clients.** Tenants with non-compliant clients see 4xx where previously the workload tier might have been lenient; migration window required.

### Operational

1. New µservice scaffolded at `microservices/api-gateway/` per ADR-0131 flat layout. PRD skeleton ships with this ADR (see Companion); full IP pack lands in the stacked PR.
2. New CI lane `cloud-ci/Rust gate packet api-gateway-tier` enforces (a) all external endpoints in any other µservice's OpenAPI 3.1 spec resolve under the api-gateway tier's route table, (b) no other µservice declares a `LoadBalancer` Service for tenant-facing traffic.
3. Per-tenant rate-limit policy defaults captured in `specs/api-gateway-tier-canonical.json`.
4. Helm chart `iac/helm/api-gateway/` ships with Envoy Gateway 1.1 + Coraza + ratelimit-redis sidecar.
5. Per-pack overlay path `iac/kustomize/components/api-gateway-overlay-{kr,eu,us,jp,ksa}/` for sovereign cell variations.

## References

- AWS API Gateway architecture — https://docs.aws.amazon.com/apigateway/
- Stripe edge architecture — Stripe engineering blog "How Stripe handles traffic" (2022).
- Cloudflare API gateway pattern — https://developers.cloudflare.com/api-shield/
- Google Cloud Armor + Cloud Load Balancing — https://cloud.google.com/armor
- Envoy Gateway — https://gateway.envoyproxy.io/
- Kubernetes Gateway API (graduated 2024) — https://gateway-api.sigs.k8s.io/
- Coraza ModSecurity-compatible WAF — https://coraza.io/
- OWASP API Security Top-10 (2023) — https://owasp.org/API-Security/editions/2023/
- OAuth 2.1 + Pushed Authorization Requests (RFC 9126).
- ADR-0001 — cohesion thesis (single canonical edge).
- ADR-0007 — Cedar authorization policy (fine-grained auth stays at workload tier).
- ADR-0009 — cell architecture (per-cell deployment).
- ADR-0049 — cross-region residency (gateway enforces cell-local traffic).
- ADR-0121 — onprem K8s stack (Envoy is already in-stack).
- ADR-0128 — hyperscaler architecture invariants (INV-SHUFFLE-SHARDING is second tier).
- ADR-0145 — inter-µservice communication reform (this ADR is the north-south complement).
- ADR-0148 — Istio service mesh (Envoy Gateway shares Envoy data plane).
