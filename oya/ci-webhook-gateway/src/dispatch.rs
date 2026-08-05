//! Event → pipeline dispatch.
//!
//! Per ADR-0366 (self-enforcing pipeline) + ADR-0367 (trustless gateway), a
//! routable PR event drives the gated pipeline in this order:
//!
//! - Stage 1, admission: historical repo-entry bridge check (`oya-vcs-admission`).
//! - Stage 2, legacy local replay (`oya gate run-all`): the bridge-era
//!   governance lanes. The gateway only kicks the Jenkins lane; it does not
//!   define protected-branch authority, which is now the cloud-ci Rust gate
//!   packet posting `oya-ci-required`.
//! - Stage 3, reviewer gate: adversarial reviewer, a CI stage powered by the
//!   Intelligence service (ADR-0367 D2), with a distinct identity from the
//!   author. NOT yet stood up — the typed `Unimplemented` boundary.
//! - Stage 4, merge-queue admit: ADR-0111 speculative rebase, parked per
//!   ADR-0363 §3 until concurrent-PR volume justifies it. Also the boundary.
//!
//! Board snapshot events are deliberately narrower: they return an observable
//! board-projection receipt and must not kick the legacy local replay.
//!
//! The gateway's PR job is the FIRST hop: verify + parse + route + dispatch the
//! pipeline kickoff. Stages 3 and 4 are not yet stood up in the substrate, so
//! they are expressed as the typed `Unimplemented` boundary (HTTP 501) and
//! tracked in `registry/placeholder-debt/`. Stages 1 and 2 dispatch only the
//! historical Jenkins `oyaCiLane` bridge; they are not merge authority.

use std::future::Future;
use std::pin::Pin;

use crate::error::{GatewayError, PipelineStage, Result};
use crate::event::{CiEvent, IssueAction};

/// Placeholder-debt tokens — these MUST match rows in
/// `registry/placeholder-debt/adr-follow-ups.yaml` so the deferred work is
/// honestly tracked (no lying stub).
pub const DEBT_REVIEWER_GATE: &str = "adr-0374-reviewer-gate-dispatch";
pub const DEBT_MERGE_QUEUE: &str = "adr-0374-merge-queue-admit";

/// A request to kick the historical Jenkins bridge pipeline for one event.
/// This is the payload the Jenkins dispatch endpoint (generic-webhook-trigger
/// or build-token) consumes to start `oyaCiLane`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PipelineKickoff {
    PullRequest {
        pr_number: u64,
        head_ref: String,
        head_sha: String,
        /// `true` when this kickoff is a fix-at-any-stage re-validation
        /// (synchronize) rather than the initial open.
        revalidation: bool,
    },
    IssueSnapshot {
        issue_number: u64,
        action: IssueAction,
        repository_full_name: String,
        snapshot_id: String,
        labels: Vec<String>,
    },
    PushSnapshot {
        reference: String,
        deliverable_id: String,
        before_sha: String,
        after_sha: String,
        sender_login: String,
        pusher_login: Option<String>,
        repository_full_name: String,
        commits: usize,
        snapshot_id: String,
    },
}

impl PipelineKickoff {
    pub fn from_event(event: &CiEvent) -> Self {
        use crate::event::PrAction;
        match event {
            CiEvent::PullRequest(event) => PipelineKickoff::PullRequest {
                pr_number: event.pr_number,
                head_ref: event.head_ref.clone(),
                head_sha: event.head_sha.clone(),
                revalidation: matches!(event.action, PrAction::Synchronized),
            },
            CiEvent::IssueSnapshot(event) => PipelineKickoff::IssueSnapshot {
                issue_number: event.issue_number,
                action: event.action,
                repository_full_name: event.repository_full_name.clone(),
                snapshot_id: event.snapshot_id.clone(),
                labels: event.labels.clone(),
            },
            CiEvent::PushSnapshot(event) => PipelineKickoff::PushSnapshot {
                reference: event.reference.clone(),
                deliverable_id: event.deliverable_id.clone(),
                before_sha: event.before_sha.clone(),
                after_sha: event.after_sha.clone(),
                sender_login: event.sender_login.clone(),
                pusher_login: event.pusher_login.clone(),
                repository_full_name: event.repository_full_name.clone(),
                commits: event.commits,
                snapshot_id: event.snapshot_id.clone(),
            },
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            PipelineKickoff::PullRequest { .. } => "pull_request",
            PipelineKickoff::IssueSnapshot { .. } => "issue_snapshot",
            PipelineKickoff::PushSnapshot { .. } => "push_snapshot",
        }
    }
}

/// What the dispatcher did with one event — surfaced in the receiver response
/// + the structured log line, so behavior is observable end-to-end.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchReceipt {
    pub subject: DispatchSubject,
    pub head_sha: String,
    pub snapshot_id: Option<String>,
    /// The furthest stage the gateway successfully kicked.
    pub kicked_through: PipelineStage,
    /// The next stage, which is the typed `Unimplemented` boundary (stages
    /// past the historical bridge replay are outside this gateway).
    pub boundary: Option<PipelineStage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchSubject {
    PullRequest { pr_number: u64 },
    Issue { issue_number: u64 },
    Push { reference: String },
}

impl DispatchSubject {
    pub fn kind(&self) -> &'static str {
        match self {
            DispatchSubject::PullRequest { .. } => "pull_request",
            DispatchSubject::Issue { .. } => "issue",
            DispatchSubject::Push { .. } => "push",
        }
    }
}

/// The dispatch port. Implementors kick the downstream pipeline. The gateway
/// depends only on this trait (clean-architecture port) so the HTTP receiver
/// is independent of the concrete Jenkins transport.
pub trait PipelineDispatcher: Send + Sync {
    /// Kick the historical admission + local-replay bridge for this event.
    /// Returns the receipt; downstream stages past the bridge are outside this
    /// gateway and do not define current protected-branch authority.
    fn dispatch<'a>(
        &'a self,
        event: &'a CiEvent,
    ) -> Pin<Box<dyn Future<Output = Result<DispatchReceipt>> + Send + 'a>>;
}

/// The Jenkins-backed dispatcher: kicks the `oyaCiLane` pipeline via the
/// configured dispatch URL. Admission (stage 1) + gate-run-all (stage 2) are
/// BOTH stages inside that one pipeline (see `oyaCiLane.groovy`), so a single
/// kick covers them; the reviewer gate + merge-queue are the not-yet-built
/// boundary.
pub struct JenkinsDispatcher<K> {
    kick: K,
    dispatch_url: Option<String>,
}

impl<K> JenkinsDispatcher<K>
where
    K: Fn(String, PipelineKickoff) -> std::result::Result<(), String> + Send + Sync,
{
    /// `dispatch_url` is the Jenkins endpoint; `kick` performs the transport
    /// (injected so tests can assert the kickoff without a live Jenkins).
    pub fn new(dispatch_url: Option<String>, kick: K) -> Self {
        JenkinsDispatcher { kick, dispatch_url }
    }
}

impl<K> PipelineDispatcher for JenkinsDispatcher<K>
where
    K: Fn(String, PipelineKickoff) -> std::result::Result<(), String> + Send + Sync,
{
    fn dispatch<'a>(
        &'a self,
        event: &'a CiEvent,
    ) -> Pin<Box<dyn Future<Output = Result<DispatchReceipt>> + Send + 'a>> {
        Box::pin(async move {
            let kickoff = PipelineKickoff::from_event(event);
            let receipt = receipt_for(event);

            if !matches!(event, CiEvent::PullRequest(_)) {
                return Ok(receipt);
            }

            // Stage 1 + 2: the Jenkins `oyaCiLane` bridge replays admission
            // then the historical gate set for provenance/local feedback.
            let Some(url) = self.dispatch_url.clone() else {
                return Err(GatewayError::DispatchTransport(
                    "Jenkins dispatch URL not configured (OYA_JENKINS_DISPATCH_URL); \
                     refusing to claim a kick that did not happen"
                        .to_owned(),
                ));
            };
            (self.kick)(url, kickoff).map_err(GatewayError::DispatchTransport)?;

            // Stage 3 (reviewer gate) is owned by the Intelligence service as a
            // CI stage; it is NOT yet stood up. We report the honest boundary
            // rather than pretend the PR is fully gated.
            Ok(receipt)
        })
    }
}

/// The bespoke oya-ci-controller dispatcher: kicks the `/gate-run` endpoint on
/// the controller (ADR-0374 Phase-B — bespoke-Prow plank role). Mirrors the
/// shape of `JenkinsDispatcher` exactly; only the URL env var and the request
/// body shape differ.
///
/// Only `PullRequest` events POST to `/gate-run`; board-projection events
/// (IssueSnapshot, PushSnapshot) are returned as receipts without a network
/// call, mirroring the Jenkins path.
///
/// The `post` closure is injected so tests can assert the outgoing request body
/// without a live controller.
pub struct ControllerDispatcher<P> {
    post: P,
    controller_url: Option<String>,
}

impl<P> ControllerDispatcher<P>
where
    P: Fn(String, GateRunBody) -> std::result::Result<(), String> + Send + Sync,
{
    /// `controller_url` is the base URL of the oya-ci-controller (e.g.
    /// `http://oya-ci-controller.oya-ci.svc:8080`). The `/gate-run` path is
    /// appended automatically. `post` performs the transport (injected for
    /// tests).
    pub fn new(controller_url: Option<String>, post: P) -> Self {
        ControllerDispatcher { post, controller_url }
    }
}

impl<P> PipelineDispatcher for ControllerDispatcher<P>
where
    P: Fn(String, GateRunBody) -> std::result::Result<(), String> + Send + Sync,
{
    fn dispatch<'a>(
        &'a self,
        event: &'a CiEvent,
    ) -> Pin<Box<dyn Future<Output = Result<DispatchReceipt>> + Send + 'a>> {
        Box::pin(async move {
            let receipt = receipt_for(event);

            if !matches!(event, CiEvent::PullRequest(_)) {
                return Ok(receipt);
            }

            let kickoff = PipelineKickoff::from_event(event);
            let body = GateRunBody::from_kickoff(&kickoff);

            let Some(base_url) = self.controller_url.clone() else {
                return Err(GatewayError::DispatchTransport(
                    "controller dispatch URL not configured (OYA_CI_CONTROLLER_URL); \
                     refusing to claim a kick that did not happen"
                        .to_owned(),
                ));
            };
            let url = format!("{base_url}/gate-run");
            (self.post)(url, body).map_err(GatewayError::DispatchTransport)?;

            Ok(receipt)
        })
    }
}

/// The JSON body POSTed to the controller's `POST /gate-run`.
///
/// Shape mirrors `GateRunRequest` in `oya-ci-controller-app`:
/// `{"pr_number": N, "head_sha": "...", "base_ref": "dev"}`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateRunBody {
    pub pr_number: u64,
    pub head_sha: String,
    pub base_ref: String,
}

impl GateRunBody {
    fn from_kickoff(kickoff: &PipelineKickoff) -> Self {
        match kickoff {
            PipelineKickoff::PullRequest {
                pr_number,
                head_sha,
                ..
            } => GateRunBody {
                pr_number: *pr_number,
                head_sha: head_sha.clone(),
                base_ref: "dev".to_owned(),
            },
            // Only PullRequest kickoffs reach the controller; this arm is
            // unreachable in normal operation (the dispatcher early-returns for
            // non-PR events above). Provide a safe fallback rather than panic.
            _ => GateRunBody {
                pr_number: 0,
                head_sha: String::new(),
                base_ref: "dev".to_owned(),
            },
        }
    }
}

fn receipt_for(event: &CiEvent) -> DispatchReceipt {
    match event {
        CiEvent::PullRequest(event) => DispatchReceipt {
            subject: DispatchSubject::PullRequest {
                pr_number: event.pr_number,
            },
            head_sha: event.head_sha.clone(),
            snapshot_id: None,
            kicked_through: PipelineStage::GateRunAll,
            boundary: Some(PipelineStage::ReviewerGate),
        },
        CiEvent::IssueSnapshot(event) => DispatchReceipt {
            subject: DispatchSubject::Issue {
                issue_number: event.issue_number,
            },
            head_sha: event.snapshot_id.clone(),
            snapshot_id: Some(event.snapshot_id.clone()),
            kicked_through: PipelineStage::BoardProjection,
            boundary: None,
        },
        CiEvent::PushSnapshot(event) => DispatchReceipt {
            subject: DispatchSubject::Push {
                reference: event.reference.clone(),
            },
            head_sha: event.after_sha.clone(),
            snapshot_id: Some(event.snapshot_id.clone()),
            kicked_through: PipelineStage::BoardProjection,
            boundary: None,
        },
    }
}

/// Explicitly signal that a downstream stage is not built. Used by callers that
/// reach past the dispatcher's kicked stages; keeps the boundary typed + named.
pub fn unimplemented(stage: PipelineStage) -> GatewayError {
    let debt_token = match stage {
        PipelineStage::ReviewerGate => DEBT_REVIEWER_GATE,
        PipelineStage::MergeQueue => DEBT_MERGE_QUEUE,
        // Admission + gate-run-all ARE built (the Jenkins lane); reaching here
        // for them is a programming error, but we still return a typed value.
        PipelineStage::Admission | PipelineStage::GateRunAll | PipelineStage::BoardProjection => {
            DEBT_REVIEWER_GATE
        }
    };
    GatewayError::Unimplemented { stage, debt_token }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{IssueSnapshotEvent, PrAction, PullRequestEvent, PushSnapshotEvent};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn pr_event(action: PrAction) -> CiEvent {
        CiEvent::PullRequest(PullRequestEvent {
            action,
            pr_number: 99,
            base_ref: "dev".to_owned(),
            head_ref: "feature/y".to_owned(),
            head_sha: "deadbeef".to_owned(),
            draft: false,
        })
    }

    fn issue_event() -> CiEvent {
        CiEvent::IssueSnapshot(IssueSnapshotEvent {
            action: IssueAction::Edited,
            issue_number: 108,
            title: "board".to_owned(),
            state: "open".to_owned(),
            updated_at: Some("2026-05-27T19:00:00Z".to_owned()),
            labels: vec!["masterplan:IP-108".to_owned()],
            html_url: None,
            repository_full_name: "owner/repo".to_owned(),
            snapshot_id:
                "issue:sha256:a71527716283639cdc975ef544828c655d501e1fac5c859e64c20af5b9a33aad"
                    .to_owned(),
        })
    }

    fn push_event() -> CiEvent {
        CiEvent::PushSnapshot(PushSnapshotEvent {
            reference: "refs/heads/claims/ADR-0377-D2".to_owned(),
            deliverable_id: "ADR-0377-D2".to_owned(),
            before_sha: "abc".to_owned(),
            after_sha: "def".to_owned(),
            sender_login: "worker-a".to_owned(),
            pusher_login: Some("worker-a".to_owned()),
            repository_full_name: "owner/repo".to_owned(),
            commits: 1,
            deleted: false,
            snapshot_id:
                "push:sha256:fc52d4f3c19063e4570ca74b1bfd138babf545ee57199d13e4fcca300257a0c3"
                    .to_owned(),
        })
    }

    #[tokio::test]
    async fn dispatch_kicks_jenkins_and_reports_boundary() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let dispatcher = JenkinsDispatcher::new(
            Some("http://jenkins/job/oya/build".to_owned()),
            move |url, kickoff| {
                calls2.fetch_add(1, Ordering::SeqCst);
                assert!(url.contains("jenkins"));
                match kickoff {
                    PipelineKickoff::PullRequest {
                        pr_number,
                        revalidation,
                        ..
                    } => {
                        assert_eq!(pr_number, 99);
                        assert!(!revalidation);
                    }
                    other => panic!("expected PR kickoff, got {other:?}"),
                }
                Ok(())
            },
        );
        let receipt = dispatcher
            .dispatch(&pr_event(PrAction::Opened))
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            receipt.subject,
            DispatchSubject::PullRequest { pr_number: 99 }
        );
        assert_eq!(receipt.kicked_through, PipelineStage::GateRunAll);
        assert_eq!(receipt.boundary, Some(PipelineStage::ReviewerGate));
    }

    #[tokio::test]
    async fn synchronize_marks_revalidation() {
        let dispatcher = JenkinsDispatcher::new(
            Some("http://jenkins/job/oya/build".to_owned()),
            |_url, kickoff| {
                assert!(matches!(
                    kickoff,
                    PipelineKickoff::PullRequest {
                        revalidation: true,
                        ..
                    }
                ));
                Ok(())
            },
        );
        let receipt = dispatcher
            .dispatch(&pr_event(PrAction::Synchronized))
            .await
            .unwrap();
        assert_eq!(receipt.kicked_through, PipelineStage::GateRunAll);
    }

    #[tokio::test]
    async fn issue_snapshot_returns_board_projection_receipt_without_jenkins() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let dispatcher = JenkinsDispatcher::new(
            Some("http://jenkins/job/oya/build".to_owned()),
            move |_url, kickoff| {
                calls2.fetch_add(1, Ordering::SeqCst);
                match kickoff {
                    PipelineKickoff::IssueSnapshot {
                        issue_number,
                        snapshot_id,
                        labels,
                        ..
                    } => {
                        assert_eq!(issue_number, 108);
                        assert_eq!(
                            snapshot_id,
                            "issue:sha256:a71527716283639cdc975ef544828c655d501e1fac5c859e64c20af5b9a33aad"
                        );
                        assert_eq!(labels, vec!["masterplan:IP-108"]);
                    }
                    other => panic!("expected issue snapshot, got {other:?}"),
                }
                Ok(())
            },
        );
        let receipt = dispatcher.dispatch(&issue_event()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            receipt.subject,
            DispatchSubject::Issue { issue_number: 108 }
        );
        assert_eq!(
            receipt.snapshot_id,
            Some(
                "issue:sha256:a71527716283639cdc975ef544828c655d501e1fac5c859e64c20af5b9a33aad"
                    .to_owned()
            )
        );
        assert_eq!(receipt.kicked_through, PipelineStage::BoardProjection);
        assert_eq!(receipt.boundary, None);
    }

    #[tokio::test]
    async fn push_snapshot_returns_board_projection_receipt_without_jenkins() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let dispatcher = JenkinsDispatcher::new(
            Some("http://jenkins/job/oya/build".to_owned()),
            move |_url, kickoff| {
                calls2.fetch_add(1, Ordering::SeqCst);
                match kickoff {
                    PipelineKickoff::PushSnapshot {
                        reference,
                        after_sha,
                        sender_login,
                        pusher_login,
                        snapshot_id,
                        ..
                    } => {
                        assert_eq!(reference, "refs/heads/claims/ADR-0377-D2");
                        assert_eq!(after_sha, "def");
                        assert_eq!(sender_login, "worker-a");
                        assert_eq!(pusher_login, Some("worker-a".to_owned()));
                        assert_eq!(
                            snapshot_id,
                            "push:sha256:fc52d4f3c19063e4570ca74b1bfd138babf545ee57199d13e4fcca300257a0c3"
                        );
                    }
                    other => panic!("expected push snapshot, got {other:?}"),
                }
                Ok(())
            },
        );
        let receipt = dispatcher.dispatch(&push_event()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            receipt.subject,
            DispatchSubject::Push {
                reference: "refs/heads/claims/ADR-0377-D2".to_owned()
            }
        );
        assert_eq!(
            receipt.snapshot_id,
            Some(
                "push:sha256:fc52d4f3c19063e4570ca74b1bfd138babf545ee57199d13e4fcca300257a0c3"
                    .to_owned()
            )
        );
        assert_eq!(receipt.kicked_through, PipelineStage::BoardProjection);
        assert_eq!(receipt.boundary, None);
    }

    #[tokio::test]
    async fn missing_url_is_transport_error_not_silent_success() {
        let dispatcher = JenkinsDispatcher::new(None, |_url, _kickoff| Ok(()));
        let err = dispatcher
            .dispatch(&pr_event(PrAction::Opened))
            .await
            .unwrap_err();
        assert!(matches!(err, GatewayError::DispatchTransport(_)));
    }

    #[tokio::test]
    async fn kick_failure_propagates_as_transport_error() {
        let dispatcher = JenkinsDispatcher::new(
            Some("http://jenkins/job/oya/build".to_owned()),
            |_url, _kickoff| Err("connection refused".to_owned()),
        );
        let err = dispatcher
            .dispatch(&pr_event(PrAction::Opened))
            .await
            .unwrap_err();
        match err {
            GatewayError::DispatchTransport(why) => assert!(why.contains("connection refused")),
            other => panic!("expected transport error, got {other:?}"),
        }
    }

    #[test]
    fn unimplemented_boundary_carries_debt_token() {
        let err = unimplemented(PipelineStage::MergeQueue);
        match err {
            GatewayError::Unimplemented { stage, debt_token } => {
                assert_eq!(stage, PipelineStage::MergeQueue);
                assert_eq!(debt_token, DEBT_MERGE_QUEUE);
            }
            other => panic!("expected unimplemented, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // ControllerDispatcher tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn controller_dispatch_posts_gate_run_and_reports_boundary() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let dispatcher = ControllerDispatcher::new(
            Some("http://oya-ci-controller.oya-ci.svc:8080".to_owned()),
            move |url, body| {
                calls2.fetch_add(1, Ordering::SeqCst);
                assert!(url.ends_with("/gate-run"), "url={url}");
                assert!(url.contains("oya-ci-controller"));
                assert_eq!(body.pr_number, 99);
                assert_eq!(body.head_sha, "deadbeef");
                assert_eq!(body.base_ref, "dev");
                Ok(())
            },
        );
        let receipt = dispatcher
            .dispatch(&pr_event(PrAction::Opened))
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            receipt.subject,
            DispatchSubject::PullRequest { pr_number: 99 }
        );
        assert_eq!(receipt.kicked_through, PipelineStage::GateRunAll);
        assert_eq!(receipt.boundary, Some(PipelineStage::ReviewerGate));
    }

    #[tokio::test]
    async fn controller_dispatch_missing_url_is_transport_error_not_silent_success() {
        let dispatcher = ControllerDispatcher::new(None, |_url, _body| Ok(()));
        let err = dispatcher
            .dispatch(&pr_event(PrAction::Opened))
            .await
            .unwrap_err();
        assert!(matches!(err, GatewayError::DispatchTransport(_)));
    }

    #[tokio::test]
    async fn controller_dispatch_post_failure_propagates_as_transport_error() {
        let dispatcher = ControllerDispatcher::new(
            Some("http://oya-ci-controller.oya-ci.svc:8080".to_owned()),
            |_url, _body| Err("connection refused".to_owned()),
        );
        let err = dispatcher
            .dispatch(&pr_event(PrAction::Opened))
            .await
            .unwrap_err();
        match err {
            GatewayError::DispatchTransport(why) => assert!(why.contains("connection refused")),
            other => panic!("expected transport error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn controller_dispatch_issue_snapshot_no_post() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let dispatcher = ControllerDispatcher::new(
            Some("http://oya-ci-controller.oya-ci.svc:8080".to_owned()),
            move |_url, _body| {
                calls2.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        );
        let receipt = dispatcher.dispatch(&issue_event()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            receipt.subject,
            DispatchSubject::Issue { issue_number: 108 }
        );
        assert_eq!(receipt.kicked_through, PipelineStage::BoardProjection);
    }

    #[tokio::test]
    async fn controller_dispatch_push_snapshot_no_post() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls2 = calls.clone();
        let dispatcher = ControllerDispatcher::new(
            Some("http://oya-ci-controller.oya-ci.svc:8080".to_owned()),
            move |_url, _body| {
                calls2.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        );
        let receipt = dispatcher.dispatch(&push_event()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            receipt.subject,
            DispatchSubject::Push {
                reference: "refs/heads/claims/ADR-0377-D2".to_owned()
            }
        );
        assert_eq!(receipt.kicked_through, PipelineStage::BoardProjection);
    }

    #[test]
    fn gate_run_body_maps_pr_kickoff_correctly() {
        let kickoff = PipelineKickoff::PullRequest {
            pr_number: 42,
            head_ref: "feat/foo".to_owned(),
            head_sha: "abc12345".to_owned(),
            revalidation: true,
        };
        let body = GateRunBody::from_kickoff(&kickoff);
        assert_eq!(body.pr_number, 42);
        assert_eq!(body.head_sha, "abc12345");
        assert_eq!(body.base_ref, "dev");
    }
}
