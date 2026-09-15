#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{Delivery, Error, Installation, InstallationSpec};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct Dispatch {
    pub installation: Installation,
    pub delivery: Delivery,
    pub lease: String,
    pub idempotency_key: String,
}

/// Durable installation generations and dispatch leases. Every handle is bound
/// to one authenticated tenant. Every operation also supplies the room; an
/// installation identity alone is never an access grant.
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

    fn enqueue(
        &self,
        room: &str,
        delivery: &Delivery,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    /// Claim is the dispatch authorization ordering point. It must serialize
    /// with revocation and reject the stale installation checked by the caller.
    fn claim(
        &self,
        expected: &Installation,
    ) -> impl std::future::Future<Output = Result<Option<Dispatch>, Error>> + Send;

    /// An already-started request may finish after revocation. Its receipt can
    /// only acknowledge its unchanged, still-owned lease; it cannot start a retry.
    fn complete(
        &self,
        dispatch: &Dispatch,
        receipt: &Value,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    fn retry(
        &self,
        dispatch: &Dispatch,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}
