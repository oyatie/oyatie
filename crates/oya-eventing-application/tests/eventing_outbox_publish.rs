use oya_eventing_application::{
    EVENTING_OUTBOX_PUBLISH_ASYNCAPI_CONTRACT, EVENTING_OUTBOX_PUBLISH_PROTO_CONTRACT,
    EVENTING_OUTBOX_PUBLISH_SCHEMA, EVENTING_OUTBOX_PUBLISH_SOURCE,
    EVENTING_OUTBOX_PUBLISH_SURFACE, EVENTING_OUTBOX_PUBLISH_TOPIC, EventingOutboxEnvelopeContext,
    EventingOutboxPublishAppError, EventingOutboxPublishAppRequest, EventingOutboxPublishAppStatus,
    EventingOutboxPublishAuthorization, EventingOutboxPublishIdempotencyLedger,
    EventingOutboxPublishPayload, publish_eventing_outbox_from_app,
};
use oya_eventing_domain::Outbox;

const EVENT_ID: &str = "evt_outbox_publish_001";
const IDEMPOTENCY_KEY: &str = "idem_eventing_outbox_publish_001";
const TARGET_TOPIC: &str = "oya.foundation.tenancy";
const PAYLOAD_REF: &str = "tenant-events/ten_kr/provisioned-001";

#[test]
fn eventing_outbox_contract_runtime_constants_are_covered() {
    assert_eq!(EVENTING_OUTBOX_PUBLISH_SURFACE, "eventing.outbox.publish");
    assert_eq!(EVENTING_OUTBOX_PUBLISH_TOPIC, "oya.foundation.eventing");
    assert_eq!(EVENTING_OUTBOX_PUBLISH_SCHEMA, "eventing.outbox.publish.v1");
    assert_eq!(EVENTING_OUTBOX_PUBLISH_SOURCE, "oyatie://platform/eventing");
    assert_eq!(
        EVENTING_OUTBOX_PUBLISH_ASYNCAPI_CONTRACT,
        "contracts/asyncapi/platform/eventing-outbox-v1.yaml"
    );
    assert_eq!(
        EVENTING_OUTBOX_PUBLISH_PROTO_CONTRACT,
        "contracts/proto/platform/eventing/v1/eventing-outbox-v1.proto"
    );
    assert_eq!(EventingOutboxPublishAppStatus::Accepted.code(), 202);
    assert_eq!(EventingOutboxPublishAppStatus::BadRequest.code(), 400);
    assert_eq!(EventingOutboxPublishAppStatus::Unauthorized.code(), 401);
    assert_eq!(EventingOutboxPublishAppStatus::Forbidden.code(), 403);
    assert_eq!(
        EventingOutboxPublishAppStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn outbox_publish_records_once_and_replays_same_idempotent_result() {
    let mut outbox = Outbox::default();
    let mut idempotency = EventingOutboxPublishIdempotencyLedger::default();
    let request = publish_request(EVENT_ID, IDEMPOTENCY_KEY);

    let first = publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, request.clone())
        .expect("first publish creates an outbox record");
    let second = publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, request)
        .expect("same request fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(outbox.records().len(), 1);
    assert_eq!(first.data.sequence, 0);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.target_topic, TARGET_TOPIC);
    assert_eq!(first.data.idempotency_key, IDEMPOTENCY_KEY);
    assert_eq!(first.data.payload_ref, PAYLOAD_REF);
    assert!(!first.data.published);
    assert_eq!(first.data.payload_schema, "tenant.provisioned.v1");
    assert_eq!(
        first.data.data_classes_touched,
        vec!["INTERNAL_ONLY".to_string()]
    );
    assert_eq!(
        first.data.regulatory_packs_consumed,
        vec!["oya-pack-kr".to_string()]
    );
    assert_eq!(first.data.schema_version, 1);
    assert_eq!(first.metadata.event_id, EVENT_ID);
    assert_eq!(first.metadata.producer_id, "producer_tenant_lifecycle");
    assert_eq!(first.metadata.schema, EVENTING_OUTBOX_PUBLISH_SCHEMA);
}

#[test]
fn outbox_publish_rejects_envelope_payload_drift_before_kernel() {
    let mut outbox = Outbox::default();
    let mut idempotency = EventingOutboxPublishIdempotencyLedger::default();
    let mut tenant_drift = publish_request(EVENT_ID, "idem_eventing_outbox_drift_tenant");
    tenant_drift.payload.tenant_id = "ten_other".to_string();

    let tenant_error =
        publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, tenant_drift)
            .expect_err("tenant drift is rejected before outbox mutation");
    assert!(matches!(
        tenant_error,
        EventingOutboxPublishAppError::EnvelopePayloadTenantMismatch { .. }
    ));
    assert_eq!(
        tenant_error.eventing_outbox_publish_status(),
        EventingOutboxPublishAppStatus::Forbidden
    );

    let mut idempotency_drift = publish_request(EVENT_ID, "idem_eventing_outbox_drift_idempotency");
    idempotency_drift.payload.idempotency_key = "idem_payload_other".to_string();
    let idempotency_error =
        publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, idempotency_drift)
            .expect_err("idempotency drift is rejected before outbox mutation");
    assert!(matches!(
        idempotency_error,
        EventingOutboxPublishAppError::EnvelopePayloadIdempotencyMismatch { .. }
    ));
    assert_eq!(idempotency_error.eventing_outbox_publish_status_code(), 400);

    assert!(idempotency.is_empty());
    assert!(outbox.records().is_empty());
}

#[test]
fn outbox_publish_separates_missing_producer_from_denied_authorization() {
    let mut outbox = Outbox::default();
    let mut idempotency = EventingOutboxPublishIdempotencyLedger::default();
    let mut unauthenticated = publish_request(EVENT_ID, "idem_eventing_outbox_authn");
    unauthenticated.envelope.producer_id.clear();

    let authn_error =
        publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, unauthenticated)
            .expect_err("missing producer is authentication failure");
    assert_eq!(
        authn_error.eventing_outbox_publish_status(),
        EventingOutboxPublishAppStatus::Unauthorized
    );

    let mut denied = publish_request(EVENT_ID, "idem_eventing_outbox_authz");
    denied.authorization.allowed_surfaces = vec!["metering.event.ingest".to_string()];
    let authz_error = publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, denied)
        .expect_err("missing surface grant is authorization failure");
    assert!(matches!(
        authz_error,
        EventingOutboxPublishAppError::AuthorizationDenied { ref surface }
            if surface == EVENTING_OUTBOX_PUBLISH_SURFACE
    ));
    assert_eq!(
        authz_error.eventing_outbox_publish_status(),
        EventingOutboxPublishAppStatus::Forbidden
    );
    assert!(idempotency.is_empty());
    assert!(outbox.records().is_empty());
}

#[test]
fn outbox_publish_rejects_invalid_topic_payload_ref_and_data_class_before_kernel() {
    let mut outbox = Outbox::default();
    let mut idempotency = EventingOutboxPublishIdempotencyLedger::default();

    let mut invalid_topic = publish_request(EVENT_ID, "idem_eventing_outbox_bad_topic");
    invalid_topic.payload.target_topic = "tenant-events".to_string();
    assert!(matches!(
        publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, invalid_topic),
        Err(EventingOutboxPublishAppError::InvalidTargetTopic { .. })
    ));

    let mut invalid_ref = publish_request(EVENT_ID, "idem_eventing_outbox_bad_ref");
    invalid_ref.payload.payload_ref = "../tenant-events/001".to_string();
    assert!(matches!(
        publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, invalid_ref),
        Err(EventingOutboxPublishAppError::InvalidPayloadRef { .. })
    ));

    let mut invalid_class = publish_request(EVENT_ID, "idem_eventing_outbox_bad_class");
    invalid_class.payload.data_classes_touched = vec!["AUDIT".to_string()];
    let class_error =
        publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, invalid_class)
            .expect_err("operational markers are not privacy data-class payload extensions");
    assert!(matches!(
        class_error,
        EventingOutboxPublishAppError::InvalidDataClassLabel { .. }
    ));
    assert_eq!(class_error.eventing_outbox_publish_status_code(), 400);

    assert!(idempotency.is_empty());
    assert!(outbox.records().is_empty());
}

#[test]
fn outbox_publish_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut outbox = Outbox::default();
    let mut idempotency = EventingOutboxPublishIdempotencyLedger::default();
    let mut request = publish_request(EVENT_ID, "idem_eventing_outbox_reused");

    publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, request.clone())
        .expect("first idempotent request succeeds");

    request.payload.payload_ref = "tenant-events/ten_kr/provisioned-002".to_string();
    let error = publish_eventing_outbox_from_app(&mut outbox, &mut idempotency, request)
        .expect_err("same idempotency key with changed payload is rejected");

    assert_eq!(
        error,
        EventingOutboxPublishAppError::IdempotencyKeyReused {
            idempotency_key: "idem_eventing_outbox_reused".to_string()
        }
    );
    assert_eq!(
        error.eventing_outbox_publish_status(),
        EventingOutboxPublishAppStatus::UnprocessableEntity
    );
    assert_eq!(idempotency.len(), 1);
    assert_eq!(outbox.records().len(), 1);
}

fn publish_request(event_id: &str, idempotency_key: &str) -> EventingOutboxPublishAppRequest {
    EventingOutboxPublishAppRequest {
        envelope: EventingOutboxEnvelopeContext {
            event_id: event_id.to_string(),
            source: EVENTING_OUTBOX_PUBLISH_SOURCE.to_string(),
            subject: "tenant/ten_kr/topic/oya.foundation.tenancy".to_string(),
            topic: EVENTING_OUTBOX_PUBLISH_TOPIC.to_string(),
            schema: EVENTING_OUTBOX_PUBLISH_SCHEMA.to_string(),
            tenant_id: "ten_kr".to_string(),
            producer_id: "producer_tenant_lifecycle".to_string(),
            idempotency_key: idempotency_key.to_string(),
            produced_at_epoch_seconds: 1_700_000_000,
        },
        authorization: EventingOutboxPublishAuthorization {
            tenant_id: "ten_kr".to_string(),
            producer_id: "producer_tenant_lifecycle".to_string(),
            decision_id: "authz_eventing_outbox_publish".to_string(),
            allowed_surfaces: vec![EVENTING_OUTBOX_PUBLISH_SURFACE.to_string()],
        },
        payload: EventingOutboxPublishPayload {
            tenant_id: "ten_kr".to_string(),
            target_topic: TARGET_TOPIC.to_string(),
            idempotency_key: idempotency_key.to_string(),
            payload_ref: PAYLOAD_REF.to_string(),
            payload_schema: "tenant.provisioned.v1".to_string(),
            data_classes_touched: vec!["INTERNAL_ONLY".to_string()],
            regulatory_packs_consumed: vec!["oya-pack-kr".to_string()],
        },
    }
}
