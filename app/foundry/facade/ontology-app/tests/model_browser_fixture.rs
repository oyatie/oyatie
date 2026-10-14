#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use foundry_model_api::{ModelBaseline, ModelCaller, ModelStore};
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_ontology_app::{
    Config, compose_with_models,
    human_session::{HttpUserSessionAuthorizer, HumanSessionConfig},
    model_runtime::{ModelRuntimeConfig, initialize_model},
    router_with_model_control,
};
use iam_identity_user_session::SessionView;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[path = "model_browser/model.rs"]
mod model;

const TENANT: &str = "ten_design";
const USER: &str = "usr_model_designer";
const COOKIE: &str = "__Host-oya_session=v1.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

struct FakeIam {
    active: AtomicBool,
    expires: i64,
    shell: String,
}

async fn session(
    State(state): State<Arc<FakeIam>>,
    Path(tenant): Path<String>,
    headers: HeaderMap,
) -> Response {
    if tenant != TENANT
        || !state.active.load(Ordering::SeqCst)
        || headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) != Some(COOKIE)
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    axum::Json(SessionView {
        tenant_id: TENANT.into(),
        user_id: USER.into(),
        expires_at: state.expires,
        roles: vec!["foundry-model-author".into(), "foundry-operator".into()],
    })
    .into_response()
}

async fn sign_in(State(state): State<Arc<FakeIam>>) -> Response {
    state.active.store(true, Ordering::SeqCst);
    let mut response = Redirect::to(&state.shell).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        format!("{COOKIE}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=3600")
            .parse()
            .unwrap(),
    );
    response
}

async fn logout(State(state): State<Arc<FakeIam>>) -> Response {
    state.active.store(false, Ordering::SeqCst);
    (
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            "__Host-oya_session=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0",
        )],
    )
        .into_response()
}

/// An explicit browser harness: native stores/routes, with only IAM replaced by a labeled fake.
#[tokio::test]
#[ignore = "explicit controlled browser fixture; requires shell origin and a dedicated durable directory"]
async fn serve_real_model_for_browser() {
    let shell =
        std::env::var("OYATIE_FOUNDRY_BROWSER_SHELL_ORIGIN").expect("explicit shell origin");
    let directory = PathBuf::from(
        std::env::var("OYATIE_FOUNDRY_BROWSER_FIXTURE_DIRECTORY")
            .expect("explicit dedicated fixture directory"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let config = Config {
        listen_addr: "127.0.0.1:0".into(),
        action_log: directory.join("actions.sqlite"),
        denial_log: directory.join("denials.sqlite"),
        projection_store: directory.join("projection.sqlite"),
        tenants: vec![TENANT.into()],
        operators: vec![],
    };
    let binding = foundry_model_api::ModelBinding {
        tenant_id: TENANT.into(),
        store_id: "controlled-browser-model".into(),
        baseline_id: "compiled-record-baseline".into(),
    };
    let models = ModelRuntimeConfig::from_values(
        Some(
            &serde_json::json!([
                {"path":directory.join("model.sqlite"),"binding":binding}
            ])
            .to_string(),
        ),
        Some(&shell),
        Some("true"),
    )
    .unwrap()
    .unwrap();
    if !directory.join("model.sqlite").exists() {
        initialize_model(&config, &models, TENANT).unwrap();
    }
    let baseline = ModelBaseline::capture(
        &foundry_ontology_app::seed::registry_for(TENANT).unwrap(),
        TENANT,
    )
    .unwrap();
    let mut store =
        SqliteModelStore::open_existing(&directory.join("model.sqlite"), binding, baseline)
            .unwrap();
    let caller = ModelCaller {
        tenant_id: TENANT.into(),
        owner: USER.into(),
    };
    if store.head(&caller).unwrap().generation == 1 {
        let document = model::document(store.head(&caller).unwrap().document);
        let draft = store.create_draft(&caller, 1, document).unwrap();
        let prepared = store
            .prepare(&caller, draft.id, draft.version, draft.base_generation)
            .unwrap();
        store.publish(&caller, &prepared).unwrap();
    }
    drop(store);
    let iam = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let iam_origin = format!("http://localhost:{}", iam.local_addr().unwrap().port());
    let fake = Arc::new(FakeIam {
        active: AtomicBool::new(false),
        expires: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600,
        shell: shell.clone(),
    });
    let iam_router = Router::new()
        .route("/fixture/sign-in", get(|| async { Html("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>Controlled fixture identity</title><h1>Controlled browser fixture</h1><p>Fake IAM identity only. Foundry model storage and routes are native and durable. This does not prove passkey sign-in.</p><form method=\"post\" action=\"/fixture/session\"><button>Use controlled designer session</button></form></html>") }))
        .route("/fixture/session", post(sign_in))
        .route("/identity/v1/{tenant}/session", get(session))
        .route("/identity/v1/{tenant}/session:logout", post(logout))
        .with_state(fake);
    let sessions =
        HttpUserSessionAuthorizer::new(HumanSessionConfig::loopback_http(&iam_origin).unwrap())
            .unwrap();
    let state = compose_with_models(&config, Some(&models)).unwrap();
    let foundry = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let foundry_origin = format!("http://localhost:{}", foundry.local_addr().unwrap().port());
    let routes =
        router_with_model_control(Arc::new(state), Arc::new(sessions), Some(models.origin));
    let receipt = serde_json::json!({"purpose":"controlled browser fixture; fake IAM only; native durable model metadata and Foundry routes", "fake_iam":true, "native_durable_model":true, "passkey_proof":false, "publication_transport":false,
        "tenant":TENANT,"account":USER,"process_id":std::process::id(),"shell_origin":shell,"iam_origin":iam_origin,
        "foundry_origin":foundry_origin,"sign_in_url":format!("{iam_origin}/fixture/sign-in"),"directory":directory});
    std::fs::write(
        directory.join("startup.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    println!("{receipt}");
    let (iam_result, foundry_result) =
        tokio::join!(axum::serve(iam, iam_router), axum::serve(foundry, routes));
    iam_result.unwrap();
    foundry_result.unwrap();
}
