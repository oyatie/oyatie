use oya_eventing_domain::Outbox;
use oya_metering_application::{
    METERING_EVENT_ASYNCAPI_CONTRACT, METERING_EVENT_INGEST_SCHEMA, METERING_EVENT_INGEST_SOURCE,
    METERING_EVENT_INGEST_SURFACE, METERING_EVENT_PROTO_CONTRACT, METERING_EVENT_TOPIC,
    MeteringEventEnvelopeContext, MeteringEventIngestAppError, MeteringEventIngestAppRequest,
    MeteringEventIngestAppStatus, MeteringEventIngestAuthorization,
    MeteringEventIngestIdempotencyLedger, MeteringEventIngestPayload, MeteringMeterUnitRequest,
    ingest_metering_event_from_app,
};
use oya_metering_domain::Meter;

const EVENT_ID: &str = "mtr_cloud_compute_001";
const IDEMPOTENCY_KEY: &str = "idem_metering_cloud_compute_001";
const CAPABILITY_ID: &str = "cap.cloud.compute.vm-hour";

#[test]
fn metering_event_contract_runtime_constants_are_covered() {
    assert_eq!(METERING_EVENT_INGEST_SURFACE, "metering.event.ingest");
    assert_eq!(METERING_EVENT_TOPIC, "oya.platform.metering");
    assert_eq!(METERING_EVENT_INGEST_SCHEMA, "metering.event.ingest.v1");
    assert_eq!(METERING_EVENT_INGEST_SOURCE, "oyatie://platform/metering");
    assert_eq!(
        METERING_EVENT_ASYNCAPI_CONTRACT,
        "contracts/asyncapi/platform/metering-events-v1.yaml"
    );
    assert_eq!(
        METERING_EVENT_PROTO_CONTRACT,
        "contracts/proto/platform/metering/v1/metering-event-v1.proto"
    );
    assert_eq!(MeteringEventIngestAppStatus::Accepted.code(), 202);
    assert_eq!(MeteringEventIngestAppStatus::BadRequest.code(), 400);
    assert_eq!(MeteringEventIngestAppStatus::Unauthorized.code(), 401);
    assert_eq!(MeteringEventIngestAppStatus::Forbidden.code(), 403);
    assert_eq!(MeteringEventIngestAppStatus::Conflict.code(), 409);
    assert_eq!(
        MeteringEventIngestAppStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn metering_event_ingest_records_meter_and_outbox_once_and_replays_idempotently() {
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = MeteringEventIngestIdempotencyLedger::default();
    let request = ingest_request(EVENT_ID, IDEMPOTENCY_KEY);

    let first =
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, request.clone())
            .expect("first ingest records meter event and outbox");
    let second = ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, request)
        .expect("same request fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(meter.events().count(), 1);
    assert_eq!(outbox.records().len(), 1);
    assert_eq!(first.data.id, EVENT_ID);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.capability_id, CAPABILITY_ID);
    assert_eq!(first.data.plane, "data");
    assert_eq!(first.data.source_axis, "cloud");
    assert_eq!(first.data.units[0].kind, "resource_second");
    assert_eq!(first.data.units[0].quantity_microunits, 3_600_000_000);
    assert_eq!(first.data.idempotency_key, IDEMPOTENCY_KEY);
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.meter_schema_version, 1);
    assert_eq!(first.data.outbox_topic, METERING_EVENT_TOPIC);
    assert_eq!(
        first.data.outbox_payload_ref,
        "meter-events/mtr_cloud_compute_001"
    );
    assert!(!first.data.outbox_published);
    assert_eq!(first.metadata.event_id, EVENT_ID);
    assert_eq!(first.metadata.producer_id, "producer_cloud_compute");
    assert_eq!(first.metadata.schema, METERING_EVENT_INGEST_SCHEMA);
}

#[test]
fn metering_event_ingest_rejects_envelope_payload_drift_before_kernel_or_outbox() {
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = MeteringEventIngestIdempotencyLedger::default();
    let mut tenant_drift = ingest_request(EVENT_ID, "idem_metering_drift_tenant");
    tenant_drift.payload.tenant_id = "ten_other".to_string();

    let tenant_error =
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, tenant_drift)
            .expect_err("tenant drift is rejected before meter/outbox mutation");
    assert!(matches!(
        tenant_error,
        MeteringEventIngestAppError::EnvelopePayloadTenantMismatch { .. }
    ));
    assert_eq!(
        tenant_error.metering_event_ingest_status(),
        MeteringEventIngestAppStatus::Forbidden
    );

    let mut idempotency_drift = ingest_request(EVENT_ID, "idem_metering_drift_idempotency");
    idempotency_drift.payload.idempotency_key = "idem_payload_other".to_string();
    let idempotency_error = ingest_metering_event_from_app(
        &mut meter,
        &mut outbox,
        &mut idempotency,
        idempotency_drift,
    )
    .expect_err("idempotency drift is rejected before meter/outbox mutation");
    assert!(matches!(
        idempotency_error,
        MeteringEventIngestAppError::EnvelopePayloadIdempotencyMismatch { .. }
    ));
    assert_eq!(idempotency_error.metering_event_ingest_status_code(), 400);

    assert!(idempotency.is_empty());
    assert_eq!(meter.events().count(), 0);
    assert!(outbox.records().is_empty());
}

#[test]
fn metering_event_ingest_separates_missing_producer_from_denied_authorization() {
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = MeteringEventIngestIdempotencyLedger::default();
    let mut unauthenticated = ingest_request(EVENT_ID, "idem_metering_authn");
    unauthenticated.envelope.producer_id.clear();

    let authn_error =
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, unauthenticated)
            .expect_err("missing producer is authentication failure");
    assert_eq!(
        authn_error.metering_event_ingest_status(),
        MeteringEventIngestAppStatus::Unauthorized
    );

    let mut denied = ingest_request(EVENT_ID, "idem_metering_authz");
    denied.authorization.allowed_surfaces = vec!["cloud.billing.event.ingest".to_string()];
    let authz_error =
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, denied)
            .expect_err("missing surface grant is authorization failure");
    assert!(matches!(
        authz_error,
        MeteringEventIngestAppError::AuthorizationDenied { ref surface }
            if surface == METERING_EVENT_INGEST_SURFACE
    ));
    assert_eq!(
        authz_error.metering_event_ingest_status(),
        MeteringEventIngestAppStatus::Forbidden
    );
    assert!(idempotency.is_empty());
    assert_eq!(meter.events().count(), 0);
    assert!(outbox.records().is_empty());
}

#[test]
fn metering_event_ingest_rejects_invalid_labels_and_units_before_mutation() {
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = MeteringEventIngestIdempotencyLedger::default();

    let mut invalid_plane = ingest_request(EVENT_ID, "idem_metering_bad_plane");
    invalid_plane.payload.plane = "compute".to_string();
    assert!(matches!(
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, invalid_plane),
        Err(MeteringEventIngestAppError::InvalidPlane { .. })
    ));

    let mut invalid_axis = ingest_request(EVENT_ID, "idem_metering_bad_axis");
    invalid_axis.payload.source_axis = "robotics".to_string();
    assert!(matches!(
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, invalid_axis),
        Err(MeteringEventIngestAppError::InvalidSourceAxis { .. })
    ));

    let mut invalid_unit = ingest_request(EVENT_ID, "idem_metering_bad_unit");
    invalid_unit.payload.units[0].kind = "banana".to_string();
    assert!(matches!(
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, invalid_unit),
        Err(MeteringEventIngestAppError::InvalidMeterUnitKind { .. })
    ));

    let mut invalid_class = ingest_request(EVENT_ID, "idem_metering_bad_class");
    invalid_class.payload.data_class = "INTERNAL_ONLY".to_string();
    let class_error =
        ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, invalid_class)
            .expect_err("metering metadata is public privacy metadata only");
    assert!(matches!(
        class_error,
        MeteringEventIngestAppError::InvalidDataClassLabel { .. }
    ));
    assert_eq!(class_error.metering_event_ingest_status_code(), 400);

    assert!(idempotency.is_empty());
    assert_eq!(meter.events().count(), 0);
    assert!(outbox.records().is_empty());
}

#[test]
fn metering_event_ingest_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = MeteringEventIngestIdempotencyLedger::default();
    let mut request = ingest_request(EVENT_ID, "idem_metering_reused");

    ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, request.clone())
        .expect("first idempotent request succeeds");

    request.payload.units[0].quantity_microunits = 7_200_000_000;
    let error = ingest_metering_event_from_app(&mut meter, &mut outbox, &mut idempotency, request)
        .expect_err("same idempotency key with changed payload is rejected");

    assert_eq!(
        error,
        MeteringEventIngestAppError::IdempotencyKeyReused {
            idempotency_key: "idem_metering_reused".to_string()
        }
    );
    assert_eq!(
        error.metering_event_ingest_status(),
        MeteringEventIngestAppStatus::UnprocessableEntity
    );
    assert_eq!(idempotency.len(), 1);
    assert_eq!(meter.events().count(), 1);
    assert_eq!(outbox.records().len(), 1);
}

#[test]
fn metering_event_ingest_maps_kernel_duplicate_id_to_conflict() {
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = MeteringEventIngestIdempotencyLedger::default();

    ingest_metering_event_from_app(
        &mut meter,
        &mut outbox,
        &mut idempotency,
        ingest_request(EVENT_ID, "idem_metering_duplicate_first"),
    )
    .expect("first event succeeds");

    let error = ingest_metering_event_from_app(
        &mut meter,
        &mut outbox,
        &mut idempotency,
        ingest_request(EVENT_ID, "idem_metering_duplicate_second"),
    )
    .expect_err("same meter event id with a new idempotency key is a conflict");

    assert_eq!(
        error.metering_event_ingest_status(),
        MeteringEventIngestAppStatus::Conflict
    );
    assert_eq!(meter.events().count(), 1);
    assert_eq!(outbox.records().len(), 1);
}

fn ingest_request(event_id: &str, idempotency_key: &str) -> MeteringEventIngestAppRequest {
    MeteringEventIngestAppRequest {
        envelope: MeteringEventEnvelopeContext {
            event_id: event_id.to_string(),
            source: METERING_EVENT_INGEST_SOURCE.to_string(),
            subject: format!("tenant/ten_kr/capability/{CAPABILITY_ID}"),
            topic: METERING_EVENT_TOPIC.to_string(),
            schema: METERING_EVENT_INGEST_SCHEMA.to_string(),
            tenant_id: "ten_kr".to_string(),
            producer_id: "producer_cloud_compute".to_string(),
            idempotency_key: idempotency_key.to_string(),
            produced_at_epoch_seconds: 1_700_000_000,
        },
        authorization: MeteringEventIngestAuthorization {
            tenant_id: "ten_kr".to_string(),
            producer_id: "producer_cloud_compute".to_string(),
            decision_id: "authz_metering_event_ingest".to_string(),
            allowed_surfaces: vec![METERING_EVENT_INGEST_SURFACE.to_string()],
        },
        payload: MeteringEventIngestPayload {
            id: event_id.to_string(),
            tenant_id: "ten_kr".to_string(),
            capability_id: CAPABILITY_ID.to_string(),
            plane: "data".to_string(),
            units: vec![MeteringMeterUnitRequest {
                kind: "resource_second".to_string(),
                quantity_microunits: 3_600_000_000,
            }],
            source_axis: "cloud".to_string(),
            recorded_at_epoch_seconds: 1_700_000_000,
            idempotency_key: idempotency_key.to_string(),
            data_class: "PUBLIC".to_string(),
        },
    }
}
