---
doc_class: Owner-PRD
owner: iam
status: Active
date: 2026-08-27
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - iam/ADR.md
---

# IAM product requirements

<product_boundary>

IAM proves who a human or workload principal is and supplies device posture and
role state. It consumes federation, provisions identities through SCIM into an
existing tenant, manages passkey and principal lifecycle, stores roles, and
compiles role state into deterministic Cedar input for Policy.

IAM does not decide whether an action is allowed, issue SVIDs or key material,
create tenants, run Kubernetes admission, or host HR, Payroll, Accounting, or
other application workflows. Those owners interact through explicit ports and
sold facades.

</product_boundary>

<users>

- Tenant identity administrators provision and suspend human and workload
  principals, bind external identities, manage passkeys and roles, and inspect
  durable evidence.
- Workloads present federation or workload credentials and receive a verified,
  tenant-bound principal outcome.
- Policy consumes stable principal and compiled-role inputs and independently
  decides `Check`.
- Application adapters consume the sold IAM contract without importing IAM
  core or receiving first-party privileges.
- Security operators rotate verification material, investigate refusals, and
  observe freshness, latency, availability, and recovery without exposing
  credential material.

</users>

<landed_scope>

## Current foundation

The reviewed tree contains useful identity behavior in these areas:

- `core/identity-domain` and `core/identity-usecase`: typed human/service
  principals, identity-provider binding, credential lifecycle, tenant binding,
  and idempotent identity operations.
- `ports/identity-api`: request/principal/tenant binding and stable identity
  operation outcomes.
- `core/identity-workload-domain` and
  `adapters/identity-workload-oidc`: workload lifecycle plus signed-token,
  algorithm, key, issuer, audience, time, type, and claim verification.
- `core/scim-server-kernel` and `adapters/identity-scim-store-postgres`: SCIM
  semantics and a durable store path.
- `ports/device-attestation` and related identity packages: the beginning of a
  posture boundary.

This list is retained behavior, not a maturity claim. The current identity
facade still carries REST and embedded authorization debt, no owner-approved
generated Connect identity contract has landed, and the role-store-to-Cedar
destination is incomplete.

The 39-crate tenant-rbac cone and its four hand-authored OpenSLO files are not
part of the foundation. Their removal remains blocked by the founder amendment
gate in `iam/ADR.md`.

</landed_scope>

<requirements>

## Principal lifecycle and federation

- Human and workload principals have stable opaque identifiers, tenant scope,
  lifecycle state, credential binding, assurance context, and auditable
  provenance.
- Federation consumption verifies signature, algorithm/key binding, issuer,
  audience, time window, token type, and required claims before constructing a
  principal. Network-fetched material is refreshed outside the verification
  hit path and an unknown or stale key never becomes an allow.
- Passkey and device-attestation results are identity context. Higher assurance
  is explicit; absence or expiry cannot be silently normalized upward.
- Credential and principal lifecycle operations are idempotent, tenant-bound,
  and durable before acknowledgement.

## SCIM and role state

- SCIM provisions users and groups only inside an existing tenant. Tenant
  lifecycle remains owned by `tenancy/`.
- Role storage preserves tenant scope, version, source, lifecycle, and
  idempotency. Its compiler produces deterministic, bounded Cedar input and a
  versioned digest for Policy; it never returns an authorization decision.
- Policy failure, snapshot staleness, or relation-store unavailability cannot
  be converted into an IAM allow. Policy owns that refusal or routing result.

## Architecture and transport

- Domain and use-case logic remain in `core/`; I/O contracts remain in
  `ports/`; OIDC/JWKS, SCIM, passkey, attestation, and storage implementations
  remain in `adapters/`; processes and generated Connect surfaces remain in
  `facade/`.
- A first-party application uses the same public identity contract and
  principal class as an external tenant. No IAM package imports an app package.
- A future Connect identity facade starts only after its protobuf, generator,
  runtime, and failure vectors are owner-approved. The tenant-rbac surface is
  neither a compatibility source nor a migration wrapper.

</requirements>

<success_and_failure>

## Success

IAM succeeds when a valid credential is deterministically projected to the
correct tenant-bound principal, identity/SCIM/role mutations survive restart
and replay, role compilation yields the same versioned artifact for the same
state, and downstream Policy receives inputs without IAM deciding the result.
The capability remains substitutable at every external boundary and exposes no
vertical application behavior.

## Failure

IAM fails when it accepts an expired, unsigned, wrong-issuer, wrong-audience,
wrong-tenant, unknown-key, or insufficient-assurance credential; acknowledges a
mutation that is not durable; emits nondeterministic role compilation; embeds a
Cedar/ReBAC decision or key/SVID issuer; imports an application; claims a
listener or SLO from static review data; or retains any tenant-rbac removal
inventory after an authorized atomic deletion.

</success_and_failure>

<service_objectives>

## Promotion objectives

These are target objectives, not claims about the current implementation. They
become promotion evidence only when generated from IR and measured on a real
facade or controller.

| Signal | Objective | Failure reading |
|---|---|---|
| Principal verification availability | At least 99.99% over a rolling 28 days for otherwise-valid requests while valid local verification material exists | Dependency or verifier failure is `UNAVAILABLE`/refused, never allowed |
| Principal verification latency | p99 service handling at or below 50 ms over 5-minute windows, excluding public-door transit | A missed window pages; bypass and downgraded validation are forbidden |
| Durable identity/SCIM mutation availability | At least 99.9% over a rolling 28 days for valid operations | No acknowledgement before durable commit; retry returns the committed idempotent result |
| Durable identity/SCIM mutation latency | p99 service handling at or below 500 ms over 5-minute windows | Backpressure or unavailable is preferred to an uncommitted acknowledgement |
| Role compilation freshness | At least 99.9% of acknowledged role changes produce a versioned Policy-consumable artifact within 60 seconds | Stale or absent output cannot authorize and is surfaced as stale/unavailable |
| Tenant-isolation safety | Zero accepted cross-tenant principal, SCIM, or role mutations | Any occurrence is a security incident and promotion blocker, not an error-budget event |

</service_objectives>

<failure_injection>

## Required failure campaigns

| Injection | Required result |
|---|---|
| Expired/not-yet-valid token, `alg=none`, algorithm substitution, unknown key, wrong issuer/audience/type, or untrusted key-source URL | Principal construction is refused before use-case execution; secret material is not logged |
| Credential tenant differs from request/resource tenant | Refused before read or mutation; one tenant-scoped denial record is emitted |
| JWKS refresh stops after cached material expires | New verification refuses or reports unavailable; no last-known-good silent allow |
| SCIM/store process crashes before commit, after commit, or after commit before reply | No partial record; reopen is deterministic; replay returns the stored result or a typed conflict |
| Role compiler crashes, reorders input, or publishes stale output | No new artifact is advertised; stale state cannot become an allow; retry produces the same digest |
| Device-attestation evidence is missing, expired, or downgraded | Principal context carries no stronger assurance and step-up remains unsatisfied |
| Policy is unavailable | IAM does not manufacture a decision; the consuming PEP follows Policy's fail-closed contract |
| Caller requests SVID, certificate, or signing-key issuance through IAM | No such IAM kernel path exists; the request resolves to the Secrets-owned facade or is refused |

</failure_injection>

<not_in_scope>

The following are not IAM feature requirements: preserving tenant-rbac REST,
splitting its manifests, building an IAM-hosted application shell, replacing
the accepted Kubernetes admission invariant, implementing Kubernetes admission
inside IAM, or constructing the future Connect facade as part of deletion.

</not_in_scope>
