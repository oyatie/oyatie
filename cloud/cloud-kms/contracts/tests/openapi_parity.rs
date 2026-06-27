// ADR-0083 Tier 3: integration tests use panic-style assertions to pin contract
// invariants under the test exemption.
#![allow(clippy::panic)]

use secrets_kms_api::{CLOUD_KMS_DECRYPT_SURFACE, CLOUD_KMS_ENCRYPT_SURFACE};

const OPENAPI: &str = include_str!("../openapi/cloud/cloud-kms-v1.yaml");

struct OperationExpectation {
    path: &'static str,
    method: &'static str,
    surface: &'static str,
    request_schema: &'static str,
    response_schema: &'static str,
}

#[test]
fn service_local_openapi_tracks_cloud_kms_crypto_surfaces() {
    assert!(
        !OPENAPI.contains("paths: {}"),
        "cloud-kms service-local OpenAPI must not remain an empty path scaffold"
    );

    for operation in [
        OperationExpectation {
            path: "/keys/{key_id}:encrypt",
            method: "post",
            surface: CLOUD_KMS_ENCRYPT_SURFACE,
            request_schema: "KmsEncryptRequest",
            response_schema: "KmsCryptoReceipt",
        },
        OperationExpectation {
            path: "/keys/{key_id}:decrypt",
            method: "post",
            surface: CLOUD_KMS_DECRYPT_SURFACE,
            request_schema: "KmsDecryptRequest",
            response_schema: "KmsCryptoReceipt",
        },
    ] {
        assert_operation(operation);
    }

    for forbidden_text in [
        "allowed_surfaces",
        "X-Principal-Tenant-Id",
        "X-Principal-Id",
        "X-Authorization-Decision-Id",
        "X-Authorization-Tenant-Id",
        "X-Authorization-Principal-Id",
        "X-Authorization-Surfaces",
    ] {
        assert!(
            !OPENAPI.contains(forbidden_text),
            "public OpenAPI must not accept caller-supplied authority field {forbidden_text}"
        );
    }

    assert_schema_contains("KmsEncryptRequest", "plaintext_ref:");
    assert_schema_contains("KmsEncryptRequest", "ciphertext_ref:");
    assert_schema_contains("KmsDecryptRequest", "ciphertext_ref:");
    assert_schema_contains("KmsCryptoReceipt", "authorization_decision_ref:");
    assert_schema_contains("KmsCryptoReceipt", "metadata_only:");
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
        "authority: verified_principal_plus_ingress_pdp_cedar_decision",
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
        "caller_supplied_authority_fields: forbidden",
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        "metadata_only_receipt: true",
        operation.path,
        operation.method,
    );

    for public_parameter in [
        "RequestId",
        "TenantId",
        "BoundaryRegionId",
        "BoundaryCellId",
        "OyatieVersion",
        "IdempotencyKey",
        "KeyId",
    ] {
        assert_contains(
            section,
            &format!("$ref: '#/components/parameters/{public_parameter}'"),
            operation.path,
            operation.method,
        );
    }

    assert_contains(
        section,
        &format!("$ref: '#/components/schemas/{}'", operation.request_schema),
        operation.path,
        operation.method,
    );
    assert_contains(
        section,
        &format!("$ref: '#/components/schemas/{}'", operation.response_schema),
        operation.path,
        operation.method,
    );

    for status in ["'200'", "'400'", "'401'", "'403'", "'404'", "'409'", "'422'"] {
        assert_contains(
            section,
            &format!("        {status}:"),
            operation.path,
            operation.method,
        );
    }
}

fn assert_contains(haystack: &str, needle: &str, path: &str, method: &str) {
    assert!(
        haystack.contains(needle),
        "OpenAPI operation {method} {path} must contain {needle}"
    );
}

fn assert_schema_contains(schema: &str, needle: &str) {
    let section = schema_section(schema);
    assert!(
        section.contains(needle),
        "OpenAPI schema {schema} must contain {needle}"
    );
}

fn operation_section(path: &str, method: &str) -> &'static str {
    let path_marker = format!("  {path}:");
    let start = OPENAPI
        .find(&path_marker)
        .unwrap_or_else(|| panic!("missing OpenAPI path {path}"));
    let after_path = &OPENAPI[start + path_marker.len()..];
    let method_marker = format!("\n    {method}:");
    let method_offset = after_path
        .find(&method_marker)
        .unwrap_or_else(|| panic!("missing method {method} for OpenAPI path {path}"));
    let method_start = start + path_marker.len() + method_offset;
    let after_method = &OPENAPI[method_start + method_marker.len()..];
    let end = after_method
        .find("\n  /")
        .map(|offset| method_start + method_marker.len() + offset)
        .unwrap_or_else(|| OPENAPI.find("\ncomponents:").unwrap_or(OPENAPI.len()));
    &OPENAPI[method_start..end]
}

fn schema_section(schema: &str) -> &'static str {
    let marker = format!("    {schema}:");
    let start = OPENAPI
        .find(&marker)
        .unwrap_or_else(|| panic!("missing OpenAPI schema {schema}"));
    let after_schema = &OPENAPI[start + marker.len()..];
    let end = after_schema
        .find("\n    ")
        .map(|offset| start + marker.len() + offset)
        .unwrap_or(OPENAPI.len());
    &OPENAPI[start..end]
}
