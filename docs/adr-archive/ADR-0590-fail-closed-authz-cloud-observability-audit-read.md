---
id: ADR-0590
title: "Fail-closed verified-principal + server-side PDP authz for the Cloud Observability audit-read surface (C18 / AUTH-005 remediation)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0510, ADR-0559, ADR-0561, ADR-0566, ADR-0572]
amends: []
related: [ADR-0566, ADR-0572, ADR-0559, ADR-0561]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0590: Fail-closed verified-principal + server-side PDP authz for the Cloud Observability audit-read surface (C18 / AUTH-005 remediation)

## Status

**Proposed - 2026-06-23 (door: one-way).**

## Context

`observability/core/api` owns the boundary
[`read_cloud_observability_audit_from_api`] for the
`cloud.observability.audit.read` surface. The surface serves immutable audit
records: control-plane mutation history under the `control_plane_mutations`
scope, and — under the broader `all_tenant_audit` scope — data-plane-security,
KMS-use, and billing audit records.

Before this ADR the only "authorization" was a request DTO,
`CloudObservabilityApiAuthorization`, carrying a caller-supplied
`allowed_surfaces: Vec<String>`. The boundary merely checked that this
self-attested list *contained* the audit-read surface
(`src/lib.rs:504`, the C18 finding). A caller who can reach the API simply sets
`allowed_surfaces = ["cloud.observability.audit.read"]` and is "authorized" —
**self-granting authorization evidence**: the caller authors the very decision
that is supposed to authorize them. This is the AUTH-005 class (the same class PR
#768 shipped for `tenancy/facade/tenant-lifecycle-app`, remediated by ADR-0572
for `iam/ports/policy-cedar-api`).

Worse, the SAME flat surface check authorized BOTH scopes. A principal entitled
to read control-plane mutations auto-acquired the much broader
`all_tenant_audit` corpus (data-plane security, KMS use, billing) — a
**coarse-scope** privilege escalation with no distinct, more-privileged
authority for the broader scope.

## Decision

Close both gaps by applying the proven fail-closed doctrine from ADR-0572
(`iam/ports/policy-cedar-api`, #815) and the secrets KMS-API boundary (#817),
adapted to this pure-library boundary (this crate has no HTTP router; the
boundary is a function the facade calls):

1. **Verified, unforgeable principal.** A new `authz` module owns a
   `PrincipalVerifier` PORT producing a `VerifiedPrincipal` whose fields are
   private and whose constructor is `pub(crate)` — external crates can only
   obtain one by running a real verifier. The reference adapter
   (`ConfiguredBearerPrincipalVerifier`) compares a bearer token with
   `constant_time_eq` (never `==`) against a configured secret, refusing
   construction on an empty secret/identity. The caller-supplied
   principal/authorization DTO fields NEVER establish identity; they are
   cross-checked against the verified principal and rejected on mismatch.

2. **Server-side PDP decision as a clean port.** A new `AuditReadAuthorizer`
   PORT (`ensure_authorized(principal, resource)`) is the server-side decision.
   The concrete cloud-iam Cedar PDP client and credential store are ADAPTERS
   OUTSIDE this crate (the owned-W5 shape, ADR-0131/ADR-0561). The port contract
   is documented default-deny: adapters MUST map every fault (error, timeout,
   unavailability) to `Err(Refused)` and MUST NOT panic — `catch_unwind` is a
   test-only backstop defeated by release `panic = "abort"`, so it is not relied
   on for production fault isolation.

3. **True blast-radius / no IDOR.** The resource handed to the PDP carries the
   TARGET tenant taken from the VERIFIED principal (not the caller DTO), the
   region, the scope-derived action, and a request hash. The boundary binds the
   request body tenant to the verified tenant and forces the served kernel read
   to the verified tenant, so a cross-tenant read is deniable at the PDP and the
   served data is scoped to the authorized tenant.

4. **Coarse-scope split.** The scope is mapped to a DISTINCT `AuditReadAction`:
   `cloud.observability.audit.read.control_plane` vs
   `cloud.observability.audit.read.all_tenant`. The broader scope requires its
   own grant; a control-plane grant no longer auto-confers all-tenant audit.

5. **No default-allow.** The boundary takes the verified principal and the
   authorizer by reference and ALWAYS calls `ensure_authorized` before any
   catalog read. The `allowed_surfaces` field is REMOVED from the DTO; the
   residual DTO fields are demoted to a non-authoritative `correlation_id`.

## Consequences

- The boundary signature changes (breaking) to require a `&VerifiedPrincipal`
  and `&dyn AuditReadAuthorizer`. The crate is a leaf (no reverse dependencies),
  so the change is self-contained; the facade that mounts this surface wires the
  cloud-iam verifier + PDP adapters.
- RED/GREEN tests prove the seam fails if removed: absent/forged credential does
  not verify (401-class); verified cross-tenant is forbidden even with an
  allow-all authorizer (blast-radius binding); PDP deny and PDP refuse both map
  to 403 (fail-closed); a control-plane grant cannot read all-tenant audit
  (coarse-scope); the happy path succeeds and is scoped to the verified tenant.
- Audit records and response metadata reflect the VERIFIED principal/tenant, not
  caller headers.

## New file born-accounting

This ADR is the justification reference for the new source file:

- `observability/core/api/src/authz.rs` — fail-closed authz seam
  (verified-principal + PDP ports + bearer adapter) for the
  `cloud.observability.audit.read` surface (ADR-0590).

## Alternatives considered

- **Keep header-trust, add a signature over `allowed_surfaces`.** Rejected: a
  caller-minted, caller-signed grant is still self-granting; the authority must
  be a server-side decision against a verified identity.
- **Single flat action for both scopes.** Rejected: that is the coarse-scope
  defect; the broader corpus must require strictly more authority.
