---
id: ADR-0061
status: Superseded
superseded_by: [ADR-709]
doc_status: published
---

# ADR-0061: Application — B2B unified shell with à-la-carte product enablement

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0001, ADR-0002, ADR-0007, ADR-0058, ADR-0059, ADR-0060, ADR-0062

---

## Context

The B2B entry point for Oyatie is a unified shell where tenants sign in and enable products from the flat catalog à-la-carte (like enabling services in an AWS console). This shell was called "Modular Product Shell" in Bominal ADR-0121. Per session decision 2026-05-13, oyatie renames it to **Application** (capital A).

User context: "In essence everything is 'shared'" + Bominal ADR-0121 model (inherited with glossary translation). The Application shell is the entry point for the B2B tenant experience; it is NOT the same as Personal (B2C, person-pillar, separate entry path).

**Naming justification:** "Application" (capital A) is the established term for a unified shell that hosts multiple products. Contrast with lowercase "application" which is the clean-architecture layer name (ADR-0056). Context distinguishes them: capital-A Application = the B2B shell µservice; lowercase application = the use-case layer in BNF v4.1. Per override #8 of ADR-0060.

---

## Decision

We adopt **Application** as the name for the B2B unified shell. Application is a microservice in the flat catalog registered as `application` in `[workspace.metadata.oya.microservices]`.

### Core model

Application implements the Bominal ADR-0121 model (inherited) with glossary translation:

- Tenants sign in via the identity substrate (ADR-0002; Bominal ADR-0123 two-cookie + PKCE + nonce).
- Tenants enable products from the flat catalog à-la-carte.
- Application renders the enabled products' surfaces within a unified shell.
- Products are loaded via the capability registry (ADR-0011); Application does not hardcode any product list.

### Tenant enables products à-la-carte

```rust
// oya-application-product-enablement-kernel
pub struct TenantProductEnablement {
    pub tenant_id: TenantId,
    pub enabled_microservices: BTreeSet<MicroserviceId>,
    pub enablement_timestamp: DateTime<Utc>,
    pub billing_subscription: SubscriptionRef,
}

pub trait ProductEnablementPort {
    fn enable(&self, tenant: TenantId, microservice: MicroserviceId, billing: BillingRef) -> Result<()>;
    fn disable(&self, tenant: TenantId, microservice: MicroserviceId) -> Result<()>;
    fn list_enabled(&self, tenant: TenantId) -> Result<Vec<MicroserviceId>>;
}
```

Enabling a microservice:
1. Validates the tenant has a valid billing subscription for that product.
2. Provisions the per-tenant cell resources for that microservice (via Workflow, ADR-0035).
3. Records the enablement event in the audit chain (ADR-0003).
4. Registers the product's capabilities in the tenant's capability namespace (ADR-0011).

### Personal is a separate entry path

Personal (B2C) does NOT go through Application. It is a separate entry path via the person-pillar (Bominal ADR-0208 Personal context). Application is exclusively the B2B shell.

### Application renders via capability registry

Application does not hardcode any product UI. Each enabled microservice registers its surface via the capability registry (ADR-0011). Application discovers and renders surfaces dynamically. This means adding a new microservice to the catalog does not require an Application code change.

### Auth flow

Per Bominal ADR-0123 (inherited):
- Two-cookie model: session cookie + CSRF cookie.
- PKCE + nonce for OAuth flows.
- Per-tenant SSO federation (SAML2 + OIDC) via identity substrate (ADR-0002).
- Application shell receives the JWT with `tenant_id` claim; Cedar policy gate (ADR-0007) enforces product access per enablement.

---

## Consequences

### Concrete crate layout (BNF v4.1)

```
oya-application-product-enablement-kernel  — enablement types + port traits
oya-application-product-enablement-domain  — enablement business logic
oya-application-product-enablement-adapter — persistence impl
oya-application-product-enablement-rest    — enablement API (tenant admin + billing)
oya-application-shell-kernel               — shell surface types
oya-application-shell-rest                 — shell HTTP API (surface discovery + rendering)
oya-application-auth-kernel                — auth types (two-cookie + PKCE + nonce)
oya-application-auth-adapter               — SSO federation impl (SAML2 + OIDC)
oya-application-navigation-kernel          — navigation graph types
oya-application-app                        — composition-root binary
```

`application` registered in `[workspace.metadata.oya.microservices]`.

### Quality / Performance / Scalability (per ADR-0062)

- **Benchmark target:** AWS Console (product enablement UX parity), Linear (navigation + surface rendering speed).
- **Shell load time p99:** ≤1s cold (TTI); ≤200ms warm (cached surface manifest).
- **Product enablement p99:** ≤5min self-serve SaaS path (per Bominal ADR-0118 inherited target).
- **Auth p99:** ≤200ms for session validation (Cedar policy evaluation on JWT claims).
- **Horizontal scalability:** `oya-application-shell-rest` and `oya-application-auth-adapter` are stateless; state lives only in Postgres (enablement records) + Redis (session cache).
- **Scale target:** 100M+ tenant users via cell architecture; Application shell horizontally scales via Kubernetes HPA.

**Clean architecture lanes enforcing Application shell rules:**

| Lane | What it enforces |
|---|---|
| `oya-shared-architecture-check-cli -- composition-root-only` | Only `oya-application-app` (the `app` layer binary) has unrestricted inward deps; NO business logic in the composition root — only DI assembly + `main()`/runtime startup |
| `oya-shared-architecture-check-cli -- dependency-direction` | Inward-only flow: `oya-application-shell-rest` → `oya-application-product-enablement-application` → `oya-application-product-enablement-domain` → `oya-application-product-enablement-kernel` |
| `oya-shared-architecture-check-cli -- port-location` | Port traits in `oya-application-product-enablement-kernel`; impls in `oya-application-product-enablement-adapter` |
| `oya-shared-architecture-check-cli -- cross-product-refusal` (LEAN-A2) | `oya-application-*` crates MUST NOT import any product crates (`oya-medical-*`, `oya-payments-*`, etc.) directly; product surfaces discovered via capability registry only |
| `oya-check-statelessness-cli` | `oya-application-shell-rest`, `oya-application-auth-adapter`, `oya-application-product-enablement-rest` have zero module-level mutable state |
| `oya-check-shardability-cli` | `oya-application-product-enablement-adapter` declares `tenant_id` partition key on enablement table |

Composition-root rule per `[[feedback-clean-architecture-requirements]]` §9: `app` layer is the ONLY layer with unrestricted inward dependencies. `application` (use-case layer) and `app` (binary) are NOT interchangeable.
Hexagonal standard per Bominal ADR-0101 (inherited). Domain naming per Bominal ADR-0125 (inherited).

### Positive

- Tenants get a single entry point; no product-specific login flows.
- New microservices join the catalog without Application code changes (capability registry).
- Unified auth (two-cookie + PKCE) makes cross-product SSO seamless.

### Negative

- Application shell becomes a hot path for all B2B tenant interactions; requires careful horizontal scaling.
- Dynamic surface discovery via capability registry adds a runtime lookup; mitigated by surface manifest caching.

---

## Related

- ADR-0001 (cohesion — Application is the B2B entry point in the flat catalog)
- ADR-0002 (identity substrate — auth)
- ADR-0007 (Cedar — product access gating per enablement)
- ADR-0011 (capability registry — dynamic surface discovery)
- ADR-0035 (Workflow — enablement provisioning flows)
- ADR-0058 (Flat microservice catalog — Application enables products from the catalog)
- ADR-0060 (Bominal-inheritance — ADR-0121 inherited with Shell→Application translation)
- ADR-0062 (Quality/Performance/Scalability bar)
- `[[feedback-flat-product-catalog]]` — Application (B2B unified shell) in canonical architecture
- `[[feedback-bominal-inheritance-precedence]]` — override #8 (Shell → Application)
- Bominal ADR-0121 (Modular Product Shell = Application in oyatie glossary)
- Bominal ADR-0123 (cross-product auth cookie + redirect contract, inherited)
