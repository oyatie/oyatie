---
id: ADR-0588
title: "Fail-closed verified-principal + PDP authorization for the audit.event.emit boundary (C15 tamper-evidence remediation)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-700]
amended_by: []
amends: []
depends_on: [ADR-0083, ADR-0131]
related: [ADR-0561, ADR-0572, ADR-0581]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W2
---

# ADR-0588: Fail-closed authz for the `audit.event.emit` boundary

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; BLOCKED pending adversarial security
review. Door: two-way — additive ports + a required provider argument behind an already-shaped
clean-architecture seam, reversible by removing the two ports, the provider argument, and the new
public fn without unwinding any SSOT. On approval the founder flips this to Accepted and admits the
born-unpropagated decision into
`ci/facade/baseline-ratchet/gate-baseline.signoff.json` per the established
new-Accepted-ADR door precedent, then propagates it into the masterplan/roadmap faces.)**

## Context

`audit/core/usecase` (`audit-usecase`) is the Platform Audit Chain app boundary: it owns
CloudEvents envelope normalization, request-fingerprint idempotency, the immutable platform
audit-chain append, and the eventing outbox publication for `audit.event.emit`. The audit chain is
the substrate the whole platform relies on for TAMPER-EVIDENCE.

Before this ADR the only "authorization" on the emit path was the in-crate `validate_authorization`,
which cross-checked a CALLER-SUPPLIED `AuditEventEmitAuthorization` DTO
(`{tenant_id, producer_id, decision_id, allowed_surfaces}`) against the envelope for internal
CONSISTENCY only. Authority was self-attested: any caller who could reach the boundary fabricated
`allowed_surfaces = ["audit.event.emit"]` with a matching tenant/producer and the request was
accepted. **Forged authorization fields emitted audit records, defeating the very tamper-evidence
the audit chain exists to provide** (the Wave-2 capability-audit C15 finding, AUTH-005 class). This
mirrors the class PR #768 shipped (an unauthenticated control plane that passed all gates green);
the founder mandate is that it must be IMPOSSIBLE to emit an audit record off self-attested authz.

The proven fail-closed doctrine already lives in `iam/ports/policy-cedar-api/src/authz.rs` (#815 /
ADR-0572) and `iam/facade/identity-workload-rest` (#816 / ADR-0581). This ADR applies the same
doctrine to the `audit.event.emit` boundary, baking in every hard-won lesson those rounds surfaced.

## Decision

1. **Two clean PORTS owned by the boundary crate** (`audit-usecase`), concrete adapters outside it
   (owned-W5 shape), in the new `audit/core/usecase/src/authz.rs`:
   - `PrincipalVerifier::verify_principal(&CallerCredential) -> Result<VerifiedProducerPrincipal,
     PrincipalVerificationError>` — caller authentication. `VerifiedProducerPrincipal` has PRIVATE
     fields and a `pub(crate)` constructor, so it can ONLY be minted by a verifier that proved an
     UNFORGEABLE credential (`constant_time_eq` bearer in the reference
     `ConfiguredBearerPrincipalVerifier`, which REFUSES an empty secret/identity at construction;
     mTLS/SPIFFE peer-SVID in a production adapter, ADR-0561). The envelope/payload tenant + producer
     ids never authorize.
   - `AuditEmitAuthorizer::ensure_authorized(&VerifiedProducerPrincipal, &AuditEmitResource) ->
     Result<(), AuditEmitAuthorizationError>` — the PDP seam (`decide` for
     `action = audit.event.emit`). The default posture is deny; any deny OR fault (timeout, network,
     unavailability) maps to a fail-closed 403. The trait documents the adapter contract: map every
     fault to `Err`, enforce a deadline, MUST NOT panic.

2. **Required, non-optional provider.** The single public emit path,
   `emit_audit_event_authorized(authz, verified, chain, outbox, ledger, request)`, takes the
   `AuditEmitAuthzProvider` as a REQUIRED argument. There is no `Default` and no allow-all provider.
   The prior public `emit_audit_event_from_app` (which authorized off the self-attested DTO) is
   REMOVED from the public API; the raw append is now `pub(crate)
   emit_audit_event_unauthorized_inner`, reachable only AFTER the gate. There is no public code path
   to the chain append that skips verification + PDP.

3. **Active cross-check (the #815-round-1 lesson — the token must be USED, not an unused param).**
   `emit_audit_event_authorized` cross-checks the verified identity against the request's
   self-attested envelope producer + tenant. A mismatch is a substitution attempt →
   `VerifiedPrincipalMismatch` (403). The recorded tenant is therefore always the verified producer's
   tenant; a forged authorization can never change the attributed tenant.

4. **True blast radius / no IDOR (the #817 lesson).** The `AuditEmitResource` handed to the PDP is
   derived from the validated payload's TARGET `{tenant, surface}`, never flattened to the caller's
   own verified tenant. A cross-tenant emit (verified producer of tenant A recording for tenant B) is
   deniable AT THE PDP. An empty payload tenant (a platform-lineage event) is presented as an explicit
   `AuditEmitScope::Platform` resource requiring platform-audit authority — a tenant producer cannot
   forge a platform-level audit record (the #815 global-scope CRITICAL, transposed).

5. **Caller-supplied authorization demoted, not trusted.** The `AuditEventEmitAuthorization` DTO is
   retained for request-fingerprint/idempotency continuity only and is documented as NON-AUTHORITATIVE:
   `decision_id` is a correlation hint, `allowed_surfaces` is never consulted to permit the emit, and
   `tenant_id`/`producer_id` are cross-checked but confer no authority.

6. **No catch_unwind overclaim (the #816 lesson).** The release profile uses `panic = "abort"`, which
   defeats `catch_unwind`; this crate deliberately does NOT wrap the adapter in `catch_unwind` and does
   NOT claim a panic-becomes-403 guarantee. Fail-closure rests on the documented adapter contract
   (every fault → `Err(Refused)` → 403).

## Consequences

- The fix is fail-closed by construction. The `audit-usecase` crate is a pure typed library with no
  HTTP router, so the cloud-ci authz-coverage gate (which keys on `.route(` route introductions) does
  not enumerate it and there is no `frozen_unauthenticated_surfaces` entry to shrink; the remediation
  is at the library boundary fn, exactly as #815's `publish_cedar_policy_from_api` is.
- The HTTP/gRPC edge that will mount `audit.event.emit` (when one lands) consumes
  `AuditEmitAuthzProvider`: it verifies the credential BEFORE body deserialization (route_layer on
  Parts + an explicit `DefaultBodyLimit`) and passes the `VerifiedProducerPrincipal` into
  `emit_audit_event_authorized`, mirroring the cedar REST edge.
- A production deployment supplies the bearer/SVID credential root and a reachable PDP; a boundary
  that cannot prove a credential root and reach a PDP can never emit (boot-refusal doctrine).

## Alternatives considered

- **Keep the self-attested DTO and "validate harder".** Rejected: no amount of internal-consistency
  checking on caller-supplied fields produces authority; the credential must be unforgeable.
- **`decide()` infallible + `catch_unwind`.** Rejected: `catch_unwind` cannot catch `abort`, muddies
  the ADR-0083 Tier-3 panic-free contract, and is the wrong tool. A `Result<(), AuditEmitAuthorizationError>`
  is fail-closed by type; the adapter surfaces its own faults.
- **A public-field verified token (the #815-round-1 mistake).** Rejected: private fields +
  `pub(crate)` constructor + a `#[cfg(test)]` test constructor are mandatory so external crates cannot
  forge the token by struct literal.

## Files

This decision introduces two new source files (born-accounting justification — the verbatim tracked
paths are named here so the total-accounting registry resolves their `justification_ref` to this ADR
and the firewall `unjustified` count does not regress):

- audit/core/usecase/src/authz.rs — the fail-closed authz seam: the `PrincipalVerifier` and
  `AuditEmitAuthorizer` PORTS, the unforgeable `VerifiedProducerPrincipal` token, the
  `AuditEmitResource`/`AuditEmitScope` true-blast-radius types, the `AuditEmitAuthzProvider`, the
  `constant_time_eq` bearer comparator, and the reference `ConfiguredBearerPrincipalVerifier` adapter
  (break-glass only; the W5 adapter is the cloud-iam mTLS/SPIFFE verifier per ADR-0561).
- audit/core/usecase/tests/audit_event_emit_authz.rs — the RED/GREEN seam tests proving forged/absent
  credential → 401, verified cross-tenant → 403 (blast-radius binding), PDP deny/fault → 403, and the
  authorized happy path → ok, each RED case asserting NO chain/outbox/idempotency emission.

The new public `emit_audit_event_authorized` fn, the demoted `AuditEventEmitAuthorization` doc, the
`pub(crate) emit_audit_event_unauthorized_inner` rename, and the new
`PrincipalUnverified`/`VerifiedPrincipalMismatch`/`PdpAuthorizationDenied` error variants are additive
edits inside the existing audit/core/usecase/src/lib.rs boundary crate.
