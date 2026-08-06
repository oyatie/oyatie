---
id: ADR-0581
title: "Fail-closed verified-caller + PDP authorization for the workload-principal lifecycle control plane (:suspend/:retire)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: [ADR-0700]
amends: []
depends_on: [ADR-0083, ADR-0105, ADR-0131]
related: [ADR-0510, ADR-0515]
related_specs:
  - /specs/capability-registry.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0581: Fail-closed authz for the workload-principal lifecycle control plane

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; BLOCKED pending adversarial security
review. Door: two-way — additive ports + required constructor params behind an already-shaped
clean-architecture seam, reversible by removing the two ports and the adapter without unwinding any
SSOT. On approval the founder flips this to Accepted and admits the born-unpropagated decision into
`ci/facade/baseline-ratchet/gate-baseline.signoff.json` per the established
new-Accepted-ADR door precedent, then propagates it into the masterplan/roadmap faces.)**

## Context

`iam/facade/identity-workload-rest` (`iam-identity-workload-rest`) is the workload-identity REST PEP
(ADR-0105 Layer 5). Among its routes it mounted two MUTATING control-plane custom methods:

- `POST /principals/{id}:suspend` — revoke a principal (denylist + state transition)
- `POST /principals/{id}:retire`  — terminally retire a principal

These two routes performed the mutation through the app-layer use-cases `suspend`/`retire`, which
take only a `WorkloadId` plus the repository/denylist — **no caller identity, no authorization
decision**. The handler (`principal_lifecycle_handler` → `lifecycle_transition`) derived no caller
principal. Result: any caller who could reach the socket could suspend or retire ANY principal,
cross-tenant, with no credential. This is the AUTH-005 class (PR #768 shipped an unauthenticated
mutating control plane that passed all gates green); the founder mandate is that it must be
IMPOSSIBLE to ship an unauthenticated mutating control plane.

The proven fail-closed doctrine already lives in `intelligence/adapters/rest` (a PDP `gate.decide`
with separate `principal_tenant`/`resource_tenant` axes + `constant_time_eq` bearer) and in the
in-flight cedar-publish remediation (task #124). This ADR applies the same doctrine to the
workload-principal lifecycle control plane, baking in the four failure classes a hostile review of
the cedar-publish remediation surfaced.

## Decision

1. **Two clean PORTS owned by the boundary crate** (`iam-identity-workload-rest`), concrete adapters
   outside it (owned-W5 shape):
   - `CallerVerifier::verify_principal(&HeaderMap) -> Option<VerifiedCaller>` — caller authentication.
     A `VerifiedCaller` has private fields and no public constructor, so it can ONLY be minted by a
     verifier that proved an UNFORGEABLE credential (`constant_time_eq` bearer in the reference
     `BearerCallerVerifier`; mTLS/SPIFFE in a production adapter). Caller-supplied `x-principal-*` /
     `x-authorization-*` headers never authorize.
   - `LifecycleAuthorizer::decide(&LifecycleAuthzRequest) -> Result<bool, AuthzFault>` — the PDP seam.
     `LifecycleAuthzRequest` carries `caller_tenant` (verified) AND `target_tenant` (the TARGET
     principal's real tenant) on separate axes, plus the `LifecycleAction` and target workload id.

2. **Required, non-optional constructor params.** `WorkloadAuthzState::new`/`with_clock` now require
   both ports. There is no `Default` and no allow-all fallback; the router cannot be built without a
   real authz seam. The production binary's `Config` gains a REQUIRED `OYA_IDENTITY_LIFECYCLE_BEARER`
   (+ caller tenant), so `Config::from_env` refuses to load — and the binary refuses to start —
   without a verified-caller credential.

3. **Guard the in-crate choke point, not just the HTTP edge.** The verify → load-target →
   PDP-decide precondition is enforced in `lifecycle_transition`, the single production caller of the
   app-layer `suspend`/`retire` (the gRPC surface does not expose lifecycle). Any in-crate caller of
   a lifecycle mutation passes the same gate.

4. **True blast radius / no IDOR.** The handler acquires the repository lock, loads the target
   principal, derives `target_tenant` from the loaded store record — never from the caller's tenant
   or a header — then DROPS the lock before calling the PDP. On an allow decision the lock is
   re-acquired and the mutation runs. A cross-tenant suspend/retire (caller in tenant A acting on
   tenant B's principal) is therefore deniable by the tenant-scoped PDP. The load→drop→decide→
   re-acquire pattern means a slow or hung PDP adapter stalls only the in-flight request, not the
   service mutex (availability fix, round-2 hardening).

5. **Authn before body + PDP-fault → deny.** The handler extracts `HeaderMap` (a `FromRequestParts`
   extractor, evaluated before any body — no body extractor is present, so no body bytes are read).
   A PDP `Err` maps to a fail-closed `403` with audit detail `"lifecycle-pdp-fault"`; `Ok(false)`
   (explicit policy deny) maps to `403` with distinct detail `"lifecycle-forbidden"`. Both never
   `500`, never allow. No verified caller is `401`. The two fault classes emit distinct audit details
   so a PDP outage is distinguishable from a real policy block during incident response
   (round-2 hardening).

6. **Audit invariant preserved with caller attribution.** Exactly one immutable `AuditRecord` is
   emitted per lifecycle authorize decision (deny-on-unverified, deny-on-policy, deny-on-pdp-fault,
   allow-before-mutation). Every deny/allow record carries `with_caller(caller_id, caller_tenant)`
   from the verified `VerifiedCaller` so incident response can answer WHO authorized a retire/suspend
   (round-2 hardening). The `LifecycleAuthorizer` adapter contract is documented on the trait:
   adapters MUST map every fault to `Err`, MUST enforce their own deadline, MUST NOT panic.

## Consequences

- The fix is fail-closed by construction and gate-green: the cloud-ci authz-coverage gate reports no
  NEW unauthenticated control-plane surface.
- **Baseline note (honest envelope).** The authz-coverage gate keys a surface by its WHOLE
  `build_router` route-signature, independent of per-handler authz. The frozen baseline entry for
  `identity-workload-rest#build_router` also covers `POST /tokens/validate` — a token-in-body PEP
  endpoint whose credential is the workload's own JWT (validated inside), not a bearer-authenticated
  admin caller, exactly like the sibling `/authorize*` endpoints. Because the gate cannot partially
  shrink a whole-scope key, and bolting a second (admin-bearer) auth scheme onto a workload-JWT PEP
  endpoint is out of scope for this AUTH-005 lifecycle remediation and would break that endpoint's
  documented contract, the baseline key is intentionally LEFT IN PLACE. The lifecycle routes now
  carry the real guard; the residual baseline entry is solely the pre-existing `/tokens/validate`
  PEP, tracked separately. Splitting the lifecycle routes into their own router scope was rejected
  because it would create a NEW un-baselined `[/authorize*; /tokens/validate]` key that the gate
  would block (or require a founder-signed `--allow-new` grandfather), trading a real security fix
  for baseline churn.
- A production deployment must provide `OYA_IDENTITY_LIFECYCLE_BEARER` (+
  `OYA_IDENTITY_LIFECYCLE_CALLER_TENANT`); the binary fails fast otherwise.
- The reference `TenantScopedLifecycleAuthorizer` (composition root) enforces same-tenant isolation;
  a richer Cedar-policy-backed cloud-iam PDP swaps in behind the `LifecycleAuthorizer` port without
  touching the REST surface.

## Accepted residuals (verified-caller-only; not security gaps)

### Existence oracle (LOW)

After a successful authn step (step 1 in `lifecycle_transition`) the handler returns `404` for an
unknown target principal and `403` for a known-but-unauthorized one. A lifecycle-bearer holder can
therefore distinguish "this workload id does not exist" from "this workload id exists but you are
denied". This is an **accepted residual**: the information is disclosed only to a verified, bearer-
authenticated caller, never to an unauthenticated one (unauthenticated callers get `401` before any
store work). The blast radius is confined to bearer holders, who are already trusted to know the
workload ids they operate on. If this residual is ever re-evaluated (e.g., a future control-plane
audit finds the distinction actionable to an insider), the fix is to return `403` uniformly after
authn (collapsing 404 and 403), which is a one-line change in `lifecycle_transition`.

### Body-limit (LOW)

The lifecycle routes previously carried `DefaultBodyLimit::max(16 KiB)` via `route_layer`. The
handler `principal_lifecycle_handler` has no body extractor (`Json`, `Bytes`, etc.) — it reads only
`HeaderMap` + `Path` (both `FromRequestParts`). Without a body extractor axum never reads the
request body; the `DefaultBodyLimit` middleware would therefore never trigger a `413`. To avoid
overclaiming ("body limit enforced" when it is not), the `DefaultBodyLimit` route layer and the
`LIFECYCLE_MAX_BODY_BYTES` const have been removed. Body-size limiting for the lifecycle surface is
deferred to the global axum `DefaultBodyLimit` layer applied at the service entry point (the binary's
`build_service` in `identity-service`), which covers all routes including the lifecycle ones.

## Alternatives considered

- **Router-level `.layer()` auth.** Rejected: it would force admin-bearer authn onto `/authorize*`
  and `/tokens/validate`, which are workload-JWT-in-body PEP endpoints, breaking their contract.
- **`decide()` infallible + `catch_unwind`.** Rejected: `catch_unwind` cannot catch `abort`, muddies
  the ADR-0083 Tier-3 panic-free contract, and is the wrong tool for a pure-Rust unsafe-forbidden
  trait. A `Result<bool, AuthzFault>` is fail-closed by type; the adapter surfaces its own faults.
- **Pushing a `VerifiedPrincipal` token into the app-crate `suspend`/`retire` signatures.** Deferred
  as defense-in-depth: there is exactly one production caller (this crate's `lifecycle_transition`),
  the boundary crate owns authz, and gRPC does not expose lifecycle, so the choke point is real.

## Files

This decision introduces one new source file (born-accounting justification — the verbatim tracked
path is named here so the total-accounting registry resolves its `justification_ref` to this ADR and
the firewall `unjustified` count does not regress):

- iam/facade/identity-service/src/lifecycle_authz.rs — the concrete `TenantScopedLifecycleAuthorizer`
  adapter (the W5-shaped reference implementation of the `LifecycleAuthorizer` port), placed in the
  composition root per ADR-0131 ports-and-adapters layering.

The two ports (`CallerVerifier`, `LifecycleAuthorizer`), the `VerifiedCaller` token, and the
`lifecycle_transition` guard are additive edits inside the existing
iam/facade/identity-workload-rest/src/lib.rs boundary crate; the `unauthorized` error envelope is an
additive edit inside iam/ports/identity-workload-api/src/lib.rs.
