use iam_tenant_rbac_readiness_gate::{
    CloudReadinessBlocker, CloudReadinessGateError, required_cloud_blockers,
    tenant_rbac_cloud_readiness_report, validate_cloud_claim,
};

#[test]
fn cloud_readiness_gate_reports_local_rehearsal_ready_but_cloud_blocked() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert_eq!(report.route_count, 19);
    assert_eq!(report.sap_module_count, 23);
    assert!(report.route_catalog_ready);
    assert!(report.in_memory_harness_ready);
    assert!(report.erp_parity_map_ready);
    assert!(report.cloud_deployment_manifest_ready);
    assert!(report.cloud_deployment_evidence_plan_ready);
    assert_eq!(report.cloud_deployment_evidence_requirement_count, 14);
    assert!(report.tenant_workload_manifest_ready);
    assert_eq!(report.tenant_workload_count, 4);
    assert!(report.tenant_admission_policy_contract_ready);
    assert_eq!(report.tenant_admission_policy_rule_count, 11);
    assert!(report.tenant_admission_policy_all_workloads_in_scope);
    assert!(report.tenant_resource_quota_contract_ready);
    assert_eq!(report.tenant_resource_quota_requirement_count, 13);
    assert!(report.tenant_resource_quota_all_workloads_in_scope);
    assert!(report.tenant_availability_contract_ready);
    assert_eq!(report.tenant_availability_requirement_count, 13);
    assert!(report.tenant_availability_all_workloads_in_scope);
    assert!(report.tenant_autoscaling_contract_ready);
    assert_eq!(report.tenant_autoscaling_requirement_count, 13);
    assert!(report.tenant_autoscaling_all_workloads_in_scope);
    assert!(report.tenant_cost_allocation_contract_ready);
    assert_eq!(report.tenant_cost_allocation_requirement_count, 13);
    assert!(report.tenant_cost_allocation_all_workloads_in_scope);
    assert!(report.tenant_residency_contract_ready);
    assert_eq!(report.tenant_residency_requirement_count, 13);
    assert!(report.tenant_residency_all_workloads_in_scope);
    assert!(report.tenant_workload_identity_contract_ready);
    assert_eq!(report.tenant_workload_identity_requirement_count, 13);
    assert!(report.tenant_workload_identity_all_workloads_in_scope);
    assert!(report.tenant_egress_policy_contract_ready);
    assert_eq!(report.tenant_egress_policy_rule_count, 11);
    assert!(report.tenant_egress_policy_all_workloads_in_scope);
    assert!(report.tenant_image_provenance_contract_ready);
    assert_eq!(report.tenant_image_provenance_requirement_count, 11);
    assert!(report.tenant_image_provenance_all_workloads_in_scope);
    assert!(report.tenant_secret_boundary_contract_ready);
    assert_eq!(report.tenant_secret_boundary_requirement_count, 11);
    assert!(report.tenant_secret_boundary_all_workloads_in_scope);
    assert!(report.tenant_workload_runtime_evidence_plan_ready);
    assert_eq!(
        report.tenant_workload_runtime_evidence_requirement_count,
        14
    );
    assert!(report.authentication_runtime_ready);
    assert!(report.identity_provider_verification_plan_ready);
    assert!(report.identity_provider_runtime_evidence_plan_ready);
    assert_eq!(
        report.identity_provider_runtime_evidence_requirement_count,
        15
    );
    assert!(report.postgres_rls_storage_plan_ready);
    assert!(report.postgres_rls_write_contract_ready);
    assert_eq!(report.postgres_rls_write_statement_count, 5);
    assert!(report.postgres_rls_transaction_contract_ready);
    assert_eq!(report.postgres_rls_transaction_plan_count, 5);
    assert!(report.postgres_rls_runtime_evidence_plan_ready);
    assert!(report.listener_gateway_plan_ready);
    assert!(report.listener_runtime_evidence_plan_ready);
    assert_eq!(report.listener_runtime_evidence_requirement_count, 12);
    assert!(report.audit_chain_emission_plan_ready);
    assert!(report.audit_chain_runtime_evidence_plan_ready);
    assert_eq!(report.audit_chain_runtime_evidence_requirement_count, 15);
    assert!(report.workflow_execution_reference_ready);
    assert!(report.workflow_runtime_evidence_plan_ready);
    assert_eq!(report.workflow_runtime_evidence_requirement_count, 14);
    assert!(report.statutory_filing_evidence_plan_ready);
    assert!(report.disbursement_evidence_plan_ready);
    assert!(report.slo_evidence_plan_ready);
    assert!(report.authentication_runtime_attached);
    assert!(report.identity_provider_verification_plan_attached);
    assert!(report.identity_provider_runtime_evidence_plan_attached);
    assert!(report.oidc_discovery_plan_attached);
    assert!(report.jwks_validation_plan_attached);
    assert!(report.oidc_discovery_runtime_evidence_contract_attached);
    assert!(report.jwks_runtime_evidence_contract_attached);
    assert!(report.jwt_signature_evidence_contract_attached);
    assert!(report.jwt_claims_evidence_contract_attached);
    assert!(report.nonce_replay_evidence_contract_attached);
    assert!(report.tenant_scope_evidence_contract_attached);
    assert!(report.sensitive_route_mfa_evidence_contract_attached);
    assert!(report.key_rotation_evidence_contract_attached);
    assert!(report.auth_failure_audit_event_evidence_contract_attached);
    assert!(!report.oidc_signature_verification_attached);
    assert!(!report.jwks_provider_attached);
    assert!(report.listener_gateway_plan_attached);
    assert!(report.listener_runtime_evidence_plan_attached);
    assert!(report.cluster_ip_service_evidence_contract_attached);
    assert!(report.gateway_route_runtime_acceptance_contract_attached);
    assert!(report.tls_certificate_binding_evidence_contract_attached);
    assert!(report.listener_probe_evidence_contract_attached);
    assert!(report.route_authz_evidence_contract_attached);
    assert!(report.network_policy_evidence_contract_attached);
    assert!(report.endpoint_slice_evidence_contract_attached);
    assert!(report.listener_audit_event_evidence_contract_attached);
    assert!(!report.listener_runtime_attached);
    assert!(!report.identity_provider_verification_attached);
    assert!(!report.identity_provider_runtime_evidence_attached);
    assert!(report.cloud_deployment_manifest_attached);
    assert!(report.tenant_workload_manifest_attached);
    assert!(report.fd001_product_goal_preserved);
    assert!(report.oyatie_cloud_substrate_dogfood_plan_attached);
    assert!(report.tenant_namespace_contract_attached);
    assert!(report.tenant_resource_quota_contract_attached);
    assert!(report.tenant_resource_quota_policy_contract_attached);
    assert!(report.tenant_limit_range_policy_contract_attached);
    assert!(report.tenant_quota_compute_boundary_contract_attached);
    assert!(report.tenant_quota_storage_object_boundary_contract_attached);
    assert!(report.tenant_quota_admission_plugin_evidence_contract_attached);
    assert!(report.tenant_quota_usage_audit_contract_attached);
    assert!(!report.tenant_quota_runtime_attached);
    assert!(report.tenant_availability_contract_attached);
    assert!(report.tenant_pod_disruption_budget_contract_attached);
    assert!(report.tenant_topology_spread_contract_attached);
    assert!(report.tenant_pod_anti_affinity_contract_attached);
    assert!(report.tenant_rolling_update_availability_contract_attached);
    assert!(report.tenant_readiness_probe_evidence_contract_attached);
    assert!(report.tenant_disruption_audit_contract_attached);
    assert!(!report.tenant_availability_runtime_attached);
    assert!(report.tenant_autoscaling_contract_attached);
    assert!(report.tenant_horizontal_pod_autoscaler_contract_attached);
    assert!(report.tenant_autoscaling_metrics_pipeline_contract_attached);
    assert!(report.tenant_autoscaling_replica_bounds_contract_attached);
    assert!(report.tenant_autoscaling_behavior_policy_contract_attached);
    assert!(report.tenant_autoscaling_audit_contract_attached);
    assert!(!report.tenant_autoscaling_runtime_attached);
    assert!(report.tenant_cost_allocation_contract_attached);
    assert!(report.tenant_cost_label_contract_attached);
    assert!(report.tenant_cost_resource_basis_contract_attached);
    assert!(report.tenant_cost_otel_resource_contract_attached);
    assert!(report.tenant_cost_finops_allocation_contract_attached);
    assert!(report.tenant_cost_shared_cost_contract_attached);
    assert!(report.tenant_cost_audit_contract_attached);
    assert!(!report.tenant_cost_allocation_runtime_attached);
    assert!(report.tenant_residency_contract_attached);
    assert!(report.tenant_residency_label_contract_attached);
    assert!(report.tenant_residency_scheduling_contract_attached);
    assert!(report.tenant_residency_storage_contract_attached);
    assert!(report.tenant_residency_telemetry_contract_attached);
    assert!(report.tenant_residency_audit_contract_attached);
    assert!(report.tenant_residency_egress_contract_attached);
    assert!(report.tenant_residency_model_contract_attached);
    assert!(!report.tenant_residency_runtime_attached);
    assert!(report.tenant_workload_identity_contract_attached);
    assert!(report.tenant_spiffe_id_contract_attached);
    assert!(report.tenant_svid_contract_attached);
    assert!(report.tenant_mtls_contract_attached);
    assert!(report.tenant_gateway_backend_tls_contract_attached);
    assert!(report.tenant_trust_bundle_contract_attached);
    assert!(report.tenant_identity_telemetry_contract_attached);
    assert!(report.tenant_identity_audit_contract_attached);
    assert!(!report.tenant_workload_identity_runtime_attached);
    assert!(report.tenant_network_policy_contract_attached);
    assert!(report.tenant_gateway_route_contract_attached);
    assert!(report.tenant_admission_policy_contract_attached);
    assert!(report.tenant_validating_admission_policy_contract_attached);
    assert!(report.tenant_admission_deny_action_contract_attached);
    assert!(report.tenant_pod_security_restricted_contract_attached);
    assert!(report.tenant_digest_pinned_image_admission_contract_attached);
    assert!(report.tenant_latest_image_tag_forbidden_contract_attached);
    assert!(report.tenant_resource_requests_limits_admission_contract_attached);
    assert!(report.tenant_service_account_admission_contract_attached);
    assert!(report.tenant_default_service_account_forbidden_contract_attached);
    assert!(report.tenant_admission_audit_annotation_contract_attached);
    assert!(!report.tenant_admission_runtime_attached);
    assert!(report.tenant_egress_policy_contract_attached);
    assert!(report.tenant_default_deny_egress_contract_attached);
    assert!(report.tenant_dns_egress_contract_attached);
    assert!(report.tenant_cross_namespace_egress_contract_attached);
    assert!(report.tenant_external_egress_exception_contract_attached);
    assert!(report.tenant_egress_audit_contract_attached);
    assert!(!report.tenant_egress_runtime_attached);
    assert!(report.tenant_image_provenance_contract_attached);
    assert!(report.tenant_image_provenance_slsa_contract_attached);
    assert!(report.tenant_image_provenance_sbom_contract_attached);
    assert!(report.tenant_image_provenance_cosign_contract_attached);
    assert!(report.tenant_image_provenance_admission_contract_attached);
    assert!(!report.tenant_image_provenance_runtime_attached);
    assert!(report.tenant_secret_boundary_contract_attached);
    assert!(report.tenant_secret_ref_boundary_contract_attached);
    assert!(report.tenant_secret_encryption_contract_attached);
    assert!(report.tenant_secret_rbac_contract_attached);
    assert!(report.tenant_secret_rotation_audit_contract_attached);
    assert!(!report.tenant_secret_runtime_attached);
    assert!(report.tenant_workload_runtime_evidence_plan_attached);
    assert!(report.tenant_runtime_namespace_evidence_contract_attached);
    assert!(report.tenant_runtime_quota_evidence_contract_attached);
    assert!(report.tenant_runtime_network_policy_evidence_contract_attached);
    assert!(report.tenant_runtime_service_account_evidence_contract_attached);
    assert!(report.tenant_runtime_pod_security_evidence_contract_attached);
    assert!(report.tenant_runtime_workload_schedule_evidence_contract_attached);
    assert!(report.tenant_runtime_probe_evidence_contract_attached);
    assert!(report.tenant_runtime_gateway_route_evidence_contract_attached);
    assert!(report.tenant_runtime_claim_evidence_contract_attached);
    assert!(report.tenant_runtime_otel_resource_evidence_contract_attached);
    assert!(report.tenant_runtime_rollout_recovery_evidence_contract_attached);
    assert!(report.tenant_runtime_audit_event_evidence_contract_attached);
    assert!(!report.tenant_workload_runtime_evidence_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.argocd_controller_attached);
    assert!(!report.gateway_controller_attached);
    assert!(!report.load_balancer_attached);
    assert!(!report.tls_certificate_attached);
    assert!(report.cloud_deployment_evidence_plan_attached);
    assert!(report.argocd_sync_evidence_contract_attached);
    assert!(report.argocd_health_evidence_contract_attached);
    assert!(report.cosign_verification_evidence_contract_attached);
    assert!(report.kubernetes_rollout_evidence_contract_attached);
    assert!(report.gateway_route_acceptance_evidence_contract_attached);
    assert!(report.otel_resource_evidence_contract_attached);
    assert!(report.deployment_audit_event_evidence_contract_attached);
    assert!(!report.cloud_deployment_evidence_attached);
    assert!(report.audit_chain_emission_plan_attached);
    assert!(report.audit_chain_event_contract_attached);
    assert!(report.audit_chain_wal_plan_attached);
    assert!(report.audit_chain_outbox_plan_attached);
    assert!(report.audit_chain_runtime_evidence_plan_attached);
    assert!(report.audit_chain_event_envelope_evidence_contract_attached);
    assert!(report.audit_chain_trace_context_evidence_contract_attached);
    assert!(report.audit_chain_otel_log_mapping_evidence_contract_attached);
    assert!(report.audit_chain_tenant_partition_evidence_contract_attached);
    assert!(report.audit_chain_payload_digest_evidence_contract_attached);
    assert!(report.audit_chain_wal_append_evidence_contract_attached);
    assert!(report.audit_chain_outbox_publish_evidence_contract_attached);
    assert!(report.audit_chain_broker_ack_evidence_contract_attached);
    assert!(report.audit_chain_merkle_seal_evidence_contract_attached);
    assert!(report.audit_chain_sink_ingestion_evidence_contract_attached);
    assert!(report.audit_chain_replay_recovery_evidence_contract_attached);
    assert!(report.audit_chain_failure_path_evidence_contract_attached);
    assert!(!report.audit_chain_runtime_evidence_attached);
    assert!(report.workflow_execution_reference_attached);
    assert!(!report.workflow_engine_execution_attached);
    assert!(!report.workflow_broker_publish_attached);
    assert!(!report.workflow_durable_queue_attached);
    assert!(report.workflow_runtime_evidence_plan_attached);
    assert!(report.workflow_definition_evidence_contract_attached);
    assert!(report.workflow_gate_evidence_contract_attached);
    assert!(report.workflow_durable_queue_evidence_contract_attached);
    assert!(report.workflow_broker_publish_evidence_contract_attached);
    assert!(report.workflow_broker_retry_dlq_evidence_contract_attached);
    assert!(report.workflow_tenant_partition_evidence_contract_attached);
    assert!(report.workflow_otel_trace_evidence_contract_attached);
    assert!(report.workflow_audit_event_evidence_contract_attached);
    assert!(report.workflow_replay_recovery_evidence_contract_attached);
    assert!(!report.workflow_runtime_evidence_attached);
    assert!(report.statutory_filing_evidence_plan_attached);
    assert!(report.statutory_authority_registry_attached);
    assert!(report.statutory_payload_digest_contract_attached);
    assert!(report.statutory_agency_receipt_contract_attached);
    assert!(!report.statutory_runtime_submission_attached);
    assert!(!report.statutory_disbursement_rail_attached);
    assert!(report.disbursement_evidence_plan_attached);
    assert!(report.disbursement_network_registry_attached);
    assert!(report.disbursement_payment_digest_contract_attached);
    assert!(report.disbursement_reconciliation_contract_attached);
    assert!(!report.disbursement_runtime_execution_attached);
    assert!(!report.disbursement_bank_connection_attached);
    assert!(report.slo_evidence_plan_attached);
    assert!(report.slo_error_budget_release_gate_attached);
    assert!(report.slo_burn_rate_alert_plan_attached);
    assert!(report.slo_openslo_manifests_attached);
    assert!(report.slo_otel_metric_streams_attached);
    assert!(!report.runtime_audit_chain_emission_attached);
    assert!(report.local_rehearsal_ready);
    assert!(!report.cloud_deployment_ready);
    assert_eq!(report.blocker_count, 11);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DeployedListenerRuntimeEvidenceMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::IdentityProviderVerificationMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DurableStorageRuntimeMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::PostgresRlsRuntimeEvidenceMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-local-inmemory-harness-1779541800.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-cloud-deployment-manifest-1779551400.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-cloud-deployment-evidence-1779702000.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-workload-manifest-1779701400.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-admission-policy-1779706800.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-resource-quota-contract-1779709200.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-availability-contract-1779709800.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-autoscaling-contract-1779710400.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-cost-allocation-contract-1779711000.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-residency-contract-1779711600.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-egress-policy-contract-1779708600.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-image-provenance-contract-1779707400.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-secret-boundary-contract-1779708000.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-runtime-evidence-1779705000.json"
    ));
    assert!(
        report
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-platform-idp-verification-1779553800.json")
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-idp-runtime-evidence-1779703200.json"
        )
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-postgres-rls-runtime-evidence-1779666600.json"
    ));
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-postgres-tx-contract-1779706200.json"
        )
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-postgres-write-contract-1779705600.json"
    ));
    assert!(
        report
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-platform-listener-gateway-1779553200.json")
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-listener-runtime-evidence-1779702600.json"
    ));
    assert!(
        report
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-platform-slo-evidence-1779664200.json")
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-tenant-rbac-workflow-execution-1779664800.json"
        )
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-tenant-rbac-workflow-runtime-evidence-1779703800.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-audit-runtime-evidence-1779704400.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-statutory-filing-evidence-1779665400.json"
    ));
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-disbursement-evidence-1779666000.json"
        )
    );
}

#[test]
fn cloud_readiness_gate_includes_manifest_foundation_without_cloud_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.cloud_deployment_manifest_ready);
    assert!(report.cloud_deployment_manifest_attached);
    assert!(!report.cloud_deployment_evidence_attached);
    assert!(!report.cloud_deployment_ready);
    assert_eq!(report.blocker_count, 11);
}

#[test]
fn cloud_readiness_gate_includes_cloud_deployment_evidence_plan_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.cloud_deployment_evidence_plan_ready);
    assert_eq!(report.cloud_deployment_evidence_requirement_count, 14);
    assert!(report.cloud_deployment_evidence_plan_attached);
    assert!(report.fd001_product_goal_preserved);
    assert!(report.oyatie_cloud_substrate_dogfood_plan_attached);
    assert!(report.argocd_sync_evidence_contract_attached);
    assert!(report.argocd_health_evidence_contract_attached);
    assert!(report.cosign_verification_evidence_contract_attached);
    assert!(report.kubernetes_rollout_evidence_contract_attached);
    assert!(report.gateway_route_acceptance_evidence_contract_attached);
    assert!(report.otel_resource_evidence_contract_attached);
    assert!(report.deployment_audit_event_evidence_contract_attached);
    assert!(!report.argocd_controller_attached);
    assert!(!report.gateway_controller_attached);
    assert!(!report.load_balancer_attached);
    assert!(!report.tls_certificate_attached);
    assert!(!report.cloud_deployment_evidence_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.runtime_audit_chain_emission_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-cloud-deployment-evidence-1779702000.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_workload_manifest_without_substrate_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_workload_manifest_ready);
    assert!(report.tenant_workload_manifest_attached);
    assert_eq!(report.tenant_workload_count, 4);
    assert!(report.fd001_product_goal_preserved);
    assert!(report.oyatie_cloud_substrate_dogfood_plan_attached);
    assert!(report.tenant_namespace_contract_attached);
    assert!(report.tenant_resource_quota_contract_attached);
    assert!(report.tenant_network_policy_contract_attached);
    assert!(report.tenant_gateway_route_contract_attached);
    assert!(report.tenant_admission_policy_contract_ready);
    assert_eq!(report.tenant_admission_policy_rule_count, 11);
    assert!(report.tenant_admission_policy_all_workloads_in_scope);
    assert!(report.tenant_resource_quota_contract_ready);
    assert_eq!(report.tenant_resource_quota_requirement_count, 13);
    assert!(report.tenant_resource_quota_all_workloads_in_scope);
    assert!(report.tenant_availability_contract_ready);
    assert_eq!(report.tenant_availability_requirement_count, 13);
    assert!(report.tenant_availability_all_workloads_in_scope);
    assert!(report.tenant_autoscaling_contract_ready);
    assert_eq!(report.tenant_autoscaling_requirement_count, 13);
    assert!(report.tenant_autoscaling_all_workloads_in_scope);
    assert!(report.tenant_cost_allocation_contract_ready);
    assert_eq!(report.tenant_cost_allocation_requirement_count, 13);
    assert!(report.tenant_cost_allocation_all_workloads_in_scope);
    assert!(report.tenant_residency_contract_ready);
    assert_eq!(report.tenant_residency_requirement_count, 13);
    assert!(report.tenant_residency_all_workloads_in_scope);
    assert!(report.tenant_workload_identity_contract_ready);
    assert_eq!(report.tenant_workload_identity_requirement_count, 13);
    assert!(report.tenant_workload_identity_all_workloads_in_scope);
    assert!(report.tenant_egress_policy_contract_ready);
    assert_eq!(report.tenant_egress_policy_rule_count, 11);
    assert!(report.tenant_egress_policy_all_workloads_in_scope);
    assert!(report.tenant_image_provenance_contract_ready);
    assert_eq!(report.tenant_image_provenance_requirement_count, 11);
    assert!(report.tenant_image_provenance_all_workloads_in_scope);
    assert!(report.tenant_secret_boundary_contract_ready);
    assert_eq!(report.tenant_secret_boundary_requirement_count, 11);
    assert!(report.tenant_secret_boundary_all_workloads_in_scope);
    assert!(report.tenant_admission_policy_contract_attached);
    assert!(!report.tenant_admission_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-admission-policy-1779706800.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-resource-quota-contract-1779709200.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-availability-contract-1779709800.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-cost-allocation-contract-1779711000.json"
    ));
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-residency-contract-1779711600.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_admission_policy_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_admission_policy_contract_ready);
    assert_eq!(report.tenant_admission_policy_rule_count, 11);
    assert!(report.tenant_admission_policy_all_workloads_in_scope);
    assert!(report.tenant_admission_policy_contract_attached);
    assert!(report.tenant_validating_admission_policy_contract_attached);
    assert!(report.tenant_admission_deny_action_contract_attached);
    assert!(report.tenant_pod_security_restricted_contract_attached);
    assert!(report.tenant_digest_pinned_image_admission_contract_attached);
    assert!(report.tenant_latest_image_tag_forbidden_contract_attached);
    assert!(report.tenant_resource_requests_limits_admission_contract_attached);
    assert!(report.tenant_service_account_admission_contract_attached);
    assert!(report.tenant_default_service_account_forbidden_contract_attached);
    assert!(report.tenant_admission_audit_annotation_contract_attached);
    assert!(!report.tenant_admission_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-admission-policy-1779706800.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_resource_quota_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_resource_quota_contract_ready);
    assert_eq!(report.tenant_resource_quota_requirement_count, 13);
    assert!(report.tenant_resource_quota_all_workloads_in_scope);
    assert!(report.tenant_resource_quota_policy_contract_attached);
    assert!(report.tenant_limit_range_policy_contract_attached);
    assert!(report.tenant_quota_compute_boundary_contract_attached);
    assert!(report.tenant_quota_storage_object_boundary_contract_attached);
    assert!(report.tenant_quota_admission_plugin_evidence_contract_attached);
    assert!(report.tenant_quota_usage_audit_contract_attached);
    assert!(!report.tenant_quota_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-resource-quota-contract-1779709200.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_availability_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_availability_contract_ready);
    assert_eq!(report.tenant_availability_requirement_count, 13);
    assert!(report.tenant_availability_all_workloads_in_scope);
    assert!(report.tenant_availability_contract_attached);
    assert!(report.tenant_pod_disruption_budget_contract_attached);
    assert!(report.tenant_topology_spread_contract_attached);
    assert!(report.tenant_pod_anti_affinity_contract_attached);
    assert!(report.tenant_rolling_update_availability_contract_attached);
    assert!(report.tenant_readiness_probe_evidence_contract_attached);
    assert!(report.tenant_disruption_audit_contract_attached);
    assert!(!report.tenant_availability_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-availability-contract-1779709800.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_autoscaling_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_autoscaling_contract_ready);
    assert_eq!(report.tenant_autoscaling_requirement_count, 13);
    assert!(report.tenant_autoscaling_all_workloads_in_scope);
    assert!(report.tenant_autoscaling_contract_attached);
    assert!(report.tenant_horizontal_pod_autoscaler_contract_attached);
    assert!(report.tenant_autoscaling_metrics_pipeline_contract_attached);
    assert!(report.tenant_autoscaling_replica_bounds_contract_attached);
    assert!(report.tenant_autoscaling_behavior_policy_contract_attached);
    assert!(report.tenant_autoscaling_audit_contract_attached);
    assert!(!report.tenant_autoscaling_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-autoscaling-contract-1779710400.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_cost_allocation_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_cost_allocation_contract_ready);
    assert_eq!(report.tenant_cost_allocation_requirement_count, 13);
    assert!(report.tenant_cost_allocation_all_workloads_in_scope);
    assert!(report.tenant_cost_allocation_contract_attached);
    assert!(report.tenant_cost_label_contract_attached);
    assert!(report.tenant_cost_resource_basis_contract_attached);
    assert!(report.tenant_cost_otel_resource_contract_attached);
    assert!(report.tenant_cost_finops_allocation_contract_attached);
    assert!(report.tenant_cost_shared_cost_contract_attached);
    assert!(report.tenant_cost_audit_contract_attached);
    assert!(!report.tenant_cost_allocation_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-cost-allocation-contract-1779711000.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_residency_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_residency_contract_ready);
    assert_eq!(report.tenant_residency_requirement_count, 13);
    assert!(report.tenant_residency_all_workloads_in_scope);
    assert!(report.tenant_residency_contract_attached);
    assert!(report.tenant_residency_label_contract_attached);
    assert!(report.tenant_residency_scheduling_contract_attached);
    assert!(report.tenant_residency_storage_contract_attached);
    assert!(report.tenant_residency_telemetry_contract_attached);
    assert!(report.tenant_residency_audit_contract_attached);
    assert!(report.tenant_residency_egress_contract_attached);
    assert!(report.tenant_residency_model_contract_attached);
    assert!(!report.tenant_residency_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-residency-contract-1779711600.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_workload_identity_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_workload_identity_contract_ready);
    assert_eq!(report.tenant_workload_identity_requirement_count, 13);
    assert!(report.tenant_workload_identity_all_workloads_in_scope);
    assert!(report.tenant_workload_identity_contract_attached);
    assert!(report.tenant_spiffe_id_contract_attached);
    assert!(report.tenant_svid_contract_attached);
    assert!(report.tenant_mtls_contract_attached);
    assert!(report.tenant_gateway_backend_tls_contract_attached);
    assert!(report.tenant_trust_bundle_contract_attached);
    assert!(report.tenant_identity_telemetry_contract_attached);
    assert!(report.tenant_identity_audit_contract_attached);
    assert!(!report.tenant_workload_identity_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-workload-identity-contract-1779712200.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_egress_policy_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_egress_policy_contract_ready);
    assert_eq!(report.tenant_egress_policy_rule_count, 11);
    assert!(report.tenant_egress_policy_all_workloads_in_scope);
    assert!(report.tenant_egress_policy_contract_attached);
    assert!(report.tenant_default_deny_egress_contract_attached);
    assert!(report.tenant_dns_egress_contract_attached);
    assert!(report.tenant_cross_namespace_egress_contract_attached);
    assert!(report.tenant_external_egress_exception_contract_attached);
    assert!(report.tenant_egress_audit_contract_attached);
    assert!(!report.tenant_egress_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-egress-policy-contract-1779708600.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_image_provenance_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_image_provenance_contract_ready);
    assert_eq!(report.tenant_image_provenance_requirement_count, 11);
    assert!(report.tenant_image_provenance_all_workloads_in_scope);
    assert!(report.tenant_image_provenance_contract_attached);
    assert!(report.tenant_image_provenance_slsa_contract_attached);
    assert!(report.tenant_image_provenance_sbom_contract_attached);
    assert!(report.tenant_image_provenance_cosign_contract_attached);
    assert!(report.tenant_image_provenance_admission_contract_attached);
    assert!(!report.tenant_image_provenance_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-image-provenance-contract-1779707400.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_secret_boundary_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_secret_boundary_contract_ready);
    assert_eq!(report.tenant_secret_boundary_requirement_count, 11);
    assert!(report.tenant_secret_boundary_all_workloads_in_scope);
    assert!(report.tenant_secret_boundary_contract_attached);
    assert!(report.tenant_secret_ref_boundary_contract_attached);
    assert!(report.tenant_secret_encryption_contract_attached);
    assert!(report.tenant_secret_rbac_contract_attached);
    assert!(report.tenant_secret_rotation_audit_contract_attached);
    assert!(!report.tenant_secret_runtime_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-secret-boundary-contract-1779708000.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_tenant_runtime_evidence_plan_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.tenant_workload_runtime_evidence_plan_ready);
    assert_eq!(
        report.tenant_workload_runtime_evidence_requirement_count,
        14
    );
    assert!(report.tenant_workload_runtime_evidence_plan_attached);
    assert!(report.tenant_runtime_namespace_evidence_contract_attached);
    assert!(report.tenant_runtime_quota_evidence_contract_attached);
    assert!(report.tenant_runtime_network_policy_evidence_contract_attached);
    assert!(report.tenant_runtime_service_account_evidence_contract_attached);
    assert!(report.tenant_runtime_pod_security_evidence_contract_attached);
    assert!(report.tenant_runtime_workload_schedule_evidence_contract_attached);
    assert!(report.tenant_runtime_probe_evidence_contract_attached);
    assert!(report.tenant_runtime_gateway_route_evidence_contract_attached);
    assert!(report.tenant_runtime_claim_evidence_contract_attached);
    assert!(report.tenant_runtime_otel_resource_evidence_contract_attached);
    assert!(report.tenant_runtime_rollout_recovery_evidence_contract_attached);
    assert!(report.tenant_runtime_audit_event_evidence_contract_attached);
    assert!(!report.tenant_workload_runtime_evidence_attached);
    assert!(!report.tenant_workload_runtime_attached);
    assert!(!report.tenant_cloud_substrate_runtime_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::CloudDeploymentEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-tenant-runtime-evidence-1779705000.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_auth_runtime_foundation_without_oidc_provider_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.authentication_runtime_ready);
    assert!(report.authentication_runtime_attached);
    assert!(!report.identity_provider_verification_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::IdentityProviderVerificationMissing)
    );
    assert!(
        report
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-platform-auth-runtime-1779552000.json")
    );
}

#[test]
fn cloud_readiness_gate_includes_identity_provider_verification_plan_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.identity_provider_verification_plan_ready);
    assert!(report.identity_provider_verification_plan_attached);
    assert!(report.oidc_discovery_plan_attached);
    assert!(report.jwks_validation_plan_attached);
    assert!(!report.oidc_signature_verification_attached);
    assert!(!report.jwks_provider_attached);
    assert!(!report.identity_provider_verification_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::IdentityProviderVerificationMissing)
    );
    assert!(
        report
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-platform-idp-verification-1779553800.json")
    );
}

#[test]
fn cloud_readiness_gate_includes_identity_provider_runtime_evidence_plan_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.identity_provider_runtime_evidence_plan_ready);
    assert_eq!(
        report.identity_provider_runtime_evidence_requirement_count,
        15
    );
    assert!(report.identity_provider_runtime_evidence_plan_attached);
    assert!(report.oidc_discovery_runtime_evidence_contract_attached);
    assert!(report.jwks_runtime_evidence_contract_attached);
    assert!(report.jwt_signature_evidence_contract_attached);
    assert!(report.jwt_claims_evidence_contract_attached);
    assert!(report.nonce_replay_evidence_contract_attached);
    assert!(report.tenant_scope_evidence_contract_attached);
    assert!(report.sensitive_route_mfa_evidence_contract_attached);
    assert!(report.key_rotation_evidence_contract_attached);
    assert!(report.auth_failure_audit_event_evidence_contract_attached);
    assert!(!report.oidc_signature_verification_attached);
    assert!(!report.jwks_provider_attached);
    assert!(!report.identity_provider_verification_attached);
    assert!(!report.identity_provider_runtime_evidence_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::IdentityProviderVerificationMissing)
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-idp-runtime-evidence-1779703200.json"
        )
    );
}

#[test]
fn cloud_readiness_gate_includes_postgres_rls_storage_plan_without_database_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.postgres_rls_storage_plan_ready);
    assert!(report.postgres_rls_storage_plan_attached);
    assert!(report.postgres_rls_write_contract_ready);
    assert_eq!(report.postgres_rls_write_statement_count, 5);
    assert!(report.postgres_rls_write_contract_attached);
    assert!(report.postgres_set_local_tenant_context_contract_attached);
    assert!(report.postgres_parameterized_insert_contract_attached);
    assert!(report.postgres_idempotency_conflict_contract_attached);
    assert!(report.postgres_tenant_scoped_readback_contract_attached);
    assert!(report.postgres_delete_statement_forbidden_contract_attached);
    assert!(!report.postgres_write_runtime_attached);
    assert!(report.postgres_rls_transaction_contract_ready);
    assert_eq!(report.postgres_rls_transaction_plan_count, 5);
    assert!(report.postgres_rls_transaction_contract_attached);
    assert!(report.postgres_explicit_transaction_contract_attached);
    assert!(report.postgres_transaction_local_tenant_context_contract_attached);
    assert!(report.postgres_prepared_statement_contract_attached);
    assert!(report.postgres_bound_parameter_execution_contract_attached);
    assert!(report.postgres_commit_rollback_contract_attached);
    assert!(!report.postgres_transaction_runtime_attached);
    assert!(!report.postgres_prepared_statement_runtime_attached);
    assert!(report.postgres_rls_runtime_evidence_plan_ready);
    assert!(report.postgres_rls_runtime_evidence_plan_attached);
    assert!(report.postgres_migration_rehearsal_contract_attached);
    assert!(report.postgres_tls_verify_full_contract_attached);
    assert!(report.postgres_rls_probe_matrix_attached);
    assert!(report.postgres_backup_restore_rehearsal_contract_attached);
    assert!(report.postgres_pitr_rehearsal_contract_attached);
    assert!(!report.durable_business_storage_attached);
    assert!(!report.postgres_database_attached);
    assert!(!report.postgres_rls_runtime_verified_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DurableStorageRuntimeMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::PostgresRlsRuntimeEvidenceMissing)
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-postgres-rls-storage-1779552600.json"
        )
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-postgres-write-contract-1779705600.json"
    ));
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-postgres-tx-contract-1779706200.json"
        )
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-postgres-rls-runtime-evidence-1779666600.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_postgres_write_contract_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.postgres_rls_write_contract_ready);
    assert_eq!(report.postgres_rls_write_statement_count, 5);
    assert!(report.postgres_rls_write_contract_attached);
    assert!(report.postgres_set_local_tenant_context_contract_attached);
    assert!(report.postgres_parameterized_insert_contract_attached);
    assert!(report.postgres_idempotency_conflict_contract_attached);
    assert!(report.postgres_tenant_scoped_readback_contract_attached);
    assert!(report.postgres_delete_statement_forbidden_contract_attached);
    assert!(!report.postgres_write_runtime_attached);
    assert!(report.postgres_rls_transaction_contract_ready);
    assert_eq!(report.postgres_rls_transaction_plan_count, 5);
    assert!(report.postgres_rls_transaction_contract_attached);
    assert!(!report.postgres_database_attached);
    assert!(!report.durable_business_storage_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DurableStorageRuntimeMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-postgres-write-contract-1779705600.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_postgres_transaction_contract_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.postgres_rls_transaction_contract_ready);
    assert_eq!(report.postgres_rls_transaction_plan_count, 5);
    assert!(report.postgres_rls_transaction_contract_attached);
    assert!(report.postgres_explicit_transaction_contract_attached);
    assert!(report.postgres_transaction_local_tenant_context_contract_attached);
    assert!(report.postgres_prepared_statement_contract_attached);
    assert!(report.postgres_bound_parameter_execution_contract_attached);
    assert!(report.postgres_commit_rollback_contract_attached);
    assert!(!report.postgres_transaction_runtime_attached);
    assert!(!report.postgres_prepared_statement_runtime_attached);
    assert!(!report.postgres_write_runtime_attached);
    assert!(!report.postgres_database_attached);
    assert!(!report.durable_business_storage_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DurableStorageRuntimeMissing)
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-postgres-tx-contract-1779706200.json"
        )
    );
}

#[test]
fn cloud_readiness_gate_includes_postgres_rls_runtime_evidence_plan_without_live_database_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.postgres_rls_runtime_evidence_plan_ready);
    assert!(report.postgres_rls_runtime_evidence_plan_attached);
    assert!(report.postgres_migration_rehearsal_contract_attached);
    assert!(report.postgres_tls_verify_full_contract_attached);
    assert!(report.postgres_rls_probe_matrix_attached);
    assert!(report.postgres_backup_restore_rehearsal_contract_attached);
    assert!(report.postgres_pitr_rehearsal_contract_attached);
    assert!(!report.postgres_database_attached);
    assert!(!report.postgres_rls_runtime_verified_attached);
    assert!(!report.durable_business_storage_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DurableStorageRuntimeMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::PostgresRlsRuntimeEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-postgres-rls-runtime-evidence-1779666600.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_listener_gateway_plan_without_deployed_listener_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.listener_gateway_plan_ready);
    assert!(report.listener_gateway_plan_attached);
    assert!(!report.deployed_listener_attached);
    assert!(!report.listener_runtime_attached);
    assert!(!report.gateway_controller_attached);
    assert!(!report.load_balancer_attached);
    assert!(!report.tls_certificate_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DeployedListenerRuntimeEvidenceMissing)
    );
    assert!(
        report
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-platform-listener-gateway-1779553200.json")
    );
}

#[test]
fn cloud_readiness_gate_includes_listener_runtime_evidence_plan_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.listener_runtime_evidence_plan_ready);
    assert_eq!(report.listener_runtime_evidence_requirement_count, 12);
    assert!(report.listener_runtime_evidence_plan_attached);
    assert!(report.cluster_ip_service_evidence_contract_attached);
    assert!(report.gateway_route_runtime_acceptance_contract_attached);
    assert!(report.tls_certificate_binding_evidence_contract_attached);
    assert!(report.listener_probe_evidence_contract_attached);
    assert!(report.route_authz_evidence_contract_attached);
    assert!(report.network_policy_evidence_contract_attached);
    assert!(report.endpoint_slice_evidence_contract_attached);
    assert!(report.listener_audit_event_evidence_contract_attached);
    assert!(!report.deployed_listener_attached);
    assert!(!report.listener_runtime_attached);
    assert!(!report.gateway_controller_attached);
    assert!(!report.load_balancer_attached);
    assert!(!report.tls_certificate_attached);
    assert!(!report.runtime_audit_chain_emission_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DeployedListenerRuntimeEvidenceMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-listener-runtime-evidence-1779702600.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_audit_chain_emission_plan_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.audit_chain_emission_plan_ready);
    assert!(report.audit_chain_emission_plan_attached);
    assert!(report.audit_chain_event_contract_attached);
    assert!(report.audit_chain_wal_plan_attached);
    assert!(report.audit_chain_outbox_plan_attached);
    assert!(report.workflow_execution_reference_attached);
    assert!(!report.workflow_engine_execution_attached);
    assert!(!report.workflow_broker_publish_attached);
    assert!(!report.workflow_durable_queue_attached);
    assert!(report.statutory_filing_evidence_plan_attached);
    assert!(!report.statutory_runtime_submission_attached);
    assert!(!report.statutory_disbursement_rail_attached);
    assert!(report.disbursement_evidence_plan_attached);
    assert!(!report.disbursement_runtime_execution_attached);
    assert!(report.slo_evidence_plan_attached);
    assert!(report.slo_error_budget_release_gate_attached);
    assert!(report.slo_burn_rate_alert_plan_attached);
    assert!(report.slo_openslo_manifests_attached);
    assert!(report.slo_otel_metric_streams_attached);
    assert!(!report.runtime_audit_chain_emission_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::RuntimeAuditEmissionMissing)
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-audit-chain-emission-1779661200.json"
        )
    );
}

#[test]
fn cloud_readiness_gate_includes_audit_chain_runtime_evidence_plan_without_runtime_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.audit_chain_runtime_evidence_plan_ready);
    assert_eq!(report.audit_chain_runtime_evidence_requirement_count, 15);
    assert!(report.audit_chain_runtime_evidence_plan_attached);
    assert!(report.audit_chain_event_envelope_evidence_contract_attached);
    assert!(report.audit_chain_trace_context_evidence_contract_attached);
    assert!(report.audit_chain_otel_log_mapping_evidence_contract_attached);
    assert!(report.audit_chain_tenant_partition_evidence_contract_attached);
    assert!(report.audit_chain_payload_digest_evidence_contract_attached);
    assert!(report.audit_chain_wal_append_evidence_contract_attached);
    assert!(report.audit_chain_outbox_publish_evidence_contract_attached);
    assert!(report.audit_chain_broker_ack_evidence_contract_attached);
    assert!(report.audit_chain_merkle_seal_evidence_contract_attached);
    assert!(report.audit_chain_sink_ingestion_evidence_contract_attached);
    assert!(report.audit_chain_replay_recovery_evidence_contract_attached);
    assert!(report.audit_chain_failure_path_evidence_contract_attached);
    assert!(!report.audit_chain_runtime_evidence_attached);
    assert!(!report.runtime_audit_chain_emission_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::RuntimeAuditEmissionMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-audit-runtime-evidence-1779704400.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_in_memory_workflow_execution_reference_without_engine_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.workflow_execution_reference_ready);
    assert!(report.workflow_execution_reference_attached);
    assert!(!report.workflow_engine_execution_attached);
    assert!(!report.workflow_broker_publish_attached);
    assert!(!report.workflow_durable_queue_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::WorkflowEngineMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::BrokerPublishMissing)
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-tenant-rbac-workflow-execution-1779664800.json"
        )
    );
}

#[test]
fn cloud_readiness_gate_includes_workflow_runtime_evidence_plan_without_engine_or_broker_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.workflow_runtime_evidence_plan_ready);
    assert_eq!(report.workflow_runtime_evidence_requirement_count, 14);
    assert!(report.workflow_runtime_evidence_plan_attached);
    assert!(report.workflow_definition_evidence_contract_attached);
    assert!(report.workflow_gate_evidence_contract_attached);
    assert!(report.workflow_durable_queue_evidence_contract_attached);
    assert!(report.workflow_broker_publish_evidence_contract_attached);
    assert!(report.workflow_broker_retry_dlq_evidence_contract_attached);
    assert!(report.workflow_tenant_partition_evidence_contract_attached);
    assert!(report.workflow_otel_trace_evidence_contract_attached);
    assert!(report.workflow_audit_event_evidence_contract_attached);
    assert!(report.workflow_replay_recovery_evidence_contract_attached);
    assert!(!report.workflow_engine_execution_attached);
    assert!(!report.workflow_broker_publish_attached);
    assert!(!report.workflow_durable_queue_attached);
    assert!(!report.workflow_runtime_evidence_attached);
    assert!(!report.runtime_audit_chain_emission_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::WorkflowEngineMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::BrokerPublishMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-tenant-rbac-workflow-runtime-evidence-1779703800.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_statutory_filing_evidence_plan_without_rail_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.statutory_filing_evidence_plan_ready);
    assert!(report.statutory_filing_evidence_plan_attached);
    assert!(report.statutory_authority_registry_attached);
    assert!(report.statutory_payload_digest_contract_attached);
    assert!(report.statutory_agency_receipt_contract_attached);
    assert!(!report.statutory_runtime_submission_attached);
    assert!(!report.statutory_disbursement_rail_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::StatutoryFilingRailMissing)
    );
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DisbursementRailMissing)
    );
    assert!(report.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-statutory-filing-evidence-1779665400.json"
    ));
}

#[test]
fn cloud_readiness_gate_includes_disbursement_evidence_plan_without_payment_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.disbursement_evidence_plan_ready);
    assert!(report.disbursement_evidence_plan_attached);
    assert!(report.disbursement_network_registry_attached);
    assert!(report.disbursement_payment_digest_contract_attached);
    assert!(report.disbursement_reconciliation_contract_attached);
    assert!(!report.disbursement_runtime_execution_attached);
    assert!(!report.disbursement_bank_connection_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::DisbursementRailMissing)
    );
    assert!(
        report.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-platform-disbursement-evidence-1779666000.json"
        )
    );
}

#[test]
fn cloud_readiness_gate_includes_slo_evidence_plan_without_production_slo_claim() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");

    assert!(report.slo_evidence_plan_ready);
    assert!(report.slo_evidence_plan_attached);
    assert!(report.slo_error_budget_release_gate_attached);
    assert!(report.slo_burn_rate_alert_plan_attached);
    assert!(report.slo_openslo_manifests_attached);
    assert!(report.slo_otel_metric_streams_attached);
    assert!(!report.multi_region_slo_evidence_attached);
    assert!(!report.cloud_deployment_ready);
    assert!(
        report
            .blockers
            .contains(&CloudReadinessBlocker::SloEvidenceMissing)
    );
    assert!(
        report
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-platform-slo-evidence-1779664200.json")
    );
}

#[test]
fn cloud_claim_validation_refuses_unresolved_blockers() {
    let report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");
    let error = validate_cloud_claim(&report).expect_err("cloud claim remains blocked");
    assert!(matches!(
        error,
        CloudReadinessGateError::CloudClaimBlocked(blockers)
            if blockers.contains(&CloudReadinessBlocker::WorkflowEngineMissing)
    ));
}

#[test]
fn cloud_claim_validation_detects_false_positive_ready_flag() {
    let mut report = tenant_rbac_cloud_readiness_report().expect("readiness report builds");
    report.cloud_deployment_ready = true;
    let error = validate_cloud_claim(&report).expect_err("ready flag cannot override blockers");
    assert!(matches!(
        error,
        CloudReadinessGateError::CloudClaimBlocked(blockers)
            if blockers.len() == report.blocker_count
    ));
}

#[test]
fn required_cloud_blockers_are_stable_and_named() {
    let blockers = required_cloud_blockers();
    assert_eq!(blockers.len(), 11);
    let names: Vec<_> = blockers.iter().map(|blocker| blocker.as_str()).collect();
    assert!(names.contains(&"deployed_listener_runtime_evidence_missing"));
    assert!(names.contains(&"identity_provider_verification_missing"));
    assert!(names.contains(&"postgres_rls_runtime_evidence_missing"));
    assert!(names.contains(&"slo_evidence_missing"));
}
