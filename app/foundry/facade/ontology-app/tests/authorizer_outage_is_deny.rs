#[path = "facade_support/mod.rs"]
mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use axum::http::StatusCode;
use foundry_ontology_app::{PepError, PolicyEnforcementPoint};
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, PolicyVersion};
use support::{Fixture, Session, WRITE_BODY as WRITE, scrape, value_of};

fn outage_version() -> PolicyVersion {
    PolicyVersion::new("psv-outage").expect("a version token")
}

/// A decision point whose engine fails every evaluation, counting the
/// calls that reached it. An evaluation error is the engine answering
/// "I cannot decide this", which the guard refuses without counting it as
/// a runtime fault when it arrives inside the deadline.
#[derive(Default)]
struct FailingEvaluationPdp {
    calls: AtomicU32, // data_class: INTERNAL_ONLY
}

impl PolicyDecisionPoint for FailingEvaluationPdp {
    fn authorize(&self, _: &AuthorizationRequest, _: &EntitySlice) -> Result<PdpOutcome, PdpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(PdpError::Evaluation {
            detail: "the engine could not evaluate this request".to_owned(),
        })
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        outage_version()
    }
}

/// A decision point that crashes on every decision, counting the crashes.
#[derive(Default)]
struct PanickingPdp {
    calls: AtomicU32, // data_class: INTERNAL_ONLY
}

impl PolicyDecisionPoint for PanickingPdp {
    fn authorize(&self, _: &AuthorizationRequest, _: &EntitySlice) -> Result<PdpOutcome, PdpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        panic!("the engine crashed mid-decision")
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        outage_version()
    }
}

/// The real decision point, answering after a delay.
struct DelayedPdp {
    inner: Arc<dyn PolicyDecisionPoint>, // data_class: INTERNAL_ONLY
    delay: std::time::Duration,          // data_class: INTERNAL_ONLY
}

impl PolicyDecisionPoint for DelayedPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        std::thread::sleep(self.delay);
        self.inner.authorize(request, entities)
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}

fn delayed_cedar(fixture: &Fixture, millis: u64) -> Arc<dyn PolicyDecisionPoint> {
    let version = fixture.state().pep.loaded_policy_version();
    Arc::new(DelayedPdp {
        inner: PolicyEnforcementPoint::cedar(version.as_str()).expect("the seed loads"),
        delay: std::time::Duration::from_millis(millis),
    })
}

fn session_with(fixture: &Fixture, pdp: Arc<dyn PolicyDecisionPoint>) -> Session {
    let mut state = fixture.state();
    state.pep = PolicyEnforcementPoint::with_pdp(pdp);
    Session::from_state(state)
}

#[test]
fn an_uncompilable_policy_version_refuses_to_load() {
    assert!(
        matches!(
            PolicyEnforcementPoint::load(""),
            Err(PepError::BundleRejected { .. })
        ),
        "the process never serves a policy set it could not validate"
    );
}

/// The positive control for the outage refusals below: the same credential
/// and, for the writes, the same body, against a healthy decision point.
#[tokio::test]
async fn a_healthy_authorizer_allows_the_write_the_outage_refuses() {
    let fixture = Fixture::new("outage-control");
    let session = fixture.session();

    let (status, body) = session.post(Some(fixture.operator_token()), WRITE).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(fixture.log_head(), 1);
}

#[tokio::test]
async fn an_authorizer_that_errors_refuses_the_write_and_appends_nothing() {
    let fixture = Fixture::new("outage-error");
    let session = session_with(&fixture, Arc::new(FailingEvaluationPdp::default()));

    let (status, body) = session.post(Some(fixture.operator_token()), WRITE).await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(
        body.contains("the policy decision point refused this invocation"),
        "{body}"
    );
    assert_eq!(fixture.log_head(), 0, "nothing reaches the action log");
    assert_eq!(
        fixture.denial_head(),
        0,
        "nothing reaches the denial trail either"
    );
    assert_eq!(
        value_of(
            &scrape(&session).await,
            "foundry_action_submit_refused_total"
        ),
        1
    );
}

#[tokio::test]
async fn an_authorizer_that_errors_refuses_the_read_surface_too() {
    let fixture = Fixture::new("outage-read");
    let session = session_with(&fixture, Arc::new(FailingEvaluationPdp::default()));

    let (status, body) = session
        .get(Some(fixture.operator_token()), "/v1/types")
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(
        body.contains("the policy decision point refused this read"),
        "{body}"
    );
}

#[tokio::test]
async fn an_authorizer_that_panics_is_a_refusal_and_the_process_keeps_answering() {
    let fixture = Fixture::new("outage-panic");
    let session = session_with(&fixture, Arc::new(PanickingPdp::default()));

    let (status, body) = session.post(Some(fixture.operator_token()), WRITE).await;
    let (read, _) = session
        .get(Some(fixture.operator_token()), "/v1/types")
        .await;
    let (health, _) = session.get(None, "/healthz").await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(
        read,
        StatusCode::FORBIDDEN,
        "the read surface refuses the crash too"
    );
    assert_eq!(
        health,
        StatusCode::OK,
        "behind the guard's unwind boundary, the process keeps answering"
    );
    assert_eq!(fixture.log_head(), 0);
}

/// The guard's deadline, pinned from the facade to lie under 300 ms: the
/// SAME correct Allow is served when it is prompt and refused when it is
/// late. The guard judges the decision after the engine returns, so the
/// caller waits out the engine either way; the deadline bounds what is
/// trusted, not what is waited for.
#[tokio::test]
async fn a_correct_allow_that_arrives_after_the_deadline_is_refused() {
    let prompt = Fixture::new("outage-prompt");
    let (status, body) = session_with(&prompt, delayed_cedar(&prompt, 0))
        .post(Some(prompt.operator_token()), WRITE)
        .await;
    assert_eq!(status, StatusCode::OK, "the wrapped engine allows: {body}");
    assert_eq!(prompt.log_head(), 1);

    let late = Fixture::new("outage-late");
    let (status, body) = session_with(&late, delayed_cedar(&late, 300))
        .post(Some(late.operator_token()), WRITE)
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "the same Allow, 300 ms late: {body}"
    );
    assert_eq!(late.log_head(), 0);
}

/// Five consecutive crashes open the circuit; until the guard's cooldown
/// probe, which lies far past this test's eight requests, the engine is not
/// asked, and every request is still a refusal. A prompt EVALUATION error
/// is not a fault streak (a timeout, a panic, a missing decision id, or a
/// failed audit emission is): the guard keeps asking, and keeps refusing.
/// This pins the kernel's classification of an evaluation error from here.
#[tokio::test]
async fn a_crash_streak_opens_the_circuit_and_stops_asking_the_engine() {
    let fixture = Fixture::new("outage-circuit");
    let crashing = Arc::new(PanickingPdp::default());
    let session = session_with(&fixture, crashing.clone());
    for _ in 0..8 {
        let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    assert_eq!(
        crashing.calls.load(Ordering::SeqCst),
        5,
        "the sixth request onward never reached the engine"
    );

    let erring = Arc::new(FailingEvaluationPdp::default());
    let session = session_with(&fixture, erring.clone());
    for _ in 0..8 {
        let (status, _) = session.post(Some(fixture.operator_token()), WRITE).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    assert_eq!(
        erring.calls.load(Ordering::SeqCst),
        8,
        "a prompt evaluation error is answered by the engine every time"
    );
    assert_eq!(fixture.log_head(), 0);
}

#[tokio::test]
async fn a_credential_for_an_unserved_tenant_is_refused_before_any_log_is_touched() {
    let fixture = Fixture::new("outage-unserved");
    let session = fixture.session();

    let (status, body) = session.post(Some(fixture.foreign_token()), WRITE).await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert!(
        body.contains("does not serve"),
        "the roster's refusal, not Cedar's: {body}"
    );
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
}

#[tokio::test]
async fn a_roleless_operator_is_refused_by_absence_of_a_permit() {
    let fixture = Fixture::new("outage-roleless");
    let session = fixture.session();

    let (status, body) = session.post(Some(fixture.roleless_token()), WRITE).await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "deny-by-default: no permit reaches a principal outside the operator role: {body}"
    );
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 0);
}
