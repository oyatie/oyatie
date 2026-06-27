// ADR-0083 Tier 3: integration tests use panic-style assertions to pin contract
// invariants under the test exemption.
#![allow(clippy::panic)]

use iam_cloud_api::{
    CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE, CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE,
    CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE, CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE,
    CLOUD_IAM_ROLE_CREATE_SURFACE, CLOUD_IAM_STS_TOKEN_SURFACE,
};

const OPENAPI: &str = include_str!("../openapi/cloud/cloud-iam-v1.yaml");

struct OperationExpectation {
    path: &'static str,
    method: &'static str,
    surface: &'static str,
    success_status: &'static str,
    idempotent: bool,
    request_schema: Option<&'static str>,
}

#[test]
fn service_local_openapi_tracks_cloud_iam_api_surfaces() {
    assert!(
        !OPENAPI.contains("paths: {}"),
        "cloud-iam service-local OpenAPI must not remain an empty path scaffold"
    );

    assert!(
        OPENAPI.lines().any(|line| line.contains("caller-supplied internal authorization headers")),
        "cloud-iam OpenAPI must keep the ingress-owned policy-engine header-strip boundary explicit"
    );

    for operation in [
        OperationExpectation {
            path: "/identity-providers",
            method: "get",
            surface: CLOUD_IAM_IDENTITY_PROVIDER_LIST_SURFACE,
            success_status: "'200'",
            idempotent: false,
            request_schema: None,
        },
        OperationExpectation {
            path: "/identity-providers/{identity_provider_id}",
            method: "post",
            surface: CLOUD_IAM_IDENTITY_PROVIDER_CREATE_SURFACE,
            success_status: "'201'",
            idempotent: true,
            request_schema: Some("IdentityProviderCreateRequest"),
        },
        OperationExpectation {
            path: "/identity-providers/{identity_provider_id}",
            method: "put",
            surface: CLOUD_IAM_IDENTITY_PROVIDER_UPDATE_SURFACE,
            success_status: "'200'",
            idempotent: true,
            request_schema: Some("IdentityProviderUpdateRequest"),
        },
        OperationExpectation {
            path: "/identity-providers/{identity_provider_id}",
            method: "delete",
            surface: CLOUD_IAM_IDENTITY_PROVIDER_DELETE_SURFACE,
            success_status: "'200'",
            idempotent: true,
            request_schema: None,
        },
        OperationExpectation {
            path: "/roles/{role_id}",
            method: "post",
            surface: CLOUD_IAM_ROLE_CREATE_SURFACE,
            success_status: "'201'",
            idempotent: true,
            request_schema: Some("RoleCreateRequest"),
        },
        OperationExpectation {
            path: "/sts/tokens",
            method: "post",
            surface: CLOUD_IAM_STS_TOKEN_SURFACE,
            success_status: "'200'",
            idempotent: true,
            request_schema: Some("StsTokenRequest"),
        },
    ] {
        assert_operation(operation);
    }

    for forbidden_header in [
        "X-Principal-Tenant-Id",
        "X-Principal-Id",
        "X-Authorization-Decision-Id",
        "X-Authorization-Tenant-Id",
        "X-Authorization-Principal-Id",
        "X-Authorization-Surfaces",
    ] {
        assert!(
            !OPENAPI.contains(forbidden_header),
            "public OpenAPI must not accept caller-supplied internal header {forbidden_header}"
        );
    }

    assert_schema_not_contains("IdentityProviderCreateRequest", "created_at_epoch_seconds:");
    assert_schema_not_contains("RoleCreateRequest", "created_at_epoch_seconds:");
    assert_schema_not_contains("StsTokenRequest", "issued_at_epoch_seconds:");
    assert_schema_contains("IdentityProviderRecord", "created_at_epoch_seconds:");
    assert_schema_contains("RoleRecord", "created_at_epoch_seconds:");
    assert_schema_contains("StsSessionRecord", "issued_at_epoch_seconds:");
    assert_schema_contains("ErrorResponse", "message_localized:");
}

fn assert_operation(operation: OperationExpectation) {
    let section = operation_section(operation.path, operation.method);

    assert_contains(
        section,
        &format!("x-oyatie-surface: {}", operation.surface),
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        "x-oyatie-authorization:",
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        &format!("required_surface: {}", operation.surface),
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        "authority: ingress_pdp_cedar_decision",
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        "default_decision: deny",
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        "tenant_scope: tenant_account_project_boundary",
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        "cross_tenant_access: forbidden",
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        "caller_supplied_authorization_headers: forbidden",
        operation.path,
        operation.method,
    );

    for public_parameter in [
        "RequestId",
        "TenantId",
        "BoundaryCellId",
        "BoundaryRegionId",
        "OyatieVersion",
    ] {
        assert_contains(
            section,
            &format!("$ref: '#/components/parameters/{public_parameter}'"),
            operation.path,
            operation.method,
        );
    }

    if operation.idempotent {
        assert_contains(
            section,
            "$ref: '#/components/parameters/IdempotencyKey'",
            operation.path,
            operation.method,
        );
    } else {
        assert_not_contains(
            section,
            "$ref: '#/components/parameters/IdempotencyKey'",
            operation.path,
            operation.method,
        );
    }

    if let Some(schema) = operation.request_schema {
        assert_contains(
            section,
            &format!("$ref: '#/components/schemas/{schema}'"),
            operation.path,
            operation.method,
        );
    } else {
        assert_not_contains(section, "requestBody:", operation.path, operation.method);
    }

    for status in [operation.success_status, "'400'", "'403'"] {
        assert_contains(
            section,
            &format!("        {status}:"),
            operation.path,
            operation.method,
        );
    }
    if operation.idempotent {
        for status in ["'409'", "'422'"] {
            assert_contains(
                section,
                &format!("        {status}:"),
                operation.path,
                operation.method,
            );
        }
    }
}

fn assert_contains(haystack: &str, needle: &str, path: &str, method: &str) {
    assert!(
        haystack.contains(needle),
        "OpenAPI {method} {path} section is missing {needle}"
    );
}

fn assert_not_contains(haystack: &str, needle: &str, path: &str, method: &str) {
    assert!(
        !haystack.contains(needle),
        "OpenAPI {method} {path} section must not contain {needle}"
    );
}

fn path_section(path: &str) -> &'static str {
    let marker = format!("  {path}:");
    let start = OPENAPI
        .find(&marker)
        .unwrap_or_else(|| panic!("OpenAPI contract is missing path {path}"));
    let rest = &OPENAPI[start + marker.len()..];
    let end = rest.find("\n  /").unwrap_or(rest.len());
    &rest[..end]
}

fn operation_section(path: &str, method: &str) -> &'static str {
    let path_section = path_section(path);
    let marker = format!("\n    {method}:");
    let start = path_section
        .find(&marker)
        .unwrap_or_else(|| panic!("OpenAPI contract is missing {method} operation for {path}"))
        + marker.len();
    let rest = &path_section[start..];
    let end = ["\n    get:", "\n    post:", "\n    put:", "\n    delete:"]
        .iter()
        .filter_map(|next_method| rest.find(next_method))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

fn schema_section(schema: &str) -> &'static str {
    // ponytail: keep this dependency-free; YAML syntax is checked by the local
    // verifier, while this test pins the small structural slices that matter.
    let marker = format!("    {schema}:");
    let start = OPENAPI
        .find(&marker)
        .unwrap_or_else(|| panic!("OpenAPI contract is missing schema {schema}"))
        + marker.len();
    let rest = &OPENAPI[start..];
    &rest[..component_section_len(rest)]
}

fn component_section_len(rest: &str) -> usize {
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if offset > 0 {
            if line.starts_with("  x-") {
                return offset;
            }
            if line.starts_with("    ") && !line.starts_with("      ") {
                return offset;
            }
        }
        offset += line.len();
    }
    rest.len()
}

fn assert_schema_contains(schema: &str, needle: &str) {
    let section = schema_section(schema);
    assert!(
        section.contains(needle),
        "OpenAPI schema {schema} is missing {needle}"
    );
}

fn assert_schema_not_contains(schema: &str, needle: &str) {
    let section = schema_section(schema);
    assert!(
        !section.contains(needle),
        "OpenAPI schema {schema} must not contain {needle}"
    );
}
