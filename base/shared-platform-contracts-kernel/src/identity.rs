//! Identity contract family: identity domain, principal, credential, token
//! claims.
//!
//! Precedent: AWS IAM principals and Google Cloud IAM identities (typed
//! principal kinds, explicit lifecycle), SPIFFE for workload identity, and
//! RFC 7519 / RFC 9068 for token-claim shapes. Credentials NEVER carry secret
//! material across this contract — only an opaque reference to public or
//! hashed material (the same rule AWS enforces by returning only access-key
//! ids, never secrets, from describe surfaces).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    ContractViolation, MAX_DISPLAY_NAME_LEN, MAX_ID_LEN, check_opaque_token, check_slug, check_text,
};

/// An identity domain (trust realm) inside which principals are issued.
/// One tenant owns one or more identity domains; the issuer is the OIDC
/// issuer URL that mints tokens for the domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityDomain {
    pub domain_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// OIDC issuer URL for the domain; MUST be `https://` (OIDC Discovery
    /// §3 requires TLS-protected issuers).
    pub issuer: String, // data_class: INTERNAL_ONLY
    pub display_name: String, // data_class: INTERNAL_ONLY
}

impl IdentityDomain {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        check_slug(
            "identity_domain.domain_id",
            &self.domain_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_slug(
            "identity_domain.tenant_id",
            &self.tenant_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_text(
            "identity_domain.display_name",
            &self.display_name,
            MAX_DISPLAY_NAME_LEN,
            &mut out,
        );
        if !self.issuer.starts_with("https://") || self.issuer.len() <= "https://".len() {
            out.push(ContractViolation::InvalidShape {
                field: "identity_domain.issuer",
                detail: "issuer must be a non-empty https:// URL".to_owned(),
            });
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }
}

/// The kind of a principal. Workload identities follow SPIFFE; humans come
/// from an OIDC identity domain; federated externals are brokered through an
/// upstream IdP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    Human,
    Workload,
    FederatedExternal,
}

/// Principal lifecycle. Only `Active` principals are operational; PDP
/// evaluation MUST fail closed for any other state (mirrored by the
/// lifecycle precondition in the identity workload authz adapter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalState {
    Pending,
    Active,
    Suspended,
    Deprovisioned,
}

impl PrincipalState {
    /// Whether a principal in this state may be authorized at all.
    #[must_use]
    pub fn is_operational(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// A principal: the authenticated subject PDP decisions are made about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Principal {
    pub principal_id: String,       // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,          // data_class: TENANT_SCOPED
    pub identity_domain_id: String, // data_class: INTERNAL_ONLY
    pub kind: PrincipalKind,        // data_class: INTERNAL_ONLY
    pub state: PrincipalState,      // data_class: INTERNAL_ONLY
    /// RBAC group memberships (sorted, unique, slug-form ids).
    pub group_ids: Vec<String>, // data_class: TENANT_SCOPED
    /// ABAC attributes (deterministic order; values are opaque strings the
    /// PDP exposes to attribute conditions).
    pub attributes: BTreeMap<String, String>, // data_class: TENANT_SCOPED
}

impl Principal {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        check_slug(
            "principal.principal_id",
            &self.principal_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_slug("principal.tenant_id", &self.tenant_id, MAX_ID_LEN, &mut out);
        check_slug(
            "principal.identity_domain_id",
            &self.identity_domain_id,
            MAX_ID_LEN,
            &mut out,
        );
        for group_id in &self.group_ids {
            check_slug("principal.group_ids", group_id, MAX_ID_LEN, &mut out);
        }
        let mut seen = std::collections::BTreeSet::new();
        for group_id in &self.group_ids {
            if !seen.insert(group_id) {
                out.push(ContractViolation::BrokenReference {
                    field: "principal.group_ids",
                    detail: format!("duplicate group membership {group_id:?}"),
                });
            }
        }
        for key in self.attributes.keys() {
            check_slug("principal.attributes", key, MAX_ID_LEN, &mut out);
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }
}

/// Credential kinds. Every kind is phishing-resistant or reference-only;
/// shared-secret passwords are deliberately absent from the FD-001 contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialKind {
    /// OIDC-federated token credential (human SSO).
    OidcFederatedToken,
    /// WebAuthn passkey (FIDO2 public-key credential).
    WebauthnPasskey,
    /// X.509 SVID (SPIFFE workload identity document).
    X509Svid,
    /// Hashed API token reference (hash stored, never the token).
    ApiTokenHash,
}

/// Credential lifecycle; `Revoked` is terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialState {
    Active,
    Rotated,
    Revoked,
}

/// A credential bound to a principal. Carries only an opaque reference to
/// public or hashed material — secret bytes never cross this contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credential {
    pub credential_id: String,  // data_class: INTERNAL_ONLY
    pub principal_id: String,   // data_class: PII_QUASI_IDENTIFIER
    pub kind: CredentialKind,   // data_class: INTERNAL_ONLY
    pub state: CredentialState, // data_class: INTERNAL_ONLY
    /// Opaque reference to the public/hashed material (e.g. a JWK thumbprint,
    /// passkey credential id, SVID serial, or token hash handle).
    pub public_material_ref: String, // data_class: INTERNAL_ONLY
    pub created_at_unix_s: u64, // data_class: INTERNAL_ONLY
    pub expires_at_unix_s: Option<u64>, // data_class: INTERNAL_ONLY
}

impl Credential {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        check_slug(
            "credential.credential_id",
            &self.credential_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_slug(
            "credential.principal_id",
            &self.principal_id,
            MAX_ID_LEN,
            &mut out,
        );
        check_opaque_token(
            "credential.public_material_ref",
            &self.public_material_ref,
            &mut out,
        );
        if let Some(expires) = self.expires_at_unix_s
            && expires <= self.created_at_unix_s
        {
            out.push(ContractViolation::InvalidTemporalOrder {
                field: "credential.expires_at_unix_s",
                detail: format!(
                    "expiry {expires} must be strictly after creation {}",
                    self.created_at_unix_s
                ),
            });
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }

    /// Whether the credential is usable at `now_unix_s` (active and unexpired).
    #[must_use]
    pub fn is_usable_at(&self, now_unix_s: u64) -> bool {
        self.state == CredentialState::Active
            && self.expires_at_unix_s.is_none_or(|exp| now_unix_s < exp)
    }
}

/// Token claims (RFC 7519 registered claims + the RFC 9068 access-token
/// profile fields the platform requires, plus the tenant binding every
/// FD-001 token MUST carry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenClaims {
    /// Issuer (`iss`) — the identity-domain issuer URL.
    pub iss: String, // data_class: INTERNAL_ONLY
    /// Subject (`sub`) — the principal id.
    pub sub: String, // data_class: PII_QUASI_IDENTIFIER
    /// Audiences (`aud`) — at least one logical service audience.
    pub aud: Vec<String>, // data_class: INTERNAL_ONLY
    /// Expiry (`exp`), seconds since the Unix epoch.
    pub exp_unix_s: u64, // data_class: INTERNAL_ONLY
    /// Issued-at (`iat`), seconds since the Unix epoch.
    pub iat_unix_s: u64, // data_class: INTERNAL_ONLY
    /// Not-before (`nbf`), seconds since the Unix epoch.
    pub nbf_unix_s: Option<u64>, // data_class: INTERNAL_ONLY
    /// Token id (`jti`) — unique, replay-detection key.
    pub jti: String, // data_class: INTERNAL_ONLY
    /// Tenant binding: every FD-001 token is tenant-scoped.
    pub tenant_id: String, // data_class: TENANT_SCOPED
    /// OAuth scopes granted to the token.
    pub scope: Vec<String>, // data_class: INTERNAL_ONLY
    /// Authentication context class reference (step-up class), if asserted.
    pub acr: Option<String>, // data_class: INTERNAL_ONLY
}

impl TokenClaims {
    /// Surface-all invariant check.
    pub fn validate(&self) -> Result<(), Vec<ContractViolation>> {
        let mut out = Vec::new();
        if !self.iss.starts_with("https://") || self.iss.len() <= "https://".len() {
            out.push(ContractViolation::InvalidShape {
                field: "token_claims.iss",
                detail: "issuer must be a non-empty https:// URL".to_owned(),
            });
        }
        check_slug("token_claims.sub", &self.sub, MAX_ID_LEN, &mut out);
        check_opaque_token("token_claims.jti", &self.jti, &mut out);
        check_slug(
            "token_claims.tenant_id",
            &self.tenant_id,
            MAX_ID_LEN,
            &mut out,
        );
        if self.aud.is_empty() {
            out.push(ContractViolation::MissingValue {
                field: "token_claims.aud",
            });
        }
        for audience in &self.aud {
            check_opaque_token("token_claims.aud", audience, &mut out);
        }
        for scope in &self.scope {
            check_opaque_token("token_claims.scope", scope, &mut out);
        }
        if self.exp_unix_s <= self.iat_unix_s {
            out.push(ContractViolation::InvalidTemporalOrder {
                field: "token_claims.exp_unix_s",
                detail: format!(
                    "expiry {} must be strictly after issuance {}",
                    self.exp_unix_s, self.iat_unix_s
                ),
            });
        }
        if let Some(nbf) = self.nbf_unix_s
            && nbf >= self.exp_unix_s
        {
            out.push(ContractViolation::InvalidTemporalOrder {
                field: "token_claims.nbf_unix_s",
                detail: format!("not-before {nbf} must precede expiry {}", self.exp_unix_s),
            });
        }
        if out.is_empty() { Ok(()) } else { Err(out) }
    }

    /// Whether the token is temporally valid at `now_unix_s` (after `nbf`,
    /// strictly before `exp`).
    #[must_use]
    pub fn is_temporally_valid_at(&self, now_unix_s: u64) -> bool {
        self.nbf_unix_s.is_none_or(|nbf| now_unix_s >= nbf) && now_unix_s < self.exp_unix_s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn principal() -> Principal {
        Principal {
            principal_id: "alice".to_owned(),
            tenant_id: "acme".to_owned(),
            identity_domain_id: "acme-workforce".to_owned(),
            kind: PrincipalKind::Human,
            state: PrincipalState::Active,
            group_ids: vec!["tenant-admins".to_owned()],
            attributes: BTreeMap::from([("step_up_class".to_owned(), "a".to_owned())]),
        }
    }

    fn claims() -> TokenClaims {
        TokenClaims {
            iss: "https://id.acme.example".to_owned(),
            sub: "alice".to_owned(),
            aud: vec!["oya-tenancy".to_owned()],
            exp_unix_s: 1_700_003_600,
            iat_unix_s: 1_700_000_000,
            nbf_unix_s: Some(1_700_000_000),
            jti: "01jx5km9w8r2tq".to_owned(),
            tenant_id: "acme".to_owned(),
            scope: vec!["tenancy.read".to_owned()],
            acr: Some("urn:oya:acr:class-a".to_owned()),
        }
    }

    #[test]
    fn valid_principal_passes_and_round_trips() {
        let p = principal();
        p.validate().unwrap();
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Principal>(&json).unwrap(), p);
    }

    #[test]
    fn principal_closed_schema_rejects_unknown_fields() {
        let mut value = serde_json::to_value(principal()).unwrap();
        value["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Principal>(value).is_err());
    }

    #[test]
    fn duplicate_group_membership_is_a_violation() {
        let mut p = principal();
        p.group_ids.push("tenant-admins".to_owned());
        let violations = p.validate().unwrap_err();
        assert!(violations
            .iter()
            .any(|v| matches!(v, ContractViolation::BrokenReference { field, .. } if *field == "principal.group_ids")));
    }

    #[test]
    fn only_active_principal_state_is_operational() {
        assert!(PrincipalState::Active.is_operational());
        for state in [
            PrincipalState::Pending,
            PrincipalState::Suspended,
            PrincipalState::Deprovisioned,
        ] {
            assert!(!state.is_operational(), "{state:?}");
        }
    }

    #[test]
    fn credential_expiry_must_follow_creation() {
        let credential = Credential {
            credential_id: "cred-1".to_owned(),
            principal_id: "alice".to_owned(),
            kind: CredentialKind::WebauthnPasskey,
            state: CredentialState::Active,
            public_material_ref: "jwk-thumb-sha256-9f8e".to_owned(),
            created_at_unix_s: 1_700_000_000,
            expires_at_unix_s: Some(1_700_000_000),
        };
        let violations = credential.validate().unwrap_err();
        assert!(matches!(
            violations.as_slice(),
            [ContractViolation::InvalidTemporalOrder { .. }]
        ));
    }

    #[test]
    fn revoked_or_expired_credential_is_unusable() {
        let mut credential = Credential {
            credential_id: "cred-1".to_owned(),
            principal_id: "alice".to_owned(),
            kind: CredentialKind::X509Svid,
            state: CredentialState::Active,
            public_material_ref: "svid-serial-77".to_owned(),
            created_at_unix_s: 100,
            expires_at_unix_s: Some(200),
        };
        assert!(credential.is_usable_at(150));
        assert!(!credential.is_usable_at(200));
        credential.state = CredentialState::Revoked;
        assert!(!credential.is_usable_at(150));
    }

    #[test]
    fn valid_token_claims_pass_and_round_trip() {
        let c = claims();
        c.validate().unwrap();
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(serde_json::from_str::<TokenClaims>(&json).unwrap(), c);
    }

    #[test]
    fn token_temporal_invariants_fire() {
        let mut c = claims();
        c.exp_unix_s = c.iat_unix_s;
        c.nbf_unix_s = Some(c.iat_unix_s + 10_000);
        let violations = c.validate().unwrap_err();
        assert_eq!(violations.len(), 2, "surface-all: {violations:?}");
    }

    #[test]
    fn token_requires_audience_and_https_issuer() {
        let mut c = claims();
        c.aud.clear();
        c.iss = "http://insecure.example".to_owned();
        let violations = c.validate().unwrap_err();
        assert!(violations.len() >= 2, "{violations:?}");
    }

    #[test]
    fn token_temporal_validity_window() {
        let c = claims();
        assert!(c.is_temporally_valid_at(1_700_000_001));
        assert!(!c.is_temporally_valid_at(1_700_003_600));
        assert!(!c.is_temporally_valid_at(1_699_999_999));
    }
}
