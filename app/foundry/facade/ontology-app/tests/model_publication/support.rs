#![allow(dead_code)]
use crate::model_support::*;
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_ontology_app::{
    AppState, compose_with_models,
    model_runtime::{ModelRuntimeConfig, ModelStoreBinding},
    router_with_model_control,
};

pub const LIFECYCLE: &str = "/v1/tenants/ten_acme/model/lifecycle";
pub struct NativeFixture {
    pub fixture: crate::facade_support::Fixture,
    pub models: ModelRuntimeConfig,
}
pub struct NativeState {
    pub router: Router,
    pub app: Arc<AppState>,
    pub sessions: Arc<FakeSessions>,
}
impl NativeFixture {
    pub fn new(case: &str) -> Self {
        let fixture = crate::facade_support::Fixture::new(case);
        let path = fixture.config().action_log.with_extension("model.sqlite");
        SqliteModelStore::initialize(&path, binding(), baseline()).unwrap();
        Self {
            fixture,
            models: ModelRuntimeConfig {
                stores: vec![ModelStoreBinding {
                    path,
                    binding: binding(),
                }],
                origin: origin(),
            },
        }
    }
    pub fn state(&self, roles: &[&str]) -> NativeState {
        let app =
            Arc::new(compose_with_models(&self.fixture.config(), Some(&self.models)).unwrap());
        let sessions = FakeSessions::new(Ok(view(roles)));
        let router = router_with_model_control(app.clone(), sessions.clone(), Some(origin()));
        NativeState {
            router,
            app,
            sessions,
        }
    }
}
impl Drop for NativeFixture {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ =
                std::fs::remove_file(format!("{}{suffix}", self.models.stores[0].path.display()));
        }
    }
}
pub fn publication_request(saved: &ModelDraftWire) -> Request<Body> {
    request(
        "POST",
        &format!("{DRAFTS}/{}/publish", saved.id.get()),
        PrepareModelDraft {
            expected_version: saved.version,
            expected_base: saved.base_generation,
        }
        .encode()
        .unwrap(),
    )
}
pub async fn publish(state: &NativeState, saved: &ModelDraftWire) -> ModelPublished {
    ModelPublished::decode(
        &body(
            send(&state.router, publication_request(saved)).await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap()
}
pub async fn lifecycle(state: &NativeState) -> ModelLifecycle {
    ModelLifecycle::decode(
        &body(
            send(&state.router, request("GET", LIFECYCLE, Body::empty())).await,
            StatusCode::OK,
        )
        .await,
    )
    .unwrap()
}
