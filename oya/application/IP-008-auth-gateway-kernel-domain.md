---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-008-auth-gateway-kernel-domain
status: pending
execution_unit: ChangeSet
owner: axis-application + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: auth-gateway kernel + domain

## Intent

Kernel: port traits (IdpClient, SessionStore, MfaVerifier) + entities
(Session, OidcProvider, SamlAssertion, Mfa). Domain: PKCE flow algebra,
nonce binding, SAML XSW-hardened canonicalisation.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-auth-gateway-kernel/{Cargo.toml,src/{lib,entities,ports,errors}.rs}` | create |
| `microservices/application/src/crates/oya-application-auth-gateway-domain/{Cargo.toml,src/{lib,pkce,nonce,saml_canonical}.rs}` | create |
| 2 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
// kernel entities
#[derive(Clone, Debug)]
pub struct Session {
    #[data_class(PII_AUTHN_CREDENTIAL)] pub session_id: [u8; 32],
    #[data_class(PII_IDENTIFYING)] pub user_id: String,
    #[data_class(SENSITIVE_PIPA_ART23)] pub tenant_id: String,
    #[data_class(AUDIT)] pub started_at: chrono::DateTime<chrono::Utc>,
    #[data_class(AUDIT)] pub expires_at: chrono::DateTime<chrono::Utc>,
    #[data_class(INTERNAL_ONLY)] pub mfa_factor: MfaFactor,
    #[data_class(PII_QUASI_IDENTIFIER)] pub ip_address: String,
    #[data_class(PII_QUASI_IDENTIFIER)] pub user_agent: String,
}

// domain
pub fn verify_pkce(verifier: &str, challenge: &str) -> bool {
    let hash = sha256(verifier.as_bytes());
    base64url(hash) == challenge
}

pub fn canonicalise_saml(xml: &[u8]) -> Result<Vec<u8>, SamlError> {
    // exclusive c14n; rejects wrapped signature constructions
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-auth-gateway-kernel --all-features
cargo nextest run -p oya-application-auth-gateway-domain --all-features
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_session_entropy_256bit` | session_id is 256-bit random |
| `test_pkce_verify` | sha256(verifier) == challenge |
| `test_nonce_binding` | nonce embedded in id_token matches |
| `test_saml_xsw_battery_1_through_8` | every XSW variant blocked |
| `test_data_class_complete` | every field annotated |

Coverage: 95 % / 90 %.

## Next IP

[`IP-009-auth-gateway-adapters-oidc-saml.md`](IP-009-auth-gateway-adapters-oidc-saml.md)
