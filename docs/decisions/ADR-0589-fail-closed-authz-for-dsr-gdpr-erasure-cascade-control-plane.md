---
id: ADR-0589
title: "Fail-closed verified-principal + PDP authorization for the DSR/GDPR-erasure cascade control plane (dsr.cascade.execute)"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-23
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0083, ADR-0105, ADR-0131]
related: [ADR-0510, ADR-0515, ADR-0561, ADR-0572, ADR-0581]
related_specs:
  - /specs/capability-registry.json
milestone: W2
---

# ADR-0589: Fail-closed authz for the DSR/GDPR-erasure cascade control plane

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; BLOCKED pending adversarial security
review. Door: two-way — additive ports + required boundary-fn params behind an already-shaped
clean-architecture seam, reversible by removing the two ports and the adapter without unwinding any
SSOT. On approval the founder flips this to Accepted and admits the born-unpropagated decision into
`cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json` per the established
new-Accepted-ADR door precedent — mirroring the ADR-0572 admission — then propagates it into the
masterplan/roadmap faces.)**

## Context

`compliance/ports/dsr-usecase` (`compliance-dsr-usecase`) is the boundary crate for
`POST /privacy/dsr/{dsr_id}:cascade-execute` (surface `dsr.cascade.execute`). This is a MUTATING,
multi-tenant, **GDPR/CCPA data-subject-erasure** control plane: executing the cascade records the
irreversible completion that proves a data subject's records were erased across every affected store.

Before this decision the only "authorization" was `validate_authorization`, which merely cross-checked
a CALLER-SUPPLIED `PlatformDsrApiAuthorization` blob — `{decision_id, tenant_id, principal_id,
allowed_surfaces}` — for internal consistency: it confirmed that the caller's self-asserted
authorization tenant/principal matched the caller's self-asserted request tenant/principal and that
the caller's self-asserted `allowed_surfaces` list contained `dsr.cascade.execute`. Every one of
those fields is controlled by the caller. Any caller who could reach the socket simply set
`allowed_surfaces = ["dsr.cascade.execute"]` with a matching `{tenant_id, principal_id}` and
**self-authorized an erasure cascade with no verified principal and no server-side policy decision**.
This is the whole-repo-review CRITICAL **C16** (the AUTH-005 class: PR #768 shipped an unauthenticated
multi-tenant control plane that passed all gates green). A forged principal could fabricate a tenant
and trigger an erasure cascade.

The proven fail-closed doctrine already lives in `intelligence/adapters/rest` (a PDP `decide` with
separate principal/resource tenant axes + `constant_time_eq` bearer), in the cedar-publish
remediation (ADR-0572 / #815), in the in-flight Cloud KMS crypto control-plane remediation (#817),
and in the workload-principal lifecycle remediation (ADR-0581 / #816). This ADR applies the same
doctrine to the DSR/GDPR-erasure cascade boundary, baking in the hard-won lessons from those reviews
(forgeable public-field token, cross-tenant flattening / IDOR, PDP-fault-must-deny, authority recorded
from the verified principal not caller headers).

## Decision

1. **Two clean PORTS owned by the boundary crate** (`compliance-dsr-usecase`), concrete adapters
   outside it (owned-W5 shape, ADR-0131):
   - `PrincipalVerifier::verify_principal(&CallerCredential) -> Result<VerifiedDsrPrincipal,
     PrincipalVerificationError>` — caller authentication. `VerifiedDsrPrincipal` has **private fields
     and only a `pub(crate)` constructor**, so external crates cannot mint one by struct literal or
     any public API; it can be obtained ONLY by running a real verifier that proved an UNFORGEABLE
     credential (a `constant_time_eq` bearer in the reference `ConfiguredBearerPrincipalVerifier`;
     an mTLS/SPIFFE peer-SVID adapter in production, ADR-0561). This deliberately does NOT repeat the
     ADR-0572 round-1 mistake of a public-field forgeable token. Caller-supplied `x-principal-*` /
     `x-authorization-*` fields never authorize.
   - `DsrCascadeAuthorizer::ensure_authorized(&VerifiedDsrPrincipal, &DsrCascadeResource) ->
     Result<(), DsrCascadeAuthorizationError>` — the server-side PDP seam (`decide`) for
     `action = dsr.cascade.execute`. Default-deny; the documented adapter contract requires every
     fault (error/timeout/unavailability) to map to `Err(Refused)` and to NOT panic (the
     `catch_unwind` in the provider is a test-only best-effort backstop — release is `panic = "abort"`
     so it is a no-op in production; the ADR does not overclaim panic→403 in prod).

2. **Required, non-optional authz seam.** `execute_dsr_cascade_from_api` now REQUIRES a
   `&VerifiedDsrPrincipal` and a `&DsrCascadeAuthzProvider` (which is itself constructed from a
   non-optional verifier + authorizer). There is no `Default` and no allow-all fallback; the mutation
   path cannot be reached without a configured authz seam. The reference
   `ConfiguredBearerPrincipalVerifier` REFUSES construction with an empty bearer secret or empty bound
   identity (boot-refusal), so a process that cannot prove a credential root can never authenticate.

3. **The boundary fn USES the verified principal.** `execute_dsr_cascade_from_api` actively
   cross-checks the request's self-asserted `principal.{principal_id, tenant_id}` against the VERIFIED
   identity and rejects any mismatch (403) — the verified principal is not an unused parameter. The
   audit/idempotency key is derived from the VERIFIED principal, never the caller headers.

4. **True blast radius / no IDOR.** The PDP resource is built by
   `DsrCascadeResource::for_target(dsr_id, target_tenant)` from TRUSTED inputs — the path `dsr_id`
   (bound equal to the body `dsr_id`) and the body tenant (which `validate_tenant_binding` has bound
   equal to the header tenant and the verified principal's tenant). A cross-tenant target is therefore
   deniable AT THE PDP and never flattened to the caller's own tenant. A platform/global-scoped
   erasure (`target_tenant == "platform"`, the `DSR_PLATFORM_TENANT_SENTINEL`) presents a DISTINCT
   `DsrCascadeScope::Platform` resource that requires platform-admin authority — never the caller's
   tenant — closing the ADR-0572 global-scope CRITICAL (a tenant admin cannot self-authorize a
   platform-wide erasure).

5. **Authn before body; PDP-fault → deny.** At the HTTP edge the `PrincipalVerifier` runs in a
   `route_layer`/middleware on request `Parts` (`CallerCredential` from headers) BEFORE body
   deserialization, with an explicit `DefaultBodyLimit`; an absent/invalid credential is a `401`
   (`PlatformDsrApiError::Unauthenticated`) and never reaches the PDP or any mutation. A PDP `Refused`
   maps to a fail-closed `403` (`AuthorizationFault`); an explicit `Denied` maps to `403`
   (`AuthorizationDenied`) with a distinct error code so a PDP outage is distinguishable from a real
   policy block during incident response. Both never `500`, never allow.

6. **Caller-supplied authorization authority removed.** `PlatformDsrApiAuthorization` (with
   `{tenant_id, principal_id, allowed_surfaces}`) is replaced by
   `PlatformDsrApiAuthorizationCorrelation`, which carries ONLY an optional `decision_id` as a
   non-authoritative log-join correlation id — it grants nothing and is never compared against an
   allow-list. The request fingerprint and audit records reflect the VERIFIED principal, not caller
   headers.

## Consequences

- The fix is fail-closed by construction. There is no public way for an external caller to obtain a
  `VerifiedDsrPrincipal` without a real credential, so a forged `{tenant, principal, allowed_surfaces}`
  request is worthless; the mutation entry point is unreachable without authentication + a server-side
  allow decision.
- **Authz-coverage baseline note.** `compliance-dsr-usecase` is a pure boundary/use-case crate with no
  axum router of its own (the REST adapter that mounts the surface lives outside this crate and is not
  yet in the tree), so it is not in the `cloud-ci-authz-coverage` scan_roots and there is no
  `frozen_unauthenticated_surfaces` entry for `dsr.cascade.execute` to shrink. The gate is enforced
  STRUCTURALLY by the required-`&VerifiedDsrPrincipal` + required-`&DsrCascadeAuthzProvider` signature
  on the only mutation entry point. When the REST adapter is added it MUST run the verifier in a
  pre-body middleware and pass the verified principal + provider into this boundary fn — at which point
  its router signature is born already-authorized.
- A production deployment must provide a credential root for the `PrincipalVerifier` adapter (a bearer
  secret for the break-glass adapter, or an mTLS/SPIFFE trust anchor for the W5 adapter); the verifier
  refuses to construct otherwise.
- The reference `ConfiguredBearerPrincipalVerifier` is documented BREAK-GLASS-only (one static
  identity bound to a shared secret); a richer cloud-iam Cedar PDP + peer-SVID verifier swaps in
  behind the two ports without touching this boundary crate.

## Accepted residuals (not security gaps)

### Existence/conflict oracle (LOW)

After a successful authz step, a duplicate cascade returns `409 CascadeAlreadyCompleted`. A verified,
authorized caller can therefore distinguish "already completed" from "newly accepted". This is
disclosed only to a caller who has already passed verification AND the PDP allow decision for that
exact tenant/resource, so the blast radius is confined to already-authorized principals. If
re-evaluated, the fix is to return a uniform accepted/idempotent envelope.

### constant_time_eq length leak (LOW)

`constant_time_eq` leaks whether the two inputs have equal length via the XOR seed (the same residual
accepted in the repo reference). Bearer tokens are fixed-length secrets in practice; use an
HMAC-SHA256 compare if length-hiding is required. The production path is the mTLS/SPIFFE adapter,
which does not bearer-compare at all.

## Alternatives considered

- **Keep `validate_authorization` but add a verified principal alongside.** Rejected: leaving the
  caller-supplied `allowed_surfaces` check in place keeps a forgeable code path and invites a future
  caller to rely on it. The authority fields are removed outright.
- **Run the PDP only at the REST edge, not in the boundary fn.** Rejected for this pure boundary
  crate: there is no router here, and requiring the provider IN the boundary fn makes the gate
  inseparable from the mutation (an in-process or future caller cannot bypass it). When the REST
  adapter is added it ALSO runs the verifier pre-body, as defense-in-depth.
- **`ensure_authorized` infallible + `catch_unwind` for fault isolation.** Rejected: `catch_unwind`
  cannot catch `abort` (release is `panic = "abort"`), so a `Result` that maps every fault to
  `Err(Refused)` is the only honest fail-closed contract. The provider's `catch_unwind` is documented
  test-only.

## Files

This decision introduces one new source file (born-accounting justification — the verbatim tracked
path is named here so the total-accounting registry resolves its `justification_ref` to this ADR and
the firewall `unjustified` count does not regress):

- compliance/ports/dsr-usecase/src/authz.rs — the fail-closed authz seam: the `PrincipalVerifier` and
  `DsrCascadeAuthorizer` ports, the unforgeable `VerifiedDsrPrincipal` token, the `DsrCascadeResource`
  / `DsrCascadeScope` blast-radius types, the `DsrCascadeAuthzProvider` composition, and the
  BREAK-GLASS reference `ConfiguredBearerPrincipalVerifier` adapter (the W5-shaped reference
  implementation), per ADR-0131 ports-and-adapters layering.

The two ports, the `VerifiedDsrPrincipal` token, the rewired `execute_dsr_cascade_from_api` /
`validate_platform_dsr_cascade_execute_request` gate, and the demotion of
`PlatformDsrApiAuthorization` to `PlatformDsrApiAuthorizationCorrelation` are additive/edit changes
inside the existing compliance/ports/dsr-usecase/src/lib.rs boundary crate; the RED/GREEN proofs are
in compliance/ports/dsr-usecase/tests/dsr_cascade_execute_api.rs.
