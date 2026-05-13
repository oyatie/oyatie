//! Platform Metering app boundary.
//!
//! This crate owns CloudEvents envelope normalization, producer authorization,
//! request fingerprint idempotency, platform metering kernel projection, and
//! eventing outbox publication for `metering.event.ingest`.

use std::collections::BTreeMap;

use oya_platform_data_boundary_kernel::{parse_data_class_label, DataClass};
use oya_platform_eventing_kernel::{EventingError, Outbox, OutboxRecord};
use oya_platform_metering_kernel::{
    AxisId, Meter, MeterEvent, MeterEventCreate, MeterUnit, MeterUnitKind, MeteringError, PlaneTag,
};

pub const METERING_EVENT_INGEST_SURFACE: &str = "metering.event.ingest";
pub const METERING_EVENT_TOPIC: &str = "oya.platform.metering";
pub const METERING_EVENT_INGEST_SCHEMA: &str = "metering.event.ingest.v1";
pub const METERING_EVENT_INGEST_SOURCE: &str = "oyatie://platform/metering";
pub const METERING_EVENT_ASYNCAPI_CONTRACT: &str =
    "contracts/asyncapi/platform/metering-events-v1.yaml";
pub const METERING_EVENT_PROTO_CONTRACT: &str =
    "contracts/proto/platform/metering/v1/metering-event-v1.proto";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeteringEventIngestAppStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl MeteringEventIngestAppStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringEventEnvelopeContext {
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
pub struct MeteringEventIngestAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub producer_id: String,           // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringMeterUnitRequest {
    pub kind: String,             // data_class: INTERNAL_ONLY
    pub quantity_microunits: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringEventIngestPayload {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub capability_id: String,                // data_class: INTERNAL_ONLY
    pub plane: String,                        // data_class: INTERNAL_ONLY
    pub units: Vec<MeteringMeterUnitRequest>, // data_class: INTERNAL_ONLY
    pub source_axis: String,                  // data_class: INTERNAL_ONLY
    pub recorded_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub data_class: String,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringEventIngestAppRequest {
    pub envelope: MeteringEventEnvelopeContext, // data_class: INTERNAL_ONLY
    pub authorization: MeteringEventIngestAuthorization, // data_class: INTERNAL_ONLY
    pub payload: MeteringEventIngestPayload,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MeteringEventIngestIdempotencyLedger {
    entries: BTreeMap<
        MeteringEventIngestIdempotencyLedgerKey,
        MeteringEventIngestIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl MeteringEventIngestIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MeteringEventIngestIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    producer_id: String,     // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MeteringEventIngestIdempotencyLedgerEntry {
    fingerprint: MeteringEventIngestRequestFingerprint, // data_class: INTERNAL_ONLY
    result: MeteringEventIngestAppResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MeteringEventIngestRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type MeteringEventIngestAppResult =
    Result<MeteringEventIngestSuccessResponse, MeteringEventIngestAppError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringEventIngestSuccessResponse {
    pub data: MeteringEventIngestRecord, // data_class: INTERNAL_ONLY
    pub metadata: MeteringEventIngestMetadata, // data_class: INTERNAL_ONLY
}

impl MeteringEventIngestSuccessResponse {
    pub fn accepted(
        data: MeteringEventIngestRecord,
        envelope: &MeteringEventEnvelopeContext,
    ) -> Self {
        Self {
            data,
            metadata: MeteringEventIngestMetadata {
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
pub struct MeteringEventIngestMetadata {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub producer_id: String,            // data_class: INTERNAL_ONLY
    pub topic: String,                  // data_class: INTERNAL_ONLY
    pub schema: String,                 // data_class: INTERNAL_ONLY
    pub produced_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringMeterUnitRecord {
    pub kind: String,             // data_class: INTERNAL_ONLY
    pub quantity_microunits: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeteringEventIngestRecord {
    pub id: String,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub capability_id: String,               // data_class: INTERNAL_ONLY
    pub plane: String,                       // data_class: INTERNAL_ONLY
    pub units: Vec<MeteringMeterUnitRecord>, // data_class: INTERNAL_ONLY
    pub source_axis: String,                 // data_class: INTERNAL_ONLY
    pub recorded_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub data_class: String,                  // data_class: PUBLIC
    pub meter_schema_version: u32,           // data_class: PUBLIC
    pub outbox_sequence: u64,                // data_class: INTERNAL_ONLY
    pub outbox_topic: String,                // data_class: INTERNAL_ONLY
    pub outbox_idempotency_key: String,      // data_class: INTERNAL_ONLY
    pub outbox_payload_ref: String,          // data_class: INTERNAL_ONLY
    pub outbox_published: bool,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeteringEventIngestAppError {
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
    EnvelopePayloadEventIdMismatch {
        envelope_event_id: String,
        payload_event_id: String,
    },
    EnvelopePayloadTenantMismatch {
        envelope_tenant_id: String,
        payload_tenant_id: String,
    },
    EnvelopePayloadIdempotencyMismatch {
        envelope_idempotency_key: String,
        payload_idempotency_key: String,
    },
    InvalidPlane {
        plane: String,
    },
    InvalidSourceAxis {
        source_axis: String,
    },
    InvalidMeterUnitKind {
        kind: String,
    },
    InvalidMeterUnitQuantity {
        kind: String,
        quantity_microunits: u64,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Metering(MeteringError),
    Eventing(EventingError),
}

impl MeteringEventIngestAppError {
    pub fn metering_event_ingest_status(&self) -> MeteringEventIngestAppStatus {
        match self {
            Self::EmptyProducerId => MeteringEventIngestAppStatus::Unauthorized,
            Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationProducerMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::EnvelopePayloadTenantMismatch { .. } => MeteringEventIngestAppStatus::Forbidden,
            Self::IdempotencyKeyReused { .. }
            | Self::Eventing(EventingError::OutboxRecordNotFound)
            | Self::Eventing(EventingError::InvalidOutboxHistory) => {
                MeteringEventIngestAppStatus::UnprocessableEntity
            }
            Self::Metering(MeteringError::DuplicateMeterEvent) => {
                MeteringEventIngestAppStatus::Conflict
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
            | Self::EnvelopePayloadEventIdMismatch { .. }
            | Self::EnvelopePayloadIdempotencyMismatch { .. }
            | Self::InvalidPlane { .. }
            | Self::InvalidSourceAxis { .. }
            | Self::InvalidMeterUnitKind { .. }
            | Self::InvalidMeterUnitQuantity { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::Metering(_)
            | Self::Eventing(EventingError::EmptyTopic)
            | Self::Eventing(EventingError::EmptyIdempotencyKey)
            | Self::Eventing(EventingError::EmptyPayloadRef) => {
                MeteringEventIngestAppStatus::BadRequest
            }
        }
    }

    pub fn metering_event_ingest_status_code(&self) -> u16 {
        self.metering_event_ingest_status().code()
    }
}

pub fn validate_metering_event_ingest_request(
    request: &MeteringEventIngestAppRequest,
) -> Result<(), MeteringEventIngestAppError> {
    validate_envelope(&request.envelope)?;
    validate_authorization(&request.envelope, &request.authorization)?;
    validate_payload_labels(&request.payload)?;
    validate_envelope_payload_binding(&request.envelope, &request.payload)
}

pub fn ingest_metering_event_from_app(
    meter: &mut Meter,
    outbox: &mut Outbox,
    idempotency_ledger: &mut MeteringEventIngestIdempotencyLedger,
    request: MeteringEventIngestAppRequest,
) -> Result<MeteringEventIngestSuccessResponse, MeteringEventIngestAppError> {
    validate_metering_event_ingest_request(&request)?;
    let key = idempotency_key_for(&request.envelope, METERING_EVENT_INGEST_SURFACE);
    let fingerprint = metering_event_ingest_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(MeteringEventIngestAppError::IdempotencyKeyReused {
            idempotency_key: request.envelope.idempotency_key,
        });
    }

    let envelope = request.envelope.clone();
    let result = metering_event_input(request.payload)
        .and_then(|input| {
            meter
                .record(input)
                .map_err(MeteringEventIngestAppError::Metering)
        })
        .and_then(|event| {
            let outbox_record = publish_outbox(outbox, &event)?;
            Ok(MeteringEventIngestSuccessResponse::accepted(
                metering_event_record(event, outbox_record),
                &envelope,
            ))
        });

    idempotency_ledger.entries.insert(
        key,
        MeteringEventIngestIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_envelope(
    envelope: &MeteringEventEnvelopeContext,
) -> Result<(), MeteringEventIngestAppError> {
    if envelope.event_id.trim().is_empty() {
        return Err(MeteringEventIngestAppError::EmptyEventId);
    }
    if envelope.source.trim().is_empty() {
        return Err(MeteringEventIngestAppError::EmptySource);
    }
    if envelope.source != METERING_EVENT_INGEST_SOURCE {
        return Err(MeteringEventIngestAppError::InvalidSource {
            source: envelope.source.clone(),
        });
    }
    if envelope.subject.trim().is_empty() {
        return Err(MeteringEventIngestAppError::EmptySubject);
    }
    if envelope.topic != METERING_EVENT_TOPIC {
        return Err(MeteringEventIngestAppError::InvalidTopic {
            topic: envelope.topic.clone(),
        });
    }
    if envelope.schema != METERING_EVENT_INGEST_SCHEMA {
        return Err(MeteringEventIngestAppError::InvalidSchema {
            schema: envelope.schema.clone(),
        });
    }
    if envelope.tenant_id.trim().is_empty() {
        return Err(MeteringEventIngestAppError::EmptyTenantId);
    }
    if envelope.producer_id.trim().is_empty() {
        return Err(MeteringEventIngestAppError::EmptyProducerId);
    }
    if envelope.idempotency_key.trim().is_empty() {
        return Err(MeteringEventIngestAppError::EmptyIdempotencyKey);
    }
    if envelope.produced_at_epoch_seconds == 0 {
        return Err(MeteringEventIngestAppError::InvalidProducedAt);
    }
    Ok(())
}

fn validate_authorization(
    envelope: &MeteringEventEnvelopeContext,
    authorization: &MeteringEventIngestAuthorization,
) -> Result<(), MeteringEventIngestAppError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(MeteringEventIngestAppError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != envelope.tenant_id {
        return Err(MeteringEventIngestAppError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            envelope_tenant_id: envelope.tenant_id.clone(),
        });
    }
    if authorization.producer_id != envelope.producer_id {
        return Err(MeteringEventIngestAppError::AuthorizationProducerMismatch {
            authorization_producer_id: authorization.producer_id.clone(),
            envelope_producer_id: envelope.producer_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == METERING_EVENT_INGEST_SURFACE)
    {
        return Err(MeteringEventIngestAppError::AuthorizationDenied {
            surface: METERING_EVENT_INGEST_SURFACE.to_string(),
        });
    }
    Ok(())
}

fn validate_payload_labels(
    payload: &MeteringEventIngestPayload,
) -> Result<(), MeteringEventIngestAppError> {
    parse_plane(payload.plane.clone())?;
    parse_source_axis(payload.source_axis.clone())?;
    for unit in &payload.units {
        let _ = parse_meter_unit_request(unit)?;
    }
    parse_public_data_class_label(payload.data_class.clone())?;
    Ok(())
}

fn validate_envelope_payload_binding(
    envelope: &MeteringEventEnvelopeContext,
    payload: &MeteringEventIngestPayload,
) -> Result<(), MeteringEventIngestAppError> {
    if envelope.event_id != payload.id {
        return Err(
            MeteringEventIngestAppError::EnvelopePayloadEventIdMismatch {
                envelope_event_id: envelope.event_id.clone(),
                payload_event_id: payload.id.clone(),
            },
        );
    }
    if envelope.tenant_id != payload.tenant_id {
        return Err(MeteringEventIngestAppError::EnvelopePayloadTenantMismatch {
            envelope_tenant_id: envelope.tenant_id.clone(),
            payload_tenant_id: payload.tenant_id.clone(),
        });
    }
    if envelope.idempotency_key != payload.idempotency_key {
        return Err(
            MeteringEventIngestAppError::EnvelopePayloadIdempotencyMismatch {
                envelope_idempotency_key: envelope.idempotency_key.clone(),
                payload_idempotency_key: payload.idempotency_key.clone(),
            },
        );
    }
    let expected_subject = expected_subject(payload);
    if envelope.subject != expected_subject {
        return Err(MeteringEventIngestAppError::SubjectMismatch {
            expected_subject,
            actual_subject: envelope.subject.clone(),
        });
    }
    Ok(())
}

fn metering_event_input(
    payload: MeteringEventIngestPayload,
) -> Result<MeterEventCreate, MeteringEventIngestAppError> {
    Ok(MeterEventCreate {
        id: payload.id,
        tenant_id: payload.tenant_id,
        capability_id: payload.capability_id,
        plane: parse_plane(payload.plane)?,
        units: payload
            .units
            .iter()
            .map(parse_meter_unit_request)
            .collect::<Result<Vec<_>, _>>()?,
        source_axis: parse_source_axis(payload.source_axis)?,
        recorded_at_epoch_seconds: payload.recorded_at_epoch_seconds,
        idempotency_key: payload.idempotency_key,
        data_class: parse_public_data_class_label(payload.data_class)?,
    })
}

fn parse_plane(label: String) -> Result<PlaneTag, MeteringEventIngestAppError> {
    match label.as_str() {
        "control" => Ok(PlaneTag::Control),
        "data" => Ok(PlaneTag::Data),
        "analytics" => Ok(PlaneTag::Analytics),
        "audit" => Ok(PlaneTag::Audit),
        _ => Err(MeteringEventIngestAppError::InvalidPlane { plane: label }),
    }
}

fn parse_source_axis(label: String) -> Result<AxisId, MeteringEventIngestAppError> {
    match label.as_str() {
        "saas" => Ok(AxisId::Saas),
        "foundry" => Ok(AxisId::Foundry),
        "cloud" => Ok(AxisId::Cloud),
        "search" => Ok(AxisId::Search),
        "ads" => Ok(AxisId::Ads),
        "marketplace" => Ok(AxisId::Marketplace),
        "vertical" => Ok(AxisId::Vertical),
        _ => Err(MeteringEventIngestAppError::InvalidSourceAxis { source_axis: label }),
    }
}

fn parse_meter_unit_request(
    unit: &MeteringMeterUnitRequest,
) -> Result<MeterUnit, MeteringEventIngestAppError> {
    let kind = parse_meter_unit_kind(unit.kind.clone())?;
    MeterUnit::new(kind, unit.quantity_microunits).map_err(|_| {
        MeteringEventIngestAppError::InvalidMeterUnitQuantity {
            kind: unit.kind.clone(),
            quantity_microunits: unit.quantity_microunits,
        }
    })
}

fn parse_meter_unit_kind(label: String) -> Result<MeterUnitKind, MeteringEventIngestAppError> {
    match label.as_str() {
        "request" => Ok(MeterUnitKind::Request),
        "byte_in" => Ok(MeterUnitKind::ByteIn),
        "byte_out" => Ok(MeterUnitKind::ByteOut),
        "millisecond" => Ok(MeterUnitKind::Millisecond),
        "gpu_second" => Ok(MeterUnitKind::GpuSecond),
        "llm_token" => Ok(MeterUnitKind::LlmToken),
        "resource_second" => Ok(MeterUnitKind::ResourceSecond),
        "storage_gb_second" => Ok(MeterUnitKind::StorageGbSecond),
        "egress_gb" => Ok(MeterUnitKind::EgressGb),
        _ => Err(MeteringEventIngestAppError::InvalidMeterUnitKind { kind: label }),
    }
}

fn parse_public_data_class_label(label: String) -> Result<DataClass, MeteringEventIngestAppError> {
    match parse_data_class_label(&label) {
        Some(DataClass::Public) => Ok(DataClass::Public),
        _ => Err(MeteringEventIngestAppError::InvalidDataClassLabel { data_class: label }),
    }
}

fn publish_outbox(
    outbox: &mut Outbox,
    event: &MeterEvent,
) -> Result<OutboxRecord, MeteringEventIngestAppError> {
    outbox
        .publish(
            event.tenant_id.value.clone(),
            METERING_EVENT_TOPIC.to_string(),
            event.idempotency_key.value.value.clone(),
            format!("meter-events/{}", event.id.value.value),
        )
        .map_err(MeteringEventIngestAppError::Eventing)
}

fn idempotency_key_for(
    envelope: &MeteringEventEnvelopeContext,
    surface: &str,
) -> MeteringEventIngestIdempotencyLedgerKey {
    MeteringEventIngestIdempotencyLedgerKey {
        tenant_id: envelope.tenant_id.clone(),
        producer_id: envelope.producer_id.clone(),
        surface: surface.to_string(),
        idempotency_key: envelope.idempotency_key.clone(),
    }
}

fn metering_event_ingest_fingerprint_for(
    request: &MeteringEventIngestAppRequest,
) -> MeteringEventIngestRequestFingerprint {
    let units = request
        .payload
        .units
        .iter()
        .map(|unit| format!("{}:{}", unit.kind, unit.quantity_microunits))
        .collect::<Vec<_>>()
        .join(",");
    MeteringEventIngestRequestFingerprint {
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
            format!("payload.id={}", request.payload.id),
            format!("payload.tenant_id={}", request.payload.tenant_id),
            format!("payload.capability_id={}", request.payload.capability_id),
            format!("payload.plane={}", request.payload.plane),
            format!("payload.units={units}"),
            format!("payload.source_axis={}", request.payload.source_axis),
            format!(
                "payload.recorded_at_epoch_seconds={}",
                request.payload.recorded_at_epoch_seconds
            ),
            format!(
                "payload.idempotency_key={}",
                request.payload.idempotency_key
            ),
            format!("payload.data_class={}", request.payload.data_class),
        ]
        .join("|"),
    }
}

fn metering_event_record(
    event: MeterEvent,
    outbox_record: OutboxRecord,
) -> MeteringEventIngestRecord {
    MeteringEventIngestRecord {
        id: event.id.value.value,
        tenant_id: event.tenant_id.value,
        capability_id: event.capability_id.value.value,
        plane: plane_label(event.plane.value).to_string(),
        units: event
            .units
            .value
            .units
            .iter()
            .map(|unit| MeteringMeterUnitRecord {
                kind: meter_unit_kind_label(unit.kind).to_string(),
                quantity_microunits: unit.quantity_microunits,
            })
            .collect(),
        source_axis: source_axis_label(event.source_axis.value).to_string(),
        recorded_at_epoch_seconds: event.recorded_at_epoch_seconds.value,
        idempotency_key: event.idempotency_key.value.value,
        data_class: event.data_class.value.label().to_string(),
        meter_schema_version: event.schema_version.value,
        outbox_sequence: outbox_record.sequence,
        outbox_topic: outbox_record.topic.value,
        outbox_idempotency_key: outbox_record.idempotency_key.value,
        outbox_payload_ref: outbox_record.payload_ref.value,
        outbox_published: outbox_record.published,
    }
}

fn expected_subject(payload: &MeteringEventIngestPayload) -> String {
    format!(
        "tenant/{}/capability/{}",
        payload.tenant_id, payload.capability_id
    )
}

fn plane_label(plane: PlaneTag) -> &'static str {
    match plane {
        PlaneTag::Control => "control",
        PlaneTag::Data => "data",
        PlaneTag::Analytics => "analytics",
        PlaneTag::Audit => "audit",
    }
}

fn source_axis_label(axis: AxisId) -> &'static str {
    match axis {
        AxisId::Saas => "saas",
        AxisId::Foundry => "foundry",
        AxisId::Cloud => "cloud",
        AxisId::Search => "search",
        AxisId::Ads => "ads",
        AxisId::Marketplace => "marketplace",
        AxisId::Vertical => "vertical",
    }
}

fn meter_unit_kind_label(kind: MeterUnitKind) -> &'static str {
    match kind {
        MeterUnitKind::Request => "request",
        MeterUnitKind::ByteIn => "byte_in",
        MeterUnitKind::ByteOut => "byte_out",
        MeterUnitKind::Millisecond => "millisecond",
        MeterUnitKind::GpuSecond => "gpu_second",
        MeterUnitKind::LlmToken => "llm_token",
        MeterUnitKind::ResourceSecond => "resource_second",
        MeterUnitKind::StorageGbSecond => "storage_gb_second",
        MeterUnitKind::EgressGb => "egress_gb",
    }
}
