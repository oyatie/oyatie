---
id: ADR-0573
title: "Fail-closed authz for the Cloud KMS crypto control plane (AUTH-005 / C5 remediation)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-702]
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0090, ADR-0559, ADR-0561, ADR-0566, ADR-0572]
amends: []
related: [ADR-0559, ADR-0561, ADR-0566, ADR-0572]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0573: Fail-closed authz for the Cloud KMS crypto control plane (AUTH-005 / C5 remediation)

## Status

**Proposed - 2026-06-23 (door: one-way).**

## Context

`secrets/ports/kms-api` is the API boundary for the **crown-jewel** Cloud KMS crypto surfaces
`cloud.kms.encrypt` / `cloud.kms.decrypt` (`authorize_cloud_kms_encrypt_from_api` /
`authorize_cloud_kms_decrypt_from_api` in `secrets/ports/kms-api/src/lib.rs`). These are
**mutating, plaintext-revealing, multi-tenant** operations on per-tenant keys.

Before this ADR the only "authz" was `validate_authorization` (`src/lib.rs`), which merely
cross-checked a **caller-supplied** `CloudKmsApiAuthorization` blob
(`{decision_id, tenant_id, principal_id, allowed_surfaces}`) for internal consistency against the
(also caller-supplied) `CloudKmsApiPrincipal`. An attacker who can reach the socket forges that
blob — sets `allowed_surfaces` to contain the requested surface and the tenant/principal to match —
and the request authorizes a KMS decrypt/encrypt with **NO verified PDP decision**. This is the
**AUTH-005 unauthenticated / caller-supplied-authz class** (the same class PR #768 shipped for
`tenancy/facade/tenant-lifecycle-app` and ADR-0572 closed for `iam/ports/policy-cedar-api`), here on
the **most security-critical surface in the repo** — the whole-repo review's **CRITICAL C5**
finding: a forgeable PDP bypass on the crypto control plane.

The repo already carries the correct fail-closed doctrine, which this remediation mirrors:
`intelligence/adapters/rest/src/lib.rs` (`constant_time_eq` bearer compare + a PDP `decide` port),
the cloud-iam PDP caller-authn precedent (`iam/facade/cloud-pdp-app/src/mtls.rs`, ADR-0561 / #38),
and the immediately-preceding Cedar policy publish remediation (ADR-0572). This ADR records the
KMS remediation.

## Decision

Make the Cloud KMS crypto surfaces fail-closed, with the authorization decision modelled as **ports**
owned by the boundary crate (clean architecture per ADR-0131; ports model the owned W5 destination so
they do not change at cutover; the concrete cloud-iam PDP client + credential store are **adapters**
that live outside this crate).

The new source file `secrets/ports/kms-api/src/authz.rs` defines the ports, the reference adapter,
and the constant-time comparison utility that implements this decision.

1. **Verify a real principal (unforgeable credential).** A new `PrincipalVerifier` port in
   `secrets/ports/kms-api/src/authz.rs` verifies an unforgeable credential into a
   `VerifiedKmsPrincipal`. The reference adapter `ConfiguredBearerPrincipalVerifier`
   (a **break-glass, single-principal adapter only** — NOT multi-tenant production) compares a bearer
   token in **constant time** (`constant_time_eq`, never a naive `==`) against a configured secret and
   binds the principal identity from configuration — NOT from the caller headers / authorization blob.
   The production W5 adapter is the cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561). Construction
   **refuses an empty secret** (boot-refusal doctrine). Unauthenticated → **HTTP 401**
   (`CloudKmsApiError::PrincipalUnauthenticated`).

2. **Authorize via a PDP port with a trusted resource binding.** A new `KmsCryptoAuthorizer` port
   decides `decide(principal, action = cloud.kms.{encrypt,decrypt}, resource = {tenant, key_id,
   data_class, purpose, request_id})`. Every authority-bearing field of the resource — `tenant_id`
   and the principal — is bound from the **VERIFIED** principal, NOT from caller input; `key_id` is the
   **target** key from the trusted path binding (already cross-checked equal to the body key id). This
   is the **true blast radius**: presenting the resource with the caller's claimed tenant instead of
   the verified tenant would let tenant A authorize a decrypt of tenant B's key (cross-tenant IDOR) —
   the binding prevents it, and the kernel additionally enforces `key.tenant == request.tenant`
   (`ResourceTenantMismatch`). Default-deny: any deny **or PDP fault (timeout/network/unavailability,
   mapped to `Refused` by the adapter contract)** → **HTTP 403**
   (`CloudKmsApiError::CryptoAuthorizationDenied`), never 500. The cloud-iam Cedar PDP client
   (ADR-0559) is the canonical W5 adapter. In test builds (`panic = unwind`) `catch_unwind` wraps the
   authorizer call and maps panics to `Refused → 403`; in release builds (`panic = "abort"`,
   `Cargo.toml [profile.release]`) `catch_unwind` is a no-op and a panicking adapter aborts the
   process — so the real guarantee is the adapter contract: adapters MUST NOT panic and MUST map every
   fault to `Err(Refused)`.

3. **Refuse to serve without a provider.** The public crate functions
   `authorize_cloud_kms_encrypt_from_api` / `authorize_cloud_kms_decrypt_from_api` require BOTH a
   `&VerifiedKmsPrincipal` (an unforgeable type — private fields, `pub(crate)` constructor; external
   crates must run a real `PrincipalVerifier`) AND a `&KmsCryptoAuthzProvider`. There is no Default
   impl, no default-allow fallback, and no path to the crypto op without passing both. The composition
   root binds the verifier + PDP adapters; a process that cannot prove a credential root cannot
   authenticate a caller.

4. **Verified identity is authoritative; caller blob is demoted.** The caller-supplied
   `CloudKmsApiAuthorization` authority fields (`tenant_id`, `principal_id`, `allowed_surfaces`) are
   **removed**. The remaining caller field is demoted to a non-authoritative
   `CloudKmsApiAuthorizationCorrelation { decision_id }` — a telemetry/audit correlation id that
   confers **no** authorization. The verified principal is cross-checked against the caller-asserted
   `CloudKmsApiPrincipal` and the request body actor/tenant: a mismatch is rejected 403
   (`VerifiedPrincipalMismatch` / `VerifiedTenantMismatch`) before any state mutation. The receipt
   actor (set by the kernel from `body.actor`, cross-checked equal to the verified principal) therefore
   reflects the **VERIFIED** identity, not a caller header.

The legacy header/body consistency checks (`validate_tenant_binding`, `validate_principal_actor`) run
as non-authoritative request-shape validation, not as the authorization boundary.

This surface is not in the `cloud-ci-authz-coverage` gate baseline (the gate's `scan_roots` cover
axum/route-introduction control planes under `billing/cloud/console/iac/iam/intelligence/k8s/libs/oya/tenancy`;
`secrets/ports/kms-api` is a pure boundary crate with no router, so there is no
`frozen_unauthenticated_surfaces` entry to shrink). The fail-closed gate is enforced structurally by
the required `&VerifiedKmsPrincipal` + `&KmsCryptoAuthzProvider` signature and proven by the RED/GREEN
suite in `secrets/ports/kms-api/tests/cloud_kms_api.rs`.

## Consequences

- The Cloud KMS crypto surfaces are fail-closed: unauthenticated → 401, authenticated-but-unauthorized
  → 403, cross-tenant key / verified-identity mismatch → 403, PDP fault → 403 (not 500), authorized →
  ok. The self-attestation / forged-`allowed_surfaces` bypass is closed (the blob no longer authorizes
  anything), proven by RED/GREEN tests.
- The authz decision is a port; the cloud-iam Cedar PDP client and the bearer/SVID credential store are
  adapters wired by the composition root (deferred to the embedding application; this change ships the
  ports + the in-process reference bearer verifier + the fail-closed boundary functions).
- The caller-supplied authorization DTO authority fields are removed (a breaking change to the boundary
  API surface; callers now pass a verified principal + provider). The idempotency ledger keys on the
  verified principal.

## Alternatives considered

- **Inline a concrete PDP in the boundary function.** Rejected: violates clean architecture
  (ADR-0131); the decision is a port the boundary depends on, the PDP is the adapter.
- **Keep the caller-supplied `allowed_surfaces` self-attestation.** Rejected: that IS the AUTH-005 /
  C5 vulnerability.
- **Optional provider with a default-allow fallback.** Rejected: the crown-jewel crypto surface must
  refuse to serve without authz (new-HTTP-surface fail-closed doctrine).
