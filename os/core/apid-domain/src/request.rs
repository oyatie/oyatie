//! Typed request envelopes for the apid API surface.
//!
//! A real apid request arrives as a gRPC call whose metadata carries the set of
//! target nodes (`nodes` / `node` headers) the call should be fanned out to, an
//! optional deadline, and a payload. This module models that envelope so the
//! router and proxy can reason about it without any gRPC machinery.

use crate::error::ApiError;
use crate::machine_service::MachineMethod;
use crate::resource_service::ResourceMethod;
use std::collections::BTreeMap;

/// Which gRPC service a [`Request`] targets, together with the concrete method.
///
/// apid multiplexes several services on one port; the [`Authorizer`] and
/// [`Router`] need to know both the service and the method to dispatch.
///
/// [`Authorizer`]: crate::auth::Authorizer
/// [`Router`]: crate::router::Router
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// A `machine.MachineService` method.
    Machine(MachineMethod),
    /// A COSI `resource.State` method.
    Resource(ResourceMethod),
}

impl Method {
    /// The fully-qualified gRPC method name.
    pub fn grpc_name(self) -> &'static str {
        match self {
            Method::Machine(m) => m.grpc_name(),
            Method::Resource(m) => m.grpc_name(),
        }
    }

    /// The short method name without the service prefix.
    pub fn short_name(self) -> &'static str {
        match self {
            Method::Machine(m) => m.short_name(),
            Method::Resource(m) => m.short_name(),
        }
    }

    /// Whether the method mutates node state (requires write RBAC).
    pub fn is_mutating(self) -> bool {
        match self {
            Method::Machine(m) => m.is_mutating(),
            Method::Resource(m) => m.is_mutating(),
        }
    }

    /// Whether the method is server-streaming.
    pub fn is_streaming(self) -> bool {
        match self {
            Method::Machine(m) => m.is_streaming(),
            Method::Resource(m) => m.is_streaming(),
        }
    }

    /// Parse a fully-qualified gRPC path into a [`Method`].
    ///
    /// The service is selected from the path prefix; the machine service is
    /// tried first, then the COSI resource service.
    pub fn parse(path: &str) -> Result<Self, ApiError> {
        if path.contains("MachineService") {
            return MachineMethod::parse(path).map(Method::Machine);
        }
        if path.contains("resource") || path.contains("State") {
            return ResourceMethod::parse(path).map(Method::Resource);
        }
        // Fall back to trying both by short name.
        if let Ok(m) = MachineMethod::parse(path) {
            return Ok(Method::Machine(m));
        }
        ResourceMethod::parse(path).map(Method::Resource)
    }
}

/// The inner request payload + metadata, independent of which nodes it targets.
///
/// Mirrors the common shape of a Talos gRPC request: a method, a free-form
/// string body (the marshalled protobuf in real apid), and string-keyed
/// metadata headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    method: Method,
    body: String,
    metadata: BTreeMap<String, String>,
}

impl Request {
    /// Build a request for `method` with an empty body and no metadata.
    pub fn new(method: Method) -> Self {
        Request {
            method,
            body: String::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Convenience constructor for a machine-service request.
    pub fn machine(method: MachineMethod) -> Self {
        Request::new(Method::Machine(method))
    }

    /// Convenience constructor for a resource-service request.
    pub fn resource(method: ResourceMethod) -> Self {
        Request::new(Method::Resource(method))
    }

    /// Set the request body, returning `self` for chaining.
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Attach a metadata header, returning `self` for chaining.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// The method this request targets.
    pub fn method(&self) -> Method {
        self.method
    }

    /// The request body (marshalled payload in real apid).
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Look up a metadata header value.
    pub fn metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }
}

/// A request together with the list of nodes it should be served by.
///
/// This is the unit the [`Router`](crate::router::Router) operates on. An empty
/// node list means "serve locally only"; a populated list means apid fans the
/// request out to each named peer (and, if the local endpoint appears, serves
/// it locally too).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRequest {
    request: Request,
    nodes: Vec<String>,
}

impl NodeRequest {
    /// A request served by the local node only (no fan-out).
    pub fn local(request: Request) -> Self {
        NodeRequest {
            request,
            nodes: Vec::new(),
        }
    }

    /// A request fanned out to an explicit list of target nodes.
    pub fn to_nodes(request: Request, nodes: impl IntoIterator<Item = String>) -> Self {
        NodeRequest {
            request,
            nodes: nodes.into_iter().collect(),
        }
    }

    /// Validate and build a node request, rejecting blank node entries.
    pub fn new(request: Request, nodes: Vec<String>) -> Result<Self, ApiError> {
        for n in &nodes {
            if n.trim().is_empty() {
                return Err(ApiError::invalid("empty node in target list"));
            }
        }
        Ok(NodeRequest { request, nodes })
    }

    /// The inner request.
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// The method targeted (forwarded from the inner request).
    pub fn method(&self) -> Method {
        self.request.method
    }

    /// The list of target nodes; empty means local-only.
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }

    /// Whether this request fans out to one or more remote peers.
    pub fn is_fanout(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// Whether `endpoint` is one of the targeted nodes (so the local node, if it
    /// matches, must serve a leg locally as well as proxy the rest).
    pub fn targets(&self, endpoint: &str) -> bool {
        self.nodes.iter().any(|n| n == endpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_parse_selects_service() {
        assert_eq!(
            Method::parse("/machine.MachineService/Version").unwrap(),
            Method::Machine(MachineMethod::Version)
        );
        assert_eq!(
            Method::parse("/cosi.resource.State/Get").unwrap(),
            Method::Resource(ResourceMethod::Get)
        );
        assert!(Method::parse("/foo.Bar/Baz").is_err());
    }

    #[test]
    fn method_forwards_flags() {
        let m = Method::Machine(MachineMethod::Reboot);
        assert!(m.is_mutating());
        assert!(!m.is_streaming());
        let r = Method::Resource(ResourceMethod::Watch);
        assert!(!r.is_mutating());
        assert!(r.is_streaming());
    }

    #[test]
    fn request_builder_sets_fields() {
        let req = Request::machine(MachineMethod::ApplyConfiguration)
            .with_body("config: yaml")
            .with_metadata("nodes", "10.0.0.1");
        assert_eq!(
            req.method(),
            Method::Machine(MachineMethod::ApplyConfiguration)
        );
        assert_eq!(req.body(), "config: yaml");
        assert_eq!(req.metadata("nodes"), Some("10.0.0.1"));
        assert_eq!(req.metadata("absent"), None);
    }

    #[test]
    fn node_request_fanout_and_targets() {
        let req = Request::machine(MachineMethod::Version);
        let local = NodeRequest::local(req.clone());
        assert!(!local.is_fanout());

        let fan = NodeRequest::to_nodes(req, ["10.0.0.1".to_string(), "10.0.0.2".to_string()]);
        assert!(fan.is_fanout());
        assert!(fan.targets("10.0.0.1"));
        assert!(!fan.targets("10.0.0.9"));
        assert_eq!(fan.nodes().len(), 2);
    }

    #[test]
    fn node_request_rejects_blank_nodes() {
        let req = Request::machine(MachineMethod::Version);
        let err = NodeRequest::new(req, vec!["ok".to_string(), "  ".to_string()]).unwrap_err();
        assert_eq!(err.grpc_code(), "InvalidArgument");
    }
}
