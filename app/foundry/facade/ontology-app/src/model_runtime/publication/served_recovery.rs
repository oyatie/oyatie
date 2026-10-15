use super::{faults::Fault, support::*};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use foundry_model_api::{control::*, *};
use http_body_util::BodyExt;
use iam_identity_user_session::{
    ActiveUserMembership, SessionRefusal, SessionView, UserSessionAuthorizer,
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tower::ServiceExt;
const TOKEN: &str = "v1.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
struct Sessions(Mutex<Vec<String>>);
impl UserSessionAuthorizer for Sessions {
    fn authorize<'a>(
        &'a self,
        _: Option<&'a str>,
        tenant: &'a str,
        instant: i64,
    ) -> Pin<Box<dyn Future<Output = Result<ActiveUserMembership, SessionRefusal>> + Send + 'a>>
    {
        Box::pin(async move {
            SessionView {
                tenant_id: TENANT.into(),
                user_id: caller().owner,
                expires_at: i64::MAX,
                roles: self.0.lock().unwrap().clone(),
            }
            .into_active_membership(tenant, instant)
        })
    }
}
async fn send(
    router: &Router,
    method: &str,
    path: &str,
    body: impl Into<Body>,
    operator: bool,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("origin", "https://shell.example")
        .header("content-type", "application/json");
    if operator {
        request = request.header("authorization", "Bearer publication-operator");
    } else {
        request = request.header("cookie", format!("__Host-oya_session={TOKEN}"));
    }
    router
        .clone()
        .oneshot(request.body(body.into()).unwrap())
        .await
        .unwrap()
}
async fn lifecycle(router: &Router) -> ModelLifecycle {
    let response = send(
        router,
        "GET",
        &format!("/v1/tenants/{TENANT}/model/lifecycle"),
        Body::empty(),
        false,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    ModelLifecycle::decode(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

#[tokio::test]
async fn served_unknown_outcome_freezes_action_and_migration_and_requires_authorized_exact_retry() {
    for (fault_mode, committed) in [
        (Fault::CommitAndLoseReadback, true),
        (Fault::RejectAndLoseReadback, false),
    ] {
        let files = Files::new();
        let (mut app, fault) = files.compose(fault_mode);
        let saved = draft(app.tenants.get_mut(TENANT).unwrap().get_mut());
        let app = Arc::new(app);
        let sessions = Arc::new(Sessions(Mutex::new(vec!["foundry-model-publisher".into()])));
        let router = crate::router_with_model_control(
            app.clone(),
            sessions.clone(),
            Some(files.models.origin.clone()),
        );
        let path = format!("/v1/tenants/{TENANT}/model/drafts/{}/publish", saved.id);
        let input = input(&saved).encode().unwrap();
        assert_eq!(
            send(&router, "POST", &path, input.clone(), false)
                .await
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            send(&router, "GET", "/readyz", Body::empty(), false)
                .await
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            send(&router, "GET", "/healthz", Body::empty(), false)
                .await
                .status(),
            StatusCode::OK
        );
        let tenant = app.tenants[TENANT].lock().await;
        assert_eq!(tenant.observed_model_generation(), Some(1));
        assert!(tenant.publication_unknown.is_some());
        drop(tenant);
        let action = r#"{"object_ref":"ent_blocked","action_type":"aty_record_write","idempotency_key":"blocked-write","occurred_at_epoch_seconds":1,"properties":{"name":"Must not append"}}"#;
        assert_eq!(
            send(&router, "POST", "/v1/actions", action, true)
                .await
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let migration = format!(
            r#"{{"tenant_id":"{TENANT}","entity_type":"ety_record","from_revision":1,"to_revision":2,"action_type":"aty_record_write","audit_event_type":"aet_upcast","declared_at_epoch_seconds":1,"transforms":[{{"kind":"copy_as","from":"note","to":"nickname"}}]}}"#
        );
        assert_eq!(
            send(&router, "POST", "/v1/migrations/run", migration, true)
                .await
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            send(
                &router,
                "POST",
                &path.replace(&format!("/{}/publish", saved.id), "/999/publish"),
                input.clone(),
                false
            )
            .await
            .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        let tenant = app.tenants[TENANT].lock().await;
        assert_eq!(tenant.action_log.head(TENANT).unwrap(), 0);
        assert_eq!(tenant.denial_log.head(TENANT).unwrap(), 0);
        drop(tenant);
        *fault.lock().unwrap() = Fault::None;
        let unknown = lifecycle(&router).await;
        assert_eq!(unknown.phase, ModelLifecyclePhase::CommitUnknown);
        assert_eq!(unknown.observed_generation.unwrap().get(), 1);
        assert_eq!(
            unknown.desired_generation.get(),
            if committed { 2 } else { 1 }
        );
        *sessions.0.lock().unwrap() = vec!["foundry-model-author".into()];
        assert_eq!(
            send(&router, "POST", &path, input.clone(), false)
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
        assert!(
            app.tenants[TENANT]
                .lock()
                .await
                .publication_unknown
                .is_some()
        );
        *sessions.0.lock().unwrap() = vec!["foundry-model-publisher".into()];
        let recovered = send(&router, "POST", &path, input.clone(), false).await;
        assert_eq!(
            recovered.status(),
            if committed {
                StatusCode::OK
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            }
        );
        assert!(
            app.tenants[TENANT]
                .lock()
                .await
                .publication_unknown
                .is_none()
        );
        assert!(lifecycle(&router).await.ready);
        if !committed {
            assert_eq!(
                send(&router, "POST", &path, input, false).await.status(),
                StatusCode::OK
            );
        }
        assert_eq!(
            lifecycle(&router).await.observed_generation.unwrap().get(),
            2
        );
    }
}
