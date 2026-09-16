use crate::{Decryption, Error, valid_room, valid_user};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArchiveEvent {
    pub id: String,
    pub sender: String,
    pub timestamp: u64,
    pub event_type: String,
    pub content: serde_json::Value,
    pub decryption: Decryption,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveEventPage {
    pub events: Vec<ArchiveEvent>,
    pub next: Option<String>,
}

impl ArchiveEvent {
    /// Required encrypted media, including replacement content and thumbnails.
    pub fn media_parts(&self) -> Result<Vec<(String, serde_json::Value)>, Error> {
        let mut parts = Vec::new();
        for (content, primary, thumbnail) in [
            (&self.content, "primary", "thumbnail"),
            (
                &self.content["m.new_content"],
                "replacement",
                "replacement_thumbnail",
            ),
        ] {
            if content.is_null() {
                continue;
            }
            let media = matches!(
                content["msgtype"].as_str(),
                Some("m.file" | "m.image" | "m.audio" | "m.video")
            );
            if content.get("url").is_some() || content["info"].get("thumbnail_url").is_some() {
                return Err(Error::Unencrypted);
            }
            for (descriptor, required, role, size) in [
                (&content["file"], media, primary, &content["info"]["size"]),
                (
                    &content["info"]["thumbnail_file"],
                    false,
                    thumbnail,
                    &content["info"]["thumbnail_info"]["size"],
                ),
            ] {
                if descriptor.is_null() {
                    if required {
                        return Err(Error::ArchiveNotReady);
                    }
                    continue;
                }
                if !descriptor.is_object()
                    || descriptor["v"] != "v2"
                    || !descriptor["url"]
                        .as_str()
                        .is_some_and(|s| s.starts_with("mxc://"))
                    || descriptor["key"]["kty"] != "oct"
                    || !descriptor["key"]["k"].is_string()
                    || !descriptor["iv"].is_string()
                    || !descriptor["hashes"]["sha256"].is_string()
                {
                    return Err(Error::ArchiveNotReady);
                }
                if !size.is_null() && size.as_u64().is_none_or(|size| size > 20 * 1024 * 1024) {
                    return Err(Error::Invalid(
                        "archived attachments must not exceed 20 MiB".into(),
                    ));
                }
                parts.push((role.into(), descriptor.clone()));
            }
        }
        Ok(parts)
    }

    pub fn validate_capture(&self, room: &str) -> Result<(), Error> {
        if !valid_room(room) {
            return Err(Error::Invalid("invalid room".into()));
        }
        if self.id.is_empty() || self.id.len() > 255 || self.id.chars().any(char::is_control) {
            return Err(Error::Invalid("invalid archive event id".into()));
        }
        if !valid_user(&self.sender) {
            return Err(Error::Invalid("invalid user".into()));
        }
        if self.timestamp == 0 {
            return Err(Error::Invalid("invalid timestamp".into()));
        }
        if self.event_type.is_empty()
            || self.event_type.len() > 255
            || self.event_type.chars().any(char::is_control)
        {
            return Err(Error::Invalid("invalid event type".into()));
        }
        if !self.content.is_object() {
            return Err(Error::Invalid("invalid event content".into()));
        }
        let encoded = serde_json::to_vec(&self.content)
            .map_err(|_| Error::Invalid("invalid event content".into()))?;
        if encoded.len() > 65_536 {
            return Err(Error::Invalid("event content exceeds 65536 bytes".into()));
        }
        self.media_parts()?;
        Ok(())
    }
}

impl ArchiveEventPage {
    pub const MAX_ARCHIVE_PAGE: u16 = 100;

    pub fn validate_page(room: &str, limit: u16) -> Result<(), Error> {
        if !valid_room(room) {
            return Err(Error::Invalid("invalid room".into()));
        }
        if limit == 0 || limit > Self::MAX_ARCHIVE_PAGE {
            return Err(Error::Invalid("invalid archive page size".into()));
        }
        Ok(())
    }
}
