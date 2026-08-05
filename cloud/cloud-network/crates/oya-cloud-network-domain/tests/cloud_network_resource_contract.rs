// ADR-0083 Tier 3: integration tests use `.expect()` to assert invariants — Tier 3 exemption.
#![allow(clippy::expect_used)]

use oya_cloud_network_domain::{
    CloudNetworkError, NetworkOperationKind, NetworkResourceContract,
    NetworkResourceContractCreate, NetworkResourceFacet, NetworkResourceScope, NetworkResourceType,
    NetworkSloTier,
};

const VPC_ID: &str = "oya:cloud:region-alpha1:ten_alpha:vpc:prod";
const SUBNET_ID: &str = "oya:cloud:region-alpha1:ten_alpha:subnet:prod-a";
const SECURITY_GROUP_ID: &str = "sg_web";
const LOAD_BALANCER_ID: &str = "oya:cloud:region-alpha1:ten_alpha:lb-v7:frontdoor";
const GATEWAY_ID: &str = "oya:cloud:region-alpha1:ten_alpha:gateway:frontdoor";

fn contract_create(
    resource_type: NetworkResourceType,
    resource_id: &str,
    scope: NetworkResourceScope,
) -> NetworkResourceContractCreate {
    NetworkResourceContractCreate {
        resource_type,
        resource_id: resource_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        account_id: "acct_alpha".to_string(),
        project_id: "proj_network".to_string(),
        resource_group_id: "rg_edge".to_string(),
        region: "region-alpha1".to_string(),
        cell_id: "cell-region-alpha1-a-001".to_string(),
        owner_principal: "sp_network_admin".to_string(),
        scope,
        operation_kind: NetworkOperationKind::Create,
        operation_id: "op_cloud_network_create_frontdoor".to_string(),
        idempotency_key: "idem-cloud-network-frontdoor".to_string(),
        policy_ref: "cedar/cloud-network/resource-contract".to_string(),
        quota_cost_units: 3,
        quota_reservation_ref: Some("quota/ten_alpha/cloud-network/frontdoor".to_string()),
        quota_refusal_reason: None,
        billing_meters: vec!["cloud.network.resource.hour".to_string()],
        meter_event_intent_ref: "meter-event/cloud-network/frontdoor/create".to_string(),
        audit_event_class: "cloud.network.resource.create.requested".to_string(),
        audit_event_ref: "audit/cloud-network/frontdoor/create".to_string(),
        slo_tier: NetworkSloTier::ControlPlaneMetadataOnly,
        metric_hook_names: vec!["cloud_network_resource_contract_intent_total".to_string()],
        trace_hook_names: vec!["cloud.network.resource_contract.create".to_string()],
        measured_slo_claimed: false,
        rollback_plan_ref: "rollback/cloud-network/frontdoor/create".to_string(),
        compensating_action: "delete-created-resource-if-provider-create-succeeds-after-timeout"
            .to_string(),
        desired_state: "creating".to_string(),
        actual_state: "pending_provider_receipt".to_string(),
        reconciliation_status: "awaiting_provider_receipt".to_string(),
        live_resource_registry_claimed: false,
        live_operation_ledger_claimed: false,
        live_reconciler_claimed: false,
        live_provider_apply_claimed: false,
    }
}

fn assert_common_facets(contract: &NetworkResourceContract) {
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::LifecycleLroEnvelope)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::IdentityBinding)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::PolicyReference)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::QuotaReservation)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::BillingMeterIntent)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::AuditEventEnvelope)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::ObservabilityHooks)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::RollbackCompensation)
    );
    assert!(
        contract
            .facets()
            .contains(&NetworkResourceFacet::DesiredActualReconciliation)
    );
    assert_eq!(contract.tenant_id(), "ten_alpha");
    assert_eq!(contract.account_id(), "acct_alpha");
    assert_eq!(contract.project_id(), "proj_network");
    assert_eq!(contract.resource_group_id(), "rg_edge");
    assert_eq!(contract.cell_id(), "cell-region-alpha1-a-001");
    assert_eq!(contract.owner_principal(), "sp_network_admin");
    assert_eq!(contract.operation_kind(), NetworkOperationKind::Create);
    assert!(
        contract
            .orn()
            .starts_with("orn:oya:region-alpha1:acct_alpha:cloud-network:")
    );
    assert_eq!(contract.region(), "region-alpha1");
    assert_eq!(contract.operation_id(), "op_cloud_network_create_frontdoor");
    assert_eq!(contract.idempotency_key(), "idem-cloud-network-frontdoor");
    assert_eq!(
        contract.policy_ref(),
        "cedar/cloud-network/resource-contract"
    );
    assert_eq!(contract.quota_cost_units(), 3);
    assert_eq!(
        contract.quota_reservation_ref(),
        Some("quota/ten_alpha/cloud-network/frontdoor")
    );
    assert_eq!(contract.quota_refusal_reason(), None);
    assert_eq!(contract.billing_meters().len(), 1);
    assert_eq!(contract.billing_meters()[0], "cloud.network.resource.hour");
    assert_eq!(
        contract.meter_event_intent_ref(),
        "meter-event/cloud-network/frontdoor/create"
    );
    assert_eq!(
        contract.audit_event_class(),
        "cloud.network.resource.create.requested"
    );
    assert_eq!(
        contract.audit_event_ref(),
        "audit/cloud-network/frontdoor/create"
    );
    assert_eq!(
        contract.slo_tier(),
        NetworkSloTier::ControlPlaneMetadataOnly
    );
    assert_eq!(contract.metric_hook_names().len(), 1);
    assert_eq!(
        contract.metric_hook_names()[0],
        "cloud_network_resource_contract_intent_total"
    );
    assert_eq!(contract.trace_hook_names().len(), 1);
    assert_eq!(
        contract.trace_hook_names()[0],
        "cloud.network.resource_contract.create"
    );
    assert_eq!(
        contract.rollback_plan_ref(),
        "rollback/cloud-network/frontdoor/create"
    );
    assert_eq!(
        contract.compensating_action(),
        "delete-created-resource-if-provider-create-succeeds-after-timeout"
    );
    assert_eq!(contract.desired_state(), "creating");
    assert_eq!(contract.actual_state(), "pending_provider_receipt");
    assert_eq!(
        contract.reconciliation_status(),
        "awaiting_provider_receipt"
    );
    assert!(!contract.claims_live_resource_registry());
    assert!(!contract.claims_live_operation_ledger());
    assert!(!contract.claims_live_reconciler());
    assert!(!contract.claims_live_provider_apply());
    assert!(!contract.claims_measured_slo());
}

#[test]
fn vpc_subnet_and_load_balancer_contracts_bind_to_existing_resource_kind_labels() {
    let vpc = NetworkResourceContract::new(contract_create(
        NetworkResourceType::Vpc,
        VPC_ID,
        NetworkResourceScope::FirstClassResource,
    ))
    .expect("VPC resource contract is valid");
    let subnet = NetworkResourceContract::new(contract_create(
        NetworkResourceType::Subnet,
        SUBNET_ID,
        NetworkResourceScope::FirstClassResource,
    ))
    .expect("Subnet resource contract is valid");
    let lb = NetworkResourceContract::new(contract_create(
        NetworkResourceType::LoadBalancerL7,
        LOAD_BALANCER_ID,
        NetworkResourceScope::FirstClassResource,
    ))
    .expect("LB resource contract is valid");

    assert_common_facets(&vpc);
    assert_common_facets(&subnet);
    assert_common_facets(&lb);
    assert_eq!(vpc.registry_kind_label(), Some("vpc"));
    assert_eq!(subnet.registry_kind_label(), Some("subnet"));
    assert_eq!(lb.registry_kind_label(), Some("lb-v7"));
    assert_eq!(
        vpc.orn(),
        "orn:oya:region-alpha1:acct_alpha:cloud-network:vpc/oya:cloud:region-alpha1:ten_alpha:vpc:prod"
    );
}

#[test]
fn resource_contract_rejects_resource_id_tenant_region_and_kind_mismatches() {
    let wrong_kind = contract_create(
        NetworkResourceType::Vpc,
        SUBNET_ID,
        NetworkResourceScope::FirstClassResource,
    );
    assert_eq!(
        NetworkResourceContract::new(wrong_kind)
            .expect_err("VPC contract must use VPC resource id"),
        CloudNetworkError::ResourceKindMismatch
    );

    let wrong_tenant = contract_create(
        NetworkResourceType::Vpc,
        "oya:cloud:region-alpha1:ten_beta:vpc:prod",
        NetworkResourceScope::FirstClassResource,
    );
    assert_eq!(
        NetworkResourceContract::new(wrong_tenant)
            .expect_err("contract resource id tenant must match contract tenant"),
        CloudNetworkError::ResourceTenantMismatch
    );

    let wrong_region = contract_create(
        NetworkResourceType::LoadBalancerL4,
        "oya:cloud:region-beta1:ten_alpha:lb-v4:edge",
        NetworkResourceScope::FirstClassResource,
    );
    assert_eq!(
        NetworkResourceContract::new(wrong_region)
            .expect_err("contract resource id region must match contract region"),
        CloudNetworkError::ResourceRegionMismatch
    );
}

#[test]
fn resource_contract_validates_child_and_gateway_resource_identity_boundaries() {
    let wrong_parent_kind = contract_create(
        NetworkResourceType::SecurityGroup,
        SECURITY_GROUP_ID,
        NetworkResourceScope::VpcChild {
            parent_vpc_id: SUBNET_ID.to_string(),
            child_path: "security-groups/sg_web".to_string(),
        },
    );
    assert_eq!(
        NetworkResourceContract::new(wrong_parent_kind)
            .expect_err("security group parent must be a VPC resource id"),
        CloudNetworkError::ResourceKindMismatch
    );

    let wrong_gateway_kind = contract_create(
        NetworkResourceType::Gateway,
        VPC_ID,
        NetworkResourceScope::Network001BoundaryReference {
            authoritative_task_id: "t_9e4e1495".to_string(),
            decision_ref: "NETWORK-001/gateway-route-mesh-boundary".to_string(),
        },
    );
    assert_eq!(
        NetworkResourceContract::new(wrong_gateway_kind)
            .expect_err("gateway contract must use gateway resource id"),
        CloudNetworkError::ResourceKindMismatch
    );

    let wrong_child_path = contract_create(
        NetworkResourceType::SecurityGroup,
        SECURITY_GROUP_ID,
        NetworkResourceScope::VpcChild {
            parent_vpc_id: VPC_ID.to_string(),
            child_path: "subnets/sg_web".to_string(),
        },
    );
    assert_eq!(
        NetworkResourceContract::new(wrong_child_path)
            .expect_err("security group child path must stay in the security-groups namespace"),
        CloudNetworkError::InvalidResourceContractScope
    );

    let wrong_gateway_decision = contract_create(
        NetworkResourceType::Gateway,
        GATEWAY_ID,
        NetworkResourceScope::Network001BoundaryReference {
            authoritative_task_id: "t_9e4e1495".to_string(),
            decision_ref: "NETWORK-002/gateway-route-mesh-boundary".to_string(),
        },
    );
    assert_eq!(
        NetworkResourceContract::new(wrong_gateway_decision)
            .expect_err("gateway contract must cite NETWORK-001 boundary authority"),
        CloudNetworkError::InvalidResourceContractScope
    );
}

#[test]
fn security_group_is_encoded_as_vpc_child_until_shared_resource_kind_adds_first_class_variant() {
    let security_group = NetworkResourceContract::new(contract_create(
        NetworkResourceType::SecurityGroup,
        SECURITY_GROUP_ID,
        NetworkResourceScope::VpcChild {
            parent_vpc_id: VPC_ID.to_string(),
            child_path: "security-groups/sg_web".to_string(),
        },
    ))
    .expect("security group child-resource contract is valid");

    assert_common_facets(&security_group);
    assert_eq!(security_group.registry_kind_label(), None);
    assert_eq!(security_group.parent_vpc_id(), Some(VPC_ID));
    assert_eq!(
        security_group.non_claim_reason(),
        Some(
            "shared ResourceKind has no SecurityGroup variant; encoded as VPC child resource contract"
        )
    );
}

#[test]
fn gateway_contract_records_network_001_boundary_without_reauthoring_route_or_mesh_semantics() {
    let gateway = NetworkResourceContract::new(contract_create(
        NetworkResourceType::Gateway,
        GATEWAY_ID,
        NetworkResourceScope::Network001BoundaryReference {
            authoritative_task_id: "t_9e4e1495".to_string(),
            decision_ref: "NETWORK-001/gateway-route-mesh-boundary".to_string(),
        },
    ))
    .expect("Gateway resource-contract boundary is valid");

    assert_common_facets(&gateway);
    assert_eq!(gateway.registry_kind_label(), None);
    assert_eq!(gateway.network_001_task_id(), Some("t_9e4e1495"));
    assert!(!gateway.claims_route_schema_authority());
    assert!(!gateway.claims_mesh_mtls_or_ext_authz_authority());
    assert_eq!(
        gateway.non_claim_reason(),
        Some(
            "NETWORK-001 owns route schema, gateway classification, mTLS/SPIFFE, and ext_authz semantics"
        )
    );
}

#[test]
fn resource_contract_rejects_runtime_and_measured_slo_claims() {
    let mut live_registry = contract_create(
        NetworkResourceType::Vpc,
        VPC_ID,
        NetworkResourceScope::FirstClassResource,
    );
    live_registry.live_resource_registry_claimed = true;
    assert_eq!(
        NetworkResourceContract::new(live_registry)
            .expect_err("registry runtime claim is out of scope"),
        CloudNetworkError::ResourceContractRuntimeClaimOutOfScope
    );

    let mut measured_slo = contract_create(
        NetworkResourceType::LoadBalancerL4,
        "oya:cloud:region-alpha1:ten_alpha:lb-v4:edge",
        NetworkResourceScope::FirstClassResource,
    );
    measured_slo.measured_slo_claimed = true;
    assert_eq!(
        NetworkResourceContract::new(measured_slo).expect_err("measured SLO is out of scope"),
        CloudNetworkError::ResourceContractMeasuredSloClaimOutOfScope
    );
}

#[test]
fn resource_contract_requires_quota_billing_audit_rollback_and_reconciliation_facets() {
    let mut missing_policy = contract_create(
        NetworkResourceType::Subnet,
        SUBNET_ID,
        NetworkResourceScope::FirstClassResource,
    );
    missing_policy.policy_ref = " ".to_string();
    assert_eq!(
        NetworkResourceContract::new(missing_policy).expect_err("policy ref is required"),
        CloudNetworkError::InvalidResourceContractPolicyRef
    );

    let mut missing_billing = contract_create(
        NetworkResourceType::Subnet,
        SUBNET_ID,
        NetworkResourceScope::FirstClassResource,
    );
    missing_billing.billing_meters.clear();
    assert_eq!(
        NetworkResourceContract::new(missing_billing)
            .expect_err("billing meter intent is required"),
        CloudNetworkError::InvalidResourceContractBillingMeter
    );

    let mut missing_reconcile = contract_create(
        NetworkResourceType::Vpc,
        VPC_ID,
        NetworkResourceScope::FirstClassResource,
    );
    missing_reconcile.reconciliation_status = " ".to_string();
    assert_eq!(
        NetworkResourceContract::new(missing_reconcile)
            .expect_err("desired-vs-actual status is required"),
        CloudNetworkError::InvalidResourceContractReconciliationStatus
    );
}
