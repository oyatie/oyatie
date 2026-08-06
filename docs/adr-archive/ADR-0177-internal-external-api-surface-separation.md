---
id: ADR-0177
status: Superseded
date: 2026-05-18
owners:
  - council-architecture
  - platform-api-sdk
  - ops-security
supersedes: []
superseded_by: [ADR-0701]
related:
  - ADR-0037-public-api-stability-tiers-and-deprecation.md
  - ADR-0157-api-gateway-tier.md
  - ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md
  - ADR-0011-cross-microservice-contract-registry.md
doc_class: Architecture-Decision-Record
purpose: >
  Split the public API surface into two gateway tiers — public
  (`api.oyatie.com`) and internal (`internal-api.oyatie.com`). Public is
  semver-stable per ADR-0037, rate-limited per public key, fully
  documented. Internal is mesh-mTLS-only, semver waived, higher rate
  limits, no external customer exposure.
enforcement_status: advisory-until-gateway-tier-deployed
enforced_by: oya gate validate api-surface-classification
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0177: Internal vs external API surface separation

## Status

Accepted — 2026-05-18. Enforcement is advisory until the two gateway
tiers are deployed and every public OpenAPI route carries the new
`api_surface` field.

## Context

ADR-0157 establishes the API gateway tier. ADR-0037 establishes API
stability tiers (Public-Stable / Public-Preview / Internal /
Experimental). ADR-0011 establishes the cross-microservice contract
registry. But the portfolio still runs every routed call through a
single gateway, and the OpenAPI catalogue blends external-customer
routes with µservice-to-µservice routes. Consequences:

- An internal-only change to a µservice-to-µservice route triggers the
  full external-customer change-management cadence (deprecation
  windows, customer migration runbooks) because the gateway sees both
  surfaces as "public".
- Public-customer rate limits apply to internal traffic, occasionally
  starving cross-µservice flows during a traffic spike.
- The mesh ingress (Cilium per ADR-0148) cannot distinguish "internal
  call from another µservice" from "external customer call".
- Security review (council-privacy, ops-security) cannot scope its
  pen-test surface cleanly.

The Stripe pattern (api.stripe.com vs internal-api.stripe.com), the
AWS pattern (api.aws.amazon.com customer-facing vs control-plane
internal endpoints), and the Google Cloud pattern (cloudresourcemanager
public vs cellservices internal) all separate the two surfaces at the
gateway tier. This ADR adopts the same separation.

## Decision

### D-1. Two gateway tiers

| Tier | Hostname | Audience | Stability tier | Auth | Rate limit |
| --- | --- | --- | --- | --- | --- |
| **Public** | `api.oyatie.com` | External customers, public-SDK consumers, partners | `Public-Stable` or `Public-Preview` per ADR-0037 | OAuth 2.0 + per-key signature | Per-public-key, per-IP (ADR-0178) |
| **Internal** | `internal-api.oyatie.com` | Other Oyatie µservices only; mesh-internal | `Internal` (semver waived) | Mesh mTLS only (SPIFFE id) | 10× public budget, per-µservice |

### D-2. Routing classification

Every OpenAPI route declares `api_surface: public | internal` in its
spec metadata. The gateway tier reads this classification and routes
to the appropriate hostname. Mis-routed requests (external client
hitting an internal-classified route) receive HTTP 404 (the route is
*not visible* outside the internal mesh).

### D-3. Semver discipline split

Public-surface changes follow ADR-0037 in full: semver, deprecation
window, customer migration runbook, public changelog entry.

Internal-surface changes:

- Semver tag is OPTIONAL.
- Deprecation window = next deployment + 7 days (vs ADR-0037's 90 days
  for Public-Stable).
- No customer migration runbook required.
- Changelog rolled up into the relevant ChangeSet (ADR-0110).

Promoting an internal route to public requires an ADR amendment plus
the full ADR-0037 surface (semver, docs, deprecation timeline).

### D-4. Ingress isolation

The mesh ingress (Cilium L7 per ADR-0148):

- `api.oyatie.com` ingress accepts requests from any source IP; OAuth
  + per-key signature mandatory.
- `internal-api.oyatie.com` ingress accepts requests only from
  intra-mesh SPIFFE ids; the hostname's public DNS record points to
  internal load balancers that drop non-mesh traffic at L4.

### D-5. Observability split

Two dashboards: `microservices/observability/dashboards/api-public.md`
and `microservices/observability/dashboards/api-internal.md`. The
public dashboard is the surface ops-sre-reliability triages against
customer-impacting incidents; the internal dashboard is the
cross-µservice flow surface.

### D-6. SDK generation

- Public SDK (the `oya-platform-api-sdk-*` family per ADR-0036) is
  generated *only* from public-classified routes.
- Internal SDK (the `oya-*-internal-sdk-*` family) is generated from
  internal-classified routes; built into Rust workspace members
  directly.

### D-7. Documentation split

`docs/products/*/prd.md` references public-surface routes only.
Internal routes live in `microservices/<ms>/contracts/internal/`.
The doc-portal (ADR-0066 live code introspection) renders both with
clear "Public" vs "Internal" badges.

## Alternatives considered

### Alt-1. Single gateway, route-level annotations

Keep one gateway; annotate each route as public or internal but route
through the same hostname. **Rejected.** Customers can still discover
internal routes via the OpenAPI catalogue. Rate limits remain coupled.
Security review surface remains uncleanable.

### Alt-2. Hostname-per-µservice

Each µservice gets its own subdomain. **Rejected.** Customers face a
fragmented surface (workflow.oyatie.com vs cloud.oyatie.com vs
search.oyatie.com); cross-product flows require multi-host knowledge;
defeats the cohesion thesis (ADR-0001).

### Alt-3. Hostname-per-product-bundle

Bundle by product (workspace.oyatie.com vs cloud.oyatie.com). **Rejected.**
The "no bundle" decision (ADR-0132) and flat-catalog thesis (ADR-0001,
ADR-0058) already established that the product surface is flat.

## Consequences

### C-1. Positive

- **Public surface is small, stable, semver-disciplined.** Customers
  see only routes that carry the ADR-0037 public guarantees.
- **Internal flows iterate faster.** A µservice-to-µservice contract
  change is a one-week internal deploy, not a 90-day external
  deprecation.
- **Security review is scopable.** Pen-test the public hostname
  externally; internal hostname only via authenticated mesh.
- **Rate-limit budgets decoupled.** External traffic spike doesn't
  starve cross-µservice flows.
- **Hyperscaler-grade.** Matches Stripe + AWS + Google Cloud
  separation.

### C-2. Negative

- **Two gateways = more operational surface.** Mitigation: same Helm
  chart parameterized by `api_surface`; same observability dashboards
  templated.
- **OpenAPI catalogue split.** Mitigation: doc-portal renders both
  with badges; the SDK generator picks the right subset per audience.
- **Internal SDK consumers must build per-deploy.** Mitigation:
  internal SDK is workspace-member Rust crates; built natively per
  `cargo build`.

### C-3. Sustainability

- Internal-surface evolution is essentially zero-cost-of-change once
  the surface is wired; the gating on cross-µservice contract churn
  drops dramatically.

## Implementation surface

- `specs/api-surface-separation.json` — canonical surface enum +
  per-surface policy.
- `docs/standards/api-surface-separation.md` — full standards doc.
- `microservices/observability/dashboards/api-public.md` and
  `api-internal.md` — dashboard schemas.
- `microservices/cloud-iac/iac/k8s/gateway/` — two gateway Deployments
  (templated).
- Existing validator extension: `oya-check-openapi-rest-route-parity`
  consumes the new `api_surface` field and enforces hostname pinning.
- New validator lane `api-surface-classification` added to
  `AGGREGATED_VALIDATE_LANES` (advisory).

## References

- Stripe Engineering — *Designing robust and predictable APIs with
  idempotency* (the API surface separation is documented in their
  internal handbook excerpt published as a public blog).
- AWS Architecture — *Control plane vs data plane in service design*.
- Google Cloud — *Public APIs vs internal endpoints* (cloudresourcemanager
  vs internal cellservices, public docs 2022–2024).
- ADR-0037 (this portfolio) — public API stability tiers + deprecation.
- ADR-0157 (this portfolio) — API gateway tier.
- ADR-0011 (this portfolio) — cross-µservice contract registry.
