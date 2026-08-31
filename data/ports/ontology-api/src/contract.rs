//! Request/response DTOs, the in-memory directory, and the idempotency
//! ledger of the upsert surface.

use std::collections::BTreeMap;

use data_ontology_domain::ObjectEntity;

use crate::codes::ObjectGraphEntityUpsertApiStatus;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityPropertyRef {
    pub name: String,       // data_class: INTERNAL_ONLY
    pub value: String,      // data_class: INTERNAL_ONLY
    pub tier: String,       // data_class: INTERNAL_ONLY
    pub data_class: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertRequest {
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub entity_id: String,   // data_class: INTERNAL_ONLY
    pub entity_type: String, // data_class: INTERNAL_ONLY
    pub property_refs: Vec<ObjectGraphEntityPropertyRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiRequest {
    pub path_tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub path_entity_id: String,                  // data_class: INTERNAL_ONLY
    pub boundary: ObjectGraphApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: ObjectGraphApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: ObjectGraphApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: ObjectGraphEntityUpsertRequest,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectGraphEntityDirectory {
    pub(crate) entities: BTreeMap<ObjectGraphEntityKey, ObjectEntity>, // data_class: INTERNAL_ONLY
    pub(crate) events: Vec<ObjectGraphEntityMutationEvent>,            // data_class: INTERNAL_ONLY
}

impl ObjectGraphEntityDirectory {
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ObjectGraphEntityKey {
    pub(crate) tenant_id: String, // data_class: INTERNAL_ONLY
    pub(crate) entity_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectGraphEntityMutationEvent {
    pub(crate) event_id: String,   // data_class: INTERNAL_ONLY
    pub(crate) tenant_id: String,  // data_class: INTERNAL_ONLY
    pub(crate) entity_id: String,  // data_class: INTERNAL_ONLY
    pub(crate) request_id: String, // data_class: INTERNAL_ONLY
    pub(crate) result: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertIdempotencyLedger {
    pub(crate) entries: BTreeMap<
        ObjectGraphEntityUpsertIdempotencyLedgerKey,
        ObjectGraphEntityUpsertIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl ObjectGraphEntityUpsertIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ObjectGraphEntityUpsertIdempotencyLedgerKey {
    pub(crate) tenant_id: String,       // data_class: INTERNAL_ONLY
    pub(crate) principal_id: String,    // data_class: INTERNAL_ONLY
    pub(crate) surface: String,         // data_class: INTERNAL_ONLY
    pub(crate) idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectGraphEntityUpsertIdempotencyLedgerEntry {
    pub(crate) fingerprint: ObjectGraphEntityUpsertRequestFingerprint, // data_class: INTERNAL_ONLY
    pub(crate) result: ObjectGraphEntityUpsertSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectGraphEntityUpsertRequestFingerprint {
    pub(crate) canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertSuccessResponse {
    pub data: ObjectGraphEntityRecord, // data_class: INTERNAL_ONLY
    pub metadata: ObjectGraphEntityUpsertMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertMetadata {
    pub request_id: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
    pub result: String,       // data_class: INTERNAL_ONLY
    pub event_id: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityRecord {
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub entity_id: String,   // data_class: INTERNAL_ONLY
    pub entity_type: String, // data_class: INTERNAL_ONLY
    pub property_refs: Vec<ObjectGraphEntityPropertyRef>, // data_class: INTERNAL_ONLY
    pub schema_version: u32, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiErrorResponse {
    pub error: ObjectGraphEntityUpsertApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiErrorBody {
    pub code: String,                                        // data_class: INTERNAL_ONLY
    pub message: String,                                     // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,                   // data_class: INTERNAL_ONLY
    pub request_id: String,                                  // data_class: INTERNAL_ONLY
    pub details: Vec<ObjectGraphEntityUpsertApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}
