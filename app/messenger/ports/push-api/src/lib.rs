#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::Deserialize;

/// Only routing IDs and aggregate counts cross the provider boundary. Message
/// bodies, sender names, room names, content, tweaks and pusher URLs are ignored.
#[derive(Clone, Deserialize)]
pub struct PushNotice {
    pub event_id: Option<String>,
    pub room_id: Option<String>,
    #[serde(default)]
    pub counts: PushCounts,
    #[serde(default)]
    pub prio: PushPriority,
    pub devices: Vec<PushDevice>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct PushCounts {
    pub unread: Option<u32>,
    pub missed_calls: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PushPriority {
    #[default]
    High,
    Low,
}

// Deliberately no Debug: a pushkey is a confidential delivery capability.
#[derive(Clone, Deserialize)]
pub struct PushDevice {
    pub app_id: String,
    pub pushkey: String,
    pub pushkey_ts: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushError {
    Invalid,
    Unavailable { retry_after: u64 },
}

impl PushNotice {
    pub fn validate(&self) -> Result<(), PushError> {
        let identity = |value: &str| {
            !value.is_empty() && !value.chars().any(|c| c.is_control() || c.is_whitespace())
        };
        if self.event_id.is_some() != self.room_id.is_some()
            || self
                .event_id
                .as_ref()
                .is_some_and(|id| !identity(id) || id.len() < 2 || !id.starts_with('$'))
            || self
                .room_id
                .as_ref()
                .is_some_and(|id| !identity(id) || id.len() < 2 || !id.starts_with('!'))
            || self
                .devices
                .iter()
                .any(|device| !identity(&device.app_id) || !identity(&device.pushkey))
        {
            return Err(PushError::Invalid);
        }
        Ok(())
    }
}

pub trait Push: Send + Sync {
    /// A rejected key is permanently invalid. Temporary failures must remain
    /// retryable; returning a key here instructs Matrix to remove its pusher.
    fn notify(
        &self,
        notice: &PushNotice,
    ) -> impl std::future::Future<Output = Result<Vec<String>, PushError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn notice() -> PushNotice {
        serde_json::from_value(json!({
            "event_id": "$event",
            "room_id": "!room:example.org",
            "counts": {"unread": 0},
            "devices": [{"app_id": "android", "pushkey": "device-token"}],
            "sender": "SECRET SENDER",
            "room_name": "SECRET ROOM",
            "content": {"body": "SECRET MESSAGE"},
            "type": "m.room.encrypted"
        }))
        .unwrap()
    }

    #[test]
    fn extra_matrix_fields_are_dropped_and_a_well_formed_notice_is_admitted() {
        let notice = notice();
        notice.validate().unwrap();
        assert_eq!(notice.event_id.as_deref(), Some("$event"));
        assert_eq!(notice.room_id.as_deref(), Some("!room:example.org"));
        assert_eq!(notice.counts.unread, Some(0));
        assert_eq!(notice.prio, PushPriority::High);
        assert_eq!(notice.devices.len(), 1);
        assert_eq!(notice.devices[0].app_id, "android");
    }

    #[test]
    fn event_and_room_must_arrive_together_with_matrix_id_prefixes() {
        let mut missing_room = notice();
        missing_room.room_id = None;
        assert_eq!(missing_room.validate(), Err(PushError::Invalid));

        let mut missing_event = notice();
        missing_event.event_id = None;
        assert_eq!(missing_event.validate(), Err(PushError::Invalid));

        let mut badge_only = notice();
        badge_only.event_id = None;
        badge_only.room_id = None;
        badge_only.validate().unwrap();

        let mut bad_event = notice();
        bad_event.event_id = Some("event".into());
        assert_eq!(bad_event.validate(), Err(PushError::Invalid));

        let mut bad_room = notice();
        bad_room.room_id = Some("#room:example.org".into());
        assert_eq!(bad_room.validate(), Err(PushError::Invalid));
    }

    #[test]
    fn identities_reject_empty_and_whitespace() {
        let mut spaced = notice();
        spaced.devices[0].app_id = "bad app".into();
        assert_eq!(spaced.validate(), Err(PushError::Invalid));

        let mut empty_key = notice();
        empty_key.devices[0].pushkey = String::new();
        assert_eq!(empty_key.validate(), Err(PushError::Invalid));
    }
}
