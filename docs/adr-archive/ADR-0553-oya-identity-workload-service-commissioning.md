---
id: ADR-0553
title: "Commission the oya-identity runnable workload-identity service (G005 slice 1)"
status: Superseded
planning_impact: false
deciders: founder (identity-layering directive 2026-06-10), agent-lane g05
date: 2026-06-11
door: two-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-702]
amended_by: []
depends_on: [ADR-0476, ADR-0510, ADR-0536]
amends: []
related: [ADR-0083, ADR-0131, ADR-0515, ADR-0543, ADR-0547, ADR-0550]
related_specs: []
milestone: W0
---

# ADR-0553: Commission the oya-identity runnable workload-identity service (G005 slice 1)

## Status

Proposed (founder ratification pending; lane sanctioned by the G005 ultragoal story; delivered
via PR #651).

## Context

G005 promotes `oya/identity` from a set of library crates into a runnable service. The founder
identity-layering directive (2026-06-10) fixes the architecture this slice must respect:
**cloud-iam is the IdP substrate** (authn authority, STS, trust root, signing via cloud-kms);
**oya/identity is product-shared identity that CONSUMES the cloud IdP**, never a parallel IdP.
Within oya/identity, the workload plane (offline JWKS validation, `WorkloadAuthorizer`, SPIFFE
trust domains) is a separate plane from human identity (ADR-0476), sharing the same trust root.

Hyperscaler precedent for the workload plane is offline credential verification from a static
key set (AWS IAM / Entra / OCI convergence; EKS-IRSA and SPIFFE OIDC-federation publish
discovery + JWKS for workload-token relying parties without being a human IdP).

## Decision

Promote `oya-identity` to a runnable workload-identity service binary that composes the
existing workload-identity crates (domain, usecase, Cedar adapter, OIDC validation adapter,
REST/gRPC delivery) behind one boot path:

- `iam/facade/identity-service/src/server.rs` — the single composition root used by both
  `main` and the E2E suite: fail-fast config -> JWKS/Cedar/seed load -> independently bound
  axum REST + tonic gRPC sockets with graceful SIGTERM drain (ADR-0083 Tier 3 panic-free boot).
- `iam/facade/identity-service/src/oidc/issuer.rs` — the OIDC issuer DELIVERY surface over
  `oya-identity-oidc-issuer-kernel`: RFC 8414 discovery + RFC 7517 JWKS publication only,
  config-gated OFF unless a signing key is mounted. Signing custody is the `Es256FileSigner`
  ADR-0510 transient adapter behind the kernel `JwsSigner` port; the G02 cloud-kms adapter
  replaces it behind the SAME trait. The RFC 9068 mint is an unrouted use-case (workload
  `at+jwt` only) — no token endpoint, no authorize endpoint, no human credential routes ship in
  this slice (the legacy webauthn module is deleted), so the service cannot act as a parallel
  human IdP. Human-OIDC-issuance expansion requires the ADR decomposing ADR-0536 D-1 plus
  founder ratification before it may land.
- `iam/facade/identity-service/tests/e2e_service.rs` — live-socket E2E rung (AMENDMENT 7):
  real ES256 mint -> validate -> authorize over REST and gRPC with the fail-closed contract
  asserted (deny is 403 never 404; deny is a gRPC response value never an RPC error), SCIM
  guard refusal classes, and graceful-drain coverage through the production boot path.

The SCIM 2.0 surface (RFC 7644 Users/Groups over `libs/oya-shared-scim-server-kernel`) is the
plane-2 capability ADR-0476 sanctions for oya-identity; in this slice it is the in-memory
reference adapter guarded by the full workload PEP path (offline validation, repository,
denylist, Cedar action `identity.scim.Manage`, tenant binding) — fail-closed.

## Governed surfaces

`iam/facade/identity-service/src/server.rs`
`iam/facade/identity-service/src/oidc/issuer.rs`
`iam/facade/identity-service/tests/e2e_service.rs`

## Consequences

- The cutover litmus holds: swapping the file signer for the G02 cloud-kms signer, or the
  in-memory principal store for the G03 durable store, replaces adapters behind unchanged
  ports; `server::start` wiring is untouched.
- The service consumes deployment-mounted verification material (the cloud IdP's published
  JWKS); it owns no identity truth. Any future surface that issues or owns identity truth must
  cite this ADR's guardrail and the founder identity-layering directive before landing.
- Tested wiring IS production wiring: `server::start` is the only boot path, exercised by the
  E2E suite on live sockets.
