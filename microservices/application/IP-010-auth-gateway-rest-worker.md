---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-010-auth-gateway-rest-worker
status: pending
execution_unit: ChangeSet
owner: axis-application + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, openapi-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: auth-gateway usecase + rest + worker

## Intent

Combined IP: usecase + rest + worker layers for auth-gateway.
- usecase: SignInUseCase, SessionRefreshUseCase, SignOutUseCase, RevokeUseCase.
- rest: axum handlers for `/auth/start`, `/auth/callback`, `/auth/saml/callback`,
  `/auth/session` GET + DELETE.
- worker: session-rotation reaper (rotates idle TTL); revocation propagation
  on `EmployeeTerminated` event.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-auth-gateway-usecase/{Cargo.toml,src/{lib,signin,refresh,signout,revoke}.rs}` | create |
| `microservices/application/src/crates/oya-application-auth-gateway-rest/{Cargo.toml,src/{lib,router,handlers,middleware}.rs}` | create |
| `microservices/application/src/crates/oya-application-auth-gateway-worker/{Cargo.toml,src/{lib,main,rotator,revoker}.rs}` | create |
| 3 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
// usecase
pub struct SignInUseCase<O, S> { oidc: O, sessions: S }

impl<O: IdpClient, S: SessionStore> SignInUseCase<O, S> {
    pub async fn callback(&self, code: &str, state: &str, pkce_verifier: &str) -> Result<Session, UseCaseError> {
        let token = self.oidc.exchange_code(code, pkce_verifier).await?;
        let claims = self.oidc.verify_id_token(&token.id_token).await?;
        let session = Session::new_random(&claims);
        self.sessions.insert(&session).await?;
        Ok(session)
    }
}

// worker
pub async fn run_rotator(deps: WorkerDeps) {
    let mut tick = tokio::time::interval(Duration::from_secs(60));
    loop {
        tick.tick().await;
        deps.sessions.expire_idle(Duration::from_minutes(15)).await.unwrap_or_default();
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-auth-gateway-usecase --all-features
cargo nextest run -p oya-application-auth-gateway-rest --all-features
cargo nextest run -p oya-application-auth-gateway-worker --all-features
cargo run -p oya-dev-cli -- gate validate openapi-conformance --crate oya-application-auth-gateway-rest
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_oidc_signin_two_cookie` | two-cookie + PKCE + nonce contract |
| `test_session_create_p99_under_200ms` | budget |
| `test_session_idle_expiry` | rotator works |
| `test_session_revoke_on_employee_terminated` | revocation propagates |
| `test_csrf_token_required_on_mutation` | T-04 |
| `test_constant_time_signin_response` | L-04 timing oracle |

## Next IP

[`IP-011-module-loader-kernel-domain.md`](IP-011-module-loader-kernel-domain.md)
