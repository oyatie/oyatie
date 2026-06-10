//! [`ConfigStore`]: the network/disk fake that resolves a [`ConfigSource`] to
//! the config bytes it would yield.
//!
//! Upstream this is real I/O: an HTTP `download.Download` against a metadata
//! service, or a mounted filesystem read. There is no network here, so we model
//! the "world" as an in-memory map keyed by the source's stable address (URL,
//! `label:path`, or `cmdline:param=value`).

use crate::source::ConfigSource;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// A fake config backend: resolves a [`ConfigSource`] to bytes, or `None` if the
/// source is absent (the upstream "not found" / `ErrNoConfigSource` case).
pub trait ConfigStore {
    /// Resolve a config source to its bytes, or `None` if unavailable.
    fn fetch(&self, source: &ConfigSource) -> Option<Vec<u8>>;

    /// Resolve a config source with platform context.
    ///
    /// Most stores do not need platform-specific behavior, but real early-boot
    /// stores can use this hook for source-guided flows such as AWS IMDSv2 token
    /// acquisition before fetching user-data.
    fn fetch_for_platform(&self, _platform: &str, source: &ConfigSource) -> Option<Vec<u8>> {
        self.fetch(source)
    }
}

/// Canonical, stable key for a [`ConfigSource`] used by [`MemoryStore`].
///
/// - HTTP: the URL.
/// - Disk: `"<first-label>:<path>"` (all candidate labels map to the same key
///   so a seed under any casing resolves).
/// - Kernel cmdline: `"cmdline:<param>=<value>"`.
fn primary_key(source: &ConfigSource) -> Vec<String> {
    match source {
        ConfigSource::Http { url, .. } => alloc::vec![url.clone()],
        ConfigSource::Disk { labels, path } => labels
            .iter()
            .map(|label| alloc::format!("{label}:{path}"))
            .collect(),
        ConfigSource::KernelCmdline { param, value } => {
            alloc::vec![alloc::format!("cmdline:{param}={value}")]
        }
    }
}

/// In-memory [`ConfigStore`] fake.
///
/// Seed it with the bytes a given source would return, then a [`Platform`] can
/// resolve its config against it deterministically and offline.
///
/// [`Platform`]: crate::Platform
#[derive(Debug, Default, Clone)]
pub struct MemoryStore {
    entries: BTreeMap<String, Vec<u8>>,
}

impl MemoryStore {
    /// A new, empty store (no sources present).
    pub fn new() -> Self {
        MemoryStore {
            entries: BTreeMap::new(),
        }
    }

    /// Seed `bytes` for every key a `source` would resolve under.
    pub fn insert(&mut self, source: &ConfigSource, bytes: impl Into<Vec<u8>>) -> &mut Self {
        let bytes = bytes.into();

        for key in primary_key(source) {
            self.entries.insert(key, bytes.clone());
        }

        self
    }

    /// Builder-style variant of [`insert`](Self::insert).
    pub fn with(mut self, source: &ConfigSource, bytes: impl Into<Vec<u8>>) -> Self {
        self.insert(source, bytes);
        self
    }
}

impl ConfigStore for MemoryStore {
    fn fetch(&self, source: &ConfigSource) -> Option<Vec<u8>> {
        primary_key(source)
            .into_iter()
            .find_map(|key| self.entries.get(&key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{ConfigSource, Header};
    use alloc::vec;

    #[test]
    fn http_round_trip() {
        let src = ConfigSource::http(
            "http://169.254.169.254/latest/user-data",
            vec![Header::new("X-aws-ec2-metadata-token", "tok")],
        );
        let store = MemoryStore::new().with(&src, b"version: v1alpha1".to_vec());
        assert_eq!(store.fetch(&src), Some(b"version: v1alpha1".to_vec()));
    }

    #[test]
    fn disk_any_label_casing_resolves() {
        // Seed under one source; a lookup that lists the same labels resolves.
        let seed = ConfigSource::disk(&["cidata", "CIDATA"], "user-data");
        let store = MemoryStore::new().with(&seed, b"machine: {}".to_vec());

        let lookup = ConfigSource::disk(&["CIDATA"], "user-data");
        assert_eq!(store.fetch(&lookup), Some(b"machine: {}".to_vec()));
    }

    #[test]
    fn missing_source_is_none() {
        let store = MemoryStore::new();
        let src = ConfigSource::http_no_headers("http://nope/");
        assert_eq!(store.fetch(&src), None);
    }

    #[test]
    fn fetch_for_platform_defaults_to_fetch() {
        let src = ConfigSource::http_no_headers("http://169.254.169.254/latest/user-data");
        let store = MemoryStore::new().with(&src, b"machine: aws".to_vec());
        assert_eq!(store.fetch_for_platform("aws", &src), store.fetch(&src));
    }
}
