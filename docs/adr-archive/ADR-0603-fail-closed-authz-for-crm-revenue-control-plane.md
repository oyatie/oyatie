---
id: ADR-0603
title: "Fail-closed authz for the CRM revenue control plane (AUTH-005 remediation)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-24
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-702]
amended_by: []
depends_on: [ADR-0083, ADR-0105, ADR-0131, ADR-0561]
amends: []
related: [ADR-0559, ADR-0561, ADR-0566, ADR-0572, ADR-0581]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0603: Fail-closed authz for the CRM revenue control plane (AUTH-005 remediation)

## Status

**Proposed - 2026-06-24 (door: two-way — the seam is an internal port shape that
can be revised or replaced before the CRM edge binds a real listener; nothing
here is an irreversible external commitment).**

## Context

`oya/crm/crates/oya-crm-revenue-app` exposes mutating multi-tenant CRM control
planes (`adapter::http`, `adapter::grpc`, `adapter::asyncapi`) over the
capabilities account-master, opportunity, quote, campaign, and service-case.

Before this decision, the request DTOs carried a **caller-supplied** identity
that was trusted as authorization:

- `HttpRequest { tenant_id, principal_id, .. }` (`#[derive(Deserialize)]`)
- `GrpcRequest { tenant_id, .. }` (`#[derive(Deserialize)]`)
- `AsyncApiMessage { tenant_id, .. }` (`#[derive(Deserialize)]`)
- `crm-v1.proto` command messages: `string tenant_id`, `string principal_id`

`usecase::UsecaseContext { actor: UsecaseActor { tenant_id, principal_id, .. } }`
was populated from those body fields, and `PolicyPort::authorize(&envelope)`
then authorized against the caller-supplied actor. An attacker who can reach the
socket sets `tenant_id` to any victim tenant and mutates that tenant's CRM
records — **forge identity → cross-tenant CRM mutation**. This is the AUTH-005
class (the fleet-wide caller-supplied-authz antipattern catalogued in the
2026-06-23 whole-repo review; same class as PR #768/#815/#816/#824/#829).

The crate is currently **dead-until-edge**: `HttpHandler::handle` /
`GrpcHandler::handle` / `AsyncApiHandler::handle` return `ContractStub` and no
socket is bound. The defect is therefore structural rather than live-exploitable
today, but the DTO shape and the authorization-from-body wiring would ship the
vulnerability the moment the edge is wired.

## Decision

Install the established unforgeable-authz seam (mirroring ADR-0572 / #815 and the
`intelligence/adapters/rest` doctrine) in a new `src/authz.rs` owned by this
crate:

1. **Unforgeable verified identity.** A `VerifiedPrincipal { principal_id,
   tenant_id }` with **private fields**, a `pub(crate)` constructor, and a
   `cfg(test)` constructor only. External crates cannot struct-literal one; they
   must run a `PrincipalVerifier` port. `CallerCredential` carries **only** the
   unforgeable transport credential (the `Authorization` header) — it holds no
   caller-asserted `tenant_id` / `principal_id`, so a body claim can never be
   mistaken for an authz input. The request-DTO `tenant_id` / `principal_id`
   remain on the wire types but are **structurally never read** by the gate or as
   the resource tenant.

2. **Verified-from-transport, not body.** The `PrincipalVerifier` port derives
   the principal from a credential the caller cannot forge (a bearer token
   compared in constant time by `ConfiguredBearerPrincipalVerifier`, or the W5
   cloud-iam mTLS/SPIFFE peer-SVID adapter per ADR-0561). The body is never the
   source of truth.

3. **PDP authorization with true blast-radius.** A `CrmAuthorizer` Cedar-PDP
   port decides `decide(principal, action = crm.<capability>.mutate, resource)`
   where `CrmResource.target_tenant_id` is bound from the **verified principal**
   (a trusted source), never from the caller body. The gate takes no request
   body at all, so there is no body tenant that could ever bind the resource;
   `authorize_crm_command` returns an `AuthorizedCrmContext` carrying only the
   verified principal + action, and the adapters bind the resource scope from
   `AuthorizedCrmContext::tenant_id()` (the verified tenant) — the body
   `tenant_id` is structurally ignored, never honored, never denied-on-mismatch
   because it is never an input. Default-deny; any PDP fault is treated as deny
   (fail-closed → HTTP 403).

4. **Authn-before-body and refuse-to-serve.** `HttpHandler::handle` /
   `GrpcHandler::handle` / `AsyncApiHandler::handle` now require a
   `&CrmAuthzProvider` plus a transport-supplied `CallerCredential` and run the
   gate FIRST (via `resolve_scope`). The capability comes from server-side
   route/method metadata, not the body. Verification failure → 401, authorization
   failure → 403, mapped to **distinct** `ServiceError` kinds
   (`Unauthenticated` / `Forbidden`) so the edge derives the HTTP status
   structurally from the kind, not by matching a message string.

## Tracked surfaces

This decision introduces and owns the following new tracked paths (cited here so
the accounting registry traces them to this ADR):

- `oya/crm/crates/oya-crm-revenue-app/src/authz.rs` — the unforgeable-authz seam
  (`VerifiedPrincipal`, `CallerCredential`, `PrincipalVerifier`, `CrmAuthorizer`,
  `AuthorizedCrmContext`, `authorize_crm_command`).
- `oya/crm/crates/oya-crm-revenue-app/OWNERS` — ownership registration (ADR-0555)
  for the crate, naming `axis-cloud-platform` as the owning team.
- `evidence/multispectrum/waveA-crm-resource-model-boundaries-20260625-1782426015.json`
  — Wave A CRM evidence that capability descriptor resource-model boundaries stay
  aligned with the CRM bounded contexts named by this decision.

## Edge obligation (deferred to when the listener binds)

Because no socket is bound yet, this ADR installs the seam and records the
obligation the edge MUST satisfy when it binds a real listener:

- Extract the bearer/SVID credential in transport middleware
  (`route_layer` / `FromRequestParts`, gRPC peer-cert extension) **before** body
  deserialization.
- Install `DefaultBodyLimit`.
- The binary refuses to boot without a configured `PrincipalVerifier` +
  `CrmAuthorizer` (no default-allow fallback), mirroring the cloud-pdp
  boot-refusal doctrine.
- The `asyncapi` consumer path runs the same gate (the command-bearing inbound
  channels; subscribe-only projection channels that mutate nothing are exempt).
- **The `usecase::ServiceInteractor` actor MUST be derived from the verified
  principal, never from the wire.** Today `usecase::UsecaseContext { actor }`
  derives `Deserialize` and `ServiceInteractor::handle` authorizes (and stamps
  the event/receipt tenant) from that caller-built actor — the un-gated residual
  of the AUTH-005 class. The edge that wires an adapter to `submit_command`
  MUST enter the interactor with an actor whose `tenant_id`/`principal_id` come
  from the `AuthorizedCrmContext` returned by `authorize_crm_command` (i.e. the
  verified principal, via `AuthorizedCrmContext::tenant_id()` /
  `::principal_id()`), and the interactor's identity fields SHOULD drop
  `Deserialize` so a caller can never supply them. Until that refactor lands, the
  adapters are the only gated entry and MUST NOT be wired to bypass the gate into
  the interactor. This residual is **not currently reachable** — no live caller
  binds an adapter to the interactor and no socket is bound — so it stays
  recorded as a deferred edge obligation, not a live defect.

## Consequences

- The request-DTO `tenant_id` / `principal_id` fields remain on the wire types
  but are non-authoritative and **structurally never read** — neither by the gate
  nor as the resource tenant. `CallerCredential` no longer carries any
  caller-asserted claim, so the gate performs no body-vs-verified cross-check; a
  forged body tenant is simply ignored. The DTO fields MAY be removed entirely
  once the proto/DTO contract is revised; that is a follow-on contract change.
- The break-glass `ConfiguredBearerPrincipalVerifier` binds a single static
  identity to one shared secret — suitable only for a single-principal
  break-glass token or tests. Multi-tenant production uses the cloud-iam SVID
  verifier (ADR-0561).
- Tests cover the RED paths (no credential → 401, bad bearer → 401, PDP deny →
  403, PDP fault → 403), the cross-tenant invariant (a forged body
  `tenant_id = victim` with a valid bearer for `alpha` resolves the resource
  tenant to `alpha`, never `victim` —
  `forged_body_tenant_never_becomes_the_resource_tenant`), the 401-vs-403
  distinct-kind mapping, the redacting `CallerCredential` `Debug`, and the GREEN
  path (verified + PDP grant → reaches the scaffolded business handler).
