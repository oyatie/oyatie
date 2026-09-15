#[path = "failing_log/mod.rs"]
#[allow(dead_code)]
mod failing_log;
#[path = "failing_store/mod.rs"]
mod failing_store;
#[path = "out_of_band/mod.rs"]
mod out_of_band;
#[path = "facade_support/mod.rs"]
mod support;

use axum::http::StatusCode;
use foundry_ontology_app::{Caller, CallerVerifier, compose, metrics::prometheus_text};
use foundry_submission_draft::{ActionSubmitter, SubmitError, SubmitRequest};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use support::{Fixture, TENANT, WRITE_BODY, value_of};

fn request() -> SubmitRequest {
    serde_json::from_str(WRITE_BODY).unwrap()
}

#[tokio::test]
async fn typed_submission_cannot_bypass_the_served_request_size_limit() {
    let fixture = Fixture::new("submission-review-size");
    let mut request = request();
    request
        .properties
        .insert("name".into(), "x".repeat(2 * 1024 * 1024));
    let body = serde_json::to_string(&request).unwrap();
    let (status, _) = fixture
        .session()
        .post(Some(fixture.operator_token()), &body)
        .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(fixture.log_head(), 0);
    let result = fixture
        .state()
        .submit(fixture.operator_token(), request)
        .await;
    assert!(
        matches!(result, Err(SubmitError::Surface { .. })),
        "{result:?}"
    );
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
}

#[tokio::test]
async fn typed_limit_counts_json_escape_expansion_and_accepts_exact_http_boundary() {
    let fixture = Fixture::new("submission-review-boundary");
    let state = fixture.state();
    let mut exact = request();
    exact.properties.insert("name".into(), String::new());
    let overhead = serde_json::to_vec(&exact).unwrap().len();
    exact
        .properties
        .insert("name".into(), "x".repeat(2 * 1024 * 1024 - overhead));
    let body = serde_json::to_string(&exact).unwrap();
    assert_eq!(body.len(), 2 * 1024 * 1024);
    let first = state
        .submit(fixture.operator_token(), exact.clone())
        .await
        .unwrap();
    assert_eq!(first.ordinal, 1);
    assert!(!first.deduplicated);
    let (status, body) = fixture
        .session()
        .post(Some(fixture.operator_token()), &body)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.contains("\"deduplicated\":true"), "{body}");
    exact.properties.get_mut("name").unwrap().push('x');
    assert!(matches!(
        state.submit(fixture.operator_token(), exact).await,
        Err(SubmitError::Surface { .. })
    ));
    let mut escaped = request();
    escaped
        .properties
        .insert("name".into(), "\n".repeat(1024 * 1024));
    assert!(serde_json::to_vec(&escaped).unwrap().len() > 2 * 1024 * 1024);
    assert!(matches!(
        state.submit(fixture.operator_token(), escaped).await,
        Err(SubmitError::Surface { .. })
    ));
    assert_eq!(fixture.log_head(), 1);
}

struct ChangingVerifier(Arc<AtomicUsize>);
impl CallerVerifier for ChangingVerifier {
    fn verify(&self, _: Option<&str>) -> Option<Caller> {
        let invocation = self.0.fetch_add(1, Ordering::SeqCst);
        Some(Caller {
            tenant_id: if invocation == 0 { TENANT } else { "ten_other" }.into(),
            principal_id: "prn_verified".into(),
            roles: vec!["foundry-operator".into()],
        })
    }
}

#[tokio::test]
async fn tenant_constraint_uses_the_same_fresh_identity_as_the_write_and_retry() {
    let fixture = Fixture::new("submission-review-tenant");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    let mut state = compose(&config).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    state.verifier = Box::new(ChangingVerifier(calls.clone()));
    state
        .submit_in_tenant("test-token", TENANT, request())
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        state
            .submit_in_tenant("test-token", TENANT, request())
            .await,
        Err(SubmitError::TenantMismatch)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    for (tenant, expected) in [(TENANT, 1), ("ten_other", 0)] {
        let state = state.tenants[tenant].lock().await;
        assert_eq!(state.action_log.head(tenant).unwrap(), expected);
        assert_eq!(state.denial_log.head(tenant).unwrap(), 0);
    }
}

#[tokio::test]
async fn durable_mirror_refusal_stays_accepted_and_lag_survives_a_deduplicated_retry() {
    let fixture = Fixture::new("submission-review-mirror");
    let mut state = fixture.state();
    state
        .tenants
        .get_mut(TENANT)
        .unwrap()
        .get_mut()
        .projection_store = Box::new(failing_store::ApplyRefusingStore::refusing_the_first(1));
    let first = state
        .submit(fixture.operator_token(), request())
        .await
        .unwrap();
    assert_eq!(first.outcome, "applied");
    assert!(!first.deduplicated);
    let retry = state
        .submit(fixture.operator_token(), request())
        .await
        .unwrap();
    assert_eq!(retry.ordinal, first.ordinal);
    assert!(retry.deduplicated);
    let metrics = prometheus_text(&state);
    assert_eq!(value_of(&metrics, "foundry_projection_lag"), 1);
    assert_eq!(value_of(&metrics, "foundry_action_submit_served_total"), 2);
    assert_eq!(value_of(&metrics, "foundry_action_submit_refused_total"), 0);
    assert_eq!(fixture.log_head(), 1);
}

#[tokio::test]
async fn poisoned_append_and_retry_keep_the_poisoned_receipt_and_observable_counter() {
    let fixture = Fixture::new("submission-review-poison");
    let state = fixture.state();
    out_of_band::append_for(&fixture.action_log_path(), TENANT, "intervening");
    let first = state
        .submit(fixture.operator_token(), request())
        .await
        .unwrap();
    assert_eq!(first.outcome, "poisoned");
    assert_eq!(first.ordinal, 2);
    assert!(first.poison_reason.is_some());
    let retry = state
        .submit(fixture.operator_token(), request())
        .await
        .unwrap();
    assert_eq!(retry.outcome, "poisoned");
    assert_eq!(retry.ordinal, first.ordinal);
    assert_eq!(retry.poison_reason, first.poison_reason);
    assert!(retry.deduplicated);
    assert_eq!(fixture.log_head(), 2);
    let metrics = prometheus_text(&state);
    assert_eq!(value_of(&metrics, "foundry_action_submit_served_total"), 2);
    assert_eq!(value_of(&metrics, "foundry_action_submit_refused_total"), 0);
    assert_eq!(
        state.tenants[TENANT].lock().await.projection.poison.len(),
        1
    );
}

#[tokio::test]
async fn log_outage_returns_coarse_failure_without_exposing_the_credential_in_storage_error() {
    let fixture = Fixture::new("submission-review-log");
    let state = failing_log::state_with_a_failing_log(&fixture.config(), fixture.operator_token());
    let result = state.submit(fixture.operator_token(), request()).await;
    assert_eq!(result, Err(SubmitError::Unavailable));
    assert!(!format!("{result:?}").contains(fixture.operator_token()));
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
    let metrics = prometheus_text(&state);
    assert_eq!(value_of(&metrics, "foundry_action_submit_served_total"), 0);
    assert_eq!(value_of(&metrics, "foundry_action_submit_refused_total"), 1);
}

struct SlowFirstVerifier(AtomicUsize);
impl CallerVerifier for SlowFirstVerifier {
    fn verify(&self, _: Option<&str>) -> Option<Caller> {
        if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        Some(Caller {
            tenant_id: TENANT.into(),
            principal_id: "prn_verified".into(),
            roles: vec!["foundry-operator".into()],
        })
    }
}

#[tokio::test]
async fn http_latency_indicator_includes_credential_verification_at_handler_entry() {
    let fixture = Fixture::new("submission-review-latency");
    let mut state = fixture.state();
    state.verifier = Box::new(SlowFirstVerifier(AtomicUsize::new(0)));
    let state = Arc::new(state);
    let session = support::Session::from_shared(state.clone());
    let (status, body) = session.post(Some("test-token"), WRITE_BODY).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let metrics = prometheus_text(&state);
    assert_eq!(
        value_of(&metrics, "foundry_action_invocation_answered_total"),
        1
    );
    assert_eq!(
        value_of(
            &metrics,
            "foundry_action_invocation_answered_within_250ms_total"
        ),
        0
    );
    assert_eq!(value_of(&metrics, "foundry_action_submit_served_total"), 1);
}
