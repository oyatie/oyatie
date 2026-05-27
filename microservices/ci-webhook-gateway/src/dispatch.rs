//! Event → pipeline dispatch.
//!
//! Per ADR-0366 (self-enforcing pipeline) + ADR-0367 (trustless gateway), a
//! routable PR/board-snapshot event drives the gated pipeline in this order:
//!
//! - Stage 1, admission: repo-entry governance check (`oya-vcs-admission`).
//! - Stage 2, `oya gate run-all`: the ~50 oyatie governance lanes. Jenkins is
//!   the TRUSTED RUNNER that re-executes hermetically and signs (ADR-0367 D1).
//!   The gateway only KICKS the Jenkins lane; it does not itself run gates and
//!   it never trusts author-reported evidence.
//! - Stage 3, reviewer gate: adversarial reviewer, a CI stage powered by the
//!   Intelligence service (ADR-0367 D2), with a distinct identity from the
//!   author. NOT yet stood up — the typed `Unimplemented` boundary.
//! - Stage 4, merge-queue admit: ADR-0111 speculative rebase, parked per
//!   ADR-0363 §3 until concurrent-PR volume justifies it. Also the boundary.
//!
//! The gateway's job is the FIRST hop: verify + parse + route + dispatch the
//! pipeline kickoff. Stages 3 and 4 are not yet stood up in the substrate, so
//! they are expressed as the typed `Unimplemented` boundary (HTTP 501) and
//! tracked in `registry/placeholder-debt/`. Stages 1 and 2 dispatch by kicking
//! the Jenkins `oyaCiLane` pipeline (which posts the Forgejo commit statuses).

use std::future::Future;
use std::pin::Pin;

use crate::error::{GatewayError, PipelineStage, Result};
use crate::event::{CiEvent, IssueAction};

/// Placeholder-debt tokens — these MUST match rows in
/// `registry/placeholder-debt/adr-follow-ups.yaml` so the deferred work is
/// honestly tracked (no lying stub).
pub const DEBT_REVIEWER_GATE: &str = "adr-0374-reviewer-gate-dispatch";
pub const DEBT_MERGE_QUEUE: &str = "adr-0374-merge-queue-admit";

/// A request to kick the trusted-runner (Jenkins) pipeline for one event.
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
        before_sha: String,
        after_sha: String,
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
                before_sha: event.before_sha.clone(),
                after_sha: event.after_sha.clone(),
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
    /// The next stage, which is the typed `Unimplemented` boundary (the
    /// trusted-runner pipeline owns advancing past `kicked_through`).
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
    /// Kick admission + the trusted-runner gate pipeline for this event.
    /// Returns the receipt; downstream stages past the kicked pipeline are the
    /// trusted runner's responsibility (it posts the Forgejo commit statuses).
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

            // Stage 1 + 2: the Jenkins `oyaCiLane` pipeline runs admission then
            // the gate suite (the trusted-runner re-execution per ADR-0367).
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
            kicked_through: PipelineStage::GateRunAll,
            boundary: Some(PipelineStage::ReviewerGate),
        },
        CiEvent::PushSnapshot(event) => DispatchReceipt {
            subject: DispatchSubject::Push {
                reference: event.reference.clone(),
            },
            head_sha: event.after_sha.clone(),
            snapshot_id: Some(event.snapshot_id.clone()),
            kicked_through: PipelineStage::GateRunAll,
            boundary: Some(PipelineStage::ReviewerGate),
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
        PipelineStage::Admission | PipelineStage::GateRunAll => DEBT_REVIEWER_GATE,
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
            snapshot_id: "issue:owner/repo:108:edited".to_owned(),
        })
    }

    fn push_event() -> CiEvent {
        CiEvent::PushSnapshot(PushSnapshotEvent {
            reference: "refs/heads/dev".to_owned(),
            before_sha: "abc".to_owned(),
            after_sha: "def".to_owned(),
            repository_full_name: "owner/repo".to_owned(),
            commits: 1,
            deleted: false,
            snapshot_id: "push:owner/repo:refs/heads/dev:def".to_owned(),
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
    async fn issue_snapshot_kicks_jenkins_with_snapshot_metadata() {
        let dispatcher = JenkinsDispatcher::new(
            Some("http://jenkins/job/oya/build".to_owned()),
            |_url, kickoff| {
                match kickoff {
                    PipelineKickoff::IssueSnapshot {
                        issue_number,
                        snapshot_id,
                        labels,
                        ..
                    } => {
                        assert_eq!(issue_number, 108);
                        assert_eq!(snapshot_id, "issue:owner/repo:108:edited");
                        assert_eq!(labels, vec!["masterplan:IP-108"]);
                    }
                    other => panic!("expected issue snapshot, got {other:?}"),
                }
                Ok(())
            },
        );
        let receipt = dispatcher.dispatch(&issue_event()).await.unwrap();
        assert_eq!(
            receipt.subject,
            DispatchSubject::Issue { issue_number: 108 }
        );
        assert_eq!(
            receipt.snapshot_id,
            Some("issue:owner/repo:108:edited".to_owned())
        );
    }

    #[tokio::test]
    async fn push_snapshot_kicks_jenkins_with_snapshot_metadata() {
        let dispatcher = JenkinsDispatcher::new(
            Some("http://jenkins/job/oya/build".to_owned()),
            |_url, kickoff| {
                match kickoff {
                    PipelineKickoff::PushSnapshot {
                        reference,
                        after_sha,
                        snapshot_id,
                        ..
                    } => {
                        assert_eq!(reference, "refs/heads/dev");
                        assert_eq!(after_sha, "def");
                        assert_eq!(snapshot_id, "push:owner/repo:refs/heads/dev:def");
                    }
                    other => panic!("expected push snapshot, got {other:?}"),
                }
                Ok(())
            },
        );
        let receipt = dispatcher.dispatch(&push_event()).await.unwrap();
        assert_eq!(
            receipt.subject,
            DispatchSubject::Push {
                reference: "refs/heads/dev".to_owned()
            }
        );
        assert_eq!(
            receipt.snapshot_id,
            Some("push:owner/repo:refs/heads/dev:def".to_owned())
        );
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
}
