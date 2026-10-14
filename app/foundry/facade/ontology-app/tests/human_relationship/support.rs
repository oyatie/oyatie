use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use foundry_ontology_app::{AppState, PolicyEnforcementPoint, router_with_user_sessions};
use foundry_projection_draft::{
    AppliedEntry, EntryOutcome, KeyDesignations, ProjectedLink, ProjectedObject,
};
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint};
use serde_json::{Value, json};
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, Decision, PolicyVersion};
use tower::ServiceExt;

use crate::facade_support::{Fixture, TENANT};
use crate::graph_support::{GraphFixture, links, objects};
use crate::human_support::{FakeSessions, body, cookie, view};

pub struct HumanPdp {
    pub denied: Mutex<Option<&'static str>>,
    pub unavailable: Mutex<bool>,
    pub resources: Mutex<Vec<String>>,
    inner: Arc<dyn PolicyDecisionPoint>,
}
impl PolicyDecisionPoint for HumanPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        assert!(matches!(
            request.principal.entity_id.as_str(),
            "usr_alice" | "prn_alice"
        ));
        self.resources
            .lock()
            .unwrap()
            .push(request.resource.entity_id.clone());
        if *self.unavailable.lock().unwrap() {
            return Err(PdpError::Evaluation {
                detail: "injected PDP failure".into(),
            });
        }
        let mut outcome = self.inner.authorize(request, entities)?;
        if *self.denied.lock().unwrap() == Some(request.resource.entity_id.as_str()) {
            outcome.response.decision = Decision::Deny;
            outcome.audit.decision = Decision::Deny;
        }
        Ok(outcome)
    }
    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}

pub struct HumanGraph {
    pub fixture: Fixture,
    pub state: Arc<AppState>,
    pub sessions: Arc<FakeSessions>,
    pub policy: Arc<HumanPdp>,
    pub router: Router,
}
impl HumanGraph {
    pub fn new(case: &str) -> Self {
        let GraphFixture {
            fixture,
            state,
            session,
            ..
        } = GraphFixture::new(case, objects(), links());
        drop(session);
        let mut state = Arc::try_unwrap(state).ok().unwrap();
        let policy = Arc::new(HumanPdp {
            denied: Mutex::new(None),
            unavailable: Mutex::new(false),
            resources: Mutex::new(vec![]),
            inner: PolicyEnforcementPoint::cedar(state.pep.loaded_policy_version().as_str())
                .unwrap(),
        });
        state.pep = PolicyEnforcementPoint::with_pdp(policy.clone());
        let state = Arc::new(state);
        let sessions = FakeSessions::new(Ok(view()));
        let router = router_with_user_sessions(state.clone(), sessions.clone());
        Self {
            fixture,
            state,
            sessions,
            policy,
            router,
        }
    }
    pub async fn walk(&self, request: &Value, status: StatusCode) -> Value {
        let response = self
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/tenants/ten_acme/search-around")
                    .header("cookie", cookie())
                    .header("content-type", "application/json")
                    .body(Body::from(request.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        body(response, status).await
    }
    pub async fn replace(&self, objects: Vec<ProjectedObject>, links: Vec<ProjectedLink>) {
        let mut tenant = self.state.tenants[TENANT].lock().await;
        tenant.projection_store.reset_tenant(TENANT).unwrap();
        tenant
            .projection_store
            .apply(
                AppliedEntry {
                    tenant_id: TENANT.into(),
                    ordinal: 1,
                    outcome: EntryOutcome::Applied { objects, links },
                },
                &KeyDesignations::default(),
            )
            .unwrap();
    }
}
pub fn request(key: &str) -> Value {
    json!({"idempotency_key":key,"root":{"entity_type":"ety_record","object_ref":"ent_record","revision":1},
        "link_type":"lty_member","direction":"outbound","observed_at_epoch_seconds":1700000000})
}
