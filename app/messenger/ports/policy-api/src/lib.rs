#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use messenger_domain::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    pub tenant: String,
    pub subject: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    CreateRoom,
    ManageRoom,
    Send,
    Invite,
    Archive,
    ReadObject,
    InvokeAction,
}

impl Action {
    /// Wire slug for `POST /v1/authorize`; matches the Cedar seed action map.
    pub fn slug(self) -> &'static str {
        match self {
            Self::CreateRoom => "messenger.room.create",
            Self::ManageRoom => "messenger.room.manage",
            Self::Send => "messenger.message.send",
            Self::Invite => "messenger.room.invite",
            Self::Archive => "messenger.archive.capture",
            Self::ReadObject => "messenger.object.read",
            Self::InvokeAction => "messenger.object.invoke",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub id: String,
    pub version: String,
}

/// Identity and resource tenant must be verified by the calling adapter.
/// Errors and absent grants deny. A client-provided allow is never accepted.
pub trait Policy: Send + Sync {
    fn authorize(
        &self,
        principal: &Principal,
        action: Action,
        resource: &str,
    ) -> impl std::future::Future<Output = Result<Decision, Error>> + Send;
}

#[cfg(test)]
mod tests {
    use super::Action;

    #[test]
    fn slugs_match_cedar_seed_action_map() {
        assert_eq!(Action::CreateRoom.slug(), "messenger.room.create");
        assert_eq!(Action::ManageRoom.slug(), "messenger.room.manage");
        assert_eq!(Action::Send.slug(), "messenger.message.send");
        assert_eq!(Action::Invite.slug(), "messenger.room.invite");
        assert_eq!(Action::Archive.slug(), "messenger.archive.capture");
        assert_eq!(Action::ReadObject.slug(), "messenger.object.read");
        assert_eq!(Action::InvokeAction.slug(), "messenger.object.invoke");
    }
}
