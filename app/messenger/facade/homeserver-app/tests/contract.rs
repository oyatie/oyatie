#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use messenger_homeserver_app::router;
use serde_json::{Value, json};
use tower::ServiceExt;

const ALICE: &str = "@alice:messenger.test";
const BOB: &str = "@bob:messenger.test";

struct Client {
    router: axum::Router,
}

impl Client {
    fn new() -> Self {
        Self {
            router: router("messenger.test"),
        }
    }

    async fn call(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        if body.is_some() {
            builder = builder.header("content-type", "application/json");
        }
        let bytes = body
            .map(|value| serde_json::to_vec(&value).unwrap())
            .unwrap_or_default();
        let response = self
            .router
            .clone()
            .oneshot(builder.body(Body::from(bytes)).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        (status, json)
    }
}

async fn login(client: &Client, user: &str) -> String {
    let (status, body) = client
        .call(
            "POST",
            "/_matrix/client/v3/login",
            None,
            Some(json!({
                "type": "m.login.password",
                "identifier": { "type": "m.id.user", "user": user },
                "password": "x",
                "device_id": "DEV",
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["access_token"].as_str().unwrap().to_owned()
}

async fn public_room(client: &Client) -> (String, String, String) {
    let alice = login(client, ALICE).await;
    let bob = login(client, BOB).await;
    let (status, body) = client
        .call(
            "POST",
            "/_matrix/client/v3/createRoom",
            Some(&alice),
            Some(json!({ "preset": "public_chat" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let room = body["room_id"].as_str().unwrap().to_owned();
    let (status, body) = client
        .call(
            "POST",
            &format!("/_matrix/client/v3/rooms/{room}/join"),
            Some(&bob),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    (alice, bob, room)
}

async fn send(
    client: &Client,
    token: &str,
    room: &str,
    txn: &str,
    body: &str,
) -> (StatusCode, Value) {
    client
        .call(
            "PUT",
            &format!("/_matrix/client/v3/rooms/{room}/send/m.room.message/{txn}"),
            Some(token),
            Some(json!({ "msgtype": "m.text", "body": body })),
        )
        .await
}

async fn sync(client: &Client, token: &str) -> Value {
    let (status, body) = client
        .call("GET", "/_matrix/client/v3/sync", Some(token), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

fn bodies(sync: &Value, room: &str) -> Vec<String> {
    sync["rooms"]["join"][room]["timeline"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["type"] == "m.room.message")
        .map(|event| event["content"]["body"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn send_persists_a_message_visible_on_sync() {
    let client = Client::new();
    let (alice, _, room) = public_room(&client).await;
    let (status, body) = send(&client, &alice, &room, "txn-1", "hello").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["event_id"].as_str().unwrap().starts_with('$'));
    assert_eq!(bodies(&sync(&client, &alice).await, &room), vec!["hello"]);
}

#[tokio::test]
async fn retry_returns_the_original_event_id() {
    let client = Client::new();
    let (alice, _, room) = public_room(&client).await;
    let (first_status, first) = send(&client, &alice, &room, "txn-1", "hello").await;
    let (retry_status, retry) = send(&client, &alice, &room, "txn-1", "changed").await;
    assert_eq!(first_status, StatusCode::OK, "{first}");
    assert_eq!(retry_status, StatusCode::OK, "{retry}");
    assert_eq!(first["event_id"], retry["event_id"]);
    assert_eq!(bodies(&sync(&client, &alice).await, &room), vec!["hello"]);
}

#[tokio::test]
async fn leave_refuses_a_later_send() {
    let client = Client::new();
    let (_, bob, room) = public_room(&client).await;
    let (status, body) = client
        .call(
            "POST",
            &format!("/_matrix/client/v3/rooms/{room}/leave"),
            Some(&bob),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = send(&client, &bob, &room, "txn-after", "too late").await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["errcode"], "M_FORBIDDEN");
}
