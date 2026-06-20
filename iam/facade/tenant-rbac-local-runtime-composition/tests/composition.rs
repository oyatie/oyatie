#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_tenant_rbac_local_runtime_composition::{
    tenant_rbac_local_runtime_composition, validate_unique_method_paths,
};

#[test]
fn tenant_rbac_local_runtime_composition_catalogs_child_routes_without_listener_claim() {
    let composition = tenant_rbac_local_runtime_composition();

    assert_eq!(composition.name, "tenant-rbac-local-runtime-composition");
    assert_eq!(composition.schema_version, 1);
    assert_eq!(composition.routes.len(), 19);
    assert!(!composition.deployed_listener_attached);
    assert!(!composition.authentication_runtime_attached);
    assert!(!composition.downstream_network_calls_attached);
    assert!(!composition.storage_integration_attached);
    assert!(!composition.workflow_execution_attached);
    assert!(!composition.cloud_deployment_attached);
    assert!(!composition.runtime_audit_chain_emission_attached);
    validate_unique_method_paths(&composition).expect("route method/path uniqueness");

    assert!(composition.routes.iter().any(|route| {
        route.service == "hr" && route.method == "POST" && route.path == "/hr/v1/employees"
    }));
    assert!(composition.routes.iter().any(|route| {
        route.service == "payroll"
            && route.method == "POST"
            && route.path == "/payroll/v1/trial-closes"
    }));
    assert!(composition.routes.iter().any(|route| {
        route.service == "accounting"
            && route.method == "POST"
            && route.path == "/accounting/v1/journals"
    }));
    assert!(composition.routes.iter().any(|route| {
        route.service == "tenant-rbac"
            && route.method == "POST"
            && route.path == "/tenant-rbac/v1/policy-admissions"
    }));
}

#[test]
fn tenant_rbac_local_runtime_composition_rejects_duplicate_method_paths() {
    let mut composition = tenant_rbac_local_runtime_composition();
    composition.routes.push(composition.routes[0].clone());

    let error = validate_unique_method_paths(&composition).expect_err("duplicate route rejected");
    assert!(format!("{error:?}").contains(composition.routes[0].path));
}
