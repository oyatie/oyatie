---
id: ADR-0182
status: Superseded
deciders: council-architecture, ops-sre-reliability, ops-security, axis-cloud-k8s, axis-network
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
amended_by: [ADR-0632]
related: [ADR-0121, ADR-0145, ADR-0148, ADR-0150, ADR-0157, ADR-0183, ADR-0184, ADR-0185, ADR-0186, ADR-0203, ADR-0258, ADR-0632]
last_reconciled: 2026-08-01
reconciled_with: [ADR-0203, ADR-0258, ADR-0632]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0182 — API Gateway (north-south) vs Service Mesh (east-west) separation; zero overlap

## Status

Accepted (2026-05-18). Mandates a clean separation between **north-south** (public ingress → cluster) and **east-west** (intra-cluster service-to-service) traffic concerns, with each direction owned by exactly one substrate and zero feature overlap.

## ADR-0632 product-protocol reconciliation

The north-south public boundary **MUST** expose HTTPS REST/OpenAPI 3.2.0, signed/versioned
webhooks, AsyncAPI/CloudEvents events, SSE by default for one-way streaming, and WebSocket only for
bidirectional sessions. It **MUST NOT** expose GraphQL, gRPC, gRPC-Web, or Connect. East-west typed
RPC remains internal-only gRPC/proto3 over HTTP/2 and never becomes a gateway public contract.

### Public-contract reconciliation

Per ADR-0203 and ADR-0258, public contract carriers are REST documented by OpenAPI 3.2 plus
webhooks, events, and streams documented by AsyncAPI 3.1. gRPC over HTTP/2 (H2) with proto3 is
internal-only service-to-service traffic under mTLS; it is not a public API contract.


## Context

ADR-0148 sets the canonical service-mesh substrate (Cilium L3/L4 + Istio Ambient L7, layered globally) for east-west traffic. ADR-0157 promotes a dedicated `api-gateway` µservice for north-south ingress.

The hyperscaler bar:

- **Consistency** — public ingress and internal mesh are configured by different tools with different responsibilities; mixing them creates "where does this policy live?" ambiguity at scale.
- **Quality** — public TLS, WAF, OIDC, public rate limiting are first-class gateway concerns; mTLS, service identity, AuthorizationPolicy are first-class mesh concerns.
- **Scalability** — the gateway scales with public traffic; the mesh scales with internal call fan-out.
- **Maintainability** — each substrate has its own control plane, its own CRDs, its own rollback path.
- **Integration** — Gateway API v1.0 is the open standard for the gateway tier; AuthorizationPolicy v1 is the open standard for mesh L7.

Anti-patterns this ADR forecloses:

1. Using Istio Gateway controller to terminate public TLS — couples gateway lifecycle to istiod; ties public ingress evolution to mesh control-plane releases; muddies the layer boundary.
2. Using a single edge proxy (e.g. AWS ALB) for both public ingress AND inter-µservice routing — vendor lock-in, mixed concerns, no mTLS at the internal hop.

## Decision

Oyatie adopts a **two-substrate ingress model** with zero feature overlap:

### North-south (public → cluster): Envoy Gateway 1.8.0

The canonical north-south substrate is **Envoy Gateway 1.8.0** (CNCF; Kubernetes Gateway API v1.0 conformant; vendor-neutral; deployed as a dedicated `api-gateway` µservice per ADR-0157).

Envoy Gateway owns:

- **TLS termination** at the public edge (TLS 1.3; per-FQDN SNI; cert-manager-rotated via ACME or per-tenant key-custody-BYOK certs).
- **Public rate limiting** (per-tenant + per-IP token bucket; Redis-cluster-backed counters per ADR-0184 storage tier 3 — note: Valkey 8.1 since Redis 7.4+ relicensed; see ADR-0184).
- **WAF** via **Coraza WASM filter** (ModSecurity-compatible; OWASP CRS rule sets).
- **OIDC** for public-facing user authentication (workforce IdP + customer IdP per tenant per ADR-0163).
- **mTLS to upstream µservices** — gateway terminates client TLS and re-originates to upstream Istio Ambient ztunnel via mTLS using gateway's SPIFFE-ID.
- **Schema enforcement** — OpenAPI 3.2.0 schema registered with the gateway; requests violating contract are rejected at the edge.
- **DDoS protection** — Envoy connection limits + Coraza rate-based rules + Cilium XDP at the underlying nodes (XDP is mesh-tier per ADR-0148, but the gateway sits on the same Cilium-hosted nodes so DDoS protection composes).

Envoy Gateway never processes east-west traffic. Internal µservice-to-µservice calls bypass the gateway entirely.

### East-west (intra-cluster): Cilium + Istio Ambient (ADR-0148)

The canonical east-west substrate is per ADR-0148 (Cilium L3/L4 + Istio Ambient L7). The mesh never terminates public TLS; public traffic must traverse Envoy Gateway first.

## Alternatives considered

### (a) Istio Gateway controller for north-south — REJECTED

- **Pros:** single mesh project; gateway CRDs and mesh CRDs are conceptually unified.
- **Cons:** couples public ingress evolution to istiod control-plane releases; istiod outage takes both gateway and mesh down simultaneously; layer boundary erodes; "gateway is part of the mesh" framing invites putting east-west policy on the gateway over time.
- **Rejected**: violates separation-of-concerns invariant.

### (b) AWS ALB + AWS WAF — REJECTED

- **Pros:** managed; AWS-native; deep AWS IAM integration.
- **Cons:** AWS-specific (oyatie ships in EU + KR + on-prem cells where AWS-native is not the default); vendor lock-in conflicts with ADR-0121 portability invariant; Gateway API conformance is partial.
- **Rejected**: lock-in.

### (c) Kong Gateway — REJECTED

- **Pros:** mature; OpenAPI plugin ecosystem; enterprise vendor support.
- **Cons:** Kong's data plane (kong-proxy / OpenResty) is non-Gateway-API-native (Kong has a Gateway API controller but the canonical Kong config language is bespoke); plugin model is Lua-first which mismatches oyatie's Rust+WASM preference; community vs Enterprise feature split.
- **Rejected**: non-Gateway-API-native; Lua plugin language mismatch.

### (d) Traefik — REJECTED

- **Pros:** simple operational model; native Gateway API conformance; good ergonomics.
- **Cons:** weaker WAF — no first-class Coraza/ModSecurity integration; smaller ecosystem at hyperscaler scale; smaller pool of OIDC integration battle-testing.
- **Rejected**: weaker WAF; smaller scale-tested deployment surface.

### (e) NGINX Ingress / NGINX Gateway Fabric — REJECTED

- **Pros:** widest deployment footprint; mature.
- **Cons:** F5 acquired NGINX; commercial pressure on OSS roadmap; ModSecurity 3.x plugin is deprecated; Gateway API conformance was retrofit.
- **Rejected**: ecosystem direction risk.

### (f) No dedicated gateway (expose mesh ingress directly) — REJECTED

- **Pros:** zero gateway-tier complexity.
- **Cons:** exposes mesh ingress publicly; public TLS termination becomes mesh concern; public rate limiting collides with internal rate limiting; the layer boundary collapses; WAF has no clean home.
- **Rejected**: cannot meet the consistency invariant.

### (g) **CHOSEN: Envoy Gateway for north-south + Cilium + Istio Ambient for east-west, ZERO overlap**

- **Pros:**
  - Each direction owned by exactly one substrate.
  - Gateway API v1.0 (open standard) for north-south.
  - AuthorizationPolicy v1 (open standard, ADR-0148) for east-west L7.
  - Envoy Gateway is CNCF + vendor-neutral; works on every K8s distro oyatie targets.
  - Coraza WASM filter for WAF integrates as a first-class Envoy extension.
  - Independent rollback for each substrate.
- **Cons:** two control planes to operate (Envoy Gateway controller + istiod). Mitigation: both are CNCF projects with similar operator skill profile; both are Helm-deployed; ops-sre-reliability operates both via Flux.
- **Accepted**.

## Consequences

### Positive

1. **Crisp ownership boundary.** Any policy question routes to one substrate: "is this public traffic?" → Envoy Gateway. "Is this internal?" → Cilium + Istio Ambient. No ambiguity.
2. **Independent evolution.** Gateway API spec evolves on its own cadence; AuthorizationPolicy spec evolves on its own cadence. Neither change forces the other.
3. **WAF + OIDC + public rate limit are gateway concerns; mTLS + AuthorizationPolicy + traceparent injection are mesh concerns.** Each concern has a single canonical home.
4. **Vendor neutrality.** Envoy Gateway (CNCF) + Cilium (CNCF) + Istio (CNCF). All three are Linux Foundation graduated projects.
5. **Coraza WASM** integrates as a first-class Envoy filter and shares operator skill with the mesh waypoint Envoys; one Envoy-debug runbook spans both substrates.

### Negative

1. **Two control planes to learn.** Envoy Gateway control plane + istiod. Mitigation: same Envoy data plane shape under the hood; same Helm operational model; same CNCF release cadence.
2. **Cross-substrate observability requires deliberate wiring.** Public flow that enters via Envoy Gateway and traverses the mesh is two trace-spans glued together by `traceparent` propagation. Mitigation: per-µservice tracing-client-kernel injects `traceparent` at every hop; ADR-0153 observability backplane stitches the spans.

### Operational

1. `api-gateway` µservice (per ADR-0157) deploys Envoy Gateway 1.8.0 via Helm.
2. North-south Gateway resources live at `microservices/api-gateway/iac/helm/api-gateway/templates/gateway.yaml`; HTTPRoute resources per public endpoint live at `microservices/api-gateway/iac/helm/api-gateway/templates/httproute-<surface>.yaml`.
3. The Cedar policy compiler does NOT emit policies for north-south. Public-facing authorization is a Coraza WAF rule + OIDC scope check + per-tenant rate-limit policy at the gateway tier; Cedar enforcement begins at the mesh waypoint inside the cluster.
4. Per ADR-0148, the mesh waypoint's `ext_authz` calls Cedar PDP for east-west; the gateway tier authenticates the public principal via OIDC and emits a signed JWT carrying the principal identity to the mesh, where the waypoint Cedar PDP authorizes the now-authenticated principal's east-west actions.

## In-house roadmap

Per user directive 2026-05-18 (in-house-stack policy), this ADR's components classify as follows:

| Component | Classification | Rationale | In-house Phase 2 plan |
|---|---|---|---|
| **Envoy Gateway 1.8.0** | KEEP (CNCF; Gateway API v1.0 conformant) | THE standard Gateway API control plane on THE standard data plane (Envoy). | None planned. Adapter at `crates/oya-shared-gateway-northsouth-kernel` wraps Envoy Gateway for theoretical swap. |
| **Envoy data plane** | KEEP (CNCF Graduated 2018) | THE standard L7 proxy at hyperscaler scale. Adobe, Lyft, AWS App Mesh, Istio, Anthos all run Envoy. | None planned. |
| **Coraza WASM WAF** | KEEP (open-source, OWASP CRS compatible) | THE standard open-source ModSecurity-compatible engine for Envoy WASM filter. | None planned. |
| **Kubernetes Gateway API v1.0** | KEEP (Kubernetes SIG-Network upstream) | THE Kubernetes-standard API surface for north-south. | None planned. |
| **OIDC** | KEEP (open standard via OpenID Foundation) | Industry-standard public auth protocol. | None planned. |
| **WebAuthn / FIDO2** | KEEP (W3C + FIDO Alliance) | Industry-standard passkey protocol. | None planned. |

Why no in-house gateway: Envoy is what AWS App Mesh, GCP Anthos, Solo.io, and Cloudflare's edge use under the hood. Building an Oya-native gateway proxy would reimplement Envoy with a smaller adoption surface; the engineering cost would not produce a better outcome. The Envoy Gateway control plane delivers Gateway-API-conformant config evolution; building an Oya-native control plane would fork from the standard.

The cell-µservice's per-tenant rate-limit counters (Tier-3 Valkey per ADR-0184) are Oya-native logic running on a KEEP-classified backend; the rate-limit policy compiler is part of the governance µservice's Cedar compiler emit path (per ADR-0183), which is itself in-house code on a KEEP-classified policy engine. This is the AWS/Google/Microsoft/Oracle pattern: standard engines, in-house policy + product assets.

## Rollback

Each substrate rolls back independently:

- **North-south rollback:** drop the Envoy Gateway Helm release; restore the prior ingress controller (if any). Mesh continues to operate east-west.
- **East-west rollback:** per ADR-0148's tier-by-tier rollback. Gateway continues to operate public ingress.

`git revert` of the relevant Helm values change followed by Flux reconciliation. No persisted state is lost.

## References

- ADR-0121 — on-prem K8s stack.
- ADR-0145 — inter-microservice communication reform.
- ADR-0148 — service-mesh canonical (Cilium L3/L4 + Istio Ambient L7 layered).
- ADR-0150 — policy engine separation (Cedar app authz vs Kyverno admission).
- ADR-0157 — dedicated API gateway tier (this ADR confirms Envoy Gateway as the data plane and refines the layer boundary).
- ADR-0183 — Kubernetes policy engine separation.
- ADR-0184 — storage tier layering.
- ADR-0185 — Workflow Studio client stack.
- ADR-0186 — observability backplane layering.
- Envoy Gateway — https://gateway.envoyproxy.io/ ; v1.8.0 May 2026.
- Kubernetes Gateway API v1.0 — https://gateway-api.sigs.k8s.io/
- Coraza WASM filter — https://coraza.io/
- CNCF Graduated projects — Envoy (2018), Cilium (2023), Istio (2024).
- LTS-rotation cadence: versions current as of 2026-05-18; review per ADR-0098 (LTS pin policy).
