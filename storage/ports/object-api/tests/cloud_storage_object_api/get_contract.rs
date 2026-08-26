use super::common::*;

#[test]
fn get_object_api_projects_authorized_object_metadata() {
    let mut catalog = catalog_with_active_bucket();
    put_fixture_object(&mut catalog);

    let response = get_cloud_storage_object_from_api(&catalog, get_request(BUCKET_ID, OBJECT_KEY))
        .expect("authorized object GET succeeds");

    assert_eq!(response.metadata.request_id, "req-storage-object-get");
    assert_eq!(response.data.bucket_id, BUCKET_ID);
    assert_eq!(response.data.key, OBJECT_KEY);
    assert_eq!(
        response.data.encryption.ciphertext_ref,
        "ct/ten_alpha/object/report"
    );
    assert_eq!(response.data.schema_version, 1);
}

#[test]
fn get_object_api_rejects_authorization_before_existence_lookup() {
    let catalog = catalog_with_active_bucket();
    let mut request = get_request(BUCKET_ID, "workspace/missing.pdf");
    request.authorization.allowed_surfaces = vec![CLOUD_STORAGE_OBJECT_PUT_SURFACE.to_string()];

    let error = get_cloud_storage_object_from_api(&catalog, request)
        .expect_err("authorization denial must win over object existence checks");

    assert_eq!(
        error,
        CloudStorageObjectApiError::AuthorizationDenied {
            surface: CLOUD_STORAGE_OBJECT_GET_SURFACE.to_string(),
        }
    );
    assert_eq!(error.object_status_code(), 403);
}

#[test]
fn get_object_api_maps_not_found_and_tenant_drift_explicitly() {
    let catalog = catalog_with_active_bucket();
    let missing = get_cloud_storage_object_from_api(
        &catalog,
        get_request(BUCKET_ID, "workspace/missing.pdf"),
    )
    .expect_err("missing object maps to not found");
    assert_eq!(missing.object_status_code(), 404);
    assert!(matches!(
        missing,
        CloudStorageObjectApiError::ObjectNotFound { .. }
    ));

    let tenant_drift = get_cloud_storage_object_from_api(
        &catalog,
        get_request(
            "oyatie:cloud:region-home:ten_other:bucket:tenant-assets",
            OBJECT_KEY,
        ),
    )
    .expect_err("bucket tenant drift is rejected before catalog lookup");
    assert_eq!(tenant_drift.object_status_code(), 403);
    assert!(matches!(
        tenant_drift,
        CloudStorageObjectApiError::TenantMismatch { .. }
    ));
}
