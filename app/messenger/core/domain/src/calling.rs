use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CallCapabilities {
    pub microphone: bool,
    pub camera: bool,
    pub screen_share: bool,
}

/// Short-lived media credential. Deliberately omits Debug to keep tokens out of logs.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallGrant {
    pub url: String,
    pub jwt: String,
    pub room: String,
    pub identity: String,
    pub expires_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CallPeer {
    pub identity: String,
    pub user: String,
    pub device: String,
}

/// Ephemeral authenticated media material. Never persist this in room history.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CallKey {
    pub room_id: String,
    pub call_id: String,
    pub epoch: u64,
    pub key: Vec<u8>,
    pub expires_at: u64,
    pub user: String,
    pub device: String,
    pub identity: String,
}

impl std::fmt::Debug for CallKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallKey")
            .field("room_id", &self.room_id)
            .field("call_id", &self.call_id)
            .field("epoch", &self.epoch)
            .field("key", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .field("user", &self.user)
            .field("device", &self.device)
            .field("identity", &self.identity)
            .finish()
    }
}

/// Olm keys may arrive before SFU membership. Retain one latest unexpired key
/// per sender until a verified installation acknowledges that exact epoch.
pub fn merge_call_keys(
    pending: &mut Vec<CallKey>,
    room: &str,
    call: &str,
    incoming: Vec<CallKey>,
    now: u64,
) {
    pending.retain(|key| key.room_id == room && key.call_id == call && key.expires_at > now);
    for key in incoming
        .into_iter()
        .filter(|key| key.room_id == room && key.call_id == call && key.expires_at > now)
    {
        if let Some(old) = pending.iter_mut().find(|old| old.identity == key.identity) {
            if key.epoch > old.epoch {
                *old = key;
            }
        } else if pending.len() < 256 {
            pending.push(key);
        }
    }
}
