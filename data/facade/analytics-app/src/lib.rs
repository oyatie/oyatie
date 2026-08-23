//! Analytics composition root (ADR-0083 layer 12, ADR-0193).
//!
//! Wires the kernel + adapter + domain + usecase + api into a runnable µservice.
//!
//! ## Honest-claims note
//!
//! Status is "planned". The [`AnalyticsApp`] struct holds the wired state;
//! the HTTP listener is deferred (IP-015 follow-up; axum route mounting not yet
//! scaffolded).
//!
//! non_claim: no live HTTP listener, no gRPC server, no production deployment.

// ADR-0083 Tier 3: tests may use unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use shared_olap_client_kernel::TenantId;

/// Boot errors raised before the service starts listening.
#[derive(Debug)]
pub enum BootError {
    /// The configured listen address is invalid.
    InvalidListenAddr(String),
    /// A required configuration value is missing.
    MissingConfig(&'static str),
    /// The tenant ID is syntactically invalid.
    InvalidTenantId(String),
}

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListenAddr(addr) => write!(f, "invalid listen address: {addr}"),
            Self::MissingConfig(key) => write!(f, "missing required config: {key}"),
            Self::InvalidTenantId(detail) => write!(f, "invalid tenant id: {detail}"),
        }
    }
}

impl std::error::Error for BootError {}

/// Runtime configuration for the analytics service.
///
/// data_class: INTERNAL_ONLY
#[derive(Clone, Debug)]
pub struct AnalyticsConfig {
    pub listen_addr: String,
    pub clickhouse_url: String,
    pub clickhouse_user: String,
    /// Sourced from OpenBao at runtime.
    /// data_class: INTERNAL_ONLY (secret)
    pub clickhouse_password: String,
    /// Primary tenant ID for the composition root's serving context.
    pub primary_tenant_id: String,
}

impl AnalyticsConfig {
    /// Validate the config. Returns [`BootError::MissingConfig`] for any empty field.
    ///
    /// # Errors
    /// Returns [`BootError::MissingConfig`] if any required field is empty, or
    /// [`BootError::InvalidTenantId`] if the tenant ID is syntactically invalid.
    pub fn validate(&self) -> Result<TenantId, BootError> {
        if self.listen_addr.is_empty() {
            return Err(BootError::MissingConfig("listen_addr"));
        }
        if self.clickhouse_url.is_empty() {
            return Err(BootError::MissingConfig("clickhouse_url"));
        }
        if self.clickhouse_user.is_empty() {
            return Err(BootError::MissingConfig("clickhouse_user"));
        }
        let tenant_id = TenantId::try_new(&self.primary_tenant_id)
            .map_err(|e| BootError::InvalidTenantId(e.to_string()))?;
        Ok(tenant_id)
    }
}

/// The wired analytics application. Holds the validated config and the
/// validated tenant ID. The OLAP adapter is wired by `main.rs` at startup.
///
/// non_claim: HTTP server / gRPC server mounting is deferred (IP-015).
pub struct AnalyticsApp {
    config: AnalyticsConfig,
    primary_tenant_id: TenantId,
}

impl AnalyticsApp {
    /// Build the app from a validated config.
    ///
    /// # Errors
    /// Returns [`BootError`] if config validation fails.
    pub fn new(config: AnalyticsConfig) -> Result<Self, BootError> {
        let primary_tenant_id = config.validate()?;
        Ok(Self {
            config,
            primary_tenant_id,
        })
    }

    /// Return the configured listen address.
    #[must_use]
    pub fn listen_addr(&self) -> &str {
        &self.config.listen_addr
    }

    /// Return the validated primary tenant ID.
    #[must_use]
    pub fn primary_tenant_id(&self) -> &TenantId {
        &self.primary_tenant_id
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> AnalyticsConfig {
        AnalyticsConfig {
            listen_addr: "127.0.0.1:8080".to_string(),
            clickhouse_url: "http://ch:8123".to_string(),
            clickhouse_user: "default".to_string(),
            clickhouse_password: "pass".to_string(),
            primary_tenant_id: "t1".to_string(),
        }
    }

    #[test]
    fn config_validate_fails_on_empty_listen_addr() {
        let mut cfg = valid_config();
        cfg.listen_addr = "".to_string();
        match cfg.validate().unwrap_err() {
            BootError::MissingConfig(key) => assert_eq!(key, "listen_addr"),
            other => panic!("wrong error: {other}"),
        }
    }

    #[test]
    fn config_validate_fails_on_invalid_tenant_id() {
        let mut cfg = valid_config();
        cfg.primary_tenant_id = "".to_string();
        assert!(matches!(
            cfg.validate().unwrap_err(),
            BootError::InvalidTenantId(_)
        ));
    }

    #[test]
    fn app_builds_successfully() {
        let app = AnalyticsApp::new(valid_config()).unwrap();
        assert_eq!(app.listen_addr(), "127.0.0.1:8080");
        assert_eq!(app.primary_tenant_id().as_str(), "t1");
    }
}
