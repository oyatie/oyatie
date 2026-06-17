use cell_bind_api::{
    CLOUD_CELL_BIND_EVIDENCE_SURFACE, CloudCellBindApiResponse, CloudCellBindApiStatus,
    CloudCellBindRequest, bind_cloud_cell_from_api,
};

fn valid_request() -> CloudCellBindRequest {
    CloudCellBindRequest {
        tenant_id: "ten_001".to_owned(),
        home_region_code: "region-home-1".to_owned(),
        residency_class: "strict_home_region".to_owned(),
        required_density: Some("shared".to_owned()),
    }
}

#[test]
fn binds_cell_and_exposes_evidence_surface() {
    assert_eq!(CLOUD_CELL_BIND_EVIDENCE_SURFACE, "cloud.cell.bind");
    let result = bind_cloud_cell_from_api(
        "req_001".to_owned(),
        "ten_001".to_owned(),
        "ten_001".to_owned(),
        "idem_001".to_owned(),
        valid_request(),
    );
    assert_eq!(result.status, CloudCellBindApiStatus::Created);
    assert!(matches!(
        result.response,
        CloudCellBindApiResponse::Created(_)
    ));
}

#[test]
fn covers_documented_status_codes() {
    assert_eq!(CloudCellBindApiStatus::Created.code(), 201);
    assert_eq!(CloudCellBindApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudCellBindApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudCellBindApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudCellBindApiStatus::NotFound.code(), 404);
    assert_eq!(CloudCellBindApiStatus::Conflict.code(), 409);
    assert_eq!(CloudCellBindApiStatus::UnprocessableEntity.code(), 422);
}
