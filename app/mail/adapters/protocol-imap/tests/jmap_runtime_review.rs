#[path = "runtime_review/support.rs"]
mod support;
use axum::http::StatusCode;
use std::{sync::atomic::Ordering, time::Duration};
use support::{Fixture, Site, TOKEN, request};
use tower::ServiceExt;

const GET_MAILBOXES: &str = r#"{"using":["urn:ietf:params:jmap:mail"],"methodCalls":[["Mailbox/get",{"accountId":"a"},"x"]]}"#;

#[test]
fn disconnected_callers_keep_queued_and_running_global_admission() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .unwrap()
        .block_on(async {
            let fixture = Fixture::new(Site::Authentication);
            let mut tasks = Vec::new();
            for _ in 0..128 {
                tasks.push(tokio::spawn(fixture.app.clone().oneshot(request(
                    "/.well-known/jmap",
                    Some(TOKEN),
                    "application/json",
                    "",
                ))));
                tokio::task::yield_now().await;
            }
            fixture.gate.entered(1).await;
            assert_eq!(
                fixture.gate.count.load(Ordering::SeqCst),
                1,
                "one executing worker; remaining jobs are queued"
            );
            for task in tasks {
                task.abort();
                let _ = task.await;
            }
            let refused = fixture
                .app
                .clone()
                .oneshot(request(
                    "/.well-known/jmap",
                    Some(TOKEN),
                    "application/json",
                    "",
                ))
                .await
                .unwrap();
            assert_eq!(refused.status(), StatusCode::SERVICE_UNAVAILABLE);
            let unauthenticated = fixture
                .app
                .clone()
                .oneshot(request("/.well-known/jmap", None, "application/json", ""))
                .await
                .unwrap();
            assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
            fixture.gate.open();
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    let response = fixture
                        .app
                        .clone()
                        .oneshot(request(
                            "/.well-known/jmap",
                            Some(TOKEN),
                            "application/json",
                            "",
                        ))
                        .await
                        .unwrap();
                    if response.status() == StatusCode::OK {
                        break;
                    }
                    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            })
            .await
            .unwrap();
        });
}

#[tokio::test]
async fn disconnected_upload_keeps_principal_admission_until_storage_finishes() {
    let fixture = Fixture::new(Site::Upload);
    let task = tokio::spawn(fixture.app.clone().oneshot(request(
        "/upload/a",
        Some(TOKEN),
        "application/octet-stream",
        "first",
    )));
    fixture.gate.entered(1).await;
    task.abort();
    let _ = task.await;
    let refused = fixture
        .app
        .clone()
        .oneshot(request(
            "/upload/a",
            Some(TOKEN),
            "application/octet-stream",
            "second",
        ))
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(fixture.gate.count.load(Ordering::SeqCst), 1);
    fixture.gate.open();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let response = fixture
                .app
                .clone()
                .oneshot(request(
                    "/upload/a",
                    Some(TOKEN),
                    "application/octet-stream",
                    "second",
                ))
                .await
                .unwrap();
            if response.status() == StatusCode::OK {
                break;
            }
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn disconnected_dispatch_keeps_all_four_principal_slots() {
    let fixture = Fixture::new(Site::AccountInfo);
    let mut tasks = Vec::new();
    for _ in 0..4 {
        tasks.push(tokio::spawn(fixture.app.clone().oneshot(request(
            "/jmap",
            Some(TOKEN),
            "application/json",
            GET_MAILBOXES,
        ))));
    }
    fixture.gate.entered(4).await;
    for task in tasks {
        task.abort();
        let _ = task.await;
    }
    let refused = fixture
        .app
        .clone()
        .oneshot(request(
            "/jmap",
            Some(TOKEN),
            "application/json",
            GET_MAILBOXES,
        ))
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let body = axum::body::to_bytes(refused.into_body(), 1024)
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap()["type"],
        "urn:ietf:params:jmap:error:limit"
    );
    assert_eq!(fixture.gate.count.load(Ordering::SeqCst), 4);
    fixture.gate.open();
}

#[tokio::test]
async fn actual_blob_read_yields_and_preserves_download_headers() {
    let fixture = Fixture::new(Site::Download);
    let path = format!("/download/a/{}/mail.eml?type=message/rfc822", fixture.blob);
    let task = tokio::spawn(fixture.app.clone().oneshot(request(
        &path,
        Some(TOKEN),
        "application/json",
        "",
    )));
    fixture.gate.entered(1).await;
    // Reaching here on the current-thread runtime proves the Store read yielded.
    fixture.gate.open();
    let response = task.await.unwrap().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "message/rfc822");
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert_eq!(
        axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap()
            .as_ref(),
        b"independent-review"
    );
}

#[tokio::test]
async fn authentication_and_authorization_still_precede_payload_errors() {
    let fixture = Fixture::new(Site::Upload);
    for (token, account, expected) in [
        (None, "a", 401),
        (Some("invalid"), "a", 401),
        (Some(TOKEN), "absent", 404),
        (Some(TOKEN), "a", 400),
    ] {
        let response = fixture
            .app
            .clone()
            .oneshot(request(
                &format!("/upload/{account}"),
                token,
                "invalid",
                "body",
            ))
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), expected);
    }
    assert_eq!(
        fixture.gate.count.load(Ordering::SeqCst),
        0,
        "refused uploads never reached storage"
    );
    let response = fixture
        .app
        .clone()
        .oneshot(request("/jmap", Some(TOKEN), "text/plain", "invalid JSON"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn worker_panic_fails_closed_and_recovers_admission() {
    let fixture = Fixture::new(Site::Upload);
    let response = fixture
        .app
        .clone()
        .oneshot(request(
            "/.well-known/jmap",
            Some("panic"),
            "application/json",
            "",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let response = fixture
        .app
        .clone()
        .oneshot(request(
            "/.well-known/jmap",
            Some(TOKEN),
            "application/json",
            "",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
