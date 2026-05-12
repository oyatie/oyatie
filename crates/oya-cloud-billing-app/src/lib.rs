//! Cloud Billing event app boundary.
//!
//! This crate owns CloudEvents envelope normalization, producer authorization,
//! request fingerprint idempotency, Cloud billing kernel projection, platform
//! metering, and eventing outbox publication for `cloud.billing.event.ingest`.

use std::collections::BTreeMap;

use oya_cloud_billing_kernel::{
    CloudBillingError, CloudBillingEvent, CloudBillingEventCreate, CloudBillingEventKind,
    CloudBillingLedger,
};
use oya_platform_data_boundary_kernel::{parse_data_class_label, DataClass};
use oya_platform_eventing_kernel::{EventingError, Outbox, OutboxRecord};
use oya_platform_metering_kernel::{Meter, MeterEvent, MeterUnit, MeterUnitKind, MeteringError};

pub const CLOUD_BILLING_EVENT_INGEST_SURFACE: &str = "cloud.billing.event.ingest";
pub const CLOUD_BILLING_EVENT_TOPIC: &str = "oya.cloud.billing";
pub const CLOUD_BILLING_EVENT_INGEST_SCHEMA: &str = "cloud.billing.event.ingest.v1";
pub const CLOUD_BILLING_EVENT_INGEST_SOURCE: &str = "oyatie://cloud/billing";
pub const CLOUD_BILLING_EVENT_ASYNCAPI_CONTRACT: &str =
    "contracts/asyncapi/cloud/cloud-billing-events-v1.yaml";
pub const CLOUD_BILLING_EVENT_PROTO_CONTRACT: &str =
    "contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudBillingEventIngestAppStatus {
    Accepted,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudBillingEventIngestAppStatus {
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
pub struct CloudBillingEventEnvelopeContext {
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
pub struct CloudBillingEventIngestAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub producer_id: String,           // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingMeterUnitRequest {
    pub kind: String,             // data_class: INTERNAL_ONLY
    pub quantity_microunits: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingEventIngestPayload {
    pub id: String,                               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub resource_id: String,                      // data_class: INTERNAL_ONLY
    pub region: String,                           // data_class: PUBLIC
    pub metering_tag: String,                     // data_class: INTERNAL_ONLY
    pub kind: String,                             // data_class: INTERNAL_ONLY
    pub units: Vec<CloudBillingMeterUnitRequest>, // data_class: INTERNAL_ONLY
    pub rate_card_ref: String,                    // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                  // data_class: INTERNAL_ONLY
    pub data_class: String,                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingEventIngestAppRequest {
    pub envelope: CloudBillingEventEnvelopeContext, // data_class: INTERNAL_ONLY
    pub authorization: CloudBillingEventIngestAuthorization, // data_class: INTERNAL_ONLY
    pub payload: CloudBillingEventIngestPayload,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudBillingEventIngestIdempotencyLedger {
    entries: BTreeMap<
        CloudBillingEventIngestIdempotencyLedgerKey,
        CloudBillingEventIngestIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl CloudBillingEventIngestIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudBillingEventIngestIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    producer_id: String,     // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudBillingEventIngestIdempotencyLedgerEntry {
    fingerprint: CloudBillingEventIngestRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudBillingEventIngestAppResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudBillingEventIngestRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudBillingEventIngestAppResult =
    Result<CloudBillingEventIngestSuccessResponse, CloudBillingEventIngestAppError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingEventIngestSuccessResponse {
    pub data: CloudBillingEventIngestRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudBillingEventIngestMetadata, // data_class: INTERNAL_ONLY
}

impl CloudBillingEventIngestSuccessResponse {
    pub fn accepted(
        data: CloudBillingEventIngestRecord,
        envelope: &CloudBillingEventEnvelopeContext,
    ) -> Self {
        Self {
            data,
            metadata: CloudBillingEventIngestMetadata {
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
pub struct CloudBillingEventIngestMetadata {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub producer_id: String,            // data_class: INTERNAL_ONLY
    pub topic: String,                  // data_class: INTERNAL_ONLY
    pub schema: String,                 // data_class: INTERNAL_ONLY
    pub produced_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingMeterUnitRecord {
    pub kind: String,             // data_class: INTERNAL_ONLY
    pub quantity_microunits: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingEventIngestRecord {
    pub id: String,                              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub resource_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                          // data_class: PUBLIC
    pub metering_tag: String,                    // data_class: INTERNAL_ONLY
    pub kind: String,                            // data_class: INTERNAL_ONLY
    pub units: Vec<CloudBillingMeterUnitRecord>, // data_class: INTERNAL_ONLY
    pub rate_card_ref: String,                   // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                 // data_class: INTERNAL_ONLY
    pub data_class: String,                      // data_class: PUBLIC
    pub billing_event_schema_version: u32,       // data_class: PUBLIC
    pub meter_event_id: String,                  // data_class: INTERNAL_ONLY
    pub meter_capability_id: String,             // data_class: INTERNAL_ONLY
    pub meter_recorded_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub meter_schema_version: u32,               // data_class: PUBLIC
    pub outbox_sequence: u64,                    // data_class: INTERNAL_ONLY
    pub outbox_topic: String,                    // data_class: INTERNAL_ONLY
    pub outbox_idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub outbox_payload_ref: String,              // data_class: INTERNAL_ONLY
    pub outbox_published: bool,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudBillingEventIngestAppError {
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
    InvalidEventKind {
        kind: String,
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
    Billing(CloudBillingError),
    Eventing(EventingError),
}

impl CloudBillingEventIngestAppError {
    pub fn billing_event_ingest_status(&self) -> CloudBillingEventIngestAppStatus {
        match self {
            Self::EmptyProducerId => CloudBillingEventIngestAppStatus::Unauthorized,
            Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationProducerMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::EnvelopePayloadTenantMismatch { .. } => {
                CloudBillingEventIngestAppStatus::Forbidden
            }
            Self::IdempotencyKeyReused { .. } => {
                CloudBillingEventIngestAppStatus::UnprocessableEntity
            }
            Self::Billing(error) => billing_error_status(error),
            Self::Eventing(_) => CloudBillingEventIngestAppStatus::UnprocessableEntity,
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
            | Self::InvalidEventKind { .. }
            | Self::InvalidMeterUnitKind { .. }
            | Self::InvalidMeterUnitQuantity { .. }
            | Self::InvalidDataClassLabel { .. } => CloudBillingEventIngestAppStatus::BadRequest,
        }
    }

    pub fn billing_event_ingest_status_code(&self) -> u16 {
        self.billing_event_ingest_status().code()
    }
}

pub fn validate_cloud_billing_event_ingest_request(
    request: &CloudBillingEventIngestAppRequest,
) -> Result<(), CloudBillingEventIngestAppError> {
    validate_envelope(&request.envelope)?;
    validate_authorization(&request.envelope, &request.authorization)?;
    validate_envelope_payload_binding(&request.envelope, &request.payload)?;
    validate_payload_labels(&request.payload)
}

pub fn ingest_cloud_billing_event_from_app(
    ledger: &mut CloudBillingLedger,
    meter: &mut Meter,
    outbox: &mut Outbox,
    idempotency_ledger: &mut CloudBillingEventIngestIdempotencyLedger,
    request: CloudBillingEventIngestAppRequest,
) -> Result<CloudBillingEventIngestSuccessResponse, CloudBillingEventIngestAppError> {
    validate_cloud_billing_event_ingest_request(&request)?;
    let key = idempotency_key_for(&request.envelope, CLOUD_BILLING_EVENT_INGEST_SURFACE);
    let fingerprint = billing_event_ingest_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudBillingEventIngestAppError::IdempotencyKeyReused {
            idempotency_key: request.envelope.idempotency_key,
        });
    }

    let envelope = request.envelope.clone();
    let result = billing_event_input(request.payload)
        .and_then(|input| {
            ledger
                .ingest(meter, input)
                .map_err(CloudBillingEventIngestAppError::Billing)
        })
        .and_then(|(event, meter_event)| {
            let outbox_record = publish_outbox(outbox, &event)?;
            Ok(CloudBillingEventIngestSuccessResponse::accepted(
                billing_event_record(event, meter_event, outbox_record),
                &envelope,
            ))
        });

    idempotency_ledger.entries.insert(
        key,
        CloudBillingEventIngestIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_envelope(
    envelope: &CloudBillingEventEnvelopeContext,
) -> Result<(), CloudBillingEventIngestAppError> {
    if envelope.event_id.trim().is_empty() {
        return Err(CloudBillingEventIngestAppError::EmptyEventId);
    }
    if envelope.source.trim().is_empty() {
        return Err(CloudBillingEventIngestAppError::EmptySource);
    }
    if envelope.source != CLOUD_BILLING_EVENT_INGEST_SOURCE {
        return Err(CloudBillingEventIngestAppError::InvalidSource {
            source: envelope.source.clone(),
        });
    }
    if envelope.subject.trim().is_empty() {
        return Err(CloudBillingEventIngestAppError::EmptySubject);
    }
    if envelope.topic != CLOUD_BILLING_EVENT_TOPIC {
        return Err(CloudBillingEventIngestAppError::InvalidTopic {
            topic: envelope.topic.clone(),
        });
    }
    if envelope.schema != CLOUD_BILLING_EVENT_INGEST_SCHEMA {
        return Err(CloudBillingEventIngestAppError::InvalidSchema {
            schema: envelope.schema.clone(),
        });
    }
    if envelope.tenant_id.trim().is_empty() {
        return Err(CloudBillingEventIngestAppError::EmptyTenantId);
    }
    if envelope.producer_id.trim().is_empty() {
        return Err(CloudBillingEventIngestAppError::EmptyProducerId);
    }
    if envelope.idempotency_key.trim().is_empty() {
        return Err(CloudBillingEventIngestAppError::EmptyIdempotencyKey);
    }
    if envelope.produced_at_epoch_seconds == 0 {
        return Err(CloudBillingEventIngestAppError::InvalidProducedAt);
    }
    Ok(())
}

fn validate_authorization(
    envelope: &CloudBillingEventEnvelopeContext,
    authorization: &CloudBillingEventIngestAuthorization,
) -> Result<(), CloudBillingEventIngestAppError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudBillingEventIngestAppError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != envelope.tenant_id {
        return Err(
            CloudBillingEventIngestAppError::AuthorizationTenantMismatch {
                authorization_tenant_id: authorization.tenant_id.clone(),
                envelope_tenant_id: envelope.tenant_id.clone(),
            },
        );
    }
    if authorization.producer_id != envelope.producer_id {
        return Err(
            CloudBillingEventIngestAppError::AuthorizationProducerMismatch {
                authorization_producer_id: authorization.producer_id.clone(),
                envelope_producer_id: envelope.producer_id.clone(),
            },
        );
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == CLOUD_BILLING_EVENT_INGEST_SURFACE)
    {
        return Err(CloudBillingEventIngestAppError::AuthorizationDenied {
            surface: CLOUD_BILLING_EVENT_INGEST_SURFACE.to_string(),
        });
    }
    Ok(())
}

fn validate_envelope_payload_binding(
    envelope: &CloudBillingEventEnvelopeContext,
    payload: &CloudBillingEventIngestPayload,
) -> Result<(), CloudBillingEventIngestAppError> {
    if envelope.event_id != payload.id {
        return Err(CloudBillingEventIngestAppError::SubjectMismatch {
            expected_subject: payload.id.clone(),
            actual_subject: envelope.event_id.clone(),
        });
    }
    if envelope.tenant_id != payload.tenant_id {
        return Err(
            CloudBillingEventIngestAppError::EnvelopePayloadTenantMismatch {
                envelope_tenant_id: envelope.tenant_id.clone(),
                payload_tenant_id: payload.tenant_id.clone(),
            },
        );
    }
    if envelope.idempotency_key != payload.idempotency_key {
        return Err(
            CloudBillingEventIngestAppError::EnvelopePayloadIdempotencyMismatch {
                envelope_idempotency_key: envelope.idempotency_key.clone(),
                payload_idempotency_key: payload.idempotency_key.clone(),
            },
        );
    }
    let expected_subject = expected_subject(payload);
    if envelope.subject != expected_subject {
        return Err(CloudBillingEventIngestAppError::SubjectMismatch {
            expected_subject,
            actual_subject: envelope.subject.clone(),
        });
    }
    Ok(())
}

fn validate_payload_labels(
    payload: &CloudBillingEventIngestPayload,
) -> Result<(), CloudBillingEventIngestAppError> {
    parse_billing_event_kind(payload.kind.clone())?;
    parse_public_data_class_label(payload.data_class.clone())?;
    for unit in &payload.units {
        let _ = parse_meter_unit_request(unit)?;
    }
    Ok(())
}

fn billing_event_input(
    payload: CloudBillingEventIngestPayload,
) -> Result<CloudBillingEventCreate, CloudBillingEventIngestAppError> {
    Ok(CloudBillingEventCreate {
        id: payload.id,
        tenant_id: payload.tenant_id,
        resource_id: payload.resource_id,
        region: payload.region,
        metering_tag: payload.metering_tag,
        kind: parse_billing_event_kind(payload.kind)?,
        units: payload
            .units
            .iter()
            .map(parse_meter_unit_request)
            .collect::<Result<Vec<_>, _>>()?,
        rate_card_ref: payload.rate_card_ref,
        occurred_at_epoch_seconds: payload.occurred_at_epoch_seconds,
        idempotency_key: payload.idempotency_key,
        data_class: parse_public_data_class_label(payload.data_class)?,
    })
}

fn parse_billing_event_kind(
    label: String,
) -> Result<CloudBillingEventKind, CloudBillingEventIngestAppError> {
    match label.as_str() {
        "resource_created" => Ok(CloudBillingEventKind::ResourceCreated),
        "resource_terminated" => Ok(CloudBillingEventKind::ResourceTerminated),
        "usage" => Ok(CloudBillingEventKind::Usage),
        "reservation" => Ok(CloudBillingEventKind::Reservation),
        "commitment" => Ok(CloudBillingEventKind::Commitment),
        "credit" => Ok(CloudBillingEventKind::Credit),
        _ => Err(CloudBillingEventIngestAppError::InvalidEventKind { kind: label }),
    }
}

fn parse_meter_unit_request(
    unit: &CloudBillingMeterUnitRequest,
) -> Result<MeterUnit, CloudBillingEventIngestAppError> {
    let kind = parse_meter_unit_kind(unit.kind.clone())?;
    MeterUnit::new(kind, unit.quantity_microunits).map_err(|_| {
        CloudBillingEventIngestAppError::InvalidMeterUnitQuantity {
            kind: unit.kind.clone(),
            quantity_microunits: unit.quantity_microunits,
        }
    })
}

fn parse_meter_unit_kind(label: String) -> Result<MeterUnitKind, CloudBillingEventIngestAppError> {
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
        _ => Err(CloudBillingEventIngestAppError::InvalidMeterUnitKind { kind: label }),
    }
}

fn parse_public_data_class_label(
    label: String,
) -> Result<DataClass, CloudBillingEventIngestAppError> {
    match parse_data_class_label(&label) {
        Some(DataClass::Public) => Ok(DataClass::Public),
        _ => Err(CloudBillingEventIngestAppError::InvalidDataClassLabel { data_class: label }),
    }
}

fn publish_outbox(
    outbox: &mut Outbox,
    event: &CloudBillingEvent,
) -> Result<OutboxRecord, CloudBillingEventIngestAppError> {
    outbox
        .publish(
            event.tenant_id.value.clone(),
            CLOUD_BILLING_EVENT_TOPIC.to_string(),
            event.idempotency_key.value.clone(),
            format!("cloud-billing-events/{}", event.id.value.value),
        )
        .map_err(CloudBillingEventIngestAppError::Eventing)
}

fn idempotency_key_for(
    envelope: &CloudBillingEventEnvelopeContext,
    surface: &str,
) -> CloudBillingEventIngestIdempotencyLedgerKey {
    CloudBillingEventIngestIdempotencyLedgerKey {
        tenant_id: envelope.tenant_id.clone(),
        producer_id: envelope.producer_id.clone(),
        surface: surface.to_string(),
        idempotency_key: envelope.idempotency_key.clone(),
    }
}

fn billing_event_ingest_fingerprint_for(
    request: &CloudBillingEventIngestAppRequest,
) -> CloudBillingEventIngestRequestFingerprint {
    let units = request
        .payload
        .units
        .iter()
        .map(|unit| format!("{}:{}", unit.kind, unit.quantity_microunits))
        .collect::<Vec<_>>()
        .join(",");
    CloudBillingEventIngestRequestFingerprint {
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
            format!("payload.resource_id={}", request.payload.resource_id),
            format!("payload.region={}", request.payload.region),
            format!("payload.metering_tag={}", request.payload.metering_tag),
            format!("payload.kind={}", request.payload.kind),
            format!("payload.units={units}"),
            format!("payload.rate_card_ref={}", request.payload.rate_card_ref),
            format!(
                "payload.occurred_at_epoch_seconds={}",
                request.payload.occurred_at_epoch_seconds
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

fn billing_event_record(
    event: CloudBillingEvent,
    meter_event: MeterEvent,
    outbox_record: OutboxRecord,
) -> CloudBillingEventIngestRecord {
    CloudBillingEventIngestRecord {
        id: event.id.value.value,
        tenant_id: event.tenant_id.value,
        resource_id: event.resource_id.value.value,
        region: event.region.value.value,
        metering_tag: event.metering_tag.value,
        kind: billing_event_kind_label(event.kind.value).to_string(),
        units: event
            .units
            .value
            .iter()
            .map(|unit| CloudBillingMeterUnitRecord {
                kind: meter_unit_kind_label(unit.kind).to_string(),
                quantity_microunits: unit.quantity_microunits,
            })
            .collect(),
        rate_card_ref: event.rate_card_ref.value.value,
        occurred_at_epoch_seconds: event.occurred_at_epoch_seconds.value,
        idempotency_key: event.idempotency_key.value,
        data_class: event.data_class.value.data_class().label().to_string(),
        billing_event_schema_version: event.schema_version.value,
        meter_event_id: meter_event.id.value.value,
        meter_capability_id: meter_event.capability_id.value.value,
        meter_recorded_at_epoch_seconds: meter_event.recorded_at_epoch_seconds.value,
        meter_schema_version: meter_event.schema_version.value,
        outbox_sequence: outbox_record.sequence,
        outbox_topic: outbox_record.topic.value,
        outbox_idempotency_key: outbox_record.idempotency_key.value,
        outbox_payload_ref: outbox_record.payload_ref.value,
        outbox_published: outbox_record.published,
    }
}

fn expected_subject(payload: &CloudBillingEventIngestPayload) -> String {
    format!(
        "tenant/{}/resource/{}",
        payload.tenant_id, payload.resource_id
    )
}

fn billing_event_kind_label(kind: CloudBillingEventKind) -> &'static str {
    match kind {
        CloudBillingEventKind::ResourceCreated => "resource_created",
        CloudBillingEventKind::ResourceTerminated => "resource_terminated",
        CloudBillingEventKind::Usage => "usage",
        CloudBillingEventKind::Reservation => "reservation",
        CloudBillingEventKind::Commitment => "commitment",
        CloudBillingEventKind::Credit => "credit",
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

fn billing_error_status(error: &CloudBillingError) -> CloudBillingEventIngestAppStatus {
    match error {
        CloudBillingError::TenantMismatch | CloudBillingError::RegionMismatch => {
            CloudBillingEventIngestAppStatus::Forbidden
        }
        CloudBillingError::DuplicateBillingEvent
        | CloudBillingError::MeteringRejected(MeteringError::DuplicateMeterEvent) => {
            CloudBillingEventIngestAppStatus::Conflict
        }
        CloudBillingError::InvalidBillingAccountId
        | CloudBillingError::InvalidCloudBillingEventId
        | CloudBillingError::InvalidInvoiceId
        | CloudBillingError::InvalidInvoiceLineItemId
        | CloudBillingError::InvalidTaxRegistrationId
        | CloudBillingError::InvalidTenantId
        | CloudBillingError::InvalidPaymentMethodRef
        | CloudBillingError::InvalidRateCardRef
        | CloudBillingError::InvalidRegionalPack
        | CloudBillingError::InvalidCurrencyCode
        | CloudBillingError::InvalidResourceId
        | CloudBillingError::InvalidMeteringTag
        | CloudBillingError::InvalidOccurredAt
        | CloudBillingError::InvalidBillingPeriod
        | CloudBillingError::InvalidInvoiceLineItem
        | CloudBillingError::InvalidInvoiceTotal
        | CloudBillingError::InvalidTaxInvoiceFormat
        | CloudBillingError::InvalidDataClass
        | CloudBillingError::BillingAccountInactive
        | CloudBillingError::DuplicateInvoice
        | CloudBillingError::MeteringRejected(_) => CloudBillingEventIngestAppStatus::BadRequest,
    }
}
