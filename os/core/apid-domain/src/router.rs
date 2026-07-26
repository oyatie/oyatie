//! The fan-out router: apid's proxy core.
//!
//! Once a call is authorized, apid decides where to serve it. With no target
//! nodes it goes straight to the local backend (machined). With a target list it
//! is fanned out: for each node either the local backend serves it (if the node
//! is this endpoint) or the matching [`RemoteBackend`] does. Each leg is folded
//! into a [`Proxy`], so a node that is missing from the registry or unreachable
//! becomes a per-node error rather than aborting the whole call.

use crate::backend::{Backend, LocalBackend, RemoteBackend};
use crate::error::ApiError;
use crate::proxy::Proxy;
use crate::registry::{BackendFactory, BackendRegistry};
use crate::request::NodeRequest;
use crate::response::{NodeResponse, Response};
use crate::stream::StreamCollector;
use std::collections::BTreeMap;

/// Routes authorized requests to local and remote backends and aggregates the
/// results.
#[derive(Debug, Default)]
pub struct Router {
    local: Option<LocalBackend>,
    remotes: BTreeMap<String, RemoteBackend>,
}

impl Router {
    /// An empty router with no backends registered.
    pub fn new() -> Self {
        Router {
            local: None,
            remotes: BTreeMap::new(),
        }
    }

    /// Register (or replace) the local backend.
    pub fn set_local(&mut self, backend: LocalBackend) {
        self.local = Some(backend);
    }

    /// Register (or replace) a remote peer backend, keyed by its endpoint.
    pub fn add_remote(&mut self, backend: RemoteBackend) {
        self.remotes.insert(backend.endpoint().to_string(), backend);
    }

    /// Number of registered remote peers.
    pub fn remote_count(&self) -> usize {
        self.remotes.len()
    }

    /// Whether a local backend is registered.
    pub fn has_local(&self) -> bool {
        self.local.is_some()
    }

    /// Serve one leg for `endpoint`, choosing the local or a remote backend.
    fn serve_leg(&self, endpoint: &str, local_endpoint: &str, req: &NodeRequest) -> NodeResponse {
        let inner = req.request();
        if endpoint == local_endpoint {
            return match &self.local {
                Some(b) => match b.serve(inner) {
                    Ok(r) => NodeResponse::ok(endpoint, r),
                    Err(e) => NodeResponse::failed(endpoint, e),
                },
                None => NodeResponse::failed(
                    endpoint,
                    ApiError::Internal("no local backend registered".into()),
                ),
            };
        }
        match self.remotes.get(endpoint) {
            Some(b) => match b.serve(inner) {
                Ok(r) => NodeResponse::ok(endpoint, r),
                Err(e) => NodeResponse::failed(endpoint, e),
            },
            None => NodeResponse::failed(
                endpoint,
                ApiError::NodeNotFound(format!("node '{endpoint}' is not registered")),
            ),
        }
    }

    /// Route a request and return the aggregated reply.
    ///
    /// - No target nodes: serve locally and return the response verbatim.
    /// - Target nodes: fan out, folding every leg (including failures) into a
    ///   [`Proxy`] and returning its aggregate.
    pub fn route(&self, req: &NodeRequest, local_endpoint: &str) -> Result<Response, ApiError> {
        if !req.is_fanout() {
            let local = self
                .local
                .as_ref()
                .ok_or_else(|| ApiError::Internal("no local backend registered".into()))?;
            return local.serve(req.request());
        }

        let mut proxy = Proxy::new();
        for node in req.nodes() {
            proxy.push(self.serve_leg(node, local_endpoint, req));
        }
        proxy.aggregate()
    }

    /// Route a fan-out request, dialing remote peers lazily through `registry`
    /// instead of requiring them to be pre-registered.
    ///
    /// This mirrors real apid, which holds no static peer list: each target node
    /// is resolved through the backend factory (dialing on first use, reusing a
    /// cached connection afterwards). The local endpoint is still served by the
    /// registered [`LocalBackend`]. A peer that fails to dial becomes a per-node
    /// error rather than aborting the whole call.
    pub fn route_via<F: BackendFactory>(
        &self,
        req: &NodeRequest,
        local_endpoint: &str,
        registry: &mut BackendRegistry<F>,
    ) -> Result<Response, ApiError> {
        if !req.is_fanout() {
            let local = self
                .local
                .as_ref()
                .ok_or_else(|| ApiError::Internal("no local backend registered".into()))?;
            return local.serve(req.request());
        }

        let inner = req.request();
        let mut proxy = Proxy::new();
        for node in req.nodes() {
            if node == local_endpoint {
                let leg = match &self.local {
                    Some(b) => match b.serve(inner) {
                        Ok(r) => NodeResponse::ok(node.clone(), r),
                        Err(e) => NodeResponse::failed(node.clone(), e),
                    },
                    None => NodeResponse::failed(
                        node.clone(),
                        ApiError::Internal("no local backend registered".into()),
                    ),
                };
                proxy.push(leg);
                continue;
            }
            let leg = match registry.get(node) {
                Ok(b) => match b.serve(inner) {
                    Ok(r) => NodeResponse::ok(node.clone(), r),
                    Err(e) => NodeResponse::failed(node.clone(), e),
                },
                Err(e) => NodeResponse::failed(node.clone(), e),
            };
            proxy.push(leg);
        }
        proxy.aggregate()
    }

    /// Route a server-streaming request, collecting every node's messages into a
    /// [`StreamCollector`]. Local-only requests are tagged with an empty node.
    pub fn route_stream(
        &self,
        req: &NodeRequest,
        local_endpoint: &str,
    ) -> Result<StreamCollector, ApiError> {
        let mut collector = StreamCollector::new();
        let inner = req.request();

        if !req.is_fanout() {
            let local = self
                .local
                .as_ref()
                .ok_or_else(|| ApiError::Internal("no local backend registered".into()))?;
            collector.extend("", local.serve_stream(inner)?);
            return Ok(collector);
        }

        for node in req.nodes() {
            if node == local_endpoint {
                match &self.local {
                    Some(b) => match b.serve_stream(inner) {
                        Ok(msgs) => collector.extend(node.clone(), msgs),
                        Err(e) => collector.push_error(node.clone(), e),
                    },
                    None => collector.push_error(
                        node.clone(),
                        ApiError::Internal("no local backend registered".into()),
                    ),
                }
            } else {
                match self.remotes.get(node) {
                    Some(b) => match b.serve_stream(inner) {
                        Ok(msgs) => collector.extend(node.clone(), msgs),
                        Err(e) => collector.push_error(node.clone(), e),
                    },
                    None => collector.push_error(
                        node.clone(),
                        ApiError::NodeNotFound(format!("node '{node}' is not registered")),
                    ),
                }
            }
        }
        Ok(collector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine_service::MachineMethod;
    use crate::request::Request;

    fn router() -> Router {
        let mut r = Router::new();
        r.set_local(LocalBackend::new("10.0.0.1", "v1.7.0", "cp-1"));
        r.add_remote(RemoteBackend::new("10.0.0.2", "v1.6.0"));
        r.add_remote(RemoteBackend::unreachable("10.0.0.3"));
        r
    }

    #[test]
    fn local_only_passes_through() {
        let r = router();
        let req = NodeRequest::local(Request::machine(MachineMethod::Version));
        assert_eq!(r.route(&req, "10.0.0.1").unwrap().body(), "v1.7.0");
    }

    #[test]
    fn fanout_includes_local_and_remote() {
        let r = router();
        let req = NodeRequest::to_nodes(
            Request::machine(MachineMethod::Version),
            ["10.0.0.1".to_string(), "10.0.0.2".to_string()],
        );
        let body = r.route(&req, "10.0.0.1").unwrap().body().to_string();
        assert!(body.contains("10.0.0.1: v1.7.0"));
        assert!(body.contains("10.0.0.2: v1.6.0"));
    }

    #[test]
    fn unreachable_peer_is_per_node_error() {
        let r = router();
        let req = NodeRequest::to_nodes(
            Request::machine(MachineMethod::Version),
            ["10.0.0.2".to_string(), "10.0.0.3".to_string()],
        );
        let body = r.route(&req, "10.0.0.1").unwrap().body().to_string();
        assert!(body.contains("10.0.0.2: v1.6.0"));
        assert!(body.contains("10.0.0.3: unavailable"));
    }

    #[test]
    fn unknown_node_is_node_not_found() {
        let r = router();
        let req = NodeRequest::to_nodes(
            Request::machine(MachineMethod::Version),
            ["10.0.0.99".to_string()],
        );
        let body = r.route(&req, "10.0.0.1").unwrap().body().to_string();
        assert!(body.contains("not registered"));
    }

    #[test]
    fn missing_local_backend_errors() {
        let r = Router::new();
        let req = NodeRequest::local(Request::machine(MachineMethod::Version));
        assert_eq!(
            r.route(&req, "10.0.0.1").unwrap_err().grpc_code(),
            "Internal"
        );
    }

    #[test]
    fn route_via_dials_peers_lazily() {
        use crate::registry::{BackendRegistry, StaticFactory};
        let mut r = Router::new();
        r.set_local(LocalBackend::new("10.0.0.1", "v1.7.0", "cp-1"));
        let mut factory = StaticFactory::new("v1.6.0");
        factory.set_version("10.0.0.2", "v1.6.5");
        let mut reg = BackendRegistry::new(factory);

        let req = NodeRequest::to_nodes(
            Request::machine(MachineMethod::Version),
            ["10.0.0.1".to_string(), "10.0.0.2".to_string()],
        );
        let body = r
            .route_via(&req, "10.0.0.1", &mut reg)
            .unwrap()
            .body()
            .to_string();
        assert!(body.contains("10.0.0.1: v1.7.0"));
        assert!(body.contains("10.0.0.2: v1.6.5"));
        // The peer was dialed once and cached.
        assert!(reg.is_cached("10.0.0.2"));
        assert_eq!(reg.misses(), 1);
    }

    #[test]
    fn route_via_records_dial_failure_per_node() {
        use crate::registry::{BackendRegistry, StaticFactory};
        let mut r = Router::new();
        r.set_local(LocalBackend::new("10.0.0.1", "v1.7.0", "cp-1"));
        let mut factory = StaticFactory::new("v1.6.0");
        factory.mark_down("10.0.0.9");
        let mut reg = BackendRegistry::new(factory);

        let req = NodeRequest::to_nodes(
            Request::machine(MachineMethod::Version),
            ["10.0.0.1".to_string(), "10.0.0.9".to_string()],
        );
        let body = r
            .route_via(&req, "10.0.0.1", &mut reg)
            .unwrap()
            .body()
            .to_string();
        assert!(body.contains("10.0.0.1: v1.7.0"));
        assert!(body.contains("10.0.0.9: unavailable"));
        assert_eq!(reg.dial_failures(), 1);
    }

    #[test]
    fn route_via_local_only_passthrough() {
        use crate::registry::{BackendRegistry, StaticFactory};
        let mut r = Router::new();
        r.set_local(LocalBackend::new("10.0.0.1", "v1.7.0", "cp-1"));
        let mut reg = BackendRegistry::new(StaticFactory::new("v1.6.0"));
        let req = NodeRequest::local(Request::machine(MachineMethod::Version));
        assert_eq!(
            r.route_via(&req, "10.0.0.1", &mut reg).unwrap().body(),
            "v1.7.0"
        );
        assert!(reg.is_empty());
    }

    #[test]
    fn stream_fanout_collects_per_node() {
        let r = router();
        let req = NodeRequest::to_nodes(
            Request::machine(MachineMethod::Logs),
            ["10.0.0.1".to_string(), "10.0.0.3".to_string()],
        );
        let c = r.route_stream(&req, "10.0.0.1").unwrap();
        // Local produced 2 log lines; unreachable peer produced an error.
        assert_eq!(c.len(), 2);
        assert_eq!(c.errors().len(), 1);
        assert_eq!(c.errors()[0].0, "10.0.0.3");
    }
}
