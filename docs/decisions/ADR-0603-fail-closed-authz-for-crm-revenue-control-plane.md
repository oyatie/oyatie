---
id: ADR-0603
title: "Fail-closed authz for the CRM revenue control plane (AUTH-005 remediation)"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-24
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
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
   must run a `PrincipalVerifier` port. The caller-supplied `tenant_id` /
   `principal_id` are demoted to **non-authoritative cross-check data** — they
   grant nothing and never select the resource tenant.

2. **Verified-from-transport, not body.** The `PrincipalVerifier` port derives
   the principal from a credential the caller cannot forge (a bearer token
   compared in constant time by `ConfiguredBearerPrincipalVerifier`, or the W5
   cloud-iam mTLS/SPIFFE peer-SVID adapter per ADR-0561). The body is never the
   source of truth.

3. **PDP authorization with true blast-radius.** A `CrmAuthorizer` Cedar-PDP
   port decides `decide(principal, action = crm.<capability>.mutate, resource)`
   where `CrmResource.target_tenant_id` is bound from the **verified principal**
   (a trusted source), never from the caller body. A verified caller whose body
   claims a different tenant is denied; a cross-tenant grant must come from the
   PDP against the verified tenant. Default-deny; any PDP fault is treated as
   deny (fail-closed → HTTP 403).

4. **Authn-before-body and refuse-to-serve.** `HttpHandler::handle` /
   `GrpcHandler::handle` now require a `&CrmAuthzProvider` plus a transport-
   supplied `CallerCredential` and run the gate FIRST. The capability comes from
   server-side route/method metadata, not the body. Verification failure → 401,
   authorization failure → 403, both mapped to an `Authorization` `ServiceError`.

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
  from the `VerifiedPrincipal` returned by `authorize_crm_command`, and the
  interactor's identity fields SHOULD drop `Deserialize` so a caller can never
  supply them. Until that refactor lands, the adapters are the only gated entry
  and MUST NOT be wired to bypass the gate into the interactor.

## Consequences

- The body `tenant_id` / `principal_id` fields are retained as non-authoritative
  cross-check data (they grant nothing). They MAY be removed entirely once the
  proto/DTO contract is revised; that is a follow-on contract change.
- The break-glass `ConfiguredBearerPrincipalVerifier` binds a single static
  identity to one shared secret — suitable only for a single-principal
  break-glass token or tests. Multi-tenant production uses the cloud-iam SVID
  verifier (ADR-0561).
- Tests cover the RED paths (no credential → 401, bad bearer → 401, cross-tenant
  body claim → 403, PDP deny → 403, PDP fault → 403) and the GREEN path
  (verified + PDP grant → reaches the scaffolded business handler).
