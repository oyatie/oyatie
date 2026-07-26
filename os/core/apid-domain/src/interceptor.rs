//! The apid request-handling pipeline: identity → routing → authorization.
//!
//! Upstream apid wraps every gRPC call in a chain of interceptors before it
//! reaches a backend:
//!
//! 1. the TLS/auth interceptor turns the connection's client certificate into a
//!    [`PeerIdentity`] and rejects unauthenticated calls when required;
//! 2. the proxy/router reads the `nodes`/`proxyfrom` metadata to decide whether
//!    to serve locally or fan out;
//! 3. the RBAC interceptor checks the identity's roles against the method.
//!
//! [`Interceptor`] composes those steps into one [`Interceptor::admit`] call
//! that yields an [`AdmittedCall`] — the authorized, route-resolved request —
//! or an [`ApiError`]. Keeping it separate from [`ApidServer`](crate::ApidServer)
//! lets the decision logic be unit-tested directly off [`Metadata`] +
//! [`PeerCertificate`], the two real inputs apid receives.

use crate::auth::Authorizer;
use crate::error::ApiError;
use crate::identity::{PeerCertificate, PeerIdentity};
use crate::metadata::{HEADER_IMPERSONATE, Metadata, RoutingMetadata};
use crate::request::{NodeRequest, Request};

/// The result of admitting a call: it has been authenticated, authorized and
/// resolved to a concrete [`NodeRequest`] ready for the router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedCall {
    /// The (possibly impersonated) caller identity.
    pub identity: PeerIdentity,
    /// The route-resolved request (local-only or fanned out).
    pub request: NodeRequest,
    /// Whether the call arrived already proxied from an upstream apid.
    pub proxied: bool,
    /// The effective per-call timeout in seconds, if any.
    pub timeout_secs: Option<u64>,
}

/// Composes auth + routing + RBAC into one admission decision.
#[derive(Debug, Clone)]
pub struct Interceptor {
    authorizer: Authorizer,
}

impl Interceptor {
    /// Build an interceptor over `authorizer`.
    pub fn new(authorizer: Authorizer) -> Self {
        Interceptor { authorizer }
    }

    /// An interceptor with the default authorizer policy.
    pub fn with_default_policy() -> Self {
        Interceptor {
            authorizer: Authorizer::default(),
        }
    }

    /// Read-only access to the authorizer.
    pub fn authorizer(&self) -> &Authorizer {
        &self.authorizer
    }

    /// Admit a call.
    ///
    /// `cert` is the verified client certificate (`None` for an unauthenticated
    /// connection). `md` is the incoming gRPC metadata. Steps:
    /// 1. derive the identity from the certificate;
    /// 2. honor an `impersonate-roles` header if the identity may impersonate;
    /// 3. parse routing metadata and build the [`NodeRequest`];
    /// 4. authorize the method against the (effective) roles.
    pub fn admit(
        &self,
        cert: Option<&PeerCertificate>,
        md: &Metadata,
        request: Request,
    ) -> Result<AdmittedCall, ApiError> {
        let base = match cert {
            Some(c) => PeerIdentity::from_certificate(c),
            None => PeerIdentity::anonymous(),
        };

        // Optional impersonation: a trusted caller may present a reduced/elevated
        // role set via the impersonate-roles header.
        let identity = match md.get(HEADER_IMPERSONATE) {
            Some(raw) if !raw.trim().is_empty() => {
                let wanted = os_kernel::role::RoleSet::parse_ous(raw.split(','));
                base.impersonate(&wanted)?
            }
            _ => base,
        };

        let routing = RoutingMetadata::parse(md)?;
        let node_request = if routing.is_fanout() {
            NodeRequest::new(request, routing.nodes.clone())?
        } else {
            NodeRequest::local(request)
        };

        // RBAC: authorize the method against the effective roles.
        self.authorizer
            .authorize(node_request.method(), identity.roles())?;

        Ok(AdmittedCall {
            identity,
            request: node_request,
            proxied: routing.proxied,
            timeout_secs: routing.timeout_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine_service::MachineMethod;
    use crate::metadata::{HEADER_NODES, HEADER_PROXY_FROM};

    fn admin_cert() -> PeerCertificate {
        PeerCertificate::new("admin@cluster", ["os:admin"])
    }

    #[test]
    fn admits_authorized_local_call() {
        let ic = Interceptor::with_default_policy();
        let call = ic
            .admit(
                Some(&admin_cert()),
                &Metadata::new(),
                Request::machine(MachineMethod::Version),
            )
            .unwrap();
        assert!(!call.request.is_fanout());
        assert_eq!(call.identity.name(), "admin@cluster");
        assert!(!call.proxied);
    }

    #[test]
    fn unauthenticated_call_is_denied() {
        let ic = Interceptor::with_default_policy();
        let err = ic
            .admit(
                None,
                &Metadata::new(),
                Request::machine(MachineMethod::Version),
            )
            .unwrap_err();
        assert_eq!(err.grpc_code(), "PermissionDenied");
    }

    #[test]
    fn reader_denied_mutation_at_admission() {
        let ic = Interceptor::with_default_policy();
        let cert = PeerCertificate::new("ro", ["os:reader"]);
        let err = ic
            .admit(
                Some(&cert),
                &Metadata::new(),
                Request::machine(MachineMethod::Reboot),
            )
            .unwrap_err();
        assert_eq!(err.grpc_code(), "PermissionDenied");
    }

    #[test]
    fn nodes_header_produces_fanout() {
        let ic = Interceptor::with_default_policy();
        let md = Metadata::new().with(HEADER_NODES, "10.0.0.2,10.0.0.3");
        let call = ic
            .admit(
                Some(&admin_cert()),
                &md,
                Request::machine(MachineMethod::Version),
            )
            .unwrap();
        assert!(call.request.is_fanout());
        assert_eq!(call.request.nodes(), ["10.0.0.2", "10.0.0.3"]);
    }

    #[test]
    fn proxied_call_is_local_only() {
        let ic = Interceptor::with_default_policy();
        let mut md = Metadata::new();
        md.set(HEADER_NODES, "10.0.0.2");
        md.set(HEADER_PROXY_FROM, "10.0.0.1");
        let call = ic
            .admit(
                Some(&admin_cert()),
                &md,
                Request::machine(MachineMethod::Version),
            )
            .unwrap();
        assert!(call.proxied);
        assert!(!call.request.is_fanout());
    }

    #[test]
    fn impersonation_downgrades_effective_roles() {
        let ic = Interceptor::with_default_policy();
        let cert = PeerCertificate::new("dash", ["os:impersonator"]);
        // Impersonator presents reader roles; a mutating call must be denied.
        let md = Metadata::new().with(HEADER_IMPERSONATE, "os:reader");
        let err = ic
            .admit(Some(&cert), &md, Request::machine(MachineMethod::Reboot))
            .unwrap_err();
        assert_eq!(err.grpc_code(), "PermissionDenied");

        // The same impersonator can read.
        let md = Metadata::new().with(HEADER_IMPERSONATE, "os:reader");
        let ok = ic
            .admit(Some(&cert), &md, Request::machine(MachineMethod::Version))
            .unwrap();
        assert!(ok.identity.roles().can_read());
    }

    #[test]
    fn non_impersonator_rejected_when_presenting_header() {
        let ic = Interceptor::with_default_policy();
        let cert = PeerCertificate::new("ro", ["os:reader"]);
        let md = Metadata::new().with(HEADER_IMPERSONATE, "os:admin");
        let err = ic
            .admit(Some(&cert), &md, Request::machine(MachineMethod::Version))
            .unwrap_err();
        assert_eq!(err.grpc_code(), "PermissionDenied");
    }
}
