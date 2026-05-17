---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P19-application
impl_plan_id: IP-001-application-kernel-scaffold
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P13-tenancy/IP-001
  reason: TenancyProductCatalogAdapter reads TenantProductRegistry from oya-tenancy-kernel
- impl_plan: P03-identity/IP-001
  reason: SsoSessionPort uses identity SessionStore + AuthChallenger ports
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: Scaffolds all 13 application crates across 5 BCs, implements the Leptos SSR + WASM composition-root shell, wires the per-tenant product catalog from TenantProductRegistry, implements the SSO hub PKCE + nonce flow per Bominal ADR-0123.
execution_variant: merge-into-existing-crates
decided_at: "2026-05-17"
decided_by: user-directive-option-2
execution_variant_note: "Delta-1 merges ProductId, ProductMetadata, ProductEntry, and ProductCatalogError into existing oya-application-app via new product_catalog.rs module. No new crates scaffolded. References F-M02B-PLAN-LIVE-CRATE-RECONCILIATION (filed P04). Adds 8 unit tests covering ProductId rejection/roundtrip, ProductMetadata validation, ProductEntry is_active/deep_link_path, enabled-product filtering, and ProductId ordering."
---
# IP-001-application-kernel-scaffold: Scaffold Application Shell — 13 Crates / 5 BCs + Leptos Shell + SSO Hub + DDL

## Intent

Scaffolds all 13 application crates across 5 BCs, implements the Leptos SSR + WASM
composition-root shell, wires the per-tenant product catalog from TenantProductRegistry,
implements the SSO hub PKCE + nonce flow per Bominal ADR-0123, and authors the full DDL.
After this IP merges, the Application B2B shell is deployable: a tenant signs in at
`app.oyatie.com`, sees their enabled products, and navigates to each product's subdomain
through the launchpad.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add 13 application workspace members; add leptos = { version = "0.6", features = ["ssr"] } to workspace deps |
| `crates/oya-application-product-enablement-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-application-product-enablement-kernel/src/lib.rs` | create | ProductCatalogPort; ProductEntry + ProductMetadata types |
| `crates/oya-application-product-enablement-application/Cargo.toml` | create | Depends on kernel only |
| `crates/oya-application-product-enablement-application/src/lib.rs` | create | GetEnabledProductsUseCase; GetProductMetadataUseCase |
| `crates/oya-application-product-enablement-adapter/Cargo.toml` | create | Depends on application + kernel + oya-tenancy-kernel |
| `crates/oya-application-product-enablement-adapter/src/lib.rs` | create | TenancyProductCatalogAdapter: impl ProductCatalogPort; reads TenantProductRegistry |
| `crates/oya-application-product-enablement-rest/Cargo.toml` | create | axum; depends on application + kernel |
| `crates/oya-application-product-enablement-rest/src/lib.rs` | create | GET /api/v1/catalog, GET /api/v1/catalog/{microservice} |
| `crates/oya-application-tenant-navigation-kernel/Cargo.toml` | create | NavigationStateStore port; NavState + NavEntry types |
| `crates/oya-application-tenant-navigation-kernel/src/lib.rs` | create | Sealed port |
| `crates/oya-application-tenant-navigation-adapter/Cargo.toml` | create | Depends on navigation-kernel + sqlx |
| `crates/oya-application-tenant-navigation-adapter/src/lib.rs` | create | PgNavigationStateAdapter |
| `crates/oya-application-branding-kernel/Cargo.toml` | create | BrandingStore port; TenantBranding + ThemeTokens types |
| `crates/oya-application-branding-kernel/src/lib.rs` | create | Sealed port |
| `crates/oya-application-branding-adapter/Cargo.toml` | create | Depends on branding-kernel + sqlx |
| `crates/oya-application-branding-adapter/src/lib.rs` | create | PgBrandingAdapter |
| `crates/oya-application-product-launchpad-kernel/Cargo.toml` | create | LaunchpadRoutingPort port; ProductRoute type |
| `crates/oya-application-product-launchpad-kernel/src/lib.rs` | create | Sealed port + deep-link URL generation |
| `crates/oya-application-product-launchpad-application/Cargo.toml` | create | Depends on launchpad-kernel + policy-engine-kernel |
| `crates/oya-application-product-launchpad-application/src/lib.rs` | create | GetProductRouteUseCase: Cedar-authorized deep-link generation |
| `crates/oya-application-sso-hub-kernel/Cargo.toml` | create | SsoSessionPort + OAuthStateStore ports; PkceChallenge + SsoTokens + SsoSession types |
| `crates/oya-application-sso-hub-kernel/src/lib.rs` | create | Sealed ports |
| `crates/oya-application-sso-hub-adapter/Cargo.toml` | create | Depends on sso-hub-kernel + oya-identity-kernel + sqlx + ring (PKCE crypto) |
| `crates/oya-application-sso-hub-adapter/src/lib.rs` | create | PkceOAuthAdapter: impl SsoSessionPort; two-cookie + PKCE + nonce per ADR-0123 |
| `crates/oya-application-app/Cargo.toml` | create | Composition root; leptos ssr feature; depends on all layers |
| `crates/oya-application-app/src/main.rs` | create | Leptos SSR server + DI assembly; axum router |
| `crates/oya-application-app/src/app.rs` | create | Leptos App component: shell layout + product catalog page |
| `contracts/application.openapi.yaml` | create | getProductCatalog, getProductMetadata, initiateSso, exchangeCode, getTenantBranding |
| `migrations/application/V001__application_schema.sql` | create | Full DDL (see Code Shape) |
| `docs/standards/bounded-contexts.md` | update | Register 5 application BCs |

---

## Crate Naming

```
NAME: oya-application-product-enablement-rest
JUSTIFICATION:
- microservice = application: B2B shell; ADR-0121 translated Shell→Application;
  ADR-0056 v4.1
- bc-tokens = product-enablement: product catalog BC
- layer = rest: HTTP REST handlers; axum; presentation-only; depends on
  application + kernel; no direct adapter import
- exemptions claimed: none

NAME: oya-application-sso-hub-adapter
JUSTIFICATION:
- microservice = application, bc-tokens = sso-hub: SSO/OAuth BC; PKCE + nonce
- layer = adapter: PkceOAuthAdapter implements SsoSessionPort; imports ring for
  PKCE crypto + oya-identity-kernel for session creation
- exemptions claimed: none
```

---

## Code Shape

### `migrations/application/V001__application_schema.sql`

```sql
CREATE SCHEMA IF NOT EXISTS application;

-- Per-tenant dashboard configuration
CREATE TABLE application.tenant_dashboards (
    dashboard_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL UNIQUE,
    layout_config jsonb NOT NULL DEFAULT '{}'::jsonb,
    pinned_products jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE application.tenant_dashboards FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON application.tenant_dashboards
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Per-tenant navigation state (active product, breadcrumbs, etc.)
CREATE TABLE application.navigation_state (
    nav_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    user_id uuid NOT NULL,
    active_product text NULL,
    breadcrumbs jsonb NOT NULL DEFAULT '[]'::jsonb,
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE application.navigation_state FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON application.navigation_state
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_nav_state_user ON application.navigation_state (tenant_id, user_id);

-- Per-tenant branding
CREATE TABLE application.branding (
    branding_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL UNIQUE,
    display_name text NULL,
    logo_url text NULL,
    primary_color text NULL DEFAULT '#0057FF',
    secondary_color text NULL DEFAULT '#F5F7FA',
    favicon_url text NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE application.branding FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON application.branding
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- SSO OAuth state (PKCE code verifier + nonce; short-lived)
CREATE TABLE application.sso_oauth_state (
    state_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NULL,
    state_param text NOT NULL UNIQUE,
    code_verifier text NOT NULL,
    nonce text NOT NULL,
    redirect_uri text NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_sso_state_expires ON application.sso_oauth_state (expires_at);
-- Cleanup: states expire after 10 minutes; cleaned by background worker
```

---

## Acceptance Gates

```bash
cargo check --workspace --all-features                                        # exit 0
cargo build --workspace --all-features                                        # exit 0
cargo clippy --workspace --all-features -- -D warnings                        # exit 0
cargo nextest run --workspace --all-features                                  # exit 0
cargo nextest run -p oya-application-app --test shell_product_catalog         # exit 0
cargo nextest run -p oya-application-sso-hub-adapter --test sso_pkce_flow     # exit 0
cargo nextest run -p oya-application-product-launchpad-application --test cedar_nav_gate  # exit 0
cargo deny check                                                              # exit 0
oya gate validate lean-a1 --phase P19-application
oya gate validate lean-a2 --phase P19-application
oya gate validate shardability --phase P19-application
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_product_catalog_from_tenant_registry` | GetEnabledProductsUseCase returns only enabled products |
| `test_deep_link_url_generation` | LaunchpadRoutingPort generates `app.oyatie.com/hr/...` correctly |
| `test_pkce_challenge_unique` | Two initiate_pkce() calls produce distinct code_verifier + state_param |
| `test_branding_defaults` | BrandingStore returns defaults when no custom branding set |
| `test_sso_state_expiry` | OAuthStateStore rejects expired state_param |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_product_catalog_rls` | Tenant A cannot read tenant B product catalog |
| `integration_sso_pkce_flow` | initiate_pkce → exchange_code → validate_tokens round-trip |
| `integration_leptos_shell_renders` | Leptos SSR server returns 200 for `GET /` with valid session |
| `integration_cedar_nav_gate` | Principal without navigation:access denied by PolicyEvaluator |
| `integration_branding_custom_theme` | Custom primary_color returned in branding response |

---

## Load Test

| Scenario | Target | Pass criterion |
|---|---|---|
| Shell page load (SSR) | p99 ≤200ms at 500 RPS | `http_req_duration{p(99)}<200` |
| Product catalog API | p99 ≤50ms at 2k RPS | `http_req_duration{p(99)}<50` |
| SSO redirect | p99 ≤100ms at 1k RPS | `http_req_duration{p(99)}<100` |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-architecture \
  --intent "IP-001-application-kernel-scaffold: Leptos B2B shell + 5 BCs + SSO hub" \
  --ttl 3600 \
  crates/oya-application-product-enablement-kernel/src/lib.rs::ProductCatalogPort \
  crates/oya-application-sso-hub-kernel/src/lib.rs::SsoSessionPort \
  crates/oya-application-app/src/app.rs::App \
  migrations/application/V001__application_schema.sql::application.tenant_dashboards
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-application-kernel-scaffold merged; Leptos shell deployable; 5 BCs; product catalog from TenantProductRegistry; SSO hub PKCE ADR-0123; Cedar nav gate; next: IP-002-application-leptos-shell" \
  -i high \
  -k "M02,P19,IP-001,application"
```

---

## Halt Conditions

1. Leptos SSR does not compile (WASM target `wasm32-unknown-unknown` build fails) — escalate.
2. SSO PKCE state parameter reuse (nonce/state not invalidated after exchange) — escalate; security vulnerability.
3. ProductCatalogPort imports TenantProductRegistry adapter directly (not kernel) — escalate; LEAN-A2 violation.
4. Cedar nav gate bypassable — escalate; authorization must not be optional.

---

## Next IP Pointer

`IP-002-application-leptos-shell.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0121 (product shell), ADR-0123 (SSO cookie), ADR-0209 (Leptos), ADR-0056 (BNF v4.1)
