#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{ConsoleCommand, ConsoleObjectRef, Error};
use serde_json::Value;

/// Business access stays with Console's authenticated writer. A room or
/// shared reference never grants permission to execute.
pub trait ConsoleCollaboration: Send + Sync {
    fn read(
        &self,
        credential: &str,
        object: &ConsoleObjectRef,
    ) -> impl std::future::Future<Output = Result<Value, Error>> + Send;

    /// Replays use the unchanged command and return the original pin.
    fn execute(
        &self,
        credential: &str,
        command: &ConsoleCommand,
    ) -> impl std::future::Future<Output = Result<(ConsoleObjectRef, bool), Error>> + Send;
}
