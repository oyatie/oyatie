---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P19-application
status: Proposed
acceptance_lanes: []
entry_gate: 'M02/P02-ontology complete; M02/P12-workflow-engine complete; M02/P03-identity

  complete; M02/P13-tenancy complete (TenantProductRegistry live); M02/P14-policy

  complete (Cedar engine live); cargo check clean across workspace; grit done on

  all P02/P03/P12/P13/P14 symbols; ICM phase-handoff rows emitted.

  '
exit_gate: 'All P19 impl-plan acceptance gates green; 5 BCs registered (product-enablement,

  tenant-navigation, branding, product-launchpad, sso-hub); Leptos web shell

  compiles and serves; per-tenant product catalog rendered from TenantProductRegistry;

  SSO hub wired per Bominal ADR-0123; all crates pass cargo check/build/clippy/

  nextest/deny; oya gate validate lean-a1/a2/a3/a4 exit 0; grit done on all P19

  symbols; ICM phase-complete row emitted.

  '
depends_on:
- milestone: M02
  phase: P02-ontology
  reason: Application shell reads Ontology object types to build product navigation
    entries; tenant dashboard state stored as Ontology objects.
- milestone: M02
  phase: P12-workflow-engine
  reason: Application shell integrates workflow run status into tenant navigation
    (active workflows visible from shell).
- milestone: M02
  phase: P03-identity
  reason: SSO hub wired through identity SessionStore + AuthChallenger ports; per-tenant
    admin user management.
- milestone: M02
  phase: P13-tenancy
  reason: Product-enablement BC reads TenantProductRegistry to build the per-tenant
    product catalog; branding BC reads tenant tier.
- milestone: M02
  phase: P14-policy
  reason: Every shell navigation action authorized through PolicyEvaluator; product-launchpad
    respects Cedar allow/deny per principal.
owner_team: council-architecture
purpose: "Delivers the Application B2B unified shell substrate: the Leptos web shell that serves as the single entry point for all B2B tenants."
---
# P19-application: Application B2B Unified Shell Substrate — Product-Enablement + Tenant Navigation + Branding + Launchpad + SSO Hub

## Purpose

Delivers the Application B2B unified shell substrate: the Leptos web shell that serves as
the single entry point for all B2B tenants. Per Bominal ADR-0121 (modular product shell,
translated Shell → Application per oyatie override #8), the Application shell renders a
per-tenant product catalog, handles SSO across all enabled products, and enforces Cedar
authz on every navigation action.

Five bounded contexts: `product-enablement` (reads TenantProductRegistry; renders enabled
products à-la-carte like AWS console), `tenant-navigation` (per-tenant left-nav + breadcrumb
state; active workflow indicators), `branding` (per-tenant theme + logo + color tokens),
`product-launchpad` (deep-link routing into each product's sub-domain per
`app.oyatie.com/<microservice>/...`), `sso-hub` (two-cookie + PKCE + nonce SSO contract
per Bominal ADR-0123; unified `auth.oyatie.com` → per-product redirect).

Leptos (Rust web framework, full-stack SSR + WASM) is the chosen client technology per
Bominal ADR-0209, translated to oyatie. The Application shell is the only Leptos surface
in M02; product UIs ship in M03+.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `application` | `product-enablement` | `crates/oya-application-product-enablement-kernel/` | `oya-application-product-enablement-kernel` |
| `application` | `product-enablement` | `crates/oya-application-product-enablement-application/` | `oya-application-product-enablement-application` |
| `application` | `product-enablement` | `crates/oya-application-product-enablement-adapter/` | `oya-application-product-enablement-adapter` |
| `application` | `product-enablement` | `crates/oya-application-product-enablement-rest/` | `oya-application-product-enablement-rest` |
| `application` | `tenant-navigation` | `crates/oya-application-tenant-navigation-kernel/` | `oya-application-tenant-navigation-kernel` |
| `application` | `tenant-navigation` | `crates/oya-application-tenant-navigation-adapter/` | `oya-application-tenant-navigation-adapter` |
| `application` | `branding` | `crates/oya-application-branding-kernel/` | `oya-application-branding-kernel` |
| `application` | `branding` | `crates/oya-application-branding-adapter/` | `oya-application-branding-adapter` |
| `application` | `product-launchpad` | `crates/oya-application-product-launchpad-kernel/` | `oya-application-product-launchpad-kernel` |
| `application` | `product-launchpad` | `crates/oya-application-product-launchpad-application/` | `oya-application-product-launchpad-application` |
| `application` | `sso-hub` | `crates/oya-application-sso-hub-kernel/` | `oya-application-sso-hub-kernel` |
| `application` | `sso-hub` | `crates/oya-application-sso-hub-adapter/` | `oya-application-sso-hub-adapter` |
| `application` | all | `crates/oya-application-app/` | `oya-application-app` |
| `application` | all | `contracts/application.openapi.yaml` | — |
| `application` | all | `migrations/application/V001__application_schema.sql` | — |

Naming justification:

```
NAME: oya-application-product-enablement-rest
JUSTIFICATION:
- microservice = application: the B2B unified shell µservice; oyatie override #8
  (Shell → Application); registered in [workspace.metadata.oya.microservices];
  ADR-0056 v4.1
- bc-tokens = product-enablement: the product-catalog / enable-disable BC; distinct
  from tenant-navigation / branding / product-launchpad / sso-hub BCs at same layer
- layer = rest: HTTP REST handlers; axum router; presentation-only; depends on
  application + kernel; no direct adapter import; ADR-0056 §"Layer semantics"
- exemptions claimed: none
```

### Out-of-scope

- Per-product UI pages (HR dashboard, Payroll module) — deferred to M03
- Connect Personal B2C entry path — deferred to M04+ (different shell, no B2B Application)
- Per-tenant SAML/OIDC IdP config — deferred to M03
- Mobile/native clients (5 native + SvelteKit prototype) — deferred to M03+ per ADR-0209

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-application-kernel-scaffold.md`](IP-001-application-kernel-scaffold.md) | Scaffold all 13 application crates; 5 BC kernels; DDL; port traits | pending | `council-architecture` |
| [`IP-002-application-leptos-shell.md`](IP-002-application-leptos-shell.md) | Leptos SSR + WASM shell; product catalog page; tenant navigation sidebar | pending | `council-architecture` |
| [`IP-003-application-sso-hub.md`](IP-003-application-sso-hub.md) | SSO hub: two-cookie + PKCE + nonce; auth.oyatie.com redirect; ADR-0123 | pending | `council-architecture` |
| [`IP-004-application-load-tests.md`](IP-004-application-load-tests.md) | k6 load tests; shell page load p99 ≤200ms; SSO redirect p99 ≤100ms | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P19-application
oya gate validate lean-a2 --phase P19-application
oya gate validate lean-a3 --phase P19-application
oya gate validate lean-a4 --phase P19-application
```

### Application-specific gates

```bash
# Leptos shell serves product catalog for a test tenant
cargo nextest run -p oya-application-app --test shell_product_catalog  # exit 0
# SSO hub PKCE + nonce flow passes
cargo nextest run -p oya-application-sso-hub-adapter --test sso_pkce_flow  # exit 0
# Cedar authz gate on navigation
cargo nextest run -p oya-application-product-launchpad-application --test cedar_nav_gate  # exit 0
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-application-product-enablement-kernel` | `kernel` | Yes — ProductCatalogPort | N/A |
| `oya-application-product-enablement-application` | `application` | N/A | N/A |
| `oya-application-product-enablement-adapter` | `adapter` | N/A | Yes — TenancyProductCatalogAdapter |
| `oya-application-product-enablement-rest` | `rest` | N/A | No direct adapter import |
| `oya-application-tenant-navigation-kernel` | `kernel` | Yes — NavigationStateStore | N/A |
| `oya-application-tenant-navigation-adapter` | `adapter` | N/A | Yes — PgNavigationStateAdapter |
| `oya-application-branding-kernel` | `kernel` | Yes — BrandingStore | N/A |
| `oya-application-branding-adapter` | `adapter` | N/A | Yes — PgBrandingAdapter |
| `oya-application-product-launchpad-kernel` | `kernel` | Yes — LaunchpadRoutingPort | N/A |
| `oya-application-product-launchpad-application` | `application` | N/A | N/A |
| `oya-application-sso-hub-kernel` | `kernel` | Yes — SsoSessionPort, OAuthStateStore | N/A |
| `oya-application-sso-hub-adapter` | `adapter` | N/A | Yes — PkceOAuthAdapter |
| `oya-application-app` | `app` | N/A | Unrestricted inward |

### Port traits declared in kernel

```rust
// oya-application-product-enablement-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait ProductCatalogPort: Send + Sync + sealed::Sealed {
    async fn enabled_products(&self, tenant_id: TenantId) -> Result<Vec<ProductEntry>, ApplicationError>;
    async fn product_metadata(&self, microservice: &str) -> Result<ProductMetadata, ApplicationError>;
}

// oya-application-sso-hub-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait SsoSessionPort: Send + Sync + sealed::Sealed {
    async fn initiate_pkce(&self, tenant_id: TenantId, redirect_uri: &str) -> Result<PkceChallenge, ApplicationError>;
    async fn exchange_code(&self, code: &str, verifier: &str, state: &str) -> Result<SsoTokens, ApplicationError>;
    async fn validate_tokens(&self, tokens: &SsoTokens) -> Result<SsoSession, ApplicationError>;
}
```

### CI lanes that must green

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P19-application` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P19-application` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P19-application` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P19-application` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P19-application` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `application-product-enablement` | `application` | pending |
| `application-tenant-navigation` | `application` | pending |
| `application-branding` | `application` | pending |
| `application-product-launchpad` | `application` | pending |
| `application-sso-hub` | `application` | pending |

---

## Grit Claim Symbols

```
crates/oya-application-product-enablement-kernel/src/lib.rs::ProductCatalogPort
crates/oya-application-sso-hub-kernel/src/lib.rs::SsoSessionPort
crates/oya-application-branding-kernel/src/lib.rs::BrandingStore
crates/oya-application-tenant-navigation-kernel/src/lib.rs::NavigationStateStore
contracts/application.openapi.yaml::getProductCatalog
migrations/application/V001__application_schema.sql::application.tenant_dashboards
```

TTL: `--ttl 3600`. Fallback: ICM `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P19-application started; Leptos B2B shell; 5 BCs; SSO hub ADR-0123; depends P02/P03/P12/P13/P14" \
  -i high \
  -k "M02,P19,phase-start,application"

icm store \
  -t context-oyatie \
  -c "Phase P19-application complete; Leptos shell deployable; product catalog live; SSO hub PKCE; Cedar nav gate; next: P20-ci-lanes" \
  -i high \
  -k "M02,P19,phase-complete,application"
```

---

## References

- Bominal ADRs inherited: ADR-0121 (modular product shell), ADR-0123 (cross-product auth cookie), ADR-0209 (Leptos client)
- oyatie override #8: Shell → Application
- oyatie ADRs cited: ADR-0056 v4.1
- M02-substrate-schema-foundation §6-N (application outlined)
