#![allow(dead_code)]
use axum::{Router, routing::get};
use foundry_model_api::{ModelBaseline, ModelBinding, ModelCaller};
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_ontology_app::{Config, model_runtime::ModelRuntimeConfig};
use iam_identity_user_session::SessionView;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

static NEXT_FIXTURE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub const TENANT: &str = "ten_model_runtime";
pub const TOKEN: &str = "v1.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
pub const ORIGIN: &str = "http://localhost:33000";
pub struct Fixture {
    pub directory: PathBuf,
    pub config: Config,
    pub models: ModelRuntimeConfig,
}
impl Fixture {
    pub fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "foundry-model-runtime-{}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let config = Config {
            listen_addr: "127.0.0.1:0".into(),
            action_log: directory.join("actions.sqlite"),
            denial_log: directory.join("denials.sqlite"),
            projection_store: directory.join("projection.sqlite"),
            tenants: vec![TENANT.into()],
            operators: vec![],
        };
        let models = ModelRuntimeConfig::from_values(
            Some(
                &serde_json::json!([
                    {"path":directory.join("model.sqlite"),"binding":Self::binding()}
                ])
                .to_string(),
            ),
            Some(ORIGIN),
            Some("true"),
        )
        .unwrap()
        .unwrap();
        Self {
            directory,
            config,
            models,
        }
    }
    pub fn binding() -> ModelBinding {
        ModelBinding {
            tenant_id: TENANT.into(),
            store_id: "runtime-model".into(),
            baseline_id: "runtime-record-baseline".into(),
        }
    }
    pub fn baseline() -> ModelBaseline {
        ModelBaseline::capture(
            &foundry_ontology_app::seed::registry_for(TENANT).unwrap(),
            TENANT,
        )
        .unwrap()
    }
    pub fn caller() -> ModelCaller {
        ModelCaller {
            tenant_id: TENANT.into(),
            owner: "usr_designer".into(),
        }
    }
    pub fn model_path(&self) -> PathBuf {
        self.directory.join("model.sqlite")
    }
    pub fn open(&self) -> SqliteModelStore {
        SqliteModelStore::open_existing(&self.model_path(), Self::binding(), Self::baseline())
            .unwrap()
    }
    pub fn initialize(&self) {
        foundry_ontology_app::model_runtime::initialize_model(&self.config, &self.models, TENANT)
            .unwrap();
    }
    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_foundry-ontology-app"));
        command
            .env("OYATIE_FOUNDRY_ONTOLOGY_TENANTS", TENANT)
            .env(
                "OYATIE_FOUNDRY_ONTOLOGY_ACTION_LOG",
                &self.config.action_log,
            )
            .env(
                "OYATIE_FOUNDRY_ONTOLOGY_DENIAL_LOG",
                &self.config.denial_log,
            )
            .env(
                "OYATIE_FOUNDRY_ONTOLOGY_PROJECTION_STORE",
                &self.config.projection_store,
            )
            .env(
                "OYATIE_FOUNDRY_ONTOLOGY_MODELS",
                serde_json::json!([{"path":self.model_path(),"binding":Self::binding()}])
                    .to_string(),
            )
            .env("OYATIE_FOUNDRY_ONTOLOGY_MODEL_ORIGIN", ORIGIN)
            .env("OYATIE_FOUNDRY_ONTOLOGY_MODEL_ALLOW_LOOPBACK_HTTP", "true")
            .env_remove("OYATIE_FOUNDRY_IAM_BASE_URL")
            .env_remove("OYATIE_FOUNDRY_IAM_ALLOW_LOOPBACK_HTTP")
            .env(
                "OYATIE_FOUNDRY_ONTOLOGY_OPERATORS",
                format!("runtime-operator:{TENANT}:prn_operator:foundry-operator"),
            );
        command
    }
    pub async fn start(&self, iam: &str) -> Process {
        let address = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let child = self
            .command()
            .env("OYATIE_FOUNDRY_ONTOLOGY_LISTEN_ADDR", address.to_string())
            .env("OYATIE_FOUNDRY_IAM_BASE_URL", iam)
            .env("OYATIE_FOUNDRY_IAM_ALLOW_LOOPBACK_HTTP", "true")
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let mut process = Process {
            child,
            origin: format!("http://{address}"),
            client: reqwest::Client::builder()
                .no_proxy()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .unwrap(),
        };
        for _ in 0..500 {
            if let Some(status) = process.child.try_wait().unwrap() {
                panic!("standard process exited before listening: {status}");
            }
            if process
                .client
                .get(format!("{}/healthz", process.origin))
                .send()
                .await
                .is_ok()
            {
                return process;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("standard process did not start")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
pub struct Process {
    pub child: std::process::Child,
    pub origin: String,
    pub client: reqwest::Client,
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Process {
    pub fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.client
            .request(
                method,
                format!("{}/v1/tenants/{TENANT}/{path}", self.origin),
            )
            .header("cookie", format!("__Host-oya_session={TOKEN}"))
            .header("origin", ORIGIN)
    }
}
pub struct Iam {
    pub origin: String,
    task: tokio::task::JoinHandle<()>,
}
impl Iam {
    pub async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route(
                    "/identity/v1/{tenant}/session",
                    get(|| async {
                        axum::Json(SessionView {
                            tenant_id: TENANT.into(),
                            user_id: "usr_designer".into(),
                            expires_at: i64::MAX,
                            roles: vec!["foundry-model-author".into(), "foundry-operator".into()],
                        })
                    }),
                ),
            )
            .await
            .unwrap();
        });
        Self { origin, task }
    }
}
impl Drop for Iam {
    fn drop(&mut self) {
        self.task.abort();
    }
}
pub async fn bytes(response: reqwest::Response, status: reqwest::StatusCode) -> Vec<u8> {
    let actual = response.status();
    assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
    let body = response.bytes().await.unwrap();
    assert_eq!(actual, status, "{}", String::from_utf8_lossy(&body));
    body.to_vec()
}
