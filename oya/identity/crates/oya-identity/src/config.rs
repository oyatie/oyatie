//! Environment-driven service configuration.
//!
//! Twelve-factor (K8s-native) configuration: every knob is an environment
//! variable so the Deployment manifest is the single configuration surface.
//! `from_lookup` keeps the parser a pure function of a key->value map so unit
//! tests never mutate process environment.

use std::fmt;

/// `OYA_IDENTITY_REST_ADDR` — REST bind address (default `0.0.0.0:8080`).
pub const ENV_REST_ADDR: &str = "OYA_IDENTITY_REST_ADDR";
/// `OYA_IDENTITY_GRPC_ADDR` — gRPC bind address (default `0.0.0.0:8081`).
pub const ENV_GRPC_ADDR: &str = "OYA_IDENTITY_GRPC_ADDR";
/// `OYA_IDENTITY_ISSUER` — expected `iss` for workload tokens (required).
pub const ENV_ISSUER: &str = "OYA_IDENTITY_ISSUER";
/// `OYA_IDENTITY_AUDIENCE` — expected `aud` for workload tokens (required).
pub const ENV_AUDIENCE: &str = "OYA_IDENTITY_AUDIENCE";
/// `OYA_IDENTITY_JWKS_PATH` — path to an RFC 7517 JWKS document (required).
pub const ENV_JWKS_PATH: &str = "OYA_IDENTITY_JWKS_PATH";
/// `OYA_IDENTITY_CEDAR_POLICY_PATH` — path to the Cedar policy set (required).
pub const ENV_CEDAR_POLICY_PATH: &str = "OYA_IDENTITY_CEDAR_POLICY_PATH";
/// `OYA_IDENTITY_PRINCIPALS_PATH` — optional path to a JSON seed of workload
/// principals for single-node bring-up (the durable store arrives behind the
/// same repository port via the G03 persistence lane).
pub const ENV_PRINCIPALS_PATH: &str = "OYA_IDENTITY_PRINCIPALS_PATH";
/// `OYA_IDENTITY_SIGNING_KEY_PATH` — optional path to a PKCS#8 (DER) ES256
/// issuer signing key. When set, the OIDC issuer surface (RFC 8414 discovery
/// + JWKS publication) is served; key custody moves behind the G02 KMS port.
pub const ENV_SIGNING_KEY_PATH: &str = "OYA_IDENTITY_SIGNING_KEY_PATH";
/// `OYA_IDENTITY_SIGNING_KID` — key id for the issuer signing key
/// (default `oya-identity-k1`).
pub const ENV_SIGNING_KID: &str = "OYA_IDENTITY_SIGNING_KID";

const DEFAULT_REST_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_GRPC_ADDR: &str = "0.0.0.0:8081";
const DEFAULT_SIGNING_KID: &str = "oya-identity-k1";

/// Service configuration resolved from the environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// REST (axum) bind address.
    pub rest_addr: String,
    /// gRPC (tonic) bind address.
    pub grpc_addr: String,
    /// Expected token issuer (`iss`).
    pub issuer: String,
    /// Expected token audience (`aud`).
    pub audience: String,
    /// Path to the RFC 7517 JWKS document holding token-verification keys.
    pub jwks_path: String,
    /// Path to the Cedar policy set text.
    pub cedar_policy_path: String,
    /// Optional path to the principal seed JSON (bring-up store).
    pub principals_path: Option<String>,
    /// Optional path to the PKCS#8 ES256 issuer signing key (issuer surface
    /// enabled when set).
    pub signing_key_path: Option<String>,
    /// Key id for the issuer signing key.
    pub signing_kid: String,
}

/// A missing required environment variable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigError {
    missing: &'static str,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing required environment variable {}", self.missing)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Resolve the configuration from process environment variables.
    ///
    /// # Errors
    /// Returns [`ConfigError`] when a required variable is unset.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Resolve the configuration through an arbitrary lookup function.
    ///
    /// # Errors
    /// Returns [`ConfigError`] when a required key resolves to `None`.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let required = |key: &'static str| lookup(key).ok_or(ConfigError { missing: key });
        Ok(Self {
            rest_addr: lookup(ENV_REST_ADDR).unwrap_or_else(|| DEFAULT_REST_ADDR.into()),
            grpc_addr: lookup(ENV_GRPC_ADDR).unwrap_or_else(|| DEFAULT_GRPC_ADDR.into()),
            issuer: required(ENV_ISSUER)?,
            audience: required(ENV_AUDIENCE)?,
            jwks_path: required(ENV_JWKS_PATH)?,
            cedar_policy_path: required(ENV_CEDAR_POLICY_PATH)?,
            principals_path: lookup(ENV_PRINCIPALS_PATH),
            signing_key_path: lookup(ENV_SIGNING_KEY_PATH),
            signing_kid: lookup(ENV_SIGNING_KID).unwrap_or_else(|| DEFAULT_SIGNING_KID.into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_lookup(key: &str) -> Option<String> {
        match key {
            ENV_REST_ADDR => Some("127.0.0.1:9080".into()),
            ENV_GRPC_ADDR => Some("127.0.0.1:9081".into()),
            ENV_ISSUER => Some("https://idp.oyatie.com".into()),
            ENV_AUDIENCE => Some("oya-cloud-kms".into()),
            ENV_JWKS_PATH => Some("/etc/oya-identity/jwks.json".into()),
            ENV_CEDAR_POLICY_PATH => Some("/etc/oya-identity/policies.cedar".into()),
            ENV_PRINCIPALS_PATH => Some("/etc/oya-identity/principals.json".into()),
            _ => None,
        }
    }

    #[test]
    fn resolves_full_configuration() {
        let config = Config::from_lookup(full_lookup).expect("config");
        assert_eq!(config.rest_addr, "127.0.0.1:9080");
        assert_eq!(config.grpc_addr, "127.0.0.1:9081");
        assert_eq!(config.issuer, "https://idp.oyatie.com");
        assert_eq!(config.audience, "oya-cloud-kms");
        assert_eq!(config.jwks_path, "/etc/oya-identity/jwks.json");
        assert_eq!(config.cedar_policy_path, "/etc/oya-identity/policies.cedar");
        assert_eq!(
            config.principals_path.as_deref(),
            Some("/etc/oya-identity/principals.json")
        );
    }

    #[test]
    fn binds_default_addresses_when_unset() {
        let config = Config::from_lookup(|key| match key {
            ENV_REST_ADDR | ENV_GRPC_ADDR | ENV_PRINCIPALS_PATH => None,
            other => full_lookup(other),
        })
        .expect("config");
        assert_eq!(config.rest_addr, DEFAULT_REST_ADDR);
        assert_eq!(config.grpc_addr, DEFAULT_GRPC_ADDR);
        assert_eq!(config.principals_path, None);
    }

    #[test]
    fn refuses_missing_issuer() {
        let err = Config::from_lookup(|key| match key {
            ENV_ISSUER => None,
            other => full_lookup(other),
        })
        .expect_err("must fail");
        assert!(err.to_string().contains(ENV_ISSUER));
    }

    #[test]
    fn refuses_missing_jwks_path() {
        let err = Config::from_lookup(|key| match key {
            ENV_JWKS_PATH => None,
            other => full_lookup(other),
        })
        .expect_err("must fail");
        assert!(err.to_string().contains(ENV_JWKS_PATH));
    }
}
