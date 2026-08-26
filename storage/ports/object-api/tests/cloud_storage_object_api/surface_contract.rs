use super::common::*;

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_STORAGE_OBJECT_PUT_SURFACE, "cloud.storage.object.put");
    assert_eq!(CLOUD_STORAGE_OBJECT_GET_SURFACE, "cloud.storage.object.get");
    assert_eq!(CloudStorageObjectPutApiStatus::Created.code(), 201);
    assert_eq!(CloudStorageObjectPutApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudStorageObjectPutApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudStorageObjectPutApiStatus::NotFound.code(), 404);
    assert_eq!(CloudStorageObjectPutApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudStorageObjectPutApiStatus::UnprocessableEntity.code(),
        422
    );
    assert_eq!(CloudStorageObjectGetApiStatus::Ok.code(), 200);
    assert_eq!(CloudStorageObjectGetApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudStorageObjectGetApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudStorageObjectGetApiStatus::NotFound.code(), 404);
}
