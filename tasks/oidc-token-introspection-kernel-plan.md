# Task plan: oidc-token-introspection-kernel

Lane: foundation
Crate: oya-identity-oidc-issuer-kernel
Branch: feat/task-oidc-token-introspection-kernel-2026-05-28

## Objective

Add a pure RFC 7662 OAuth 2.0 Token Introspection response surface to the
existing OIDC issuer kernel: an `IntrospectionRequest` validator, an
`IntrospectionResponse` value type with RFC 7662 privacy semantics, and a
pure builder (`build_introspection_response`) that reuses the existing
`check_temporal_window`/`TokenTemporalStatus` state machine. No I/O, no
panic, no new crate, no root Cargo.toml edit.

## Subtasks

### oidc-introspect-1 — `IntrospectionRequest` structural validator

File: `crates/oya-identity-oidc-issuer-kernel/src/lib.rs`

Add `IntrospectionRequest` with a `validate` constructor (mirrors
`RefreshRequest::validate` style):

```rust
pub struct IntrospectionRequest {
    pub token: String,
    pub token_type_hint: Option<TokenTypeHint>,
}

pub enum TokenTypeHint {
    AccessToken,
    RefreshToken,
}

impl IntrospectionRequest {
    pub fn validate(
        token: impl Into<String>,
        token_type_hint: Option<TokenTypeHint>,
    ) -> Result<Self, IssuerError>
}
```

Validation rules:
- `token` must not be empty or whitespace-only → `IssuerError::MalformedIntrospectionRequest("token must not be empty")`
- `token_type_hint` is already typed; callers that pass a raw string are
  adapter-side and must parse before calling (no free-text hint accepted
  by the kernel).

Add `IssuerError::MalformedIntrospectionRequest(&'static str)` variant
and its `Display` arm.

Acceptance:
- `cargo check -p oya-identity-oidc-issuer-kernel --all-targets` passes.
- Unit test (`#[cfg(test)] mod tests` inline) rejects empty token and
  accepts well-formed request with and without a hint.

### oidc-introspect-2 — `IntrospectionResponse` value type

File: `crates/oya-identity-oidc-issuer-kernel/src/lib.rs`

Add `IntrospectionResponse` with RFC 7662 privacy rule: an inactive
response carries only `active: false` and nothing else.

```rust
pub struct IntrospectionResponse {
    pub active: bool,
    pub sub: Option<String>,
    pub aud: Option<Vec<String>>,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
    pub scope: Option<String>,
    pub client_id: Option<String>,
    pub tenant_id: Option<String>,
    pub token_type: Option<String>,
}

impl IntrospectionResponse {
    /// Inactive constructor — guarantees no other field is populated (RFC 7662 §2.2).
    pub fn inactive() -> Self

    /// Active constructor — caller supplies the disclosed claim set.
    pub fn active(
        sub: String,
        aud: Vec<String>,
        exp: i64,
        iat: i64,
        scope: Option<String>,
        client_id: Option<String>,
        tenant_id: Option<String>,
        token_type: Option<String>,
    ) -> Self
}
```

Acceptance:
- `cargo nextest run -p oya-identity-oidc-issuer-kernel` passes.
- Test asserts `inactive()` yields `active=false` with all `Option` fields
  `None`.
- Test asserts `active(...)` carries `active=true` and the expected fields.

### oidc-introspect-3 — `build_introspection_response` builder

File: `crates/oya-identity-oidc-issuer-kernel/src/lib.rs`

Add a pure builder that reuses `check_temporal_window`/`TokenTemporalStatus`:

```rust
pub fn build_introspection_response(
    claims: &AccessTokenClaims,
    now_epoch_seconds: i64,
    skew: ClockSkewTolerance,
) -> Result<IntrospectionResponse, IssuerError>
```

Logic:
1. Call `check_temporal_window(now, claims.nbf, claims.exp, skew)`.
2. If `Err(_)` (expired or not-yet-valid) → return `Ok(IntrospectionResponse::inactive())`.
3. If `Ok(TokenTemporalStatus::Valid)` → map `AccessTokenClaims` fields
   into an active `IntrospectionResponse` (sub, aud, exp, iat, scope,
   tenant_id as `client_id` is `None` at kernel layer since `AccessTokenClaims`
   has no `client_id` field; token_type from `claims.token_type`).

Disclosed claim mapping from `AccessTokenClaims`:
- `sub` → `claims.sub.clone()`
- `aud` → `claims.aud.clone()`
- `exp` → `claims.exp`
- `iat` → `claims.iat`
- `scope` → `Some(claims.scope.clone())` (empty string promoted to `None`)
- `client_id` → `None` (kernel layer has no client_id in AccessTokenClaims)
- `tenant_id` → `Some(claims.tenant_id.clone())`
- `token_type` → `Some(claims.token_type.clone())`

Acceptance:
- `cargo nextest run -p oya-identity-oidc-issuer-kernel` passes.
- Tests cover:
  - active token → `active=true` with correct disclosed claims
  - expired token (`now > exp + skew`) → `active=false`
  - not-yet-valid token (`now + skew < nbf`) → `active=false`
- No panic/unwrap/expect in non-test code (clippy clean under workspace lints).

## Verification commands

```
cargo check -p oya-identity-oidc-issuer-kernel --all-targets
cargo nextest run -p oya-identity-oidc-issuer-kernel
```

Run from worktree root:
`/tmp/oya-task-oidc-token-introspection-kernel-2026-05-28`

## Boundaries

- Touch ONLY `crates/oya-identity-oidc-issuer-kernel/src/lib.rs`.
- Do NOT edit root `Cargo.toml`.
- Do NOT add any new crate or new file.
- No I/O, no crypto, no external dependencies.
- Reuse `check_temporal_window` + `TokenTemporalStatus`; do not re-implement
  expiry math.
