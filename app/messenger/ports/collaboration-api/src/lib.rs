#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{ConsoleCommand, ConsoleObjectRef, Error};
use serde_json::Value;

/// Pinned object a credential may read. The body is adapter-owned state, not a
/// Console HTTP envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollaborationObject {
    pub object: ConsoleObjectRef,
    pub body: Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollaborationPreflight {
    pub would_execute: bool,
}

/// Commit identity for one `command_id`. `object.revision` is the post-commit
/// pin. A replay reports `reused` and must not advance that pin again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollaborationReceipt {
    pub command_id: String,
    pub object: ConsoleObjectRef,
    pub reused: bool,
}

/// Business access stays with Console's authenticated writer. A room, shared
/// reference, or preflight result never grants permission to execute.
pub trait ConsoleCollaboration: Send + Sync {
    fn read(
        &self,
        credential: &str,
        object: &ConsoleObjectRef,
    ) -> impl std::future::Future<Output = Result<CollaborationObject, Error>> + Send;

    fn preflight(
        &self,
        credential: &str,
        command: &ConsoleCommand,
    ) -> impl std::future::Future<Output = Result<CollaborationPreflight, Error>> + Send;

    /// Replays go to the same writer with the unchanged command; repeating
    /// preflight can reject an already-consumed approval.
    fn execute(
        &self,
        credential: &str,
        command: &ConsoleCommand,
    ) -> impl std::future::Future<Output = Result<CollaborationReceipt, Error>> + Send;
}
