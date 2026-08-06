---
id: ADR-0572
title: "Fail-closed authz for the Cedar policy publish control plane (AUTH-005 remediation)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0702]
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0090]
amends: []
related: [ADR-0559, ADR-0561, ADR-0564, ADR-0566]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0572: Fail-closed authz for the Cedar policy publish control plane (AUTH-005 remediation)

## Status

**Accepted - 2026-06-23 (door: one-way).**

## Context

`iam/ports/policy-cedar-api` mounts `POST /policies/{policy_id}/versions/{version}` →
`publish_handler`, a **mutating multi-tenant control plane** that publishes Cedar policy versions.
Before this ADR the only "authz" was `validate_authorization` (`src/lib.rs`), which merely
cross-checks the **self-attested** `x-principal-*` / `x-authorization-*` headers for internal
consistency: there was no bearer/mTLS verification of a real principal and no PDP decision. An
attacker who can reach the socket sets those headers consistently and the request is accepted — the
**AUTH-005 unauthenticated-control-plane class** (the same class PR #768 shipped for
`tenancy/facade/tenant-lifecycle-app`).

The PR #780 authz-coverage gate (ADR-0566) discovered this surface and baselined it as category-A
genuine debt (`frozen_unauthenticated_surfaces`), escalating remediation to the IAM team as
**task #124**. This ADR records the remediation.

The repo already carries the correct fail-closed doctrine, which this remediation mirrors:
`intelligence/adapters/rest/src/lib.rs` (`constant_time_eq` bearer compare + a PDP `decide` port)
and the cloud-iam PDP caller-authn precedent (`iam/facade/cloud-pdp-app/src/mtls.rs`, ADR-0561 /
#38: authenticate the caller from a VERIFIED peer SVID; the request-supplied tenant is only ever a
cross-check input, never the source of truth).

## Decision

Make the publish surface fail-closed, with the authorization decision modelled as **ports** owned by
the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so they do
not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters** that
live outside this crate).

The new source file `iam/ports/policy-cedar-api/src/authz.rs` defines the ports, the reference
adapter, and the constant-time comparison utility that implements this decision.

1. **Verify a real principal (middleware layer, before body deserialization).** A new
   `PrincipalVerifier` port in `iam/ports/policy-cedar-api/src/authz.rs` verifies an unforgeable
   credential into a `VerifiedPrincipal`. Bearer verification runs in a `route_layer` middleware
   that operates on request Parts BEFORE the body is deserialized, so unauthenticated callers are
   rejected with 401 before any JSON parse occurs. The reference adapter
   `ConfiguredBearerPrincipalVerifier` (a **break-glass, single-principal adapter only** — NOT
   multi-tenant production) compares a bearer token in **constant time** (`constant_time_eq`, never
   a naive `==`) against a configured secret and binds the principal identity from configuration —
   NOT from the caller headers. The production W5 adapter is the cloud-iam mTLS/SPIFFE peer-SVID
   verifier (ADR-0561). Construction **refuses an empty secret** (boot-refusal doctrine).
   Unauthenticated → **HTTP 401**.

2. **Authorize via a PDP port with explicit scope.** A new `PublishAuthorizer` port decides
   `decide(principal, action = cedar.policy.publish, resource = {policy_id, scope, tenant})`. The
   `PublishResource` carries a `PublishScope` enum explicitly (`Tenant` or `Global`) so the PDP
   sees the **true blast radius** of the action. A global policy applies to ALL tenants; it MUST
   NOT be presented to the PDP as a per-tenant resource (that would silently authorize tenant-admins
   for platform-wide policy control — the CRITICAL escalation that a prior implementation contained
   by mapping `scope:global → operator_tenant`). Default-deny: any deny/refusal → **HTTP 403**. The
   cloud-iam Cedar PDP client (ADR-0559) is the canonical W5 adapter. In test builds
   (`panic = unwind`) `catch_unwind` wraps the authorizer call and maps panics to `Refused → 403`;
   in release builds (`panic = "abort"`, `Cargo.toml [profile.release]`) `catch_unwind` is a no-op
   and a panicking adapter aborts the process. The real production guarantee is the
   `PublishAuthorizer` adapter contract: adapters MUST NOT panic and MUST map every fault to
   `Err(Refused)`.

3. **Refuse to serve without a provider.** `CedarPolicyRestState` carries a **required,
   non-optional** `CedarPolicyAuthzProvider`; there is no constructor that yields router state
   without it, so the binary/router can never mount this control plane with a default-allow
   fallback.

4. **Cross-tenant guard.** The verified tenant is authoritative: a request whose operator
   (`x-tenant-id`) or self-attested principal tenant differs from the verified tenant is denied
   (403) before the PDP decision.

5. **Type-level defense-in-depth at the boundary API.** `publish_cedar_policy_from_api` (the
   public crate API, `iam/ports/policy-cedar-api/src/lib.rs`) requires a `&VerifiedPrincipal`
   as its first argument. External crates cannot build this type by struct literal (private fields,
   `pub(crate)` constructor); they must run a real `PrincipalVerifier`. This is structural
   defense-in-depth — it prevents accidental bypass and proves a verifier ran — but it is NOT a
   cryptographic guarantee: hostile in-process code could construct its own
   `ConfiguredBearerPrincipalVerifier` with a known secret. The real security comes from the
   combination of bearer middleware + PDP decision + active principal cross-check in
   `publish_cedar_policy_from_api`. The REST handler is not the only guard.

6. **Audit fields derived from the verified identity.** The `CedarPolicyApiAuthorization` fields
   `principal_id` and `tenant_id` are populated from the `VerifiedPrincipal` (not from the
   caller-supplied `x-authorization-principal-id` / `x-authorization-tenant-id` headers), so the
   audit trail cannot be forged by header manipulation. The `decision_id` field records the
   caller-supplied `x-authorization-decision-id` as a **correlation hint** only (not an
   authorization grant); a future fast-follow will extend the `PublishAuthorizer` port to return a
   server-derived decision record, making the decision id fully authoritative end-to-end.

The legacy `validate_authorization` header-consistency checks run after the verified-principal + PDP
gate as non-authoritative correlation/consistency validation, not as the authorization boundary.

The AUTH-005 baseline entry for this surface is **removed** from
`ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json`
`frozen_unauthenticated_surfaces` (shrink-only is allowed). The gate now recognizes `publish_handler`
as covered (it invokes `verify_principal` + `ensure_authorized`, recognized `authz_guard_idents`),
so the surface is no longer a baselined violation and the gate remains green.

## Consequences

- The publish surface is fail-closed: unauthenticated → 401, authenticated-but-unauthorized → 403,
  cross-tenant → 403, authorized → 201. The self-attestation bypass is closed (attacker-set
  `x-authorization-*` headers with no verified credential → 401, proven by RED/GREEN router tests).
- The authz decision is a port; the cloud-iam PDP client and the bearer/SVID credential store are
  adapters wired by the composition root (deferred to the embedding application; this PR ships the
  ports + the in-process reference bearer verifier + a fail-closed router).
- The authz-coverage baseline shrinks by one (AUTH-005 class debt retired for this surface).

## Alternatives considered

- **Inline a concrete PDP in the handler.** Rejected: violates clean architecture (ADR-0131); the
  decision is a port the facade depends on, the PDP is the adapter.
- **Keep header self-attestation only.** Rejected: that IS the AUTH-005 vulnerability.
- **Optional provider with a default-allow fallback.** Rejected: a control plane must refuse to
  serve without authz (new-HTTP-surface fail-closed doctrine).
