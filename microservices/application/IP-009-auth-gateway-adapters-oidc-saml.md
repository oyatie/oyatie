---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-009-auth-gateway-adapters-oidc-saml
status: pending
execution_unit: ChangeSet
owner: axis-application + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: auth-gateway adapter-oidc + adapter-saml + adapter

## Intent

Backend-qualified adapters per ADR-0105 Amendment 3:

- `adapter-oidc`: OIDC IdP client; JWKS pinning; alg allow-list ES256/RS256;
  audience pin.
- `adapter-saml`: SAML IdP client; XSW-hardened verify; metadata refresh.
- `adapter`: protocol-neutral Postgres SessionStore + Valkey session-cache.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-auth-gateway-adapter-oidc/{Cargo.toml,src/{lib,jwks,verify}.rs}` | create |
| `microservices/application/src/crates/oya-application-auth-gateway-adapter-saml/{Cargo.toml,src/{lib,verify,metadata}.rs}` | create |
| `microservices/application/src/crates/oya-application-auth-gateway-adapter/{Cargo.toml,src/{lib,postgres,valkey}.rs}` | create |
| `microservices/application/src/crates/oya-application-auth-gateway-api/{Cargo.toml,src/lib.rs}` | create — protocol-neutral types |
| 4 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
pub struct OidcAdapter { jwks_client: JwksClient, alg_allow: AlgAllowList }

#[async_trait]
impl IdpClient for OidcAdapter {
    async fn verify_id_token(&self, token: &str) -> Result<Claims, IdpError> {
        let header = decode_header(token)?;
        if !self.alg_allow.contains(&header.alg) {
            return Err(IdpError::AlgNotAllowed); // alg=none / HS256 with public key blocked
        }
        let jwks = self.jwks_client.get_pinned().await?;
        let claims = decode_and_verify(token, &jwks, &self.audience)?;
        Ok(claims)
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-auth-gateway-adapter-oidc --all-features
cargo nextest run -p oya-application-auth-gateway-adapter-saml --all-features
cargo nextest run -p oya-application-auth-gateway-adapter --all-features
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_oidc_alg_none_rejected` | RFC 7515 attack blocked |
| `test_oidc_hs256_with_pub_rejected` | key-confusion blocked |
| `test_oidc_audience_pin` | wrong audience refused |
| `test_oidc_jwks_pinning_kid_rotation` | new kid loaded; old still valid for grace |
| `test_saml_xsw_battery` | all 8 XSW variants blocked |
| `test_postgres_session_insert` | row + RLS |
| `test_valkey_session_ttl` | 8-h absolute; 15-min idle |

Coverage: 90 % / 80 %.

## Next IP

[`IP-010-auth-gateway-rest-worker.md`](IP-010-auth-gateway-rest-worker.md)
