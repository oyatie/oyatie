use axum::http::StatusCode;
use iam_identity_user_session::SessionRefusal;
use std::sync::atomic::Ordering;

use crate::facade_support::Fixture;
use crate::human_support::*;

#[tokio::test]
async fn fake_session_is_admitted_before_real_transport_and_denials_never_observe_sources() {
    let fixture = Fixture::new("human-session-denials");
    let sessions = FakeSessions::new(Ok(view()));
    let (router, reads, pdp) = instrument(fixture.state(), sessions.clone());
    for cookies in [
        vec![],
        vec!["__Host-oya_session=malformed".to_owned()],
        vec![cookie(), cookie()],
        vec![format!("{}; {}", cookie(), cookie())],
        vec![format!("__Host-oya_session={}=", TOKEN)],
    ] {
        let values: Vec<&str> = cookies.iter().map(|s| s.as_str()).collect();
        body(
            request(&router, PATH, &values).await,
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }
    assert_eq!(sessions.calls.load(Ordering::SeqCst), 0);
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 0);
    assert_eq!(reads.load(Ordering::SeqCst), 0);
    for (refusal, status) in [
        (SessionRefusal::Unauthenticated, StatusCode::UNAUTHORIZED),
        (SessionRefusal::InvalidCredential, StatusCode::UNAUTHORIZED),
        (SessionRefusal::Expired, StatusCode::UNAUTHORIZED),
        (SessionRefusal::InvalidSubject, StatusCode::UNAUTHORIZED),
        (SessionRefusal::TenantMismatch, StatusCode::FORBIDDEN),
        (SessionRefusal::MissingMembership, StatusCode::FORBIDDEN),
        (SessionRefusal::InactiveMembership, StatusCode::FORBIDDEN),
        (SessionRefusal::AmbiguousMembership, StatusCode::FORBIDDEN),
        (
            SessionRefusal::MembershipUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
    ] {
        *sessions.answer.lock().unwrap() = Err(refusal);
        body(request(&router, PATH, &[&cookie()]).await, status).await;
    }
    let mut expired = view();
    expired.expires_at = now();
    *sessions.answer.lock().unwrap() = Ok(expired);
    body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    *sessions.answer.lock().unwrap() = Ok(view());
    body(
        request(&router, "/v1/tenants/ten_other/status", &[&cookie()]).await,
        StatusCode::FORBIDDEN,
    )
    .await;
    let mut unserved = view();
    unserved.tenant_id = "ten_unserved".into();
    *sessions.answer.lock().unwrap() = Ok(unserved);
    body(
        request(&router, "/v1/tenants/ten_unserved/status", &[&cookie()]).await,
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 0);
    assert_eq!(reads.load(Ordering::SeqCst), 0);

    for roles in [vec![], vec!["ungranted-role".into()]] {
        let mut roleless = view();
        roleless.roles = roles;
        *sessions.answer.lock().unwrap() = Ok(roleless);
        body(
            request(&router, PATH, &[&cookie()]).await,
            StatusCode::FORBIDDEN,
        )
        .await;
    }
    *sessions.answer.lock().unwrap() = Ok(view());
    pdp.unavailable.store(true, Ordering::SeqCst);
    body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 3);
    assert_eq!(
        reads.load(Ordering::SeqCst),
        0,
        "credential, tenant and policy denial all precede observation"
    );
}

#[tokio::test]
async fn fresh_cedar_reads_exact_tenant_and_revoked_roles_deny_same_user() {
    let fixture = Fixture::new("human-real-cedar");
    let sessions = FakeSessions::new(Ok(view()));
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    let state = foundry_ontology_app::compose(&config).unwrap();
    let (router, reads, pdp) = instrument(state, sessions.clone());
    let first = body(
        request(&router, PATH, &[&format!("unrelated=value; {}", cookie())]).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(first["projection_lag"], 0);
    assert_eq!(first["served_tenants"], 2);
    assert_eq!(first["entity_types"], serde_json::json!(["ety_record"]));
    crate::out_of_band::append_for(
        &fixture.action_log_path(),
        "ten_other",
        "idem_other_human_status",
    );
    let other = body(request(&router, PATH, &[&cookie()]).await, StatusCode::OK).await;
    assert_eq!(other["projection_lag"], 1);
    assert_eq!(
        other["tenant"]["log_head"], 0,
        "only the authenticated tenant contributes its tenant facts"
    );
    crate::out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_human_status");
    let second = body(request(&router, PATH, &[&cookie()]).await, StatusCode::OK).await;
    assert_eq!(second["projection_lag"], 2);
    assert_eq!(second["tenant"]["log_head"], 1);
    let observations = reads.load(Ordering::SeqCst);
    assert!(observations > 0);
    let mut revoked_role = view();
    revoked_role.roles.clear();
    *sessions.answer.lock().unwrap() = Ok(revoked_role);
    body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::FORBIDDEN,
    )
    .await;
    assert_eq!(
        pdp.calls.load(Ordering::SeqCst),
        4,
        "same user/cache key with fresh entity facts is denied"
    );
    assert_eq!(reads.load(Ordering::SeqCst), observations);
    *sessions.answer.lock().unwrap() = Err(SessionRefusal::Unauthenticated);
    body(
        request(&router, PATH, &[&cookie()]).await,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    assert_eq!(
        sessions.calls.load(Ordering::SeqCst),
        5,
        "each request resolves fresh identity; logout cannot reuse a prior result"
    );
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 4);
    assert_eq!(reads.load(Ordering::SeqCst), observations);
}

#[tokio::test]
async fn default_operator_router_does_not_enable_human_access() {
    let fixture = Fixture::new("human-default-disabled");
    let router = foundry_ontology_app::router(fixture.state());
    assert_eq!(
        request(&router, PATH, &[&cookie()]).await.status(),
        StatusCode::NOT_FOUND
    );
}
