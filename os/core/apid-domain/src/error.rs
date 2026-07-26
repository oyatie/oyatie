//! The crate-local error enum for the apid API surface.

use core::fmt;
use os_kernel::error::Error as CoreError;

/// Errors produced while authorizing, routing or serving an apid API call.
///
/// These map onto the gRPC status codes Talos `apid` returns: `PermissionDenied`
/// for failed mTLS/RBAC checks, `Unimplemented` for unknown methods,
/// `Unavailable` when a backend node can't be reached, and so on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// The caller's role set does not grant access to the method.
    PermissionDenied(String),
    /// The requested method is not implemented by the addressed service.
    Unimplemented(String),
    /// A requested node / backend is not registered with the router.
    NodeNotFound(String),
    /// A backend was reachable but reported the resource as absent.
    NotFound(String),
    /// The request envelope failed validation (empty method, bad node, ...).
    InvalidRequest(String),
    /// A backend node could not be reached / returned a transport failure.
    Unavailable(String),
    /// The request or one of its fan-out legs exceeded its deadline.
    DeadlineExceeded,
    /// Catch-all internal failure.
    Internal(String),
}

impl ApiError {
    /// Construct a [`ApiError::PermissionDenied`].
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        ApiError::PermissionDenied(msg.into())
    }

    /// Construct a [`ApiError::Unimplemented`].
    pub fn unimplemented(msg: impl Into<String>) -> Self {
        ApiError::Unimplemented(msg.into())
    }

    /// Construct a [`ApiError::InvalidRequest`].
    pub fn invalid(msg: impl Into<String>) -> Self {
        ApiError::InvalidRequest(msg.into())
    }

    /// Construct a [`ApiError::Unavailable`].
    pub fn unavailable(msg: impl Into<String>) -> Self {
        ApiError::Unavailable(msg.into())
    }

    /// The gRPC-style status code string this error maps onto.
    pub fn grpc_code(&self) -> &'static str {
        match self {
            ApiError::PermissionDenied(_) => "PermissionDenied",
            ApiError::Unimplemented(_) => "Unimplemented",
            ApiError::NodeNotFound(_) | ApiError::NotFound(_) => "NotFound",
            ApiError::InvalidRequest(_) => "InvalidArgument",
            ApiError::Unavailable(_) => "Unavailable",
            ApiError::DeadlineExceeded => "DeadlineExceeded",
            ApiError::Internal(_) => "Internal",
        }
    }

    /// Whether retrying the same request might succeed (transient failures).
    pub fn is_retryable(&self) -> bool {
        matches!(self, ApiError::Unavailable(_) | ApiError::DeadlineExceeded)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::PermissionDenied(m) => write!(f, "permission denied: {m}"),
            ApiError::Unimplemented(m) => write!(f, "unimplemented: {m}"),
            ApiError::NodeNotFound(m) => write!(f, "node not found: {m}"),
            ApiError::NotFound(m) => write!(f, "not found: {m}"),
            ApiError::InvalidRequest(m) => write!(f, "invalid request: {m}"),
            ApiError::Unavailable(m) => write!(f, "unavailable: {m}"),
            ApiError::DeadlineExceeded => write!(f, "deadline exceeded"),
            ApiError::Internal(m) => write!(f, "internal: {m}"),
        }
    }
}

impl From<ApiError> for CoreError {
    fn from(e: ApiError) -> CoreError {
        match e {
            ApiError::PermissionDenied(m) => CoreError::PermissionDenied(m),
            ApiError::Unimplemented(m) => CoreError::Unsupported(m),
            ApiError::NodeNotFound(m) | ApiError::NotFound(m) => CoreError::NotFound(m),
            ApiError::InvalidRequest(m) => CoreError::Invalid(m),
            ApiError::Unavailable(m) | ApiError::Internal(m) => CoreError::Other(m),
            ApiError::DeadlineExceeded => CoreError::Timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_codes() {
        assert_eq!(
            ApiError::permission_denied("x").grpc_code(),
            "PermissionDenied"
        );
        assert_eq!(ApiError::unavailable("x").grpc_code(), "Unavailable");
        assert_eq!(ApiError::DeadlineExceeded.grpc_code(), "DeadlineExceeded");
        assert_eq!(ApiError::NodeNotFound("n".into()).grpc_code(), "NotFound");
    }

    #[test]
    fn retryable_classification() {
        assert!(ApiError::unavailable("net").is_retryable());
        assert!(ApiError::DeadlineExceeded.is_retryable());
        assert!(!ApiError::permission_denied("x").is_retryable());
    }

    #[test]
    fn converts_into_core_error() {
        let core: CoreError = ApiError::permission_denied("nope").into();
        assert_eq!(core.kind(), "permission_denied");
        let core: CoreError = ApiError::DeadlineExceeded.into();
        assert_eq!(core, CoreError::Timeout);
    }

    #[test]
    fn display_has_message() {
        assert_eq!(
            ApiError::invalid("empty method").to_string(),
            "invalid request: empty method"
        );
    }
}
