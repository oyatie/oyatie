---
doc_class: PolicySpec
title: Route Isolation Policy (per-tenant route scoping)
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-application + ops-security
deciders: council-architecture, ops-security, axis-application, council-privacy
related_adrs: [ADR-0056, ADR-0117, ADR-0121, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/threat-model.md (I-01, I-03, E-01, E-04)
  - microservices/application/dpia.md (R-04, R-09)
  - microservices/application/policy/tenant-scope.cedar
  - microservices/application/policy/ci-scope.cedar
  - microservices/application/policy/auditor-scope.cedar
  - microservices/application/policy/public-read.cedar
review_cadence: quarterly + on every new product onboarding
doc_status: published
---

# Route Isolation Policy (application µservice)

## Purpose

Define the per-tenant, per-role route-scoping invariants that
`shell-routing` enforces via Cedar policy on every request. Every route in
the Application Shell maps to a `(RouteScope, RouteRegistration)` pair,
where the scope encodes the tenant binding, required role(s), and pack
residency. This document is the canonical reference for axis-application
when registering a new product's routes and for ops-security when
reviewing route-scope changes.

## Invariants

These invariants are LANE-ENFORCED. The `oya-application-route-isolation`
lane refuses any PR that violates them.

### RI-01 — Default deny

Every route is `forbid (principal, action, resource);` by default. A route
is reachable only when at least one explicit `permit` matches AND no
`forbid` matches (Cedar deny-overrides semantics).

### RI-02 — Tenant binding on every route

`RouteRegistration` MUST carry a `tenant_scope` field, one of:

- `global-public` — anonymous reachable (rare; e.g., status page, login page);
- `tenant-scoped` — must equal the principal's `tenant_id` claim;
- `cross-tenant-operator` — oyatie operators only with JIT elevation.

The lane refuses a route declaration without `tenant_scope`.

### RI-03 — Role gating

`RouteRegistration` MUST carry a `required_roles` set. The principal's
role-set MUST intersect the required set. Empty `required_roles` is
forbidden (no implicit "any role").

### RI-04 — Pack residency

`RouteRegistration` MUST carry a `pack_residency` field; the served
resource MUST resolve to the same pack as the principal's pack assignment.
Cross-pack route resolution fails closed.

### RI-05 — Audit emission

Every route resolve (allow OR deny) emits an audit record. Denied requests
emit a Sev-3 event into the audit chain; ≥3 denials/minute per principal
escalate to Sev-2.

### RI-06 — No URL-embedded PII

Route paths MUST NOT contain raw email, name, or other PII. The lane
refuses path patterns matching `@` or `[A-Z][a-z]+ [A-Z][a-z]+`. Use
opaque `user_id`.

### RI-07 — Cedar evaluation budget

Cedar evaluation per request MUST complete in ≤10 ms p99. Slow-path
circuit-breaker engages default-deny at 50 ms.

## Route Categories

| Category | tenant_scope | required_roles | Example | Audit |
|---|---|---|---|---|
| Login / SSO | `global-public` | none | `GET /login`, `POST /oauth/callback` | session-start record |
| Public assets | `global-public` | none | `GET /assets/main.<hash>.wasm` | aggregated metric only |
| Status page | `global-public` | none | `GET /status` | aggregated |
| Tenant home | `tenant-scoped` | `employee` | `GET /` | record |
| Product surface | `tenant-scoped` | `employee` + product-specific | `GET /hr/*` | record |
| Admin portal | `tenant-scoped` | `tenant-admin` | `GET /admin/users`, `POST /admin/products/enable` | sealed + 2-person-rule on write |
| Operator console | `cross-tenant-operator` | `oyatie-operator` + JIT | `GET /__ops/health` | sealed + JIT-trace |
| Auditor read | `cross-tenant-operator` | `external-auditor` + time-boxed | `GET /__audit/export` | sealed + watermark |

## Worked Examples

### Example 1: HR product home

```yaml
route: GET /hr/dashboard
tenant_scope: tenant-scoped
required_roles: [employee, hr-viewer]
pack_residency: inherit-from-tenant
csp_module_id: oya-hr-module
required_mfa: false
```

Cedar evaluation:

```cedar
permit (
  principal in TenantUser::?u,
  action == Action::"render_route",
  resource == Route::"/hr/dashboard"
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  ("hr-viewer" in principal.roles || "employee" in principal.roles)
};
```

### Example 2: Tenant-admin user provisioning

```yaml
route: POST /admin/users
tenant_scope: tenant-scoped
required_roles: [tenant-admin]
pack_residency: inherit-from-tenant
csp_module_id: oya-tenancy-admin-module
required_mfa: webauthn-step-up
```

Cedar evaluation requires both role match AND `principal.mfa_factor ==
"webauthn"`.

### Example 3: Operator console (oyatie internal)

```yaml
route: GET /__ops/cluster-health
tenant_scope: cross-tenant-operator
required_roles: [oyatie-operator]
pack_residency: any
required_jit_token: true
```

Cedar evaluation also requires `principal.jit_token.valid_until > now() &&
principal.jit_token.reason.matches("ops:.*")`.

## Lane Enforcement

| Lane | What it checks |
|---|---|
| `oya-application-route-isolation` | Every route declaration has tenant_scope + required_roles + pack_residency |
| `oya-application-cedar-policy-compiles` | All policies in `policy/*.cedar` schema-validate |
| `oya-application-cedar-default-deny` | Schema asserts default `forbid` rule present |
| `oya-application-route-pii-path-block` | Path patterns refuse `@` and human-name shapes |
| `oya-application-cedar-eval-budget` | Benchmark asserts p99 ≤10 ms over canonical corpus |
| `oya-application-route-audit-emission` | Every route resolution emits audit record |

## Anti-patterns

- "Hidden" admin route reachable only via URL knowledge — refused; must be Cedar-gated.
- Wildcard `permit` on `Route::?` — refused; must enumerate or use `RouteScope` set membership.
- Per-tenant Cedar policy fragments (every tenant gets bespoke .cedar) — refused; tenant data is data, not policy.
- Embedded JWT inspection in handler bypassing tenant-context middleware — refused; lane scans for `decode_jwt` outside `oya-application-auth-gateway-adapter-oidc`.

## Onboarding a new product's routes

1. Product team authors `route-registration.yaml` in their µservice folder.
2. PR includes evidence: tenant_scope + required_roles + pack_residency.
3. CODEOWNERS in `microservices/application/CODEOWNERS` requires
   axis-application + ops-security sign-off on route-registration changes.
4. Lane `oya-application-route-isolation` runs end-to-end check.
5. On merge: `module-loader` registers the routes; `audit-chain` records
   the registration; `observability` opens an OpenSLO target for the new
   route group.

## References

- ADR-0123 cross-product auth + redirect contract.
- ADR-0028 audit chain.
- Bominal ADR-0018 RLS posture.
- `microservices/application/threat-model.md` STRIDE catalogue.
- `microservices/application/policy/*.cedar` (the policies themselves).
