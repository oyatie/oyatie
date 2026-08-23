//! Tenant RBAC listener/gateway foundation.
//!
//! This control-plane crate composes the local runtime route catalog, the
//! route-level auth policy, and the cloud deployment manifest into a review-only
//! Kubernetes Service + Gateway API HTTPRoute plan. It deliberately does not
//! start a listener, bind a socket, apply Kubernetes resources, attach a Gateway
//! controller, provision a load balancer, terminate TLS, enforce runtime auth
//! middleware, emit runtime audit-chain events, or claim cloud deployment/SLO
//! evidence.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use iam_tenant_rbac_auth_app::{
    TenantRbacAuthRoutePolicy, tenant_rbac_auth_runtime_policy,
    validate_tenant_rbac_auth_runtime_policy,
};
use iam_tenant_rbac_deployment_manifest::{
    tenant_rbac_deployment_manifest, validate_cloud_deployment_manifest,
};
use iam_tenant_rbac_local_runtime_composition::{
    TenantRbacLocalRuntimeRoute, tenant_rbac_local_runtime_composition,
    validate_unique_method_paths,
};

const SCHEMA_VERSION: u32 = 1;
const EXPECTED_ROUTE_COUNT: usize = 19;
const REQUEST_TIMEOUT_MS: u32 = 30_000;
const BACKEND_REQUEST_TIMEOUT_MS: u32 = 25_000;
const GRACEFUL_SHUTDOWN_SECONDS: u16 = 30;
const MAX_BODY_BYTES: u32 = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacGatewayRoute {
    pub service: &'static str,                 // data_class: PUBLIC
    pub method: &'static str,                  // data_class: PUBLIC
    pub path: &'static str,                    // data_class: PUBLIC
    pub operation_id: &'static str,            // data_class: PUBLIC
    pub required_scope: &'static str,          // data_class: INTERNAL_ONLY
    pub request_data_class: &'static str,      // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str,     // data_class: INTERNAL_ONLY
    pub sensitive_data: bool,                  // data_class: INTERNAL_ONLY
    pub mfa_required: bool,                    // data_class: INTERNAL_ONLY
    pub backend_service_name: &'static str,    // data_class: INTERNAL_ONLY
    pub backend_port: u16,                     // data_class: PUBLIC
    pub gateway_path_match_type: &'static str, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacListenerGatewayPlan {
    pub plan_name: &'static str,                        // data_class: PUBLIC
    pub namespace: &'static str,                        // data_class: INTERNAL_ONLY
    pub service_name: &'static str,                     // data_class: PUBLIC
    pub kubernetes_service_type: &'static str,          // data_class: PUBLIC
    pub gateway_api_version: &'static str,              // data_class: PUBLIC
    pub gateway_route_kind: &'static str,               // data_class: PUBLIC
    pub gateway_class_ref: &'static str,                // data_class: INTERNAL_ONLY
    pub gateway_name: &'static str,                     // data_class: INTERNAL_ONLY
    pub http_route_name: &'static str,                  // data_class: INTERNAL_ONLY
    pub external_hostname: &'static str,                // data_class: INTERNAL_ONLY
    pub gateway_listener_port: u16,                     // data_class: PUBLIC
    pub service_port: u16,                              // data_class: PUBLIC
    pub backend_port: u16,                              // data_class: PUBLIC
    pub application_health_route_path: &'static str,    // data_class: PUBLIC
    pub platform_readiness_probe_path: &'static str,    // data_class: PUBLIC
    pub platform_liveness_probe_path: &'static str,     // data_class: PUBLIC
    pub request_timeout_ms: u32,                        // data_class: PUBLIC
    pub backend_request_timeout_ms: u32,                // data_class: PUBLIC
    pub graceful_shutdown_seconds: u16,                 // data_class: PUBLIC
    pub max_body_bytes: u32,                            // data_class: PUBLIC
    pub route_count: usize,                             // data_class: PUBLIC
    pub auth_policy_route_count: usize,                 // data_class: PUBLIC
    pub routes: Vec<TenantRbacGatewayRoute>,            // data_class: PUBLIC
    pub gateway_api_required: bool,                     // data_class: PUBLIC
    pub ingress_tls_required: bool,                     // data_class: PUBLIC
    pub network_policy_required: bool,                  // data_class: PUBLIC
    pub authz_required: bool,                           // data_class: PUBLIC
    pub deny_by_default_required: bool,                 // data_class: PUBLIC
    pub direct_public_node_port_allowed: bool,          // data_class: PUBLIC
    pub direct_public_load_balancer_allowed: bool,      // data_class: PUBLIC
    pub deployed_listener_attached: bool,               // data_class: INTERNAL_ONLY
    pub gateway_controller_attached: bool,              // data_class: INTERNAL_ONLY
    pub load_balancer_attached: bool,                   // data_class: INTERNAL_ONLY
    pub tls_certificate_attached: bool,                 // data_class: INTERNAL_ONLY
    pub runtime_auth_middleware_attached: bool,         // data_class: INTERNAL_ONLY
    pub cloud_deployment_evidence_attached: bool,       // data_class: INTERNAL_ONLY
    pub production_slo_evidence_attached: bool,         // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,                            // data_class: PUBLIC
    pub rendered_kubernetes_manifest_review_only: bool, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacListenerGatewayError {
    InvalidPlan,
    InvalidNamespace,
    InvalidServiceName,
    InvalidServiceType,
    InvalidGatewayApi,
    InvalidHostname,
    InvalidPort,
    InvalidProbePath,
    InvalidTimeout,
    InvalidRoute,
    DuplicateRoute(String),
    MissingRouteCoverage(String),
    MissingAuthPolicyCoverage(String),
    PublicIngressOverclaim,
    SecurityControlMissing,
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_listener_gateway_plan() -> TenantRbacListenerGatewayPlan {
    let composition = tenant_rbac_local_runtime_composition();
    let auth_policy = tenant_rbac_auth_runtime_policy();
    let deployment_manifest = tenant_rbac_deployment_manifest();
    let auth_by_route = auth_route_policy_map(&auth_policy.route_policies);
    let routes = composition
        .routes
        .iter()
        .map(|route| {
            let key = method_path_key(route.method, route.path);
            let auth = auth_by_route.get(&key);
            TenantRbacGatewayRoute {
                service: route.service,
                method: route.method,
                path: route.path,
                operation_id: route.operation_id,
                required_scope: auth
                    .map(|policy| policy.required_scope)
                    .unwrap_or("missing:route-scope"),
                request_data_class: route.request_data_class,
                response_data_class: route.response_data_class,
                sensitive_data: auth.map(|policy| policy.sensitive_data).unwrap_or(false),
                mfa_required: auth.map(|policy| policy.mfa_required).unwrap_or(false),
                backend_service_name: deployment_manifest.service_name,
                backend_port: deployment_manifest.container_port,
                gateway_path_match_type: "Exact",
            }
        })
        .collect::<Vec<_>>();

    TenantRbacListenerGatewayPlan {
        plan_name: "tenant-rbac-listener-gateway-plan",
        namespace: deployment_manifest.namespace,
        service_name: deployment_manifest.service_name,
        kubernetes_service_type: "ClusterIP",
        gateway_api_version: "gateway.networking.k8s.io/v1",
        gateway_route_kind: "HTTPRoute",
        gateway_class_ref: "tenant-rbac-gateway-class",
        gateway_name: "tenant-rbac-gateway",
        http_route_name: "tenant-rbac-routes",
        external_hostname: "tenant-rbac.dev.oyatie.internal",
        gateway_listener_port: 443,
        service_port: deployment_manifest.container_port,
        backend_port: deployment_manifest.container_port,
        application_health_route_path: "/tenant-rbac/v1/healthz",
        platform_readiness_probe_path: deployment_manifest.readiness_probe_path,
        platform_liveness_probe_path: deployment_manifest.liveness_probe_path,
        request_timeout_ms: REQUEST_TIMEOUT_MS,
        backend_request_timeout_ms: BACKEND_REQUEST_TIMEOUT_MS,
        graceful_shutdown_seconds: GRACEFUL_SHUTDOWN_SECONDS,
        max_body_bytes: MAX_BODY_BYTES,
        route_count: routes.len(),
        auth_policy_route_count: auth_policy.route_policies.len(),
        routes,
        gateway_api_required: true,
        ingress_tls_required: true,
        network_policy_required: true,
        authz_required: true,
        deny_by_default_required: true,
        direct_public_node_port_allowed: false,
        direct_public_load_balancer_allowed: false,
        deployed_listener_attached: false,
        gateway_controller_attached: false,
        load_balancer_attached: false,
        tls_certificate_attached: false,
        runtime_auth_middleware_attached: false,
        cloud_deployment_evidence_attached: false,
        production_slo_evidence_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
        rendered_kubernetes_manifest_review_only: true,
    }
}

pub fn validate_tenant_rbac_listener_gateway_plan(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<(), TenantRbacListenerGatewayError> {
    validate_slug(plan.plan_name, TenantRbacListenerGatewayError::InvalidPlan)?;
    validate_namespace(plan.namespace)?;
    validate_slug(
        plan.service_name,
        TenantRbacListenerGatewayError::InvalidServiceName,
    )?;
    validate_gateway_refs(plan)?;
    validate_hostname(plan.external_hostname)?;
    validate_ports(plan)?;
    validate_probe_path(plan.application_health_route_path)?;
    validate_probe_path(plan.platform_readiness_probe_path)?;
    validate_probe_path(plan.platform_liveness_probe_path)?;
    validate_timeouts(plan)?;
    validate_security_controls(plan)?;
    validate_nonclaims(plan)?;
    validate_route_coverage(plan)?;
    Ok(())
}

pub fn render_tenant_rbac_listener_gateway_review_manifest(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<String, TenantRbacListenerGatewayError> {
    validate_tenant_rbac_listener_gateway_plan(plan)?;

    let mut manifest = format!(
        "# review_only: true\napiVersion: v1\nkind: Service\nmetadata:\n  name: {}\n  namespace: {}\nspec:\n  type: {}\n  ports:\n    - name: http\n      port: {}\n      targetPort: {}\n  selector:\n    app.kubernetes.io/name: {}\n---\napiVersion: {}\nkind: {}\nmetadata:\n  name: {}\n  namespace: {}\nspec:\n  parentRefs:\n    - name: {}\n      sectionName: https\n  hostnames:\n    - {}\n  rules:\n",
        plan.service_name,
        plan.namespace,
        plan.kubernetes_service_type,
        plan.service_port,
        plan.backend_port,
        plan.service_name,
        plan.gateway_api_version,
        plan.gateway_route_kind,
        plan.http_route_name,
        plan.namespace,
        plan.gateway_name,
        plan.external_hostname,
    );

    for (index, route) in plan.routes.iter().enumerate() {
        manifest.push_str(&format!(
            "    - name: {}\n      matches:\n        - method: {}\n          path:\n            type: {}\n            value: {}\n      backendRefs:\n        - name: {}\n          port: {}\n      timeouts:\n        request: {}ms\n        backendRequest: {}ms\n",
            gateway_rule_name(index),
            route.method,
            route.gateway_path_match_type,
            route.path,
            route.backend_service_name,
            route.backend_port,
            plan.request_timeout_ms,
            plan.backend_request_timeout_ms,
        ));
    }

    Ok(manifest)
}

fn validate_gateway_refs(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<(), TenantRbacListenerGatewayError> {
    if plan.kubernetes_service_type != "ClusterIP" {
        return Err(TenantRbacListenerGatewayError::InvalidServiceType);
    }
    if plan.gateway_api_version != "gateway.networking.k8s.io/v1"
        || plan.gateway_route_kind != "HTTPRoute"
    {
        return Err(TenantRbacListenerGatewayError::InvalidGatewayApi);
    }
    validate_slug(
        plan.gateway_class_ref,
        TenantRbacListenerGatewayError::InvalidGatewayApi,
    )?;
    validate_slug(
        plan.gateway_name,
        TenantRbacListenerGatewayError::InvalidGatewayApi,
    )?;
    validate_slug(
        plan.http_route_name,
        TenantRbacListenerGatewayError::InvalidGatewayApi,
    )?;
    Ok(())
}

fn validate_ports(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<(), TenantRbacListenerGatewayError> {
    let deployment_manifest = tenant_rbac_deployment_manifest();
    validate_cloud_deployment_manifest(&deployment_manifest)
        .map_err(|_| TenantRbacListenerGatewayError::InvalidPlan)?;
    if plan.gateway_listener_port != 443
        || plan.service_port != deployment_manifest.container_port
        || plan.backend_port != deployment_manifest.container_port
        || plan.backend_port < 1024
    {
        return Err(TenantRbacListenerGatewayError::InvalidPort);
    }
    Ok(())
}

fn validate_timeouts(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<(), TenantRbacListenerGatewayError> {
    if plan.request_timeout_ms == 0
        || plan.backend_request_timeout_ms == 0
        || plan.backend_request_timeout_ms > plan.request_timeout_ms
        || plan.request_timeout_ms > 120_000
        || plan.graceful_shutdown_seconds == 0
        || plan.graceful_shutdown_seconds > 120
        || plan.max_body_bytes == 0
        || plan.max_body_bytes > 1024 * 1024
    {
        return Err(TenantRbacListenerGatewayError::InvalidTimeout);
    }
    Ok(())
}

fn validate_security_controls(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<(), TenantRbacListenerGatewayError> {
    if plan.direct_public_node_port_allowed || plan.direct_public_load_balancer_allowed {
        return Err(TenantRbacListenerGatewayError::PublicIngressOverclaim);
    }
    if !plan.gateway_api_required
        || !plan.ingress_tls_required
        || !plan.network_policy_required
        || !plan.authz_required
        || !plan.deny_by_default_required
        || !plan.rendered_kubernetes_manifest_review_only
    {
        return Err(TenantRbacListenerGatewayError::SecurityControlMissing);
    }
    Ok(())
}

fn validate_nonclaims(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<(), TenantRbacListenerGatewayError> {
    if plan.deployed_listener_attached
        || plan.gateway_controller_attached
        || plan.load_balancer_attached
        || plan.tls_certificate_attached
        || plan.runtime_auth_middleware_attached
        || plan.cloud_deployment_evidence_attached
        || plan.production_slo_evidence_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacListenerGatewayError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_route_coverage(
    plan: &TenantRbacListenerGatewayPlan,
) -> Result<(), TenantRbacListenerGatewayError> {
    let composition = tenant_rbac_local_runtime_composition();
    validate_unique_method_paths(&composition)
        .map_err(|_| TenantRbacListenerGatewayError::InvalidRoute)?;
    let auth_policy = tenant_rbac_auth_runtime_policy();
    validate_tenant_rbac_auth_runtime_policy(&auth_policy)
        .map_err(|_| TenantRbacListenerGatewayError::InvalidRoute)?;

    if plan.schema_version != SCHEMA_VERSION
        || plan.route_count != plan.routes.len()
        || plan.route_count != EXPECTED_ROUTE_COUNT
        || composition.routes.len() != EXPECTED_ROUTE_COUNT
        || plan.auth_policy_route_count != auth_policy.route_policies.len()
        || plan.auth_policy_route_count != EXPECTED_ROUTE_COUNT
    {
        return Err(TenantRbacListenerGatewayError::MissingRouteCoverage(
            "route-count".to_string(),
        ));
    }

    let local_by_route = local_route_map(&composition.routes);
    let auth_by_route = auth_route_policy_map(&auth_policy.route_policies);
    let mut seen = BTreeSet::new();
    for route in &plan.routes {
        validate_gateway_route(route, plan.backend_port)?;
        let key = method_path_key(route.method, route.path);
        if !seen.insert(key.clone()) {
            return Err(TenantRbacListenerGatewayError::DuplicateRoute(key));
        }
        let Some(local_route) = local_by_route.get(&key) else {
            return Err(TenantRbacListenerGatewayError::MissingRouteCoverage(key));
        };
        if route.service != local_route.service
            || route.operation_id != local_route.operation_id
            || route.request_data_class != local_route.request_data_class
            || route.response_data_class != local_route.response_data_class
        {
            return Err(TenantRbacListenerGatewayError::MissingRouteCoverage(key));
        }
        let Some(auth_route) = auth_by_route.get(&key) else {
            return Err(TenantRbacListenerGatewayError::MissingAuthPolicyCoverage(
                key,
            ));
        };
        if route.required_scope != auth_route.required_scope
            || route.sensitive_data != auth_route.sensitive_data
            || route.mfa_required != auth_route.mfa_required
        {
            return Err(TenantRbacListenerGatewayError::MissingAuthPolicyCoverage(
                key,
            ));
        }
    }

    for key in local_by_route.keys() {
        if !seen.contains(key) {
            return Err(TenantRbacListenerGatewayError::MissingRouteCoverage(
                key.clone(),
            ));
        }
    }
    for key in auth_by_route.keys() {
        if !seen.contains(key) {
            return Err(TenantRbacListenerGatewayError::MissingAuthPolicyCoverage(
                key.clone(),
            ));
        }
    }
    Ok(())
}

fn validate_gateway_route(
    route: &TenantRbacGatewayRoute,
    backend_port: u16,
) -> Result<(), TenantRbacListenerGatewayError> {
    validate_slug(route.service, TenantRbacListenerGatewayError::InvalidRoute)?;
    if !matches!(route.method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE")
        || route.path.len() < 2
        || !route.path.starts_with('/')
        || has_unsafe_text(route.path)
        || has_path_traversal(route.path)
        || has_credential_shape(route.path)
        || !valid_operation_id(route.operation_id)
        || !valid_scope(route.required_scope)
        || !valid_data_class(route.request_data_class)
        || !valid_data_class(route.response_data_class)
        || route.backend_service_name != "tenant-rbac"
        || route.backend_port != backend_port
        || route.gateway_path_match_type != "Exact"
    {
        return Err(TenantRbacListenerGatewayError::InvalidRoute);
    }
    Ok(())
}

fn local_route_map(
    routes: &[TenantRbacLocalRuntimeRoute],
) -> BTreeMap<String, &TenantRbacLocalRuntimeRoute> {
    routes
        .iter()
        .map(|route| (method_path_key(route.method, route.path), route))
        .collect()
}

fn auth_route_policy_map(
    policies: &[TenantRbacAuthRoutePolicy],
) -> BTreeMap<String, &TenantRbacAuthRoutePolicy> {
    policies
        .iter()
        .map(|policy| (method_path_key(policy.method, policy.path), policy))
        .collect()
}

fn method_path_key(method: &str, path: &str) -> String {
    format!("{method} {path}")
}

fn validate_namespace(value: &str) -> Result<(), TenantRbacListenerGatewayError> {
    validate_slug(value, TenantRbacListenerGatewayError::InvalidNamespace)?;
    if !value.starts_with("oyatie-") || matches!(value, "default" | "kube-system" | "kube-public") {
        return Err(TenantRbacListenerGatewayError::InvalidNamespace);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: TenantRbacListenerGatewayError,
) -> Result<(), TenantRbacListenerGatewayError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(error);
    }
    Ok(())
}

fn validate_probe_path(value: &str) -> Result<(), TenantRbacListenerGatewayError> {
    if value.len() < 2
        || !value.starts_with('/')
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(TenantRbacListenerGatewayError::InvalidProbePath);
    }
    Ok(())
}

fn validate_hostname(value: &str) -> Result<(), TenantRbacListenerGatewayError> {
    if value.len() < 4
        || value.len() > 253
        || !value.contains('.')
        || value.contains(':')
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || looks_like_ipv4_literal(value)
        || value.starts_with('.')
        || value.ends_with('.')
        || value.split('.').any(str::is_empty)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '.')
    {
        return Err(TenantRbacListenerGatewayError::InvalidHostname);
    }
    Ok(())
}

fn gateway_rule_name(index: usize) -> String {
    format!("tenant-rbac-route-{index:02}")
}

fn looks_like_ipv4_literal(value: &str) -> bool {
    let labels = value.split('.').collect::<Vec<_>>();
    labels.len() == 4
        && labels.iter().all(|label| {
            !label.is_empty()
                && label.len() <= 3
                && label.chars().all(|ch| ch.is_ascii_digit())
                && label.parse::<u8>().is_ok()
        })
}

fn valid_operation_id(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_alphanumeric())
}

fn valid_scope(value: &str) -> bool {
    !value.is_empty()
        && value.contains(':')
        && !has_unsafe_text(value)
        && !has_path_traversal(value)
        && !has_credential_shape(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == ':' || ch == '-')
}

fn valid_data_class(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_' || ch == '+')
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('\\') || value.contains("//")
}

fn has_credential_shape(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("api_key")
}
