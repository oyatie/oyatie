//! Tenant RBAC local runtime composition manifest.
//!
//! This crate catalogs the router-ready HR, Payroll, Accounting, and TenantRbac
//! Platform runtime adapter route manifests for later cloud listener composition.
//! It does not start a listener, perform authentication, persist data, execute
//! Workflow, call downstream services over the network, emit runtime audit-chain
//! events, or deploy cloud infrastructure.
//! ADR-0083 Tier 3: tests legitimately use assertion helpers under the
//! `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacLocalRuntimeRoute {
    pub service: &'static str,             // data_class: PUBLIC
    pub method: &'static str,              // data_class: PUBLIC
    pub path: &'static str,                // data_class: PUBLIC
    pub operation_id: &'static str,        // data_class: PUBLIC
    pub request_data_class: &'static str,  // data_class: INTERNAL_ONLY
    pub response_data_class: &'static str, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacLocalRuntimeComposition {
    pub name: &'static str,                          // data_class: PUBLIC
    pub routes: Vec<TenantRbacLocalRuntimeRoute>,    // data_class: PUBLIC
    pub schema_version: u32,                         // data_class: PUBLIC
    pub deployed_listener_attached: bool,            // data_class: PUBLIC
    pub authentication_runtime_attached: bool,       // data_class: PUBLIC
    pub downstream_network_calls_attached: bool,     // data_class: PUBLIC
    pub storage_integration_attached: bool,          // data_class: PUBLIC
    pub workflow_execution_attached: bool,           // data_class: PUBLIC
    pub cloud_deployment_attached: bool,             // data_class: PUBLIC
    pub runtime_audit_chain_emission_attached: bool, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacLocalRuntimeCompositionError {
    DuplicateMethodPath(String),
}

pub fn tenant_rbac_local_runtime_composition() -> TenantRbacLocalRuntimeComposition {
    let mut routes = Vec::new();

    routes.extend(
        hr_employment_infrastructure::hr_runtime_routes()
            .into_iter()
            .map(|route| TenantRbacLocalRuntimeRoute {
                service: "hr",
                method: route.method,
                path: route.path,
                operation_id: route.operation_id,
                request_data_class: route.request_data_class,
                response_data_class: route.response_data_class,
            }),
    );
    routes.extend(
        oya_payroll_run_infrastructure::payroll_runtime_routes()
            .into_iter()
            .map(|route| TenantRbacLocalRuntimeRoute {
                service: "payroll",
                method: route.method,
                path: route.path,
                operation_id: route.operation_id,
                request_data_class: route.request_data_class,
                response_data_class: route.response_data_class,
            }),
    );
    routes.extend(
        billing_accounting_http_adapter::accounting_runtime_routes()
            .into_iter()
            .map(|route| TenantRbacLocalRuntimeRoute {
                service: "accounting",
                method: route.method,
                path: route.path,
                operation_id: route.operation_id,
                request_data_class: route.request_data_class,
                response_data_class: route.response_data_class,
            }),
    );
    routes.extend(
        iam_tenant_rbac_app::tenant_rbac_runtime_routes()
            .into_iter()
            .map(|route| TenantRbacLocalRuntimeRoute {
                service: "tenant-rbac",
                method: route.method,
                path: route.path,
                operation_id: route.operation_id,
                request_data_class: route.request_data_class,
                response_data_class: route.response_data_class,
            }),
    );

    TenantRbacLocalRuntimeComposition {
        name: "tenant-rbac-local-runtime-composition",
        routes,
        schema_version: 1,
        deployed_listener_attached: false,
        authentication_runtime_attached: false,
        downstream_network_calls_attached: false,
        storage_integration_attached: false,
        workflow_execution_attached: false,
        cloud_deployment_attached: false,
        runtime_audit_chain_emission_attached: false,
    }
}

pub fn validate_unique_method_paths(
    composition: &TenantRbacLocalRuntimeComposition,
) -> Result<(), TenantRbacLocalRuntimeCompositionError> {
    let mut seen = BTreeSet::new();
    for route in &composition.routes {
        let key = format!("{} {}", route.method, route.path);
        if !seen.insert(key.clone()) {
            return Err(TenantRbacLocalRuntimeCompositionError::DuplicateMethodPath(
                key,
            ));
        }
    }
    Ok(())
}
