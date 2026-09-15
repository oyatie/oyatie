use crate::{Decryption, Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
}
