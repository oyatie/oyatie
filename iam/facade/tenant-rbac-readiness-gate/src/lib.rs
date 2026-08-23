//! Tenant RBAC cloud-integration readiness gate.
//!
//! This control-plane crate combines the local runtime composition manifest,
//! process-local in-memory harness, and executable ERP/SAP parity map into one
//! pre-cloud readiness report for dogfooding FD-001 microservices as tenant
//! workloads on the later Oyatie Cloud substrate. It deliberately reports cloud
//! deployment as blocked until external identity-provider runtime evidence,
//! durable storage runtime attachment, live Postgres/RLS verification,
//! Workflow/broker execution, statutory rails, audit emission, deployment
//! evidence, and SLO evidence land in future slices.
#![forbid(unsafe_code)]

use iam_tenant_rbac_audit_chain_emission::{
    TenantRbacAuditChainEmissionError, tenant_rbac_audit_chain_emission_plan,
    validate_tenant_rbac_audit_chain_emission_plan,
};
use iam_tenant_rbac_audit_chain_runtime_evidence::{
    TenantRbacAuditChainRuntimeEvidenceError, tenant_rbac_audit_chain_runtime_evidence_plan,
    validate_tenant_rbac_audit_chain_runtime_evidence_plan,
};
use iam_tenant_rbac_auth_app::{
    TenantRbacAuthRuntimeError, tenant_rbac_auth_runtime_policy,
    validate_tenant_rbac_auth_runtime_policy,
};
use iam_tenant_rbac_deployment_evidence::{
    TenantRbacCloudDeploymentEvidenceError, tenant_rbac_deployment_evidence_plan,
    validate_tenant_rbac_deployment_evidence_plan,
};
use iam_tenant_rbac_deployment_manifest::{
    CloudDeploymentManifestError, tenant_rbac_deployment_manifest,
    validate_cloud_deployment_manifest,
};
use iam_tenant_rbac_disbursement_evidence::{
    TenantRbacDisbursementEvidenceError, tenant_rbac_disbursement_evidence_plan,
    validate_tenant_rbac_disbursement_evidence_plan,
};
use iam_tenant_rbac_erp_parity_map::{
    ErpParityMapError, tenant_rbac_erp_parity_map, validate_erp_parity_map,
};
use iam_tenant_rbac_identity_provider_runtime_evidence::{
    TenantRbacIdentityProviderRuntimeEvidenceError,
    tenant_rbac_identity_provider_runtime_evidence_plan,
    validate_tenant_rbac_identity_provider_runtime_evidence_plan,
};
use iam_tenant_rbac_identity_provider_verification::{
    IdentityProviderVerificationError, tenant_rbac_identity_provider_verification_plan,
    validate_tenant_rbac_identity_provider_verification_plan,
};
use iam_tenant_rbac_listener_gateway::{
    TenantRbacListenerGatewayError, tenant_rbac_listener_gateway_plan,
    validate_tenant_rbac_listener_gateway_plan,
};
use iam_tenant_rbac_listener_runtime_evidence::{
    TenantRbacListenerRuntimeEvidenceError, tenant_rbac_listener_runtime_evidence_plan,
    validate_tenant_rbac_listener_runtime_evidence_plan,
};
use iam_tenant_rbac_local_inmemory_harness::TenantRbacLocalInMemoryHarness;
use iam_tenant_rbac_local_runtime_composition::{
    TenantRbacLocalRuntimeCompositionError, tenant_rbac_local_runtime_composition,
    validate_unique_method_paths,
};
use iam_tenant_rbac_postgres_rls_runtime_evidence::{
    TenantRbacPostgresRlsRuntimeEvidenceError, tenant_rbac_postgres_rls_runtime_evidence_plan,
    validate_tenant_rbac_postgres_rls_runtime_evidence_plan,
};
use iam_tenant_rbac_postgres_rls_storage::{
    TenantRbacPostgresRlsStorageError, tenant_rbac_postgres_rls_storage_plan,
    validate_tenant_rbac_postgres_rls_storage_plan,
};
use iam_tenant_rbac_postgres_rls_transaction_contract::{
    TenantRbacPostgresRlsTransactionContractError, tenant_rbac_postgres_rls_transaction_contract,
    validate_tenant_rbac_postgres_rls_transaction_contract,
};
use iam_tenant_rbac_postgres_rls_write_contract::{
    TenantRbacPostgresRlsWriteContractError, tenant_rbac_postgres_rls_write_contract,
    validate_tenant_rbac_postgres_rls_write_contract,
};
use iam_tenant_rbac_slo_evidence::{
    TenantRbacSloEvidenceError, tenant_rbac_slo_evidence_plan,
    validate_tenant_rbac_slo_evidence_plan,
};
use iam_tenant_rbac_statutory_filing_evidence::{
    TenantRbacStatutoryFilingEvidenceError, tenant_rbac_statutory_filing_evidence_plan,
    validate_tenant_rbac_statutory_filing_evidence_plan,
};
use iam_tenant_rbac_tenant_admission_policy::{
    Fd001TenantAdmissionPolicyError, fd001_tenant_admission_policy_contract,
    validate_fd001_tenant_admission_policy_contract,
};
use iam_tenant_rbac_tenant_autoscaling_contract::{
    Fd001TenantAutoscalingError, fd001_tenant_autoscaling_contract,
    validate_fd001_tenant_autoscaling_contract,
};
use iam_tenant_rbac_tenant_availability_contract::{
    Fd001TenantAvailabilityError, fd001_tenant_availability_contract,
    validate_fd001_tenant_availability_contract,
};
use iam_tenant_rbac_tenant_cost_allocation_contract::{
    Fd001TenantCostAllocationError, fd001_tenant_cost_allocation_contract,
    validate_fd001_tenant_cost_allocation_contract,
};
use iam_tenant_rbac_tenant_egress_policy_contract::{
    Fd001TenantEgressPolicyError, fd001_tenant_egress_policy_contract,
    validate_fd001_tenant_egress_policy_contract,
};
use iam_tenant_rbac_tenant_image_provenance_contract::{
    Fd001TenantImageProvenanceError, fd001_tenant_image_provenance_contract,
    validate_fd001_tenant_image_provenance_contract,
};
use iam_tenant_rbac_tenant_residency_contract::{
    Fd001TenantResidencyError, fd001_tenant_residency_contract,
    validate_fd001_tenant_residency_contract,
};
use iam_tenant_rbac_tenant_resource_quota_contract::{
    Fd001TenantResourceQuotaError, fd001_tenant_resource_quota_contract,
    validate_fd001_tenant_resource_quota_contract,
};
use iam_tenant_rbac_tenant_secret_boundary_contract::{
    Fd001TenantSecretBoundaryError, fd001_tenant_secret_boundary_contract,
    validate_fd001_tenant_secret_boundary_contract,
};
use iam_tenant_rbac_tenant_workload_identity_contract::{
    Fd001TenantWorkloadIdentityError, fd001_tenant_workload_identity_contract,
    validate_fd001_tenant_workload_identity_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::{
    Fd001TenantWorkloadManifestError, fd001_tenant_workload_manifest,
    validate_fd001_tenant_workload_manifest,
};
use iam_tenant_rbac_tenant_workload_runtime_evidence::{
    TenantRbacTenantWorkloadRuntimeEvidenceError,
    tenant_rbac_tenant_workload_runtime_evidence_plan,
    validate_tenant_rbac_tenant_workload_runtime_evidence_plan,
};
use iam_tenant_rbac_workflow_inmemory::tenant_rbac_workflow_queue_capabilities;
use iam_tenant_rbac_workflow_runtime_evidence::{
    TenantRbacWorkflowRuntimeEvidenceError, tenant_rbac_workflow_runtime_evidence_plan,
    validate_tenant_rbac_workflow_runtime_evidence_plan,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudReadinessBlocker {
    DeployedListenerRuntimeEvidenceMissing,
    IdentityProviderVerificationMissing,
    DurableStorageRuntimeMissing,
    PostgresRlsRuntimeEvidenceMissing,
    WorkflowEngineMissing,
    BrokerPublishMissing,
    StatutoryFilingRailMissing,
    DisbursementRailMissing,
    RuntimeAuditEmissionMissing,
    CloudDeploymentEvidenceMissing,
    SloEvidenceMissing,
}

impl CloudReadinessBlocker {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeployedListenerRuntimeEvidenceMissing => {
                "deployed_listener_runtime_evidence_missing"
            }
            Self::IdentityProviderVerificationMissing => "identity_provider_verification_missing",
            Self::DurableStorageRuntimeMissing => "durable_storage_runtime_missing",
            Self::PostgresRlsRuntimeEvidenceMissing => "postgres_rls_runtime_evidence_missing",
            Self::WorkflowEngineMissing => "workflow_engine_missing",
            Self::BrokerPublishMissing => "broker_publish_missing",
            Self::StatutoryFilingRailMissing => "statutory_filing_rail_missing",
            Self::DisbursementRailMissing => "disbursement_rail_missing",
            Self::RuntimeAuditEmissionMissing => "runtime_audit_emission_missing",
            Self::CloudDeploymentEvidenceMissing => "cloud_deployment_evidence_missing",
            Self::SloEvidenceMissing => "slo_evidence_missing",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacCloudReadinessReport {
    pub report_name: &'static str,                  // data_class: PUBLIC
    pub route_count: usize,                         // data_class: PUBLIC
    pub sap_module_count: usize,                    // data_class: PUBLIC
    pub route_catalog_ready: bool,                  // data_class: PUBLIC
    pub in_memory_harness_ready: bool,              // data_class: PUBLIC
    pub erp_parity_map_ready: bool,                 // data_class: PUBLIC
    pub cloud_deployment_manifest_ready: bool,      // data_class: PUBLIC
    pub cloud_deployment_evidence_plan_ready: bool, // data_class: PUBLIC
    pub cloud_deployment_evidence_requirement_count: usize, // data_class: PUBLIC
    pub tenant_workload_manifest_ready: bool,       // data_class: PUBLIC
    pub tenant_workload_count: usize,               // data_class: PUBLIC
    pub tenant_admission_policy_contract_ready: bool, // data_class: PUBLIC
    pub tenant_admission_policy_rule_count: usize,  // data_class: PUBLIC
    pub tenant_admission_policy_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_resource_quota_contract_ready: bool, // data_class: PUBLIC
    pub tenant_resource_quota_requirement_count: usize, // data_class: PUBLIC
    pub tenant_resource_quota_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_availability_contract_ready: bool,   // data_class: PUBLIC
    pub tenant_availability_requirement_count: usize, // data_class: PUBLIC
    pub tenant_availability_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_autoscaling_contract_ready: bool,    // data_class: PUBLIC
    pub tenant_autoscaling_requirement_count: usize, // data_class: PUBLIC
    pub tenant_autoscaling_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_cost_allocation_contract_ready: bool, // data_class: PUBLIC
    pub tenant_cost_allocation_requirement_count: usize, // data_class: PUBLIC
    pub tenant_cost_allocation_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_residency_contract_ready: bool,      // data_class: PUBLIC
    pub tenant_residency_requirement_count: usize,  // data_class: PUBLIC
    pub tenant_residency_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_workload_identity_contract_ready: bool, // data_class: PUBLIC
    pub tenant_workload_identity_requirement_count: usize, // data_class: PUBLIC
    pub tenant_workload_identity_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_egress_policy_contract_ready: bool,  // data_class: PUBLIC
    pub tenant_egress_policy_rule_count: usize,     // data_class: PUBLIC
    pub tenant_egress_policy_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_image_provenance_contract_ready: bool, // data_class: PUBLIC
    pub tenant_image_provenance_requirement_count: usize, // data_class: PUBLIC
    pub tenant_image_provenance_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_secret_boundary_contract_ready: bool, // data_class: PUBLIC
    pub tenant_secret_boundary_requirement_count: usize, // data_class: PUBLIC
    pub tenant_secret_boundary_all_workloads_in_scope: bool, // data_class: PUBLIC
    pub tenant_workload_runtime_evidence_plan_ready: bool, // data_class: PUBLIC
    pub tenant_workload_runtime_evidence_requirement_count: usize, // data_class: PUBLIC
    pub authentication_runtime_ready: bool,         // data_class: PUBLIC
    pub identity_provider_verification_plan_ready: bool, // data_class: PUBLIC
    pub identity_provider_runtime_evidence_plan_ready: bool, // data_class: PUBLIC
    pub identity_provider_runtime_evidence_requirement_count: usize, // data_class: PUBLIC
    pub postgres_rls_storage_plan_ready: bool,      // data_class: PUBLIC
    pub postgres_rls_write_contract_ready: bool,    // data_class: PUBLIC
    pub postgres_rls_write_statement_count: usize,  // data_class: PUBLIC
    pub postgres_rls_transaction_contract_ready: bool, // data_class: PUBLIC
    pub postgres_rls_transaction_plan_count: usize, // data_class: PUBLIC
    pub postgres_rls_runtime_evidence_plan_ready: bool, // data_class: PUBLIC
    pub listener_gateway_plan_ready: bool,          // data_class: PUBLIC
    pub listener_runtime_evidence_plan_ready: bool, // data_class: PUBLIC
    pub listener_runtime_evidence_requirement_count: usize, // data_class: PUBLIC
    pub audit_chain_emission_plan_ready: bool,      // data_class: PUBLIC
    pub audit_chain_runtime_evidence_plan_ready: bool, // data_class: PUBLIC
    pub audit_chain_runtime_evidence_requirement_count: usize, // data_class: PUBLIC
    pub workflow_execution_reference_ready: bool,   // data_class: PUBLIC
    pub workflow_runtime_evidence_plan_ready: bool, // data_class: PUBLIC
    pub workflow_runtime_evidence_requirement_count: usize, // data_class: PUBLIC
    pub statutory_filing_evidence_plan_ready: bool, // data_class: PUBLIC
    pub disbursement_evidence_plan_ready: bool,     // data_class: PUBLIC
    pub slo_evidence_plan_ready: bool,              // data_class: PUBLIC
    pub local_rehearsal_ready: bool,                // data_class: PUBLIC
    pub cloud_deployment_ready: bool,               // data_class: PUBLIC
    pub blocker_count: usize,                       // data_class: PUBLIC
    pub blockers: Vec<CloudReadinessBlocker>,       // data_class: PUBLIC
    pub evidence_refs: Vec<&'static str>,           // data_class: PUBLIC
    pub tenant_workload_manifest_attached: bool,    // data_class: INTERNAL_ONLY
    pub fd001_product_goal_preserved: bool,         // data_class: INTERNAL_ONLY
    pub oyatie_cloud_substrate_dogfood_plan_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_namespace_contract_attached: bool,   // data_class: INTERNAL_ONLY
    pub tenant_resource_quota_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_resource_quota_policy_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_limit_range_policy_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_quota_compute_boundary_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_quota_storage_object_boundary_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_quota_admission_plugin_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_quota_usage_audit_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_quota_runtime_attached: bool,        // data_class: INTERNAL_ONLY
    pub tenant_availability_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_pod_disruption_budget_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_topology_spread_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_pod_anti_affinity_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_rolling_update_availability_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_readiness_probe_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_disruption_audit_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_availability_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_autoscaling_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_horizontal_pod_autoscaler_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_autoscaling_metrics_pipeline_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_autoscaling_replica_bounds_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_autoscaling_behavior_policy_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_autoscaling_audit_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_autoscaling_runtime_attached: bool,  // data_class: INTERNAL_ONLY
    pub tenant_cost_allocation_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_cost_label_contract_attached: bool,  // data_class: INTERNAL_ONLY
    pub tenant_cost_resource_basis_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_cost_otel_resource_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_cost_finops_allocation_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_cost_shared_cost_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_cost_audit_contract_attached: bool,  // data_class: INTERNAL_ONLY
    pub tenant_cost_allocation_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_contract_attached: bool,   // data_class: INTERNAL_ONLY
    pub tenant_residency_label_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_scheduling_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_storage_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_telemetry_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_audit_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_egress_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_model_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_residency_runtime_attached: bool,    // data_class: INTERNAL_ONLY
    pub tenant_workload_identity_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_spiffe_id_contract_attached: bool,   // data_class: INTERNAL_ONLY
    pub tenant_svid_contract_attached: bool,        // data_class: INTERNAL_ONLY
    pub tenant_mtls_contract_attached: bool,        // data_class: INTERNAL_ONLY
    pub tenant_gateway_backend_tls_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_trust_bundle_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_identity_telemetry_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_identity_audit_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_workload_identity_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_network_policy_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_gateway_route_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_admission_policy_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_validating_admission_policy_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_admission_deny_action_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_pod_security_restricted_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_digest_pinned_image_admission_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_latest_image_tag_forbidden_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_resource_requests_limits_admission_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_service_account_admission_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_default_service_account_forbidden_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_admission_audit_annotation_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_admission_runtime_attached: bool,                   // data_class: INTERNAL_ONLY
    pub tenant_egress_policy_contract_attached: bool,              // data_class: INTERNAL_ONLY
    pub tenant_default_deny_egress_contract_attached: bool,        // data_class: INTERNAL_ONLY
    pub tenant_dns_egress_contract_attached: bool,                 // data_class: INTERNAL_ONLY
    pub tenant_cross_namespace_egress_contract_attached: bool,     // data_class: INTERNAL_ONLY
    pub tenant_external_egress_exception_contract_attached: bool,  // data_class: INTERNAL_ONLY
    pub tenant_egress_audit_contract_attached: bool,               // data_class: INTERNAL_ONLY
    pub tenant_egress_runtime_attached: bool,                      // data_class: INTERNAL_ONLY
    pub tenant_image_provenance_contract_attached: bool,           // data_class: INTERNAL_ONLY
    pub tenant_image_provenance_slsa_contract_attached: bool,      // data_class: INTERNAL_ONLY
    pub tenant_image_provenance_sbom_contract_attached: bool,      // data_class: INTERNAL_ONLY
    pub tenant_image_provenance_cosign_contract_attached: bool,    // data_class: INTERNAL_ONLY
    pub tenant_image_provenance_admission_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_image_provenance_runtime_attached: bool,            // data_class: INTERNAL_ONLY
    pub tenant_secret_boundary_contract_attached: bool,            // data_class: INTERNAL_ONLY
    pub tenant_secret_ref_boundary_contract_attached: bool,        // data_class: INTERNAL_ONLY
    pub tenant_secret_encryption_contract_attached: bool,          // data_class: INTERNAL_ONLY
    pub tenant_secret_rbac_contract_attached: bool,                // data_class: INTERNAL_ONLY
    pub tenant_secret_rotation_audit_contract_attached: bool,      // data_class: INTERNAL_ONLY
    pub tenant_secret_runtime_attached: bool,                      // data_class: INTERNAL_ONLY
    pub tenant_workload_runtime_evidence_plan_attached: bool,      // data_class: INTERNAL_ONLY
    pub tenant_runtime_namespace_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_quota_evidence_contract_attached: bool,     // data_class: INTERNAL_ONLY
    pub tenant_runtime_network_policy_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_service_account_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_pod_security_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_workload_schedule_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_probe_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_gateway_route_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_claim_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_otel_resource_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_rollout_recovery_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_runtime_audit_event_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub tenant_workload_runtime_evidence_attached: bool,             // data_class: INTERNAL_ONLY
    pub tenant_workload_runtime_attached: bool,                      // data_class: INTERNAL_ONLY
    pub tenant_cloud_substrate_runtime_attached: bool,               // data_class: INTERNAL_ONLY
    pub deployed_listener_attached: bool,                            // data_class: INTERNAL_ONLY
    pub listener_gateway_plan_attached: bool,                        // data_class: INTERNAL_ONLY
    pub listener_runtime_evidence_plan_attached: bool,               // data_class: INTERNAL_ONLY
    pub cluster_ip_service_evidence_contract_attached: bool,         // data_class: INTERNAL_ONLY
    pub gateway_route_runtime_acceptance_contract_attached: bool,    // data_class: INTERNAL_ONLY
    pub tls_certificate_binding_evidence_contract_attached: bool,    // data_class: INTERNAL_ONLY
    pub listener_probe_evidence_contract_attached: bool,             // data_class: INTERNAL_ONLY
    pub route_authz_evidence_contract_attached: bool,                // data_class: INTERNAL_ONLY
    pub network_policy_evidence_contract_attached: bool,             // data_class: INTERNAL_ONLY
    pub endpoint_slice_evidence_contract_attached: bool,             // data_class: INTERNAL_ONLY
    pub listener_audit_event_evidence_contract_attached: bool,       // data_class: INTERNAL_ONLY
    pub listener_runtime_attached: bool,                             // data_class: INTERNAL_ONLY
    pub authentication_runtime_attached: bool,                       // data_class: INTERNAL_ONLY
    pub identity_provider_verification_plan_attached: bool,          // data_class: INTERNAL_ONLY
    pub identity_provider_runtime_evidence_plan_attached: bool,      // data_class: INTERNAL_ONLY
    pub oidc_discovery_plan_attached: bool,                          // data_class: INTERNAL_ONLY
    pub jwks_validation_plan_attached: bool,                         // data_class: INTERNAL_ONLY
    pub oidc_discovery_runtime_evidence_contract_attached: bool,     // data_class: INTERNAL_ONLY
    pub jwks_runtime_evidence_contract_attached: bool,               // data_class: INTERNAL_ONLY
    pub jwt_signature_evidence_contract_attached: bool,              // data_class: INTERNAL_ONLY
    pub jwt_claims_evidence_contract_attached: bool,                 // data_class: INTERNAL_ONLY
    pub nonce_replay_evidence_contract_attached: bool,               // data_class: INTERNAL_ONLY
    pub tenant_scope_evidence_contract_attached: bool,               // data_class: INTERNAL_ONLY
    pub sensitive_route_mfa_evidence_contract_attached: bool,        // data_class: INTERNAL_ONLY
    pub key_rotation_evidence_contract_attached: bool,               // data_class: INTERNAL_ONLY
    pub auth_failure_audit_event_evidence_contract_attached: bool,   // data_class: INTERNAL_ONLY
    pub oidc_signature_verification_attached: bool,                  // data_class: INTERNAL_ONLY
    pub jwks_provider_attached: bool,                                // data_class: INTERNAL_ONLY
    pub identity_provider_verification_attached: bool,               // data_class: INTERNAL_ONLY
    pub identity_provider_runtime_evidence_attached: bool,           // data_class: INTERNAL_ONLY
    pub durable_business_storage_attached: bool,                     // data_class: INTERNAL_ONLY
    pub postgres_rls_storage_plan_attached: bool,                    // data_class: INTERNAL_ONLY
    pub postgres_rls_write_contract_attached: bool,                  // data_class: INTERNAL_ONLY
    pub postgres_set_local_tenant_context_contract_attached: bool,   // data_class: INTERNAL_ONLY
    pub postgres_parameterized_insert_contract_attached: bool,       // data_class: INTERNAL_ONLY
    pub postgres_idempotency_conflict_contract_attached: bool,       // data_class: INTERNAL_ONLY
    pub postgres_tenant_scoped_readback_contract_attached: bool,     // data_class: INTERNAL_ONLY
    pub postgres_delete_statement_forbidden_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub postgres_write_runtime_attached: bool,                       // data_class: INTERNAL_ONLY
    pub postgres_rls_transaction_contract_attached: bool,            // data_class: INTERNAL_ONLY
    pub postgres_explicit_transaction_contract_attached: bool,       // data_class: INTERNAL_ONLY
    pub postgres_transaction_local_tenant_context_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub postgres_prepared_statement_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub postgres_bound_parameter_execution_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub postgres_commit_rollback_contract_attached: bool,    // data_class: INTERNAL_ONLY
    pub postgres_transaction_runtime_attached: bool,         // data_class: INTERNAL_ONLY
    pub postgres_prepared_statement_runtime_attached: bool,  // data_class: INTERNAL_ONLY
    pub postgres_rls_runtime_evidence_plan_attached: bool,   // data_class: INTERNAL_ONLY
    pub postgres_migration_rehearsal_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub postgres_tls_verify_full_contract_attached: bool,    // data_class: INTERNAL_ONLY
    pub postgres_rls_probe_matrix_attached: bool,            // data_class: INTERNAL_ONLY
    pub postgres_backup_restore_rehearsal_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub postgres_pitr_rehearsal_contract_attached: bool,     // data_class: INTERNAL_ONLY
    pub postgres_database_attached: bool,                    // data_class: INTERNAL_ONLY
    pub postgres_rls_runtime_verified_attached: bool,        // data_class: INTERNAL_ONLY
    pub workflow_engine_execution_attached: bool,            // data_class: INTERNAL_ONLY
    pub workflow_execution_reference_attached: bool,         // data_class: INTERNAL_ONLY
    pub workflow_broker_publish_attached: bool,              // data_class: INTERNAL_ONLY
    pub workflow_durable_queue_attached: bool,               // data_class: INTERNAL_ONLY
    pub workflow_runtime_evidence_plan_attached: bool,       // data_class: INTERNAL_ONLY
    pub workflow_definition_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_gate_evidence_contract_attached: bool,      // data_class: INTERNAL_ONLY
    pub workflow_durable_queue_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_broker_publish_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_broker_retry_dlq_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_tenant_partition_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_otel_trace_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_audit_event_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_replay_recovery_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_runtime_evidence_attached: bool,            // data_class: INTERNAL_ONLY
    pub statutory_filing_evidence_plan_attached: bool,       // data_class: INTERNAL_ONLY
    pub statutory_authority_registry_attached: bool,         // data_class: INTERNAL_ONLY
    pub statutory_payload_digest_contract_attached: bool,    // data_class: INTERNAL_ONLY
    pub statutory_agency_receipt_contract_attached: bool,    // data_class: INTERNAL_ONLY
    pub statutory_runtime_submission_attached: bool,         // data_class: INTERNAL_ONLY
    pub statutory_disbursement_rail_attached: bool,          // data_class: INTERNAL_ONLY
    pub disbursement_evidence_plan_attached: bool,           // data_class: INTERNAL_ONLY
    pub disbursement_network_registry_attached: bool,        // data_class: INTERNAL_ONLY
    pub disbursement_payment_digest_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub disbursement_reconciliation_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub disbursement_runtime_execution_attached: bool,       // data_class: INTERNAL_ONLY
    pub disbursement_bank_connection_attached: bool,         // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool,         // data_class: INTERNAL_ONLY
    pub audit_chain_emission_plan_attached: bool,            // data_class: INTERNAL_ONLY
    pub audit_chain_event_contract_attached: bool,           // data_class: INTERNAL_ONLY
    pub audit_chain_wal_plan_attached: bool,                 // data_class: INTERNAL_ONLY
    pub audit_chain_outbox_plan_attached: bool,              // data_class: INTERNAL_ONLY
    pub audit_chain_runtime_evidence_plan_attached: bool,    // data_class: INTERNAL_ONLY
    pub audit_chain_event_envelope_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_trace_context_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_otel_log_mapping_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_tenant_partition_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_payload_digest_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_wal_append_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_outbox_publish_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_broker_ack_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_merkle_seal_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_sink_ingestion_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_replay_recovery_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_failure_path_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub audit_chain_runtime_evidence_attached: bool,         // data_class: INTERNAL_ONLY
    pub cloud_deployment_manifest_attached: bool,            // data_class: INTERNAL_ONLY
    pub cloud_deployment_evidence_plan_attached: bool,       // data_class: INTERNAL_ONLY
    pub argocd_sync_evidence_contract_attached: bool,        // data_class: INTERNAL_ONLY
    pub argocd_health_evidence_contract_attached: bool,      // data_class: INTERNAL_ONLY
    pub cosign_verification_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub kubernetes_rollout_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub gateway_route_acceptance_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub otel_resource_evidence_contract_attached: bool,      // data_class: INTERNAL_ONLY
    pub deployment_audit_event_evidence_contract_attached: bool, // data_class: INTERNAL_ONLY
    pub argocd_controller_attached: bool,                    // data_class: INTERNAL_ONLY
    pub gateway_controller_attached: bool,                   // data_class: INTERNAL_ONLY
    pub load_balancer_attached: bool,                        // data_class: INTERNAL_ONLY
    pub tls_certificate_attached: bool,                      // data_class: INTERNAL_ONLY
    pub cloud_deployment_evidence_attached: bool,            // data_class: INTERNAL_ONLY
    pub slo_evidence_plan_attached: bool,                    // data_class: INTERNAL_ONLY
    pub slo_error_budget_release_gate_attached: bool,        // data_class: INTERNAL_ONLY
    pub slo_burn_rate_alert_plan_attached: bool,             // data_class: INTERNAL_ONLY
    pub slo_openslo_manifests_attached: bool,                // data_class: INTERNAL_ONLY
    pub slo_otel_metric_streams_attached: bool,              // data_class: INTERNAL_ONLY
    pub multi_region_slo_evidence_attached: bool,            // data_class: INTERNAL_ONLY
    pub schema_version: u32,                                 // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudReadinessGateError {
    RouteComposition(TenantRbacLocalRuntimeCompositionError),
    AuditChainEmission(TenantRbacAuditChainEmissionError),
    AuditChainRuntimeEvidence(TenantRbacAuditChainRuntimeEvidenceError),
    AuthRuntime(TenantRbacAuthRuntimeError),
    IdentityProviderVerification(IdentityProviderVerificationError),
    IdentityProviderRuntimeEvidence(TenantRbacIdentityProviderRuntimeEvidenceError),
    DeploymentManifest(CloudDeploymentManifestError),
    CloudDeploymentEvidence(TenantRbacCloudDeploymentEvidenceError),
    TenantWorkloadManifest(Fd001TenantWorkloadManifestError),
    TenantAdmissionPolicy(Fd001TenantAdmissionPolicyError),
    TenantAvailability(Fd001TenantAvailabilityError),
    TenantAutoscaling(Fd001TenantAutoscalingError),
    TenantCostAllocation(Fd001TenantCostAllocationError),
    TenantResidency(Fd001TenantResidencyError),
    TenantWorkloadIdentity(Fd001TenantWorkloadIdentityError),
    TenantEgressPolicy(Fd001TenantEgressPolicyError),
    TenantResourceQuota(Fd001TenantResourceQuotaError),
    TenantImageProvenance(Fd001TenantImageProvenanceError),
    TenantSecretBoundary(Fd001TenantSecretBoundaryError),
    TenantWorkloadRuntimeEvidence(TenantRbacTenantWorkloadRuntimeEvidenceError),
    PostgresRlsStorage(TenantRbacPostgresRlsStorageError),
    PostgresRlsWriteContract(TenantRbacPostgresRlsWriteContractError),
    PostgresRlsTransactionContract(TenantRbacPostgresRlsTransactionContractError),
    PostgresRlsRuntimeEvidence(TenantRbacPostgresRlsRuntimeEvidenceError),
    ListenerGateway(TenantRbacListenerGatewayError),
    ListenerRuntimeEvidence(TenantRbacListenerRuntimeEvidenceError),
    WorkflowRuntimeEvidence(TenantRbacWorkflowRuntimeEvidenceError),
    SloEvidence(TenantRbacSloEvidenceError),
    StatutoryFilingEvidence(TenantRbacStatutoryFilingEvidenceError),
    DisbursementEvidence(TenantRbacDisbursementEvidenceError),
    ErpParityMap(ErpParityMapError),
    CloudClaimBlocked(Vec<CloudReadinessBlocker>),
    CloudClaimMissingLocalGate(&'static str),
}

pub fn tenant_rbac_cloud_readiness_report()
-> Result<TenantRbacCloudReadinessReport, CloudReadinessGateError> {
    let composition = tenant_rbac_local_runtime_composition();
    validate_unique_method_paths(&composition)
        .map_err(CloudReadinessGateError::RouteComposition)?;

    let harness_capabilities = TenantRbacLocalInMemoryHarness::new().capabilities();
    let workflow_queue_capabilities = tenant_rbac_workflow_queue_capabilities();
    let workflow_runtime_evidence_plan = tenant_rbac_workflow_runtime_evidence_plan()
        .map_err(CloudReadinessGateError::WorkflowRuntimeEvidence)?;
    validate_tenant_rbac_workflow_runtime_evidence_plan(&workflow_runtime_evidence_plan)
        .map_err(CloudReadinessGateError::WorkflowRuntimeEvidence)?;
    let deployment_manifest = tenant_rbac_deployment_manifest();
    validate_cloud_deployment_manifest(&deployment_manifest)
        .map_err(CloudReadinessGateError::DeploymentManifest)?;
    let cloud_deployment_evidence_plan = tenant_rbac_deployment_evidence_plan()
        .map_err(CloudReadinessGateError::CloudDeploymentEvidence)?;
    validate_tenant_rbac_deployment_evidence_plan(&cloud_deployment_evidence_plan)
        .map_err(CloudReadinessGateError::CloudDeploymentEvidence)?;
    let tenant_workload_manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&tenant_workload_manifest)
        .map_err(CloudReadinessGateError::TenantWorkloadManifest)?;
    let tenant_admission_policy_contract = fd001_tenant_admission_policy_contract()
        .map_err(CloudReadinessGateError::TenantAdmissionPolicy)?;
    validate_fd001_tenant_admission_policy_contract(&tenant_admission_policy_contract)
        .map_err(CloudReadinessGateError::TenantAdmissionPolicy)?;
    let tenant_resource_quota_contract = fd001_tenant_resource_quota_contract()
        .map_err(CloudReadinessGateError::TenantResourceQuota)?;
    validate_fd001_tenant_resource_quota_contract(&tenant_resource_quota_contract)
        .map_err(CloudReadinessGateError::TenantResourceQuota)?;
    let tenant_availability_contract = fd001_tenant_availability_contract()
        .map_err(CloudReadinessGateError::TenantAvailability)?;
    validate_fd001_tenant_availability_contract(&tenant_availability_contract)
        .map_err(CloudReadinessGateError::TenantAvailability)?;
    let tenant_autoscaling_contract =
        fd001_tenant_autoscaling_contract().map_err(CloudReadinessGateError::TenantAutoscaling)?;
    validate_fd001_tenant_autoscaling_contract(&tenant_autoscaling_contract)
        .map_err(CloudReadinessGateError::TenantAutoscaling)?;
    let tenant_cost_allocation_contract = fd001_tenant_cost_allocation_contract()
        .map_err(CloudReadinessGateError::TenantCostAllocation)?;
    validate_fd001_tenant_cost_allocation_contract(&tenant_cost_allocation_contract)
        .map_err(CloudReadinessGateError::TenantCostAllocation)?;
    let tenant_residency_contract =
        fd001_tenant_residency_contract().map_err(CloudReadinessGateError::TenantResidency)?;
    validate_fd001_tenant_residency_contract(&tenant_residency_contract)
        .map_err(CloudReadinessGateError::TenantResidency)?;
    let tenant_workload_identity_contract = fd001_tenant_workload_identity_contract()
        .map_err(CloudReadinessGateError::TenantWorkloadIdentity)?;
    validate_fd001_tenant_workload_identity_contract(&tenant_workload_identity_contract)
        .map_err(CloudReadinessGateError::TenantWorkloadIdentity)?;
    let tenant_egress_policy_contract = fd001_tenant_egress_policy_contract()
        .map_err(CloudReadinessGateError::TenantEgressPolicy)?;
    validate_fd001_tenant_egress_policy_contract(&tenant_egress_policy_contract)
        .map_err(CloudReadinessGateError::TenantEgressPolicy)?;
    let tenant_image_provenance_contract = fd001_tenant_image_provenance_contract()
        .map_err(CloudReadinessGateError::TenantImageProvenance)?;
    validate_fd001_tenant_image_provenance_contract(&tenant_image_provenance_contract)
        .map_err(CloudReadinessGateError::TenantImageProvenance)?;
    let tenant_secret_boundary_contract = fd001_tenant_secret_boundary_contract()
        .map_err(CloudReadinessGateError::TenantSecretBoundary)?;
    validate_fd001_tenant_secret_boundary_contract(&tenant_secret_boundary_contract)
        .map_err(CloudReadinessGateError::TenantSecretBoundary)?;
    let tenant_workload_runtime_evidence_plan = tenant_rbac_tenant_workload_runtime_evidence_plan()
        .map_err(CloudReadinessGateError::TenantWorkloadRuntimeEvidence)?;
    validate_tenant_rbac_tenant_workload_runtime_evidence_plan(
        &tenant_workload_runtime_evidence_plan,
    )
    .map_err(CloudReadinessGateError::TenantWorkloadRuntimeEvidence)?;
    let audit_chain_plan = tenant_rbac_audit_chain_emission_plan();
    validate_tenant_rbac_audit_chain_emission_plan(&audit_chain_plan)
        .map_err(CloudReadinessGateError::AuditChainEmission)?;
    let audit_chain_runtime_evidence_plan = tenant_rbac_audit_chain_runtime_evidence_plan()
        .map_err(CloudReadinessGateError::AuditChainRuntimeEvidence)?;
    validate_tenant_rbac_audit_chain_runtime_evidence_plan(&audit_chain_runtime_evidence_plan)
        .map_err(CloudReadinessGateError::AuditChainRuntimeEvidence)?;
    let auth_policy = tenant_rbac_auth_runtime_policy();
    validate_tenant_rbac_auth_runtime_policy(&auth_policy)
        .map_err(CloudReadinessGateError::AuthRuntime)?;
    let identity_provider_plan = tenant_rbac_identity_provider_verification_plan();
    validate_tenant_rbac_identity_provider_verification_plan(&identity_provider_plan)
        .map_err(CloudReadinessGateError::IdentityProviderVerification)?;
    let identity_provider_runtime_evidence_plan =
        tenant_rbac_identity_provider_runtime_evidence_plan()
            .map_err(CloudReadinessGateError::IdentityProviderRuntimeEvidence)?;
    validate_tenant_rbac_identity_provider_runtime_evidence_plan(
        &identity_provider_runtime_evidence_plan,
    )
    .map_err(CloudReadinessGateError::IdentityProviderRuntimeEvidence)?;
    let storage_plan = tenant_rbac_postgres_rls_storage_plan();
    validate_tenant_rbac_postgres_rls_storage_plan(&storage_plan)
        .map_err(CloudReadinessGateError::PostgresRlsStorage)?;
    let postgres_write_contract = tenant_rbac_postgres_rls_write_contract()
        .map_err(CloudReadinessGateError::PostgresRlsWriteContract)?;
    validate_tenant_rbac_postgres_rls_write_contract(&postgres_write_contract)
        .map_err(CloudReadinessGateError::PostgresRlsWriteContract)?;
    let postgres_transaction_contract = tenant_rbac_postgres_rls_transaction_contract()
        .map_err(CloudReadinessGateError::PostgresRlsTransactionContract)?;
    validate_tenant_rbac_postgres_rls_transaction_contract(&postgres_transaction_contract)
        .map_err(CloudReadinessGateError::PostgresRlsTransactionContract)?;
    let postgres_runtime_evidence_plan = tenant_rbac_postgres_rls_runtime_evidence_plan()
        .map_err(CloudReadinessGateError::PostgresRlsRuntimeEvidence)?;
    validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&postgres_runtime_evidence_plan)
        .map_err(CloudReadinessGateError::PostgresRlsRuntimeEvidence)?;
    let listener_gateway_plan = tenant_rbac_listener_gateway_plan();
    validate_tenant_rbac_listener_gateway_plan(&listener_gateway_plan)
        .map_err(CloudReadinessGateError::ListenerGateway)?;
    let listener_runtime_evidence_plan = tenant_rbac_listener_runtime_evidence_plan()
        .map_err(CloudReadinessGateError::ListenerRuntimeEvidence)?;
    validate_tenant_rbac_listener_runtime_evidence_plan(&listener_runtime_evidence_plan)
        .map_err(CloudReadinessGateError::ListenerRuntimeEvidence)?;
    let slo_plan = tenant_rbac_slo_evidence_plan();
    validate_tenant_rbac_slo_evidence_plan(&slo_plan)
        .map_err(CloudReadinessGateError::SloEvidence)?;
    let statutory_filing_plan = tenant_rbac_statutory_filing_evidence_plan();
    validate_tenant_rbac_statutory_filing_evidence_plan(&statutory_filing_plan)
        .map_err(CloudReadinessGateError::StatutoryFilingEvidence)?;
    let disbursement_plan = tenant_rbac_disbursement_evidence_plan();
    validate_tenant_rbac_disbursement_evidence_plan(&disbursement_plan)
        .map_err(CloudReadinessGateError::DisbursementEvidence)?;
    let parity_rows = tenant_rbac_erp_parity_map();
    validate_erp_parity_map(parity_rows).map_err(CloudReadinessGateError::ErpParityMap)?;

    let route_catalog_ready = !composition.routes.is_empty()
        && !composition.deployed_listener_attached
        && !composition.authentication_runtime_attached
        && !composition.cloud_deployment_attached
        && !composition.runtime_audit_chain_emission_attached;
    let in_memory_harness_ready = harness_capabilities.in_memory_storage_integration_attached
        && !harness_capabilities.durable_storage_attached
        && !harness_capabilities.deployed_listener_attached
        && !harness_capabilities.cloud_deployment_attached
        && !harness_capabilities.runtime_audit_chain_emission_attached;
    let erp_parity_map_ready = parity_rows.len() == 23;
    let cloud_deployment_manifest_ready = deployment_manifest.service_name == "tenant-rbac"
        && !deployment_manifest.manual_kubectl_apply_allowed
        && !deployment_manifest.helm_cli_deploy_allowed
        && !deployment_manifest.cloud_deployment_evidence_attached
        && !deployment_manifest.production_slo_evidence_attached;
    let cloud_deployment_evidence_plan_ready = cloud_deployment_evidence_plan
        .fd001_product_delivery_master_goal_preserved
        && cloud_deployment_evidence_plan.oyatie_cloud_substrate_proof_required
        && cloud_deployment_evidence_plan.official_docs_required
        && cloud_deployment_evidence_plan.argocd_sync_evidence_required
        && cloud_deployment_evidence_plan.argocd_health_evidence_required
        && cloud_deployment_evidence_plan.git_revision_pin_required
        && cloud_deployment_evidence_plan.cosign_verification_required
        && cloud_deployment_evidence_plan.namespace_observation_required
        && cloud_deployment_evidence_plan.quota_observation_required
        && cloud_deployment_evidence_plan.network_policy_observation_required
        && cloud_deployment_evidence_plan.service_account_observation_required
        && cloud_deployment_evidence_plan.deployment_available_required
        && cloud_deployment_evidence_plan.readiness_probe_required
        && cloud_deployment_evidence_plan.gateway_route_acceptance_required
        && cloud_deployment_evidence_plan.otel_resource_identity_required
        && cloud_deployment_evidence_plan.deployment_audit_event_required
        && cloud_deployment_evidence_plan.rollback_plan_required
        && cloud_deployment_evidence_plan.review_only_contract
        && cloud_deployment_evidence_plan.requirements.len() >= 14
        && !cloud_deployment_evidence_plan.argocd_controller_attached
        && !cloud_deployment_evidence_plan.kubernetes_cluster_attached
        && !cloud_deployment_evidence_plan.namespace_created_attached
        && !cloud_deployment_evidence_plan.quota_applied_attached
        && !cloud_deployment_evidence_plan.network_policy_applied_attached
        && !cloud_deployment_evidence_plan.gateway_route_attached
        && !cloud_deployment_evidence_plan.workload_runtime_deployed_attached
        && !cloud_deployment_evidence_plan.runtime_otel_export_attached
        && !cloud_deployment_evidence_plan.runtime_audit_chain_emission_attached
        && !cloud_deployment_evidence_plan.production_cloud_deployment_evidence_attached;
    let tenant_workload_manifest_ready = tenant_workload_manifest.fd001_product_goal_preserved
        && tenant_workload_manifest.oyatie_cloud_substrate_only
        && tenant_workload_manifest.review_only_contract
        && tenant_workload_manifest.namespace_isolation_required
        && tenant_workload_manifest.resource_quota_required
        && tenant_workload_manifest.network_policy_required
        && tenant_workload_manifest.service_account_boundary_required
        && tenant_workload_manifest.gateway_route_required
        && tenant_workload_manifest.route_auth_scope_required
        && tenant_workload_manifest.tenant_claim_required
        && tenant_workload_manifest.legal_entity_claim_required
        && tenant_workload_manifest.otel_resource_identity_required
        && tenant_workload_manifest.per_workload_evidence_required
        && tenant_workload_manifest.workloads.len() >= 4
        && !tenant_workload_manifest.production_tenant_attached
        && !tenant_workload_manifest.kubernetes_namespace_created
        && !tenant_workload_manifest.resource_quota_applied
        && !tenant_workload_manifest.network_policy_applied
        && !tenant_workload_manifest.gateway_route_attached
        && !tenant_workload_manifest.workload_runtime_deployed
        && !tenant_workload_manifest.cloud_substrate_runtime_attached
        && !tenant_workload_manifest.runtime_audit_chain_emission_attached;
    let tenant_admission_policy_contract_ready = tenant_admission_policy_contract
        .official_docs_required
        && tenant_admission_policy_contract.validating_admission_policy_required
        && tenant_admission_policy_contract.admission_binding_required
        && tenant_admission_policy_contract.failure_policy_fail_required
        && tenant_admission_policy_contract.deny_action_required
        && tenant_admission_policy_contract.pod_security_restricted_required
        && tenant_admission_policy_contract.digest_pinned_image_required
        && tenant_admission_policy_contract.latest_image_tag_forbidden
        && tenant_admission_policy_contract.tenant_labels_required
        && tenant_admission_policy_contract.resource_requests_limits_required
        && tenant_admission_policy_contract.service_account_boundary_required
        && tenant_admission_policy_contract.default_service_account_forbidden
        && tenant_admission_policy_contract.automount_service_account_token_forbidden
        && tenant_admission_policy_contract.resource_quota_required
        && tenant_admission_policy_contract.network_policy_default_deny_required
        && tenant_admission_policy_contract.admission_audit_annotation_required
        && tenant_admission_policy_contract.review_only_contract
        && tenant_admission_policy_contract.all_manifest_workloads_in_scope
        && tenant_admission_policy_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_admission_policy_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_admission_policy_contract.rules.len() >= 10
        && !tenant_admission_policy_contract.kubernetes_cluster_attached
        && !tenant_admission_policy_contract.admission_controller_runtime_attached
        && !tenant_admission_policy_contract.admission_policy_applied
        && !tenant_admission_policy_contract.admission_runtime_enforced
        && !tenant_admission_policy_contract.workload_runtime_deployed
        && !tenant_admission_policy_contract.cloud_substrate_runtime_attached
        && !tenant_admission_policy_contract.runtime_audit_chain_emission_attached;
    let tenant_resource_quota_contract_ready = tenant_resource_quota_contract
        .official_docs_required
        && tenant_resource_quota_contract.all_manifest_workloads_in_scope
        && tenant_resource_quota_contract.namespace_resource_quota_required
        && tenant_resource_quota_contract.compute_requests_quota_required
        && tenant_resource_quota_contract.compute_limits_quota_required
        && tenant_resource_quota_contract.object_count_quota_required
        && tenant_resource_quota_contract.persistent_storage_quota_required
        && tenant_resource_quota_contract.limit_range_defaults_required
        && tenant_resource_quota_contract.limit_range_min_max_required
        && tenant_resource_quota_contract.container_requests_limits_required
        && tenant_resource_quota_contract.resource_quota_admission_evidence_required
        && tenant_resource_quota_contract.limit_ranger_admission_evidence_required
        && tenant_resource_quota_contract.tenant_label_selector_required
        && tenant_resource_quota_contract.quota_usage_audit_evidence_required
        && tenant_resource_quota_contract.admission_policy_evidence_required
        && tenant_resource_quota_contract.review_only_contract
        && tenant_resource_quota_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_resource_quota_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_resource_quota_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_resource_quota_contract.requirements.len() >= 13
        && !tenant_resource_quota_contract.kubernetes_cluster_attached
        && !tenant_resource_quota_contract.resource_quota_applied
        && !tenant_resource_quota_contract.limit_range_applied
        && !tenant_resource_quota_contract.quota_admission_runtime_attached
        && !tenant_resource_quota_contract.limit_ranger_runtime_attached
        && !tenant_resource_quota_contract.quota_usage_runtime_observed
        && !tenant_resource_quota_contract.workload_runtime_deployed
        && !tenant_resource_quota_contract.cloud_substrate_runtime_attached
        && !tenant_resource_quota_contract.runtime_audit_chain_emission_attached;
    let tenant_availability_contract_ready = tenant_availability_contract.official_docs_required
        && tenant_availability_contract.all_manifest_workloads_in_scope
        && tenant_availability_contract.pod_disruption_budget_required
        && tenant_availability_contract.minimum_available_budget_required
        && tenant_availability_contract.multi_replica_workload_required
        && tenant_availability_contract.zone_topology_spread_required
        && tenant_availability_contract.hostname_topology_spread_required
        && tenant_availability_contract.pod_anti_affinity_required
        && tenant_availability_contract.node_topology_label_evidence_required
        && tenant_availability_contract.rolling_update_availability_required
        && tenant_availability_contract.progress_deadline_required
        && tenant_availability_contract.readiness_probe_evidence_required
        && tenant_availability_contract.tenant_label_selector_required
        && tenant_availability_contract.disruption_audit_evidence_required
        && tenant_availability_contract.admission_policy_evidence_required
        && tenant_availability_contract.review_only_contract
        && tenant_availability_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_availability_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_availability_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_availability_contract.requirements.len() >= 13
        && !tenant_availability_contract.kubernetes_cluster_attached
        && !tenant_availability_contract.pod_disruption_budget_applied
        && !tenant_availability_contract.topology_spread_applied
        && !tenant_availability_contract.pod_anti_affinity_applied
        && !tenant_availability_contract.scheduler_runtime_observed
        && !tenant_availability_contract.rolling_update_runtime_observed
        && !tenant_availability_contract.readiness_probe_runtime_observed
        && !tenant_availability_contract.workload_runtime_deployed
        && !tenant_availability_contract.cloud_substrate_runtime_attached
        && !tenant_availability_contract.runtime_audit_chain_emission_attached;
    let tenant_autoscaling_contract_ready = tenant_autoscaling_contract.official_docs_required
        && tenant_autoscaling_contract.all_manifest_workloads_in_scope
        && tenant_autoscaling_contract.horizontal_pod_autoscaler_required
        && tenant_autoscaling_contract.autoscaling_v2_api_required
        && tenant_autoscaling_contract.min_replica_floor_required
        && tenant_autoscaling_contract.max_replica_ceiling_required
        && tenant_autoscaling_contract.cpu_resource_metric_required
        && tenant_autoscaling_contract.memory_resource_metric_required
        && tenant_autoscaling_contract.metrics_pipeline_evidence_required
        && tenant_autoscaling_contract.scale_up_behavior_policy_required
        && tenant_autoscaling_contract.scale_down_behavior_policy_required
        && tenant_autoscaling_contract.stabilization_window_required
        && tenant_autoscaling_contract.tenant_label_selector_required
        && tenant_autoscaling_contract.scaling_audit_evidence_required
        && tenant_autoscaling_contract.admission_policy_evidence_required
        && tenant_autoscaling_contract.review_only_contract
        && tenant_autoscaling_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_autoscaling_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_autoscaling_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_autoscaling_contract.requirements.len() >= 13
        && !tenant_autoscaling_contract.kubernetes_cluster_attached
        && !tenant_autoscaling_contract.metrics_server_runtime_attached
        && !tenant_autoscaling_contract.custom_metrics_api_attached
        && !tenant_autoscaling_contract.horizontal_pod_autoscaler_applied
        && !tenant_autoscaling_contract.autoscaling_controller_runtime_observed
        && !tenant_autoscaling_contract.scale_event_runtime_observed
        && !tenant_autoscaling_contract.workload_runtime_deployed
        && !tenant_autoscaling_contract.cloud_substrate_runtime_attached
        && !tenant_autoscaling_contract.runtime_audit_chain_emission_attached;
    let tenant_cost_allocation_contract_ready = tenant_cost_allocation_contract
        .official_docs_required
        && tenant_cost_allocation_contract.all_manifest_workloads_in_scope
        && tenant_cost_allocation_contract.tenant_cost_allocation_labels_required
        && tenant_cost_allocation_contract.kubernetes_recommended_labels_required
        && tenant_cost_allocation_contract.namespace_cost_boundary_required
        && tenant_cost_allocation_contract.workload_resource_requests_required
        && tenant_cost_allocation_contract.resource_quota_usage_evidence_required
        && tenant_cost_allocation_contract.opentelemetry_service_resource_required
        && tenant_cost_allocation_contract.opentelemetry_kubernetes_resource_attributes_required
        && tenant_cost_allocation_contract.finops_allocation_strategy_required
        && tenant_cost_allocation_contract.shared_cost_policy_required
        && tenant_cost_allocation_contract.allocation_coverage_kpi_required
        && tenant_cost_allocation_contract.tenant_label_selector_required
        && tenant_cost_allocation_contract.cost_allocation_audit_evidence_required
        && tenant_cost_allocation_contract.admission_policy_evidence_required
        && tenant_cost_allocation_contract.review_only_contract
        && tenant_cost_allocation_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_cost_allocation_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_cost_allocation_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_cost_allocation_contract.requirements.len() >= 13
        && !tenant_cost_allocation_contract.kubernetes_cluster_attached
        && !tenant_cost_allocation_contract.resource_metrics_runtime_attached
        && !tenant_cost_allocation_contract.otel_collector_runtime_attached
        && !tenant_cost_allocation_contract.finops_runtime_attached
        && !tenant_cost_allocation_contract.cost_report_runtime_generated
        && !tenant_cost_allocation_contract.billing_export_runtime_attached
        && !tenant_cost_allocation_contract.workload_runtime_deployed
        && !tenant_cost_allocation_contract.cloud_substrate_runtime_attached
        && !tenant_cost_allocation_contract.runtime_audit_chain_emission_attached;
    let tenant_residency_contract_ready = tenant_residency_contract.official_docs_required
        && tenant_residency_contract.all_manifest_workloads_in_scope
        && tenant_residency_contract.tenant_residency_region_label_required
        && tenant_residency_contract.namespace_residency_label_required
        && tenant_residency_contract.workload_node_affinity_required
        && tenant_residency_contract.topology_region_constraint_required
        && tenant_residency_contract.storage_residency_policy_ref_required
        && tenant_residency_contract.telemetry_residency_policy_ref_required
        && tenant_residency_contract.audit_residency_policy_ref_required
        && tenant_residency_contract.cross_region_egress_policy_ref_required
        && tenant_residency_contract.tenant_model_jurisdiction_ref_required
        && tenant_residency_contract.cell_placement_residency_ref_required
        && tenant_residency_contract.admission_policy_evidence_required
        && tenant_residency_contract.workload_manifest_evidence_required
        && tenant_residency_contract.residency_audit_evidence_required
        && tenant_residency_contract.review_only_contract
        && tenant_residency_contract.tenant_namespace == tenant_workload_manifest.tenant_namespace
        && tenant_residency_contract.tenant_cell_id == tenant_workload_manifest.tenant_cell_id
        && tenant_residency_contract.residency_region == tenant_workload_manifest.residency_region
        && tenant_residency_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_residency_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_residency_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_residency_contract.requirements.len() >= 13
        && !tenant_residency_contract.kubernetes_cluster_attached
        && !tenant_residency_contract.namespace_created
        && !tenant_residency_contract.node_affinity_applied
        && !tenant_residency_contract.scheduler_runtime_observed
        && !tenant_residency_contract.storage_residency_runtime_attached
        && !tenant_residency_contract.telemetry_residency_runtime_attached
        && !tenant_residency_contract.audit_residency_runtime_attached
        && !tenant_residency_contract.cross_region_egress_runtime_observed
        && !tenant_residency_contract.workload_runtime_deployed
        && !tenant_residency_contract.cloud_substrate_runtime_attached
        && !tenant_residency_contract.runtime_audit_chain_emission_attached;
    let tenant_workload_identity_contract_ready = tenant_workload_identity_contract
        .official_docs_required
        && tenant_workload_identity_contract.all_manifest_workloads_in_scope
        && tenant_workload_identity_contract.spiffe_id_required
        && tenant_workload_identity_contract.trust_domain_pinned
        && tenant_workload_identity_contract.x509_svid_required
        && tenant_workload_identity_contract.jwt_svid_policy_required
        && tenant_workload_identity_contract.mutual_tls_required
        && tenant_workload_identity_contract.gateway_backend_tls_policy_required
        && tenant_workload_identity_contract.certificate_rotation_evidence_required
        && tenant_workload_identity_contract.trust_bundle_evidence_required
        && tenant_workload_identity_contract.workload_api_boundary_required
        && tenant_workload_identity_contract.workload_attestation_selector_required
        && tenant_workload_identity_contract.service_telemetry_identity_required
        && tenant_workload_identity_contract.authorization_policy_binding_required
        && tenant_workload_identity_contract.identity_audit_evidence_required
        && tenant_workload_identity_contract.review_only_contract
        && tenant_workload_identity_contract.tenant_namespace
            == tenant_workload_manifest.tenant_namespace
        && tenant_workload_identity_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_workload_identity_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_workload_identity_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_workload_identity_contract.requirements.len() >= 13
        && !tenant_workload_identity_contract.kubernetes_cluster_attached
        && !tenant_workload_identity_contract.spiffe_workload_api_attached
        && !tenant_workload_identity_contract.spire_server_runtime_attached
        && !tenant_workload_identity_contract.spire_agent_runtime_attached
        && !tenant_workload_identity_contract.svid_runtime_issued
        && !tenant_workload_identity_contract.mtls_handshake_observed
        && !tenant_workload_identity_contract.certificate_rotation_runtime_observed
        && !tenant_workload_identity_contract.gateway_backend_tls_applied
        && !tenant_workload_identity_contract.authorization_policy_runtime_attached
        && !tenant_workload_identity_contract.workload_runtime_deployed
        && !tenant_workload_identity_contract.cloud_substrate_runtime_attached
        && !tenant_workload_identity_contract.runtime_audit_chain_emission_attached;
    let tenant_egress_policy_contract_ready = tenant_egress_policy_contract.official_docs_required
        && tenant_egress_policy_contract.all_manifest_workloads_in_scope
        && tenant_egress_policy_contract.default_deny_egress_required
        && tenant_egress_policy_contract.dns_egress_only_required
        && tenant_egress_policy_contract.same_namespace_service_egress_required
        && tenant_egress_policy_contract.cross_namespace_egress_explicit_selector_required
        && tenant_egress_policy_contract.external_cidr_egress_forbidden_by_default
        && tenant_egress_policy_contract.ip_block_exception_evidence_required
        && tenant_egress_policy_contract.protocol_port_pinned_required
        && tenant_egress_policy_contract.tenant_label_selector_required
        && tenant_egress_policy_contract.network_policy_provider_evidence_required
        && tenant_egress_policy_contract.egress_audit_evidence_required
        && tenant_egress_policy_contract.admission_policy_evidence_required
        && tenant_egress_policy_contract.review_only_contract
        && tenant_egress_policy_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_egress_policy_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_egress_policy_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_egress_policy_contract.rules.len() >= 11
        && !tenant_egress_policy_contract.kubernetes_cluster_attached
        && !tenant_egress_policy_contract.network_policy_provider_attached
        && !tenant_egress_policy_contract.network_policy_applied
        && !tenant_egress_policy_contract.egress_runtime_enforced
        && !tenant_egress_policy_contract.dns_probe_runtime_attached
        && !tenant_egress_policy_contract.external_egress_runtime_allowed
        && !tenant_egress_policy_contract.workload_runtime_deployed
        && !tenant_egress_policy_contract.cloud_substrate_runtime_attached
        && !tenant_egress_policy_contract.runtime_audit_chain_emission_attached;
    let tenant_image_provenance_contract_ready = tenant_image_provenance_contract
        .official_docs_required
        && tenant_image_provenance_contract.all_manifest_workloads_in_scope
        && tenant_image_provenance_contract.oci_digest_pinned_required
        && tenant_image_provenance_contract.cosign_signature_required
        && tenant_image_provenance_contract.keyless_oidc_identity_required
        && tenant_image_provenance_contract.transparency_log_required
        && tenant_image_provenance_contract.intoto_statement_required
        && tenant_image_provenance_contract.slsa_provenance_required
        && tenant_image_provenance_contract.builder_id_pin_required
        && tenant_image_provenance_contract.source_revision_pin_required
        && tenant_image_provenance_contract.sbom_required
        && tenant_image_provenance_contract.vulnerability_scan_gate_required
        && tenant_image_provenance_contract.admission_policy_evidence_required
        && tenant_image_provenance_contract.review_only_contract
        && tenant_image_provenance_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_image_provenance_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_image_provenance_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_image_provenance_contract.requirements.len() >= 11
        && !tenant_image_provenance_contract.image_registry_attached
        && !tenant_image_provenance_contract.image_published
        && !tenant_image_provenance_contract.cosign_runtime_verification_attached
        && !tenant_image_provenance_contract.transparency_log_runtime_verified
        && !tenant_image_provenance_contract.slsa_provenance_runtime_verified
        && !tenant_image_provenance_contract.sbom_runtime_published
        && !tenant_image_provenance_contract.vulnerability_scanner_attached
        && !tenant_image_provenance_contract.admission_controller_runtime_attached
        && !tenant_image_provenance_contract.workload_runtime_deployed
        && !tenant_image_provenance_contract.cloud_substrate_runtime_attached
        && !tenant_image_provenance_contract.runtime_audit_chain_emission_attached;
    let tenant_secret_boundary_contract_ready = tenant_secret_boundary_contract
        .official_docs_required
        && tenant_secret_boundary_contract.all_manifest_workloads_in_scope
        && tenant_secret_boundary_contract.inline_secret_material_forbidden
        && tenant_secret_boundary_contract.kubernetes_secret_reference_required
        && tenant_secret_boundary_contract.secret_at_rest_encryption_required
        && tenant_secret_boundary_contract.rbac_least_privilege_required
        && tenant_secret_boundary_contract.namespace_secret_isolation_required
        && tenant_secret_boundary_contract.workload_scoped_service_account_required
        && tenant_secret_boundary_contract.automount_service_account_token_forbidden
        && tenant_secret_boundary_contract.short_lived_projected_token_boundary_required
        && tenant_secret_boundary_contract.external_secret_store_boundary_required
        && tenant_secret_boundary_contract.secret_rotation_evidence_required
        && tenant_secret_boundary_contract.secret_access_audit_evidence_required
        && tenant_secret_boundary_contract.review_only_contract
        && tenant_secret_boundary_contract.workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_secret_boundary_contract.workload_manifest_count
            == tenant_workload_manifest.workloads.len()
        && tenant_secret_boundary_contract.tenant_admission_policy_contract_name
            == tenant_admission_policy_contract.contract_name
        && tenant_secret_boundary_contract.requirements.len() >= 11
        && !tenant_secret_boundary_contract.kubernetes_secret_created
        && !tenant_secret_boundary_contract.secret_data_materialized
        && !tenant_secret_boundary_contract.encryption_provider_runtime_attached
        && !tenant_secret_boundary_contract.external_secret_store_runtime_attached
        && !tenant_secret_boundary_contract.rbac_runtime_applied
        && !tenant_secret_boundary_contract.projected_token_runtime_attached
        && !tenant_secret_boundary_contract.secret_rotation_runtime_attached
        && !tenant_secret_boundary_contract.secret_access_runtime_audited
        && !tenant_secret_boundary_contract.admission_controller_runtime_attached
        && !tenant_secret_boundary_contract.workload_runtime_deployed
        && !tenant_secret_boundary_contract.cloud_substrate_runtime_attached
        && !tenant_secret_boundary_contract.runtime_audit_chain_emission_attached;
    let tenant_workload_runtime_evidence_plan_ready = tenant_workload_runtime_evidence_plan
        .fd001_product_delivery_master_goal_preserved
        && tenant_workload_runtime_evidence_plan.oyatie_cloud_substrate_proof_required
        && tenant_workload_runtime_evidence_plan.official_docs_required
        && tenant_workload_runtime_evidence_plan.tenant_namespace_runtime_evidence_required
        && tenant_workload_runtime_evidence_plan.per_workload_runtime_evidence_required
        && tenant_workload_runtime_evidence_plan.namespace_observation_required
        && tenant_workload_runtime_evidence_plan.resource_quota_usage_required
        && tenant_workload_runtime_evidence_plan.network_policy_default_deny_required
        && tenant_workload_runtime_evidence_plan.service_account_boundary_required
        && tenant_workload_runtime_evidence_plan.pod_security_context_required
        && tenant_workload_runtime_evidence_plan.workload_scheduled_required
        && tenant_workload_runtime_evidence_plan.resource_requests_limits_required
        && tenant_workload_runtime_evidence_plan.readiness_probe_required
        && tenant_workload_runtime_evidence_plan.liveness_probe_required
        && tenant_workload_runtime_evidence_plan.gateway_route_acceptance_required
        && tenant_workload_runtime_evidence_plan.tenant_claim_propagation_required
        && tenant_workload_runtime_evidence_plan.otel_resource_identity_required
        && tenant_workload_runtime_evidence_plan.rollout_recovery_required
        && tenant_workload_runtime_evidence_plan.workload_audit_event_required
        && tenant_workload_runtime_evidence_plan.review_only_contract
        && tenant_workload_runtime_evidence_plan.tenant_workload_manifest_name
            == tenant_workload_manifest.manifest_name
        && tenant_workload_runtime_evidence_plan.manifest_workload_count
            == tenant_workload_manifest.workloads.len()
        && tenant_workload_runtime_evidence_plan.requirements.len() >= 14
        && !tenant_workload_runtime_evidence_plan.production_tenant_attached
        && !tenant_workload_runtime_evidence_plan.kubernetes_runtime_attached
        && !tenant_workload_runtime_evidence_plan.workload_runtime_deployed_attached
        && !tenant_workload_runtime_evidence_plan.gateway_controller_attached
        && !tenant_workload_runtime_evidence_plan.cloud_substrate_runtime_attached
        && !tenant_workload_runtime_evidence_plan.runtime_audit_chain_emission_attached
        && !tenant_workload_runtime_evidence_plan.production_workload_evidence_attached;
    let authentication_runtime_ready = auth_policy.deny_by_default
        && auth_policy.tenant_isolation_required
        && auth_policy.mfa_for_sensitive_required
        && auth_policy.break_glass_audit_required
        && !auth_policy.external_identity_provider_attached
        && !auth_policy.oidc_signature_verification_attached
        && !auth_policy.jwks_provider_attached;
    let identity_provider_verification_plan_ready = identity_provider_plan.oidc_discovery_required
        && identity_provider_plan.jwks_required
        && identity_provider_plan.tls_required
        && identity_provider_plan.issuer_match_required
        && identity_provider_plan.audience_match_required
        && identity_provider_plan.expiration_required
        && identity_provider_plan.not_before_and_issued_at_checked
        && identity_provider_plan.nonce_required
        && identity_provider_plan.key_id_required
        && identity_provider_plan.alg_none_forbidden
        && identity_provider_plan.symmetric_algorithms_forbidden
        && identity_provider_plan.tenant_claim_required
        && identity_provider_plan.subject_claim_required
        && identity_provider_plan.mfa_claim_required_for_sensitive_routes
        && identity_provider_plan.route_policy_scope_alignment_required
        && !identity_provider_plan.discovery_fetch_runtime_attached
        && !identity_provider_plan.jwks_fetch_runtime_attached
        && !identity_provider_plan.oidc_signature_verification_attached
        && !identity_provider_plan.external_identity_provider_attached
        && !identity_provider_plan.token_introspection_attached
        && !identity_provider_plan.durable_session_store_attached
        && !identity_provider_plan.runtime_auth_middleware_attached
        && !identity_provider_plan.cloud_gateway_enforcement_attached
        && !identity_provider_plan.runtime_audit_chain_emission_attached;
    let identity_provider_runtime_evidence_plan_ready = identity_provider_runtime_evidence_plan
        .fd001_product_delivery_master_goal_preserved
        && identity_provider_runtime_evidence_plan.oyatie_cloud_substrate_proof_required
        && identity_provider_runtime_evidence_plan.official_docs_required
        && identity_provider_runtime_evidence_plan.discovery_document_observation_required
        && identity_provider_runtime_evidence_plan.issuer_metadata_match_required
        && identity_provider_runtime_evidence_plan.jwks_fetch_evidence_required
        && identity_provider_runtime_evidence_plan.jwks_kid_match_required
        && identity_provider_runtime_evidence_plan.jwt_signature_verification_evidence_required
        && identity_provider_runtime_evidence_plan.algorithm_allowlist_required
        && identity_provider_runtime_evidence_plan.issuer_claim_match_required
        && identity_provider_runtime_evidence_plan.audience_claim_match_required
        && identity_provider_runtime_evidence_plan.temporal_claims_check_required
        && identity_provider_runtime_evidence_plan.nonce_replay_denial_required
        && identity_provider_runtime_evidence_plan.tenant_claim_mapping_required
        && identity_provider_runtime_evidence_plan.route_scope_authorization_required
        && identity_provider_runtime_evidence_plan.sensitive_route_mfa_enforcement_required
        && identity_provider_runtime_evidence_plan.key_rotation_overlap_evidence_required
        && identity_provider_runtime_evidence_plan.auth_failure_audit_event_required
        && identity_provider_runtime_evidence_plan.review_only_contract
        && identity_provider_runtime_evidence_plan.identity_provider_verification_plan_name
            == identity_provider_plan.plan_name
        && identity_provider_runtime_evidence_plan.issuer == identity_provider_plan.issuer
        && identity_provider_runtime_evidence_plan.audience == identity_provider_plan.audience
        && identity_provider_runtime_evidence_plan.requirements.len() >= 15
        && !identity_provider_runtime_evidence_plan.discovery_fetch_runtime_attached
        && !identity_provider_runtime_evidence_plan.jwks_fetch_runtime_attached
        && !identity_provider_runtime_evidence_plan.oidc_signature_verification_attached
        && !identity_provider_runtime_evidence_plan.external_identity_provider_attached
        && !identity_provider_runtime_evidence_plan.token_introspection_attached
        && !identity_provider_runtime_evidence_plan.durable_session_store_attached
        && !identity_provider_runtime_evidence_plan.runtime_auth_middleware_attached
        && !identity_provider_runtime_evidence_plan.cloud_gateway_enforcement_attached
        && !identity_provider_runtime_evidence_plan.runtime_audit_chain_emission_attached
        && !identity_provider_runtime_evidence_plan.production_identity_provider_evidence_attached;
    let postgres_rls_storage_plan_ready = storage_plan.default_deny_when_policy_missing
        && storage_plan.owner_force_rls_required
        && storage_plan.bypassrls_role_forbidden
        && storage_plan.migration_sql_review_only
        && !storage_plan.runtime_database_attached
        && !storage_plan.postgres_connection_attached
        && !storage_plan.migration_applied_attached
        && !storage_plan.rls_runtime_verified_attached
        && !storage_plan.durable_storage_runtime_attached;
    let postgres_rls_write_contract_ready = postgres_write_contract.official_docs_required
        && postgres_write_contract.set_local_tenant_context_required
        && postgres_write_contract.parameterized_insert_required
        && postgres_write_contract.idempotency_conflict_do_nothing_required
        && postgres_write_contract.tenant_scoped_readback_required
        && postgres_write_contract.schema_version_return_required
        && postgres_write_contract.delete_statement_forbidden
        && postgres_write_contract.review_only_contract
        && postgres_write_contract.storage_plan_table_count == storage_plan.tables.len()
        && postgres_write_contract.statements.len() >= storage_plan.tables.len()
        && !postgres_write_contract.database_connection_attached
        && !postgres_write_contract.prepared_statement_runtime_attached
        && !postgres_write_contract.write_runtime_attached
        && !postgres_write_contract.durable_storage_runtime_attached
        && !postgres_write_contract.runtime_audit_chain_emission_attached;
    let postgres_rls_transaction_contract_ready = postgres_transaction_contract
        .official_docs_required
        && postgres_transaction_contract.explicit_transaction_required
        && postgres_transaction_contract.transaction_local_tenant_context_required
        && postgres_transaction_contract.prepared_statement_required
        && postgres_transaction_contract.bound_parameter_execution_required
        && postgres_transaction_contract.tenant_scoped_readback_required
        && postgres_transaction_contract.commit_after_readback_required
        && postgres_transaction_contract.rollback_on_error_required
        && postgres_transaction_contract.autocommit_write_forbidden
        && postgres_transaction_contract.review_only_contract
        && postgres_transaction_contract.write_contract_statement_count
            == postgres_write_contract.statements.len()
        && postgres_transaction_contract.transaction_plans.len()
            >= postgres_write_contract.statements.len()
        && !postgres_transaction_contract.database_connection_attached
        && !postgres_transaction_contract.transaction_runtime_attached
        && !postgres_transaction_contract.prepared_statement_runtime_attached
        && !postgres_transaction_contract.write_runtime_attached
        && !postgres_transaction_contract.durable_storage_runtime_attached
        && !postgres_transaction_contract.runtime_audit_chain_emission_attached;
    let postgres_rls_runtime_evidence_plan_ready = postgres_runtime_evidence_plan
        .official_docs_required
        && postgres_runtime_evidence_plan.tls_verify_full_required
        && postgres_runtime_evidence_plan.migration_digest_required
        && postgres_runtime_evidence_plan.migration_transaction_required
        && postgres_runtime_evidence_plan.role_matrix_required
        && postgres_runtime_evidence_plan.rls_probe_matrix_required
        && postgres_runtime_evidence_plan.tenant_cross_read_denial_required
        && postgres_runtime_evidence_plan.tenant_cross_write_denial_required
        && postgres_runtime_evidence_plan.delete_forbidden_probe_required
        && postgres_runtime_evidence_plan.bypassrls_absence_required
        && postgres_runtime_evidence_plan.backup_restore_rehearsal_required
        && postgres_runtime_evidence_plan.pitr_rehearsal_required
        && postgres_runtime_evidence_plan.storage_plan_table_count == storage_plan.tables.len()
        && postgres_runtime_evidence_plan.probes.len() >= 11
        && !postgres_runtime_evidence_plan.runtime_database_attached
        && !postgres_runtime_evidence_plan.postgres_connection_attached
        && !postgres_runtime_evidence_plan.migration_applied_attached
        && !postgres_runtime_evidence_plan.rls_runtime_verified_attached
        && !postgres_runtime_evidence_plan.durable_storage_runtime_attached
        && !postgres_runtime_evidence_plan.cloud_database_attached
        && !postgres_runtime_evidence_plan.production_backup_restore_attached
        && !postgres_runtime_evidence_plan.runtime_audit_chain_emission_attached;
    let audit_chain_emission_plan_ready = audit_chain_plan.cloud_events_json_required
        && audit_chain_plan.w3c_trace_context_required
        && audit_chain_plan.opentelemetry_log_mapping_required
        && audit_chain_plan.traceparent_required
        && audit_chain_plan.tenant_partition_required
        && audit_chain_plan.idempotency_key_required
        && audit_chain_plan.payload_digest_required
        && audit_chain_plan.source_evidence_ref_required
        && audit_chain_plan.merkle_seal_required
        && audit_chain_plan.write_ahead_log_required
        && audit_chain_plan.broker_outbox_required
        && audit_chain_plan.sensitive_context_forbidden
        && audit_chain_plan.raw_payload_storage_forbidden
        && audit_chain_plan.credential_material_forbidden
        && audit_chain_plan.event_schemas.len() == 9
        && !audit_chain_plan.runtime_emitter_attached
        && !audit_chain_plan.write_ahead_log_runtime_attached
        && !audit_chain_plan.broker_publish_runtime_attached
        && !audit_chain_plan.merkle_sealer_runtime_attached
        && !audit_chain_plan.cloud_audit_sink_attached
        && !audit_chain_plan.runtime_audit_chain_emission_attached;
    let audit_chain_runtime_evidence_plan_ready = audit_chain_runtime_evidence_plan
        .fd001_product_delivery_master_goal_preserved
        && audit_chain_runtime_evidence_plan.oyatie_cloud_substrate_proof_required
        && audit_chain_runtime_evidence_plan.official_docs_required
        && audit_chain_runtime_evidence_plan.cloudevents_envelope_evidence_required
        && audit_chain_runtime_evidence_plan.trace_context_evidence_required
        && audit_chain_runtime_evidence_plan.otel_log_record_mapping_required
        && audit_chain_runtime_evidence_plan.tenant_partition_evidence_required
        && audit_chain_runtime_evidence_plan.idempotency_dedupe_evidence_required
        && audit_chain_runtime_evidence_plan.payload_digest_match_required
        && audit_chain_runtime_evidence_plan.sensitive_payload_redaction_required
        && audit_chain_runtime_evidence_plan.wal_append_evidence_required
        && audit_chain_runtime_evidence_plan.outbox_publish_evidence_required
        && audit_chain_runtime_evidence_plan.broker_ack_evidence_required
        && audit_chain_runtime_evidence_plan.merkle_leaf_inclusion_required
        && audit_chain_runtime_evidence_plan.merkle_root_seal_required
        && audit_chain_runtime_evidence_plan.sink_ingestion_required
        && audit_chain_runtime_evidence_plan.replay_recovery_required
        && audit_chain_runtime_evidence_plan.failure_path_audit_required
        && audit_chain_runtime_evidence_plan.review_only_contract
        && audit_chain_runtime_evidence_plan.audit_chain_emission_plan_name
            == audit_chain_plan.plan_name
        && audit_chain_runtime_evidence_plan.outbox_topic == audit_chain_plan.outbox_topic
        && audit_chain_runtime_evidence_plan.event_schema_count
            == audit_chain_plan.event_schemas.len()
        && audit_chain_runtime_evidence_plan.requirements.len() >= 15
        && !audit_chain_runtime_evidence_plan.runtime_emitter_attached
        && !audit_chain_runtime_evidence_plan.write_ahead_log_runtime_attached
        && !audit_chain_runtime_evidence_plan.broker_publish_runtime_attached
        && !audit_chain_runtime_evidence_plan.merkle_sealer_runtime_attached
        && !audit_chain_runtime_evidence_plan.cloud_audit_sink_attached
        && !audit_chain_runtime_evidence_plan.runtime_audit_chain_emission_attached
        && !audit_chain_runtime_evidence_plan.production_audit_emission_evidence_attached;
    let listener_gateway_plan_ready = listener_gateway_plan.gateway_api_required
        && listener_gateway_plan.ingress_tls_required
        && listener_gateway_plan.network_policy_required
        && listener_gateway_plan.authz_required
        && listener_gateway_plan.deny_by_default_required
        && listener_gateway_plan.kubernetes_service_type == "ClusterIP"
        && listener_gateway_plan.route_count == composition.routes.len()
        && listener_gateway_plan.auth_policy_route_count == auth_policy.route_policies.len()
        && !listener_gateway_plan.direct_public_node_port_allowed
        && !listener_gateway_plan.direct_public_load_balancer_allowed
        && !listener_gateway_plan.deployed_listener_attached
        && !listener_gateway_plan.gateway_controller_attached
        && !listener_gateway_plan.load_balancer_attached
        && !listener_gateway_plan.tls_certificate_attached
        && !listener_gateway_plan.runtime_auth_middleware_attached
        && !listener_gateway_plan.cloud_deployment_evidence_attached
        && !listener_gateway_plan.production_slo_evidence_attached
        && !listener_gateway_plan.runtime_audit_chain_emission_attached;
    let listener_runtime_evidence_plan_ready = listener_runtime_evidence_plan
        .fd001_product_delivery_master_goal_preserved
        && listener_runtime_evidence_plan.oyatie_cloud_substrate_proof_required
        && listener_runtime_evidence_plan.official_docs_required
        && listener_runtime_evidence_plan.cluster_ip_service_observation_required
        && listener_runtime_evidence_plan.gateway_route_acceptance_required
        && listener_runtime_evidence_plan.tls_certificate_binding_required
        && listener_runtime_evidence_plan.readiness_probe_success_required
        && listener_runtime_evidence_plan.liveness_probe_success_required
        && listener_runtime_evidence_plan.synthetic_health_check_required
        && listener_runtime_evidence_plan.route_authz_enforcement_required
        && listener_runtime_evidence_plan.default_deny_network_policy_required
        && listener_runtime_evidence_plan.endpoint_slice_ready_required
        && listener_runtime_evidence_plan.graceful_shutdown_drain_required
        && listener_runtime_evidence_plan.access_log_trace_correlation_required
        && listener_runtime_evidence_plan.listener_deployment_audit_event_required
        && listener_runtime_evidence_plan.review_only_contract
        && listener_runtime_evidence_plan.listener_gateway_route_count == composition.routes.len()
        && listener_runtime_evidence_plan.listener_gateway_service_type == "ClusterIP"
        && listener_runtime_evidence_plan.requirements.len() >= 12
        && !listener_runtime_evidence_plan.deployed_listener_attached
        && !listener_runtime_evidence_plan.bound_socket_attached
        && !listener_runtime_evidence_plan.gateway_controller_attached
        && !listener_runtime_evidence_plan.load_balancer_attached
        && !listener_runtime_evidence_plan.tls_certificate_attached
        && !listener_runtime_evidence_plan.runtime_auth_middleware_attached
        && !listener_runtime_evidence_plan.network_policy_applied_attached
        && !listener_runtime_evidence_plan.readiness_probe_runtime_attached
        && !listener_runtime_evidence_plan.liveness_probe_runtime_attached
        && !listener_runtime_evidence_plan.production_listener_evidence_attached
        && !listener_runtime_evidence_plan.runtime_audit_chain_emission_attached;
    let slo_evidence_plan_ready = slo_plan.error_budget_release_gate_required
        && slo_plan.multi_window_burn_rate_alert_required
        && slo_plan.openslo_manifests_required
        && slo_plan.otel_metric_streams_required
        && slo_plan.objectives.len() >= 4
        && !slo_plan.runtime_otel_export_attached
        && !slo_plan.metrics_backend_attached
        && !slo_plan.alert_manager_attached
        && !slo_plan.canary_runtime_attached
        && !slo_plan.rollback_automation_attached
        && !slo_plan.production_slo_evidence_attached
        && !slo_plan.multi_region_slo_evidence_attached;
    let workflow_execution_reference_ready = workflow_queue_capabilities
        .in_memory_execution_reference_attached
        && !workflow_queue_capabilities.workflow_engine_attached
        && !workflow_queue_capabilities.broker_publish_attached
        && !workflow_queue_capabilities.durable_queue_attached
        && !workflow_queue_capabilities.runtime_execution_attached
        && !workflow_queue_capabilities.downstream_service_calls_attached
        && !workflow_queue_capabilities.audit_chain_emission_attached;
    let workflow_runtime_evidence_plan_ready = workflow_runtime_evidence_plan
        .fd001_product_delivery_master_goal_preserved
        && workflow_runtime_evidence_plan.oyatie_cloud_substrate_proof_required
        && workflow_runtime_evidence_plan.official_docs_required
        && workflow_runtime_evidence_plan.workflow_definition_version_pin_required
        && workflow_runtime_evidence_plan.deterministic_gate_evidence_required
        && workflow_runtime_evidence_plan.dispatch_idempotency_required
        && workflow_runtime_evidence_plan.execution_state_transition_required
        && workflow_runtime_evidence_plan.durable_queue_ack_required
        && workflow_runtime_evidence_plan.broker_publish_confirmation_required
        && workflow_runtime_evidence_plan.broker_retry_or_dlq_required
        && workflow_runtime_evidence_plan.tenant_partition_required
        && workflow_runtime_evidence_plan.payload_digest_required
        && workflow_runtime_evidence_plan.downstream_service_boundary_required
        && workflow_runtime_evidence_plan.otel_messaging_trace_required
        && workflow_runtime_evidence_plan.workflow_audit_event_required
        && workflow_runtime_evidence_plan.replay_recovery_required
        && workflow_runtime_evidence_plan.review_only_contract
        && workflow_runtime_evidence_plan.workflow_queue_adapter
            == workflow_queue_capabilities.adapter.as_str()
        && workflow_runtime_evidence_plan.in_memory_execution_reference_attached
            == workflow_queue_capabilities.in_memory_execution_reference_attached
        && workflow_runtime_evidence_plan.requirements.len() >= 14
        && !workflow_runtime_evidence_plan.workflow_engine_runtime_attached
        && !workflow_runtime_evidence_plan.broker_publish_runtime_attached
        && !workflow_runtime_evidence_plan.durable_queue_runtime_attached
        && !workflow_runtime_evidence_plan.downstream_service_calls_runtime_attached
        && !workflow_runtime_evidence_plan.cloud_workflow_runtime_attached
        && !workflow_runtime_evidence_plan.runtime_otel_export_attached
        && !workflow_runtime_evidence_plan.runtime_audit_chain_emission_attached
        && !workflow_runtime_evidence_plan.production_workflow_evidence_attached;
    let statutory_filing_evidence_plan_ready = statutory_filing_plan
        .source_rulepack_manifests_required
        && statutory_filing_plan.authority_endpoint_registry_required
        && statutory_filing_plan.payload_digest_required
        && statutory_filing_plan.agency_receipt_required
        && statutory_filing_plan.legal_entity_isolation_required
        && statutory_filing_plan.credential_attestation_required
        && statutory_filing_plan.human_approval_required
        && !statutory_filing_plan.manual_submission_workaround_allowed
        && statutory_filing_plan.requirements.len() >= 4
        && !statutory_filing_plan.runtime_submission_attached
        && !statutory_filing_plan.agency_credential_attached
        && !statutory_filing_plan.agency_connection_attached
        && !statutory_filing_plan.filing_rail_runtime_attached
        && !statutory_filing_plan.disbursement_rail_attached
        && !statutory_filing_plan.tax_payment_execution_attached
        && !statutory_filing_plan.durable_statutory_archive_attached
        && !statutory_filing_plan.cloud_deployment_attached
        && !statutory_filing_plan.production_filing_evidence_attached
        && !statutory_filing_plan.runtime_audit_chain_emission_attached;
    let disbursement_evidence_plan_ready = disbursement_plan
        .source_rulepack_or_invoice_evidence_required
        && disbursement_plan.bank_network_registry_required
        && disbursement_plan.payment_file_digest_required
        && disbursement_plan.beneficiary_tokenization_required
        && disbursement_plan.approval_workflow_required
        && disbursement_plan.segregation_of_duties_required
        && disbursement_plan.dual_approval_required
        && disbursement_plan.reconciliation_receipt_required
        && disbursement_plan.rollback_or_reversal_runbook_required
        && !disbursement_plan.manual_bank_portal_workaround_allowed
        && disbursement_plan.requirements.len() >= 4
        && !disbursement_plan.runtime_payment_execution_attached
        && !disbursement_plan.bank_credential_attached
        && !disbursement_plan.bank_connection_attached
        && !disbursement_plan.disbursement_rail_runtime_attached
        && !disbursement_plan.tax_payment_execution_attached
        && !disbursement_plan.durable_payment_archive_attached
        && !disbursement_plan.cloud_deployment_attached
        && !disbursement_plan.production_disbursement_evidence_attached
        && !disbursement_plan.runtime_audit_chain_emission_attached;
    let local_rehearsal_ready = route_catalog_ready
        && in_memory_harness_ready
        && erp_parity_map_ready
        && cloud_deployment_manifest_ready
        && cloud_deployment_evidence_plan_ready
        && tenant_workload_manifest_ready
        && tenant_admission_policy_contract_ready
        && tenant_resource_quota_contract_ready
        && tenant_availability_contract_ready
        && tenant_autoscaling_contract_ready
        && tenant_cost_allocation_contract_ready
        && tenant_residency_contract_ready
        && tenant_workload_identity_contract_ready
        && tenant_egress_policy_contract_ready
        && tenant_image_provenance_contract_ready
        && tenant_secret_boundary_contract_ready
        && tenant_workload_runtime_evidence_plan_ready
        && authentication_runtime_ready
        && identity_provider_verification_plan_ready
        && identity_provider_runtime_evidence_plan_ready
        && postgres_rls_storage_plan_ready
        && postgres_rls_write_contract_ready
        && postgres_rls_transaction_contract_ready
        && postgres_rls_runtime_evidence_plan_ready
        && listener_gateway_plan_ready
        && listener_runtime_evidence_plan_ready
        && audit_chain_emission_plan_ready
        && audit_chain_runtime_evidence_plan_ready
        && workflow_execution_reference_ready
        && workflow_runtime_evidence_plan_ready
        && statutory_filing_evidence_plan_ready
        && disbursement_evidence_plan_ready
        && slo_evidence_plan_ready;
    let blockers = required_cloud_blockers();
    let cloud_deployment_ready = local_rehearsal_ready && blockers.is_empty();

    Ok(TenantRbacCloudReadinessReport {
        report_name: "tenant-rbac-readiness-gate",
        route_count: composition.routes.len(),
        sap_module_count: parity_rows.len(),
        route_catalog_ready,
        in_memory_harness_ready,
        erp_parity_map_ready,
        cloud_deployment_manifest_ready,
        cloud_deployment_evidence_plan_ready,
        cloud_deployment_evidence_requirement_count: cloud_deployment_evidence_plan
            .requirements
            .len(),
        tenant_workload_manifest_ready,
        tenant_workload_count: tenant_workload_manifest.workloads.len(),
        tenant_admission_policy_contract_ready,
        tenant_admission_policy_rule_count: tenant_admission_policy_contract.rules.len(),
        tenant_admission_policy_all_workloads_in_scope: tenant_admission_policy_contract
            .all_manifest_workloads_in_scope,
        tenant_resource_quota_contract_ready,
        tenant_resource_quota_requirement_count: tenant_resource_quota_contract.requirements.len(),
        tenant_resource_quota_all_workloads_in_scope: tenant_resource_quota_contract
            .all_manifest_workloads_in_scope,
        tenant_availability_contract_ready,
        tenant_availability_requirement_count: tenant_availability_contract.requirements.len(),
        tenant_availability_all_workloads_in_scope: tenant_availability_contract
            .all_manifest_workloads_in_scope,
        tenant_autoscaling_contract_ready,
        tenant_autoscaling_requirement_count: tenant_autoscaling_contract.requirements.len(),
        tenant_autoscaling_all_workloads_in_scope: tenant_autoscaling_contract
            .all_manifest_workloads_in_scope,
        tenant_cost_allocation_contract_ready,
        tenant_cost_allocation_requirement_count: tenant_cost_allocation_contract
            .requirements
            .len(),
        tenant_cost_allocation_all_workloads_in_scope: tenant_cost_allocation_contract
            .all_manifest_workloads_in_scope,
        tenant_residency_contract_ready,
        tenant_residency_requirement_count: tenant_residency_contract.requirements.len(),
        tenant_residency_all_workloads_in_scope: tenant_residency_contract
            .all_manifest_workloads_in_scope,
        tenant_workload_identity_contract_ready,
        tenant_workload_identity_requirement_count: tenant_workload_identity_contract
            .requirements
            .len(),
        tenant_workload_identity_all_workloads_in_scope: tenant_workload_identity_contract
            .all_manifest_workloads_in_scope,
        tenant_egress_policy_contract_ready,
        tenant_egress_policy_rule_count: tenant_egress_policy_contract.rules.len(),
        tenant_egress_policy_all_workloads_in_scope: tenant_egress_policy_contract
            .all_manifest_workloads_in_scope,
        tenant_image_provenance_contract_ready,
        tenant_image_provenance_requirement_count: tenant_image_provenance_contract
            .requirements
            .len(),
        tenant_image_provenance_all_workloads_in_scope: tenant_image_provenance_contract
            .all_manifest_workloads_in_scope,
        tenant_secret_boundary_contract_ready,
        tenant_secret_boundary_requirement_count: tenant_secret_boundary_contract
            .requirements
            .len(),
        tenant_secret_boundary_all_workloads_in_scope: tenant_secret_boundary_contract
            .all_manifest_workloads_in_scope,
        tenant_workload_runtime_evidence_plan_ready,
        tenant_workload_runtime_evidence_requirement_count: tenant_workload_runtime_evidence_plan
            .requirements
            .len(),
        authentication_runtime_ready,
        identity_provider_verification_plan_ready,
        identity_provider_runtime_evidence_plan_ready,
        identity_provider_runtime_evidence_requirement_count:
            identity_provider_runtime_evidence_plan.requirements.len(),
        postgres_rls_storage_plan_ready,
        postgres_rls_write_contract_ready,
        postgres_rls_write_statement_count: postgres_write_contract.statements.len(),
        postgres_rls_transaction_contract_ready,
        postgres_rls_transaction_plan_count: postgres_transaction_contract.transaction_plans.len(),
        postgres_rls_runtime_evidence_plan_ready,
        listener_gateway_plan_ready,
        listener_runtime_evidence_plan_ready,
        listener_runtime_evidence_requirement_count: listener_runtime_evidence_plan
            .requirements
            .len(),
        audit_chain_emission_plan_ready,
        audit_chain_runtime_evidence_plan_ready,
        audit_chain_runtime_evidence_requirement_count: audit_chain_runtime_evidence_plan
            .requirements
            .len(),
        workflow_execution_reference_ready,
        workflow_runtime_evidence_plan_ready,
        workflow_runtime_evidence_requirement_count: workflow_runtime_evidence_plan
            .requirements
            .len(),
        statutory_filing_evidence_plan_ready,
        disbursement_evidence_plan_ready,
        slo_evidence_plan_ready,
        local_rehearsal_ready,
        cloud_deployment_ready,
        blocker_count: blockers.len(),
        blockers,
        evidence_refs: vec![
            "evidence/multispectrum/cs-ent-platform-local-runtime-composition-1779541200.json",
            "evidence/multispectrum/cs-ent-platform-local-inmemory-harness-1779541800.json",
            "evidence/multispectrum/cs-ent-platform-erp-parity-map-1779542400.json",
            "evidence/multispectrum/cs-ent-platform-cloud-deployment-manifest-1779551400.json",
            "evidence/multispectrum/cs-ent-platform-cloud-deployment-evidence-1779702000.json",
            "evidence/multispectrum/cs-ent-platform-tenant-workload-manifest-1779701400.json",
            "evidence/multispectrum/cs-ent-platform-tenant-admission-policy-1779706800.json",
            "evidence/multispectrum/cs-ent-platform-tenant-resource-quota-contract-1779709200.json",
            "evidence/multispectrum/cs-ent-platform-tenant-availability-contract-1779709800.json",
            "evidence/multispectrum/cs-ent-platform-tenant-autoscaling-contract-1779710400.json",
            "evidence/multispectrum/cs-ent-platform-tenant-cost-allocation-contract-1779711000.json",
            "evidence/multispectrum/cs-ent-platform-tenant-residency-contract-1779711600.json",
            "evidence/multispectrum/cs-ent-platform-tenant-workload-identity-contract-1779712200.json",
            "evidence/multispectrum/cs-ent-platform-tenant-egress-policy-contract-1779708600.json",
            "evidence/multispectrum/cs-ent-platform-image-provenance-contract-1779707400.json",
            "evidence/multispectrum/cs-ent-platform-secret-boundary-contract-1779708000.json",
            "evidence/multispectrum/cs-ent-platform-tenant-runtime-evidence-1779705000.json",
            "evidence/multispectrum/cs-ent-platform-auth-runtime-1779552000.json",
            "evidence/multispectrum/cs-ent-platform-idp-verification-1779553800.json",
            "evidence/multispectrum/cs-ent-platform-idp-runtime-evidence-1779703200.json",
            "evidence/multispectrum/cs-ent-platform-audit-chain-emission-1779661200.json",
            "evidence/multispectrum/cs-ent-platform-audit-runtime-evidence-1779704400.json",
            "evidence/multispectrum/cs-ent-platform-postgres-rls-storage-1779552600.json",
            "evidence/multispectrum/cs-ent-platform-postgres-write-contract-1779705600.json",
            "evidence/multispectrum/cs-ent-platform-postgres-tx-contract-1779706200.json",
            "evidence/multispectrum/cs-ent-platform-postgres-rls-runtime-evidence-1779666600.json",
            "evidence/multispectrum/cs-ent-platform-listener-gateway-1779553200.json",
            "evidence/multispectrum/cs-ent-platform-listener-runtime-evidence-1779702600.json",
            "evidence/multispectrum/cs-ent-platform-slo-evidence-1779664200.json",
            "evidence/multispectrum/cs-ent-tenant-rbac-workflow-execution-1779664800.json",
            "evidence/multispectrum/cs-ent-tenant-rbac-workflow-runtime-evidence-1779703800.json",
            "evidence/multispectrum/cs-ent-platform-statutory-filing-evidence-1779665400.json",
            "evidence/multispectrum/cs-ent-platform-disbursement-evidence-1779666000.json",
        ],
        tenant_workload_manifest_attached: true,
        fd001_product_goal_preserved: tenant_workload_manifest.fd001_product_goal_preserved,
        oyatie_cloud_substrate_dogfood_plan_attached: tenant_workload_manifest
            .oyatie_cloud_substrate_only,
        tenant_namespace_contract_attached: tenant_workload_manifest.namespace_isolation_required,
        tenant_resource_quota_contract_attached: tenant_workload_manifest.resource_quota_required,
        tenant_resource_quota_policy_contract_attached: tenant_resource_quota_contract
            .namespace_resource_quota_required
            && tenant_resource_quota_contract.compute_requests_quota_required
            && tenant_resource_quota_contract.compute_limits_quota_required,
        tenant_limit_range_policy_contract_attached: tenant_resource_quota_contract
            .limit_range_defaults_required
            && tenant_resource_quota_contract.limit_range_min_max_required
            && tenant_resource_quota_contract.container_requests_limits_required,
        tenant_quota_compute_boundary_contract_attached: tenant_resource_quota_contract
            .compute_requests_quota_required
            && tenant_resource_quota_contract.compute_limits_quota_required
            && tenant_resource_quota_contract.container_requests_limits_required,
        tenant_quota_storage_object_boundary_contract_attached: tenant_resource_quota_contract
            .object_count_quota_required
            && tenant_resource_quota_contract.persistent_storage_quota_required,
        tenant_quota_admission_plugin_evidence_contract_attached: tenant_resource_quota_contract
            .resource_quota_admission_evidence_required
            && tenant_resource_quota_contract.limit_ranger_admission_evidence_required
            && tenant_resource_quota_contract.admission_policy_evidence_required,
        tenant_quota_usage_audit_contract_attached: tenant_resource_quota_contract
            .quota_usage_audit_evidence_required
            && tenant_resource_quota_contract.tenant_label_selector_required,
        tenant_quota_runtime_attached: false,
        tenant_availability_contract_attached: true,
        tenant_pod_disruption_budget_contract_attached: tenant_availability_contract
            .pod_disruption_budget_required
            && tenant_availability_contract.minimum_available_budget_required
            && tenant_availability_contract.multi_replica_workload_required,
        tenant_topology_spread_contract_attached: tenant_availability_contract
            .zone_topology_spread_required
            && tenant_availability_contract.hostname_topology_spread_required
            && tenant_availability_contract.node_topology_label_evidence_required,
        tenant_pod_anti_affinity_contract_attached: tenant_availability_contract
            .pod_anti_affinity_required,
        tenant_rolling_update_availability_contract_attached: tenant_availability_contract
            .rolling_update_availability_required
            && tenant_availability_contract.progress_deadline_required,
        tenant_readiness_probe_evidence_contract_attached: tenant_availability_contract
            .readiness_probe_evidence_required,
        tenant_disruption_audit_contract_attached: tenant_availability_contract
            .disruption_audit_evidence_required
            && tenant_availability_contract.tenant_label_selector_required,
        tenant_availability_runtime_attached: false,
        tenant_autoscaling_contract_attached: true,
        tenant_horizontal_pod_autoscaler_contract_attached: tenant_autoscaling_contract
            .horizontal_pod_autoscaler_required
            && tenant_autoscaling_contract.autoscaling_v2_api_required,
        tenant_autoscaling_metrics_pipeline_contract_attached: tenant_autoscaling_contract
            .cpu_resource_metric_required
            && tenant_autoscaling_contract.memory_resource_metric_required
            && tenant_autoscaling_contract.metrics_pipeline_evidence_required,
        tenant_autoscaling_replica_bounds_contract_attached: tenant_autoscaling_contract
            .min_replica_floor_required
            && tenant_autoscaling_contract.max_replica_ceiling_required,
        tenant_autoscaling_behavior_policy_contract_attached: tenant_autoscaling_contract
            .scale_up_behavior_policy_required
            && tenant_autoscaling_contract.scale_down_behavior_policy_required
            && tenant_autoscaling_contract.stabilization_window_required,
        tenant_autoscaling_audit_contract_attached: tenant_autoscaling_contract
            .scaling_audit_evidence_required
            && tenant_autoscaling_contract.tenant_label_selector_required
            && tenant_autoscaling_contract.admission_policy_evidence_required,
        tenant_autoscaling_runtime_attached: false,
        tenant_cost_allocation_contract_attached: true,
        tenant_cost_label_contract_attached: tenant_cost_allocation_contract
            .tenant_cost_allocation_labels_required
            && tenant_cost_allocation_contract.kubernetes_recommended_labels_required
            && tenant_cost_allocation_contract.namespace_cost_boundary_required
            && tenant_cost_allocation_contract.tenant_label_selector_required,
        tenant_cost_resource_basis_contract_attached: tenant_cost_allocation_contract
            .workload_resource_requests_required
            && tenant_cost_allocation_contract.resource_quota_usage_evidence_required,
        tenant_cost_otel_resource_contract_attached: tenant_cost_allocation_contract
            .opentelemetry_service_resource_required
            && tenant_cost_allocation_contract
                .opentelemetry_kubernetes_resource_attributes_required,
        tenant_cost_finops_allocation_contract_attached: tenant_cost_allocation_contract
            .finops_allocation_strategy_required
            && tenant_cost_allocation_contract.allocation_coverage_kpi_required,
        tenant_cost_shared_cost_contract_attached: tenant_cost_allocation_contract
            .shared_cost_policy_required,
        tenant_cost_audit_contract_attached: tenant_cost_allocation_contract
            .cost_allocation_audit_evidence_required
            && tenant_cost_allocation_contract.admission_policy_evidence_required,
        tenant_cost_allocation_runtime_attached: false,
        tenant_residency_contract_attached: true,
        tenant_residency_label_contract_attached: tenant_residency_contract
            .tenant_residency_region_label_required
            && tenant_residency_contract.namespace_residency_label_required,
        tenant_residency_scheduling_contract_attached: tenant_residency_contract
            .workload_node_affinity_required
            && tenant_residency_contract.topology_region_constraint_required
            && tenant_residency_contract.cell_placement_residency_ref_required,
        tenant_residency_storage_contract_attached: tenant_residency_contract
            .storage_residency_policy_ref_required,
        tenant_residency_telemetry_contract_attached: tenant_residency_contract
            .telemetry_residency_policy_ref_required,
        tenant_residency_audit_contract_attached: tenant_residency_contract
            .audit_residency_policy_ref_required
            && tenant_residency_contract.residency_audit_evidence_required
            && tenant_residency_contract.admission_policy_evidence_required
            && tenant_residency_contract.workload_manifest_evidence_required,
        tenant_residency_egress_contract_attached: tenant_residency_contract
            .cross_region_egress_policy_ref_required,
        tenant_residency_model_contract_attached: tenant_residency_contract
            .tenant_model_jurisdiction_ref_required,
        tenant_residency_runtime_attached: false,
        tenant_workload_identity_contract_attached: true,
        tenant_spiffe_id_contract_attached: tenant_workload_identity_contract.spiffe_id_required
            && tenant_workload_identity_contract.trust_domain_pinned,
        tenant_svid_contract_attached: tenant_workload_identity_contract.x509_svid_required
            && tenant_workload_identity_contract.jwt_svid_policy_required
            && tenant_workload_identity_contract.certificate_rotation_evidence_required,
        tenant_mtls_contract_attached: tenant_workload_identity_contract.mutual_tls_required
            && tenant_workload_identity_contract.authorization_policy_binding_required,
        tenant_gateway_backend_tls_contract_attached: tenant_workload_identity_contract
            .gateway_backend_tls_policy_required,
        tenant_trust_bundle_contract_attached: tenant_workload_identity_contract
            .trust_bundle_evidence_required
            && tenant_workload_identity_contract.workload_api_boundary_required
            && tenant_workload_identity_contract.workload_attestation_selector_required,
        tenant_identity_telemetry_contract_attached: tenant_workload_identity_contract
            .service_telemetry_identity_required,
        tenant_identity_audit_contract_attached: tenant_workload_identity_contract
            .identity_audit_evidence_required,
        tenant_workload_identity_runtime_attached: false,
        tenant_network_policy_contract_attached: tenant_workload_manifest.network_policy_required,
        tenant_gateway_route_contract_attached: tenant_workload_manifest.gateway_route_required,
        tenant_admission_policy_contract_attached: true,
        tenant_validating_admission_policy_contract_attached: tenant_admission_policy_contract
            .validating_admission_policy_required
            && tenant_admission_policy_contract.admission_binding_required,
        tenant_admission_deny_action_contract_attached: tenant_admission_policy_contract
            .failure_policy_fail_required
            && tenant_admission_policy_contract.deny_action_required,
        tenant_pod_security_restricted_contract_attached: tenant_admission_policy_contract
            .pod_security_restricted_required,
        tenant_digest_pinned_image_admission_contract_attached: tenant_admission_policy_contract
            .digest_pinned_image_required,
        tenant_latest_image_tag_forbidden_contract_attached: tenant_admission_policy_contract
            .latest_image_tag_forbidden,
        tenant_resource_requests_limits_admission_contract_attached:
            tenant_admission_policy_contract.resource_requests_limits_required,
        tenant_service_account_admission_contract_attached: tenant_admission_policy_contract
            .service_account_boundary_required,
        tenant_default_service_account_forbidden_contract_attached:
            tenant_admission_policy_contract.default_service_account_forbidden,
        tenant_admission_audit_annotation_contract_attached: tenant_admission_policy_contract
            .admission_audit_annotation_required,
        tenant_admission_runtime_attached: false,
        tenant_egress_policy_contract_attached: true,
        tenant_default_deny_egress_contract_attached: tenant_egress_policy_contract
            .default_deny_egress_required
            && tenant_egress_policy_contract.external_cidr_egress_forbidden_by_default,
        tenant_dns_egress_contract_attached: tenant_egress_policy_contract.dns_egress_only_required,
        tenant_cross_namespace_egress_contract_attached: tenant_egress_policy_contract
            .cross_namespace_egress_explicit_selector_required
            && tenant_egress_policy_contract.same_namespace_service_egress_required,
        tenant_external_egress_exception_contract_attached: tenant_egress_policy_contract
            .ip_block_exception_evidence_required
            && tenant_egress_policy_contract.protocol_port_pinned_required,
        tenant_egress_audit_contract_attached: tenant_egress_policy_contract
            .egress_audit_evidence_required
            && tenant_egress_policy_contract.network_policy_provider_evidence_required,
        tenant_egress_runtime_attached: false,
        tenant_image_provenance_contract_attached: true,
        tenant_image_provenance_slsa_contract_attached: tenant_image_provenance_contract
            .slsa_provenance_required
            && tenant_image_provenance_contract.builder_id_pin_required
            && tenant_image_provenance_contract.source_revision_pin_required,
        tenant_image_provenance_sbom_contract_attached: tenant_image_provenance_contract
            .sbom_required
            && tenant_image_provenance_contract.vulnerability_scan_gate_required,
        tenant_image_provenance_cosign_contract_attached: tenant_image_provenance_contract
            .cosign_signature_required
            && tenant_image_provenance_contract.keyless_oidc_identity_required
            && tenant_image_provenance_contract.transparency_log_required,
        tenant_image_provenance_admission_contract_attached: tenant_image_provenance_contract
            .oci_digest_pinned_required
            && tenant_image_provenance_contract.admission_policy_evidence_required,
        tenant_image_provenance_runtime_attached: false,
        tenant_secret_boundary_contract_attached: true,
        tenant_secret_ref_boundary_contract_attached: tenant_secret_boundary_contract
            .inline_secret_material_forbidden
            && tenant_secret_boundary_contract.kubernetes_secret_reference_required
            && tenant_secret_boundary_contract.external_secret_store_boundary_required,
        tenant_secret_encryption_contract_attached: tenant_secret_boundary_contract
            .secret_at_rest_encryption_required,
        tenant_secret_rbac_contract_attached: tenant_secret_boundary_contract
            .rbac_least_privilege_required
            && tenant_secret_boundary_contract.namespace_secret_isolation_required
            && tenant_secret_boundary_contract.workload_scoped_service_account_required
            && tenant_secret_boundary_contract.automount_service_account_token_forbidden,
        tenant_secret_rotation_audit_contract_attached: tenant_secret_boundary_contract
            .secret_rotation_evidence_required
            && tenant_secret_boundary_contract.secret_access_audit_evidence_required,
        tenant_secret_runtime_attached: false,
        tenant_workload_runtime_evidence_plan_attached: true,
        tenant_runtime_namespace_evidence_contract_attached: tenant_workload_runtime_evidence_plan
            .namespace_observation_required,
        tenant_runtime_quota_evidence_contract_attached: tenant_workload_runtime_evidence_plan
            .resource_quota_usage_required
            && tenant_workload_runtime_evidence_plan.resource_requests_limits_required,
        tenant_runtime_network_policy_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.network_policy_default_deny_required,
        tenant_runtime_service_account_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.service_account_boundary_required,
        tenant_runtime_pod_security_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.pod_security_context_required,
        tenant_runtime_workload_schedule_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.workload_scheduled_required,
        tenant_runtime_probe_evidence_contract_attached: tenant_workload_runtime_evidence_plan
            .readiness_probe_required
            && tenant_workload_runtime_evidence_plan.liveness_probe_required,
        tenant_runtime_gateway_route_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.gateway_route_acceptance_required,
        tenant_runtime_claim_evidence_contract_attached: tenant_workload_runtime_evidence_plan
            .tenant_claim_propagation_required,
        tenant_runtime_otel_resource_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.otel_resource_identity_required,
        tenant_runtime_rollout_recovery_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.rollout_recovery_required,
        tenant_runtime_audit_event_evidence_contract_attached:
            tenant_workload_runtime_evidence_plan.workload_audit_event_required,
        tenant_workload_runtime_evidence_attached: false,
        tenant_workload_runtime_attached: false,
        tenant_cloud_substrate_runtime_attached: false,
        deployed_listener_attached: false,
        listener_gateway_plan_attached: true,
        listener_runtime_evidence_plan_attached: true,
        cluster_ip_service_evidence_contract_attached: listener_runtime_evidence_plan
            .cluster_ip_service_observation_required,
        gateway_route_runtime_acceptance_contract_attached: listener_runtime_evidence_plan
            .gateway_route_acceptance_required,
        tls_certificate_binding_evidence_contract_attached: listener_runtime_evidence_plan
            .tls_certificate_binding_required,
        listener_probe_evidence_contract_attached: listener_runtime_evidence_plan
            .readiness_probe_success_required
            && listener_runtime_evidence_plan.liveness_probe_success_required
            && listener_runtime_evidence_plan.synthetic_health_check_required,
        route_authz_evidence_contract_attached: listener_runtime_evidence_plan
            .route_authz_enforcement_required,
        network_policy_evidence_contract_attached: listener_runtime_evidence_plan
            .default_deny_network_policy_required,
        endpoint_slice_evidence_contract_attached: listener_runtime_evidence_plan
            .endpoint_slice_ready_required,
        listener_audit_event_evidence_contract_attached: listener_runtime_evidence_plan
            .listener_deployment_audit_event_required,
        listener_runtime_attached: false,
        authentication_runtime_attached: true,
        identity_provider_verification_plan_attached: true,
        identity_provider_runtime_evidence_plan_attached: true,
        oidc_discovery_plan_attached: true,
        jwks_validation_plan_attached: true,
        oidc_discovery_runtime_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .discovery_document_observation_required
            && identity_provider_runtime_evidence_plan.issuer_metadata_match_required,
        jwks_runtime_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .jwks_fetch_evidence_required
            && identity_provider_runtime_evidence_plan.jwks_kid_match_required,
        jwt_signature_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .jwt_signature_verification_evidence_required
            && identity_provider_runtime_evidence_plan.algorithm_allowlist_required,
        jwt_claims_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .issuer_claim_match_required
            && identity_provider_runtime_evidence_plan.audience_claim_match_required
            && identity_provider_runtime_evidence_plan.temporal_claims_check_required,
        nonce_replay_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .nonce_replay_denial_required,
        tenant_scope_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .tenant_claim_mapping_required
            && identity_provider_runtime_evidence_plan.route_scope_authorization_required,
        sensitive_route_mfa_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .sensitive_route_mfa_enforcement_required,
        key_rotation_evidence_contract_attached: identity_provider_runtime_evidence_plan
            .key_rotation_overlap_evidence_required,
        auth_failure_audit_event_evidence_contract_attached:
            identity_provider_runtime_evidence_plan.auth_failure_audit_event_required,
        oidc_signature_verification_attached: false,
        jwks_provider_attached: false,
        identity_provider_verification_attached: false,
        identity_provider_runtime_evidence_attached: false,
        durable_business_storage_attached: false,
        postgres_rls_storage_plan_attached: true,
        postgres_rls_write_contract_attached: true,
        postgres_set_local_tenant_context_contract_attached: postgres_write_contract
            .set_local_tenant_context_required,
        postgres_parameterized_insert_contract_attached: postgres_write_contract
            .parameterized_insert_required,
        postgres_idempotency_conflict_contract_attached: postgres_write_contract
            .idempotency_conflict_do_nothing_required,
        postgres_tenant_scoped_readback_contract_attached: postgres_write_contract
            .tenant_scoped_readback_required,
        postgres_delete_statement_forbidden_contract_attached: postgres_write_contract
            .delete_statement_forbidden,
        postgres_write_runtime_attached: false,
        postgres_rls_transaction_contract_attached: true,
        postgres_explicit_transaction_contract_attached: postgres_transaction_contract
            .explicit_transaction_required,
        postgres_transaction_local_tenant_context_contract_attached: postgres_transaction_contract
            .transaction_local_tenant_context_required,
        postgres_prepared_statement_contract_attached: postgres_transaction_contract
            .prepared_statement_required,
        postgres_bound_parameter_execution_contract_attached: postgres_transaction_contract
            .bound_parameter_execution_required,
        postgres_commit_rollback_contract_attached: postgres_transaction_contract
            .commit_after_readback_required
            && postgres_transaction_contract.rollback_on_error_required,
        postgres_transaction_runtime_attached: false,
        postgres_prepared_statement_runtime_attached: false,
        postgres_rls_runtime_evidence_plan_attached: true,
        postgres_migration_rehearsal_contract_attached: true,
        postgres_tls_verify_full_contract_attached: true,
        postgres_rls_probe_matrix_attached: true,
        postgres_backup_restore_rehearsal_contract_attached: true,
        postgres_pitr_rehearsal_contract_attached: true,
        postgres_database_attached: false,
        postgres_rls_runtime_verified_attached: false,
        workflow_engine_execution_attached: false,
        workflow_execution_reference_attached: true,
        workflow_broker_publish_attached: false,
        workflow_durable_queue_attached: false,
        workflow_runtime_evidence_plan_attached: true,
        workflow_definition_evidence_contract_attached: workflow_runtime_evidence_plan
            .workflow_definition_version_pin_required,
        workflow_gate_evidence_contract_attached: workflow_runtime_evidence_plan
            .deterministic_gate_evidence_required
            && workflow_runtime_evidence_plan.dispatch_idempotency_required
            && workflow_runtime_evidence_plan.execution_state_transition_required,
        workflow_durable_queue_evidence_contract_attached: workflow_runtime_evidence_plan
            .durable_queue_ack_required,
        workflow_broker_publish_evidence_contract_attached: workflow_runtime_evidence_plan
            .broker_publish_confirmation_required,
        workflow_broker_retry_dlq_evidence_contract_attached: workflow_runtime_evidence_plan
            .broker_retry_or_dlq_required,
        workflow_tenant_partition_evidence_contract_attached: workflow_runtime_evidence_plan
            .tenant_partition_required
            && workflow_runtime_evidence_plan.payload_digest_required,
        workflow_otel_trace_evidence_contract_attached: workflow_runtime_evidence_plan
            .otel_messaging_trace_required,
        workflow_audit_event_evidence_contract_attached: workflow_runtime_evidence_plan
            .workflow_audit_event_required,
        workflow_replay_recovery_evidence_contract_attached: workflow_runtime_evidence_plan
            .replay_recovery_required,
        workflow_runtime_evidence_attached: false,
        statutory_filing_evidence_plan_attached: true,
        statutory_authority_registry_attached: true,
        statutory_payload_digest_contract_attached: true,
        statutory_agency_receipt_contract_attached: true,
        statutory_runtime_submission_attached: false,
        statutory_disbursement_rail_attached: false,
        disbursement_evidence_plan_attached: true,
        disbursement_network_registry_attached: true,
        disbursement_payment_digest_contract_attached: true,
        disbursement_reconciliation_contract_attached: true,
        disbursement_runtime_execution_attached: false,
        disbursement_bank_connection_attached: false,
        runtime_audit_chain_emission_attached: false,
        audit_chain_emission_plan_attached: true,
        audit_chain_event_contract_attached: true,
        audit_chain_wal_plan_attached: true,
        audit_chain_outbox_plan_attached: true,
        audit_chain_runtime_evidence_plan_attached: true,
        audit_chain_event_envelope_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .cloudevents_envelope_evidence_required,
        audit_chain_trace_context_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .trace_context_evidence_required,
        audit_chain_otel_log_mapping_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .otel_log_record_mapping_required,
        audit_chain_tenant_partition_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .tenant_partition_evidence_required
            && audit_chain_runtime_evidence_plan.idempotency_dedupe_evidence_required,
        audit_chain_payload_digest_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .payload_digest_match_required
            && audit_chain_runtime_evidence_plan.sensitive_payload_redaction_required,
        audit_chain_wal_append_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .wal_append_evidence_required,
        audit_chain_outbox_publish_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .outbox_publish_evidence_required,
        audit_chain_broker_ack_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .broker_ack_evidence_required,
        audit_chain_merkle_seal_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .merkle_leaf_inclusion_required
            && audit_chain_runtime_evidence_plan.merkle_root_seal_required,
        audit_chain_sink_ingestion_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .sink_ingestion_required,
        audit_chain_replay_recovery_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .replay_recovery_required,
        audit_chain_failure_path_evidence_contract_attached: audit_chain_runtime_evidence_plan
            .failure_path_audit_required,
        audit_chain_runtime_evidence_attached: false,
        cloud_deployment_manifest_attached: true,
        cloud_deployment_evidence_plan_attached: true,
        argocd_sync_evidence_contract_attached: cloud_deployment_evidence_plan
            .argocd_sync_evidence_required,
        argocd_health_evidence_contract_attached: cloud_deployment_evidence_plan
            .argocd_health_evidence_required,
        cosign_verification_evidence_contract_attached: cloud_deployment_evidence_plan
            .cosign_verification_required,
        kubernetes_rollout_evidence_contract_attached: cloud_deployment_evidence_plan
            .deployment_available_required,
        gateway_route_acceptance_evidence_contract_attached: cloud_deployment_evidence_plan
            .gateway_route_acceptance_required,
        otel_resource_evidence_contract_attached: cloud_deployment_evidence_plan
            .otel_resource_identity_required,
        deployment_audit_event_evidence_contract_attached: cloud_deployment_evidence_plan
            .deployment_audit_event_required,
        argocd_controller_attached: false,
        gateway_controller_attached: false,
        load_balancer_attached: false,
        tls_certificate_attached: false,
        cloud_deployment_evidence_attached: false,
        slo_evidence_plan_attached: true,
        slo_error_budget_release_gate_attached: true,
        slo_burn_rate_alert_plan_attached: true,
        slo_openslo_manifests_attached: true,
        slo_otel_metric_streams_attached: true,
        multi_region_slo_evidence_attached: false,
        schema_version: 1,
    })
}

pub fn required_cloud_blockers() -> Vec<CloudReadinessBlocker> {
    vec![
        CloudReadinessBlocker::DeployedListenerRuntimeEvidenceMissing,
        CloudReadinessBlocker::IdentityProviderVerificationMissing,
        CloudReadinessBlocker::DurableStorageRuntimeMissing,
        CloudReadinessBlocker::PostgresRlsRuntimeEvidenceMissing,
        CloudReadinessBlocker::WorkflowEngineMissing,
        CloudReadinessBlocker::BrokerPublishMissing,
        CloudReadinessBlocker::StatutoryFilingRailMissing,
        CloudReadinessBlocker::DisbursementRailMissing,
        CloudReadinessBlocker::RuntimeAuditEmissionMissing,
        CloudReadinessBlocker::CloudDeploymentEvidenceMissing,
        CloudReadinessBlocker::SloEvidenceMissing,
    ]
}

pub fn validate_cloud_claim(
    report: &TenantRbacCloudReadinessReport,
) -> Result<(), CloudReadinessGateError> {
    if !report.route_catalog_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "route_catalog_ready",
        ));
    }
    if !report.in_memory_harness_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "in_memory_harness_ready",
        ));
    }
    if !report.erp_parity_map_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "erp_parity_map_ready",
        ));
    }
    if !report.cloud_deployment_manifest_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "cloud_deployment_manifest_ready",
        ));
    }
    if !report.cloud_deployment_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "cloud_deployment_evidence_plan_ready",
        ));
    }
    if !report.tenant_workload_manifest_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_workload_manifest_ready",
        ));
    }
    if !report.tenant_admission_policy_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_admission_policy_contract_ready",
        ));
    }
    if !report.tenant_resource_quota_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_resource_quota_contract_ready",
        ));
    }
    if !report.tenant_availability_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_availability_contract_ready",
        ));
    }
    if !report.tenant_autoscaling_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_autoscaling_contract_ready",
        ));
    }
    if !report.tenant_cost_allocation_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_cost_allocation_contract_ready",
        ));
    }
    if !report.tenant_residency_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_residency_contract_ready",
        ));
    }
    if !report.tenant_workload_identity_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_workload_identity_contract_ready",
        ));
    }
    if !report.tenant_egress_policy_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_egress_policy_contract_ready",
        ));
    }
    if !report.tenant_image_provenance_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_image_provenance_contract_ready",
        ));
    }
    if !report.tenant_secret_boundary_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_secret_boundary_contract_ready",
        ));
    }
    if !report.tenant_workload_runtime_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "tenant_workload_runtime_evidence_plan_ready",
        ));
    }
    if !report.authentication_runtime_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "authentication_runtime_ready",
        ));
    }
    if !report.identity_provider_verification_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "identity_provider_verification_plan_ready",
        ));
    }
    if !report.identity_provider_runtime_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "identity_provider_runtime_evidence_plan_ready",
        ));
    }
    if !report.postgres_rls_storage_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "postgres_rls_storage_plan_ready",
        ));
    }
    if !report.postgres_rls_write_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "postgres_rls_write_contract_ready",
        ));
    }
    if !report.postgres_rls_transaction_contract_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "postgres_rls_transaction_contract_ready",
        ));
    }
    if !report.postgres_rls_runtime_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "postgres_rls_runtime_evidence_plan_ready",
        ));
    }
    if !report.listener_gateway_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "listener_gateway_plan_ready",
        ));
    }
    if !report.listener_runtime_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "listener_runtime_evidence_plan_ready",
        ));
    }
    if !report.audit_chain_emission_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "audit_chain_emission_plan_ready",
        ));
    }
    if !report.audit_chain_runtime_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "audit_chain_runtime_evidence_plan_ready",
        ));
    }
    if !report.workflow_execution_reference_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "workflow_execution_reference_ready",
        ));
    }
    if !report.workflow_runtime_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "workflow_runtime_evidence_plan_ready",
        ));
    }
    if !report.statutory_filing_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "statutory_filing_evidence_plan_ready",
        ));
    }
    if !report.disbursement_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "disbursement_evidence_plan_ready",
        ));
    }
    if !report.slo_evidence_plan_ready {
        return Err(CloudReadinessGateError::CloudClaimMissingLocalGate(
            "slo_evidence_plan_ready",
        ));
    }
    if report.cloud_deployment_ready && !report.blockers.is_empty() {
        return Err(CloudReadinessGateError::CloudClaimBlocked(
            report.blockers.clone(),
        ));
    }
    if report.cloud_deployment_ready {
        Ok(())
    } else {
        Err(CloudReadinessGateError::CloudClaimBlocked(
            report.blockers.clone(),
        ))
    }
}
