#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod apply;
mod archive;
mod authority;
mod calling;
mod collaboration;
mod conversation;
mod error;
mod integrations;
mod message;
mod object;
mod query;
mod record;
mod rooms;

pub const ENTERPRISE_ROOM_TYPE: &str = "dev.oyatie.enterprise";

pub use archive::{ArchiveEvent, ArchiveEventPage};
pub use authority::{
    Admission, AdmitCommand, AuthorityEvent, AuthorityRoomDelta, AuthoritySync, ban_allowed,
    join_allowed, member_can_leave, member_can_send, send_endpoint, valid_room, valid_txn,
    valid_user, validate_admit,
};
pub use calling::{CallCapabilities, CallGrant, CallKey, CallPeer, merge_call_keys};
pub use collaboration::{ConsoleCommand, ConsoleObjectRef};
pub use conversation::{
    ReactionChange, ReactionSummary, reaction_key, search_messages, visible_messages,
};
pub use error::Error;
pub use integrations::{
    Delivery, Installation, InstallationSpec, IntegrationCapability, IntegrationKind,
};
pub use message::{
    Attachment, Decryption, MAX_MESSAGE_BYTES, MediaFile, Message, MessagePage, OutgoingMessage,
    RoomDelta, SyncUpdate, merge_messages, message_body,
};
pub use object::ObjectRef;
pub use record::{AuthorityRecord, Room, StoredEvent};
pub use rooms::{DeviceSummary, LoginMethods, MemberSummary, RoomMode, RoomSummary};
