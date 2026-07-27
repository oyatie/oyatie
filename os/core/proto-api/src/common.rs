//! Common API types shared across every Talos service.
//!
//! Mirrors `pkg/machinery/api/common/common.proto`: the [`Code`] status enum,
//! [`Metadata`] response headers (set per-node when a request is fanned out
//! through `apid`), [`Error`] payloads, [`Data`] chunks for streaming, and the
//! request/response [`Envelope`] that wraps every multi-node call.

use std::fmt;

use os_kernel::role::{Role, RoleSet};

/// gRPC-style status code, mirroring `common.Code` plus the canonical gRPC
/// codes Talos relies on for error mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    /// The call succeeded.
    Ok,
    /// The caller cancelled the request.
    Cancelled,
    /// An unspecified error occurred.
    Unknown,
    /// A field/argument was invalid.
    InvalidArgument,
    /// The operation exceeded its deadline.
    DeadlineExceeded,
    /// A referenced resource does not exist.
    NotFound,
    /// The resource already exists.
    AlreadyExists,
    /// The caller lacks permission for the operation.
    PermissionDenied,
    /// A precondition for the operation was not satisfied.
    FailedPrecondition,
    /// The feature/operation is not implemented on this node.
    Unimplemented,
    /// An internal invariant was violated.
    Internal,
    /// The service is temporarily unavailable.
    Unavailable,
    /// The caller is not authenticated.
    Unauthenticated,
}

impl Code {
    /// Canonical gRPC numeric value.
    pub fn as_i32(self) -> i32 {
        match self {
            Code::Ok => 0,
            Code::Cancelled => 1,
            Code::Unknown => 2,
            Code::InvalidArgument => 3,
            Code::DeadlineExceeded => 4,
            Code::NotFound => 5,
            Code::AlreadyExists => 6,
            Code::PermissionDenied => 7,
            Code::FailedPrecondition => 9,
            Code::Unimplemented => 12,
            Code::Internal => 13,
            Code::Unavailable => 14,
            Code::Unauthenticated => 16,
        }
    }

    /// Whether the code denotes success.
    pub fn is_ok(self) -> bool {
        matches!(self, Code::Ok)
    }

    /// Map a [`os_kernel::Error`] kind onto a gRPC status code, matching the
    /// way Talos's apid translates internal errors at the wire boundary.
    pub fn from_core_error(err: &os_kernel::Error) -> Code {
        match err {
            os_kernel::Error::Invalid(_) | os_kernel::Error::Parse(_) => Code::InvalidArgument,
            os_kernel::Error::NotFound(_) => Code::NotFound,
            os_kernel::Error::PermissionDenied(_) => Code::PermissionDenied,
            os_kernel::Error::InvalidState(_) => Code::FailedPrecondition,
            os_kernel::Error::Timeout => Code::DeadlineExceeded,
            os_kernel::Error::Unsupported(_) => Code::Unimplemented,
            os_kernel::Error::Other(_) => Code::Unknown,
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Code::Ok => "OK",
            Code::Cancelled => "CANCELLED",
            Code::Unknown => "UNKNOWN",
            Code::InvalidArgument => "INVALID_ARGUMENT",
            Code::DeadlineExceeded => "DEADLINE_EXCEEDED",
            Code::NotFound => "NOT_FOUND",
            Code::AlreadyExists => "ALREADY_EXISTS",
            Code::PermissionDenied => "PERMISSION_DENIED",
            Code::FailedPrecondition => "FAILED_PRECONDITION",
            Code::Unimplemented => "UNIMPLEMENTED",
            Code::Internal => "INTERNAL",
            Code::Unavailable => "UNAVAILABLE",
            Code::Unauthenticated => "UNAUTHENTICATED",
        };
        f.write_str(s)
    }
}

/// A structured API error, mirroring `common.Error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// The status code.
    pub code: Code,
    /// A human-readable message.
    pub message: String,
    /// Optional opaque details (the proto `Any`-typed `details` field).
    pub details: Vec<u8>,
}

impl ApiError {
    /// Construct an error with no details.
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        ApiError {
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }
}

impl From<os_kernel::Error> for ApiError {
    fn from(err: os_kernel::Error) -> Self {
        ApiError::new(Code::from_core_error(&err), err.to_string())
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Per-node response metadata, mirroring `common.Metadata`.
///
/// When a request is fanned out across nodes via `apid`, each per-node reply
/// carries the originating node's address and, on failure, an embedded error so
/// a partial failure of one node does not abort the whole response.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Metadata {
    /// The node (hostname or IP) this reply came from. Empty for the local node.
    pub hostname: String,
    /// Per-node error, if this node failed. `None` means success.
    pub error: Option<ApiError>,
}

impl Metadata {
    /// Metadata for a successful local reply.
    pub fn local() -> Self {
        Metadata::default()
    }

    /// Metadata tagged with an originating node.
    pub fn for_node(hostname: impl Into<String>) -> Self {
        Metadata {
            hostname: hostname.into(),
            error: None,
        }
    }

    /// Attach an error to this node's metadata.
    pub fn with_error(mut self, err: ApiError) -> Self {
        self.error = Some(err);
        self
    }

    /// Whether this node's reply represents an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// A streaming data chunk, mirroring `common.Data` (used by logs/copy/dmesg).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data {
    /// Per-node metadata for this chunk.
    pub metadata: Metadata,
    /// The raw bytes of this chunk.
    pub bytes: Vec<u8>,
}

impl Data {
    /// A local data chunk from a byte slice.
    pub fn local(bytes: impl Into<Vec<u8>>) -> Self {
        Data {
            metadata: Metadata::local(),
            bytes: bytes.into(),
        }
    }
}

/// The authenticated request context derived from the client certificate /
/// gRPC metadata: which node(s) the call targets and the caller's roles.
///
/// Mirrors the `nodes` / `node` gRPC metadata headers plus the RBAC role set
/// extracted from the client certificate's organizational units.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestContext {
    /// Explicit target nodes (the `nodes` header). Empty means "this node".
    pub nodes: Vec<String>,
    /// The proxy/endpoint node the request entered through (the `node` header).
    pub endpoint: Option<String>,
    /// Roles carried by the authenticated identity.
    pub roles: RoleSet,
}

impl RequestContext {
    /// A context for an admin caller targeting the local node.
    pub fn admin_local() -> Self {
        RequestContext {
            nodes: Vec::new(),
            endpoint: None,
            roles: RoleSet::from_roles([Role::Admin]),
        }
    }

    /// A context with the given roles targeting the local node.
    pub fn with_roles(roles: RoleSet) -> Self {
        RequestContext {
            nodes: Vec::new(),
            endpoint: None,
            roles,
        }
    }

    /// Whether this call is fanned out to remote nodes.
    pub fn is_fanout(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// Authorize the call against a required role, returning a
    /// `PermissionDenied` error if the caller's role set does not satisfy it.
    ///
    /// Authorization is by *implication* (matching Talos RBAC), not by exact
    /// membership: a write-capable identity (`admin`/`os`) satisfies a `reader`
    /// requirement, an `operator` satisfies a `reader` requirement, and so on.
    pub fn authorize(&self, required: Role) -> Result<(), ApiError> {
        let granted = match required {
            // read-only capability is satisfied by any read-capable role.
            Role::Reader => self.roles.can_read(),
            // write capability (admin/os) requirement.
            Role::Admin | Role::Os => self.roles.can_write(),
            // operator management APIs: operator itself or any write-capable role.
            Role::Operator => self.roles.includes(Role::Operator) || self.roles.can_write(),
            // etcd snapshot/backup capability.
            Role::EtcdBackup => self.roles.can_etcd_backup(),
            // impersonation is an exact, privileged capability (or admin/os).
            Role::Impersonator => self.roles.can_impersonate(),
            // exact capabilities, also implied by admin/os.
            Role::ImageVerifier => {
                self.roles.includes(Role::ImageVerifier) || self.roles.can_write()
            }
            Role::MetaWriter => self.roles.includes(Role::MetaWriter) || self.roles.can_write(),
        };
        if granted {
            Ok(())
        } else {
            Err(ApiError::new(
                Code::PermissionDenied,
                format!(
                    "caller roles [{}] do not satisfy required role '{}'",
                    self.roles.to_string_list(),
                    required
                ),
            ))
        }
    }
}

/// A generic multi-node response envelope.
///
/// Every Talos API method returns a list of per-node messages; this envelope
/// models that, exposing helpers to collect successes and surface partial
/// failures the way `talosctl` does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope<T> {
    /// One entry per responding node.
    pub messages: Vec<NodeMessage<T>>,
}

/// A single node's contribution to an [`Envelope`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeMessage<T> {
    /// The originating-node metadata (including any per-node error).
    pub metadata: Metadata,
    /// The payload, present when the node succeeded.
    pub payload: Option<T>,
}

impl<T> Envelope<T> {
    /// An empty envelope.
    pub fn new() -> Self {
        Envelope {
            messages: Vec::new(),
        }
    }

    /// Push a successful local payload.
    pub fn push_ok(&mut self, payload: T) {
        self.messages.push(NodeMessage {
            metadata: Metadata::local(),
            payload: Some(payload),
        });
    }

    /// Push a successful payload tagged with a node hostname.
    pub fn push_node(&mut self, hostname: impl Into<String>, payload: T) {
        self.messages.push(NodeMessage {
            metadata: Metadata::for_node(hostname),
            payload: Some(payload),
        });
    }

    /// Push a per-node error.
    pub fn push_error(&mut self, hostname: impl Into<String>, err: ApiError) {
        self.messages.push(NodeMessage {
            metadata: Metadata::for_node(hostname).with_error(err),
            payload: None,
        });
    }

    /// Number of node responses.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Whether there are no responses.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Whether any node reported an error.
    pub fn has_errors(&self) -> bool {
        self.messages.iter().any(|m| m.metadata.is_error())
    }

    /// Iterate over the successful payloads.
    pub fn ok_payloads(&self) -> impl Iterator<Item = &T> {
        self.messages.iter().filter_map(|m| m.payload.as_ref())
    }

    /// Collect all per-node errors.
    pub fn errors(&self) -> Vec<&ApiError> {
        self.messages
            .iter()
            .filter_map(|m| m.metadata.error.as_ref())
            .collect()
    }
}

impl<T> Default for Envelope<T> {
    fn default() -> Self {
        Envelope::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_grpc_values_and_mapping() {
        assert_eq!(Code::Ok.as_i32(), 0);
        assert_eq!(Code::NotFound.as_i32(), 5);
        assert_eq!(Code::Unauthenticated.as_i32(), 16);
        assert!(Code::Ok.is_ok());
        assert!(!Code::Internal.is_ok());

        assert_eq!(
            Code::from_core_error(&os_kernel::Error::not_found("x")),
            Code::NotFound
        );
        assert_eq!(
            Code::from_core_error(&os_kernel::Error::permission_denied("x")),
            Code::PermissionDenied
        );
        assert_eq!(
            Code::from_core_error(&os_kernel::Error::Timeout),
            Code::DeadlineExceeded
        );
        assert_eq!(
            Code::from_core_error(&os_kernel::Error::invalid_state("x")),
            Code::FailedPrecondition
        );
    }

    #[test]
    fn api_error_from_core() {
        let e: ApiError = os_kernel::Error::not_found("missing").into();
        assert_eq!(e.code, Code::NotFound);
        assert!(e.message.contains("missing"));
        assert_eq!(e.to_string(), "NOT_FOUND: not found: missing");
    }

    #[test]
    fn request_context_authorize() {
        let ctx = RequestContext::admin_local();
        assert!(ctx.authorize(Role::Reader).is_ok());
        assert!(ctx.authorize(Role::Admin).is_ok());

        let reader = RequestContext::with_roles(RoleSet::from_roles([Role::Reader]));
        assert!(reader.authorize(Role::Reader).is_ok());
        let denied = reader.authorize(Role::Admin).unwrap_err();
        assert_eq!(denied.code, Code::PermissionDenied);
    }

    #[test]
    fn envelope_partial_failure() {
        let mut env: Envelope<u32> = Envelope::new();
        env.push_node("node-a", 1);
        env.push_error("node-b", ApiError::new(Code::Unavailable, "down"));
        env.push_ok(2);

        assert_eq!(env.len(), 3);
        assert!(env.has_errors());
        let oks: Vec<u32> = env.ok_payloads().copied().collect();
        assert_eq!(oks, vec![1, 2]);
        assert_eq!(env.errors().len(), 1);
        assert_eq!(env.errors()[0].code, Code::Unavailable);
    }

    #[test]
    fn data_chunk_local() {
        let d = Data::local(b"hello".to_vec());
        assert_eq!(d.bytes, b"hello");
        assert!(!d.metadata.is_error());
    }
}
