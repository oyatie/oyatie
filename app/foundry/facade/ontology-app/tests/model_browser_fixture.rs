#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use foundry_model_api::{ModelBaseline, ModelCaller, ModelDocument, ModelDraft, ModelStore};
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_ontology_app::{
    Config, compose_with_models,
    human_session::{HttpUserSessionAuthorizer, HumanSessionConfig},
    model_runtime::{ModelRuntimeConfig, initialize_model},
    router_with_model_control,
};
use iam_identity_user_session::SessionView;
use std::{
    collections::BTreeSet,
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
    serve_model(model::document, "controlled-browser-model", false).await;
}

#[tokio::test]
#[ignore = "explicit native scale browser fixture; requires shell origin and a nonexistent dedicated directory"]
async fn serve_real_scale_model_for_browser() {
    serve_model(model::scale_document, "controlled-scale-graph-model", true).await;
}

fn binding(store_id: &str) -> foundry_model_api::ModelBinding {
    foundry_model_api::ModelBinding {
        tenant_id: TENANT.into(),
        store_id: store_id.into(),
        baseline_id: "compiled-record-baseline".into(),
    }
}

async fn serve_model(
    document: fn(foundry_model_api::ModelDocument) -> foundry_model_api::ModelDocument,
    store_id: &str,
    fresh: bool,
) {
    let shell =
        std::env::var("OYATIE_FOUNDRY_BROWSER_SHELL_ORIGIN").expect("explicit shell origin");
    let directory = PathBuf::from(
        std::env::var("OYATIE_FOUNDRY_BROWSER_FIXTURE_DIRECTORY")
            .expect("explicit dedicated fixture directory"),
    );
    if fresh {
        std::fs::create_dir(&directory).expect("scale fixture requires a nonexistent directory");
    } else {
        std::fs::create_dir_all(&directory).unwrap();
    }
    let config = Config {
        listen_addr: "127.0.0.1:0".into(),
        action_log: directory.join("actions.sqlite"),
        denial_log: directory.join("denials.sqlite"),
        projection_store: directory.join("projection.sqlite"),
        tenants: vec![TENANT.into()],
        operators: vec![],
    };
    let binding = binding(store_id);
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
    let runtime = foundry_ontology_app::seed::registry_for(TENANT).unwrap();
    let baseline = ModelBaseline::capture(&runtime, TENANT).unwrap();
    let mut store =
        SqliteModelStore::open_existing(&directory.join("model.sqlite"), binding, baseline)
            .unwrap();
    let caller = ModelCaller {
        tenant_id: TENANT.into(),
        owner: USER.into(),
    };
    if store.head(&caller).unwrap().generation == 1 {
        let document = document(store.head(&caller).unwrap().document);
        let draft = store.create_draft(&caller, 1, document).unwrap();
        let prepared = store
            .prepare(&caller, draft.id, draft.version, draft.base_generation)
            .unwrap();
        if fresh {
            assert_scale_document(
                &ModelBaseline::capture(prepared.model_registry(), TENANT)
                    .unwrap()
                    .document,
            );
        }
        store.publish(&caller, &prepared).unwrap();
    }
    let head = store.head(&caller).unwrap();
    if fresh {
        assert_eq!(head.generation, 2);
        assert_scale_document(&head.document);
        let registry = store
            .history(&caller)
            .unwrap()
            .registry_with_runtime_baseline(&runtime)
            .unwrap();
        assert_scale_document(&ModelBaseline::capture(&registry, TENANT).unwrap().document);
    }
    let editable = fresh.then(|| editable_scale_draft(&mut store, &caller));
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
    let mut receipt = serde_json::json!({"purpose":"controlled browser fixture; fake IAM only; native durable model metadata and Foundry routes", "fake_iam":true, "native_durable_model":true, "passkey_proof":false, "publication_transport":false,
        "tenant":TENANT,"account":USER,"process_id":std::process::id(),"shell_origin":shell,"iam_origin":iam_origin,
        "foundry_origin":foundry_origin,"sign_in_url":format!("{iam_origin}/fixture/sign-in"),"directory":directory,
        "store_id":store_id,"model_generation":head.generation,"entity_type_count":head.document.entities.len(),"link_type_count":head.document.links.len()});
    if let Some(draft) = editable {
        receipt["editable_draft"] = serde_json::to_value(draft).unwrap();
    }
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

fn editable_scale_draft(store: &mut dyn ModelStore, caller: &ModelCaller) -> ModelDraft {
    let history = store.history(caller).unwrap();
    let head = store.head(caller).unwrap();
    assert_eq!(head.generation, 2);
    assert_scale_document(&head.document);
    let draft = store
        .create_draft(caller, head.generation, head.document.clone())
        .unwrap();
    assert_eq!(draft.tenant_id, caller.tenant_id);
    assert_eq!(draft.owner, caller.owner);
    assert_eq!(draft.version, 1);
    assert_eq!(draft.base_generation, head.generation);
    assert_eq!(draft.document, head.document);
    assert_eq!(draft.publication, None);
    assert_eq!(store.open_draft(caller, draft.id).unwrap(), draft);
    assert_eq!(store.head(caller).unwrap(), head);
    assert_eq!(store.history(caller).unwrap(), history);
    draft
}

fn assert_scale_document(model: &ModelDocument) {
    assert_eq!((model.entities.len(), model.links.len()), (52, 148));
    let types: BTreeSet<_> = model.entities.iter().map(|row| row.id.as_str()).collect();
    let links: BTreeSet<_> = model.links.iter().map(|row| row.id.as_str()).collect();
    assert_eq!((types.len(), links.len()), (52, 148));
    let mut related = BTreeSet::from(["ety_employee"]);
    for row in &model.links {
        assert!(types.contains(row.from_entity_type.as_str()));
        assert!(types.contains(row.to_entity_type.as_str()));
        if row.from_entity_type == "ety_employee" || row.to_entity_type == "ety_employee" {
            related.extend([row.from_entity_type.as_str(), row.to_entity_type.as_str()]);
        }
    }
    assert_eq!(related.len(), 8);
    assert_eq!(
        model
            .links
            .iter()
            .filter(|row| related.contains(row.from_entity_type.as_str())
                && related.contains(row.to_entity_type.as_str()))
            .count(),
        11
    );
    assert_eq!(
        model
            .links
            .iter()
            .filter(|row| !related.contains(row.from_entity_type.as_str())
                && !related.contains(row.to_entity_type.as_str()))
            .count(),
        137
    );
}
