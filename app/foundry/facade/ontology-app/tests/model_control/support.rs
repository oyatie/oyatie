#![allow(dead_code)]
pub use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode},
};
pub use foundry_model_api::{control::*, *};
use foundry_ontology_app::{
    AppState, PolicyEnforcementPoint, model_control::ModelMutationOrigin, router_with_model_control,
};
use http_body_util::BodyExt;
pub use iam_identity_user_session::{
    ActiveUserMembership, SessionRefusal, SessionView, UserSessionAuthorizer,
};
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, PolicyVersion};
use std::future::Future;
use std::pin::Pin;
pub use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tower::ServiceExt;

#[path = "store.rs"]
mod store;
pub use store::{CountingStore, HistoryFault, publish_next};

pub const TOKEN: &str = "v1.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
pub const TENANT: &str = "ten_acme";
pub const WORKSPACE: &str = "/v1/tenants/ten_acme/model";
pub const DRAFTS: &str = "/v1/tenants/ten_acme/model/drafts";
pub const ORIGIN: &str = "https://shell.example";

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
pub fn view(roles: &[&str]) -> SessionView {
    SessionView {
        tenant_id: TENANT.into(),
        user_id: "usr_alice".into(),
        expires_at: now() + 3600,
        roles: roles.iter().map(|role| (*role).into()).collect(),
    }
}
pub fn binding() -> ModelBinding {
    ModelBinding {
        tenant_id: TENANT.into(),
        store_id: "fixture-model".into(),
        baseline_id: "fixture-baseline".into(),
    }
}
pub fn baseline() -> ModelBaseline {
    ModelBaseline::capture(
        &foundry_ontology_app::seed::registry_for(TENANT).unwrap(),
        TENANT,
    )
    .unwrap()
}
pub fn changed() -> ModelDocument {
    let mut document = baseline().document;
    let entity = document
        .entities
        .iter_mut()
        .find(|entity| entity.id == "ety_record")
        .unwrap();
    entity.revision += 1;
    entity
        .properties
        .push(foundry_model_api::conformance::property("notes", false));
    document
}
pub fn number(value: u64) -> ModelNumber {
    ModelNumber::new(value).unwrap()
}

pub struct FakeSessions {
    pub answer: Mutex<Result<SessionView, SessionRefusal>>,
    pub calls: AtomicUsize,
    pub delay: Mutex<std::time::Duration>,
    pub changed: tokio::sync::Notify,
    pub after_call: Mutex<Option<(usize, Result<SessionView, SessionRefusal>)>>,
}
impl FakeSessions {
    pub fn new(answer: Result<SessionView, SessionRefusal>) -> Arc<Self> {
        Arc::new(Self {
            answer: Mutex::new(answer),
            calls: AtomicUsize::new(0),
            delay: Mutex::new(std::time::Duration::ZERO),
            changed: tokio::sync::Notify::new(),
            after_call: Mutex::new(None),
        })
    }
}
impl UserSessionAuthorizer for FakeSessions {
    fn authorize<'a>(
        &'a self,
        presented: Option<&'a str>,
        tenant: &'a str,
        instant: i64,
    ) -> Pin<Box<dyn Future<Output = Result<ActiveUserMembership, SessionRefusal>> + Send + 'a>>
    {
        Box::pin(async move {
            assert_eq!(presented, Some(TOKEN));
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            let answer = self
                .after_call
                .lock()
                .unwrap()
                .as_ref()
                .filter(|(after, _)| call >= *after)
                .map(|(_, answer)| answer.clone())
                .unwrap_or_else(|| self.answer.lock().unwrap().clone());
            let delay = *self.delay.lock().unwrap();
            self.changed.notify_one();
            tokio::time::sleep(delay).await;
            answer?.into_active_membership(tenant, instant)
        })
    }
}
pub async fn wait_for_calls(sessions: &FakeSessions, count: usize) {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while sessions.calls.load(Ordering::SeqCst) < count {
            sessions.changed.notified().await;
        }
    })
    .await
    .unwrap();
}

pub struct TrackingPdp {
    pub calls: AtomicUsize,
    pub unavailable: AtomicBool,
    pub inner: Arc<dyn PolicyDecisionPoint>,
}
impl PolicyDecisionPoint for TrackingPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if request.action.starts_with("foundry.ontology.model.") {
            assert_eq!(request.resource.entity_type, "OntologyModel");
            assert_eq!(request.resource.entity_id, request.tenant_id);
            assert!(request.context.is_empty());
        }
        if self.unavailable.load(Ordering::SeqCst) {
            Err(PdpError::Evaluation {
                detail: "private storage metadata must not escape".into(),
            })
        } else {
            self.inner.authorize(request, entities)
        }
    }
    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}

pub struct Instrumented {
    pub app: Arc<AppState>,
    pub router: Router,
    pub sessions: Arc<FakeSessions>,
    pub pdp: Arc<TrackingPdp>,
    pub store_calls: Arc<AtomicUsize>,
    pub store_failure: Arc<AtomicBool>,
    pub oversized_response: Arc<AtomicBool>,
    pub history_fault: Arc<Mutex<Option<HistoryFault>>>,
    pub reference: ReferenceModelStore,
}
pub fn instrument(
    mut app: AppState,
    roles: &[&str],
    origin: Option<ModelMutationOrigin>,
) -> Instrumented {
    let reference = ReferenceModelStore::initialize(binding(), baseline()).unwrap();
    let store_calls = Arc::new(AtomicUsize::new(0));
    let store_failure = Arc::new(AtomicBool::new(false));
    let oversized_response = Arc::new(AtomicBool::new(false));
    let history_fault = Arc::new(Mutex::new(None));
    app.inject_established_model_store(
        TENANT,
        binding(),
        baseline(),
        Box::new(CountingStore {
            inner: reference.clone(),
            calls: store_calls.clone(),
            failure: store_failure.clone(),
            oversized: oversized_response.clone(),
            history_fault: history_fault.clone(),
        }),
    )
    .unwrap();
    store_calls.store(0, Ordering::SeqCst);
    let pdp = Arc::new(TrackingPdp {
        calls: AtomicUsize::new(0),
        unavailable: AtomicBool::new(false),
        inner: PolicyEnforcementPoint::cedar("psv-000001").unwrap(),
    });
    app.pep = PolicyEnforcementPoint::with_pdp(pdp.clone());
    let app = Arc::new(app);
    let sessions = FakeSessions::new(Ok(view(roles)));
    let router = router_with_model_control(app.clone(), sessions.clone(), origin);
    Instrumented {
        app,
        router,
        sessions,
        pdp,
        store_calls,
        store_failure,
        oversized_response,
        history_fault,
        reference,
    }
}
pub fn origin() -> ModelMutationOrigin {
    ModelMutationOrigin::https(ORIGIN).unwrap()
}
pub fn request(method: &str, path: &str, body: impl Into<Body>) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("cookie", format!("__Host-oya_session={TOKEN}"))
        .header("origin", ORIGIN)
        .header("content-type", "application/json")
        .body(body.into())
        .unwrap()
}
pub async fn send(router: &Router, request: Request<Body>) -> Response<Body> {
    router.clone().oneshot(request).await.unwrap()
}
pub async fn body(response: Response<Body>, status: StatusCode) -> Vec<u8> {
    assert_eq!(response.status(), status);
    assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
    response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}
pub async fn refusal(response: Response<Body>, status: StatusCode, code: ModelRefusalCode) {
    let bytes = body(response, status).await;
    assert_eq!(ModelRefusal::decode(&bytes).unwrap().error, code);
    assert!(!String::from_utf8_lossy(&bytes).contains("private storage"));
}
pub async fn create(router: &Router) -> ModelDraftWire {
    let input = CreateModelDraft {
        expected_generation: number(1),
        document: changed(),
    };
    let bytes = body(
        send(router, request("POST", DRAFTS, input.encode().unwrap())).await,
        StatusCode::CREATED,
    )
    .await;
    ModelDraftWire::decode(&bytes).unwrap()
}
