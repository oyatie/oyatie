use super::*;

#[tokio::test]
async fn group_create_and_list() {
    let fixture = fixture();
    let token = fixture.token.as_str();
    let (status, created) = send(
        &fixture.router,
        "POST",
        "/scim/v2/ten_acme/Groups",
        Some(token),
        Some(serde_json::json!({"displayName": "platform-admins"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(created["id"].as_str().is_some());

    let (status, listing) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_acme/Groups",
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listing["totalResults"].as_u64(), Some(1));
}
