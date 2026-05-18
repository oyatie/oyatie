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
related_adrs: [ADR-0056, ADR-0065, ADR-0105, ADR-0117, ADR-0123, ADR-0139, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-application
doc_status: published
---

# PRD-application: Application Shell (Modular Product Shell µservice)

## Purpose

The `application` microservice is oyatie's **Application Shell** — the host
surface that mounts every other product µservice for a given tenant. It is the
single browser entry point per tenant: routes resolve here, sessions live here,
the module loader fetches per-product front-end bundles here, and the auth
gateway is enforced here before any product surface renders.

Inherits Bominal ADR-0121 (Modular Product Shell) with the canonical
oyatie glossary translation `Shell → Application` per
`feedback_glossary_shared_not_platform.md` and
`feedback_bominal_inheritance_precedence.md` override #8. The retired
"platform" terminology does not appear in oyatie surfaces; every interaction
the tenant has with oyatie passes through Application.

This µservice is the user-facing front door. It is NOT a product grouping
layer (no product owns Application; products integrate INTO it via a
module-registration protocol). It hosts; it does not author.

## Tenant Value

- **Tenant Outcome 1 — One front door per tenant.** A single TLS-pinned
  origin (e.g., `<tenant>.app.oyatie.dev`) routes every employee to every
  enabled product surface; no per-product origin sprawl.
- **Tenant Outcome 2 — Sub-second time-to-interactive (TTI ≤ 2 s).** The
  Leptos WASM frontend ships from CDN; the module-loader code-splits
  per-product bundles; auth cookies prime in parallel with bundle fetch.
- **Tenant Outcome 3 — Unified sign-in, per-tenant scope.** OIDC/SAML SSO;
  two-cookie + PKCE + nonce contract (ADR-0123); one login serves every
  enabled product; tenant scope on every downstream call.
- **Tenant Outcome 4 — Module isolation.** A failing or compromised product
  bundle cannot escape its shell-iframe / route-scope boundary; supply-chain
  attacks on one module never cascade to others.
- **Tenant Outcome 5 — Audit + admin surface.** Tenant admins enable products,
  manage seats, sync users from HR, and view audit logs — all from the
  Application admin portal.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Employee | hit `<tenant>.app.oyatie.dev` and land on a routed product surface in ≤2 s TTI | I work without per-product origin hopping | `shell-routing` | Must |
| FR-02 | Employee | sign in once with SSO (OIDC or SAML) + MFA | one login for every enabled product | `auth-gateway` | Must |
| FR-03 | Tenant admin | enable / disable µservice products à-la-carte | I control which products my org pays for and accesses | `module-loader` | Must |
| FR-04 | Employee | navigate via the app-switcher; lazy-load product bundles on click | the shell stays responsive even when individual products are heavy | `module-loader` | Must |
| FR-05 | Employee | resume on session expiry without losing route state | session refresh is seamless | `auth-gateway` | Must |
| FR-06 | Tenant admin | view audit log of all admin actions | compliance reporting satisfied | `tenant-context` | Must |
| FR-07 | Operator | route every product navigation through Cedar policy gate before render | unauthorized routes refuse fail-closed | `shell-routing` | Must |
| FR-08 | Module owner | publish a product bundle with content-hash + signed integrity manifest | the loader refuses tampered bundles | `module-loader` | Must |
| FR-09 | CDN operator | purge stale shell assets globally in ≤60 s | hotfix rollouts reach all tenants quickly | `frontend-bundle-serve` | Must |
| FR-10 | Employee | use global search across enabled products (Ontology-backed) | I find entities without product hopping | `shell-routing` | Should |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Time-to-Interactive (TTI), warm cache | 600 ms | 2 s | 4 s | Leptos WASM + CDN; per `feedback_quality_performance_scalability_bar.md` |
| Time-to-Interactive (TTI), cold cache | 1.2 s | 3 s | 6 s | First visit; budget allowance |
| Shell frame render | 30 ms | 100 ms | 200 ms | Leptos hydration only; product content scheduled-for-distinct-tracked-work |
| Route resolution (server-side) | 20 ms | 100 ms | 250 ms | Cedar-gated routing |
| OIDC sign-in round-trip | 50 ms | 200 ms | 500 ms | per Bominal ADR-0123 |
| Module-loader bundle fetch | 100 ms | 500 ms | 1 s | CDN edge; cache-hit |
| Session-cookie refresh | 10 ms | 50 ms | 150 ms | rotating PKCE nonce |
| CDN global purge | 10 s | 60 s | 120 s | per ADR-0131 multi-region |

### Security

- Two-cookie + PKCE + nonce auth contract per ADR-0123; no plain bearer tokens
  in cookies; tokens are HttpOnly + Secure + SameSite=Lax.
- All routes Cedar-gated; default-deny; deny-overrides semantics. See
  `policy/route-isolation.md`.
- Module loader verifies SubResource Integrity (SRI) hash AND a signed
  integrity manifest before executing any product bundle (supply-chain
  defence). See `threat-model.md` §M-04.
- Tenant context (`tenant_id` claim) is asserted on every downstream gRPC /
  REST call; mismatched tenant on the downstream side fails closed.
- CSP: `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'`; per-module
  bundle SRI hashes; no inline scripts.
- HSTS preload; HTTP/3; TLS 1.3-only; per-pack cert via ACME.

### Audit + Compliance

- Every admin action (product enable/disable, user role change, audit-log
  export) Ed25519-sealed per Bominal ADR-0028; seal latency ≤1 s.
- OIDC/SAML assertions retained per tenant jurisdiction retention policy
  (pack-kr: 5 y; pack-eu: 6 y per GDPR Art. 30; pack-us-healthcare: 6 y
  per HIPAA §164.530(j)).

### Availability + SLO

- **99.95 % monthly** (higher than product µservices; Application outage =
  every product unreachable for affected tenants).
- RTO ≤15 s; RPO ≤5 s.
- Error budget: 0.05 % monthly. Burn-rate alarm: 3× (more sensitive than
  product µservices).

### Data residency

- Shell state (sessions, route audit) inherits the tenant's `jurisdiction_code`
  per ADR-0117; pack-pinned Postgres + Citus shard.
- CDN POPs serve **public-class** assets only (WASM bundle, fonts, CSS);
  per-tenant data never reaches a CDN POP.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0131 (per-microservice
flat layout). The application µservice declares five sibling BCs.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `shell-routing` | `oya-application-shell-routing-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Cedar-gated server-side routing; SSR hydration boundary | `Route`, `RouteRegistration`, `RouteScope` |
| `tenant-context` | `oya-application-tenant-context-{kernel,domain,usecase,api,adapter,rest,app}` | Tenant resolution from hostname + JWT; tenant scope per request | `TenantContext`, `TenantBinding` |
| `auth-gateway` | `oya-application-auth-gateway-{kernel,domain,usecase,api,adapter,adapter-oidc,adapter-saml,rest,worker,app}` | OIDC/SAML sign-in; two-cookie + PKCE + nonce contract; session lifecycle | `Session`, `OidcProvider`, `SamlAssertion`, `Mfa` |
| `module-loader` | `oya-application-module-loader-{kernel,domain,usecase,api,adapter,adapter-cdn,rest,sdk,app}` | Per-product bundle fetch, SRI + signature verification, code-splitting | `Module`, `ModuleManifest`, `IntegrityClaim` |
| `frontend-bundle-serve` | `oya-application-frontend-bundle-serve-{kernel,usecase,api,adapter,adapter-cdn,adapter-postgres,worker,app}` | CDN-fronted Leptos WASM bundle + per-tenant shell HTML; cache-control + purge | `Bundle`, `BundleVersion`, `CdnPop`, `PurgeJob` |

Total crates introduced by this µservice: **44** (8 + 7 + 10 + 9 + 8 +
2 shared composition apps = 44 deterministic layer crates).

### Naming justification — `shell-routing`

```
NAME: oya-application-shell-routing-<layer>
JUSTIFICATION:
- microservice = application: B2B Application Shell µservice; flat catalog per ADR-0131; "application" not "shell" per oyatie override #8.
- bc-tokens = shell-routing: primary BC owning Route + RouteScope entities and Cedar-gated routing use-cases. ADR-0056 v4.1 BC-optionality honoured (siblings tenant-context, auth-gateway, module-loader, frontend-bundle-serve also exist).
- layer = <layer>: one crate per ADR-0105 13-value enum.
  - kernel: port traits (RouteRegistry, RouteResolver, RouteScopeStore) + entities; zero I/O.
  - domain: pure route-matching algebra (longest-prefix, scope-set intersection).
  - usecase: orchestrators (ResolveRoute, RegisterRoute).
  - api: protocol-neutral typed request/response types.
  - adapter: protocol-neutral adapter (Postgres-backed RouteRegistry; in-memory cache layer).
  - rest: axum router + Cedar middleware.
  - sdk: client library for product µservices to register routes at boot.
  - app: composition root.
- exemptions claimed: none.
```

### Naming justification — `auth-gateway`

```
NAME: oya-application-auth-gateway-<layer>
JUSTIFICATION:
- microservice = application.
- bc-tokens = auth-gateway: owns Session + OidcProvider + SamlAssertion + Mfa entities.
- layer = <layer>:
  - adapter-oidc + adapter-saml: backend-qualified adapters per ADR-0105 Amendment 3 *-adapter-<backend> pattern (two IdP protocols; one adapter each).
  - worker: session-rotation + revocation reaper (background).
- exemptions claimed: none.
```

### Naming justification — `module-loader`

```
NAME: oya-application-module-loader-<layer>
JUSTIFICATION:
- microservice = application.
- bc-tokens = module-loader: owns Module + ModuleManifest + IntegrityClaim.
- layer = <layer>:
  - adapter-cdn: backend-qualified adapter for CDN purge / invalidation APIs (per-pack: OCI CDN; Cloudflare for global pack overlays).
  - sdk: server-side SDK for product µservices to publish + register a module bundle.
- exemptions claimed: none.
```

### Naming justification — `frontend-bundle-serve`

```
NAME: oya-application-frontend-bundle-serve-<layer>
JUSTIFICATION:
- microservice = application.
- bc-tokens = frontend-bundle-serve: owns Bundle + CdnPop + PurgeJob.
- layer = <layer>:
  - adapter-cdn: CDN admin API (publish, purge, invalidate).
  - adapter-postgres: bundle version + purge-job ledger.
  - worker: continuous health-check of CDN POPs; purge-queue consumer.
- exemptions claimed: none.
```

Cross-product rule: `application` MAY import the SDK crates of every other
µservice (route + module registration is the integration shape) but MUST NOT
import any other µservice's domain / usecase / adapter crate. LEAN-A2 lane
enforces.

Layer mapping per BC:

| BC | kernel | domain | usecase | api | adapter | adapter-* | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|
| `shell-routing` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | ✓ | ✓ |
| `tenant-context` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | ✓ |
| `auth-gateway` | ✓ | ✓ | ✓ | ✓ | ✓ | `-oidc`, `-saml` | ✓ | ✓ | — | ✓ |
| `module-loader` | ✓ | ✓ | ✓ | ✓ | ✓ | `-cdn` | ✓ | — | ✓ | ✓ |
| `frontend-bundle-serve` | ✓ | — | ✓ | ✓ | ✓ | `-cdn`, `-postgres` | — | ✓ | — | ✓ |

CI lanes that must green for every IP:

- `oya gate validate lean-a1 --microservice application` (dependency direction)
- `oya gate validate lean-a2 --microservice application` (cross-product refusal)
- `oya gate validate port-location --microservice application`
- `oya gate validate layer-correctness --microservice application`
- `oya gate validate per-microservice-layout --microservice application`
- `oya gate validate statelessness --microservice application`
- `oya gate validate shardability --microservice application`
- `oya gate validate authority-cohesion --microservice application` (HG-APP)

## Integration via Workflow + Ontology

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `EmployeeHired` | `hr` | `tenant-context` | Provision TenantUser; default role |
| `EmployeeTerminated` | `hr` | `auth-gateway` | Revoke active sessions; suspend user |
| `ProductEnabled` | `tenancy` (admin action proxied) | `module-loader` | Register product routes; mark bundle eligible to load |
| `ProductDisabled` | `tenancy` | `module-loader` | Unregister routes; drain active sessions |
| `EligibilityChanged` | `observability` | `frontend-bundle-serve` | Pause CDN promotion of held / rejected bundle versions |

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `SessionStarted` | OIDC/SAML sign-in completes | `audit-chain`, `tenancy` | `session-sm` |
| `SessionEnded` | Logout / timeout / forced revocation | `audit-chain`, `tenancy` | `session-sm` |
| `ModuleLoaded` | Product bundle fetched + verified | `observability` (TTI signal) | — |
| `ModuleLoadRejected` | SRI / signature mismatch on bundle fetch | `audit-chain`, `grafana-oncall` (paging) | — |
| `RouteAccessDenied` | Cedar policy denies a route | `audit-chain`, `tenancy` | — |
| `CdnPurgeRequested` | Operator-initiated purge | `audit-chain` | — |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Session{user_id, tenant_id, started_at}` | `session_for→TenantUser` | `auth-gateway` | Ed25519 |
| `Module{name, version, sri_hash, signer}` | `module_of→EnabledProduct` | `module-loader` | Ed25519 |
| `Bundle{version, content_hash, cdn_etag}` | `bundle_of→Module` | `frontend-bundle-serve` | Ed25519 |
| `RouteScope{path_glob, required_role}` | `scope_for→EnabledProduct` | `shell-routing` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `EnabledProduct` | `module-loader`, `shell-routing` | `filter(tenant_id).where(enabled=true)` |
| `TenantUser` | `auth-gateway` | `filter(tenant_id, user_id)` on every sign-in |
| `Role` | `shell-routing` | `filter(tenant_user_id)` per Cedar gate |

## Competitive Benchmark

| Competitor | Surface | Parity dimensions | Primary source |
|---|---|---|---|
| Vercel | Vercel Platform | Edge-network frontend serve; per-tenant project isolation; preview deployments | `vercel.com/docs` |
| Next.js | Next.js App Router | File-system routing; server components; module federation | `nextjs.org/docs` |
| Stripe | Stripe Dashboard | Tenant-scoped shell; product-area switching; admin actions UX | `stripe.com/docs/dashboard` |
| Linear | Linear App Shell | TTI ≤1 s; module-isolated workspace; offline-first | `linear.app` |
| Notion | Notion App | Block-loader code-splitting; multi-workspace shell; lazy module fetch | `notion.so` |
| Palantir | Foundry App Shell | Per-tenant Workshop hosting; module registration protocol; per-org RBAC | `palantir.com/foundry/` |

Key parity gaps (ordered):

1. **Sub-2-s TTI under real-tenant data load** — Vercel + Linear set the bar; oyatie matches via CDN-fronted WASM + code-split per-product bundles + parallel auth-cookie prime.
2. **Signed module manifest** — Vercel/Notion lack cryptographic per-module signature verification at load time; oyatie adds Ed25519 signed integrity manifest (supply-chain hardening).
3. **Cedar-gated routing** — Stripe Dashboard / Linear use bespoke RBAC at route render; oyatie expresses route policy in Cedar (auditable, externally verifiable).
4. **Per-tenant module enablement vs. per-account flags** — Stripe + Linear toggle features per account; oyatie toggles whole products per tenant via product-enablement protocol (à-la-carte console à la AWS).

## Performance Targets

(Repeated in Non-Functional for SLO-engine ingestion.)

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| TTI (warm) | 600 ms | 2 s | 4 s | |
| Route resolve | 20 ms | 100 ms | 250 ms | |
| Module bundle fetch | 100 ms | 500 ms | 1 s | CDN cache-hit |
| OIDC sign-in | 50 ms | 200 ms | 500 ms | |
| Session refresh | 10 ms | 50 ms | 150 ms | |
| CDN purge global | 10 s | 60 s | 120 s | |
| Audit seal | — | 1 s | — | Per (tenant_id, period) per ADR-0028 |

Error budget: **0.05 % monthly**. SLO burn-rate alarm: **3×**.

## Horizontal Scalability

**State strategy**: `mixed` — Application Shell is stateless at the HTTP
edge (route resolution + bundle serve); sessions live in Valkey / Valkey
(eviction-friendly); admin / audit state lives in Postgres + Citus
(tenant-partitioned).

**Active-active compatibility**:

- `shell-routing`, `frontend-bundle-serve` REST/WASM serve layers: **stateless-compatible**.
- `auth-gateway`: **stateless-compatible** for sign-in; **single-writer-compatible** for session-revocation (consistent-replication required).
- `tenant-context`, `module-loader`: **stateless-compatible**.

| Dimension | Baseline / cell | Max / cell | Scale-out trigger |
|---|---|---|---|
| Concurrent active users | 50 k | 5 M | Session-store memory > 80% |
| Sessions / sec (sign-in) | 5 k | 100 k | Auth-gateway CPU > 70% |
| Routes resolved / sec | 50 k | 1 M | Shell-routing CPU > 70% |
| Module bundle fetches / sec | 100 k | 5 M | CDN POP origin shield miss rate > 1% |
| Tenants / cell | 1 k | 50 k | Postgres Citus shard count > 80% |
| Audit seal QPS | 1 k | 50 k | Audit-chain lane queue depth > 30 s |

Scale-out policy:
- Kubernetes HPA on each REST layer (CPU 70%); cold-start budget ≤500 ms.
- Valkey/Valkey Sentinel/Cluster for session store; multi-AZ.
- Postgres + Citus sharded on `tenant_id`; RLS row-level scope.
- CDN: pack-pinned primary (OCI CDN) + global overlay (Cloudflare) for
  public-class assets only.

Cross-region: M03 KR-only launch; subsequent-to-M03-completion multi-region requires DNS
(OCI Traffic Management) + cert sync + per-pack session-store federation.
See `multi-region.md`.

Sharding: shell partitions by `tenant_id`; route registration + module
manifest tables partitioned on the same key. LEAN shardability lane verifies.

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | OIDC sign-in round-trip; two-cookie contract; tenant_id JWT claim present | integration test `test_oidc_signin_two_cookie` |
| AC-02 | TTI ≤2 s p99 at 10 k concurrent sessions (warm cache) | k6 smoke; `http_req_duration{p(99)}<2000` |
| AC-03 | Route resolve p99 ≤100 ms | k6 + Mimir SLI |
| AC-04 | Module-loader rejects bundle with broken SRI hash | `cargo nextest run -p oya-application-module-loader-usecase test_sri_mismatch_rejected` |
| AC-05 | Module-loader rejects bundle with invalid Ed25519 signature | `cargo nextest run -p oya-application-module-loader-usecase test_signature_invalid_rejected` |
| AC-06 | Cedar policy: employee cannot access admin portal | `oya gate validate cedar-policy --ms application` |
| AC-07 | LEAN-A2: Application imports no product-specific domain crates | `oya gate validate lean-a2 --ms application` exits 0 |
| AC-08 | `EmployeeTerminated` → all active sessions revoked in ≤5 s | integration test `test_session_revocation_on_termination` |
| AC-09 | CDN global purge completes within 60 s | timed e2e drill |
| AC-10 | Audit chain: all admin actions sealed; export verifies | `oya gate validate audit-chain --ms application` |
| AC-11 | `oya gate validate per-microservice-layout --ms application` exits 0 | ADR-0131 lane |
| AC-12 | HG-APP gate registered green in `/specs/hyperscaler-gates.json` | ADR-0123 lane |

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | SAML IdP for M03: Azure AD / Okta / KR-specific (KISA-approved)? | council-product | M03/P01 |
| 2 | Module-loader sandbox: iframe-with-postMessage vs. Web Workers + structured-clone? | council-architecture | ADR-#### |
| 3 | CDN provider for global overlay (Cloudflare vs. Fastly) — cost vs. KR-residency latency | ops-finops | M03/P02 |
| 4 | Native client tiers (iOS / Android / desktop) — same module-loader contract or distinct? | council-architecture | subsequent-to-M03-completion ADR |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0121 | Modular Product Shell | inherited — Shell → Application translation |
| Bominal ADR-0123 | Cross-product auth cookie + redirect | inherited |
| Bominal ADR-0209 | Client architecture (Leptos + 5 native tiers) | inherited |
| Bominal ADR-0018 | Tenancy RLS posture | inherited |
| Bominal ADR-0028 | Audit chain (Ed25519) | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0065 | Leptos docs/webapp framework | frontend framework authority |
| ADR-0105 | 13-layer canonical enum | layer authority |
| ADR-0117 | Data residency packs | residency authority |
| ADR-0139 | Agentic SLO-gated promotion | release gate |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0123 | Hyperscaler maturity claim gate | HG-APP registers here |
