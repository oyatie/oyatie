#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{AuthorityEvent, AuthorityRecord, AuthoritySync, Error};

/// Durable room authority. Adapters persist one compare-and-swap attempt.
/// Transaction reuse, serializable retry, and outbox identity live in
/// messenger-admission-usecase.
pub trait RoomAuthority: Send + Sync {
    fn create_room(
        &self,
        creator: &str,
        join_rule: &str,
    ) -> impl std::future::Future<Output = Result<String, Error>> + Send;

    fn snapshot(&self) -> impl std::future::Future<Output = Result<AuthorityRecord, Error>> + Send;

    /// Ok stores `record`. Unavailable means the caller should retry.
    fn commit(
        &self,
        expected_generation: u64,
        record: AuthorityRecord,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

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
