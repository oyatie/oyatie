use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeAddress {
    pub email: String,
    pub parameters: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionEnvelope {
    pub mail_from: EnvelopeAddress,
    pub rcpt_to: Vec<EnvelopeAddress>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UndoStatus {
    Pending,
    Final,
    Canceled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmissionDelivered {
    Unknown,
    Queued,
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionDeliveryStatus {
    pub smtp_reply: String,
    pub delivered: SubmissionDelivered,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionRecord {
    pub id: String,
    pub identity_id: String,
    pub email_id: String,
    pub thread_id: String,
    pub envelope: SubmissionEnvelope,
    pub send_at: i64,
    pub undo_status: UndoStatus,
    pub delivery_status: BTreeMap<String, SubmissionDeliveryStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmissionFilter {
    All,
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Vec<Self>),
    IdentityIds(Vec<String>),
    EmailIds(Vec<String>),
    ThreadIds(Vec<String>),
    UndoStatus(UndoStatus),
    Before(i64),
    After(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionSortField {
    EmailId,
    ThreadId,
    SentAt,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmissionQuery {
    pub filter: SubmissionFilter,
    pub sort: Vec<(SubmissionSortField, bool)>,
    pub position: i64,
    pub anchor: Option<String>,
    pub anchor_offset: i64,
    pub limit: usize,
}
