# oya-identity

Bespoke Rust human identity substrate (Wave C).

Supersedes Keycloak Phase-1 per **ADR-0476**.

## Scope

- OIDC provider state machine
- OAuth 2.x grant flows
- Token issuance and validation
- WebAuthn / passkey relying party
- SCIM server surface

## Crate layout

| Crate | Layer | Purpose |
|---|---|---|
| `oya-identity-kernel` | kernel | Pure-Rust no-I/O domain logic (OIDC, OAuth, token) |
| `oya-identity-rest` | rest | Axum HTTP adapter exposing OIDC/OAuth endpoints |
| `oya-identity-app` | app | Binary: wires kernel + REST + storage |

## ADR reference

- [ADR-0476](../../docs/adr-archive/ADR-0476-oya-identity-bespoke-human-identity.md) — authoritative design decision superseding Keycloak Phase-1

## SLOs

See `slos/availability.openslo.yaml` — 99.95% availability target.
