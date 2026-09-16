#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{ArchiveEvent, ArchiveEventPage, Error};

/// Durable enterprise archive. Capture is idempotent on `(room, event.id)`.
pub trait Archive: Send + Sync {
    fn capture(
        &self,
        room: &str,
        event: ArchiveEvent,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    fn page(
        &self,
        room: &str,
        after: Option<&str>,
        limit: u16,
    ) -> impl std::future::Future<Output = Result<ArchiveEventPage, Error>> + Send;
}
