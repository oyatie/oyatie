use oya_cloud_billing_app::{
    ingest_cloud_billing_event_from_app, CloudBillingEventEnvelopeContext,
    CloudBillingEventIngestAppError, CloudBillingEventIngestAppRequest,
    CloudBillingEventIngestAppStatus, CloudBillingEventIngestAuthorization,
    CloudBillingEventIngestIdempotencyLedger, CloudBillingEventIngestPayload,
    CloudBillingMeterUnitRequest, CLOUD_BILLING_EVENT_ASYNCAPI_CONTRACT,
    CLOUD_BILLING_EVENT_INGEST_SCHEMA, CLOUD_BILLING_EVENT_INGEST_SOURCE,
    CLOUD_BILLING_EVENT_INGEST_SURFACE, CLOUD_BILLING_EVENT_PROTO_CONTRACT,
    CLOUD_BILLING_EVENT_TOPIC,
};
use oya_cloud_billing_kernel::CloudBillingLedger;
use oya_platform_eventing_kernel::Outbox;
use oya_platform_metering_kernel::Meter;

const EVENT_ID: &str = "cbill_ten_kr_resource_created_001";
const RESOURCE_ID: &str = "oya:cloud:kr-seoul:ten_kr:instance:api-001";
const IDEMPOTENCY_KEY: &str = "idem_cloud_billing_event_001";

#[test]
fn event_contract_runtime_constants_are_covered() {
    assert_eq!(
        CLOUD_BILLING_EVENT_INGEST_SURFACE,
        "cloud.billing.event.ingest"
    );
    assert_eq!(CLOUD_BILLING_EVENT_TOPIC, "oya.cloud.billing");
    assert_eq!(
        CLOUD_BILLING_EVENT_INGEST_SCHEMA,
        "cloud.billing.event.ingest.v1"
    );
    assert_eq!(CLOUD_BILLING_EVENT_INGEST_SOURCE, "oyatie://cloud/billing");
    assert_eq!(
        CLOUD_BILLING_EVENT_ASYNCAPI_CONTRACT,
        "contracts/asyncapi/cloud/cloud-billing-events-v1.yaml"
    );
    assert_eq!(
        CLOUD_BILLING_EVENT_PROTO_CONTRACT,
        "contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto"
    );
    assert_eq!(CloudBillingEventIngestAppStatus::Accepted.code(), 202);
    assert_eq!(CloudBillingEventIngestAppStatus::BadRequest.code(), 400);
    assert_eq!(CloudBillingEventIngestAppStatus::Unauthorized.code(), 401);
    assert_eq!(CloudBillingEventIngestAppStatus::Forbidden.code(), 403);
    assert_eq!(CloudBillingEventIngestAppStatus::Conflict.code(), 409);
    assert_eq!(
        CloudBillingEventIngestAppStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn billing_event_ingest_records_event_meter_and_outbox_once_and_replays_idempotently() {
    let mut ledger = CloudBillingLedger::default();
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = CloudBillingEventIngestIdempotencyLedger::default();
    let request = ingest_request(EVENT_ID, IDEMPOTENCY_KEY);

    let first = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        request.clone(),
    )
    .expect("first event ingestion succeeds");
    let second = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        request,
    )
    .expect("same request fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(ledger.events().count(), 1);
    assert_eq!(meter.events().count(), 1);
    assert_eq!(outbox.records().len(), 1);
    assert_eq!(first.data.id, EVENT_ID);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.resource_id, RESOURCE_ID);
    assert_eq!(first.data.region, "kr-seoul");
    assert_eq!(first.data.metering_tag, "oya:metering:ten_kr:instance");
    assert_eq!(first.data.kind, "resource_created");
    assert_eq!(first.data.units.len(), 1);
    assert_eq!(first.data.units[0].kind, "resource_second");
    assert_eq!(first.data.units[0].quantity_microunits, 3_600_000_000);
    assert_eq!(first.data.rate_card_ref, "rate/cloud/kr-seoul/standard");
    assert_eq!(first.data.occurred_at_epoch_seconds, 1_700_000_000);
    assert_eq!(first.data.idempotency_key, IDEMPOTENCY_KEY);
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.billing_event_schema_version, 1);
    assert_eq!(first.data.meter_event_id, "mtr_ten_kr_resource_created_001");
    assert_eq!(
        first.data.meter_capability_id,
        "cap.cloud.billing.resource-lifecycle"
    );
    assert_eq!(first.data.meter_recorded_at_epoch_seconds, 1_700_000_000);
    assert_eq!(first.data.meter_schema_version, 1);
    assert_eq!(first.data.outbox_sequence, 0);
    assert_eq!(first.data.outbox_topic, CLOUD_BILLING_EVENT_TOPIC);
    assert_eq!(first.data.outbox_idempotency_key, IDEMPOTENCY_KEY);
    assert_eq!(
        first.data.outbox_payload_ref,
        "cloud-billing-events/cbill_ten_kr_resource_created_001"
    );
    assert!(!first.data.outbox_published);
    assert_eq!(first.metadata.tenant_id, "ten_kr");
    assert_eq!(
        first.metadata.producer_id,
        "producer_cloud_resource_lifecycle"
    );
    assert_eq!(first.metadata.schema, CLOUD_BILLING_EVENT_INGEST_SCHEMA);
}

#[test]
fn billing_event_ingest_rejects_envelope_payload_drift_before_kernel_or_outbox() {
    let mut ledger = CloudBillingLedger::default();
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = CloudBillingEventIngestIdempotencyLedger::default();
    let mut tenant_drift = ingest_request(EVENT_ID, "idem_cloud_billing_event_drift_tenant");
    tenant_drift.payload.tenant_id = "ten_other".to_string();

    let tenant_error = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        tenant_drift,
    )
    .expect_err("tenant drift is rejected");
    assert!(matches!(
        tenant_error,
        CloudBillingEventIngestAppError::EnvelopePayloadTenantMismatch { .. }
    ));
    assert_eq!(
        tenant_error.billing_event_ingest_status(),
        CloudBillingEventIngestAppStatus::Forbidden
    );

    let mut idempotency_drift = ingest_request(EVENT_ID, "idem_cloud_billing_event_drift_idem");
    idempotency_drift.payload.idempotency_key = "idem_payload_other".to_string();
    let idempotency_error = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        idempotency_drift,
    )
    .expect_err("idempotency drift is rejected");
    assert!(matches!(
        idempotency_error,
        CloudBillingEventIngestAppError::EnvelopePayloadIdempotencyMismatch { .. }
    ));
    assert_eq!(idempotency_error.billing_event_ingest_status_code(), 400);

    assert!(idempotency.is_empty());
    assert_eq!(ledger.events().count(), 0);
    assert_eq!(meter.events().count(), 0);
    assert!(outbox.records().is_empty());
}

#[test]
fn billing_event_ingest_separates_missing_producer_from_denied_authorization() {
    let mut ledger = CloudBillingLedger::default();
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = CloudBillingEventIngestIdempotencyLedger::default();
    let mut unauthenticated = ingest_request(EVENT_ID, "idem_cloud_billing_event_authn");
    unauthenticated.envelope.producer_id.clear();

    let authn_error = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        unauthenticated,
    )
    .expect_err("missing producer is authentication failure");
    assert_eq!(
        authn_error.billing_event_ingest_status(),
        CloudBillingEventIngestAppStatus::Unauthorized
    );

    let mut denied = ingest_request(EVENT_ID, "idem_cloud_billing_event_authz");
    denied.authorization.allowed_surfaces = vec!["cloud.billing.invoice.generate".to_string()];
    let authz_error = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        denied,
    )
    .expect_err("missing surface grant is authorization failure");
    assert!(matches!(
        authz_error,
        CloudBillingEventIngestAppError::AuthorizationDenied { ref surface }
            if surface == CLOUD_BILLING_EVENT_INGEST_SURFACE
    ));
    assert_eq!(
        authz_error.billing_event_ingest_status(),
        CloudBillingEventIngestAppStatus::Forbidden
    );
    assert!(idempotency.is_empty());
    assert_eq!(ledger.events().count(), 0);
    assert!(outbox.records().is_empty());
}

#[test]
fn billing_event_ingest_rejects_invalid_kind_unit_and_data_class_before_kernel() {
    let mut ledger = CloudBillingLedger::default();
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = CloudBillingEventIngestIdempotencyLedger::default();

    let mut invalid_kind = ingest_request(EVENT_ID, "idem_cloud_billing_event_bad_kind");
    invalid_kind.payload.kind = "lifecycle".to_string();
    assert!(matches!(
        ingest_cloud_billing_event_from_app(
            &mut ledger,
            &mut meter,
            &mut outbox,
            &mut idempotency,
            invalid_kind,
        ),
        Err(CloudBillingEventIngestAppError::InvalidEventKind { .. })
    ));

    let mut invalid_unit = ingest_request(EVENT_ID, "idem_cloud_billing_event_bad_unit");
    invalid_unit.payload.units[0].kind = "cpu_second".to_string();
    assert!(matches!(
        ingest_cloud_billing_event_from_app(
            &mut ledger,
            &mut meter,
            &mut outbox,
            &mut idempotency,
            invalid_unit,
        ),
        Err(CloudBillingEventIngestAppError::InvalidMeterUnitKind { .. })
    ));

    let mut invalid_class = ingest_request(EVENT_ID, "idem_cloud_billing_event_bad_class");
    invalid_class.payload.data_class = "INTERNAL_ONLY".to_string();
    let class_error = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        invalid_class,
    )
    .expect_err("event metadata must stay public before kernel projection");
    assert!(matches!(
        class_error,
        CloudBillingEventIngestAppError::InvalidDataClassLabel { .. }
    ));
    assert_eq!(class_error.billing_event_ingest_status_code(), 400);

    assert!(idempotency.is_empty());
    assert_eq!(ledger.events().count(), 0);
    assert_eq!(meter.events().count(), 0);
    assert!(outbox.records().is_empty());
}

#[test]
fn billing_event_ingest_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut ledger = CloudBillingLedger::default();
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = CloudBillingEventIngestIdempotencyLedger::default();
    let mut request = ingest_request(EVENT_ID, "idem_cloud_billing_event_reused");

    ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        request.clone(),
    )
    .expect("first idempotent request succeeds");

    request.payload.rate_card_ref = "rate/cloud/kr-seoul/premium".to_string();
    let error = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        request,
    )
    .expect_err("same idempotency key with changed payload is rejected");

    assert_eq!(
        error,
        CloudBillingEventIngestAppError::IdempotencyKeyReused {
            idempotency_key: "idem_cloud_billing_event_reused".to_string()
        }
    );
    assert_eq!(
        error.billing_event_ingest_status(),
        CloudBillingEventIngestAppStatus::UnprocessableEntity
    );
    assert_eq!(idempotency.len(), 1);
    assert_eq!(ledger.events().count(), 1);
    assert_eq!(outbox.records().len(), 1);
}

#[test]
fn billing_event_ingest_maps_kernel_duplicate_and_scope_errors() {
    let mut ledger = CloudBillingLedger::default();
    let mut meter = Meter::default();
    let mut outbox = Outbox::default();
    let mut idempotency = CloudBillingEventIngestIdempotencyLedger::default();

    ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        ingest_request(EVENT_ID, "idem_cloud_billing_event_first"),
    )
    .expect("first event succeeds");

    let duplicate = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        ingest_request(EVENT_ID, "idem_cloud_billing_event_duplicate"),
    )
    .expect_err("same billing event id through a new idempotency key conflicts");
    assert!(matches!(
        duplicate,
        CloudBillingEventIngestAppError::Billing(_)
    ));
    assert_eq!(
        duplicate.billing_event_ingest_status(),
        CloudBillingEventIngestAppStatus::Conflict
    );

    let mut tenant_mismatch = ingest_request(
        "cbill_ten_kr_resource_created_002",
        "idem_cloud_billing_event_scope",
    );
    tenant_mismatch.payload.resource_id =
        "oya:cloud:kr-seoul:ten_other:instance:api-002".to_string();
    tenant_mismatch.envelope.subject = format!(
        "tenant/ten_kr/resource/{}",
        tenant_mismatch.payload.resource_id
    );
    let scoped = ingest_cloud_billing_event_from_app(
        &mut ledger,
        &mut meter,
        &mut outbox,
        &mut idempotency,
        tenant_mismatch,
    )
    .expect_err("kernel rejects resource tenant drift");
    assert!(matches!(
        scoped,
        CloudBillingEventIngestAppError::Billing(_)
    ));
    assert_eq!(
        scoped.billing_event_ingest_status(),
        CloudBillingEventIngestAppStatus::Forbidden
    );
}

fn ingest_request(event_id: &str, idempotency_key: &str) -> CloudBillingEventIngestAppRequest {
    CloudBillingEventIngestAppRequest {
        envelope: CloudBillingEventEnvelopeContext {
            event_id: event_id.to_string(),
            source: CLOUD_BILLING_EVENT_INGEST_SOURCE.to_string(),
            subject: format!("tenant/ten_kr/resource/{RESOURCE_ID}"),
            topic: CLOUD_BILLING_EVENT_TOPIC.to_string(),
            schema: CLOUD_BILLING_EVENT_INGEST_SCHEMA.to_string(),
            tenant_id: "ten_kr".to_string(),
            producer_id: "producer_cloud_resource_lifecycle".to_string(),
            idempotency_key: idempotency_key.to_string(),
            produced_at_epoch_seconds: 1_700_000_001,
        },
        authorization: CloudBillingEventIngestAuthorization {
            tenant_id: "ten_kr".to_string(),
            producer_id: "producer_cloud_resource_lifecycle".to_string(),
            decision_id: "authz_cloud_billing_event_ingest".to_string(),
            allowed_surfaces: vec![CLOUD_BILLING_EVENT_INGEST_SURFACE.to_string()],
        },
        payload: CloudBillingEventIngestPayload {
            id: event_id.to_string(),
            tenant_id: "ten_kr".to_string(),
            resource_id: RESOURCE_ID.to_string(),
            region: "kr-seoul".to_string(),
            metering_tag: "oya:metering:ten_kr:instance".to_string(),
            kind: "resource_created".to_string(),
            units: vec![CloudBillingMeterUnitRequest {
                kind: "resource_second".to_string(),
                quantity_microunits: 3_600_000_000,
            }],
            rate_card_ref: "rate/cloud/kr-seoul/standard".to_string(),
            occurred_at_epoch_seconds: 1_700_000_000,
            idempotency_key: idempotency_key.to_string(),
            data_class: "PUBLIC".to_string(),
        },
    }
}
