//! Response aggregation across fanned-out nodes.
//!
//! When apid proxies a request to several peers it collects each leg into a
//! [`NodeResponse`] and folds them into one reply. `talosctl` then renders one
//! block per node, prefixing each with the node's endpoint. A leg that failed is
//! still represented (with its error) rather than dropped, so a single
//! unreachable peer never hides the data returned by the others.

use crate::error::ApiError;
use crate::response::{NodeResponse, Response};

/// Aggregates per-node legs into a single client-facing reply.
#[derive(Debug, Clone, Default)]
pub struct Proxy {
    legs: Vec<NodeResponse>,
}

impl Proxy {
    /// An empty aggregator.
    pub fn new() -> Self {
        Proxy { legs: Vec::new() }
    }

    /// Record one node's leg.
    pub fn push(&mut self, leg: NodeResponse) {
        self.legs.push(leg);
    }

    /// The recorded legs.
    pub fn legs(&self) -> &[NodeResponse] {
        &self.legs
    }

    /// Number of legs.
    pub fn len(&self) -> usize {
        self.legs.len()
    }

    /// Whether no legs were recorded.
    pub fn is_empty(&self) -> bool {
        self.legs.is_empty()
    }

    /// How many legs succeeded.
    pub fn ok_count(&self) -> usize {
        self.legs.iter().filter(|l| l.is_ok()).count()
    }

    /// How many legs failed.
    pub fn error_count(&self) -> usize {
        self.legs.iter().filter(|l| !l.is_ok()).count()
    }

    /// Aggregate the legs into a single [`Response`].
    ///
    /// Each leg is rendered as `"<node>: <body-or-error>"` on its own line, in
    /// the order the legs were pushed. A single local-only leg (one successful
    /// leg whose node tag is empty) is returned verbatim without a prefix, which
    /// matches apid passing an un-fanned request straight through.
    ///
    /// Returns an error only if there are zero legs at all (nothing to serve);
    /// partial failures are folded into the body.
    pub fn aggregate(&self) -> Result<Response, ApiError> {
        if self.legs.is_empty() {
            return Err(ApiError::Internal("no backend legs to aggregate".into()));
        }
        if self.legs.len() == 1 {
            let leg = &self.legs[0];
            if leg.node.is_empty() {
                return match (&leg.response, &leg.error) {
                    (Some(r), _) => Ok(r.clone()),
                    (None, Some(e)) => Err(e.clone()),
                    (None, None) => Ok(Response::empty()),
                };
            }
        }
        let mut body = String::new();
        for (i, leg) in self.legs.iter().enumerate() {
            if i > 0 {
                body.push('\n');
            }
            body.push_str(&leg.node);
            body.push_str(": ");
            match (&leg.response, &leg.error) {
                (Some(r), _) => body.push_str(r.body()),
                (None, Some(e)) => body.push_str(&e.to_string()),
                (None, None) => {}
            }
        }
        Ok(Response::ok(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_local_leg_passes_through() {
        let mut p = Proxy::new();
        p.push(NodeResponse::ok("", Response::ok("v1.7.0")));
        assert_eq!(p.aggregate().unwrap().body(), "v1.7.0");
    }

    #[test]
    fn multi_node_legs_are_prefixed() {
        let mut p = Proxy::new();
        p.push(NodeResponse::ok("10.0.0.1", Response::ok("v1.7.0")));
        p.push(NodeResponse::ok("10.0.0.2", Response::ok("v1.6.0")));
        assert_eq!(
            p.aggregate().unwrap().body(),
            "10.0.0.1: v1.7.0\n10.0.0.2: v1.6.0"
        );
        assert_eq!(p.ok_count(), 2);
    }

    #[test]
    fn partial_failure_is_folded_not_fatal() {
        let mut p = Proxy::new();
        p.push(NodeResponse::ok("10.0.0.1", Response::ok("v1.7.0")));
        p.push(NodeResponse::failed(
            "10.0.0.2",
            ApiError::unavailable("peer down"),
        ));
        let body = p.aggregate().unwrap().body().to_string();
        assert!(body.contains("10.0.0.1: v1.7.0"));
        assert!(body.contains("10.0.0.2: unavailable: peer down"));
        assert_eq!(p.ok_count(), 1);
        assert_eq!(p.error_count(), 1);
    }

    #[test]
    fn empty_proxy_errors() {
        let p = Proxy::new();
        assert!(p.is_empty());
        assert_eq!(p.aggregate().unwrap_err().grpc_code(), "Internal");
    }
}
