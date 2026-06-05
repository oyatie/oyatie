---
doc_class: Standard
standard_id: identity-vendor-isolation
status: Accepted
date: 2026-05-18
owner_team: axis-identity + council-architecture
related_adrs: [ADR-0173, ADR-0187]
related_lanes: [lean-a18-identity-vendor-isolation]
---

# Identity Vendor Isolation — Standard

Per ADR-0187 §"In-house roadmap" and ADR-0173 vendor-lock-in-avoidance, Zitadel-specific dependencies live ONLY inside the explicit adapter set. The rest of the fleet talks to identity through stable open-standard surfaces (OIDC / SAML / SCIM / WebAuthn). This ensures Phase-2 swap to `oya-identity-server` is mechanical.

## Allowed-to-import-Zitadel-types crates

ONLY these crates may reference Zitadel-specific gRPC types or Postgres schema:

- `oya-identity-oidc-issuer-adapter-zitadel`
- `oya-identity-scim-server-adapter-zitadel`
- `oya-identity-zitadel-instance-controller-adapter`

## Forbidden-to-import-Zitadel-types

Every other crate in the fleet — including all `oya-shared-*` kernels — MUST NOT import Zitadel-specific types. They consume identity through:

- OIDC: `oya-shared-oidc-client-kernel` (vendor-neutral trait).
- WebAuthn: `oya-shared-webauthn-server-kernel` (vendor-neutral trait).
- SCIM: `oya-shared-scim-server-kernel` (vendor-neutral trait).
- SAML: standard SAML 2.0 XML envelope; no Zitadel-specific shape.

## Forbidden patterns

```rust
// FORBIDDEN outside adapter set:
use zitadel_admin_pb::Management_AddHumanUserRequest;
use zitadel::orgs::OrgsClient;

// REQUIRED instead:
use oya_shared_oidc_client_kernel::{OidcClient, OidcClaims};
use oya_shared_scim_server_kernel::{ScimServer, User};
```

## Forbidden Helm references

Only `microservices/identity/iac/helm/zitadel/` references Zitadel chart artefacts. Other µservices' Helm charts MUST NOT have `dependencies` on Zitadel.

## Forbidden DB schema references

Only `oya-identity-zitadel-instance-controller-*` crates read or migrate Zitadel's Postgres event-store schema.

## Verification

CI lane `lean-a18-identity-vendor-isolation` (advisory-mode initially; blocker after 60 days clean) runs:

- Buck2/Prow dependency-graph target for every workspace crate; refuses
  Zitadel transitive deps outside the allowlist while treating Cargo metadata as
  graph input only.
- Buck2/Prow IaC reference scan target over identity-owned KRM/CUE surfaces;
  retired `microservices/*/iac` Helm path checks are not the active authority.

## Phase-2 swap protocol

When ADR-0187 §"In-house roadmap" Phase-2 trigger is met:

1. Land `oya-identity-server` in a parallel µservice path.
2. Re-wire the adapter set's trait impls to call `oya-identity-server`.
3. NO consumer crate changes required (the kernel trait surface is stable).
4. Run shadow-traffic comparison for 30 days.
5. Cutover one pack at a time, bellwether first.
6. Decommission Zitadel deployment per pack.

## Cross-references

- ADR-0187 (§"In-house roadmap")
- ADR-0173 (vendor-lock-in)
- ADR-0188 (§"In-house roadmap" — analogous discipline for webauthn-rs)
- `crates/oya-shared-oidc-client-kernel`
- `crates/oya-shared-webauthn-server-kernel`
- `crates/oya-shared-scim-server-kernel`
