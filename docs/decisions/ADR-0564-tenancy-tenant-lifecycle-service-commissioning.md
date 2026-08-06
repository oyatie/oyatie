---
id: ADR-0564
title: "Commission the tenancy tenant-lifecycle registration service (G006 slice 1): a runnable tenant register/provision/read delivery surface over the locked lifecycle core"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-20
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0550, ADR-0562]
related: [ADR-0083, ADR-0105, ADR-0131, ADR-0243, ADR-0476, ADR-0510, ADR-0536, ADR-0541, ADR-0547, ADR-0553, ADR-0555, ADR-0559, ADR-0562]
related_specs: []
milestone: W0
---

# ADR-0564: Commission the tenancy tenant-lifecycle registration service (G006 slice 1)

## Status

**Proposed - 2026-06-20 (G006 vertical opener; door: two-way — no consumer points at the
service yet, deleting the two crates restores the prior state).**

## Context

The auth/onboarding E2E ground-truth audit (`.omc/ultragoal/auth-onboarding-e2e-audit-findings.md`,
2026-06-19) found tenant registration **PARTIAL**: a real, conformance-tested lifecycle finite
state machine exists, but it dead-ends in-memory with no delivery layer — "adapter/rest/app/worker
crates MISSING; FSM real but in-memory only", so a new tenant cannot actually be registered or
provisioned through any callable surface.

The locked tenancy core already exists and is tested:

- `tenancy/core/tenant-lifecycle-domain` — the pure reconcile planner (one contract operation per
  pass, level-triggered convergence) over the locked G001 lifecycle FSM.
- `tenancy/core/tenant-lifecycle-kernel` — the `TenantLifecycleStore` storage port + persisted
  record shapes (the owned oya-data ordered-keyed shape).
- `tenancy/core/tenant-lifecycle-usecase` — `TenantLifecycleProvider`, the AIP-121/151/155
  resource-provider control plane (create lands `Provisioning`; lifecycle moves go through the
  operation ledger; `Retired` is a terminal tombstone; client-UUID idempotency throughout).

The tenant aggregate, the closed lifecycle state machine, and the resource/operation/idempotency
shapes come from `libs/oya-shared-platform-contracts-kernel` and
`libs/oya-shared-resource-provider-contract-kernel`. What is missing is a runnable delivery surface
that wires this core into HTTP so registration is E2E-callable.

## Decision

Commission the tenancy capability's tenant-lifecycle service as G006 slice 1: a **runnable
tenant registration / lifecycle delivery surface** in the ADR-0550 adapter/app shape, reusing the
locked lifecycle usecase + contract FSM wholesale (zero forked transition logic).

### D1 — Service shape (ADR-0131 / ADR-0550 seams)

- `tenancy/adapters/tenant-lifecycle-store-inmemory` — a faithful in-memory `TenantLifecycleStore`
  realization (ordered-keyed records, idempotency dedup table, monotonic operation-ledger ordinal)
  for single-node bring-up and acceptance tests. Deliberately transitional behind the kernel port
  (ADR-0510/ADR-0550): the G03 persistent (sqlx/Postgres) adapter plugs in behind the SAME port
  with no usecase or delivery change.
- `tenancy/facade/tenant-lifecycle-app` — the composition root and delivery surface: an axum HTTP
  service exposing register / read / list / provision / suspend / resume / retire over ONE shared
  decision core (`TenantLifecycleProvider`), plus the binary entrypoint. Owns NO lifecycle
  algorithm: every transition is decided by the contract FSM inside the usecase.

### D2 — REST surface

```text
POST   /v1/tenants                  — register a tenant (born Provisioning)
GET    /v1/tenants/{id}             — read the tenant's current state
GET    /v1/tenants                  — list tenants (AIP-158 paged)
POST   /v1/tenants/{id}/provision   — drive Provisioning -> Active (contract FSM)
POST   /v1/tenants/{id}/suspend     — Active -> Suspended
POST   /v1/tenants/{id}/resume      — Suspended -> Active
DELETE /v1/tenants/{id}             — retire (terminal; the id is never reused)
GET    /healthz                     — liveness probe
```

Mutating requests carry a client-generated `Idempotency-Key` header (canonical UUID, AIP-155 /
AWS-client-token shape): the same key replays the original outcome; a reused key with different
parameters is rejected. `:provision` starts an AIP-151 `Activate` operation and polls it to
completion in the same request, so the synchronous caller observes the converged `Active` state;
the operation ledger remains the single source of truth (a reconciler simply polls instead).

### D3 — Doctrine bindings

- **API-only service** (cli_surface_policy): no CLI surface; the REST surface + (later) declarative
  tenant CRs are the management surface; K8s-native env config (twelve-factor `LISTEN_ADDR`).
- **Closed FSM, single decision algorithm**: every state move goes through the contract transition
  function (ADR-0243 spirit: one decision algorithm); the delivery layer never invents transitions.
- **Panic-free** (ADR-0083 Tier-3): production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
- **Clean-arch face-direction** (ADR-0131): facade → { usecase (core), adapter } path-inward; the
  adapter depends only on the kernel port + contracts; ZERO core/ports → adapters/facade inversion.

### D4 — Testing ladder (founder standard; unit alone inadequate)

- adapter unit suite (ordered prefix-bounded paged scan, monotonic ledger ordinal, dedup/ledger
  round-trips);
- composition-root unit suite (view projection, provider-error → HTTP status mapping);
- a real integration/acceptance suite driving the FULL HTTP surface end-to-end against the
  in-memory store (no mocks): register → read (Provisioning) → provision → read (Active), the full
  provision/suspend/resume/retire FSM over HTTP, idempotency replay, AlreadyExists conflict,
  forbidden-transition precondition conflict, unknown-tenant 404, missing-idempotency-key 400, list.

### D5 — Destination (follow-up slices, not this one)

The in-memory store is transitional. The destination (follow-up slices) is the owned persistent
oya-data store behind the unchanged `TenantLifecycleStore` port, plus a tenant CRD + reconciler
driving desired-state convergence through the already-built reconcile planner, and the
provision → invite-initial-admin → first-run-setup bridge the audit names. Cutover litmus:
`TenantLifecycleProvider` and the REST surface survive unchanged; storage transports come and go
behind the port.

### D6 — Ownership + justification manifest (ADR-0555 D2)

Owner: `tenancy/OWNERS` = `axis-cloud-platform` (the existing tenancy capability owner). Files
commissioned by this decision:

`tenancy/adapters/tenant-lifecycle-store-inmemory/BUCK`,
`tenancy/adapters/tenant-lifecycle-store-inmemory/Cargo.toml`,
`tenancy/adapters/tenant-lifecycle-store-inmemory/src/lib.rs`,
`tenancy/facade/tenant-lifecycle-app/BUCK`,
`tenancy/facade/tenant-lifecycle-app/Cargo.toml`,
`tenancy/facade/tenant-lifecycle-app/src/lib.rs`,
`tenancy/facade/tenant-lifecycle-app/src/main.rs`,
`tenancy/facade/tenant-lifecycle-app/tests/acceptance.rs`,
`registry/catalog/tenancy-tenant-lifecycle-store-inmemory.yaml`,
`registry/catalog/tenancy-tenant-lifecycle-app.yaml`.

Files commissioned by the D7 authorization amendment (AUTH-005 BLOCKING fix):

`tenancy/ports/tenant-lifecycle-authz/BUCK`,
`tenancy/ports/tenant-lifecycle-authz/Cargo.toml`,
`tenancy/ports/tenant-lifecycle-authz/src/lib.rs`,
`tenancy/adapters/tenant-lifecycle-authz-pdp/BUCK`,
`tenancy/adapters/tenant-lifecycle-authz-pdp/Cargo.toml`,
`tenancy/adapters/tenant-lifecycle-authz-pdp/src/lib.rs`,
`tenancy/adapters/tenant-lifecycle-authz-pdp/cedar/tenancy.cedarschema`,
`tenancy/adapters/tenant-lifecycle-authz-pdp/cedar/tenancy-policies.cedar`,
`registry/catalog/tenancy-tenant-lifecycle-authz-port.yaml`,
`registry/catalog/tenancy-tenant-lifecycle-authz-pdp.yaml`.

### D7 — Authorization posture (AUTH-005 BLOCKING fix; fail-closed, default-deny, dogfood cloud-iam PDP)

An independent adversarial review found the D2 REST surface as first commissioned was an
**unauthenticated, unauthorized multi-tenant control plane**: any network caller could register,
suspend, or RETIRE (irreversible) any tenant by id, because handlers took the URL `{id}` straight
from the path with zero authentication and zero authorization. This is a CRITICAL trust-boundary
defect. This amendment closes it before the service can ship; the founder decision is to implement
fail-closed authz NOW, dogfooding the in-repo cloud-iam PDP, with no default-allow path.

**D7-a — Authz as a PORT the facade depends on (clean-arch faces, ADR-0131 / ADR-0562).** The
authorization decision is a PORT (`tenancy/ports/tenant-lifecycle-authz`,
`TenantLifecycleAuthorizer`); the facade (PEP) depends INWARD on the port, never on a concrete
engine. The decision ADAPTER (`tenancy/adapters/tenant-lifecycle-authz-pdp`) implements the port by
**dogfooding the repo's own embedded Cedar PDP substrate** — `oya-shared-pdp-kernel`'s
`PolicyDecisionPoint` realized by `iam-pdp-cedar`'s `CedarPdp` (ADR-0536 D-2;
cloud-iam IS the IdP/PDP per ADR-0559). The tenancy service is therefore a Policy Enforcement Point
over the SAME formally-verified Cedar engine cloud-iam ships, NOT a parallel authz stack. Face
direction is enumerated clean: no `core`/`ports` crate path-depends on any `adapter`/`facade`, and
the authz adapter has zero facade dependency.

**D7-b — Full-spectrum Cedar authz (RBAC + ABAC + PBAC).** The tenancy authz bundle (embedded Cedar
schema + policy seed in the adapter) is deny-by-default and forbid-overrides-permit. It carries a
**structural cross-tenant forbid** (a tenant-scoped principal can never administer another tenant,
whatever else is permitted), a **tenant-operator permit** (a principal whose proven tenant scope
equals the target tenant may administer it), and **platform-admin permits** for register/list. The
masterplan "RBAC" shorthand is read as full-spectrum Cedar-backed authz.

**D7-c — Fail-closed enforcement matrix.** Every per-tenant op (read/provision/suspend/resume/retire)
authorizes the VERIFIED caller against the TARGET `{id}` via the PDP — the URL `{id}` alone NEVER
authorizes. `register` (POST /v1/tenants) and `list` (GET /v1/tenants, which discloses all tenants)
require the platform-admin scope, a DISTINCT axis from any tenant scope. Outcomes:
unauthenticated → **401**; authenticated-but-unauthorized (incl. cross-tenant, and any fail-closed
engine refusal) → **403**. The verified bearer is compared constant-time (mirroring the established
intelligence-REST `constant_time_eq` doctrine — no naive `==`), and the bearer ALONE never grants the
tenant axis (a tenant operator must also assert its `x-oya-tenant` axis, bound only after the bearer
is verified).

**D7-d — Fail-closed boot (no default-allow, ever).** The composition root (`serve()`) REFUSES to
serve when the embedded authz bundle cannot compile/strict-validate (`BootError::Authz`), so a
misconfigured policy never degrades to default-allow, and when no bearer credential is configured at
all (`BootError::NoCredentialConfigured`). There is no authorizer-less router overload — the only way
to mount the routes is to supply a fail-closed authorizer.

**D7-e — Poison-lock hardening + concurrency seam (review MED findings).** The provider lock recovers
from poisoning via `PoisonError::into_inner` rather than propagating it, so one panicked handler can
fail its own request without bricking the service (no single-panic DoS). The single coarse global
lock is a documented, deliberate single-node bring-up seam: per-tenant / row-level concurrency moves
into the persistent store adapter behind the unchanged `TenantLifecycleStore` port (D5), which owns
the contention model.

**D7-f — Authorization audit trail (AC-W-13).** The PDP adapter surfaces the full
`AuthorizationOutcome { decision, decision_id, determining_policy_ids }` from `PdpOutcome.audit`
back through the port trait. The PEP (`authorize()` in the facade) emits ONE structured `tracing`
event per decision — `message = "tenancy.authz.decision"` with `decision_id`, `principal_id`,
`action`, `target_tenant`, `decision`, and `determining_policy_ids` — for EVERY call, allow AND
deny. No decision is ever silently discarded (AC-W-13: every Cedar decision is attributable via the
PDP-minted ULID `decision_id`).

> **SECURITY NOTE — self-asserted tenant axis (transitional debt, LOW / accepted):** The
> `x-oya-tenant` header that binds a tenant-operator bearer to a specific tenant is
> **self-asserted by the client under a shared operator secret**. The bearer verifies the operator
> credential class only; the tenant axis it carries is not independently verified by a per-tenant
> claim or mTLS certificate. A rogue operator who knows the shared secret can assert any tenant id.
> This is **accepted transitional debt** for the single-operator bring-up phase. The destination
> (ADR-0561) is a verified-claim / mTLS cutover where the tenant axis is bound cryptographically
> (per-tenant mTLS cert or a JWT claim signed by the IdP) and the shared operator secret is retired.
> Until that cutover, every tenant-operator request carries an un-verified tenant axis and must be
> treated accordingly in threat models. No new work rides this transitional model without explicit
> acknowledgment of this finding.

## Precedent

- **ADR-0559 / ADR-0553 service-commissioning pattern**: a runnable service in the ADR-0550
  kernel/adapter/app shape reusing a locked core wholesale, two-way door, born-accounted via a D6
  ownership + justification manifest — this ADR is the direct tenancy analogue.
- **Azure ARM resource providers / Google AIP-121/151/155**: one uniform contract per resource,
  client-UUID idempotency, async mutations as pollable operation resources — the shape the
  `TenantLifecycleProvider` already implements and this surface exposes over HTTP.
- **AWS SaaS Well-Architected isolation + cell-based architecture**: every tenant pinned to one
  home cell with an isolation posture — the tenant aggregate fields the register surface carries.

## Rejected

- **Forking the transition logic into the delivery layer** (ADR-0243: two decision algorithms must
  never coexist; the contract FSM in the usecase is the single engine).
- **A persistent store in slice 1** — the owned oya-data store + tenant CRD/reconciler are the
  destination (D5); fabricating a half-wired Postgres adapter now would be fake delivery. The
  in-memory store is an honest, valid ports/adapters realization for single-node bring-up.
- **A CLI management surface** — retirement-marked class (cli_surface_policy); tenant management is
  API + declarative state.

## Consequences

- The G006 vertical is open with a runnable, fully-tested tenant registration/lifecycle service;
  later slices are the persistent store, the tenant CRD/reconciler, and the onboarding bridge —
  each independently shippable.
- The auth/onboarding audit's tenant-registration delivery-chain gap (rest/app crates MISSING) is
  closed for the register → provision → read path.
- The AUTH-005 trust-boundary CRITICAL is closed (D7): the control plane is now fail-closed and
  default-deny on every route — unauthenticated → 401, unauthorized/cross-tenant → 403 — dogfooding
  the in-repo embedded Cedar PDP, with the binary refusing to serve without a valid authz provider.
- The authz decision is a port with a Cedar-PDP adapter; the destination policy-store delivery
  fabric (ADR-0536) swaps the seed bundle for content-addressed signed bundles behind the unchanged
  port with no facade change.
- Deleting the four crates restores the prior state (two-way door); no consumer points at the
  service yet.
