#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    CreateRoom,
    ManageRoom,
    Send,
    Invite,
    Archive,
    ReadObject,
    InvokeAction,
}

/// Identity and resource tenant must be verified by the calling adapter.
/// Errors and absent grants deny. A client-provided allow is never accepted.
pub trait Policy: Send + Sync {
    fn authorize(
        &self,
        tenant: &str,
        subject: &str,
        action: Action,
        resource: &str,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}
