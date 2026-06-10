//! Lazy backend creation + caching, mirroring apid's `backend.Factory`.
//!
//! Upstream apid does not hold a static list of peer connections: when a call
//! must be proxied to node `X`, it asks a `backend.Factory` for the backend for
//! `X`, which dials a new mTLS connection to that peer's apid (port 50000) the
//! first time and caches it for reuse. Idle connections are reaped.
//!
//! This module models that with a [`BackendFactory`] trait (produces a
//! [`RemoteBackend`] for an endpoint, or fails to "dial") and a
//! [`BackendRegistry`] that caches produced backends and tracks hit/miss/dial
//! statistics — enough to test the lazy-create + reuse behavior the router
//! relies on without any sockets.

use crate::backend::RemoteBackend;
use crate::error::ApiError;
use std::collections::BTreeMap;

/// Creates a [`RemoteBackend`] for a peer endpoint on demand.
///
/// In real apid this dials a new mTLS connection; a failure to connect surfaces
/// as [`ApiError::Unavailable`].
pub trait BackendFactory {
    /// Produce a backend for `endpoint`, or report why it can't be reached.
    fn create(&self, endpoint: &str) -> Result<RemoteBackend, ApiError>;
}

/// A factory that mints reachable peers reporting a fixed version, except for
/// endpoints explicitly marked down (which fail to dial). Used for tests and as
/// a stand-in for the real dialer.
#[derive(Debug, Clone)]
pub struct StaticFactory {
    default_version: String,
    /// Endpoints that should fail to dial entirely (connection refused).
    down: Vec<String>,
    /// Per-endpoint version overrides.
    versions: BTreeMap<String, String>,
}

impl StaticFactory {
    /// A factory whose peers all report `default_version`.
    pub fn new(default_version: impl Into<String>) -> Self {
        StaticFactory {
            default_version: default_version.into(),
            down: Vec::new(),
            versions: BTreeMap::new(),
        }
    }

    /// Mark `endpoint` as failing to dial (connection refused).
    pub fn mark_down(&mut self, endpoint: impl Into<String>) {
        self.down.push(endpoint.into());
    }

    /// Override the version reported by `endpoint`.
    pub fn set_version(&mut self, endpoint: impl Into<String>, version: impl Into<String>) {
        self.versions.insert(endpoint.into(), version.into());
    }
}

impl BackendFactory for StaticFactory {
    fn create(&self, endpoint: &str) -> Result<RemoteBackend, ApiError> {
        if self.down.iter().any(|d| d == endpoint) {
            return Err(ApiError::unavailable(format!(
                "dial {endpoint}: connection refused"
            )));
        }
        let version = self
            .versions
            .get(endpoint)
            .cloned()
            .unwrap_or_else(|| self.default_version.clone());
        Ok(RemoteBackend::new(endpoint, version))
    }
}

/// A connection cache over a [`BackendFactory`].
///
/// Lookups create-on-miss and cache the result; subsequent lookups for the same
/// endpoint reuse the cached backend. Dial failures are *not* cached (so a peer
/// that recovers can be dialed again). Statistics let tests assert the
/// lazy-create + reuse behavior.
#[derive(Debug)]
pub struct BackendRegistry<F: BackendFactory> {
    factory: F,
    cache: BTreeMap<String, RemoteBackend>,
    hits: usize,
    misses: usize,
    dial_failures: usize,
}

impl<F: BackendFactory> BackendRegistry<F> {
    /// A fresh registry over `factory`.
    pub fn new(factory: F) -> Self {
        BackendRegistry {
            factory,
            cache: BTreeMap::new(),
            hits: 0,
            misses: 0,
            dial_failures: 0,
        }
    }

    /// Get (creating + caching on miss) the backend for `endpoint`.
    pub fn get(&mut self, endpoint: &str) -> Result<&RemoteBackend, ApiError> {
        if self.cache.contains_key(endpoint) {
            self.hits += 1;
        } else {
            match self.factory.create(endpoint) {
                Ok(b) => {
                    self.misses += 1;
                    self.cache.insert(endpoint.to_string(), b);
                }
                Err(e) => {
                    self.dial_failures += 1;
                    return Err(e);
                }
            }
        }
        Ok(self.cache.get(endpoint).expect("just inserted"))
    }

    /// Whether a backend for `endpoint` is currently cached.
    pub fn is_cached(&self, endpoint: &str) -> bool {
        self.cache.contains_key(endpoint)
    }

    /// Evict `endpoint` from the cache (e.g. after a transport error), returning
    /// whether it was present. Mirrors apid dropping a broken connection so the
    /// next call re-dials.
    pub fn evict(&mut self, endpoint: &str) -> bool {
        self.cache.remove(endpoint).is_some()
    }

    /// Number of cached connections.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Whether nothing is cached.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Cache hits (reuse of an existing connection).
    pub fn hits(&self) -> usize {
        self.hits
    }

    /// Cache misses (a new connection was dialed).
    pub fn misses(&self) -> usize {
        self.misses
    }

    /// Failed dial attempts (not cached).
    pub fn dial_failures(&self) -> usize {
        self.dial_failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Backend;
    use crate::machine_service::MachineMethod;
    use crate::request::Request;

    #[test]
    fn lazy_create_then_reuse() {
        let mut reg = BackendRegistry::new(StaticFactory::new("v1.7.0"));
        assert!(reg.is_empty());
        // First lookup dials (miss), second reuses (hit).
        assert_eq!(reg.get("10.0.0.2").unwrap().endpoint(), "10.0.0.2");
        assert!(reg.is_cached("10.0.0.2"));
        let _ = reg.get("10.0.0.2").unwrap();
        assert_eq!(reg.misses(), 1);
        assert_eq!(reg.hits(), 1);
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn per_endpoint_version_override() {
        let mut f = StaticFactory::new("v1.7.0");
        f.set_version("10.0.0.9", "v1.5.0");
        let mut reg = BackendRegistry::new(f);
        let be = reg.get("10.0.0.9").unwrap();
        let resp = be.serve(&Request::machine(MachineMethod::Version)).unwrap();
        assert_eq!(resp.body(), "v1.5.0");
    }

    #[test]
    fn dial_failure_is_not_cached() {
        let mut f = StaticFactory::new("v1.7.0");
        f.mark_down("10.0.0.3");
        let mut reg = BackendRegistry::new(f);
        assert_eq!(reg.get("10.0.0.3").unwrap_err().grpc_code(), "Unavailable");
        assert!(!reg.is_cached("10.0.0.3"));
        assert_eq!(reg.dial_failures(), 1);
        // A second attempt re-dials (still fails) rather than serving a cache hit.
        assert!(reg.get("10.0.0.3").is_err());
        assert_eq!(reg.dial_failures(), 2);
        assert_eq!(reg.hits(), 0);
    }

    #[test]
    fn evict_forces_redial() {
        let mut reg = BackendRegistry::new(StaticFactory::new("v1.7.0"));
        reg.get("10.0.0.2").unwrap();
        assert!(reg.evict("10.0.0.2"));
        assert!(!reg.is_cached("10.0.0.2"));
        reg.get("10.0.0.2").unwrap();
        assert_eq!(reg.misses(), 2);
    }

    #[test]
    fn evict_absent_is_false() {
        let mut reg = BackendRegistry::new(StaticFactory::new("v1.7.0"));
        assert!(!reg.evict("nope"));
    }
}
