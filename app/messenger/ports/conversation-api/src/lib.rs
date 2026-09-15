#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{Admission, AdmitCommand, AuthorityEvent, AuthoritySync, Error};

/// Durable room admission: one transaction binds the event, retry result and
/// downstream work. Adapters retry serializable conflicts internally.
pub trait RoomAuthority: Send + Sync {
    fn create_room(
        &self,
        creator: &str,
        join_rule: &str,
    ) -> impl std::future::Future<Output = Result<String, Error>> + Send;

    /// A reused Matrix transaction identity returns the original committed event.
    fn admit(
        &self,
        command: AdmitCommand,
    ) -> impl std::future::Future<Output = Result<Admission, Error>> + Send;

    fn sync(
        &self,
        user: &str,
        device: &str,
        since: Option<&str>,
    ) -> impl std::future::Future<Output = Result<AuthoritySync, Error>> + Send;

    fn state(
        &self,
        room: &str,
        event_type: &str,
        state_key: &str,
    ) -> impl std::future::Future<Output = Result<Option<AuthorityEvent>, Error>> + Send;
}
