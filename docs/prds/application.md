---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-application
microservice: application
status: Accepted
sales_segment: Enterprise
tier: B2B
milestone_first_ship: M03-first-paying-tenant
bominal_source:
  - ADR-0121  # Modular Product Shell (translated: Application)
  - ADR-0123  # cross-product auth cookie + redirect contract
  - ADR-0209  # client architecture (Leptos web + 5 native tiers)
  - ADR-0018  # tenancy RLS posture
  - ADR-0019  # runtime target metadata model
doc_status: published
---

# PRD-application: Application µservice (B2B unified shell)

---

## Purpose

Application is the B2B unified shell: the tenant-facing surface through which
organizations sign in, enable µservice products à-la-carte, manage billing and
seats, and navigate across all enabled capabilities. It is the entry point for
every B2B tenant interaction.

Inherits from Bominal ADR-0121 (Modular Product Shell), translated to oyatie
glossary: `Shell → Application` (per `feedback_bominal_inheritance_precedence.md`
override #8). No further divergences.

Application is NOT a product grouping layer — it is one µservice in the flat
catalog that provides the shell UX. Products integrate into Application via a
capability-registration protocol; Application never owns product-specific logic.

---

## Tenant Value

- **Unified sign-in**: OIDC/SAML SSO; MFA; two-cookie + PKCE + nonce contract
  (ADR-0123); one login for all enabled products.
- **Product enablement console**: tenants enable/disable µservice products
  à-la-carte like the AWS console; per-product seat licensing; billing integration.
- **Navigation shell**: app-switcher; product sidebar; global search (Ontology-
  backed); notification center; user preferences.
- **Admin portal**: tenant configuration; user provisioning; role assignment
  (Cedar policy sync); audit log viewer.
- **Billing + seat management**: tenant plan tier; per-seat counts; invoice view;
  upgrade/downgrade flows.

---

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Tenant admin | sign in with SSO (OIDC/SAML) or username/password + MFA | all employees authenticate securely | `auth` | Must |
| FR-02 | Tenant admin | enable HR, Payroll, Connect-Pro from a product catalog | I control which products my org pays for and accesses | `product-enablement` | Must |
| FR-03 | Employee | navigate to any enabled product from a unified app-switcher | no separate logins per product | `navigation` | Must |
| FR-04 | Tenant admin | provision users, assign roles, sync with HR on `EmployeeHired` | user accounts match HR system automatically | `user-provisioning` | Must |
| FR-05 | Tenant admin | view and download invoices; manage billing plan | finance team handles billing without engineering involvement | `billing` | Must |
| FR-06 | Employee | use global search across all enabled products (Ontology-backed) | I find entities across HR, Payroll, without switching apps | `search` | Should |
| FR-07 | Tenant admin | view audit log of all admin actions (product enablement, user changes) | compliance reporting satisfied | `audit` | Must |

---

## Non-Functional Requirements

### Performance
- P99 sign-in (OIDC token exchange): ≤200 ms.
- P99 app-switcher navigation render: ≤100 ms (shell frame; product content loads separately).
- P99 product enablement toggle: ≤500 ms.
- Global search (Ontology query): ≤200 ms at p99.

### Security
- Two-cookie + PKCE + nonce auth contract per ADR-0123; no plain bearer tokens
  in cookies.
- Cedar policy enforced at capability-registry layer; Application never
  duplicates product-level authorization.
- JWT `tenant_id` on every downstream call; Application is the trust boundary.
- MFA enforcement configurable per tenant; TOTP + WebAuthn supported.

### Audit + Compliance
- Every admin action (product enable/disable, user role change) Ed25519-sealed
  per ADR-0028; seal latency ≤1 s.
- OIDC/SAML assertion logs retained per tenant's jurisdiction retention policy.
- Jurisdiction overlay `KR` per ADR-0127 for M03 tenants.

### Availability + SLO
- 99.95% monthly (higher than product µservices; Application outage = all products
  unreachable for affected tenants).
- RTO ≤15 s; RPO ≤5 s.

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `auth` | `application-auth-{domain,application,infrastructure,rest}` | OIDC/SAML sign-in; token issuance; MFA | `Session`, `OidcProvider` |
| `product-enablement` | `application-product-enablement-{domain,application,infrastructure,rest}` | Product catalog; enable/disable; seat licensing | `ProductLicense`, `EnabledProduct` |
| `user-provisioning` | `application-user-provisioning-{domain,application}` | User create/suspend; role assignment; HR sync | `TenantUser` |
| `billing` | `application-billing-{domain,application,infrastructure,rest}` | Plan management; invoicing; seat counts | `BillingPlan`, `Invoice` |
| `navigation` | `application-navigation-{domain,application,rest}` | App-switcher; product links; notifications | `NavItem` |

```
NAME: application-product-enablement-rest
JUSTIFICATION:
- microservice = application: B2B unified shell µservice; flat catalog; ADR-0056 v4.1; "Application" not "Shell" per oyatie override
- bc-tokens = product-enablement: application has multiple BCs; product-enablement BC owns ProductLicense entity + enable/disable use-cases; ADR-0056 v4.1 BC-optionality
- layer = rest: HTTP handler wiring for product catalog API; maps HTTP → application commands; no business logic; ADR-0056 §"Layer semantics"
- exemptions: none
```

---

## Integration via Workflow + Ontology

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `EmployeeHired` | `hr` | `user-provisioning` | Create TenantUser; assign default role |
| `EmployeeTerminated` | `hr` | `user-provisioning` | Suspend TenantUser; revoke active sessions |

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `ProductEnabled` | Tenant enables a µservice | `<µservice>` (onboarding flow) | `product-activation-sm` |
| `ProductDisabled` | Tenant disables a µservice | `<µservice>` (teardown flow) | `product-activation-sm` |
| `UserProvisioned` | TenantUser created | `connector` (provisioning) | `user-provisioning-sm` |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `TenantUser` | `HasRole` → `Role` | `user-provisioning` | Ed25519 on every change |
| `EnabledProduct` | `LicensedBy` → `BillingPlan` | `product-enablement` | Ed25519 |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Employee` | `user-provisioning` | `filter(tenant_id)` — sync on hire |

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| AWS | AWS Console | Product catalog UX; service enable/disable; IAM role assignment; billing console | https://aws.amazon.com/console |
| Salesforce | AppExchange + Admin Console | App enablement; user provisioning; permission sets | https://admin.salesforce.com |
| Google Workspace | Admin Console | SSO; user provisioning; product licensing; audit logs | https://admin.google.com |
| Rippling | Rippling Platform | Unified shell; HR-driven user provisioning; app access management | https://www.rippling.com |

Key parity gaps:
1. **AWS Console à-la-carte UX**: service enable/disable with per-feature pricing display — must reach AWS console clarity of product scope per-enable action.
2. **SCIM 2.0 provisioning**: Rippling/Google Workspace parity for SCIM-based user sync (M03 manual; SCIM in M04).
3. **Audit log export**: Google Admin Console parity — structured JSON export of all admin events.

---

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Sign-in (OIDC token exchange) | 50 ms | 200 ms | 500 ms | |
| Shell frame render (app-switcher) | 30 ms | 100 ms | 200 ms | Leptos SSR; product content deferred |
| Product enable toggle | 100 ms | 500 ms | 1 s | Triggers Workflow activation |
| Global search (Ontology) | 30 ms | 200 ms | 500 ms | pgroonga + Tantivy |
| Audit chain seal | — | 1 s | — | Per (tenant_id, period); ADR-0028 |

Error budget: 0.05% monthly (higher bar; shell outage = all products down).
SLO burn-rate alarm: 3× (more sensitive than product µservices).

---

## Horizontal Scalability

**State strategy**: `postgres` — sessions, product licenses, tenant users in
Postgres + Citus; `tenant_id` partition key; Postgres RLS.

**Active-active compatibility**: `stateless-compatible` for auth/navigation REST
layers; `single-writer-compatible` for billing mutations.

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max tenants per cell | 1,000 | 50,000 | Shard count > 80% |
| Max concurrent sessions | 50,000 | 5,000,000 | Session store memory > 80% |
| Max QPS (auth) | 5,000 | 100,000 | CPU > 70% |

Scale-out: auth layer stateless HPA; Valkey/Redis cluster for session store;
Postgres Citus sharding on `tenant_id`.
Cross-region: M03 KR only; global multi-region post-M03 (Application is the
entry point — requires cross-region DNS + OCI Traffic Management).

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | OIDC sign-in round-trip; JWT `tenant_id` claim present; two-cookie contract met | integration test `test_oidc_signin_two_cookie` |
| AC-02 | Product enable/disable triggers `ProductEnabled`/`ProductDisabled` Workflow event | integration test `test_product_enablement_workflow` |
| AC-03 | `EmployeeHired` → TenantUser created in ≤5 s | integration test `test_user_provisioning_on_hire` |
| AC-04 | Cedar policy: employee cannot access admin portal | `presubmit` (retired CLI `gate validate cedar-policy --ms application`) |
| AC-05 | LEAN-A2: Application shell imports no product-specific crates | `presubmit` (retired CLI `gate validate lean-a2 --ms application`) exits 0 |
| AC-06 | Shell frame p99 ≤100 ms at 10k concurrent sessions | k6 smoke; `http_req_duration{p(99)}<100` |
| AC-07 | Audit log: all admin actions sealed; export correct | `presubmit` (retired CLI `gate validate audit-chain --ms application`) |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | SAML IdP for M03: Azure AD / Okta / or KR-specific? | council-product | M03/P01 |
| 2 | Billing engine: built-in or third-party (Stripe Billing)? | council-architecture | ADR-#### |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0121 | Modular Product Shell | inherited — translated Shell → Application |
| Bominal ADR-0123 | Cross-product auth cookie + redirect | inherited |
| Bominal ADR-0209 | Client architecture | inherited — Leptos web + 5 native tiers |
| Bominal ADR-0018 | Tenancy RLS posture | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | integration plane |
| ADR-0106 | Ontology architecture | information plane |
