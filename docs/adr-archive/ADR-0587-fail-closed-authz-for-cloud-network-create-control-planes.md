---
id: ADR-0587
title: "Fail-closed verified-principal + PDP authorization for the Cloud Network LB/VPC/DNS create control planes"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-23
door: two-way
owner: axis-cloud-platform
supersedes: []
superseded_by: [ADR-700]
amends: []
depends_on: [ADR-0083, ADR-0131]
related: [ADR-0510, ADR-0515, ADR-0561, ADR-0572, ADR-0581]
related_specs:
  - /specs/capability-registry.json
milestone: W2
---

# ADR-0587: Fail-closed authz for the Cloud Network LB/VPC/DNS create control planes

## Status

**Proposed - 2026-06-23 (authored for founder sign-off; BLOCKED pending adversarial security
review. Door: two-way — additive ports + a required boundary parameter behind an already-shaped
clean-architecture seam, reversible by removing the `authz` modules and the provider argument
without unwinding any SSOT. On approval the founder flips this to Accepted and admits the
born-unpropagated decision into
`ci/facade/baseline-ratchet/gate-baseline.signoff.json` per the established
new-Accepted-ADR door precedent, then propagates it into the masterplan/roadmap faces.)**

## Context

The Cloud Network boundary crates own the tenant-facing create surfaces for three resource types:

- `network/ports/lb` (`network-lb`) — `cloud.network.lb.create`
  (`create_cloud_network_load_balancer_from_api`)
- `network/ports/vpc` (`network-vpc`) — `cloud.network.vpc.create`
  (`create_cloud_network_vpc_from_api`)
- `network/ports/dns` (`network-dns`) — `cloud.network.dns.zone.create`
  (`create_cloud_network_dns_zone_from_api`)

Each is a MUTATING multi-tenant control plane (it inserts a tenant-scoped resource into the shared
`CloudNetworkCatalog`). Before this change, the only "authorization" was a request-supplied
`CloudNetwork{Lb,Vpc,Dns}ApiAuthorization` blob carrying a self-attested `{decision_id, tenant_id,
principal_id, allowed_surfaces}`. The boundary merely cross-checked that blob against the equally
self-attested `principal` for internal consistency:

```text
authorization.tenant_id    == principal.tenant_id
authorization.principal_id == principal.principal_id
authorization.allowed_surfaces.contains(surface)
```

An attacker who can reach the call constructs all three fields consistently —
`allowed_surfaces = ["cloud.network.lb.create"]`, matching `tenant_id`/`principal_id` — and the
request is accepted. There is no verified credential and no server-side policy decision: the request
authorizes itself. This is the AUTH-005 / lethal-trifecta class (findings C9 / C10 / C11) the
founder mandate requires to be impossible to ship: "a recorded gap is not a license to ship the
antipattern."

The proven fail-closed doctrine already lives in `iam/ports/policy-cedar-api/src/authz.rs` (#815,
ADR-0572) and the workload-principal lifecycle remediation (`iam/facade/identity-workload-rest`,
#816, ADR-0581). This ADR applies the same doctrine to the three Cloud Network create control
planes, baking in the hard-won lessons of those rounds (unforgeable token type; the boundary must
USE the verified principal; the PDP must see the TARGET tenant, not a flattened caller tenant;
fail-closed on PDP fault; no `panic -> 403` overclaim).

## Decision

For each of the three boundary crates, add an in-crate `authz` module with two clean PORTS owned by
the boundary crate; the concrete adapters (cloud-iam PDP client, mTLS/SPIFFE or bearer credential
store) live OUTSIDE the crate (owned-W5 shape, so the port shapes do not change at cutover):

1. **`PrincipalVerifier::verify_principal(&CallerCredential) -> Result<VerifiedPrincipal, _>`** —
   caller authentication. `VerifiedPrincipal` has PRIVATE fields, NO public constructor, and a
   `pub(crate)`-only `new` plus a `#[cfg(test)]` test constructor, so it can ONLY be minted by a
   verifier that proved an UNFORGEABLE credential (`constant_time_eq` bearer in the reference
   `ConfiguredBearerPrincipalVerifier`; mTLS/SPIFFE peer-SVID in the production W5 adapter). The
   request-supplied `principal`/credential headers are NEVER the source of truth.

2. **`{Lb,Vpc,DnsZone}CreateAuthorizer::ensure_authorized(&VerifiedPrincipal, &{...}CreateResource)
   -> Result<(), _>`** — the server-side PDP decision (`decide(principal, action, resource)`). The
   `{...}CreateResource` carries the TARGET `{tenant_id, resource_id}` derived from the trusted
   request body (already cross-checked equal to the verified principal's tenant), so a cross-tenant
   create is deniable AT THE PDP (no IDOR / no flatten-to-caller-tenant). Default-deny; the
   documented adapter contract requires every fault (error/timeout/unavailability) to map to a deny
   `Result` and to NOT panic (the release profile is `panic = "abort"`, so `catch_unwind` is not
   relied upon — there is no `panic -> 403` claim).

3. **The boundary REFUSES to serve without the seam.** `create_cloud_network_*_from_api` now takes a
   REQUIRED `&CloudNetwork{Lb,Vpc,Dns}AuthzProvider` (a verifier port + an authorizer port). There
   is no `Default` and no allow-all fallback; the composition root cannot build the provider without
   a credential root (`ConfiguredBearerPrincipalVerifier::new` refuses an empty secret/identity).

4. **The flow is fail-closed and ordered.** validate request shape → verify credential (`401`
   `CallerUnauthenticated` on missing/invalid) → cross-check the request principal/tenant against the
   VERIFIED identity (`403` `VerifiedPrincipalMismatch` / `VerifiedTenantMismatch`) → PDP decide
   against the TARGET resource (`403` `AuthorizationDenied` on deny OR fault) → only then mutate. No
   code path reaches the catalog insert without passing the gate.

5. **The request-supplied authorization blob is DELETED.** The
   `CloudNetwork{Lb,Vpc,Dns}ApiAuthorization` DTO and its `allowed_surfaces` self-grant are removed
   from the request and from the idempotency fingerprint. Authorization is now derived solely from
   the verified principal and the PDP decision; audit/idempotency reflect the verified principal, not
   caller headers.

## Consequences

- **Positive:** the three create surfaces are default-deny; a forged authorization blob no longer
  authorizes anything (RED tests: forged/absent credential → 401; verified cross-tenant → 403 with an
  otherwise-allowing PDP, proving blast-radius binding; PDP-deny → 403; PDP-fault → 403; happy path →
  ok). The ports are owned-W5-shaped and do not change at the cloud-iam PDP cutover.
- **Negative / cost:** callers must now present a real credential and the composition root must wire
  the cloud-iam PDP adapter. The reference `ConfiguredBearerPrincipalVerifier` is BREAK-GLASS ONLY
  (single static principal/tenant); multi-tenant production requires the mTLS/SPIFFE peer-SVID
  adapter (ADR-0561). That adapter and the composition-root wiring are follow-up work tracked behind
  this ADR; the boundary is fail-closed regardless (it refuses to build without a provider).
- **Reversibility (two-way door):** remove the `authz` modules + the provider parameter to revert; no
  SSOT is unwound.

## Born-accounting justification (verbatim tracked paths)

This ADR is the born-accounting justification for the new boundary-authz source files it
introduces. Each path below is listed verbatim so the accounting-registry producer
(`git ls-files × OWNERS × ADR-front-matter`) binds `justification_ref: ADR-0587` to it and the
firewall `unjustified` count does not regress:

- `network/ports/lb/src/authz.rs` — the `cloud.network.lb.create` fail-closed authz seam (C9).
- `network/ports/vpc/src/authz.rs` — the `cloud.network.vpc.create` fail-closed authz seam (C10).
- `network/ports/dns/src/authz.rs` — the `cloud.network.dns.zone.create` fail-closed authz seam
  (C11).

The owning `network/ports/lb/src/lib.rs`, `network/ports/vpc/src/lib.rs`, and
`network/ports/dns/src/lib.rs` boundaries are amended in the same change to require the injected
authz provider; their pre-existing registry rows are unchanged in kind.

## Alternatives considered

- **Keep the request-supplied blob but sign it.** Rejected: a signed self-grant is still a client
  asserting its own authorization; the server must make the decision (defense-in-depth and the
  founder authz doctrine require a server-side PDP, not client-presented decisions).
- **A shared `network-authz` crate for all three.** Deferred: the three ports are isomorphic but the
  resource/action types differ per surface; a premature shared crate would couple them. The in-crate
  modules keep each boundary self-contained; a later extraction is a clean refactor if a fourth
  surface appears.
