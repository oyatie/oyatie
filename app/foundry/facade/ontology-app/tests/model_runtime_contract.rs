#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::{Router, routing::get};
use foundry_model_api::{ModelBaseline, ModelBinding, control::ModelWorkspace};
use foundry_model_sqlite_draft::SqliteModelStore;
use iam_identity_user_session::SessionView;
use std::{path::PathBuf, process::Command};

const TENANT: &str = "ten_model_runtime";
const TOKEN: &str = "v1.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

struct Process(std::process::Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "foundry-model-runtime-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn standard_process_opens_established_model_through_real_human_routes() {
    let directory = Directory::new();
    let binding = ModelBinding {
        tenant_id: TENANT.into(),
        store_id: "runtime-model".into(),
        baseline_id: "runtime-record-baseline".into(),
    };
    let baseline = ModelBaseline::capture(
        &foundry_ontology_app::seed::registry_for(TENANT).unwrap(),
        TENANT,
    )
    .unwrap();
    let model_path = directory.0.join("model.sqlite");
    drop(SqliteModelStore::initialize(&model_path, binding.clone(), baseline).unwrap());
    let iam = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let iam_origin = format!("http://{}", iam.local_addr().unwrap());
    let iam_task = tokio::spawn(async move {
        axum::serve(
            iam,
            Router::new().route(
                "/identity/v1/{tenant}/session",
                get(|| async {
                    axum::Json(SessionView {
                        tenant_id: TENANT.into(),
                        user_id: "usr_designer".into(),
                        expires_at: i64::MAX,
                        roles: vec!["foundry-model-author".into()],
                    })
                }),
            ),
        )
        .await
        .unwrap();
    });
    let address = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let mut process = Command::new(env!("CARGO_BIN_EXE_foundry-ontology-app"));
    process
        .env("OYATIE_FOUNDRY_ONTOLOGY_LISTEN_ADDR", address.to_string())
        .env("OYATIE_FOUNDRY_ONTOLOGY_TENANTS", TENANT)
        .env(
            "OYATIE_FOUNDRY_ONTOLOGY_ACTION_LOG",
            directory.0.join("actions.sqlite"),
        )
        .env(
            "OYATIE_FOUNDRY_ONTOLOGY_DENIAL_LOG",
            directory.0.join("denials.sqlite"),
        )
        .env(
            "OYATIE_FOUNDRY_ONTOLOGY_PROJECTION_STORE",
            directory.0.join("projection.sqlite"),
        )
        .env(
            "OYATIE_FOUNDRY_ONTOLOGY_MODELS",
            serde_json::json!([{"path":model_path,"binding":binding}]).to_string(),
        )
        .env(
            "OYATIE_FOUNDRY_ONTOLOGY_MODEL_ORIGIN",
            "http://localhost:33000",
        )
        .env("OYATIE_FOUNDRY_ONTOLOGY_MODEL_ALLOW_LOOPBACK_HTTP", "true")
        .env("OYATIE_FOUNDRY_IAM_BASE_URL", iam_origin)
        .env("OYATIE_FOUNDRY_IAM_ALLOW_LOOPBACK_HTTP", "true");
    let _process = Process(process.spawn().unwrap());
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let origin = format!("http://{address}");
    let mut ready = false;
    for _ in 0..100 {
        if client.get(format!("{origin}/healthz")).send().await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(ready, "the standard process did not start");
    let response = client
        .get(format!("{origin}/v1/tenants/{TENANT}/model"))
        .header("cookie", format!("__Host-oya_session={TOKEN}"))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let body = response.bytes().await.unwrap();
    iam_task.abort();
    assert_eq!(
        status,
        reqwest::StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&body)
    );
    assert_eq!(ModelWorkspace::decode(&body).unwrap().generation.get(), 1);
}
