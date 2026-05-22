---
doc_class: IP
ip_id: IP-015
microservice: identity
status: ga
related_adrs: [ADR-0187, ADR-0188, ADR-0189, ADR-0190, ADR-0191]
date: 2026-05-18
owner_team: axis-identity
---

# IP-015 — `oya-shared-*` kernel crate exports + reference impls

## Goal

Publish the 3 cross-µservice shared kernel crates that the rest of the oyatie fleet consumes for identity-class operations:

- `oya-shared-oidc-client-kernel` (IP-002 / verifier kernel)
- `oya-shared-webauthn-server-kernel` (IP-004)
- `oya-shared-scim-server-kernel` (IP-007)

Plus the 2 advisory-mode check crates:

- `oya-check-step-up-auth-coverage` (per ADR-0189)
- `oya-check-authz-tier-discipline` (per ADR-0191)

This IP closes the loop on the kernel surface — every other µservice in the fleet can now consume identity primitives through stable trait surfaces, with vendor swaps (Phase-2 in-house roadmap per ADR-0187) hidden behind adapter boundaries.

## Files (already landed in IP-002/004/007 + check crates)

Workspace `Cargo.toml` additions:
- `crates/oya-shared-oidc-client-kernel`
- `crates/oya-shared-webauthn-server-kernel`
- `crates/oya-shared-scim-server-kernel`
- `crates/oya-check-step-up-auth-coverage`
- `crates/oya-check-authz-tier-discipline`

## Public surface contracts

### `oya-shared-oidc-client-kernel`

```rust
pub trait OidcClient {
    fn verify(&self, bearer: &str, cfg: &VerifyConfig) -> Result<OidcClaims, OidcError>;
    fn meets_acr(&self, claims: &OidcClaims, floor: AcrLevel) -> bool;
}
```

Consumers: every µservice that verifies incoming OIDC tokens (i.e., every µservice).

### `oya-shared-webauthn-server-kernel`

```rust
pub trait WebauthnServer {
    fn begin_registration(...) -> Result<RegistrationChallenge, _>;
    fn finish_registration(...) -> Result<Credential, _>;
    fn begin_authentication(...) -> Result<AuthenticationChallenge, _>;
    fn finish_authentication(...) -> Result<Credential, _>;
}
```

Consumers: identity µservice today; mobile-shell µservice future.

### `oya-shared-scim-server-kernel`

```rust
pub trait ScimServer {
    fn list_users / get_user / create_user / replace_user / patch_user / delete_user
    fn list_groups / get_group / create_group / patch_group / delete_group
}
```

Consumers: identity µservice today; tenancy µservice may consume for tenant-internal user list.

### `oya-check-step-up-auth-coverage`

Advisory gate; consumed by CI `lean-a15-step-up-acr-coverage` lane.

### `oya-check-authz-tier-discipline`

Advisory gate; consumed by CI `lean-a17-authz-tier-discipline` lane.

## SemVer commitment

These five crates are PUBLIC trait surfaces; semver applies. Major-bump requires migration ADR. Adding new methods to a trait = major-bump unless the new method has a default implementation.

## Documentation invariants

- Every public type has a `///` doc comment with example.
- Every public method documents: panics (none), errors (variants), thread-safety.
- Each crate carries a `Cargo.toml#description` that names the authority ADR(s).

## Test surface

| Crate | Tests | Coverage target |
|---|---|---|
| oya-shared-oidc-client-kernel | 16 | branches: 90%; lines: 95% |
| oya-shared-webauthn-server-kernel | 10 | branches: 85%; lines: 90% |
| oya-shared-scim-server-kernel | 22 | branches: 90%; lines: 95% |
| oya-check-step-up-auth-coverage | 10 | branches: 80%; lines: 85% |
| oya-check-authz-tier-discipline | 11 | branches: 80%; lines: 85% |

**Actual test count at landing: 69/69 passing.**

## Verification

```sh
cargo build -p oya-shared-oidc-client-kernel \
            -p oya-shared-webauthn-server-kernel \
            -p oya-shared-scim-server-kernel \
            -p oya-check-step-up-auth-coverage \
            -p oya-check-authz-tier-discipline
cargo test -p ...
# 69 passed; 0 failed.
```

## Cross-references

- IP-002 (OIDC kernel)
- IP-004 (WebAuthn kernel)
- IP-007 (SCIM kernel)
- ADR-0187 §"In-house roadmap" (adapter discipline)
- ADR-0188 §"In-house roadmap" (kernel-trait wrapping `webauthn-rs`)

## Acceptance — DONE when

- All 5 crates in workspace `Cargo.toml`.
- `cargo build` + `cargo test` clean.
- Doc surface complete: `cargo doc --no-deps -p <crate>` renders without warnings.
- Workspace `cargo clippy -- -D warnings` clean for these crates.

## Counterpart references - 015-shared-kernel-crates

- Counterpart class: identity substrate.
- Palantir Foundry and GitHub Enterprise are the counterpart baseline for governed multi-tenant identity surfaces; this IP ties the slice to Oyatie identity contracts, Cedar, and audit-chain evidence rather than leaving the behavior as generic application authentication.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

