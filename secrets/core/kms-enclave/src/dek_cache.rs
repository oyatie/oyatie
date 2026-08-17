//! Bounded-TTL DEK cache: data-plane static stability (ADR-0536 D-8).
//!
//! AWS KMS precedent: the data plane never makes a per-request KMS call.
//! Unwrapped DEKs are cached with a hard TTL; while the KMS control plane is
//! down, cached DEKs keep serving until their TTL bound, and after the bound
//! the cache FAILS CLOSED — an expired DEK is never served, an unavailable
//! control plane is never silently retried around.
//!
//! Entry count is capped (cardinality cap doctrine); evicted and expired
//! entries scrub their key material via `DekMaterial`'s zeroize-on-drop
//! buffer.

use std::collections::HashMap;
use std::fmt;
use std::num::{NonZeroU64, NonZeroUsize};
use std::time::{SystemTime, UNIX_EPOCH};

use secrets_kms_domain::envelope_keys::{DekId, KekId};

use crate::material::{DekMaterial, KekVersion};

/// Injectable time source so TTL behavior is deterministic under test and
/// alignable with the platform clock substrate (G003 HLC `ClockSource`).
pub trait ClockSource {
    /// Milliseconds since the Unix epoch.
    fn now_epoch_millis(&self) -> u64;
}

/// Production clock backed by `SystemTime`.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClockSource;

impl ClockSource for SystemClockSource {
    fn now_epoch_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }
}

/// Cache key: a DEK is identified by the exact KEK id + version that wrapped
/// it plus its own id — never across versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DekCacheKey {
    /// KEK that wrapped the DEK.
    pub kek_id: KekId,
    /// KEK version that wrapped the DEK.
    pub kek_version: KekVersion,
    /// The DEK's own identifier.
    pub dek_id: DekId,
}

impl fmt::Display for DekCacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.kek_id, self.kek_version, self.dek_id)
    }
}

/// Loader-side error: the KMS control plane could not produce the DEK.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlPlaneUnavailable;

/// Cache-side errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DekCacheError {
    /// No fresh cached DEK and the control plane is unavailable: fail closed.
    ControlPlaneUnavailable {
        /// The key that could not be served.
        key: String,
        /// When the previously cached entry expired, if one existed.
        expired_at_epoch_millis: Option<u64>,
    },
}

impl fmt::Display for DekCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ControlPlaneUnavailable {
                key,
                expired_at_epoch_millis,
            } => match expired_at_epoch_millis {
                Some(at) => write!(
                    f,
                    "dek-cache: '{key}' expired at {at}ms and the control plane is unavailable; failing closed"
                ),
                None => write!(
                    f,
                    "dek-cache: '{key}' not cached and the control plane is unavailable; failing closed"
                ),
            },
        }
    }
}

impl std::error::Error for DekCacheError {}

/// Where a served DEK came from — observability + static-stability evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchSource {
    /// Served from a fresh cached entry; no control-plane call was made.
    Cache,
    /// Fetched from the control plane and (re)cached.
    ControlPlane,
}

struct CacheEntry {
    dek: DekMaterial,
    inserted_at: u64,
    expires_at: u64,
}

/// Bounded-TTL, bounded-cardinality DEK cache.
pub struct BoundedTtlDekCache<C: ClockSource> {
    ttl_millis: NonZeroU64,
    max_entries: NonZeroUsize,
    clock: C,
    entries: HashMap<DekCacheKey, CacheEntry>,
}

impl<C: ClockSource> BoundedTtlDekCache<C> {
    /// Build a cache with a hard TTL (the static-stability window) and a hard
    /// entry cap.
    pub fn new(ttl_millis: NonZeroU64, max_entries: NonZeroUsize, clock: C) -> Self {
        Self {
            ttl_millis,
            max_entries,
            clock,
            entries: HashMap::new(),
        }
    }

    /// Number of currently held entries (fresh or not-yet-collected).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serve a DEK: from cache while fresh, otherwise via `fetch` against the
    /// control plane. A fresh hit never invokes `fetch` (no per-request KMS
    /// call); a stale/missing entry with an unavailable control plane fails
    /// closed and scrubs any stale entry.
    pub fn get_or_fetch<F>(
        &mut self,
        key: &DekCacheKey,
        fetch: F,
    ) -> Result<(&DekMaterial, FetchSource), DekCacheError>
    where
        F: FnOnce() -> Result<DekMaterial, ControlPlaneUnavailable>,
    {
        let now = self.clock.now_epoch_millis();
        let fresh = self.entries.get(key).is_some_and(|e| e.expires_at > now);
        let source = if fresh {
            FetchSource::Cache
        } else {
            match fetch() {
                Ok(dek) => {
                    // Drop (and thereby zeroize) any stale entry, make room,
                    // then cache the refreshed DEK.
                    self.entries.remove(key);
                    self.evict_expired(now);
                    self.evict_to_capacity();
                    let expires_at = now.saturating_add(self.ttl_millis.get());
                    self.entries.insert(
                        key.clone(),
                        CacheEntry {
                            dek,
                            inserted_at: now,
                            expires_at,
                        },
                    );
                    FetchSource::ControlPlane
                }
                Err(ControlPlaneUnavailable) => {
                    let expired_at_epoch_millis =
                        self.entries.remove(key).map(|entry| entry.expires_at);
                    return Err(DekCacheError::ControlPlaneUnavailable {
                        key: key.to_string(),
                        expired_at_epoch_millis,
                    });
                }
            }
        };
        match self.entries.get(key) {
            Some(entry) => Ok((&entry.dek, source)),
            // Unreachable by construction (the entry is fresh or was just
            // inserted above); if it ever happens, fail closed.
            None => Err(DekCacheError::ControlPlaneUnavailable {
                key: key.to_string(),
                expired_at_epoch_millis: None,
            }),
        }
    }

    /// Drop every entry whose TTL has elapsed.
    fn evict_expired(&mut self, now: u64) {
        self.entries.retain(|_, entry| entry.expires_at > now);
    }

    /// Evict oldest-inserted entries until one slot is free.
    fn evict_to_capacity(&mut self) {
        while self.entries.len() >= self.max_entries.get() {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.inserted_at)
                .map(|(key, _)| key.clone());
            match oldest {
                Some(key) => {
                    self.entries.remove(&key);
                }
                None => return,
            }
        }
    }
}

impl<C: ClockSource> fmt::Debug for BoundedTtlDekCache<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BoundedTtlDekCache {{ ttl_millis: {}, max_entries: {}, entries: {}, keys: [REDACTED] }}",
            self.ttl_millis,
            self.max_entries,
            self.entries.len()
        )
    }
}
