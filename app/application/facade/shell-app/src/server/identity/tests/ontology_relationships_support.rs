use super::super::super::{SignInConfig, router};
use super::super::support::*;
use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, Uri},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub const PATH: &str = "/api/ontology/ten_acme/search-around";
pub type Calls = Arc<Mutex<Vec<(Method, String, HeaderMap, Bytes)>>>;

pub fn query() -> Value {
    json!({"idempotency_key":"attempt-1","root":{"entity_type":"ety_record","object_ref":"ent_alpha","revision":1},"link_type":"lty_related","direction":"outbound","observed_at_epoch_seconds":123})
}
pub fn graph() -> Value {
    json!({"nodes":[{"entity_id":"ent_alpha","entity_type_id":"ety_record","written_revision":1},{"entity_id":"ent_beta","entity_type_id":"ety_profile","written_revision":7}],"edges":[{"from_entity_id":"ent_alpha","to_entity_id":"ent_beta","edge_type_id":"lty_related"}],"observed_at_epoch_seconds":123,"truncated":false})
}

#[derive(Clone)]
struct BackendState {
    calls: Calls,
    reply: Arc<Mutex<Reply>>,
    graph_gate: Arc<Mutex<Option<GraphGate>>>,
}
struct GraphGate {
    reached: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}
pub struct Backend {
    url: String,
    pub calls: Calls,
    pub reply: Arc<Mutex<Reply>>,
    graph_gate: Arc<Mutex<Option<GraphGate>>>,
    task: tokio::task::JoinHandle<()>,
}
impl Backend {
    pub async fn new() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let calls = Arc::new(Mutex::new(Vec::new()));
        let reply = Arc::new(Mutex::new(Reply::json(&graph().to_string())));
        let graph_gate = Arc::new(Mutex::new(None));
        let state = BackendState {
            calls: calls.clone(),
            reply: reply.clone(),
            graph_gate: graph_gate.clone(),
        };
        let task = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(backend).with_state(state))
                .await
                .unwrap();
        });
        Self {
            url,
            calls,
            reply,
            graph_gate,
            task,
        }
    }
    pub fn shell(&self, iam: &Fixture) -> Router {
        router(Some(
            SignInConfig::for_loopback(ORIGIN, &iam.url, &self.url).unwrap(),
        ))
    }
    pub fn hold_graph(&self) -> (oneshot::Receiver<()>, oneshot::Sender<()>) {
        let (reached, arriving) = oneshot::channel();
        let (release, released) = oneshot::channel();
        assert!(
            self.graph_gate
                .lock()
                .unwrap()
                .replace(GraphGate {
                    reached,
                    release: released,
                })
                .is_none()
        );
        (arriving, release)
    }
}
impl Drop for Backend {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn backend(
    State(state): State<BackendState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    let graph_request =
        method == Method::POST && uri.path() == "/v1/tenants/ten_acme/search-around";
    state
        .calls
        .lock()
        .unwrap()
        .push((method, uri.to_string(), headers, bytes));
    if uri.path().ends_with("/link-types") {
        return axum::Json(json!([{"link_type":"lty_related","from_entity_type":"ety_record","to_entity_type":"ety_profile","cardinality":"one_to_many","allow_cross_tenant":false,"revision":1}])).into_response();
    }
    if graph_request {
        let gate = state.graph_gate.lock().unwrap().take();
        if let Some(gate) = gate {
            gate.reached.send(()).unwrap();
            gate.release.await.unwrap();
        }
    }
    let reply = state.reply.lock().unwrap().clone();
    if reply.delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(reply.delay_ms)).await;
    }
    (reply.status, reply.headers, reply.body).into_response()
}
