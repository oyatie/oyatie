#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{Error, InstallationSpec, IntegrationCapability};

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
}
