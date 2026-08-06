---
id: ADR-0591
title: "Fail-closed authz for the Cloud FinOps report API (AUTH-005 capability-billing remediation)"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0090, ADR-0559, ADR-0561, ADR-0566, ADR-0572]
amends: []
related: [ADR-0559, ADR-0561, ADR-0566, ADR-0572]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0591: Fail-closed authz for the Cloud FinOps report API (AUTH-005 capability-billing remediation)

<!--
  Born-accounting justification (ADR-0552 / accounting-registry): the new source
  file `billing/ports/finops-api/src/authz.rs` is justified by this ADR. The
  accounting-registry derives `justification_ref` by matching a tracked path
  mentioned verbatim in an ADR body, so the verbatim path token below (and in the
  Decision section) is the load-bearing citation:
  billing/ports/finops-api/src/authz.rs
-->

## Status

**Proposed - 2026-06-23 (door: one-way).**

## Context

`billing/ports/finops-api` is the boundary library for the Cloud FinOps report surface
(`cloud.finops.report`): it normalizes a request and generates a multi-tenant **cloud-spend report**
— `FINANCIAL_REGULATED_CREDIT`-class cost data (axis/resource costs, anomalies, total spend, gross
margin) for a target tenant. The public entry point is
`generate_cloud_finops_report_from_api(...)`.

Before this ADR the only "authz" was `validate_authorization` (`src/lib.rs`), which merely
cross-checked the **caller-supplied** `CloudFinopsApiAuthorization` DTO
(`decision_id` / `tenant_id` / `principal_id` / `allowed_surfaces`) for internal consistency: there
was no verification of a real principal and no PDP decision. Any caller could self-assert
`allowed_surfaces = ["cloud.finops.report"]` and the request was accepted — letting **any caller
exfiltrate a tenant's cloud-spend report by self-asserting authz**. This is the **AUTH-005
forgeable-caller-supplied-authz class** (the same class PR #768 shipped for
`tenancy/facade/tenant-lifecycle-app`, and the class ADR-0572 / #815 closed for
`iam/ports/policy-cedar-api`). It is a gap-fill CRIT: the library is not yet wired to a facade, so the
fix lands the fail-closed seam BEFORE the surface is exposed.

The repo already carries the correct fail-closed doctrine, which this remediation mirrors:
`iam/ports/policy-cedar-api/src/authz.rs` (ADR-0572: `PrincipalVerifier` + `PublishAuthorizer` PDP
port + `VerifiedPrincipal` + `constant_time_eq` bearer compare), `intelligence/adapters/rest`
(`constant_time_eq` + PDP `decide`), and the true-blast-radius lesson from `secrets/ports/kms-api`
(#817: the resource handed to the PDP is the TARGET resource from a trusted source, never flattened to
the caller's own tenant; cross-tenant must be deniable AT the PDP).

## Decision

Make the Cloud FinOps report surface fail-closed, with the authorization decision modelled as
**ports** owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5
destination so they do not change at cutover; the concrete cloud-iam PDP client + credential store are
**adapters** that live outside this crate).

The new source file `billing/ports/finops-api/src/authz.rs` defines the ports, the reference adapter,
and the constant-time comparison utility that implements this decision.

1. **Verify a real principal.** A new `PrincipalVerifier` port verifies an unforgeable credential into
   a `VerifiedPrincipal`. The reference adapter `ConfiguredBearerPrincipalVerifier` (a **break-glass,
   single-principal adapter only** — NOT multi-tenant production) compares a bearer token in
   **constant time** (`constant_time_eq`, never a naive `==`) against a configured secret and binds the
   principal identity from configuration, NOT from the caller-supplied fields. The production W5 adapter
   is the cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561). Construction **refuses an empty secret or
   bound identity** (boot-refusal doctrine). Unverified → **401** (`PrincipalUnverified`).

2. **Authorize via a PDP port bound to the TARGET tenant.** A new `FinopsReportAuthorizer` port decides
   `ensure_authorized(principal, action = cloud.finops.report, resource = {report_id, scope, tenant})`.
   The `FinopsReportResource` carries a `FinopsReportScope` enum explicitly (`Tenant` or `Platform`) so
   the PDP sees the **true blast radius** of the read. The resource tenant is the report's TARGET tenant
   derived from the validated request body — a trusted source after the verified cross-check — NOT
   echoed from a caller header and NOT flattened to the caller's own tenant. A cross-tenant read is
   deniable at the PDP. A platform-wide aggregate (the reserved `PLATFORM_AGGREGATE_TENANT_ID =
   "ten_platform"` target) is presented as a `Platform` resource requiring platform-admin authority — a
   tenant-finops-admin cannot exfiltrate platform-wide spend by self-asserting the platform tenant (the
   #815 global-scope CRITICAL). Default-deny: any deny **or** refusal/fault → **403** (`PdpDenied`).
   Both Denied and Refused collapse to a single opaque 403 so probing cannot distinguish "policy says
   no" from "PDP unavailable". The adapter contract REQUIRES mapping every fault to `Err(Refused)`,
   enforcing a deadline, and NOT panicking (the release profile is `panic = "abort"`, so `catch_unwind`
   is not relied upon; we do not overclaim panic→403).

3. **Refuse to serve without both ports.** `FinopsReportAuthzProvider` carries **required,
   non-optional** verifier + authorizer ports; there is no `Default` and no constructor that yields a
   provider without a configured authz seam — no default-allow fallback.

4. **Unforgeable verified principal + active cross-check at the boundary API.**
   `generate_cloud_finops_report_from_api` takes a `&VerifiedPrincipal` as its first argument. The type
   has private fields and a `pub(crate)` constructor (a `#[cfg(test)]`-only test constructor exists);
   external crates cannot build it by struct literal or any public API — they must run a real
   `PrincipalVerifier`. The function then ACTIVELY cross-checks the request's self-asserted
   `principal_id` / `tenant_id` against the verified identity and rejects any mismatch (403:
   `VerifiedPrincipalMismatch` / `VerifiedTenantMismatch`) — a verified principal of tenant A may not
   operate as tenant B. This is structural defense-in-depth (it prevents accidental bypass and proves a
   verifier ran), NOT a cryptographic guarantee; the real security comes from the combination of
   verifier + PDP decision + active cross-check. The gate runs BEFORE the idempotency ledger and the
   kernel call: a denied or unverified request never mutates the ledger and never reads spend data.

5. **Demote caller-supplied authorization to a non-authoritative correlation hint.** The
   `CloudFinopsApiAuthorization.allowed_surfaces` self-assertion **no longer grants anything** — the
   PDP decision is authoritative. The legacy `validate_authorization` is renamed
   `validate_authorization_correlation` and retained only for internal-consistency / log-join coherence
   (non-empty `decision_id`; tenant/principal consistency); the forgeable `AuthorizationDenied`
   self-grant path is removed. `decision_id` is a caller-supplied correlation id, NOT a grant.

The authz-coverage gate (`oya-cloud-ci-authz-coverage-app`) discovers HTTP route surfaces
(`.route(` / `.route_service(`); `tests` dirs excluded). `billing/ports/finops-api` is a pure function
library with NO router, so it is not a discovered surface and carries no
`frozen_unauthenticated_surfaces` baseline entry to shrink. This ADR hardens the library seam itself so
that whichever facade later mounts the report cannot reach report generation without a verified
principal and a PDP decision.

## Consequences

- The report surface is fail-closed: unverified → 401, verified-but-cross-tenant → 403, PDP
  deny/refusal → 403, authorized → 201. The self-attestation bypass is closed (a caller who self-asserts
  `allowed_surfaces = ["cloud.finops.report"]` is STILL denied by a deny PDP — proven by RED/GREEN
  tests).
- The authz decision is a port; the cloud-iam PDP client and the bearer/SVID credential store are
  adapters wired by the composition root (deferred to the embedding application; this change ships the
  ports + the in-process reference bearer verifier + the fail-closed boundary fn).
- `generate_cloud_finops_report_from_api`'s signature changes (now takes `&VerifiedPrincipal` +
  `&dyn FinopsReportAuthorizer`). The crate has no downstream consumers yet, so no caller breaks.

## Alternatives considered

- **Inline a concrete PDP in the boundary fn.** Rejected: violates clean architecture (ADR-0131); the
  decision is a port the facade depends on, the PDP is the adapter.
- **Keep caller-supplied `allowed_surfaces` as the grant.** Rejected: that IS the AUTH-005
  vulnerability.
- **Optional provider with a default-allow fallback.** Rejected: a sensitive multi-tenant read must
  refuse to serve without authz (new-surface fail-closed doctrine).
- **Flatten the PDP resource to the caller's tenant.** Rejected: that defeats true-blast-radius
  binding (#817) and re-enables the platform-aggregate escalation (#815).
