#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use messenger_domain::{Error, ObjectRef};
use serde_json::Value;

/// Foundry caller-credential gateway. Present the user's credential; never a
/// bot, room grant, or archive identity.
pub trait Foundry: Send + Sync {
    /// Read a pinned object through the caller's own Foundry credential.
    fn read(
        &self,
        credential: &str,
        object: &ObjectRef,
    ) -> impl std::future::Future<Output = Result<Value, Error>> + Send;

    fn invoke(
        &self,
        credential: &str,
        object: &ObjectRef,
        action: &str,
        idempotency_key: &str,
        occurred_at: u64,
        properties: BTreeMap<String, String>,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}
