use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use foundry_ontology_app::{AppState, PolicyEnforcementPoint, router_with_user_sessions};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use http_body_util::BodyExt;
use iam_identity_user_session::{
    ActiveUserMembership, SessionRefusal, SessionView, UserSessionAuthorizer,
};
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, PolicyVersion};
use tower::ServiceExt;

pub const TOKEN: &str = "v1.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
pub const PATH: &str = "/v1/tenants/ten_acme/status";

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn view() -> SessionView {
    SessionView {
        tenant_id: "ten_acme".into(),
        user_id: "usr_alice".into(),
        expires_at: now() + 3600,
        roles: vec!["foundry-operator".into()],
    }
}

pub struct FakeSessions {
    pub answer: Mutex<Result<SessionView, SessionRefusal>>,
    pub calls: AtomicUsize,
    pub delay: std::time::Duration,
}

impl FakeSessions {
    pub fn new(answer: Result<SessionView, SessionRefusal>) -> Arc<Self> {
        Arc::new(Self {
            answer: Mutex::new(answer),
            calls: AtomicUsize::new(0),
            delay: std::time::Duration::ZERO,
        })
    }
}

impl UserSessionAuthorizer for FakeSessions {
    fn authorize<'a>(
        &'a self,
        presented: Option<&'a str>,
        tenant: &'a str,
        now: i64,
    ) -> Pin<Box<dyn Future<Output = Result<ActiveUserMembership, SessionRefusal>> + Send + 'a>>
    {
        Box::pin(async move {
            assert_eq!(presented, Some(TOKEN));
            self.calls.fetch_add(1, Ordering::SeqCst);
            let answer = self.answer.lock().unwrap().clone();
            tokio::time::sleep(self.delay).await;
            answer?.into_active_membership(tenant, now)
        })
    }
}

struct CountingLog {
    inner: Box<dyn RecordsLog + Send>,
    reads: Arc<AtomicUsize>,
}

impl RecordsLog for CountingLog {
    fn append(&mut self, envelope: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        self.inner.append(envelope)
    }
    fn replay(&self, tenant: &str, from: u64) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.replay(tenant, from)
    }
    fn head(&self, tenant: &str) -> Result<u64, RecordsLogError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.head(tenant)
    }
}

pub struct TrackingPdp {
    pub calls: AtomicUsize,
    pub unavailable: AtomicBool,
    inner: Arc<dyn PolicyDecisionPoint>,
}

impl PolicyDecisionPoint for TrackingPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        assert_eq!(request.principal.entity_id, "usr_alice");
        assert_eq!(request.action, "foundry.ontology.use");
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.unavailable.load(Ordering::SeqCst) {
            Err(PdpError::Evaluation {
                detail: "injected unavailable policy engine".into(),
            })
        } else {
            self.inner.authorize(request, entities)
        }
    }
    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}

pub fn instrument(
    mut state: AppState,
    sessions: Arc<dyn UserSessionAuthorizer>,
) -> (Router, Arc<AtomicUsize>, Arc<TrackingPdp>) {
    let reads = Arc::new(AtomicUsize::new(0));
    for tenant in state.tenants.values_mut() {
        let tenant = tenant.get_mut();
        let inner = std::mem::replace(
            &mut tenant.action_log,
            Box::new(
                foundry_records_sqlite_draft::SqliteRecordsLog::open(std::path::Path::new(
                    ":memory:",
                ))
                .unwrap(),
            ),
        );
        tenant.action_log = Box::new(CountingLog {
            inner,
            reads: reads.clone(),
        });
    }
    let pdp = Arc::new(TrackingPdp {
        calls: AtomicUsize::new(0),
        unavailable: AtomicBool::new(false),
        inner: PolicyEnforcementPoint::cedar(state.pep.loaded_policy_version().as_str()).unwrap(),
    });
    state.pep = PolicyEnforcementPoint::with_pdp(pdp.clone());
    (
        router_with_user_sessions(Arc::new(state), sessions),
        reads,
        pdp,
    )
}

pub async fn request(router: &Router, path: &str, cookies: &[&str]) -> Response<Body> {
    let mut request = Request::builder().uri(path);
    for cookie in cookies {
        request = request.header("cookie", *cookie);
    }
    request = request.header("authorization", "Bearer never-forward-this");
    router
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

pub fn cookie() -> String {
    format!("__Host-oya_session={TOKEN}")
}

pub async fn body(response: Response<Body>, expected: StatusCode) -> serde_json::Value {
    assert_eq!(response.status(), expected);
    assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}
