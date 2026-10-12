use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityType {
    pub entity_type: String,     // data_class: INTERNAL_ONLY
    pub revision: u32,           // data_class: INTERNAL_ONLY
    pub properties: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Property {
    pub value_type: String,       // data_class: INTERNAL_ONLY
    pub data_class: String,       // data_class: INTERNAL_ONLY
    pub value: serde_json::Value, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpcastState {
    Current,
    UpcastPending,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedObject {
    pub object_ref: String,                     // data_class: TENANT_SCOPED
    pub written_revision: u32,                  // data_class: INTERNAL_ONLY
    pub upcast_state: UpcastState,              // data_class: INTERNAL_ONLY
    pub properties: BTreeMap<String, Property>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

/// Caller-supplied pin metadata, not a native snapshot or validation receipt.
/// Foundry reports the stored revision separately.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectDetail {
    pub requested_revision: u32, // data_class: INTERNAL_ONLY
    pub object: PinnedObject,    // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectPage {
    /// Caller-supplied request pin, independent of each row's stored revision.
    pub requested_revision: u32, // data_class: INTERNAL_ONLY
    pub objects: Vec<PinnedObject>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    /// Last admitted object reference, only when more matching rows exist.
    pub next: Option<String>, // data_class: TENANT_SCOPED
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRow {
    pub ordinal: u64,             // data_class: INTERNAL_ONLY
    pub object_sequence: u64,     // data_class: INTERNAL_ONLY
    pub principal_id: String,     // data_class: TENANT_SCOPED
    pub decision_id: String,      // data_class: INTERNAL_ONLY
    pub action_type: String,      // data_class: INTERNAL_ONLY
    pub audit_event_type: String, // data_class: INTERNAL_ONLY
    pub schema_revision: u32,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditDisposition {
    Applied,
    Poisoned,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRow {
    pub ordinal: u64,                  // data_class: INTERNAL_ONLY
    pub object_ref: String,            // data_class: TENANT_SCOPED
    pub action_type: String,           // data_class: INTERNAL_ONLY
    pub disposition: AuditDisposition, // data_class: INTERNAL_ONLY
    pub poison_reason: Option<String>, // data_class: INTERNAL_ONLY
    pub principal_id: Option<String>,  // data_class: TENANT_SCOPED
    pub decision_id: Option<String>,   // data_class: INTERNAL_ONLY
}
