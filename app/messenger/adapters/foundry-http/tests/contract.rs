use messenger_domain::{Error, ObjectRef};
use messenger_foundry_api::Foundry;
use messenger_foundry_http::HttpFoundry;

fn object(tenant: &str, id: &str, revision: u32) -> ObjectRef {
    ObjectRef {
        tenant: tenant.into(),
        object: id.into(),
        revision,
    }
}

async fn listen() -> (tokio::net::TcpListener, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    (listener, origin)
}

#[tokio::test]
async fn only_canonical_noncommitting_statuses_are_definitive_rejections() {
    let (listener, origin) = listen().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new().route(
                "/v1/actions",
                axum::routing::post(
                    |axum::Json(body): axum::Json<serde_json::Value>| async move {
                        (
                            axum::http::StatusCode::from_u16(
                                body["action_type"].as_str().unwrap().parse().unwrap(),
                            )
                            .unwrap(),
                            axum::Json(serde_json::json!({"accepted": true})),
                        )
                    },
                ),
            ),
        )
        .await
        .unwrap();
    });
    let foundry = HttpFoundry::new(&origin, "acme").unwrap();
    let object = object("acme", "item", 1);
    for status in [200, 307, 400, 401, 403, 409, 429, 500, 503] {
        let result = foundry
            .invoke(
                "viewer",
                &object,
                &status.to_string(),
                "stable-id",
                1,
                Default::default(),
            )
            .await;
        match status {
            200 => assert!(result.is_ok(), "HTTP {status}: {result:?}"),
            400 | 401 | 403 => {
                assert!(
                    matches!(result, Err(Error::ActionRejected)),
                    "HTTP {status}: {result:?}"
                );
            }
            429 | 500 | 503 => {
                assert!(
                    matches!(result, Err(Error::Unavailable(_))),
                    "HTTP {status}: {result:?}"
                );
            }
            _ => assert!(result.is_err(), "HTTP {status}: {result:?}"),
        }
    }
    server.abort();
}

#[tokio::test]
async fn foundry_503_is_unavailable() {
    let (listener, origin) = listen().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new().route(
                "/v1/actions",
                axum::routing::post(|| async { axum::http::StatusCode::SERVICE_UNAVAILABLE }),
            ),
        )
        .await
        .unwrap();
    });
    let foundry = HttpFoundry::new(&origin, "acme").unwrap();
    let result = foundry
        .invoke(
            "viewer",
            &object("acme", "item", 1),
            "assign",
            "stable-id",
            1,
            Default::default(),
        )
        .await;
    assert!(matches!(result, Err(Error::Unavailable(_))), "{result:?}");
    server.abort();
}

#[tokio::test]
async fn redirects_cannot_acknowledge_an_action_or_return_an_object() {
    let (listener, origin) = listen().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            axum::Router::new().fallback(|| async {
                (
                    axum::http::StatusCode::TEMPORARY_REDIRECT,
                    axum::Json(serde_json::json!({"accepted": true})),
                )
            }),
        )
        .await
        .unwrap();
    });
    let foundry = HttpFoundry::new(&origin, "acme").unwrap();
    let object = object("acme", "item", 1);
    assert!(
        foundry
            .invoke(
                "viewer",
                &object,
                "assign",
                "stable-id",
                1,
                Default::default()
            )
            .await
            .is_err()
    );
    assert!(foundry.read("viewer", &object).await.is_err());
    server.abort();
}

#[tokio::test]
async fn oversized_upstream_json_is_rejected_with_or_without_content_length() {
    use std::io::{Read, Write};
    for chunked in [false, true] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_write_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            let mut request = [0; 4096];
            let _ = socket.read(&mut request);
            let body =
                serde_json::to_vec(&serde_json::json!({"value": "a".repeat(1_048_576)})).unwrap();
            let header = if chunked {
                "Transfer-Encoding: chunked".to_owned()
            } else {
                format!("Content-Length: {}", body.len())
            };
            let _ = write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{header}\r\nConnection: close\r\n\r\n"
            );
            if chunked {
                let _ = write!(socket, "{:x}\r\n", body.len());
            }
            let _ = socket.write_all(&body);
            if chunked {
                let _ = socket.write_all(b"\r\n0\r\n\r\n");
            }
        });
        let client = HttpFoundry::new(&origin, "acme").unwrap();
        let result = client.read("user-token", &object("acme", "item", 1)).await;
        tokio::task::spawn_blocking(move || server.join().unwrap())
            .await
            .unwrap();
        assert!(
            matches!(result, Err(Error::Unavailable(ref detail)) if detail == "Foundry response exceeds 1 MiB"),
            "responses over 1MiB must be bounded, chunked={chunked}"
        );
    }
}

#[tokio::test]
async fn user_credential_revision_and_idempotency_reach_foundry_unchanged() {
    let (listener, origin) = listen().await;
    let app = axum::Router::new()
        .route(
            "/v1/objects/{id}",
            axum::routing::get(
                |axum::extract::Path(id): axum::extract::Path<String>,
                 axum::extract::RawQuery(query): axum::extract::RawQuery,
                 headers: axum::http::HeaderMap| async move {
                    assert_eq!(id, "entity:part/1");
                    assert_eq!(query.as_deref(), Some("revision=7"));
                    if headers["authorization"] != "Bearer user-credential" {
                        return Err(axum::http::StatusCode::FORBIDDEN);
                    }
                    Ok(axum::Json(
                        serde_json::json!({"object_ref": id, "revision": 7}),
                    ))
                },
            ),
        )
        .route(
            "/v1/actions",
            axum::routing::post(
                |headers: axum::http::HeaderMap,
                 axum::Json(body): axum::Json<serde_json::Value>| async move {
                    assert_eq!(headers["authorization"], "Bearer user-credential");
                    assert_eq!(body["idempotency_key"], "operation-1");
                    assert_eq!(body["object_ref"], "entity:part/1");
                    axum::Json(serde_json::json!({"accepted": true}))
                },
            ),
        );
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = HttpFoundry::new(&origin, "acme").unwrap();
    let mut pinned = object("acme", "entity:part/1", 7);
    assert!(client.read("user-credential", &pinned).await.is_ok());
    assert!(client.read("bot-credential", &pinned).await.is_err());
    client
        .invoke(
            "user-credential",
            &pinned,
            "assign",
            "operation-1",
            1,
            Default::default(),
        )
        .await
        .unwrap();
    pinned.tenant = "other".into();
    assert!(client.read("user-credential", &pinned).await.is_err());
    task.abort();
    pinned.tenant = "acme".into();
    assert!(client.read("user-credential", &pinned).await.is_err());
}

#[tokio::test]
async fn invoke_maps_local_refusals_to_action_rejected() {
    let foundry = HttpFoundry::new("http://127.0.0.1:1", "acme").unwrap();
    let pinned = object("acme", "item", 1);
    assert!(matches!(
        foundry
            .invoke("", &pinned, "assign", "k", 1, Default::default())
            .await,
        Err(Error::ActionRejected)
    ));
    assert!(matches!(
        foundry
            .invoke("viewer", &pinned, "", "k", 1, Default::default())
            .await,
        Err(Error::ActionRejected)
    ));
}
