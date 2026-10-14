use super::support::*;
use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use foundry_ontology_app::{
    Config, OperatorCredential, compose,
    human_session::{HttpUserSessionAuthorizer, HumanSessionConfig},
    router_from, router_with_user_sessions,
};
use std::{path::PathBuf, sync::Arc};
use tower::ServiceExt;

pub(super) struct NativeFixture {
    pub(super) state: Arc<foundry_ontology_app::AppState>,
    files: Vec<PathBuf>,
    url: String,
    server: tokio::task::JoinHandle<()>,
}
impl NativeFixture {
    pub(super) async fn new(iam: &Fixture) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let files: Vec<_> = ["action", "denial", "store"]
            .iter()
            .map(|name| {
                std::env::temp_dir().join(format!(
                    "shell-native-ontology-{}-{stamp}-{name}.sqlite",
                    std::process::id()
                ))
            })
            .collect();
        let config = Config {
            listen_addr: "127.0.0.1:0".into(),
            action_log: files[0].clone(),
            denial_log: files[1].clone(),
            projection_store: files[2].clone(),
            tenants: vec!["ten_acme".into()],
            operators: vec![OperatorCredential {
                token: "fixture-operator".into(),
                tenant_id: "ten_acme".into(),
                principal_id: "prn_fixture".into(),
                roles: vec!["foundry-operator".into()],
            }],
        };
        let state = Arc::new(compose(&config).unwrap());
        let write=router_from(state).oneshot(Request::builder().method("POST").uri("/v1/actions").header(header::AUTHORIZATION,"Bearer fixture-operator").header(header::CONTENT_TYPE,"application/json").body(Body::from(r#"{"object_ref":"ent_alpha","action_type":"aty_record_write","idempotency_key":"idem_fixture","occurred_at_epoch_seconds":1700000000,"properties":{"name":"Ada"}}"#)).unwrap()).await.unwrap();
        assert_eq!(write.status(), StatusCode::OK);
        // A second composition reads durable state rather than the write handler's in-memory fold.
        let sessions =
            HttpUserSessionAuthorizer::new(HumanSessionConfig::loopback_http(&iam.url).unwrap())
                .unwrap();
        let state = Arc::new(compose(&config).unwrap());
        let router = router_with_user_sessions(state.clone(), Arc::new(sessions));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            files,
            url,
            server,
            state,
        }
    }
    pub(super) fn shell(&self, iam: &Fixture) -> Router {
        super::super::router(Some(
            super::super::SignInConfig::for_loopback(ORIGIN, &iam.url, &self.url).unwrap(),
        ))
    }
}
impl Drop for NativeFixture {
    fn drop(&mut self) {
        self.server.abort();
        for path in &self.files {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
            }
        }
    }
}
const READS: [&str; 8] = [
    "types",
    "schemas",
    "objects?type=ety_record&revision=1&limit=1",
    "objects/ent_alpha?type=ety_record&revision=1",
    "objects/ent_alpha/history",
    "audit",
    "link-types",
    "type-labels",
];
async fn read(shell: &Router, suffix: &str, status: StatusCode) -> serde_json::Value {
    let r = call(
        shell.clone(),
        Method::GET,
        &format!("/api/ontology/ten_acme/{suffix}"),
        headers(),
        Body::empty(),
    )
    .await;
    expect_no_store(&r, status);
    if status == StatusCode::OK {
        assert_eq!(r.headers()["x-oyatie-tenant"], "ten_acme");
        assert_eq!(r.headers()["x-oyatie-user"], "usr_alice");
        assert!(
            r.headers()["x-oyatie-session-expires"]
                .to_str()
                .unwrap()
                .parse::<i64>()
                .unwrap()
                > super::super::config::now()
        );
    }
    json(r).await
}
#[tokio::test]
async fn real_foundry_human_routes_answer_durable_data_and_revalidate_local_iam_on_every_shell_read()
 {
    let iam = Fixture::new(&session()).await;
    let native = NativeFixture::new(&iam).await;
    let shell = native.shell(&iam);
    let types = read(&shell, READS[0], StatusCode::OK).await;
    assert_eq!(
        types,
        serde_json::json!([{"entity_type":"ety_record","revision":1,"properties":["name","note"]}])
    );
    let schemas = read(&shell, READS[1], StatusCode::OK).await;
    assert_eq!(
        schemas,
        serde_json::json!([{
            "entity_type":"ety_record", "revision":1,
            "properties":[
                {"name":"name","tier":"scalar","data_class":"INTERNAL_ONLY","required":true,"value_type":null},
                {"name":"note","tier":"scalar","data_class":"INTERNAL_ONLY","required":false,"value_type":null}
            ],
            "primary_key_property":null,"title_property":"name"
        }])
    );
    let page = read(&shell, READS[2], StatusCode::OK).await;
    assert_eq!(page["requested_revision"], 1);
    assert_eq!(page["objects"][0]["object_ref"], "ent_alpha");
    assert_eq!(page["next"], serde_json::Value::Null);
    let detail = read(&shell, READS[3], StatusCode::OK).await;
    assert_eq!(detail["object"], page["objects"][0]);
    assert_eq!(detail["object"]["properties"]["name"]["value"], "Ada");
    assert_eq!(detail["object"]["upcast_state"], "current");
    assert_eq!(detail["object"]["written_revision"], 1);
    let history = read(&shell, READS[4], StatusCode::OK).await;
    assert_eq!(history[0]["ordinal"], 1);
    assert_eq!(history[0]["principal_id"], "prn_fixture");
    assert_eq!(history[0]["audit_event_type"], "record.written");
    let audit = read(&shell, READS[5], StatusCode::OK).await;
    assert_eq!(audit[0]["object_ref"], "ent_alpha");
    assert_eq!(audit[0]["disposition"], "applied");
    let links = read(&shell, READS[6], StatusCode::OK).await;
    assert_eq!(links, serde_json::json!([]));
    let labels = read(&shell, READS[7], StatusCode::OK).await;
    assert_eq!(
        labels,
        serde_json::json!([{"entity_type":"ety_record","revision":1,"display_name":"Record"}])
    );
    println!(
        "native_ontology_fixture_results={}",
        serde_json::json!({"types":types,"schemas":schemas,"links":links,"labels":labels,"page":page,"detail":detail,"history":history,"audit":audit})
    );
    assert_eq!(iam.calls.lock().unwrap().len(), READS.len() * 2);
    iam.reply.lock().unwrap().body = session().replace(r#"["foundry-operator"]"#, "[]");
    for suffix in READS {
        assert_eq!(
            read(&shell, suffix, StatusCode::FORBIDDEN).await["error"],
            "forbidden"
        );
    }
    assert_eq!(iam.calls.lock().unwrap().len(), READS.len() * 4);
    {
        let mut reply = iam.reply.lock().unwrap();
        reply.status = StatusCode::UNAUTHORIZED;
        reply.body = "{}".into();
    }
    for suffix in READS {
        assert_eq!(
            read(&shell, suffix, StatusCode::UNAUTHORIZED).await["error"],
            "unauthenticated"
        );
    }
    let calls = iam.calls.lock().unwrap();
    assert_eq!(calls.len(), READS.len() * 5);
    for (path, headers, body) in calls.iter() {
        assert_eq!(path, "/identity/v1/ten_acme/session");
        assert_eq!(headers[header::COOKIE], cookie());
        assert!(!headers.contains_key(header::AUTHORIZATION));
        assert!(body.is_empty());
    }
}
