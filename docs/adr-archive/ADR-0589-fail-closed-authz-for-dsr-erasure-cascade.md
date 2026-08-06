---
id: ADR-0589
title: "Fail-closed authz for the DSR erasure cascade (AUTH-005 / Wave-2b remediation)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-702]
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0536, ADR-0561, ADR-0572, ADR-0573]
amends: []
related: [ADR-0561, ADR-0572, ADR-0573]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0589: Fail-closed authz for the DSR erasure cascade (AUTH-005 / Wave-2b remediation)

## Status

**Proposed - 2026-06-23 (door: one-way).**

## Context

`compliance/ports/dsr-usecase` is the application boundary for the GDPR **erasure cascade**
(`dsr.cascade.execute`, `execute_dsr_cascade_from_api`). A single accepted request fans out
irreversible erasure / correction dispatches across every store axis (a non-exhaustive set
including `saas`, `workspace`, `vertical`, `cloud`, `search`, `ads`, `analytics`) that holds a
data subject's records,
then assembles the signed DSR completion record. It is one of the most destructive,
compliance-critical surfaces in the repo.

Before this ADR, the only "authorization" was `validate_authorization`, which merely cross-checked a
**caller-supplied** `PlatformDsrApiAuthorization` blob
(`{decision_id, tenant_id, principal_id, allowed_surfaces}`) for internal consistency against the
(also caller-supplied) `PlatformDsrApiPrincipal`. The decision was literally:

```rust
if !authorization.allowed_surfaces.iter().any(|s| s == "dsr.cascade.execute") {
    return Err(PlatformDsrApiError::AuthorizationDenied { .. });
}
```

An attacker who can reach the boundary forges that blob — sets `allowed_surfaces` to include
`dsr.cascade.execute`, and the tenant/principal to match the (forged) principal — and the request
authorizes an **irreversible erasure cascade** with NO verified principal and NO server-side policy
decision. This is the AUTH-005 caller-supplied-authz class (the codex Wave-2b `dsr-authz` CRITICAL):
the "PDP decision" was just the caller echoing the surface name back at the gate. There was no
unforgeable identity and no `decide()` call.

This is the same class already remediated for the Cloud KMS crypto control plane (ADR-0573), the
Cedar policy publish control plane (ADR-0572), and the cloud-iam PDP caller-authn precedent
(ADR-0561). This ADR applies the identical fail-closed doctrine to the DSR erasure cascade.

## Decision

The erasure cascade is UNREACHABLE without (1) a verified principal and (2) a passing server-side
PDP decision. The caller-supplied `allowed_surfaces` field is removed entirely.

1. **Unforgeable verified principal.** A new `compliance/ports/dsr-usecase/src/authz.rs` introduces
   `VerifiedDsrPrincipal` (private fields, `pub(crate)` constructor, public accessors, `cfg(test)`
   test-only constructor). External crates cannot build one by struct literal or any public API;
   they must run a real `DsrCascadePrincipalVerifier` port. The reference adapter
   `ConfiguredBearerDsrPrincipalVerifier` compares the bearer token in **constant time**
   (`constant_time_eq`, never `==`) and binds the identity from its configuration — never from the
   caller headers — and REFUSES construction with an empty secret or bound identity (boot-refusal).

2. **Caller-asserted blob is cross-check-only.** `PlatformDsrApiAuthorization` is reduced to a
   single non-authoritative `decision_id` correlation field (telemetry only; confers no authority).
   `PlatformDsrApiPrincipal` survives as a forgeable cross-check that MUST agree with the verified
   identity; a mismatch is `VerifiedPrincipalMismatch` / `VerifiedTenantMismatch` (403). The
   idempotency key and the completion record are derived from the VERIFIED principal, not the blob.

3. **Server-side PDP decision (true blast-radius).** `execute_dsr_cascade_from_api` now requires a
   `&VerifiedDsrPrincipal` and a `&DsrCascadeAuthzProvider`, and calls the `DsrCascadeAuthorizer`
   PDP port (`decide`) for `action = dsr.cascade.execute` over a `DsrCascadeResource` bound to the
   VERIFIED principal's tenant (the target tenant whose stores are erased, cross-checked equal to
   the body tenant) and the TARGET dsr id from the trusted path binding — never to a forged surface
   list. A cross-tenant cascade is therefore deniable: the PDP sees the real
   `{principal, target tenant, dsr}` tuple.

4. **PDP-fault-denies.** The `DsrCascadeAuthorizer` trait contract maps every fault (deny, timeout,
   network, unavailability) to `Err`, and the boundary maps `Err(Denied | Refused)` to
   `CascadeAuthorizationDenied` (403, NOT 500). The provider's `catch_unwind` is a test-only
   best-effort backstop and is explicitly NOT relied upon in production (`panic = "abort"`).

5. **Authn-before-body is the edge's responsibility.** This crate is the app-boundary library; it
   has no live HTTP/gRPC router (no REST adapter is wired yet). When a transport binary is added it
   MUST verify the credential over request `Parts` (route_layer / `FromRequestParts` / middleware +
   `DefaultBodyLimit`) and pass the resulting `VerifiedDsrPrincipal` here BEFORE body deserialization
   — mirroring `intelligence/adapters/rest` and the ADR-0573 KMS edge. The unforgeable type makes
   this the only way to reach `execute_dsr_cascade_from_api`.

### Clean architecture (ADR-0131 / ports-for-owned-stack)

`DsrCascadePrincipalVerifier` and `DsrCascadeAuthorizer` are PORTS owned by this boundary crate. The
concrete cloud-iam Cedar PDP client (the W5 destination, embedded-PDP per ADR-0536 D-2) and the
bearer/SVID credential store are ADAPTERS that live OUTSIDE this crate. The port shapes model the
destination decision surface so they do not change at cutover; transient infra is absorbed by the
adapter.

## Born-accounting impact

This change adds one new module file (`compliance/ports/dsr-usecase/src/authz.rs`) inside an
existing crate; it introduces **no new crate, no new BUCK target, and no new dependency** (the authz
module is std-only). The existing `rust_library` glob (`src/**/*.rs`) and the existing
`rust_test` target already cover the new file and the rewritten integration tests, so the
generated born-accounting faces require no new rows — only the byte-parity refresh that the
materialize step performs. The public API of `execute_dsr_cascade_from_api` changes signature
(adds `verified` + `authz` parameters) and `PlatformDsrApiAuthorization` drops three authority
fields; these are source-level breaks absorbed in the same change set (the only repo consumer was a
naming-lint test row, unaffected).

## Consequences

- A forged principal or forged authorization blob can no longer trigger an erasure cascade: the
  RED tests assert 403 (PDP deny, PDP fault, forged caller principal, forged body tenant) with no
  directory/ledger mutation, and the verifier refuses missing/wrong credentials.
- The boundary now exposes a 401 (`PrincipalUnauthenticated`) status for the edge to map when no
  verifiable credential is presented.
- The composition root MUST bind a `DsrCascadeAuthzProvider`; there is no default-allow fallback and
  no `Default` impl, so a misconfigured deployment fails closed rather than open.

## Alternatives considered

- **Keep the blob, add a signature.** Rejected: signing a caller-supplied grant still trusts the
  caller to assemble it; the authority must come from a server-side decision over a verified
  identity (the AUTH-005 lesson).
- **Bind the PDP resource to the caller's own tenant.** Rejected: that flattens global/cross-tenant
  scope to the caller tenant and re-introduces the blast-radius escalation; the resource tenant is
  the verified target tenant, cross-checked against the body.
