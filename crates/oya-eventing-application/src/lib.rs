//! Platform Eventing app boundary.
//!
//! This crate owns CloudEvents envelope normalization, producer authorization,
//! request fingerprint idempotency, privacy-program data-class validation, and
//! platform eventing outbox publication for `eventing.outbox.publish`.

use std::collections::BTreeMap;

use oya_data_boundary_kernel::{PrivacyDataClass, parse_data_class_label};
use oya_eventing_domain::{EventingError, Outbox, OutboxRecord};

pub const EVENTING_OUTBOX_PUBLISH_SURFACE: &str = "eventing.outbox.publish";
pub const EVENTING_OUTBOX_PUBLISH_TOPIC: &str = "oya.foundation.eventing";
pub const EVENTING_OUTBOX_PUBLISH_SCHEMA: &str = "eventing.outbox.publish.v1";
pub const EVENTING_OUTBOX_PUBLISH_SOURCE: &str = "oyatie://platform/eventing";
pub const EVENTING_OUTBOX_PUBLISH_ASYNCAPI_CONTRACT: &str =
    "contracts/asyncapi/platform/eventing-outbox-v1.yaml";
pub const EVENTING_OUTBOX_PUBLISH_PROTO_CONTRACT: &str =
    "contracts/proto/platform/eventing/v1/eventing-outbox-v1.proto";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventingOutboxPublishAppStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl EventingOutboxPublishAppStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventingOutboxEnvelopeContext {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub source: String,                 // data_class: INTERNAL_ONLY
    pub subject: String,                // data_class: INTERNAL_ONLY
    pub topic: String,                  // data_class: INTERNAL_ONLY
    pub schema: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub producer_id: String,            // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub produced_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventingOutboxPublishAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub producer_id: String,           // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventingOutboxPublishPayload {
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub target_topic: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
    pub payload_ref: String,                    // data_class: INTERNAL_ONLY
    pub payload_schema: String,                 // data_class: INTERNAL_ONLY
    pub data_classes_touched: Vec<String>,      // data_class: INTERNAL_ONLY
    pub regulatory_packs_consumed: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventingOutboxPublishAppRequest {
    pub envelope: EventingOutboxEnvelopeContext, // data_class: INTERNAL_ONLY
    pub authorization: EventingOutboxPublishAuthorization, // data_class: INTERNAL_ONLY
    pub payload: EventingOutboxPublishPayload,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventingOutboxPublishIdempotencyLedger {
    entries: BTreeMap<
        EventingOutboxPublishIdempotencyLedgerKey,
        EventingOutboxPublishIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl EventingOutboxPublishIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct EventingOutboxPublishIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    producer_id: String,     // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EventingOutboxPublishIdempotencyLedgerEntry {
    fingerprint: EventingOutboxPublishRequestFingerprint, // data_class: INTERNAL_ONLY
    result: EventingOutboxPublishAppResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EventingOutboxPublishRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type EventingOutboxPublishAppResult =
    Result<EventingOutboxPublishSuccessResponse, EventingOutboxPublishAppError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventingOutboxPublishSuccessResponse {
    pub data: EventingOutboxPublishRecord, // data_class: INTERNAL_ONLY
    pub metadata: EventingOutboxPublishMetadata, // data_class: INTERNAL_ONLY
}

impl EventingOutboxPublishSuccessResponse {
    pub fn accepted(
        data: EventingOutboxPublishRecord,
        envelope: &EventingOutboxEnvelopeContext,
    ) -> Self {
        Self {
            data,
            metadata: EventingOutboxPublishMetadata {
                event_id: envelope.event_id.clone(),
                tenant_id: envelope.tenant_id.clone(),
                producer_id: envelope.producer_id.clone(),
                topic: envelope.topic.clone(),
                schema: envelope.schema.clone(),
                produced_at_epoch_seconds: envelope.produced_at_epoch_seconds,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventingOutboxPublishMetadata {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub producer_id: String,            // data_class: INTERNAL_ONLY
    pub topic: String,                  // data_class: INTERNAL_ONLY
    pub schema: String,                 // data_class: INTERNAL_ONLY
    pub produced_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventingOutboxPublishRecord {
    pub sequence: u64,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub target_topic: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
    pub payload_ref: String,                    // data_class: INTERNAL_ONLY
    pub published: bool,                        // data_class: INTERNAL_ONLY
    pub payload_schema: String,                 // data_class: INTERNAL_ONLY
    pub data_classes_touched: Vec<String>,      // data_class: INTERNAL_ONLY
    pub regulatory_packs_consumed: Vec<String>, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventingOutboxPublishAppError {
    EmptyEventId,
    EmptySource,
    InvalidSource {
        source: String,
    },
    EmptySubject,
    SubjectMismatch {
        expected_subject: String,
        actual_subject: String,
    },
    InvalidTopic {
        topic: String,
    },
    InvalidSchema {
        schema: String,
    },
    EmptyTenantId,
    EmptyProducerId,
    EmptyIdempotencyKey,
    InvalidProducedAt,
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        envelope_tenant_id: String,
    },
    AuthorizationProducerMismatch {
        authorization_producer_id: String,
        envelope_producer_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    EnvelopePayloadTenantMismatch {
        envelope_tenant_id: String,
        payload_tenant_id: String,
    },
    EnvelopePayloadIdempotencyMismatch {
        envelope_idempotency_key: String,
        payload_idempotency_key: String,
    },
    InvalidTargetTopic {
        topic: String,
    },
    InvalidPayloadRef {
        payload_ref: String,
    },
    InvalidPayloadSchema {
        payload_schema: String,
    },
    MissingDataClassesTouched,
    InvalidDataClassLabel {
        data_class: String,
    },
    MissingRegulatoryPacksConsumed,
    InvalidRegulatoryPack {
        regulatory_pack: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Eventing(EventingError),
}

impl EventingOutboxPublishAppError {
    pub fn eventing_outbox_publish_status(&self) -> EventingOutboxPublishAppStatus {
        match self {
            Self::EmptyProducerId => EventingOutboxPublishAppStatus::Unauthorized,
            Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationProducerMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::EnvelopePayloadTenantMismatch { .. } => {
                EventingOutboxPublishAppStatus::Forbidden
            }
            Self::IdempotencyKeyReused { .. }
            | Self::Eventing(EventingError::OutboxRecordNotFound)
            | Self::Eventing(EventingError::InvalidOutboxHistory) => {
                EventingOutboxPublishAppStatus::UnprocessableEntity
            }
            Self::EmptyEventId
            | Self::EmptySource
            | Self::InvalidSource { .. }
            | Self::EmptySubject
            | Self::SubjectMismatch { .. }
            | Self::InvalidTopic { .. }
            | Self::InvalidSchema { .. }
            | Self::EmptyTenantId
            | Self::EmptyIdempotencyKey
            | Self::InvalidProducedAt
            | Self::EmptyAuthorizationDecisionId
            | Self::EnvelopePayloadIdempotencyMismatch { .. }
            | Self::InvalidTargetTopic { .. }
            | Self::InvalidPayloadRef { .. }
            | Self::InvalidPayloadSchema { .. }
            | Self::MissingDataClassesTouched
            | Self::InvalidDataClassLabel { .. }
            | Self::MissingRegulatoryPacksConsumed
            | Self::InvalidRegulatoryPack { .. }
            | Self::Eventing(EventingError::EmptyTopic)
            | Self::Eventing(EventingError::EmptyTopicAxis)
            | Self::Eventing(EventingError::EmptyTopicDescription)
            | Self::Eventing(EventingError::InvalidTopicName)
            | Self::Eventing(EventingError::DuplicateTopic)
            | Self::Eventing(EventingError::TopicNotFound)
            | Self::Eventing(EventingError::EmptyIdempotencyKey)
            | Self::Eventing(EventingError::EmptyPayloadRef) => {
                EventingOutboxPublishAppStatus::BadRequest
            }
        }
    }

    pub fn eventing_outbox_publish_status_code(&self) -> u16 {
        self.eventing_outbox_publish_status().code()
    }
}

pub fn validate_eventing_outbox_publish_request(
    request: &EventingOutboxPublishAppRequest,
) -> Result<(), EventingOutboxPublishAppError> {
    validate_envelope(&request.envelope)?;
    validate_authorization(&request.envelope, &request.authorization)?;
    validate_payload(&request.payload)?;
    validate_envelope_payload_binding(&request.envelope, &request.payload)
}

pub fn publish_eventing_outbox_from_app(
    outbox: &mut Outbox,
    idempotency_ledger: &mut EventingOutboxPublishIdempotencyLedger,
    request: EventingOutboxPublishAppRequest,
) -> Result<EventingOutboxPublishSuccessResponse, EventingOutboxPublishAppError> {
    validate_eventing_outbox_publish_request(&request)?;
    let key = idempotency_key_for(&request.envelope, EVENTING_OUTBOX_PUBLISH_SURFACE);
    let fingerprint = eventing_outbox_publish_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(EventingOutboxPublishAppError::IdempotencyKeyReused {
            idempotency_key: request.envelope.idempotency_key,
        });
    }

    let envelope = request.envelope.clone();
    let payload = request.payload.clone();
    let result = publish_outbox(outbox, request.payload).map(|record| {
        EventingOutboxPublishSuccessResponse::accepted(
            outbox_publish_record(record, payload),
            &envelope,
        )
    });

    idempotency_ledger.entries.insert(
        key,
        EventingOutboxPublishIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_envelope(
    envelope: &EventingOutboxEnvelopeContext,
) -> Result<(), EventingOutboxPublishAppError> {
    if envelope.event_id.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::EmptyEventId);
    }
    if envelope.source.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::EmptySource);
    }
    if envelope.source != EVENTING_OUTBOX_PUBLISH_SOURCE {
        return Err(EventingOutboxPublishAppError::InvalidSource {
            source: envelope.source.clone(),
        });
    }
    if envelope.subject.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::EmptySubject);
    }
    if envelope.topic != EVENTING_OUTBOX_PUBLISH_TOPIC {
        return Err(EventingOutboxPublishAppError::InvalidTopic {
            topic: envelope.topic.clone(),
        });
    }
    if envelope.schema != EVENTING_OUTBOX_PUBLISH_SCHEMA {
        return Err(EventingOutboxPublishAppError::InvalidSchema {
            schema: envelope.schema.clone(),
        });
    }
    if envelope.tenant_id.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::EmptyTenantId);
    }
    if envelope.producer_id.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::EmptyProducerId);
    }
    if envelope.idempotency_key.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::EmptyIdempotencyKey);
    }
    if envelope.produced_at_epoch_seconds == 0 {
        return Err(EventingOutboxPublishAppError::InvalidProducedAt);
    }
    Ok(())
}

fn validate_authorization(
    envelope: &EventingOutboxEnvelopeContext,
    authorization: &EventingOutboxPublishAuthorization,
) -> Result<(), EventingOutboxPublishAppError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != envelope.tenant_id {
        return Err(EventingOutboxPublishAppError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            envelope_tenant_id: envelope.tenant_id.clone(),
        });
    }
    if authorization.producer_id != envelope.producer_id {
        return Err(
            EventingOutboxPublishAppError::AuthorizationProducerMismatch {
                authorization_producer_id: authorization.producer_id.clone(),
                envelope_producer_id: envelope.producer_id.clone(),
            },
        );
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == EVENTING_OUTBOX_PUBLISH_SURFACE)
    {
        return Err(EventingOutboxPublishAppError::AuthorizationDenied {
            surface: EVENTING_OUTBOX_PUBLISH_SURFACE.to_string(),
        });
    }
    Ok(())
}

fn validate_envelope_payload_binding(
    envelope: &EventingOutboxEnvelopeContext,
    payload: &EventingOutboxPublishPayload,
) -> Result<(), EventingOutboxPublishAppError> {
    if envelope.tenant_id != payload.tenant_id {
        return Err(
            EventingOutboxPublishAppError::EnvelopePayloadTenantMismatch {
                envelope_tenant_id: envelope.tenant_id.clone(),
                payload_tenant_id: payload.tenant_id.clone(),
            },
        );
    }
    if envelope.idempotency_key != payload.idempotency_key {
        return Err(
            EventingOutboxPublishAppError::EnvelopePayloadIdempotencyMismatch {
                envelope_idempotency_key: envelope.idempotency_key.clone(),
                payload_idempotency_key: payload.idempotency_key.clone(),
            },
        );
    }
    let expected_subject = expected_subject(payload);
    if envelope.subject != expected_subject {
        return Err(EventingOutboxPublishAppError::SubjectMismatch {
            expected_subject,
            actual_subject: envelope.subject.clone(),
        });
    }
    Ok(())
}

fn validate_payload(
    payload: &EventingOutboxPublishPayload,
) -> Result<(), EventingOutboxPublishAppError> {
    validate_target_topic(&payload.target_topic)?;
    validate_payload_ref(&payload.payload_ref)?;
    if payload.payload_schema.trim().is_empty() {
        return Err(EventingOutboxPublishAppError::InvalidPayloadSchema {
            payload_schema: payload.payload_schema.clone(),
        });
    }
    validate_data_classes_touched(&payload.data_classes_touched)?;
    validate_regulatory_packs_consumed(&payload.regulatory_packs_consumed)
}

fn validate_target_topic(topic: &str) -> Result<(), EventingOutboxPublishAppError> {
    let trimmed = topic.trim();
    let suffix = trimmed.strip_prefix("oya.").unwrap_or_default();
    if trimmed != topic || suffix.is_empty() || !suffix.contains('.') || trimmed.contains("..") {
        return Err(EventingOutboxPublishAppError::InvalidTargetTopic {
            topic: topic.to_string(),
        });
    }
    Ok(())
}

fn validate_payload_ref(payload_ref: &str) -> Result<(), EventingOutboxPublishAppError> {
    let trimmed = payload_ref.trim();
    if trimmed != payload_ref
        || trimmed.is_empty()
        || trimmed.starts_with('/')
        || trimmed.contains("..")
        || !trimmed.contains('/')
    {
        return Err(EventingOutboxPublishAppError::InvalidPayloadRef {
            payload_ref: payload_ref.to_string(),
        });
    }
    Ok(())
}

fn validate_data_classes_touched(
    data_classes: &[String],
) -> Result<(), EventingOutboxPublishAppError> {
    if data_classes.is_empty() {
        return Err(EventingOutboxPublishAppError::MissingDataClassesTouched);
    }
    for label in data_classes {
        let Some(data_class) = parse_data_class_label(label) else {
            return Err(EventingOutboxPublishAppError::InvalidDataClassLabel {
                data_class: label.clone(),
            });
        };
        PrivacyDataClass::try_from(data_class).map_err(|_| {
            EventingOutboxPublishAppError::InvalidDataClassLabel {
                data_class: label.clone(),
            }
        })?;
    }
    Ok(())
}

fn validate_regulatory_packs_consumed(
    regulatory_packs: &[String],
) -> Result<(), EventingOutboxPublishAppError> {
    if regulatory_packs.is_empty() {
        return Err(EventingOutboxPublishAppError::MissingRegulatoryPacksConsumed);
    }
    for regulatory_pack in regulatory_packs {
        if !regulatory_pack.starts_with("oya-pack-") || regulatory_pack.trim() != regulatory_pack {
            return Err(EventingOutboxPublishAppError::InvalidRegulatoryPack {
                regulatory_pack: regulatory_pack.clone(),
            });
        }
    }
    Ok(())
}

fn publish_outbox(
    outbox: &mut Outbox,
    payload: EventingOutboxPublishPayload,
) -> Result<OutboxRecord, EventingOutboxPublishAppError> {
    outbox
        .publish(
            payload.tenant_id,
            payload.target_topic,
            payload.idempotency_key,
            payload.payload_ref,
        )
        .map_err(EventingOutboxPublishAppError::Eventing)
}

fn idempotency_key_for(
    envelope: &EventingOutboxEnvelopeContext,
    surface: &str,
) -> EventingOutboxPublishIdempotencyLedgerKey {
    EventingOutboxPublishIdempotencyLedgerKey {
        tenant_id: envelope.tenant_id.clone(),
        producer_id: envelope.producer_id.clone(),
        surface: surface.to_string(),
        idempotency_key: envelope.idempotency_key.clone(),
    }
}

fn eventing_outbox_publish_fingerprint_for(
    request: &EventingOutboxPublishAppRequest,
) -> EventingOutboxPublishRequestFingerprint {
    EventingOutboxPublishRequestFingerprint {
        canonical: [
            format!("event.id={}", request.envelope.event_id),
            format!("event.source={}", request.envelope.source),
            format!("event.subject={}", request.envelope.subject),
            format!("event.topic={}", request.envelope.topic),
            format!("event.schema={}", request.envelope.schema),
            format!("event.tenant_id={}", request.envelope.tenant_id),
            format!("event.producer_id={}", request.envelope.producer_id),
            format!("event.idempotency_key={}", request.envelope.idempotency_key),
            format!(
                "event.produced_at_epoch_seconds={}",
                request.envelope.produced_at_epoch_seconds
            ),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.producer_id={}",
                request.authorization.producer_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("payload.tenant_id={}", request.payload.tenant_id),
            format!("payload.target_topic={}", request.payload.target_topic),
            format!(
                "payload.idempotency_key={}",
                request.payload.idempotency_key
            ),
            format!("payload.payload_ref={}", request.payload.payload_ref),
            format!("payload.payload_schema={}", request.payload.payload_schema),
            format!(
                "payload.data_classes_touched={}",
                request.payload.data_classes_touched.join(",")
            ),
            format!(
                "payload.regulatory_packs_consumed={}",
                request.payload.regulatory_packs_consumed.join(",")
            ),
        ]
        .join("|"),
    }
}

fn outbox_publish_record(
    record: OutboxRecord,
    payload: EventingOutboxPublishPayload,
) -> EventingOutboxPublishRecord {
    EventingOutboxPublishRecord {
        sequence: record.sequence,
        tenant_id: record.tenant_id,
        target_topic: record.topic.value,
        idempotency_key: record.idempotency_key.value,
        payload_ref: record.payload_ref.value,
        published: record.published,
        payload_schema: payload.payload_schema,
        data_classes_touched: payload.data_classes_touched,
        regulatory_packs_consumed: payload.regulatory_packs_consumed,
        schema_version: 1,
    }
}

fn expected_subject(payload: &EventingOutboxPublishPayload) -> String {
    format!(
        "tenant/{}/topic/{}",
        payload.tenant_id, payload.target_topic
    )
}
