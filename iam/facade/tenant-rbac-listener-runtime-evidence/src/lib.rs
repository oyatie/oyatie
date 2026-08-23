//! Tenant RBAC deployed listener runtime evidence contract.
//!
//! This review-only crate records the listener runtime evidence that must exist
//! before FD-001 tenant workloads can claim a deployed Tenant RBAC listener
//! on the future Oyatie Cloud substrate. It validates source refs, official
//! runtime evidence requirements, and non-claim boundaries, but it does not bind
//! sockets, query Kubernetes, attach a Gateway controller, provision load
//! balancers or certificates, enforce runtime auth middleware, export telemetry,
//! emit audit-chain events, or claim production listener evidence.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_listener_gateway::{
    TenantRbacListenerGatewayError, tenant_rbac_listener_gateway_plan,
    validate_tenant_rbac_listener_gateway_plan,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 12;
const PLAN_NAME: &str = "tenant-rbac-listener-runtime-evidence-plan";
const SERVICE_NAME: &str = "tenant-rbac";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const EXPECTED_ROUTE_COUNT: usize = 19;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ListenerRuntimeEvidenceRequirementKind {
    ClusterIpServiceObserved,
    GatewayHttpRouteAccepted,
    TlsCertificateBound,
    ReadinessProbeSucceeded,
    LivenessProbeSucceeded,
    SyntheticHealthCheckSucceeded,
    RouteAuthzEnforced,
    NetworkPolicyIngressDenied,
    EndpointSliceReadyObserved,
    GracefulShutdownObserved,
    AccessLogTraceCorrelated,
    ListenerDeploymentAuditRecorded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListenerRuntimeEvidenceRequirement {
    pub requirement_id: &'static str, // data_class: PUBLIC
    pub requirement_kind: ListenerRuntimeEvidenceRequirementKind, // data_class: PUBLIC
    pub workload_scope: &'static str, // data_class: PUBLIC
    pub official_doc_url: &'static str, // data_class: PUBLIC
    pub expected_evidence_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_plan_ref: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_namespace: &'static str, // data_class: INTERNAL_ONLY
    pub required_route_count: usize,  // data_class: PUBLIC
    pub requires_cluster_observation: bool, // data_class: PUBLIC
    pub requires_gateway_observation: bool, // data_class: PUBLIC
    pub requires_runtime_probe: bool, // data_class: PUBLIC
    pub requires_security_enforcement: bool, // data_class: PUBLIC
    pub runtime_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacListenerRuntimeEvidencePlan {
    pub plan_name: &'static str,                     // data_class: PUBLIC
    pub service_name: &'static str,                  // data_class: PUBLIC
    pub substrate_name: &'static str,                // data_class: PUBLIC
    pub tenant_namespace: &'static str,              // data_class: INTERNAL_ONLY
    pub listener_gateway_plan_name: &'static str,    // data_class: INTERNAL_ONLY
    pub listener_gateway_route_count: usize,         // data_class: PUBLIC
    pub listener_gateway_service_type: &'static str, // data_class: PUBLIC
    pub requirements: Vec<ListenerRuntimeEvidenceRequirement>, // data_class: INTERNAL_ONLY
    pub fd001_product_delivery_master_goal_preserved: bool, // data_class: PUBLIC
    pub oyatie_cloud_substrate_proof_required: bool, // data_class: PUBLIC
    pub official_docs_required: bool,                // data_class: PUBLIC
    pub cluster_ip_service_observation_required: bool, // data_class: PUBLIC
    pub gateway_route_acceptance_required: bool,     // data_class: PUBLIC
    pub tls_certificate_binding_required: bool,      // data_class: PUBLIC
    pub readiness_probe_success_required: bool,      // data_class: PUBLIC
    pub liveness_probe_success_required: bool,       // data_class: PUBLIC
    pub synthetic_health_check_required: bool,       // data_class: PUBLIC
    pub route_authz_enforcement_required: bool,      // data_class: PUBLIC
    pub default_deny_network_policy_required: bool,  // data_class: PUBLIC
    pub endpoint_slice_ready_required: bool,         // data_class: PUBLIC
    pub graceful_shutdown_drain_required: bool,      // data_class: PUBLIC
    pub access_log_trace_correlation_required: bool, // data_class: PUBLIC
    pub listener_deployment_audit_event_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,                  // data_class: PUBLIC
    pub deployed_listener_attached: bool,            // data_class: INTERNAL_ONLY
    pub bound_socket_attached: bool,                 // data_class: INTERNAL_ONLY
    pub gateway_controller_attached: bool,           // data_class: INTERNAL_ONLY
    pub load_balancer_attached: bool,                // data_class: INTERNAL_ONLY
    pub tls_certificate_attached: bool,              // data_class: INTERNAL_ONLY
    pub runtime_auth_middleware_attached: bool,      // data_class: INTERNAL_ONLY
    pub network_policy_applied_attached: bool,       // data_class: INTERNAL_ONLY
    pub readiness_probe_runtime_attached: bool,      // data_class: INTERNAL_ONLY
    pub liveness_probe_runtime_attached: bool,       // data_class: INTERNAL_ONLY
    pub production_listener_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacListenerRuntimeEvidenceError {
    ListenerGateway(TenantRbacListenerGatewayError),
    InvalidPlanName,
    InvalidServiceName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidListenerGatewayPlanName,
    InvalidListenerGatewayRouteCount,
    InvalidListenerGatewayServiceType,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingRequirementKind(ListenerRuntimeEvidenceRequirementKind),
    InvalidRequirementId,
    InvalidWorkloadScope,
    InvalidOfficialDocUrl,
    InvalidExpectedEvidenceRef,
    InvalidSourcePlanRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_listener_runtime_evidence_plan()
-> Result<TenantRbacListenerRuntimeEvidencePlan, TenantRbacListenerRuntimeEvidenceError> {
    let listener_gateway_plan = tenant_rbac_listener_gateway_plan();
    validate_tenant_rbac_listener_gateway_plan(&listener_gateway_plan)
        .map_err(TenantRbacListenerRuntimeEvidenceError::ListenerGateway)?;

    Ok(TenantRbacListenerRuntimeEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: TENANT_NAMESPACE,
        listener_gateway_plan_name: listener_gateway_plan.plan_name,
        listener_gateway_route_count: listener_gateway_plan.route_count,
        listener_gateway_service_type: listener_gateway_plan.kubernetes_service_type,
        requirements: vec![
            requirement(
                "clusterip-service-observed",
                ListenerRuntimeEvidenceRequirementKind::ClusterIpServiceObserved,
                "tenant-rbac-service",
                "https://kubernetes.io/docs/concepts/services-networking/service/",
                "evidence/listener-runtime/tenant-rbac/service-clusterip.json",
            ),
            requirement(
                "gateway-httproute-accepted",
                ListenerRuntimeEvidenceRequirementKind::GatewayHttpRouteAccepted,
                "tenant-rbac-gateway-route",
                "https://gateway-api.sigs.k8s.io/concepts/api-overview/",
                "evidence/listener-runtime/tenant-rbac/gateway-httproute-accepted.json",
            ),
            requirement(
                "tls-certificate-bound",
                ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound,
                "tenant-rbac-gateway-listener",
                "https://gateway-api.sigs.k8s.io/concepts/api-overview/",
                "evidence/listener-runtime/tenant-rbac/tls-certificate-bound.json",
            ),
            requirement(
                "readiness-probe-succeeded",
                ListenerRuntimeEvidenceRequirementKind::ReadinessProbeSucceeded,
                "tenant-rbac-runtime",
                "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/",
                "evidence/listener-runtime/tenant-rbac/readiness-probe.json",
            ),
            requirement(
                "liveness-probe-succeeded",
                ListenerRuntimeEvidenceRequirementKind::LivenessProbeSucceeded,
                "tenant-rbac-runtime",
                "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/",
                "evidence/listener-runtime/tenant-rbac/liveness-probe.json",
            ),
            requirement(
                "synthetic-health-check-succeeded",
                ListenerRuntimeEvidenceRequirementKind::SyntheticHealthCheckSucceeded,
                "tenant-rbac-runtime",
                "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/",
                "evidence/listener-runtime/tenant-rbac/synthetic-health-check.json",
            ),
            requirement(
                "route-authz-enforced",
                ListenerRuntimeEvidenceRequirementKind::RouteAuthzEnforced,
                "tenant-rbac-routes",
                "https://kubernetes.io/docs/concepts/services-networking/service/",
                "evidence/listener-runtime/tenant-rbac/route-authz.json",
            ),
            requirement(
                "network-policy-ingress-denied",
                ListenerRuntimeEvidenceRequirementKind::NetworkPolicyIngressDenied,
                "tenant-rbac-network-policy",
                "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
                "evidence/listener-runtime/tenant-rbac/network-policy-ingress-denied.json",
            ),
            requirement(
                "endpoint-slice-ready-observed",
                ListenerRuntimeEvidenceRequirementKind::EndpointSliceReadyObserved,
                "tenant-rbac-service",
                "https://kubernetes.io/docs/concepts/services-networking/endpoint-slices/",
                "evidence/listener-runtime/tenant-rbac/endpoint-slice-ready.json",
            ),
            requirement(
                "graceful-shutdown-observed",
                ListenerRuntimeEvidenceRequirementKind::GracefulShutdownObserved,
                "tenant-rbac-runtime",
                "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/",
                "evidence/listener-runtime/tenant-rbac/graceful-shutdown.json",
            ),
            requirement(
                "access-log-trace-correlated",
                ListenerRuntimeEvidenceRequirementKind::AccessLogTraceCorrelated,
                "tenant-rbac-runtime",
                "https://opentelemetry.io/docs/specs/semconv/http/http-spans/",
                "evidence/listener-runtime/tenant-rbac/access-log-trace-correlation.json",
            ),
            requirement(
                "listener-deployment-audit-event-recorded",
                ListenerRuntimeEvidenceRequirementKind::ListenerDeploymentAuditRecorded,
                "tenant-rbac-runtime",
                "https://cloudevents.io/",
                "evidence/listener-runtime/tenant-rbac/listener-audit-event.json",
            ),
        ],
        fd001_product_delivery_master_goal_preserved: true,
        oyatie_cloud_substrate_proof_required: true,
        official_docs_required: true,
        cluster_ip_service_observation_required: true,
        gateway_route_acceptance_required: true,
        tls_certificate_binding_required: true,
        readiness_probe_success_required: true,
        liveness_probe_success_required: true,
        synthetic_health_check_required: true,
        route_authz_enforcement_required: true,
        default_deny_network_policy_required: true,
        endpoint_slice_ready_required: true,
        graceful_shutdown_drain_required: true,
        access_log_trace_correlation_required: true,
        listener_deployment_audit_event_required: true,
        review_only_contract: true,
        deployed_listener_attached: false,
        bound_socket_attached: false,
        gateway_controller_attached: false,
        load_balancer_attached: false,
        tls_certificate_attached: false,
        runtime_auth_middleware_attached: false,
        network_policy_applied_attached: false,
        readiness_probe_runtime_attached: false,
        liveness_probe_runtime_attached: false,
        production_listener_evidence_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_tenant_rbac_listener_runtime_evidence_plan(
    plan: &TenantRbacListenerRuntimeEvidencePlan,
) -> Result<(), TenantRbacListenerRuntimeEvidenceError> {
    validate_slug(
        plan.plan_name,
        TenantRbacListenerRuntimeEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidServiceName);
    }
    if plan.substrate_name != SUBSTRATE_NAME {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidSubstrateName);
    }
    if plan.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if plan.listener_gateway_plan_name != "tenant-rbac-listener-gateway-plan" {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidListenerGatewayPlanName);
    }
    if plan.listener_gateway_route_count != EXPECTED_ROUTE_COUNT {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidListenerGatewayRouteCount);
    }
    if plan.listener_gateway_service_type != "ClusterIP" {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidListenerGatewayServiceType);
    }
    if plan.requirements.len() < MIN_REQUIREMENT_COUNT {
        return Err(TenantRbacListenerRuntimeEvidenceError::MissingRequirements);
    }
    for control in [
        (
            plan.fd001_product_delivery_master_goal_preserved,
            "fd001_product_delivery_master_goal_preserved",
        ),
        (
            plan.oyatie_cloud_substrate_proof_required,
            "oyatie_cloud_substrate_proof_required",
        ),
        (plan.official_docs_required, "official_docs_required"),
        (
            plan.cluster_ip_service_observation_required,
            "cluster_ip_service_observation_required",
        ),
        (
            plan.gateway_route_acceptance_required,
            "gateway_route_acceptance_required",
        ),
        (
            plan.tls_certificate_binding_required,
            "tls_certificate_binding_required",
        ),
        (
            plan.readiness_probe_success_required,
            "readiness_probe_success_required",
        ),
        (
            plan.liveness_probe_success_required,
            "liveness_probe_success_required",
        ),
        (
            plan.synthetic_health_check_required,
            "synthetic_health_check_required",
        ),
        (
            plan.route_authz_enforcement_required,
            "route_authz_enforcement_required",
        ),
        (
            plan.default_deny_network_policy_required,
            "default_deny_network_policy_required",
        ),
        (
            plan.endpoint_slice_ready_required,
            "endpoint_slice_ready_required",
        ),
        (
            plan.graceful_shutdown_drain_required,
            "graceful_shutdown_drain_required",
        ),
        (
            plan.access_log_trace_correlation_required,
            "access_log_trace_correlation_required",
        ),
        (
            plan.listener_deployment_audit_event_required,
            "listener_deployment_audit_event_required",
        ),
        (plan.review_only_contract, "review_only_contract"),
    ] {
        require_control(control.0, control.1)?;
    }
    if plan.deployed_listener_attached
        || plan.bound_socket_attached
        || plan.gateway_controller_attached
        || plan.load_balancer_attached
        || plan.tls_certificate_attached
        || plan.runtime_auth_middleware_attached
        || plan.network_policy_applied_attached
        || plan.readiness_probe_runtime_attached
        || plan.liveness_probe_runtime_attached
        || plan.production_listener_evidence_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacListenerRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }

    let mut seen_requirements = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(
                TenantRbacListenerRuntimeEvidenceError::DuplicateRequirement(
                    requirement.requirement_id.to_owned(),
                ),
            );
        }
        seen_kinds.insert(requirement.requirement_kind);
    }
    for kind in required_requirement_kinds() {
        if !seen_kinds.contains(&kind) {
            return Err(TenantRbacListenerRuntimeEvidenceError::MissingRequirementKind(kind));
        }
    }
    Ok(())
}

pub fn listener_runtime_evidence_doc_urls(
    plan: &TenantRbacListenerRuntimeEvidencePlan,
) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.official_doc_url)
        .collect()
}

fn requirement(
    requirement_id: &'static str,
    requirement_kind: ListenerRuntimeEvidenceRequirementKind,
    workload_scope: &'static str,
    official_doc_url: &'static str,
    expected_evidence_ref: &'static str,
) -> ListenerRuntimeEvidenceRequirement {
    let requires_gateway_observation = matches!(
        requirement_kind,
        ListenerRuntimeEvidenceRequirementKind::GatewayHttpRouteAccepted
            | ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound
    );
    let requires_runtime_probe = matches!(
        requirement_kind,
        ListenerRuntimeEvidenceRequirementKind::ReadinessProbeSucceeded
            | ListenerRuntimeEvidenceRequirementKind::LivenessProbeSucceeded
            | ListenerRuntimeEvidenceRequirementKind::SyntheticHealthCheckSucceeded
            | ListenerRuntimeEvidenceRequirementKind::GracefulShutdownObserved
    );
    let requires_security_enforcement = matches!(
        requirement_kind,
        ListenerRuntimeEvidenceRequirementKind::RouteAuthzEnforced
            | ListenerRuntimeEvidenceRequirementKind::NetworkPolicyIngressDenied
            | ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound
    );

    ListenerRuntimeEvidenceRequirement {
        requirement_id,
        requirement_kind,
        workload_scope,
        official_doc_url,
        expected_evidence_ref,
        source_plan_ref: "crates/tenant-rbac-listener-gateway/src/lib.rs::tenant_rbac_listener_gateway_plan",
        tenant_namespace: TENANT_NAMESPACE,
        required_route_count: EXPECTED_ROUTE_COUNT,
        requires_cluster_observation: !matches!(
            requirement_kind,
            ListenerRuntimeEvidenceRequirementKind::AccessLogTraceCorrelated
                | ListenerRuntimeEvidenceRequirementKind::ListenerDeploymentAuditRecorded
        ),
        requires_gateway_observation,
        requires_runtime_probe,
        requires_security_enforcement,
        runtime_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_requirement(
    requirement: &ListenerRuntimeEvidenceRequirement,
) -> Result<(), TenantRbacListenerRuntimeEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        TenantRbacListenerRuntimeEvidenceError::InvalidRequirementId,
    )?;
    validate_workload_scope(requirement.workload_scope)?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/listener-runtime/tenant-rbac/",
        TenantRbacListenerRuntimeEvidenceError::InvalidExpectedEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.source_plan_ref,
        "crates/tenant-rbac-listener-gateway/",
        TenantRbacListenerRuntimeEvidenceError::InvalidSourcePlanRef,
    )?;
    if requirement.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if requirement.required_route_count != EXPECTED_ROUTE_COUNT {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidListenerGatewayRouteCount);
    }
    if matches!(
        requirement.requirement_kind,
        ListenerRuntimeEvidenceRequirementKind::ClusterIpServiceObserved
            | ListenerRuntimeEvidenceRequirementKind::GatewayHttpRouteAccepted
            | ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound
            | ListenerRuntimeEvidenceRequirementKind::ReadinessProbeSucceeded
            | ListenerRuntimeEvidenceRequirementKind::LivenessProbeSucceeded
            | ListenerRuntimeEvidenceRequirementKind::SyntheticHealthCheckSucceeded
            | ListenerRuntimeEvidenceRequirementKind::RouteAuthzEnforced
            | ListenerRuntimeEvidenceRequirementKind::NetworkPolicyIngressDenied
            | ListenerRuntimeEvidenceRequirementKind::EndpointSliceReadyObserved
            | ListenerRuntimeEvidenceRequirementKind::GracefulShutdownObserved
    ) {
        require_control(
            requirement.requires_cluster_observation,
            "requirement_requires_cluster_observation",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        ListenerRuntimeEvidenceRequirementKind::GatewayHttpRouteAccepted
            | ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound
    ) {
        require_control(
            requirement.requires_gateway_observation,
            "requirement_requires_gateway_observation",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        ListenerRuntimeEvidenceRequirementKind::ReadinessProbeSucceeded
            | ListenerRuntimeEvidenceRequirementKind::LivenessProbeSucceeded
            | ListenerRuntimeEvidenceRequirementKind::SyntheticHealthCheckSucceeded
            | ListenerRuntimeEvidenceRequirementKind::GracefulShutdownObserved
    ) {
        require_control(
            requirement.requires_runtime_probe,
            "requirement_requires_runtime_probe",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound
            | ListenerRuntimeEvidenceRequirementKind::RouteAuthzEnforced
            | ListenerRuntimeEvidenceRequirementKind::NetworkPolicyIngressDenied
    ) {
        require_control(
            requirement.requires_security_enforcement,
            "requirement_requires_security_enforcement",
        )?;
    }
    if requirement.runtime_evidence_attached {
        return Err(TenantRbacListenerRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn required_requirement_kinds() -> [ListenerRuntimeEvidenceRequirementKind; 12] {
    [
        ListenerRuntimeEvidenceRequirementKind::ClusterIpServiceObserved,
        ListenerRuntimeEvidenceRequirementKind::GatewayHttpRouteAccepted,
        ListenerRuntimeEvidenceRequirementKind::TlsCertificateBound,
        ListenerRuntimeEvidenceRequirementKind::ReadinessProbeSucceeded,
        ListenerRuntimeEvidenceRequirementKind::LivenessProbeSucceeded,
        ListenerRuntimeEvidenceRequirementKind::SyntheticHealthCheckSucceeded,
        ListenerRuntimeEvidenceRequirementKind::RouteAuthzEnforced,
        ListenerRuntimeEvidenceRequirementKind::NetworkPolicyIngressDenied,
        ListenerRuntimeEvidenceRequirementKind::EndpointSliceReadyObserved,
        ListenerRuntimeEvidenceRequirementKind::GracefulShutdownObserved,
        ListenerRuntimeEvidenceRequirementKind::AccessLogTraceCorrelated,
        ListenerRuntimeEvidenceRequirementKind::ListenerDeploymentAuditRecorded,
    ]
}

fn validate_slug(
    value: &str,
    error: TenantRbacListenerRuntimeEvidenceError,
) -> Result<(), TenantRbacListenerRuntimeEvidenceError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_workload_scope(value: &str) -> Result<(), TenantRbacListenerRuntimeEvidenceError> {
    if value.is_empty() || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidWorkloadScope);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), TenantRbacListenerRuntimeEvidenceError> {
    let allowed = [
        "https://kubernetes.io/docs/concepts/services-networking/service/",
        "https://gateway-api.sigs.k8s.io/concepts/api-overview/",
        "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/",
        "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
        "https://kubernetes.io/docs/concepts/services-networking/endpoint-slices/",
        "https://opentelemetry.io/docs/specs/semconv/http/http-spans/",
        "https://cloudevents.io/",
    ];
    if !allowed.contains(&url) {
        return Err(TenantRbacListenerRuntimeEvidenceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacListenerRuntimeEvidenceError,
) -> Result<(), TenantRbacListenerRuntimeEvidenceError> {
    if !value.starts_with(prefix) || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    enabled: bool,
    field: &'static str,
) -> Result<(), TenantRbacListenerRuntimeEvidenceError> {
    if enabled {
        Ok(())
    } else {
        Err(TenantRbacListenerRuntimeEvidenceError::MissingRequiredControl(field))
    }
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('~') || value.starts_with('/')
}

fn has_unsafe_text(value: &str) -> bool {
    value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("credential")
}
