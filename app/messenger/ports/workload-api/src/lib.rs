#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{Error, InstallationSpec, IntegrationCapability};

/// IAM projection of a validated workload JWT. `service` is IAM's owning
/// capability, not a callback URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkloadIdentity {
    pub tenant: String,
    pub workload: String,
    pub service: String,
    pub state: String,
}

/// IAM workload identity for a scoped service installation. Adapters speak
/// HTTP; this crate does not.
pub trait Workload: Send + Sync {
    /// Authorize `spec.workload` to perform `capability` on `spec.room`.
    fn authorize(
        &self,
        tenant: &str,
        spec: &InstallationSpec,
        capability: IntegrationCapability,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    /// Project a presented workload JWT. Does not authorize an action.
    fn identify(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<WorkloadIdentity, Error>> + Send;
}
