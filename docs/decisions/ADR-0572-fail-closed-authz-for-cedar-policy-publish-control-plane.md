---
id: ADR-0572
title: "Fail-closed authz for the Cedar policy publish control plane (AUTH-005 remediation)"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0083, ADR-0131, ADR-0090, ADR-0559, ADR-0561, ADR-0566]
amends: []
related: [ADR-0559, ADR-0561, ADR-0564, ADR-0566]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0572: Fail-closed authz for the Cedar policy publish control plane (AUTH-005 remediation)

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; door: one-way).**

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

1. **Verify a real principal.** A new `PrincipalVerifier` port verifies an unforgeable credential
   into a `VerifiedPrincipal`. The reference adapter `ConfiguredBearerPrincipalVerifier` compares a
   bearer token in **constant time** (`constant_time_eq`, never a naive `==`) against a configured
   secret and binds the principal identity from configuration — NOT from the caller headers. A
   cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561) is a drop-in alternate adapter. Construction
   **refuses an empty secret** (a process that cannot prove a credential root must never
   authenticate a caller — the cloud-pdp boot-refusal doctrine). Unauthenticated → **HTTP 401**.

2. **Authorize via a PDP port.** A new `PublishAuthorizer` port decides
   `decide(principal, action = cedar.policy.publish, resource = {policy_id, tenant})`. The tenant
   axis is asserted by the decision — a verified principal alone never grants the tenant; the
   resource tenant is the scope tenant for tenant-scoped policies (so cross-tenant publish — a
   principal of tenant A publishing tenant B's policy — is denied). Default-deny: any deny/refusal →
   **HTTP 403**. The cloud-iam Cedar PDP client (ADR-0559) is the canonical W5 adapter.

3. **Refuse to serve without a provider.** `CedarPolicyRestState` carries a **required, non-optional**
   `CedarPolicyAuthzProvider`; there is no constructor that yields router state without it, so the
   binary/router can never mount this control plane with a default-allow fallback.

4. **Cross-tenant guard.** The verified tenant is authoritative: a request whose operator
   (`x-tenant-id`) or self-attested principal tenant differs from the verified tenant is denied
   (403) before any state mutation.

The legacy `validate_authorization` header-consistency checks are retained as defense-in-depth after
the verified-principal + PDP gate, not as the authorization boundary.

The AUTH-005 baseline entry for this surface is **removed** from
`cloud/cloud-ci/gates/oya-cloud-ci-authz-coverage-app/authz-coverage-policy.json`
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
