#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use messenger_domain::Error;
use messenger_enterprise_admission_app::{Admission, AppState, Config, Tenant, compose, router};
use messenger_policy_api::{Action, Policy};
use serde_json::{Value, json};
use tower::ServiceExt;

const TOKEN: &str = "test-token-32-bytes-minimum-ok!!";

#[derive(Clone)]
struct Script(Arc<Inner>);

struct Inner {
    result: Mutex<Result<(), Error>>,
    calls: Mutex<Vec<(String, String, Action, String)>>,
}

impl Script {
    fn new(result: Result<(), Error>) -> Self {
        Self(Arc::new(Inner {
            result: Mutex::new(result),
            calls: Mutex::new(Vec::new()),
        }))
    }

    fn set(&self, result: Result<(), Error>) {
        *self.0.result.lock().unwrap() = result;
    }

    fn calls(&self) -> Vec<(String, String, Action, String)> {
        self.0.calls.lock().unwrap().clone()
    }
}

impl Policy for Script {
    async fn authorize(
        &self,
        tenant: &str,
        subject: &str,
        action: Action,
        resource: &str,
    ) -> Result<(), Error> {
        self.0
            .calls
            .lock()
            .unwrap()
            .push((tenant.into(), subject.into(), action, resource.into()));
        self.0.result.lock().unwrap().clone()
    }
}

fn config() -> Config {
    Config {
        listen: "127.0.0.1:0".parse().unwrap(),
        policy_origin: "http://127.0.0.1:1".into(),
        policy_version: "v1".into(),
        workload_identity_file: None,
        tenants: BTreeMap::from([(
            "acme".into(),
            Tenant {
                audit_bot: "@bot:local".into(),
                subjects: BTreeMap::from([
                    ("@alice:local".into(), "usr_alice".into()),
                    ("@bot:local".into(), "svc_archive".into()),
                ]),
            },
        )]),
    }
}

fn state(policy: Script) -> Arc<AppState<Script>> {
    AppState::bind(config(), TOKEN, policy).unwrap()
}

fn encrypted() -> Value {
    json!({"event_id":"$1","tenant":"acme","audit_bot":"@bot:local",
        "event":{"sender":"@alice:local","room_id":"!work:local","type":"m.room.encrypted"},
        "create":{"type":"dev.oyatie.enterprise","m.federate":false,"dev.oyatie.tenant":"acme",
            "dev.oyatie.audit_bot":"@bot:local","dev.oyatie.policy_resource":"work-order:42"}})
}

struct Client {
    router: axum::Router,
    state: Arc<AppState<Script>>,
}

impl Client {
    fn of(policy: Script) -> Self {
        let state = state(policy);
        Self {
            router: router(state.clone()),
            state,
        }
    }

    async fn send(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        body: &[u8],
    ) -> (StatusCode, Vec<u8>) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        if !body.is_empty() {
            builder = builder.header("content-type", "application/json");
        }
        let response = self
            .router
            .clone()
            .oneshot(builder.body(Body::from(body.to_vec())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, bytes.to_vec())
    }

    async fn admit(&self, body: Value) -> (StatusCode, Value) {
        let (status, bytes) = self
            .send(
                "POST",
                "/v1/messenger/admit",
                Some(TOKEN),
                &serde_json::to_vec(&body).unwrap(),
            )
            .await;
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }
}

#[tokio::test]
async fn unauthenticated_body_is_not_parsed() {
    let (status, _) = Client::of(Script::new(Ok(())))
        .send("POST", "/v1/messenger/admit", None, b"malformed")
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn allow_calls_the_use_case_with_parc() {
    let policy = Script::new(Ok(()));
    let (status, body) = Client::of(policy.clone()).admit(encrypted()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["event_id"], "$1");
    assert_eq!(body["allow"], true);
    assert_eq!(body["obligations"], json!([]));
    assert_eq!(
        policy.calls(),
        vec![(
            "acme".into(),
            "usr_alice".into(),
            Action::Send,
            "work-order:42".into()
        )]
    );
}

#[tokio::test]
async fn deny_and_outage_stay_errors() {
    let (status, _) = Client::of(Script::new(Err(Error::Denied)))
        .admit(encrypted())
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let outage = Script::new(Err(Error::Unavailable("policy down".into())));
    let client = Client::of(outage.clone());
    let (status, _) = client.admit(encrypted()).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let (ready, _) = client.send("GET", "/readyz", None, b"").await;
    assert_eq!(ready, StatusCode::OK);
    outage.set(Ok(()));
    let (status, body) = client.admit(encrypted()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn roster_and_room_binding_cannot_be_replaced() {
    let policy = Script::new(Ok(()));
    let app = state(policy.clone());
    let mut request: Admission = serde_json::from_value(encrypted()).unwrap();
    assert!(app.authorize_admission(&request).await.is_ok());
    request.tenant = "other".into();
    assert_eq!(app.authorize_admission(&request).await, Err(Error::Denied));
    request.tenant = "acme".into();
    request.event["sender"] = json!("@outsider:local");
    assert_eq!(app.authorize_admission(&request).await, Err(Error::Denied));
    request.event["sender"] = json!("@alice:local");
    request.create["type"] = Value::Null;
    assert_eq!(app.authorize_admission(&request).await, Err(Error::Denied));
    assert_eq!(policy.calls().len(), 1);
}

#[tokio::test]
async fn empty_resource_and_unknown_type_skip_policy() {
    let policy = Script::new(Ok(()));
    let app = state(policy.clone());
    let mut request: Admission = serde_json::from_value(encrypted()).unwrap();
    request.create["dev.oyatie.policy_resource"] = json!("");
    assert_eq!(app.authorize_admission(&request).await, Err(Error::Denied));
    request.create["dev.oyatie.policy_resource"] = json!("work-order:42");
    request.event["type"] = json!("m.room.message");
    assert_eq!(app.authorize_admission(&request).await, Err(Error::Denied));
    assert!(policy.calls().is_empty());
}

#[tokio::test]
async fn manager_can_remove_departed_users() {
    let policy = Script::new(Ok(()));
    let app = state(policy.clone());
    let mut request: Admission = serde_json::from_value(encrypted()).unwrap();
    request.event = json!({
        "sender":"@alice:local","room_id":"!work:local","type":"m.room.member",
        "state_key":"@departed:local","content":{"membership":"leave"}
    });
    for membership in ["leave", "ban"] {
        request.event["content"]["membership"] = json!(membership);
        assert_eq!(
            app.authorize_admission(&request).await,
            Ok(()),
            "{membership}"
        );
    }
    for membership in ["invite", "join", "knock", "unknown"] {
        request.event["content"]["membership"] = json!(membership);
        assert_eq!(
            app.authorize_admission(&request).await,
            Err(Error::Denied),
            "{membership}"
        );
    }
    assert_eq!(policy.calls().len(), 2);
    assert!(
        policy
            .calls()
            .iter()
            .all(|call| call.2 == Action::ManageRoom)
    );
}

#[tokio::test]
async fn saturation_is_unavailable_without_a_queue() {
    let client = Client::of(Script::new(Ok(())));
    let _held: Vec<_> = (0..64).map(|_| client.state.try_hold().unwrap()).collect();
    let (status, _) = client
        .send("POST", "/v1/messenger/admit", Some(TOKEN), b"{}")
        .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn compose_wires_oyatie_policy_on_loopback() {
    assert!(compose(config(), TOKEN).is_ok());
}
