# Spec: oidc-token-introspection-kernel

## Objective

Extend `identity-oidc-issuer-kernel` with a pure RFC 7662 OAuth 2.0
Token Introspection response surface. The addition consists of three
tightly-scoped pieces added to `src/lib.rs`:

1. `IntrospectionRequest` — structural validator (token non-empty;
   optional `token_type_hint` restricted to the `access_token` |
   `refresh_token` set via a typed enum).
2. `IntrospectionResponse` — RFC 7662 value type: `active: bool` plus
   optional disclosed claims; an `inactive()` constructor that guarantees
   the RFC 7662 §2.2 privacy rule (no additional fields when inactive).
3. `build_introspection_response` — pure builder that reuses the existing
   `check_temporal_window` / `TokenTemporalStatus` state machine to decide
   active vs inactive, then maps `AccessTokenClaims` into the disclosed
   claim set.

All additions are pure, deterministic, no-I/O, panic-free, and return
`Result<_, IssuerError>`. No new crate, no new file, no root Cargo.toml
edit.

## Vertical and crate

- Lane: `foundation`
- Crate: `identity-oidc-issuer-kernel`
  (`crates/identity-oidc-issuer-kernel/`)
- ADR alignment:
  - ADR-0083 Tier 3 panic-free invariant
  - ADR-0131 per-microservice flat layout
  - ADR-0509 single-crate-per-service with mod-based subsystems
- RFC authority: RFC 7662 (OAuth 2.0 Token Introspection)

## Contracts

### No HTTP/gRPC surface

This crate is a pure domain kernel with no network surface. All contracts
are Rust type signatures. HTTP serialization (JSON field names per RFC 7662
§2.2) belongs in the REST adapter, not here.

### `TokenTypeHint` — typed hint enum

```rust
/// Hint about the token type presented for introspection (RFC 7662 §2.1).
/// The kernel accepts only the two standardised values; free-text hints
/// are an adapter responsibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenTypeHint {
    /// `access_token` hint.
    AccessToken,
    /// `refresh_token` hint.
    RefreshToken,
}
```

### `IntrospectionRequest` — structural validator

```rust
/// Structural representation of an RFC 7662 §2.1 introspection request.
/// The kernel validates token non-emptiness; `token_type_hint` is already
/// typed so the allowed-set constraint is enforced by the type system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrospectionRequest {
    /// The token string to introspect (opaque to the kernel).
    pub token: String,
    /// Optional hint about the token type.
    pub token_type_hint: Option<TokenTypeHint>,
}

impl IntrospectionRequest {
    /// Validate the shape of an introspection request.
    ///
    /// # Errors
    /// Returns [`IssuerError::MalformedIntrospectionRequest`] when `token`
    /// is empty or whitespace-only.
    pub fn validate(
        token: impl Into<String>,
        token_type_hint: Option<TokenTypeHint>,
    ) -> Result<Self, IssuerError>
}
```

New `IssuerError` variant:

```rust
/// Introspection request was malformed (e.g. empty token).
MalformedIntrospectionRequest(&'static str),
```

### `ActiveIntrospectionClaims` — disclosed claim set carrier

```rust
/// Disclosed claim set for an active RFC 7662 introspection response.
/// Passed to [`IntrospectionResponse::active`] to keep the constructor
/// under the `clippy::too_many_arguments` limit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveIntrospectionClaims {
    pub sub: String,
    pub aud: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub scope: Option<String>,
    pub client_id: Option<String>,
    pub tenant_id: Option<String>,
    pub token_type: Option<String>,
}
```

### `IntrospectionResponse` — RFC 7662 value type

```rust
/// RFC 7662 §2.2 introspection response.
///
/// Privacy rule: when `active` is `false`, all other fields MUST be absent.
/// The [`IntrospectionResponse::inactive`] constructor enforces this.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntrospectionResponse {
    /// Whether the token is currently active (RFC 7662 §2.2).
    pub active: bool,
    /// `sub` — subject identifier.
    pub sub: Option<String>,
    /// `aud` — intended audience.
    pub aud: Option<Vec<String>>,
    /// `exp` — expiry (epoch seconds).
    pub exp: Option<i64>,
    /// `iat` — issuance time (epoch seconds).
    pub iat: Option<i64>,
    /// `scope` — space-separated scope string.
    pub scope: Option<String>,
    /// `client_id` — OAuth 2.0 client identifier.
    pub client_id: Option<String>,
    /// `tenant_id` — oyatie tenant identifier (ADR-0244 superset).
    pub tenant_id: Option<String>,
    /// `token_type` — e.g. `"at+jwt"` for RFC 9068 access tokens.
    pub token_type: Option<String>,
}

impl IntrospectionResponse {
    /// Construct an inactive response. RFC 7662 §2.2 privacy rule:
    /// only `{"active": false}` is disclosed for unknown or invalid tokens.
    pub fn inactive() -> Self

    /// Construct an active response from the disclosed claim set.
    pub fn active(claims: ActiveIntrospectionClaims) -> Self
}
```

### `build_introspection_response` — pure builder

```rust
/// Build an RFC 7662 introspection response from an `AccessTokenClaims`
/// value and the caller's validation clock.
///
/// Reuses [`check_temporal_window`] + [`TokenTemporalStatus`] for the
/// active/inactive verdict. Expired and not-yet-valid tokens collapse to
/// `{"active": false}` per RFC 7662 §2.2 without leaking error details.
///
/// # Errors
/// This function is currently infallible (returns `Ok` in all branches)
/// but is typed `Result<_, IssuerError>` for forward compatibility with
/// future kernel extensions that may need to signal structural faults.
pub fn build_introspection_response(
    claims: &AccessTokenClaims,
    now_epoch_seconds: i64,
    skew: ClockSkewTolerance,
) -> Result<IntrospectionResponse, IssuerError>
```

Disclosed claim mapping (active branch only):

| `AccessTokenClaims` field | `IntrospectionResponse` field |
|---------------------------|-------------------------------|
| `sub`                     | `sub`                         |
| `aud`                     | `aud`                         |
| `exp`                     | `exp`                         |
| `iat`                     | `iat`                         |
| `scope` (non-empty)       | `scope`                       |
| _(no client_id field)_    | `client_id = None`            |
| `tenant_id`               | `tenant_id`                   |
| `token_type`              | `token_type`                  |

`scope` promotion: an empty string scope field is promoted to `None`
(an access token with no scope discloses nothing rather than `""`).

## Module layout (flat clean-arch, single file)

All additions land in `crates/identity-oidc-issuer-kernel/src/lib.rs`.
No new files, no new modules. Insertion order in `lib.rs`:

```
… existing types …
TokenTypeHint          (new)
IntrospectionRequest   (new)
IssuerError variant    (new variant added to existing enum)
IntrospectionResponse  (new)
build_introspection_response  (new free fn)
… existing #[cfg(test)] mod tests … (new test cases appended)
```

## Testing strategy

All tests follow the existing pattern: inline `#[cfg(test)] mod tests` in
`lib.rs`, with the integration-test file `tests/oidc_issuer_kernel.rs`
extended for cross-module coverage if needed.

### oidc-introspect-1 acceptance tests

| Test name | Assertion |
|-----------|-----------|
| `introspection_request_rejects_empty_token` | `validate("", None)` → `Err(MalformedIntrospectionRequest(_))` |
| `introspection_request_rejects_whitespace_token` | `validate("  ", None)` → `Err(MalformedIntrospectionRequest(_))` |
| `introspection_request_accepts_token_without_hint` | `validate("tok", None)` → `Ok(...)` with `token_type_hint = None` |
| `introspection_request_accepts_token_with_hint` | `validate("tok", Some(AccessToken))` → `Ok(...)` with hint set |

### oidc-introspect-2 acceptance tests

| Test name | Assertion |
|-----------|-----------|
| `introspection_response_inactive_has_no_disclosed_fields` | `inactive()` → `active=false`, all `Option` fields `None` |
| `introspection_response_active_carries_disclosed_claims` | `active(...)` → `active=true`, fields match inputs |

### oidc-introspect-3 acceptance tests

| Test name | Assertion |
|-----------|-----------|
| `build_introspection_response_active_token` | `now` within validity window → `active=true`, claims mapped correctly |
| `build_introspection_response_expired_token` | `now > exp + skew` → `active=false`, no other fields |
| `build_introspection_response_not_yet_valid_token` | `now + skew < nbf` → `active=false`, no other fields |

All tests use a fixed `now_epoch_seconds` value to remain deterministic.

## Boundaries

- Modify ONLY `crates/identity-oidc-issuer-kernel/src/lib.rs`.
- Do NOT edit root `Cargo.toml`.
- Do NOT add any new crate or new source file.
- Do NOT re-implement expiry math; delegate entirely to
  `check_temporal_window` + `TokenTemporalStatus`.
- No external dependencies; this crate remains zero-dependency.
- No I/O, no clock access, no RNG.
- No `panic!`, `unwrap`, or `expect` outside `#[cfg(test)]`.
