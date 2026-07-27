//! # talos-api-proto
//!
//! Hand-modeled equivalents of the Talos gRPC API protobuf definitions found in
//! `pkg/machinery/api/*`, expressed as plain Rust types with no wire-codec or
//! external protobuf dependency.
//!
//! Each Talos service is modeled as:
//!
//! * request/response **message types** ([`machine::ApplyConfigurationRequest`],
//!   [`cluster::HealthCheckProgress`], [`storage::Disk`], ...),
//! * a **service struct** that enforces RBAC ([`common::RequestContext`]) and
//!   precondition/state-machine logic, and
//! * an **OS-boundary trait** (e.g. [`machine::MachineBackend`],
//!   [`network::NetworkBackend`]) with an in-memory implementation used by the
//!   tests, so no real syscalls/etcd/containerd/netlink are required.
//!
//! The [`common`] module provides the cross-cutting envelope: the gRPC-style
//! [`common::Code`], multi-node [`common::Envelope`], per-node
//! [`common::Metadata`], and the role-aware [`common::RequestContext`] that maps
//! onto Talos's certificate-OU RBAC model from `talos-core`.
//!
//! Covered service surfaces: `machine`, `cluster`, `resource`, `time`,
//! `network`, `storage`, plus the `common`, `inspect`, `security`, and `health`
//! helpers folded into [`common`] and the [`Service`] registry below.

pub mod cluster;
pub mod common;
pub mod machine;
pub mod network;
pub mod resource;
pub mod storage;
pub mod time;
pub mod wire;

pub use common::{ApiError, Code, Data, Envelope, Metadata, NodeMessage, RequestContext};
pub use wire::{
    Cursor, Decode, Encode, MAX_FRAME_LEN, Request, Response, ServiceEntry, VersionReply,
    WireError, WireErrorReply, WireRebootMode, WireResult, read_frame, read_frame_opt,
    read_message, read_message_opt, write_frame, write_message,
};

/// The set of Talos gRPC services exposed by `apid`/`machined`, used for
/// service discovery and the `inspect`/reflection surface.
///
/// Mirrors the registered service names in `pkg/machinery/api/*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Service {
    /// `machine.MachineService`.
    Machine,
    /// `cluster.ClusterService`.
    Cluster,
    /// `resource.ResourceService` (COSI state).
    Resource,
    /// `time.TimeService`.
    Time,
    /// `network.NetworkService`.
    Network,
    /// `storage.StorageService`.
    Storage,
    /// `inspect.InspectService` (controller dependency graph).
    Inspect,
    /// `security.SecurityService` (certificate issuance).
    Security,
}

impl Service {
    /// The fully-qualified gRPC service name.
    pub fn grpc_name(self) -> &'static str {
        match self {
            Service::Machine => "machine.MachineService",
            Service::Cluster => "cluster.ClusterService",
            Service::Resource => "cosi.resource.State",
            Service::Time => "time.TimeService",
            Service::Network => "network.NetworkService",
            Service::Storage => "storage.StorageService",
            Service::Inspect => "inspect.InspectService",
            Service::Security => "security.SecurityService",
        }
    }

    /// The minimum role required to reach any method of this service. machined
    /// surfaces (`machine`, `cluster`, `resource`, ...) require at least the
    /// reader role; `security` (cert issuance) requires admin.
    pub fn minimum_role(self) -> os_kernel::role::Role {
        match self {
            Service::Security => os_kernel::role::Role::Admin,
            _ => os_kernel::role::Role::Reader,
        }
    }

    /// All registered services, in `Ord` order.
    pub fn all() -> &'static [Service] {
        &[
            Service::Machine,
            Service::Cluster,
            Service::Resource,
            Service::Time,
            Service::Network,
            Service::Storage,
            Service::Inspect,
            Service::Security,
        ]
    }

    /// Whether the caller's context may reach this service at all (the coarse
    /// service-level RBAC gate apid applies before dispatching to a method).
    pub fn is_reachable_by(self, ctx: &RequestContext) -> bool {
        ctx.authorize(self.minimum_role()).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::role::{Role, RoleSet};

    #[test]
    fn grpc_names_are_stable() {
        assert_eq!(Service::Machine.grpc_name(), "machine.MachineService");
        assert_eq!(Service::Resource.grpc_name(), "cosi.resource.State");
        assert_eq!(Service::all().len(), 8);
    }

    #[test]
    fn service_level_rbac_gate() {
        let reader = RequestContext::with_roles(RoleSet::from_roles([Role::Reader]));
        assert!(Service::Machine.is_reachable_by(&reader));
        assert!(Service::Time.is_reachable_by(&reader));
        // Security (cert issuance) requires admin.
        assert!(!Service::Security.is_reachable_by(&reader));

        let admin = RequestContext::admin_local();
        assert!(Service::all().iter().all(|s| s.is_reachable_by(&admin)));
    }

    #[test]
    fn minimum_roles() {
        assert_eq!(Service::Security.minimum_role(), Role::Admin);
        assert_eq!(Service::Cluster.minimum_role(), Role::Reader);
    }

    #[test]
    fn reexports_are_wired() {
        // Smoke test that the top-level re-exports resolve.
        let mut env: Envelope<u8> = Envelope::new();
        env.push_ok(1);
        assert_eq!(env.len(), 1);
        assert_eq!(Code::Ok.as_i32(), 0);
        let _ctx: RequestContext = RequestContext::admin_local();
        let _m: Metadata = Metadata::local();
        let _d: Data = Data::local(vec![]);
    }
}
