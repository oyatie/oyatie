use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomMode {
    Personal,
    Enterprise,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomSummary {
    pub id: String,
    pub name: String,
    pub encrypted: bool,
    pub invited: bool,
    pub audit_bot: Option<String>,
    pub tenant: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub unread: u64,
    #[serde(default)]
    pub mentions: u64,
    #[serde(default)]
    pub notifications: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemberSummary {
    pub user: String,
    pub name: String,
    pub membership: String,
    pub audit_bot: bool,
    pub can_kick: bool,
    pub can_ban: bool,
    pub can_unban: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginMethods {
    pub password: bool,
    pub sso: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSummary {
    pub user: String,
    pub id: String,
    pub fingerprint: String,
    pub verified: bool,
}
