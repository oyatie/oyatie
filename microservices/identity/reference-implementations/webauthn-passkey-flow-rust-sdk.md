---
doc_class: ReferenceImplementation
microservice: identity
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Full WebAuthn passkey flow via the identity Rust SDK

A runnable example that:

1. Registers a tenant + user.
2. Generates a WebAuthn Level 3 registration challenge.
3. Simulates a hardware authenticator producing attestation.
4. Verifies registration server-side + binds the credential.
5. Performs an authentication ceremony.
6. Issues an OIDC token with all required claims.
7. Performs session step-up to AAL3 for high-risk action.
8. Verifies audit-chain emissions.

## Cargo.toml

```toml
[package]
name = "identity-passkey-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-identity-client = { path = "../../../../crates/oya-identity-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
webauthn-rs = "0.5"  # Pure Rust WebAuthn implementation
webauthn-rs-proto = "0.5"
rand = "0.8"
ed25519-dalek = "2.1"
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use oya_identity_client::{
    IdentityClient, IdentityClientConfig,
    UserCreate, AudienceType, SessionClass, AuthPolicy,
    WebauthnRegistrationOptions, WebauthnRegistrationVerify,
    WebauthnAuthenticationOptions, WebauthnAuthenticationVerify,
    OidcTokenIssue, SessionStepUp, RequiredAcr,
};
use oya_cedar_client::CedarPrincipal;
use webauthn_rs_proto::{
    PublicKeyCredentialCreationOptions, PublicKeyCredentialRequestOptions,
    RegisterPublicKeyCredential, PublicKeyCredential,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Tenant admin client to create user.
    let admin_principal = CedarPrincipal::from_env("IDENTITY_ADMIN_JWT")?;
    let admin_client = IdentityClient::connect(IdentityClientConfig {
        cell_endpoint: std::env::var("IDENTITY_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: admin_principal,
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Create a workforce user.
    let user = admin_client.user_create(UserCreate {
        principal_id: "u-alice@acme-corp.com".into(),
        email: "alice@acme-corp.com".into(),
        display_name: "Alice Anderson".into(),
        audience_type: AudienceType::Workforce,
        session_class: SessionClass::WorkforceStandard,
        auth_policy: AuthPolicy::PasskeyPrimary,
        require_hardware_passkey_for_roles: vec![
            "drive_admin".into(),
            "compliance_admin".into(),
        ],
    }).await?;
    info!("User created: {} (onboarding_token={})",
          user.principal_id, user.onboarding_token);

    // 3. Generate WebAuthn registration options.
    let reg_options = admin_client.webauthn_registration_options(
        WebauthnRegistrationOptions {
            user_id: user.principal_id.clone(),
            onboarding_token: Some(user.onboarding_token.clone()),
            required_authenticator_class: Some("hardware-backed".into()),
        }
    ).await?;
    info!("Registration challenge issued: id={}", reg_options.challenge_id);

    // 4. Simulate hardware authenticator (in production, navigator.credentials.create()).
    let attestation = simulate_yubikey_attestation(&reg_options).await?;

    // 5. Server verifies + binds credential.
    let credential_binding = admin_client.webauthn_registration_verify(
        WebauthnRegistrationVerify {
            challenge_id: reg_options.challenge_id.clone(),
            user_id: user.principal_id.clone(),
            client_data_json_b64: attestation.client_data_json_b64,
            attestation_object_b64: attestation.attestation_object_b64,
        }
    ).await?;
    info!("Credential bound: id_hash={}, aaguid={}, attestation_class={}",
          credential_binding.credential_id_hash,
          credential_binding.aaguid,
          credential_binding.attestation_class);

    // 6. Switch to user's client context for authentication.
    let user_client = IdentityClient::connect_unauthenticated(IdentityClientConfig {
        cell_endpoint: std::env::var("IDENTITY_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: CedarPrincipal::unauthenticated(),
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 7. Authentication ceremony.
    let auth_options = user_client.webauthn_authentication_options(
        WebauthnAuthenticationOptions {
            user_id: user.principal_id.clone(),
        }
    ).await?;
    info!("Authentication challenge issued: id={}", auth_options.challenge_id);

    let assertion = simulate_yubikey_assertion(&auth_options).await?;
    let session = user_client.webauthn_authentication_verify(
        WebauthnAuthenticationVerify {
            challenge_id: auth_options.challenge_id.clone(),
            client_data_json_b64: assertion.client_data_json_b64,
            authenticator_data_b64: assertion.authenticator_data_b64,
            signature_b64: assertion.signature_b64,
        }
    ).await?;
    info!("Session created: id={}, acr={:?}, amr={:?}, credential_epoch={}",
          session.session_id, session.acr, session.amr, session.credential_epoch);

    // 8. Issue OIDC token.
    let user_authenticated = user_client.with_session(session.session_id.clone());
    let token = user_authenticated.oidc_token_issue(OidcTokenIssue {
        audience: "drive-api".into(),
        scopes: vec!["drive:files:read".into(), "drive:files:write".into()],
    }).await?;
    info!("OIDC token issued: token={}, expires_in={}",
          &token.access_token[..32], token.expires_in);

    // Decode the JWT claims (the JWT is human-readable for inspection).
    let claims = decode_jwt_claims(&token.access_token)?;
    info!("Token claims: iss={}, sub={}, acr={}, tenant_id={}, audience_type={}, home_cell={}, credential_epoch={}, recovery_epoch={}",
          claims.iss, claims.sub, claims.acr, claims.tenant_id, claims.audience_type,
          claims.home_cell, claims.credential_epoch, claims.recovery_epoch);

    // 9. Step-up for high-risk action (e.g., drive::cmk::rotate).
    let stepup = user_authenticated.session_step_up(SessionStepUp {
        action_being_attempted: "drive::cmk::rotate".into(),
        required_acr: RequiredAcr::Aal3HardwareBound,
    }).await?;
    info!("Step-up completed: id={}, new_acr={:?}, valid_until={}",
          stepup.stepup_id, stepup.new_acr, stepup.valid_until);

    Ok(())
}

async fn simulate_yubikey_attestation(_opts: &impl std::any::Any) -> Result<Attestation> {
    // In production, this is the browser's navigator.credentials.create() result.
    // For the drill, we simulate the wire format using a YubiKey-style attestation.
    todo!("Use a fixture YubiKey 5C attestation for the drill")
}

async fn simulate_yubikey_assertion(_opts: &impl std::any::Any) -> Result<Assertion> {
    todo!("Use a fixture YubiKey assertion for the drill")
}

struct Attestation { client_data_json_b64: String, attestation_object_b64: String }
struct Assertion { client_data_json_b64: String, authenticator_data_b64: String, signature_b64: String }

#[derive(Debug)]
struct JwtClaims {
    iss: String, sub: String, acr: String, tenant_id: String, audience_type: String,
    home_cell: String, credential_epoch: u32, recovery_epoch: u32,
}

fn decode_jwt_claims(token: &str) -> Result<JwtClaims> { todo!() }
```

## Expected output (against a paid with per_seat billing_component-tenant_class cell)

```
INFO User created: u-alice@acme-corp.com (onboarding_token=oyatie-onboard-7c4a2b8e9f...)
INFO Registration challenge issued: id=ch_acme_001
INFO Credential bound: id_hash=blake3:7c4a2b8e9f..., aaguid=ee882879-721c-4913-9775-3dfcce97072a, attestation_class=hardware-backed
INFO Authentication challenge issued: id=ch_auth_acme_001
INFO Session created: id=s_acme_alice_001, acr=Aal3HardwareBound, amr=["hwk","mfa","uv"], credential_epoch=1
INFO OIDC token issued: token=eyJhbGciOiJFZERTQSIsImtpZCI6Imlz..., expires_in=3600
INFO Token claims: iss=https://identity.acme-corp.oyatie.local/oidc/v1, sub=u-alice@acme-corp.com, acr=aal3_hardware_bound, tenant_id=acme-corp, audience_type=workforce, home_cell=prod-us-east-1, credential_epoch=1, recovery_epoch=0
INFO Step-up completed: id=su_acme_001, new_acr=Aal3HardwareBound, valid_until=2026-05-20T15:32:17Z
```

## HTTP alternative (curl)

```sh
# 1. Create user (admin)
curl -X POST https://identity.prod-us-east-1.oyatie.local/v1/identity/users \
    -H "Authorization: Bearer $IDENTITY_ADMIN_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "principal_id":"u-alice@acme-corp.com",
        "email":"alice@acme-corp.com",
        "display_name":"Alice Anderson",
        "audience_type":"workforce",
        "session_class":"workforce_standard"
    }'

# 2. Registration options
curl -X POST https://identity.prod-us-east-1.oyatie.local/v1/identity/webauthn/registration/options \
    -H "Authorization: Bearer $IDENTITY_ADMIN_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "user_id":"u-alice@acme-corp.com",
        "onboarding_token":"oyatie-onboard-7c4a2b8e9f...",
        "required_authenticator_class":"hardware-backed"
    }'

# 3. Registration verify
curl -X POST https://identity.prod-us-east-1.oyatie.local/v1/identity/webauthn/registration/verify \
    -H "Content-Type: application/json" \
    -d '{
        "challenge_id":"ch_acme_001",
        "user_id":"u-alice@acme-corp.com",
        "client_data_json_b64":"...",
        "attestation_object_b64":"..."
    }'

# 4. Authentication options
curl -X POST https://identity.prod-us-east-1.oyatie.local/v1/identity/webauthn/authentication/options \
    -H "Content-Type: application/json" \
    -d '{
        "user_id":"u-alice@acme-corp.com"
    }'

# 5. Authentication verify (gets session)
curl -X POST https://identity.prod-us-east-1.oyatie.local/v1/identity/webauthn/authentication/verify \
    -H "Content-Type: application/json" \
    -d '{
        "challenge_id":"ch_auth_acme_001",
        "client_data_json_b64":"...",
        "authenticator_data_b64":"...",
        "signature_b64":"..."
    }'

# 6. Issue OIDC token
curl -X POST https://identity.prod-us-east-1.oyatie.local/oidc/v1/token \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=session&session_id=s_acme_alice_001&audience=drive-api&scope=drive:files:read+drive:files:write"

# 7. Step-up
curl -X POST https://identity.prod-us-east-1.oyatie.local/v1/identity/sessions/s_acme_alice_001/step-up \
    -H "Authorization: Bearer $USER_JWT" \
    -H "Content-Type: application/json" \
    -d '{
        "action_being_attempted":"drive::cmk::rotate",
        "required_acr":"aal3_hardware_bound"
    }'

# 8. SCIM bulk provision (admin)
curl -X POST https://identity.prod-us-east-1.oyatie.local/scim/v2/Bulk \
    -H "Authorization: Bearer $SCIM_TOKEN" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/scim+json" \
    -d '{
        "schemas":["urn:ietf:params:scim:api:messages:2.0:BulkRequest"],
        "Operations":[
            {"method":"POST","path":"/Users","data":{...user1...}},
            {"method":"POST","path":"/Users","data":{...user2...}}
        ]
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `identity::*` permission |
| `onboarding_token_invalid` | 401 | No | Token expired or used; generate new |
| `webauthn_challenge_expired` | 410 | No | Challenge timed out (60s); restart ceremony |
| `attestation_verification_failed` | 422 | No | Attestation chain invalid |
| `aaguid_not_in_trust_catalog` | 403 | No | Authenticator class not allowed by tenant policy |
| `credential_already_registered` | 409 | No | Credential ID already exists in `exclude_credentials` |
| `recovery_envelope_required` | 422 | No | Tenant requires recovery envelope before high-risk action |
| `passphrase_verifier_mismatch` | 401 | No | Wrong passphrase; check Argon2id derivation |
| `step_up_acr_too_low` | 403 | No | Required ACR exceeds current session; complete additional ceremony |
| `external_idp_federation_failed` | 502 | Yes | Upstream IdP unreachable |
| `scim_rate_limit` | 429 | Yes (auto, backoff) | Tenant SCIM ops/sec exceeded per ADR-identity-003 |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `user_create` | `identity.user.created.v1` |
| `webauthn_registration_verify` | `identity.passkey.registered.v1` |
| `webauthn_authentication_verify` | `identity.authentication.completed.v1` |
| `oidc_token_issue` | `identity.token.issued.v1` |
| `session_step_up` | `identity.session.assurance.changed.v1` |
| `recovery_envelope_create` | `identity.recovery.envelope.rotated.v1` |
| `recovery_initiate` | `identity.recovery.started.v1` |
| `recovery_complete` | `identity.recovery.completed.v1` |
| `aaguid_revoked` | `identity.aaguid.revoked.v1` |
| `scim_bulk_provisioned` | `identity.scim.bulk.completed.v1` |
| `external_idp_token_exchange` | `identity.external_idp.federated.v1` |
| Cedar deny anywhere | `identity.cedar.denied.v1` |

## Where this file lives

`microservices/identity/reference-implementations/webauthn-passkey-flow-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/identity/reference-implementations/webauthn-example/` once `oya-identity-client` ships.
