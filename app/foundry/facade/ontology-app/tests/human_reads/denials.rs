use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use iam_identity_user_session::SessionRefusal;
use tower::ServiceExt;

use crate::facade_support::{Fixture, Session, WRITE_BODY, write_a_record};
use crate::human_support::{FakeSessions, body, cookie, now, request, view};
use crate::read_support::{SUFFIXES, instrument_reads};

#[tokio::test]
async fn credentials_and_membership_refuse_before_every_human_read() {
    let fixture = Fixture::new("human-read-denials");
    let sessions = FakeSessions::new(Ok(view()));
    let (router, log_reads, store_reads, pdp) = instrument_reads(fixture.state(), sessions.clone());
    for suffix in SUFFIXES {
        let path = format!("/v1/tenants/ten_acme{suffix}");
        for cookies in [
            vec![],
            vec!["__Host-oya_session=malformed".to_owned()],
            vec![cookie(), cookie()],
            vec![format!("{}; {}", cookie(), cookie())],
            vec![format!("{}=", cookie())],
        ] {
            let values: Vec<&str> = cookies.iter().map(String::as_str).collect();
            let refusal = body(
                request(&router, &path, &values).await,
                StatusCode::UNAUTHORIZED,
            )
            .await;
            assert_eq!(refusal["gate"], "session");
        }
        let operator_only = Request::builder()
            .uri(&path)
            .header(
                "authorization",
                format!("Bearer {}", fixture.operator_token()),
            )
            .body(Body::empty())
            .unwrap();
        body(
            router.clone().oneshot(operator_only).await.unwrap(),
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }
    assert_eq!(sessions.calls.load(Ordering::SeqCst), 0);
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 0);
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
        for suffix in SUFFIXES {
            body(
                request(
                    &router,
                    &format!("/v1/tenants/ten_acme{suffix}"),
                    &[&cookie()],
                )
                .await,
                status,
            )
            .await;
        }
    }
    let mut expired = view();
    expired.expires_at = now();
    *sessions.answer.lock().unwrap() = Ok(expired);
    for suffix in SUFFIXES {
        body(
            request(
                &router,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::UNAUTHORIZED,
        )
        .await;
    }
    let mut unserved = view();
    unserved.tenant_id = "ten_unserved".into();
    *sessions.answer.lock().unwrap() = Ok(unserved);
    for suffix in SUFFIXES {
        body(
            request(
                &router,
                &format!("/v1/tenants/ten_unserved{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::FORBIDDEN,
        )
        .await;
    }
    assert_eq!(pdp.calls.load(Ordering::SeqCst), 0);
    assert_eq!(log_reads.load(Ordering::SeqCst), 0);
    assert_eq!(store_reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn revoked_roles_and_pdp_fault_refuse_before_query_or_sources() {
    let fixture = Fixture::new("human-read-revocation");
    write_a_record(&fixture, "ent_alpha", "idem_revoked_read").await;
    let sessions = FakeSessions::new(Ok(view()));
    let (router, log_reads, store_reads, pdp) = instrument_reads(fixture.state(), sessions.clone());
    for suffix in SUFFIXES {
        body(
            request(
                &router,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::OK,
        )
        .await;
    }
    let before = (
        log_reads.load(Ordering::SeqCst),
        store_reads.load(Ordering::SeqCst),
    );
    assert!(
        before.0 > 0 && before.1 > 0,
        "healthy native reads must reach both sources"
    );
    let mut revoked = view();
    revoked.roles.clear();
    *sessions.answer.lock().unwrap() = Ok(revoked);
    for suffix in SUFFIXES.into_iter().chain([
        "/objects?type=ety_record&revision=invalid",
        "/objects/ent_alpha?type=ety_record&revision=invalid",
    ]) {
        let refusal = body(
            request(
                &router,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::FORBIDDEN,
        )
        .await;
        assert_eq!(refusal["gate"], "authorization");
    }
    *sessions.answer.lock().unwrap() = Ok(view());
    pdp.unavailable.store(true, Ordering::SeqCst);
    for suffix in SUFFIXES.into_iter().chain([
        "/objects?type=ety_record&revision=invalid",
        "/objects/ent_alpha?type=ety_record&revision=invalid",
    ]) {
        body(
            request(
                &router,
                &format!("/v1/tenants/ten_acme{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::FORBIDDEN,
        )
        .await;
    }
    assert_eq!(
        (
            log_reads.load(Ordering::SeqCst),
            store_reads.load(Ordering::SeqCst)
        ),
        before
    );
    assert_eq!(
        sessions.calls.load(Ordering::SeqCst),
        SUFFIXES.len() * 3 + 4,
        "every read rechecks IAM, including warm native decisions"
    );
}

#[tokio::test]
async fn foreign_present_and_absent_objects_are_indistinguishable() {
    let fixture = Fixture::new("human-read-foreign");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    let state = Arc::new(foundry_ontology_app::compose(&config).unwrap());
    let operator = Session::from_shared(state.clone());
    assert_eq!(
        operator
            .post(Some(fixture.foreign_token()), WRITE_BODY)
            .await
            .0,
        StatusCode::OK
    );
    let sessions = FakeSessions::new(Ok(view()));
    let router = foundry_ontology_app::router_with_user_sessions(state, sessions.clone());
    for (tenant, expected) in [
        ("ten_acme", StatusCode::NOT_FOUND),
        ("ten_other", StatusCode::FORBIDDEN),
    ] {
        let present = body(
            request(
                &router,
                &format!("/v1/tenants/{tenant}/objects/ent_alpha?type=ety_record&revision=1"),
                &[&cookie()],
            )
            .await,
            expected,
        )
        .await;
        let absent = body(
            request(
                &router,
                &format!("/v1/tenants/{tenant}/objects/ent_absent?type=ety_record&revision=1"),
                &[&cookie()],
            )
            .await,
            expected,
        )
        .await;
        assert_eq!(present, absent);
    }
    for suffix in SUFFIXES {
        body(
            request(
                &router,
                &format!("/v1/tenants/ten_other{suffix}"),
                &[&cookie()],
            )
            .await,
            StatusCode::FORBIDDEN,
        )
        .await;
    }
    assert_eq!(
        body(
            request(
                &router,
                "/v1/tenants/ten_acme/objects/ent_alpha/history",
                &[&cookie()]
            )
            .await,
            StatusCode::OK
        )
        .await,
        serde_json::json!([])
    );
    assert_eq!(
        body(
            request(&router, "/v1/tenants/ten_acme/audit", &[&cookie()]).await,
            StatusCode::OK
        )
        .await,
        serde_json::json!([])
    );
}
