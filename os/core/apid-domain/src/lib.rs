//! # talos-apid
//!
//! A `no_std` port of the Talos `apid` machine API surface
//! (`internal/app/apid` + the `machine` gRPC API in `siderolabs/talos`).
//!
//! `apid` is the mTLS-terminating front door of a Talos node. It exposes the
//! `machine.MachineService` and the COSI `resource` gRPC APIs, authorizes each
//! call by the RBAC [`Role`](os_kernel::Role) carried in the client
//! certificate, and either serves the request from the local backend or proxies
//! ("fans out") it to one or more peer nodes named in the request metadata.
//!
//! Because real `apid` performs TLS termination, gRPC framing and network I/O,
//! this crate models those boundaries as traits ([`Backend`]) with in-memory
//! implementations so the routing, authorization and request/response logic can
//! be exercised in pure, dependency-free Rust.
//!
//! ## Modules
//! - [`request`] / [`response`]: typed request and response envelopes.
//! - [`machine_service`]: the `machine.MachineService` trait + method enum.
//! - [`resource_service`]: the COSI resource gRPC trait.
//! - [`auth`]: per-method [`Authorizer`] gated on [`Role`](os_kernel::Role).
//! - [`backend`]: the [`Backend`] abstraction, [`LocalBackend`],
//!   [`RemoteBackend`].
//! - [`router`]: the [`Router`] that fans a request to local + remote backends.
//! - [`proxy`]: response aggregation across fanned-out nodes.
//! - [`stream`]: streaming response collection.

pub mod auth;
pub mod backend;
pub mod error;
pub mod identity;
pub mod interceptor;
pub mod machine_service;
pub mod metadata;
pub mod proxy;
pub mod registry;
pub mod request;
pub mod resource_service;
pub mod response;
pub mod router;
pub mod server;
pub mod stream;

pub use auth::Authorizer;
pub use backend::{Backend, LocalBackend, RemoteBackend};
pub use error::ApiError;
pub use identity::{PeerCertificate, PeerIdentity};
pub use interceptor::{AdmittedCall, Interceptor};
pub use machine_service::{MachineMethod, MachineService};
pub use metadata::{Metadata, RoutingMetadata};
pub use proxy::Proxy;
pub use registry::{BackendFactory, BackendRegistry, StaticFactory};
pub use request::{NodeRequest, Request};
pub use resource_service::{ResourceMethod, ResourceService};
pub use response::{NodeResponse, Response, StreamResponse};
pub use router::Router;
pub use server::{
    Backend as ServerBackend, FakeBackend, Server, ServerHandle, ServiceInfo, dispatch,
};
pub use stream::StreamCollector;

use os_kernel::role::RoleSet;

/// The top-level apid server: the wired-together router + authorizer that
/// represents the machine API surface of a single Talos node.
///
/// Mirrors `apid.Server`: it owns the local node identity, the [`Router`] that
/// dispatches to backends, and the [`Authorizer`] that gates every call.
#[derive(Debug)]
pub struct ApidServer {
    /// The endpoint (hostname/IP) of the node this server runs on.
    local_endpoint: String,
    router: Router,
    authorizer: Authorizer,
}

impl ApidServer {
    /// Build a server bound to `local_endpoint` using the default authorizer.
    pub fn new(local_endpoint: impl Into<String>) -> Self {
        ApidServer {
            local_endpoint: local_endpoint.into(),
            router: Router::new(),
            authorizer: Authorizer::default(),
        }
    }

    /// Build a server from explicit parts.
    pub fn with_parts(
        local_endpoint: impl Into<String>,
        router: Router,
        authorizer: Authorizer,
    ) -> Self {
        ApidServer {
            local_endpoint: local_endpoint.into(),
            router,
            authorizer,
        }
    }

    /// The endpoint this node is reachable at.
    pub fn local_endpoint(&self) -> &str {
        &self.local_endpoint
    }

    /// Mutable access to the router (to register backends).
    pub fn router_mut(&mut self) -> &mut Router {
        &mut self.router
    }

    /// Mutable access to the authorizer (to override the policy).
    pub fn authorizer_mut(&mut self) -> &mut Authorizer {
        &mut self.authorizer
    }

    /// Handle a fully-formed [`NodeRequest`]: authorize the caller's roles for
    /// the requested method, then route it. The single integration entry point
    /// that mirrors apid's gRPC interceptor + proxy pipeline.
    pub fn handle(&self, req: &NodeRequest, roles: &RoleSet) -> Result<Response, ApiError> {
        self.authorizer.authorize(req.method(), roles)?;
        self.router.route(req, &self.local_endpoint)
    }

    /// Authorize then route a server-streaming request, returning the collected
    /// per-node messages.
    pub fn handle_stream(
        &self,
        req: &NodeRequest,
        roles: &RoleSet,
    ) -> Result<StreamCollector, ApiError> {
        self.authorizer.authorize(req.method(), roles)?;
        self.router.route_stream(req, &self.local_endpoint)
    }

    /// The full front-door pipeline mirroring apid's interceptor chain: derive
    /// the caller identity from the (optional) client certificate, parse routing
    /// metadata, authorize, then route.
    ///
    /// This is the closest analog to a real gRPC handler entry point — its
    /// inputs (`cert` + `md` + `request`) are exactly what apid receives off the
    /// wire after TLS termination.
    pub fn serve(
        &self,
        cert: Option<&PeerCertificate>,
        md: &Metadata,
        request: Request,
    ) -> Result<Response, ApiError> {
        let interceptor = Interceptor::new(self.authorizer.clone());
        let admitted = interceptor.admit(cert, md, request)?;
        self.router.route(&admitted.request, &self.local_endpoint)
    }

    /// The streaming counterpart of [`serve`](ApidServer::serve).
    pub fn serve_stream(
        &self,
        cert: Option<&PeerCertificate>,
        md: &Metadata,
        request: Request,
    ) -> Result<StreamCollector, ApiError> {
        let interceptor = Interceptor::new(self.authorizer.clone());
        let admitted = interceptor.admit(cert, md, request)?;
        self.router
            .route_stream(&admitted.request, &self.local_endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use backend::{LocalBackend, RemoteBackend};
    use machine_service::MachineMethod;
    use request::Request;
    use os_kernel::role::Role;

    fn server() -> ApidServer {
        let mut srv = ApidServer::new("10.0.0.1");
        srv.router_mut()
            .set_local(LocalBackend::new("10.0.0.1", "v1.7.0", "cp-1"));
        srv.router_mut()
            .add_remote(RemoteBackend::new("10.0.0.2", "v1.6.0"));
        srv
    }

    #[test]
    fn authorized_local_call_succeeds() {
        let srv = server();
        let roles = RoleSet::from_roles([Role::Reader]);
        let req = NodeRequest::local(Request::machine(MachineMethod::Version));
        assert_eq!(srv.handle(&req, &roles).unwrap().body(), "v1.7.0");
    }

    #[test]
    fn unauthorized_mutation_denied_before_routing() {
        let srv = server();
        let roles = RoleSet::from_roles([Role::Reader]);
        let req = NodeRequest::local(Request::machine(MachineMethod::Reboot));
        let err = srv.handle(&req, &roles).unwrap_err();
        assert_eq!(err.grpc_code(), "PermissionDenied");
    }

    #[test]
    fn admin_fanout_aggregates_nodes() {
        let srv = server();
        let roles = RoleSet::from_roles([Role::Admin]);
        let req = NodeRequest::to_nodes(
            Request::machine(MachineMethod::Version),
            ["10.0.0.1".to_string(), "10.0.0.2".to_string()],
        );
        let body = srv.handle(&req, &roles).unwrap().body().to_string();
        assert!(body.contains("10.0.0.1: v1.7.0"));
        assert!(body.contains("10.0.0.2: v1.6.0"));
    }

    #[test]
    fn stream_handle_requires_read() {
        let srv = server();
        let req = NodeRequest::local(Request::machine(MachineMethod::Logs));
        let denied = srv.handle_stream(&req, &RoleSet::new()).unwrap_err();
        assert_eq!(denied.grpc_code(), "PermissionDenied");

        let roles = RoleSet::from_roles([Role::Reader]);
        let c = srv.handle_stream(&req, &roles).unwrap();
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn local_endpoint_is_exposed() {
        let srv = server();
        assert_eq!(srv.local_endpoint(), "10.0.0.1");
    }

    #[test]
    fn serve_pipeline_authorizes_and_routes_local() {
        let srv = server();
        let cert = PeerCertificate::new("admin", ["os:admin"]);
        let resp = srv
            .serve(
                Some(&cert),
                &Metadata::new(),
                Request::machine(MachineMethod::Version),
            )
            .unwrap();
        assert_eq!(resp.body(), "v1.7.0");
    }

    #[test]
    fn serve_pipeline_fans_out_from_nodes_metadata() {
        let srv = server();
        let cert = PeerCertificate::new("admin", ["os:admin"]);
        let md = Metadata::new().with(metadata::HEADER_NODES, "10.0.0.1,10.0.0.2");
        let body = srv
            .serve(Some(&cert), &md, Request::machine(MachineMethod::Version))
            .unwrap()
            .body()
            .to_string();
        assert!(body.contains("10.0.0.1: v1.7.0"));
        assert!(body.contains("10.0.0.2: v1.6.0"));
    }

    #[test]
    fn serve_pipeline_denies_anonymous() {
        let srv = server();
        let err = srv
            .serve(
                None,
                &Metadata::new(),
                Request::machine(MachineMethod::Version),
            )
            .unwrap_err();
        assert_eq!(err.grpc_code(), "PermissionDenied");
    }

    #[test]
    fn serve_stream_pipeline_collects() {
        let srv = server();
        let cert = PeerCertificate::new("ro", ["os:reader"]);
        let c = srv
            .serve_stream(
                Some(&cert),
                &Metadata::new(),
                Request::machine(MachineMethod::Logs),
            )
            .unwrap();
        assert_eq!(c.len(), 2);
    }
}
