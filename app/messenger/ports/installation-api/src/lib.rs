#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{Error, Installation, InstallationSpec};

/// Installation registry. Every handle is bound to one authenticated tenant.
/// Every operation also supplies the room; an installation identity alone is
/// never an access grant.
pub trait InstallationStore: Send + Sync {
    fn get(
        &self,
        room: &str,
        id: &str,
    ) -> impl std::future::Future<Output = Result<Option<Installation>, Error>> + Send;

    fn list(
        &self,
        room: &str,
        after: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Vec<Installation>, Error>> + Send;

    fn install(
        &self,
        spec: &InstallationSpec,
        expected: u64,
        command: &str,
    ) -> impl std::future::Future<Output = Result<Installation, Error>> + Send;

    fn revoke(
        &self,
        room: &str,
        id: &str,
        expected: u64,
        command: &str,
    ) -> impl std::future::Future<Output = Result<Installation, Error>> + Send;
}
