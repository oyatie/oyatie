//! Authentication seam — fail-closed principal verification (distinct from the
//! D7 Cedar authz PDP).
//!
//! Authn answers "who is the caller, and is their credential cryptographically
//! valid?"; authz ([`crate::AuthzGate`]) answers "is this verified principal
//! allowed to take this action on this resource?". The two are deliberately
//! separate seams: a request must pass authn *first* (produce a
//! [`VerifiedPrincipal`]) and only then is that principal fed to the PDP.
//!
//! The kernel owns only the *port* and its value objects. All verification I/O
//! (JWKS key material, signature math, clock) lives in adapter crates behind
//! [`PrincipalVerifier`]. cloud-iam is the issuing IdP — adapters validate
//! tokens *minted by* cloud-iam, they never mint a parallel identity.
//!
//! Contract: deny by default. Every failure mode is an explicit [`AuthnError`]
//! variant; there is no "verified-but-unknown" state. A caller that receives
//! `Err(_)` MUST reject the request (401), never fall through to authz.

use crate::{AgentId, TenantId};

/// A cryptographically verified caller identity.
///
/// Only constructed by a [`PrincipalVerifier`] after the credential's signature,
/// issuer, audience, and time bounds have all been validated. Holding one is
/// proof the bearer authenticated successfully; it carries the tenant + agent
/// the downstream [`crate::AuthzRequest`] is built from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPrincipal {
    /// Tenant the caller belongs to (from the IdP tenant claim).
    pub tenant: TenantId, // data_class: INTERNAL_ONLY
    /// Per-tenant agent identity (from the token `sub` claim).
    pub agent: AgentId, // data_class: INTERNAL_ONLY
    /// Raw subject claim (`sub`) as presented by the IdP.
    pub subject: String, // data_class: INTERNAL_ONLY
    /// Issuer (`iss`) that minted and signed the token.
    pub issuer: String, // data_class: INTERNAL_ONLY
    /// Token expiry (`exp`) in Unix seconds — already validated as in the future.
    pub expires_at_unix: u64, // data_class: INTERNAL_ONLY
}

/// Fail-closed authentication errors. Every variant is a *deny*; there is no
/// success-shaped error. Variants are intentionally coarse so a caller cannot
/// branch on them in a way that leaks why a token was rejected (return 401 for
/// all of them at the edge).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthnError {
    /// No credential presented.
    MissingToken,
    /// Token is not a well-formed compact JWS (wrong segment count, bad base64,
    /// or undecodable JSON in the header/payload).
    MalformedToken,
    /// The header `alg` is absent, `none`, symmetric (`HS*`), or otherwise not
    /// an asymmetric algorithm this verifier accepts. Defends against
    /// algorithm-confusion downgrade attacks.
    UnsupportedAlgorithm,
    /// No JWKS key matched the token's `kid` (or the `kid` was ambiguous/absent
    /// against a multi-key set).
    UnknownKeyId,
    /// The signature did not verify against the selected key.
    SignatureInvalid,
    /// `exp` is in the past (accounting for allowed clock skew).
    Expired,
    /// `nbf` is in the future (accounting for allowed clock skew).
    NotYetValid,
    /// `iss` did not equal the expected issuer.
    IssuerMismatch,
    /// `aud` did not contain the expected audience.
    AudienceMismatch,
    /// A required claim was absent.
    MissingClaim(&'static str),
    /// A claim was present but malformed (empty, wrong type, fails value-object
    /// validation).
    InvalidClaim(&'static str),
}

impl core::fmt::Display for AuthnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AuthnError::MissingToken => write!(f, "no credential presented"),
            AuthnError::MalformedToken => write!(f, "malformed token"),
            AuthnError::UnsupportedAlgorithm => write!(f, "unsupported or unsafe signing algorithm"),
            AuthnError::UnknownKeyId => write!(f, "no JWKS key matched the token kid"),
            AuthnError::SignatureInvalid => write!(f, "signature verification failed"),
            AuthnError::Expired => write!(f, "token expired"),
            AuthnError::NotYetValid => write!(f, "token not yet valid"),
            AuthnError::IssuerMismatch => write!(f, "issuer mismatch"),
            AuthnError::AudienceMismatch => write!(f, "audience mismatch"),
            AuthnError::MissingClaim(c) => write!(f, "missing required claim: {c}"),
            AuthnError::InvalidClaim(c) => write!(f, "invalid claim: {c}"),
        }
    }
}

impl std::error::Error for AuthnError {}

/// Fail-closed principal-verification port.
///
/// Implementors validate a presented credential and return the
/// [`VerifiedPrincipal`] it proves, or an [`AuthnError`] (deny). The kernel
/// never sees the token bytes or key material — those stay in the adapter.
pub trait PrincipalVerifier {
    /// Verify a bearer credential (a compact JWS for the JWT/OIDC adapter).
    ///
    /// # Errors
    /// Any [`AuthnError`] — all are denials. The caller must reject on `Err`.
    fn verify(&self, token: &str) -> Result<VerifiedPrincipal, AuthnError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stub verifier proving the port is object-safe and usable behind `dyn`,
    /// and that the deny-by-default contract is expressible.
    struct AlwaysDeny;
    impl PrincipalVerifier for AlwaysDeny {
        fn verify(&self, _token: &str) -> Result<VerifiedPrincipal, AuthnError> {
            Err(AuthnError::MissingToken)
        }
    }

    #[test]
    fn port_is_object_safe_and_denies_by_default() {
        let verifier: Box<dyn PrincipalVerifier> = Box::new(AlwaysDeny);
        assert_eq!(verifier.verify("anything"), Err(AuthnError::MissingToken));
    }

    #[test]
    fn verified_principal_feeds_identity_value_objects() {
        let vp = VerifiedPrincipal {
            tenant: TenantId::new("acme").unwrap(),
            agent: AgentId::new("agent-1").unwrap(),
            subject: "agent-1".to_string(),
            issuer: "https://iam.cloud.example/realms/oyatie".to_string(),
            expires_at_unix: 9_999_999_999,
        };
        assert_eq!(vp.tenant.as_str(), "acme");
        assert_eq!(vp.agent.as_str(), "agent-1");
    }

    #[test]
    fn errors_render_without_leaking_token_material() {
        // Display strings are stable, generic, and contain no token bytes.
        assert_eq!(AuthnError::SignatureInvalid.to_string(), "signature verification failed");
        assert_eq!(AuthnError::MissingClaim("exp").to_string(), "missing required claim: exp");
    }
}
