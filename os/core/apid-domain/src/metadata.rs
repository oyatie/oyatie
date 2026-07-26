//! gRPC request metadata parsing, mirroring apid's metadata-driven routing.
//!
//! Talos `apid` does not look at the request payload to decide where a call
//! goes: it inspects the gRPC metadata headers attached by `talosctl` (or by an
//! upstream apid that is itself proxying). The headers that matter are:
//!
//! - `nodes` — a comma-separated list of target endpoints to fan the call out
//!   to. When present, apid proxies one leg per node.
//! - `node` — a single target endpoint (the older, singular form). Treated as a
//!   one-element `nodes` list.
//! - `proxyfrom` — set by apid on the *outgoing* leg so the receiving apid knows
//!   the call was already proxied and must serve it locally (preventing an
//!   infinite proxy loop).
//! - `timeout` — an optional deadline in whole seconds; apid propagates it to
//!   each backend leg.
//!
//! This module parses those headers off a [`Metadata`] map into a structured
//! [`RoutingMetadata`] that the [`Router`](crate::router::Router) and
//! interceptor can act on, exactly the way `internal/app/apid/pkg/provider` and
//! the proxy router do in upstream Talos.

use crate::error::ApiError;
use std::collections::BTreeMap;

/// The metadata header key carrying the fan-out target list.
pub const HEADER_NODES: &str = "nodes";
/// The metadata header key carrying a single target node.
pub const HEADER_NODE: &str = "node";
/// The metadata header apid sets on a proxied leg to stop re-proxying.
pub const HEADER_PROXY_FROM: &str = "proxyfrom";
/// The metadata header carrying a per-call timeout in seconds.
pub const HEADER_TIMEOUT: &str = "timeout";
/// The metadata header a trusted caller uses to present an impersonated role
/// set (comma-separated `os:<role>` strings).
pub const HEADER_IMPERSONATE: &str = "impersonate-roles";

/// A case-insensitive, multi-valued gRPC metadata map.
///
/// gRPC metadata keys are ASCII-case-insensitive and may hold several values
/// per key (e.g. repeated `nodes` headers). This models that faithfully so the
/// parsing matches how Go's `metadata.MD` behaves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    inner: BTreeMap<String, Vec<String>>,
}

impl Metadata {
    /// An empty metadata map.
    pub fn new() -> Self {
        Metadata {
            inner: BTreeMap::new(),
        }
    }

    /// Append a value under `key` (lower-cased), preserving existing values.
    pub fn append(&mut self, key: impl AsRef<str>, value: impl Into<String>) {
        let key = key.as_ref().to_ascii_lowercase();
        self.inner.entry(key).or_default().push(value.into());
    }

    /// Set `key` to exactly `value`, replacing any prior values.
    pub fn set(&mut self, key: impl AsRef<str>, value: impl Into<String>) {
        let key = key.as_ref().to_ascii_lowercase();
        self.inner.insert(key, vec![value.into()]);
    }

    /// Builder-style [`set`](Metadata::set).
    pub fn with(mut self, key: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.set(key, value);
        self
    }

    /// All values for `key` (case-insensitive), or an empty slice.
    pub fn get_all(&self, key: &str) -> &[String] {
        self.inner
            .get(&key.to_ascii_lowercase())
            .map_or(&[][..], Vec::as_slice)
    }

    /// The first value for `key`, if any.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.get_all(key).first().map(String::as_str)
    }

    /// Whether `key` is present.
    pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(&key.to_ascii_lowercase())
    }

    /// Remove `key` entirely, returning whether it was present.
    pub fn remove(&mut self, key: &str) -> bool {
        self.inner.remove(&key.to_ascii_lowercase()).is_some()
    }

    /// Number of distinct keys.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the map has no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// The routing decision parsed off request metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoutingMetadata {
    /// The ordered, de-duplicated target nodes (`nodes` ∪ `node`).
    pub nodes: Vec<String>,
    /// Whether the call was already proxied (the `proxyfrom` header was set),
    /// in which case this apid must serve it locally without re-fanning.
    pub proxied: bool,
    /// An optional per-call timeout in whole seconds.
    pub timeout_secs: Option<u64>,
}

impl RoutingMetadata {
    /// Parse routing info from a [`Metadata`] map.
    ///
    /// Both `nodes` (comma-separated and/or repeated) and the singular `node`
    /// header contribute to the target list; blanks are dropped and duplicates
    /// collapsed while preserving first-seen order. A malformed `timeout`
    /// surfaces as [`ApiError::InvalidRequest`].
    pub fn parse(md: &Metadata) -> Result<Self, ApiError> {
        let mut nodes: Vec<String> = Vec::new();
        let mut push = |raw: &str| {
            for part in raw.split(',') {
                let n = part.trim();
                if !n.is_empty() && !nodes.iter().any(|e| e == n) {
                    nodes.push(n.to_string());
                }
            }
        };
        for v in md.get_all(HEADER_NODES) {
            push(v);
        }
        for v in md.get_all(HEADER_NODE) {
            push(v);
        }

        let proxied = md.contains(HEADER_PROXY_FROM);

        let timeout_secs = match md.get(HEADER_TIMEOUT) {
            None => None,
            Some(raw) => {
                let secs = raw
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| ApiError::invalid(format!("invalid timeout header '{raw}'")))?;
                Some(secs)
            }
        };

        Ok(RoutingMetadata {
            nodes,
            proxied,
            timeout_secs,
        })
    }

    /// Whether this call should be fanned out to remote peers.
    ///
    /// A proxied call (`proxyfrom` set) is never re-fanned even if it still
    /// carries a `nodes` list, mirroring apid's loop-prevention.
    pub fn is_fanout(&self) -> bool {
        !self.proxied && !self.nodes.is_empty()
    }

    /// Produce the outgoing metadata for a proxied leg to `node`: strip the
    /// fan-out headers, stamp `proxyfrom`, and re-attach the timeout. This is
    /// what apid sends on the wire to a downstream peer.
    pub fn outgoing(&self, from_endpoint: &str) -> Metadata {
        let mut md = Metadata::new();
        md.set(HEADER_PROXY_FROM, from_endpoint);
        if let Some(secs) = self.timeout_secs {
            md.set(HEADER_TIMEOUT, secs.to_string());
        }
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_is_case_insensitive_and_multivalued() {
        let mut md = Metadata::new();
        md.append("Nodes", "10.0.0.1");
        md.append("NODES", "10.0.0.2");
        assert_eq!(md.get_all("nodes").len(), 2);
        assert_eq!(md.get("nodes"), Some("10.0.0.1"));
        assert!(md.contains("NoDeS"));
        assert_eq!(md.len(), 1);
    }

    #[test]
    fn set_replaces_existing_values() {
        let mut md = Metadata::new();
        md.append("node", "a");
        md.set("node", "b");
        assert_eq!(md.get_all("node"), ["b"]);
    }

    #[test]
    fn parse_merges_nodes_and_node_headers() {
        let mut md = Metadata::new();
        md.set(HEADER_NODES, "10.0.0.1, 10.0.0.2 ,");
        md.append(HEADER_NODE, "10.0.0.3");
        let r = RoutingMetadata::parse(&md).unwrap();
        assert_eq!(r.nodes, ["10.0.0.1", "10.0.0.2", "10.0.0.3"]);
        assert!(r.is_fanout());
    }

    #[test]
    fn parse_deduplicates_preserving_order() {
        let mut md = Metadata::new();
        md.append(HEADER_NODES, "a,b,a");
        md.append(HEADER_NODE, "b");
        let r = RoutingMetadata::parse(&md).unwrap();
        assert_eq!(r.nodes, ["a", "b"]);
    }

    #[test]
    fn proxied_call_is_not_refanned() {
        let mut md = Metadata::new();
        md.set(HEADER_NODES, "10.0.0.2");
        md.set(HEADER_PROXY_FROM, "10.0.0.1");
        let r = RoutingMetadata::parse(&md).unwrap();
        assert!(r.proxied);
        assert!(!r.is_fanout());
    }

    #[test]
    fn timeout_parsing() {
        let md = Metadata::new().with(HEADER_TIMEOUT, "30");
        assert_eq!(RoutingMetadata::parse(&md).unwrap().timeout_secs, Some(30));

        let bad = Metadata::new().with(HEADER_TIMEOUT, "soon");
        assert_eq!(
            RoutingMetadata::parse(&bad).unwrap_err().grpc_code(),
            "InvalidArgument"
        );
    }

    #[test]
    fn outgoing_strips_fanout_and_stamps_proxyfrom() {
        let mut md = Metadata::new();
        md.set(HEADER_NODES, "10.0.0.2,10.0.0.3");
        md.set(HEADER_TIMEOUT, "15");
        let r = RoutingMetadata::parse(&md).unwrap();
        let out = r.outgoing("10.0.0.1");
        assert_eq!(out.get(HEADER_PROXY_FROM), Some("10.0.0.1"));
        assert_eq!(out.get(HEADER_TIMEOUT), Some("15"));
        assert!(!out.contains(HEADER_NODES));
        // The downstream peer would see this as a proxied, local-only call.
        let downstream = RoutingMetadata::parse(&out).unwrap();
        assert!(downstream.proxied);
        assert!(!downstream.is_fanout());
    }

    #[test]
    fn empty_metadata_is_local_only() {
        let r = RoutingMetadata::parse(&Metadata::new()).unwrap();
        assert!(r.nodes.is_empty());
        assert!(!r.is_fanout());
        assert!(!r.proxied);
    }
}
