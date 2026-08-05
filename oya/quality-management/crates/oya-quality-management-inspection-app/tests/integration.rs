#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_quality_management_inspection_app::adapter::AdapterRegistry;
use oya_quality_management_inspection_app::adapter::asyncapi::{
    AsyncApiHandler, AsyncApiMessage, ChannelDirection,
};
use oya_quality_management_inspection_app::adapter::cedar::CedarInspectionPlanPolicyFixture;
use oya_quality_management_inspection_app::adapter::grpc::{GrpcHandler, GrpcRequest};
use oya_quality_management_inspection_app::adapter::http::{HttpHandler, HttpRequest};
use oya_quality_management_inspection_app::adapter::repository::InMemoryRepositoryFixture;
use oya_quality_management_inspection_app::config::ServiceConfig;
use oya_quality_management_inspection_app::domain::{
    BoundedContext, Capability, CapabilityDescriptor, IdempotencyKey, PrincipalId, RequestId,
    ResourceId, ServiceCommand, ServiceEvent, TenantId, UsecaseActor,
};
use oya_quality_management_inspection_app::error::ServiceErrorKind;
use oya_quality_management_inspection_app::usecase::{
    AuditPort, ClockPort, CommandEnvelope, EventPort, PolicyPort, RepositoryPort,
    ServiceInteractor, UsecaseContext,
};
use oya_quality_management_inspection_app::{public_api_surface, scaffold};
use serde_json::json;

// Keep this as a local fixture instead of include_str!("../../../policy/...") so the Buck2
// rust_test remains self-contained; the fixture preserves the two source-policy predicates this
// app-layer review owns: inspection-plan approval action plus tenant equality.
const INSPECTION_PLAN_POLICY: &str = r#"
permit (principal, action in [Action::"inspection-plan.approve"], resource)
when {
  principal.tenant_id == resource.tenant_id
};
"#;

fn inspection_plan_envelope(resource_tenant: &str, idempotency_key: &str) -> CommandEnvelope {
    CommandEnvelope {
        context: UsecaseContext {
            actor: UsecaseActor {
                tenant_id: TenantId::new("tenant-alpha").unwrap(),
                principal_id: PrincipalId::new("quality-approver").unwrap(),
                request_id: RequestId::new("request-quality-0001").unwrap(),
                idempotency_key: IdempotencyKey::new(idempotency_key).unwrap(),
            },
            source: "contract-fixture".to_owned(),
            data_residency_pack: "KR".to_owned(),
        },
        command: ServiceCommand::Submit {
            capability: Capability::InspectionPlan,
            resource_id: ResourceId::new(format!(
                "{resource_tenant}:qplan_laptop_final_acceptance"
            ))
            .unwrap(),
        },
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct AcceptAllAudit;

impl AuditPort for AcceptAllAudit {
    fn append(&self, _event: &ServiceEvent) -> oya_quality_management_inspection_app::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PublishAllEvents;

impl EventPort for PublishAllEvents {
    fn publish(&self, _event: &ServiceEvent) -> oya_quality_management_inspection_app::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct FixedClock;

impl ClockPort for FixedClock {
    fn now_rfc3339(&self) -> oya_quality_management_inspection_app::Result<String> {
        Ok("2026-05-23T00:00:00Z".to_owned())
    }
}

#[test]
fn scaffold_declares_expected_contracts() {
    let scaffold = scaffold();
    assert_eq!(scaffold.microservice, "quality-management");
    assert_eq!(scaffold.contracts.openapi, "contracts/openapi-v1.yaml");
    assert_eq!(scaffold.contracts.asyncapi, "contracts/asyncapi-v1.yaml");
    assert_eq!(
        scaffold.contracts.grpc,
        "contracts/quality-management-v1.proto"
    );
}

#[test]
fn scaffold_declares_adr_0105_layers() {
    assert_eq!(scaffold().layers.len(), 13);
}

#[test]
fn scaffold_declares_domain_capabilities() {
    let scaffold = scaffold();
    assert!(scaffold.capabilities.contains(&Capability::InspectionPlan));
    assert!(scaffold.capabilities.contains(&Capability::QualityHold));
}

#[test]
fn config_default_validates_with_named_cli_args() {
    ServiceConfig::local_default("tenant-alpha", 9080)
        .validate()
        .expect("default config validates");
}

#[test]
fn adapter_registry_contains_three_contract_surfaces() {
    let registry = AdapterRegistry::scaffolded();
    registry.validate().expect("registry validates");
    assert!(registry.http_routes.len() >= 6);
    assert!(registry.grpc_methods.len() >= 6);
    assert!(registry.asyncapi_channels.len() >= 6);
    assert!(registry.http_routes.iter().any(|route| {
        route.method == "POST"
            && route.path == "/v1/quality-management/quality-notification"
            && route.capability == "quality-notification"
            && route.idempotent
    }));
    assert!(registry.asyncapi_channels.iter().all(|channel| {
        channel.direction == ChannelDirection::Publish && channel.channel.ends_with(".events.v1")
    }));
    assert!(registry.asyncapi_channels.iter().any(|channel| {
        channel.channel == "quality-management.quality-notification.events.v1"
            && channel.message == "QualityNotificationChanged"
    }));
}

#[test]
fn capability_descriptors_preserve_quality_notification_context() {
    let descriptors = CapabilityDescriptor::descriptors();
    let notification = descriptors
        .iter()
        .find(|descriptor| descriptor.capability == Capability::QualityNotification)
        .expect("quality-notification descriptor exists");
    assert_eq!(
        notification.bounded_context,
        BoundedContext::QualityNotification
    );
}

#[test]
fn tenant_id_rejects_empty_value() {
    assert!(TenantId::new("   ").is_err());
}

#[test]
fn idempotency_key_requires_stable_length() {
    assert!(IdempotencyKey::new("short").is_err());
    assert!(IdempotencyKey::new("quality-management-key-0001").is_ok());
}

#[test]
fn openapi_command_fixture_round_trips() {
    let response = HttpHandler::handle(HttpRequest {
        tenant_id: "tenant-alpha".to_owned(),
        principal_id: "quality-approver".to_owned(),
        request_id: "request-openapi-0001".to_owned(),
        idempotency_key: "quality-management-key-0001".to_owned(),
        body: json!({
            "capability": "inspection-plan",
            "resource_id": "tenant-alpha:qplan_laptop_final_acceptance"
        }),
    })
    .expect("OpenAPI inspection-plan command fixture is accepted");

    assert_eq!(response.status, 202);
    assert_eq!(response.body["accepted"], json!(true));
    assert_eq!(response.body["tenant_id"], json!("tenant-alpha"));
    assert_eq!(response.body["capability"], json!("inspection-plan"));
    assert_eq!(response.body["audit_event_type"], json!("command-accepted"));
    assert_eq!(response.body["runtime_deployed"], json!(false));
    assert_eq!(response.body["durable_persistence"], json!(false));
}

#[test]
fn grpc_command_fixture_round_trips() {
    assert!(
        GrpcHandler::methods()
            .iter()
            .any(|method| method.method == "MutateInspectionPlan")
    );

    let response = GrpcHandler::handle(GrpcRequest {
        tenant_id: "tenant-alpha".to_owned(),
        method: "MutateInspectionPlan".to_owned(),
        payload_json: json!({
            "principal_id": "quality-approver",
            "idempotency_key": "quality-management-key-0002",
            "resource_id": "tenant-alpha:qplan_laptop_final_acceptance"
        }),
    })
    .expect("gRPC inspection-plan command fixture is accepted");

    assert!(response.accepted);
    assert_eq!(response.payload_json["tenant_id"], json!("tenant-alpha"));
    assert_eq!(
        response.payload_json["capability"],
        json!("inspection-plan")
    );
    assert_eq!(
        response.payload_json["audit_event_type"],
        json!("command-accepted")
    );
    assert_eq!(response.payload_json["runtime_deployed"], json!(false));
    assert_eq!(response.payload_json["durable_persistence"], json!(false));
}

#[test]
fn asyncapi_event_fixture_round_trips() {
    AsyncApiHandler::handle(AsyncApiMessage {
        tenant_id: "tenant-alpha".to_owned(),
        message_type: "InspectionPlanChanged".to_owned(),
        payload_json: json!({
            "audit_event_class": "EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-CHANGED",
            "bounded_context": "inspection-plan",
            "runtime_audit_chain_emitted": false
        }),
    })
    .expect("AsyncAPI inspection-plan event fixture is accepted without runtime audit-chain claim");
}

#[test]
fn cedar_policy_denies_cross_tenant_command() {
    let policy = CedarInspectionPlanPolicyFixture::from_fragment(INSPECTION_PLAN_POLICY)
        .expect("inspection-plan Cedar fixture validates tenant equality rule");

    policy
        .authorize(&inspection_plan_envelope(
            "tenant-alpha",
            "quality-management-key-0003",
        ))
        .expect("matching tenant command is permitted");

    let denied = policy
        .authorize(&inspection_plan_envelope(
            "tenant-beta",
            "quality-management-key-0004",
        ))
        .expect_err("cross-tenant command is denied");
    assert_eq!(denied.kind(), ServiceErrorKind::Authorization);
}

#[test]
fn repository_port_enforces_idempotency() {
    let repository = InMemoryRepositoryFixture::default();
    let envelope = inspection_plan_envelope("tenant-alpha", "quality-management-key-0005");
    repository
        .reserve_idempotency_key(&envelope)
        .expect("first reserve succeeds");

    let duplicate = repository
        .reserve_idempotency_key(&envelope)
        .expect_err("second reserve conflicts for same tenant/idempotency key");
    assert_eq!(duplicate.kind(), ServiceErrorKind::Conflict);

    let interactor = ServiceInteractor::new(
        CedarInspectionPlanPolicyFixture::from_fragment(INSPECTION_PLAN_POLICY).unwrap(),
        AcceptAllAudit,
        PublishAllEvents,
        InMemoryRepositoryFixture::default(),
        FixedClock,
    );
    let receipt = interactor
        .submit_command(inspection_plan_envelope(
            "tenant-alpha",
            "quality-management-key-0006",
        ))
        .expect("repository fixture supports accepted command persistence");
    assert!(receipt.accepted);
    assert_eq!(receipt.capability, "InspectionPlan");
}

#[test]
fn public_surface_names_required_handlers() {
    let surface = public_api_surface();
    assert!(surface.iter().any(|name| name.contains("HttpHandler")));
    assert!(surface.iter().any(|name| name.contains("GrpcHandler")));
    assert!(surface.iter().any(|name| name.contains("AsyncApiHandler")));
}
