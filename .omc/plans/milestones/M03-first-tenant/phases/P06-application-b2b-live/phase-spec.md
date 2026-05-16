---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P06-application-b2b-live
status: Proposed
acceptance_lanes: []
entry_gate: "M03/P01-hr through P05-connect-pro-messenger complete (all \xB5services\
  \ ship);\noya-tenancy-kernel + oya-identity-kernel ship (M02 substrate);\nCedar\
  \ policy engine bootstrapped; Workflow event bus operational.\n"
exit_gate: "All IP acceptance gates green; OIDC sign-in round-trip test green; two-cookie\
  \ contract met (ADR-0123);\nHR/Payroll/Connect-Pro product enable/disable triggers\
  \ ProductEnabled/ProductDisabled Workflow events;\nEmployeeHired \u2192 TenantUser\
  \ created \u22645s;\ntenant onboarding sub-5-min activation verified (end-to-end\
  \ Playwright test);\nshell frame p99 \u2264100ms at 10k concurrent sessions;\n`oya\
  \ gate validate lean-a2 --ms application` exits 0;\n`oya gate validate cedar-policy\
  \ --ms application` exits 0;\ngrit done on all P06 symbols; ICM phase-handoff row\
  \ emitted.\n"
depends_on:
- milestone: M03
  phase: P05-connect-pro-messenger
  reason: "Application shell enables the M03 \xB5services (HR/Payroll/Accounting/Connect-Pro);\
    \ all \xB5service REST APIs must exist before product-enablement toggles can route\
    \ to them."
parallel_wave: 4
owner_team: council-architecture
purpose: "Delivers the `oya-application-*` µservice: the B2B unified shell through which KR group tenants sign in (OIDC/SAML SSO."
---
# P06-application-b2b-live: Application B2B shell — OIDC/SAML SSO, product-enablement console, tenant onboarding, Leptos SSR

## Purpose

Delivers the `oya-application-*` µservice: the B2B unified shell through which
KR group tenants sign in (OIDC/SAML SSO; passkey-first per ADR-0210), enable
HR/Payroll/Accounting/Connect-Pro à-la-carte, manage seats and billing, and
navigate across all enabled products via a single Leptos SSR web shell. Tenant
onboarding flow completes in under 5 minutes (sub-5-min activation per
ADR-0118 / Bominal M3 bar).

Implements Bominal ADR-0121 (Modular Product Shell) translated to oyatie:
`Shell → Application` (per `feedback_bominal_inheritance_precedence.md` override #8).
Auth contract follows ADR-0123 (two-cookie + PKCE + nonce).

---

## Scope

### In-scope

| µservice | Bounded Contexts | Crate family (BNF v4.1) |
|---|---|---|
| `application` | `auth` | `oya-application-auth-{kernel,domain,application,adapter,rest}` |
| `application` | `product-enablement` | `oya-application-product-enablement-{kernel,domain,application,adapter,rest}` |
| `application` | `user-provisioning` | `oya-application-user-provisioning-{domain,application}` |
| `application` | `billing` | `oya-application-billing-{kernel,domain,application,adapter,rest}` |
| `application` | `navigation` | `oya-application-navigation-{domain,application,rest}` |
| `application` | `app` | `oya-application-app` |
| `application` | `web` | `oya-application-web` (Leptos SSR/SPA; pre-auth SSR, post-auth SPA per ADR-0209) |

Naming justifications:

```
NAME: oya-application-auth-kernel
JUSTIFICATION:
- microservice = application: B2B unified shell µservice; registered; ADR-0056 v4.1; "Application" not "Shell" per oyatie override
- bc-tokens = auth: application has multiple BCs (auth/product-enablement/user-provisioning/billing/navigation); auth BC owns Session + OidcProvider entities + SessionStore/OidcPort port-traits; two-cookie PKCE+nonce contract (ADR-0123); ADR-0056 v4.1 BC-optionality
- layer = kernel: pure SessionId value types + SessionStore/OidcPort port declarations; zero logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-application-product-enablement-kernel
JUSTIFICATION:
- microservice = application; bc-tokens = product-enablement: product-enablement BC owns ProductLicense + EnabledProduct entities + ProductLicenseRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure LicenseId value types + ProductLicenseRepository port declaration; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-application-product-enablement-rest
JUSTIFICATION:
- microservice = application; bc-tokens = product-enablement; layer = rest: Axum HTTP handlers for product catalog API; maps HTTP → application commands; no business logic; ADR-0056 §"Layer semantics"
- exemptions: none (this exact name appears as example in ADR-0056)

NAME: oya-application-user-provisioning-domain
JUSTIFICATION:
- microservice = application; bc-tokens = user-provisioning: user-provisioning BC owns TenantUser entity + role assignment + HR sync logic; application wires Workflow event subscription (EmployeeHired); ADR-0056 v4.1 BC-optionality
- layer = domain: TenantUser aggregate + role invariants; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-application-billing-kernel
JUSTIFICATION:
- microservice = application; bc-tokens = billing: billing BC owns BillingPlan + Invoice entities + BillingStore port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure PlanId/InvoiceId value types + BillingStore port declaration; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-application-navigation-domain
JUSTIFICATION:
- microservice = application; bc-tokens = navigation: navigation BC owns NavItem entity + app-switcher + notification center logic; domain only — no heavy infra; ADR-0056 v4.1 BC-optionality
- layer = domain: NavItem aggregate + routing logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-application-app
JUSTIFICATION:
- microservice = application; bc-tokens: OMITTED — composition-root binary; ADR-0056 §"BC optionality"
- layer = app: main.rs + DI wiring; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-application-web
JUSTIFICATION:
- microservice = application; bc-tokens: OMITTED — single Leptos web frontend; SSR pre-auth + SPA post-auth per ADR-0209; no layer suffix as this is the frontend crate (BNF v4.1: "web" is not in the 12-value layer enum; treated as a peer binary registered separately under application µservice)
- layer = sdk: client-facing Leptos application; depends on kernel types only for data contracts; ADR-0056 §"sdk-kernel-only" exemption acknowledged — frontend has additional Leptos framework deps
- exemptions: sdk-kernel-only relaxed for Leptos web app (framework deps unavoidable; noted in workspace.metadata.oya.microservices.application.public_layers)
```

### Out-of-scope

- SCIM 2.0 provisioning — deferred to M04 per PRD-application open question #2.
- Billing engine (Stripe Billing integration) — deferred per PRD open question #2; M03 billing is manual/invoice only.
- Global search (Ontology-backed full-text) — deferred to M04 (PRD FR-06 priority: Should).

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Application B2B shell: OIDC/SAML SSO (two-cookie + PKCE), product-enablement console (HR/Payroll/Accounting/Connect-Pro), Leptos SSR shell frame, tenant onboarding flow (<5 min activation), TenantUser provisioning from EmployeeHired, billing plan management, Cedar policy integration, audit log, load tests | pending | council-architecture |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features                                                      # exit 0
cargo build -p oya-application-app --all-features                                           # exit 0
cargo clippy -p oya-application-auth-domain -p oya-application-product-enablement-domain -- -D warnings  # exit 0
cargo nextest run --test test_oidc_signin_two_cookie                                        # exit 0; JWT tenant_id claim present
cargo nextest run --test test_product_enablement_workflow                                   # exit 0; ProductEnabled/ProductDisabled events
cargo nextest run --test test_user_provisioning_on_hire                                     # exit 0; EmployeeHired → TenantUser ≤5s
cargo deny check                                                                            # exit 0
```

### E2E gates (Playwright + Leptos SSR)

```bash
# Tenant onboarding sub-5-min activation
rtk playwright test tests/e2e/tenant-onboarding.spec.ts
# Shell frame render p99 ≤100ms
rtk playwright test tests/e2e/shell-frame-perf.spec.ts
```

### Fitness lane gates

```bash
oya gate validate lean-a2 --ms application        # Application shell imports no product-specific crates
oya gate validate lean-a1 --ms application        # layer ordering
oya gate validate cedar-policy --ms application   # employee cannot access admin portal
oya gate validate audit-chain --ms application    # all admin actions sealed; Ed25519
oya gate validate jurisdiction-overlay --ms application  # KR jurisdiction overlay
```

### Performance gate

```bash
# k6: shell frame p99 ≤100ms at 10k concurrent sessions
k6 run tests/load/smoke-application-shell.js --env BASE_URL=http://localhost:8085
# k6: OIDC sign-in p99 ≤200ms
k6 run tests/load/smoke-application-auth.js --env BASE_URL=http://localhost:8085
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-application-auth-kernel` | `kernel` | Yes — `SessionStore`, `OidcPort` | N/A |
| `oya-application-auth-domain` | `domain` | N/A | N/A |
| `oya-application-auth-adapter` | `adapter` | N/A | Yes — `ValKeySessionStore`, `OidcProviderAdapter` |
| `oya-application-product-enablement-kernel` | `kernel` | Yes — `ProductLicenseRepository` | N/A |
| `oya-application-product-enablement-adapter` | `adapter` | N/A | Yes — `PostgresProductLicenseRepository` |
| `oya-application-user-provisioning-domain` | `domain` | N/A | N/A |
| `oya-application-billing-kernel` | `kernel` | Yes — `BillingStore` | N/A |
| `oya-application-billing-adapter` | `adapter` | N/A | Yes — `PostgresBillingStore` |
| `oya-application-navigation-domain` | `domain` | N/A | N/A |
| `oya-application-app` | `app` | N/A | Unrestricted inward |
| `oya-application-web` | `sdk` | N/A | Leptos frontend (public_layers exemption) |

Cross-product: Application NEVER imports product-specific crates.
Product capability registration via capability-registry µservice only.
Employee sync via `oya-ontology-entity-kernel::ObjectStore` port.

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `auth` | `application` | pending |
| `product-enablement` | `application` | pending |
| `user-provisioning` | `application` | pending |
| `billing` | `application` | pending |
| `navigation` | `application` | pending |

---

## Grit Claim Symbols

```
crates/oya-application-auth-kernel/src/ports.rs::SessionStore
crates/oya-application-auth-kernel/src/ports.rs::OidcPort
crates/oya-application-product-enablement-domain/src/product_license.rs::ProductLicense
crates/oya-application-user-provisioning-domain/src/tenant_user.rs::TenantUser
crates/oya-application-billing-kernel/src/ports.rs::BillingStore
contracts/application.openapi.yaml::enableProduct
contracts/application.openapi.yaml::signIn
docs/standards/bounded-contexts.md::application.auth
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P06-application-b2b-live started; all M03 µservices available; scope: OIDC/SAML SSO (ADR-0123), product-enablement console, Leptos SSR shell, tenant onboarding <5 min, TenantUser from EmployeeHired, billing" \
  -i high \
  -k "M03,P06,phase-start,application"

icm store \
  -t context-oyatie \
  -c "Phase P06-application-b2b-live complete; B2B shell live; OIDC + two-cookie (ADR-0123); HR/Payroll/Accounting/Connect-Pro enabled; Leptos SSR; sub-5-min onboarding; next: P08-kr-acceptance-evidence (P07-workflow-studio-editor runs parallel)" \
  -i high \
  -k "M03,P06,phase-complete,application"
```

---

## References

- PRD: `docs/prds/application.md`
- Bominal ADRs inherited: ADR-0121 (Modular Product Shell → Application), ADR-0123 (two-cookie auth), ADR-0209 (Leptos web + 5 native tiers), ADR-0018 (tenancy RLS), ADR-0028 (audit chain)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- oyatie override: Shell → Application (feedback_bominal_inheritance_precedence.md override #8)
