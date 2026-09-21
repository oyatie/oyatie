//! Environment-driven service configuration.
//!
//! Twelve-factor (K8s-native) configuration: every knob is an environment
//! variable so the Deployment manifest is the single configuration surface.
//! `from_lookup` keeps the parser a pure function of a key->value map so unit
//! tests never mutate process environment.

use std::fmt;

/// `OYATIE_IDENTITY_REST_ADDR` — REST bind address (default `0.0.0.0:8080`).
pub const ENV_REST_ADDR: &str = "OYATIE_IDENTITY_REST_ADDR";
/// `OYATIE_IDENTITY_GRPC_ADDR` — gRPC bind address (default `0.0.0.0:8081`).
pub const ENV_GRPC_ADDR: &str = "OYATIE_IDENTITY_GRPC_ADDR";
/// `OYATIE_IDENTITY_ISSUER` — expected `iss` for workload tokens (required).
pub const ENV_ISSUER: &str = "OYATIE_IDENTITY_ISSUER";
/// `OYATIE_IDENTITY_AUDIENCE` — expected `aud` for workload tokens (required).
pub const ENV_AUDIENCE: &str = "OYATIE_IDENTITY_AUDIENCE";
/// `OYATIE_IDENTITY_JWKS_PATH` — path to an RFC 7517 JWKS document (required).
pub const ENV_JWKS_PATH: &str = "OYATIE_IDENTITY_JWKS_PATH";
/// `OYATIE_IDENTITY_CEDAR_POLICY_PATH` — path to the Cedar policy set (required).
pub const ENV_CEDAR_POLICY_PATH: &str = "OYATIE_IDENTITY_CEDAR_POLICY_PATH";
/// `OYATIE_IDENTITY_PRINCIPALS_PATH` — optional path to a JSON seed of workload
/// principals for single-node bring-up (the durable store arrives behind the
/// same repository port via the G03 persistence lane).
pub const ENV_PRINCIPALS_PATH: &str = "OYATIE_IDENTITY_PRINCIPALS_PATH";
/// `OYATIE_IDENTITY_SIGNING_KEY_PATH` — optional path to a PKCS#8 (DER) ES256
/// issuer signing key. When set, the OIDC issuer surface (RFC 8414 discovery
/// + JWKS publication) is served; key custody moves behind the G02 KMS port.
pub const ENV_SIGNING_KEY_PATH: &str = "OYATIE_IDENTITY_SIGNING_KEY_PATH";
/// `OYATIE_IDENTITY_SIGNING_KID` — key id for the issuer signing key
/// (default `identity-k1`).
pub const ENV_SIGNING_KID: &str = "OYATIE_IDENTITY_SIGNING_KID";
/// `OYATIE_IDENTITY_LIFECYCLE_BEARER` — REQUIRED bearer credential the mutating
/// principal-lifecycle control plane (`:suspend`/`:retire`) verifies in constant
/// time before any mutation (ADR-0581 / AUTH-005). The binary REFUSES to start
/// without it: there is no unauthenticated mutating control plane. Production
/// custody moves behind the iam credential store / mTLS-SPIFFE adapter.
pub const ENV_LIFECYCLE_BEARER: &str = "OYATIE_IDENTITY_LIFECYCLE_BEARER";
/// `OYATIE_IDENTITY_LIFECYCLE_CALLER_TENANT` — the tenant the verified lifecycle
/// caller acts within (REQUIRED; bound to the credential, never a header).
pub const ENV_LIFECYCLE_CALLER_TENANT: &str = "OYATIE_IDENTITY_LIFECYCLE_CALLER_TENANT";
/// `OYATIE_IDENTITY_LIFECYCLE_CALLER_ID` — a stable identity label for the verified
/// lifecycle caller (default `lifecycle-control-plane`).
pub const ENV_LIFECYCLE_CALLER_ID: &str = "OYATIE_IDENTITY_LIFECYCLE_CALLER_ID";

const DEFAULT_REST_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_GRPC_ADDR: &str = "0.0.0.0:8081";
const DEFAULT_SIGNING_KID: &str = "identity-k1";
const DEFAULT_LIFECYCLE_CALLER_ID: &str = "lifecycle-control-plane";

/// Service configuration resolved from the environment.
#[derive(Clone, Eq, PartialEq)]
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
    /// REQUIRED bearer credential for the mutating lifecycle control plane
    /// (`:suspend`/`:retire`). The binary refuses to start without it
    /// (ADR-0581 / AUTH-005 — no unauthenticated mutating control plane).
    pub lifecycle_bearer: String,
    /// Tenant the verified lifecycle caller acts within (REQUIRED).
    pub lifecycle_caller_tenant: String,
    /// Stable identity label for the verified lifecycle caller.
    pub lifecycle_caller_id: String,
}

/// Redact the lifecycle control-plane credential from logs and panic output.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("rest_addr", &self.rest_addr)
            .field("grpc_addr", &self.grpc_addr)
            .field("issuer", &self.issuer)
            .field("audience", &self.audience)
            .field("jwks_path", &self.jwks_path)
            .field("cedar_policy_path", &self.cedar_policy_path)
            .field("principals_path", &self.principals_path)
            .field("signing_key_path", &self.signing_key_path)
            .field("signing_kid", &self.signing_kid)
            .field("lifecycle_bearer", &"[REDACTED]")
            .field("lifecycle_caller_tenant", &self.lifecycle_caller_tenant)
            .field("lifecycle_caller_id", &self.lifecycle_caller_id)
            .finish()
    }
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
        let required = |key: &'static str| {
            lookup(key)
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .ok_or(ConfigError { missing: key })
        };
        let optional = |key: &'static str| {
            lookup(key)
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        };
        Ok(Self {
            rest_addr: optional(ENV_REST_ADDR).unwrap_or_else(|| DEFAULT_REST_ADDR.into()),
            grpc_addr: optional(ENV_GRPC_ADDR).unwrap_or_else(|| DEFAULT_GRPC_ADDR.into()),
            issuer: required(ENV_ISSUER)?,
            audience: required(ENV_AUDIENCE)?,
            jwks_path: required(ENV_JWKS_PATH)?,
            cedar_policy_path: required(ENV_CEDAR_POLICY_PATH)?,
            principals_path: optional(ENV_PRINCIPALS_PATH),
            signing_key_path: optional(ENV_SIGNING_KEY_PATH),
            signing_kid: optional(ENV_SIGNING_KID).unwrap_or_else(|| DEFAULT_SIGNING_KID.into()),
            lifecycle_bearer: required(ENV_LIFECYCLE_BEARER)?,
            lifecycle_caller_tenant: required(ENV_LIFECYCLE_CALLER_TENANT)?,
            lifecycle_caller_id: optional(ENV_LIFECYCLE_CALLER_ID)
                .unwrap_or_else(|| DEFAULT_LIFECYCLE_CALLER_ID.into()),
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
            ENV_AUDIENCE => Some("cloud-kms".into()),
            ENV_JWKS_PATH => Some("/etc/identity/jwks.json".into()),
            ENV_CEDAR_POLICY_PATH => Some("/etc/identity/policies.cedar".into()),
            ENV_PRINCIPALS_PATH => Some("/etc/identity/principals.json".into()),
            ENV_LIFECYCLE_BEARER => Some("super-secret-lifecycle-bearer".into()),
            ENV_LIFECYCLE_CALLER_TENANT => Some("ten_platform".into()),
            _ => None,
        }
    }

    #[test]
    fn resolves_full_configuration() {
        let config = Config::from_lookup(full_lookup).expect("config");
        assert_eq!(config.rest_addr, "127.0.0.1:9080");
        assert_eq!(config.grpc_addr, "127.0.0.1:9081");
        assert_eq!(config.issuer, "https://idp.oyatie.com");
        assert_eq!(config.audience, "cloud-kms");
        assert_eq!(config.jwks_path, "/etc/identity/jwks.json");
        assert_eq!(config.cedar_policy_path, "/etc/identity/policies.cedar");
        assert_eq!(
            config.principals_path.as_deref(),
            Some("/etc/identity/principals.json")
        );
        assert_eq!(config.lifecycle_bearer, "super-secret-lifecycle-bearer");
        assert_eq!(config.lifecycle_caller_tenant, "ten_platform");
        assert_eq!(config.lifecycle_caller_id, DEFAULT_LIFECYCLE_CALLER_ID);
    }

    #[test]
    fn debug_redacts_lifecycle_bearer() {
        let config = Config::from_lookup(full_lookup).expect("config");
        let debug = format!("{config:?}");

        assert!(!debug.contains("super-secret-lifecycle-bearer"));
        assert!(debug.contains("[REDACTED]"));
        assert!(debug.contains("https://idp.oyatie.com"));
    }

    #[test]
    fn refuses_missing_lifecycle_bearer() {
        // AUTH-005: the mutating lifecycle control plane cannot be served without
        // a verified-caller credential. A missing bearer must fail config load,
        // so the binary refuses to start (no unauthenticated mutating control plane).
        let err = Config::from_lookup(|key| match key {
            ENV_LIFECYCLE_BEARER => None,
            other => full_lookup(other),
        })
        .expect_err("must fail");
        assert!(err.to_string().contains(ENV_LIFECYCLE_BEARER));
    }

    #[test]
    fn refuses_missing_lifecycle_caller_tenant() {
        let err = Config::from_lookup(|key| match key {
            ENV_LIFECYCLE_CALLER_TENANT => None,
            other => full_lookup(other),
        })
        .expect_err("must fail");
        assert!(err.to_string().contains(ENV_LIFECYCLE_CALLER_TENANT));
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

    #[test]
    fn refuses_blank_required_values_and_ignores_blank_optionals() {
        let err = Config::from_lookup(|key| match key {
            ENV_AUDIENCE => Some("   ".into()),
            other => full_lookup(other),
        })
        .expect_err("blank required value must fail");
        assert!(err.to_string().contains(ENV_AUDIENCE));

        let config = Config::from_lookup(|key| match key {
            ENV_REST_ADDR => Some("   ".into()),
            ENV_PRINCIPALS_PATH | ENV_SIGNING_KEY_PATH => Some("   ".into()),
            ENV_SIGNING_KID => Some("   ".into()),
            other => full_lookup(other),
        })
        .expect("config");
        assert_eq!(config.rest_addr, DEFAULT_REST_ADDR);
        assert_eq!(config.principals_path, None);
        assert_eq!(config.signing_key_path, None);
        assert_eq!(config.signing_kid, DEFAULT_SIGNING_KID);
    }
}
