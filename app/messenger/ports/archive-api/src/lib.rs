#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::{ArchiveEvent, ArchiveEventPage, Error, valid_room, valid_user};

pub const MAX_ARCHIVE_PAGE: u16 = 100;

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

pub fn validate_capture(room: &str, event: &ArchiveEvent) -> Result<(), Error> {
    if !valid_room(room) {
        return Err(Error::Invalid("invalid room".into()));
    }
    if event.id.is_empty() || event.id.len() > 255 || event.id.chars().any(char::is_control) {
        return Err(Error::Invalid("invalid archive event id".into()));
    }
    if !valid_user(&event.sender) {
        return Err(Error::Invalid("invalid user".into()));
    }
    if event.timestamp == 0 {
        return Err(Error::Invalid("invalid timestamp".into()));
    }
    if event.event_type.is_empty()
        || event.event_type.len() > 255
        || event.event_type.chars().any(char::is_control)
    {
        return Err(Error::Invalid("invalid event type".into()));
    }
    if !event.content.is_object() {
        return Err(Error::Invalid("invalid event content".into()));
    }
    let encoded = serde_json::to_vec(&event.content)
        .map_err(|_| Error::Invalid("invalid event content".into()))?;
    if encoded.len() > 65_536 {
        return Err(Error::Invalid("event content exceeds 65536 bytes".into()));
    }
    event.media_parts()?;
    Ok(())
}

pub fn validate_page(room: &str, limit: u16) -> Result<(), Error> {
    if !valid_room(room) {
        return Err(Error::Invalid("invalid room".into()));
    }
    if limit == 0 || limit > MAX_ARCHIVE_PAGE {
        return Err(Error::Invalid("invalid archive page size".into()));
    }
    Ok(())
}

pub fn same_archive_event(left: &ArchiveEvent, right: &ArchiveEvent) -> bool {
    left.id == right.id
        && left.sender == right.sender
        && left.timestamp == right.timestamp
        && left.event_type == right.event_type
        && left.content == right.content
        && left.decryption == right.decryption
}
