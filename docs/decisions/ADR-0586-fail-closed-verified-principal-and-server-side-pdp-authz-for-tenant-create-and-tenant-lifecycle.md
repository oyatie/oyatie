---
id: ADR-0586
title: "Fail-closed verified-principal + server-side PDP authorization for tenant.create and the tenant-lifecycle operator scope (C7/C8)"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-06-23
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0083, ADR-0131]
related: [ADR-0510, ADR-0515, ADR-0564, ADR-0566, ADR-0581]
related_specs:
  - /specs/capability-registry.json
milestone: W2
---

# ADR-0586: Fail-closed verified-principal + server-side PDP authz for tenant.create and the tenant-lifecycle operator scope

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; BLOCKED pending adversarial security
review. Door: two-way — additive ports + a required, non-optional constructor param and an
unforgeable token behind already-shaped clean-architecture seams, reversible by removing the ports
and reverting the two boundary functions without unwinding any SSOT. On approval the founder flips
this to Accepted and admits the born-unpropagated decision into
`ci/facade/baseline-ratchet/gate-baseline.signoff.json` per the established
new-Accepted-ADR door precedent, then propagates it into the masterplan/roadmap faces.)**

## Context

A Wave-2 capability-tenancy security review surfaced two related self-attested-authorization defects
in the tenancy delivery surface — the AUTH-005 class (PR #768 shipped an unauthenticated mutating
control plane that passed all gates green; the founder mandate is that this must be impossible to
ship):

### C7 — `tenancy/facade/tenant-lifecycle-app` (operator scope self-attested via header)

The tenant-lifecycle PEP authenticates a tenant operator with a **shared** bearer
(`TENANCY_TENANT_OPERATOR_TOKEN`) and then derived the operator's tenant axis **directly from the
caller-supplied `x-oya-tenant` header** (`authenticate_caller`, `src/lib.rs:446`). The embedded Cedar
PDP's structural forbid checks `principal.tenant_id == resource.tenant_id`, but BOTH sides were fed
from the same self-attested header + the path id. So any holder of the shared operator bearer could
set `x-oya-tenant: <victim>` on `/v1/tenants/<victim>/...` and, because axis == target, be permitted
to suspend/resume/retire **any** tenant. The pre-fix cross-tenant test only caught the case where the
attacker set a *mismatched* header; a matched self-attested header was an allow. The bearer proved
the caller was *some* operator but never **which tenants** it may act for.

### C8 — `tenancy/ports/api` (authorization read from caller-supplied decision fields)

`validate_authorization` (`src/lib.rs:534`) authorized `tenant.create` purely from a caller-supplied
`TenantApiAuthorization { decision_id, tenant_id, principal_id, allowed_surfaces }` DTO: it
cross-checked those fields against the **also caller-supplied** `TenantApiPrincipal`. A caller simply
supplied `allowed_surfaces: ["tenant.create"]` plus a matching principal and was authorized. There
was no PDP and no verified principal anywhere on the path; the entire authorization was self-attested.

The proven fail-closed doctrine already lives in `intelligence/adapters/rest`, in the cedar-publish
remediation (ADR-0572 / #815), and in the workload-principal lifecycle remediation (ADR-0581 / #816).
This ADR applies the same doctrine to both tenancy surfaces, baking in the hard-won lessons from
those rounds (an unforgeable token, a server-side PDP port, true blast-radius binding to the target,
and faults-map-to-deny).

## Decision

### Common posture

- **Default-deny**: no code path reaches the mutation/sensitive op without passing the gate. 401
  without a verified principal, 403 without authorization.
- **Unforgeable verified principal**: authority derives ONLY from a verified credential. Any
  `VerifiedTenantPrincipal` / membership-bound scope has **private fields + a `pub(crate)`-only
  constructor + accessors + a `#[cfg(test)]` test constructor** — caller-supplied
  `x-principal-*` / `x-authorization-*` / `allowed_surfaces` fields NEVER authorize. (The #815-round1
  mistake — a public-field forgeable token — is deliberately not repeated.)
- **Server-side PDP as a clean port**: the decision is made server-side from trusted inputs. The port
  CONTRACT (documented on the trait) requires adapters to map every fault (error/timeout/
  unavailability) to a deny `Result` and to NOT panic. Release builds use `panic = "abort"`, so
  `catch_unwind` is not a backstop — the ADR does not overclaim panic→403.

### C8 — `tenancy/ports/api`

1. The caller-supplied `TenantApiAuthorization` DTO and the `principal`/`authorization`
   cross-check are **removed**. `TenantCreateApiRequest.principal` is now an unforgeable
   `VerifiedTenantPrincipal`, NOT deserialized from the request — minted only by a credential verifier
   (`TenantPrincipalVerifier`; the reference `BearerTenantPrincipalVerifier` uses a `constant_time_eq`
   bearer; production swaps mTLS/SPIFFE or a cloud-iam credential store).
2. A new `TenantCreateAuthorizer` **PDP port** the boundary owns: `decide(&TenantCreateAuthzRequest)
   -> Result<bool, AuthzFault>`. `create_tenant_from_api` now **requires** the authorizer and asks it
   before any directory mutation. `Ok(false)` → 403 `AUTHORIZATION_DENIED`; `Err(_)` → 403
   `AUTHORIZATION_FAULT` (fail-closed). Concrete PDP clients live OUTSIDE the crate (owned-W5 shape).
3. **True blast radius / no IDOR**: the `TenantCreateAuthzRequest` carries `caller_tenant_id`
   (verified) AND `target_tenant_id` on **separate axes**, where the target is the **path** tenant id
   (a trusted source, verified equal to `body.tenant_id` first) — never flattened to the caller's own
   tenant. A cross-tenant create is deniable AT THE PDP.
4. Audit metadata (`TenantCreateMetadata`) and the idempotency-ledger key now reflect the **verified**
   caller, never a caller-supplied header/grant.

### C7 — `tenancy/facade/tenant-lifecycle-app`

1. A new `TenantMembershipResolver` **server-side port** (in the authz port crate): given the VERIFIED
   operator principal id, it returns the exact set of tenants that operator is **assigned** to, from a
   trusted source (`assigned_tenants(&str) -> Result<Vec<String>, MembershipFault>`). Default-deny: an
   unknown operator resolves to the EMPTY set; any store fault maps to a deny.
2. `authenticate_caller` no longer derives the tenant axis from `x-oya-tenant`. It resolves the
   verified operator's assigned tenants server-side and binds the axis ONLY to a tenant in that set:
   the `x-oya-tenant` header may **SELECT** among assigned tenants but can never **grant** an
   unassigned one. An unassigned selection → 403; no assigned tenants → 403; a membership fault → 403;
   a multi-membership operator that omits a selection → 400 `TENANT_SELECTION_REQUIRED`.
3. The resolver is a **required, non-optional** `AppState` field; `build_router` /
   `build_inmemory_router` / `build_postgres_router` all require it — there is no allow-all default and
   the router cannot be built without the seam. The embedded Cedar PDP still backstops the
   membership-bound axis against the target `{id}` (the structural forbid is unchanged).
4. The reference `InMemoryTenantMembershipResolver` (composition-root adapter, seeded from
   `TENANCY_TENANT_OPERATOR_MEMBERSHIPS`) is the W5-shaped seed; production swaps a per-credential
   verifier + a cloud-iam membership adapter behind the unchanged port.

## Consequences

- Both surfaces are fail-closed by construction. The C8 boundary cannot create a tenant without a PDP
  allow over the verified caller + the path-derived target; the C7 facade cannot grant an operator a
  tenant it is not a server-side member of.
- The cloud-ci authz-coverage gate already reports `tenancy/facade/tenant-lifecycle-app` GREEN by
  authz detection (it carries `authenticate_caller` + `authorize()` per route); this change tightens
  the *scope binding* behind that gate without changing the route signature, so no NEW unauthenticated
  surface is introduced. `tenancy/ports/api` is a pure library boundary (no router), so it is not a
  route-discovery surface for the authz-coverage gate; its fix is proven by the crate's RED/GREEN
  tests.
- A production tenancy deployment must provide a per-operator membership source
  (`TENANCY_TENANT_OPERATOR_MEMBERSHIPS` for the seed adapter, or a cloud-iam adapter); an absent
  source means every per-tenant operator op denies (default-deny), never an open surface.
- No new **source** file is introduced: both ports, the unforgeable token, and the reference adapters
  are additive edits inside the existing boundary/port crates. The born-accounting `unjustified` count
  therefore does not regress from new code; the only new tracked file is this ADR (justified below).

## Accepted residuals (not security gaps)

### Shared operator bearer binds all operators to one reference principal (LOW; reference adapter only)

The reference `BearerTenantPrincipalVerifier` binds every caller presenting the shared operator bearer
to one stable operator principal id (`tenant-operator`), so the seed membership set is per-deployment,
not per-human-operator. This is an **accepted residual of the reference adapter only**: it is the
single-node bring-up seed. The structural fix is the `TenantMembershipResolver` PORT — a production
mTLS/SPIFFE/OIDC verifier yields a per-credential subject as the membership key with no change to the
PEP. The vulnerability the founder mandate targets (self-attested tenant selection) is closed at the
port regardless of the reference verifier's coarseness.

### Existence behaviour after authn (LOW)

After a verified operator selects an ASSIGNED tenant, a non-existent target id still returns 404 and a
denied-by-PDP one returns 403 — the same verified-caller-only existence distinction accepted in
ADR-0581. The information is disclosed only to a membership-bound caller already trusted to operate
its assigned tenants. If re-evaluated, the fix is a uniform 403 after authn.

## Alternatives considered

- **Keep `x-oya-tenant` as the axis and rely on the Cedar forbid.** Rejected: the forbid compares
  `principal.tenant_id` to `resource.tenant_id`, both fed from the same self-attested header + path —
  it cannot distinguish a legitimate operator from one self-selecting a victim. The authority MUST
  come from a server-side membership source.
- **Demote the C8 `TenantApiAuthorization` to a correlation id.** Rejected as insufficient: any
  caller-supplied field that participates in the allow decision is forgeable. The grant is removed
  entirely and replaced by a server-side PDP decision.
- **`decide()` infallible + `catch_unwind`.** Rejected (same reasoning as ADR-0581): `catch_unwind`
  cannot catch `abort`, muddies the ADR-0083 Tier-3 panic-free contract, and is the wrong tool. A
  `Result<bool, AuthzFault>` / `Result<_, MembershipFault>` is fail-closed by type.

## Files

This decision introduces ONE new tracked file (born-accounting justification — the verbatim tracked
path is named here so the total-accounting registry resolves its `justification_ref` to this ADR and
the firewall `unjustified` count does not regress):

- docs/decisions/ADR-0586-fail-closed-verified-principal-and-server-side-pdp-authz-for-tenant-create-and-tenant-lifecycle.md — this decision record.

All code changes are additive edits inside EXISTING crates (no new source file):

- tenancy/ports/api/src/lib.rs — removes `TenantApiAuthorization`, adds the unforgeable
  `VerifiedTenantPrincipal`, the `TenantPrincipalVerifier` + `BearerTenantPrincipalVerifier`, and the
  `TenantCreateAuthorizer` PDP port; rewires `create_tenant_from_api` to a server-side decision.
- tenancy/ports/api/tests/tenant_create_api.rs — RED/GREEN seam tests (forged/absent credential,
  cross-tenant deny under an otherwise-allow authorizer, PDP-deny, PDP-fault, happy path).
- tenancy/ports/tenant-lifecycle-authz/src/lib.rs — adds the `TenantMembershipResolver` port +
  `MembershipFault`.
- tenancy/facade/tenant-lifecycle-app/src/lib.rs — binds the operator tenant axis to the server-side
  `TenantMembershipResolver` (required `AppState` field), adds the reference
  `InMemoryTenantMembershipResolver`.
- tenancy/facade/tenant-lifecycle-app/tests/acceptance.rs — the headline C7 RED test (an operator
  cannot self-attest an unassigned victim tenant) + membership-bound selection coverage.
