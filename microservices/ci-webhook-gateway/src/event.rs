//! Forgejo webhook event parsing + the closed router table.
//!
//! The gateway acts on `pull_request` events whose base branch is the gated
//! target (default `dev`) and on issue/push events that need to snapshot board
//! metadata without standing up a second long-running service. Board push
//! snapshots are restricted to `refs/heads/claims/*`; ordinary branch pushes do
//! not drive board projection. Forgejo's
//! `pull_request` payload mirrors Gitea's:
//!   { "action": "opened|synchronized|...",
//!     "number": 42,
//!     "pull_request": {
//!       "number": 42,
//!       "base": { "ref": "dev", "sha": "..." },
//!       "head": { "ref": "feature/x", "sha": "..." },
//!       "draft": false } }
//!
//! NOTE the Forgejo/Gitea spelling: the synchronize action is `synchronized`
//! (GitHub uses `synchronize`). We accept BOTH so the gateway is robust to the
//! GitHub-bootstrap host (ADR-0247) as well as the Forgejo target.
//!
//! The router is a CLOSED mapping: any `(event, action)` not enumerated here
//! is a typed `UnroutableEvent` (logged + rejected), never a silent drop.

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::{GatewayError, Result};

/// The webhook event class, taken from Forgejo's `X-Forgejo-Event` /
/// `X-Gitea-Event` / GitHub's `X-GitHub-Event` header.
pub const EVENT_HEADER_FORGEJO: &str = "x-forgejo-event";
pub const EVENT_HEADER_GITEA: &str = "x-gitea-event";
pub const EVENT_HEADER_GITHUB: &str = "x-github-event";
/// Delivery-id header used for idempotent dedup (UUID, stable across redelivery).
pub const DELIVERY_HEADER_FORGEJO: &str = "x-forgejo-delivery";
pub const DELIVERY_HEADER_GITEA: &str = "x-gitea-delivery";
pub const DELIVERY_HEADER_GITHUB: &str = "x-github-delivery";

/// The closed set of PR actions the gateway routes to the gated pipeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrAction {
    /// A PR was opened or reopened against the target branch — begin gating.
    Opened,
    /// New commits were pushed to the PR head (fix-at-any-stage, ADR-0111) —
    /// re-validate.
    Synchronized,
}

impl PrAction {
    fn parse(action: &str) -> Option<Self> {
        match action {
            "opened" | "reopened" => Some(PrAction::Opened),
            // Forgejo/Gitea: "synchronized"; GitHub: "synchronize".
            "synchronized" | "synchronize" => Some(PrAction::Synchronized),
            _ => None,
        }
    }
}

/// The closed set of issue actions that should refresh board snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueAction {
    Opened,
    Edited,
    Reopened,
    Closed,
    Labeled,
    Unlabeled,
    Assigned,
    Unassigned,
    Milestoned,
    Demilestoned,
}

impl IssueAction {
    fn parse(action: &str) -> Option<Self> {
        match action {
            "opened" => Some(IssueAction::Opened),
            "edited" => Some(IssueAction::Edited),
            "reopened" => Some(IssueAction::Reopened),
            "closed" => Some(IssueAction::Closed),
            "labeled" => Some(IssueAction::Labeled),
            "unlabeled" => Some(IssueAction::Unlabeled),
            "assigned" => Some(IssueAction::Assigned),
            "unassigned" => Some(IssueAction::Unassigned),
            "milestoned" => Some(IssueAction::Milestoned),
            "demilestoned" => Some(IssueAction::Demilestoned),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            IssueAction::Opened => "opened",
            IssueAction::Edited => "edited",
            IssueAction::Reopened => "reopened",
            IssueAction::Closed => "closed",
            IssueAction::Labeled => "labeled",
            IssueAction::Unlabeled => "unlabeled",
            IssueAction::Assigned => "assigned",
            IssueAction::Unassigned => "unassigned",
            IssueAction::Milestoned => "milestoned",
            IssueAction::Demilestoned => "demilestoned",
        }
    }
}

/// A parsed, routable pull-request event — the input to the dispatcher.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullRequestEvent {
    pub action: PrAction,
    pub pr_number: u64,
    pub base_ref: String,
    pub head_ref: String,
    pub head_sha: String,
    pub draft: bool,
}

/// A parsed issue event that refreshes the downstream board snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssueSnapshotEvent {
    pub action: IssueAction,
    pub issue_number: u64,
    pub title: String,
    pub state: String,
    pub updated_at: Option<String>,
    pub labels: Vec<String>,
    pub html_url: Option<String>,
    pub repository_full_name: String,
    pub snapshot_id: String,
}

/// A parsed push event that refreshes the downstream board snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushSnapshotEvent {
    pub reference: String,
    pub deliverable_id: String,
    pub before_sha: String,
    pub after_sha: String,
    pub repository_full_name: String,
    pub commits: usize,
    pub deleted: bool,
    pub snapshot_id: String,
}

/// Every delivery the gateway can dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CiEvent {
    PullRequest(PullRequestEvent),
    IssueSnapshot(IssueSnapshotEvent),
    PushSnapshot(PushSnapshotEvent),
}

// ---- raw payload shapes (serde) ------------------------------------------

#[derive(Deserialize)]
struct RawPullRequestPayload {
    action: String,
    #[serde(default)]
    number: Option<u64>,
    pull_request: RawPullRequest,
}

#[derive(Deserialize)]
struct RawPullRequest {
    #[serde(default)]
    number: Option<u64>,
    base: RawRef,
    head: RawRef,
    #[serde(default)]
    draft: bool,
}

#[derive(Deserialize)]
struct RawRef {
    #[serde(rename = "ref")]
    ref_name: String,
    #[serde(default)]
    sha: String,
}

#[derive(Deserialize)]
struct RawIssuePayload {
    action: String,
    issue: RawIssue,
    #[serde(default)]
    repository: Option<RawRepository>,
}

#[derive(Deserialize)]
struct RawIssue {
    number: u64,
    title: String,
    state: String,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    labels: Vec<RawLabel>,
    #[serde(default)]
    html_url: Option<String>,
}

#[derive(Deserialize)]
struct RawLabel {
    name: String,
}

#[derive(Deserialize)]
struct RawRepository {
    full_name: String,
}

#[derive(Deserialize)]
struct RawPushPayload {
    #[serde(rename = "ref")]
    reference: String,
    before: String,
    after: String,
    #[serde(default)]
    repository: Option<RawRepository>,
    #[serde(default)]
    commits: Vec<serde_json::Value>,
    #[serde(default)]
    deleted: bool,
}

/// The outcome of routing a raw delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteOutcome {
    /// An event the pipeline should act on.
    Dispatch(CiEvent),
    /// Authentic + parseable, but deliberately ignored (wrong base branch, a
    /// draft PR, or a PR action we don't gate). Distinguished from an error so
    /// the receiver returns 200 (no Forgejo redelivery storm) without dispatch.
    Ignored { reason: String },
}

/// Route a raw delivery to an outcome.
///
/// `event` is the `X-*-Event` header value; `body` is the RAW (already
/// signature-verified) JSON body; `target_branch` is the gated base branch.
pub fn route(event: &str, body: &[u8], target_branch: &str) -> Result<RouteOutcome> {
    match event {
        "pull_request" => route_pull_request(body, target_branch),
        "issues" | "issue" => route_issue(body),
        "push" => route_push(body, target_branch),
        // `ping` is Forgejo's webhook-registration handshake — accept + ignore.
        "ping" => Ok(RouteOutcome::Ignored {
            reason: "ping handshake".to_owned(),
        }),
        other => Err(GatewayError::UnroutableEvent {
            event: other.to_owned(),
            action: String::new(),
        }),
    }
}

fn route_pull_request(body: &[u8], target_branch: &str) -> Result<RouteOutcome> {
    let raw: RawPullRequestPayload = serde_json::from_slice(body)
        .map_err(|e| GatewayError::MalformedPayload(format!("pull_request: {e}")))?;

    let Some(action) = PrAction::parse(&raw.action) else {
        return Ok(RouteOutcome::Ignored {
            reason: format!("pull_request action {:?} not gated", raw.action),
        });
    };

    if raw.pull_request.base.ref_name != target_branch {
        return Ok(RouteOutcome::Ignored {
            reason: format!(
                "base ref {:?} != gated target {:?}",
                raw.pull_request.base.ref_name, target_branch
            ),
        });
    }

    if raw.pull_request.draft {
        return Ok(RouteOutcome::Ignored {
            reason: "PR is a draft".to_owned(),
        });
    }

    let pr_number =
        raw.pull_request.number.or(raw.number).ok_or_else(|| {
            GatewayError::MalformedPayload("missing pull_request.number".to_owned())
        })?;

    if raw.pull_request.head.sha.trim().is_empty() {
        return Err(GatewayError::MalformedPayload(
            "missing pull_request.head.sha".to_owned(),
        ));
    }

    Ok(RouteOutcome::Dispatch(CiEvent::PullRequest(
        PullRequestEvent {
            action,
            pr_number,
            base_ref: raw.pull_request.base.ref_name,
            head_ref: raw.pull_request.head.ref_name,
            head_sha: raw.pull_request.head.sha,
            draft: raw.pull_request.draft,
        },
    )))
}

fn route_issue(body: &[u8]) -> Result<RouteOutcome> {
    let raw: RawIssuePayload = serde_json::from_slice(body)
        .map_err(|e| GatewayError::MalformedPayload(format!("issues: {e}")))?;

    let Some(action) = IssueAction::parse(&raw.action) else {
        return Ok(RouteOutcome::Ignored {
            reason: format!("issues action {:?} not snapshot-worthy", raw.action),
        });
    };

    let repository_full_name = raw
        .repository
        .map(|repo| repo.full_name)
        .unwrap_or_else(|| "unknown".to_owned());
    let mut labels = raw
        .issue
        .labels
        .into_iter()
        .map(|label| label.name)
        .collect::<Vec<_>>();
    labels.sort();
    labels.dedup();
    let snapshot_id = content_addressed_snapshot_id(
        "issue",
        &serde_json::json!({
            "action": action.id(),
            "issue_number": raw.issue.number,
            "labels": labels,
            "repository_full_name": repository_full_name,
            "state": raw.issue.state,
            "title": raw.issue.title,
            "updated_at": raw.issue.updated_at,
        }),
    );

    Ok(RouteOutcome::Dispatch(CiEvent::IssueSnapshot(
        IssueSnapshotEvent {
            action,
            issue_number: raw.issue.number,
            title: raw.issue.title,
            state: raw.issue.state,
            updated_at: raw.issue.updated_at,
            labels,
            html_url: raw.issue.html_url,
            repository_full_name,
            snapshot_id,
        },
    )))
}

fn route_push(body: &[u8], _target_branch: &str) -> Result<RouteOutcome> {
    let raw: RawPushPayload = serde_json::from_slice(body)
        .map_err(|e| GatewayError::MalformedPayload(format!("push: {e}")))?;

    let Some(deliverable_id) = claim_ref_deliverable_id(&raw.reference) else {
        return Ok(RouteOutcome::Ignored {
            reason: format!("push ref {:?} is not a claim ref", raw.reference),
        });
    };

    if raw.deleted {
        return Ok(RouteOutcome::Ignored {
            reason: format!("push ref {:?} deleted", raw.reference),
        });
    }

    if raw.after.trim().is_empty() || raw.after.chars().all(|c| c == '0') {
        return Err(GatewayError::MalformedPayload(
            "missing push.after sha".to_owned(),
        ));
    }

    let repository_full_name = raw
        .repository
        .map(|repo| repo.full_name)
        .unwrap_or_else(|| "unknown".to_owned());
    let snapshot_id = content_addressed_snapshot_id(
        "push",
        &serde_json::json!({
            "after_sha": raw.after,
            "before_sha": raw.before,
            "commits": raw.commits.len(),
            "deliverable_id": deliverable_id,
            "reference": raw.reference,
            "repository_full_name": repository_full_name,
        }),
    );

    Ok(RouteOutcome::Dispatch(CiEvent::PushSnapshot(
        PushSnapshotEvent {
            reference: raw.reference,
            deliverable_id,
            before_sha: raw.before,
            after_sha: raw.after,
            repository_full_name,
            commits: raw.commits.len(),
            deleted: raw.deleted,
            snapshot_id,
        },
    )))
}

fn claim_ref_deliverable_id(reference: &str) -> Option<String> {
    let suffix = reference.strip_prefix("refs/heads/claims/")?;
    let deliverable_id = suffix.trim();
    (!deliverable_id.is_empty()).then(|| deliverable_id.to_owned())
}

fn content_addressed_snapshot_id(kind: &str, canonical: &Value) -> String {
    let bytes = serde_json::to_string(canonical).unwrap_or_else(|_| canonical.to_string());
    let digest = Sha256::digest(bytes.as_bytes());
    format!("{kind}:sha256:{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr_body(action: &str, base: &str, draft: bool) -> Vec<u8> {
        format!(
            r#"{{"action":"{action}","number":42,
                 "pull_request":{{"number":42,
                   "base":{{"ref":"{base}","sha":"basesha"}},
                   "head":{{"ref":"feature/x","sha":"headsha123"}},
                   "draft":{draft}}}}}"#
        )
        .into_bytes()
    }

    #[test]
    fn opened_against_dev_dispatches() {
        let outcome = route("pull_request", &pr_body("opened", "dev", false), "dev").unwrap();
        match outcome {
            RouteOutcome::Dispatch(CiEvent::PullRequest(ev)) => {
                assert_eq!(ev.action, PrAction::Opened);
                assert_eq!(ev.pr_number, 42);
                assert_eq!(ev.head_sha, "headsha123");
            }
            other => panic!("expected dispatch, got {other:?}"),
        }
    }

    #[test]
    fn synchronized_forgejo_spelling_dispatches() {
        let outcome = route(
            "pull_request",
            &pr_body("synchronized", "dev", false),
            "dev",
        )
        .unwrap();
        assert!(matches!(
            outcome,
            RouteOutcome::Dispatch(CiEvent::PullRequest(PullRequestEvent {
                action: PrAction::Synchronized,
                ..
            }))
        ));
    }

    #[test]
    fn synchronize_github_spelling_also_dispatches() {
        let outcome = route("pull_request", &pr_body("synchronize", "dev", false), "dev").unwrap();
        assert!(matches!(
            outcome,
            RouteOutcome::Dispatch(CiEvent::PullRequest(PullRequestEvent {
                action: PrAction::Synchronized,
                ..
            }))
        ));
    }

    #[test]
    fn wrong_base_branch_is_ignored() {
        let outcome = route("pull_request", &pr_body("opened", "main", false), "dev").unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }

    #[test]
    fn draft_pr_is_ignored() {
        let outcome = route("pull_request", &pr_body("opened", "dev", true), "dev").unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }

    #[test]
    fn unrelated_action_is_ignored() {
        let outcome = route(
            "pull_request",
            &pr_body("label_updated", "dev", false),
            "dev",
        )
        .unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }

    #[test]
    fn ping_is_ignored_not_errored() {
        let outcome = route("ping", b"{}", "dev").unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }

    #[test]
    fn unknown_event_is_unroutable() {
        let err = route("wiki", b"{}", "dev").unwrap_err();
        assert!(matches!(err, GatewayError::UnroutableEvent { .. }));
    }

    #[test]
    fn malformed_json_is_typed_error() {
        let err = route("pull_request", b"not json", "dev").unwrap_err();
        assert!(matches!(err, GatewayError::MalformedPayload(_)));
    }

    #[test]
    fn missing_head_sha_is_rejected() {
        let body = br#"{"action":"opened","number":7,
            "pull_request":{"number":7,
              "base":{"ref":"dev","sha":"b"},
              "head":{"ref":"f","sha":""},"draft":false}}"#;
        let err = route("pull_request", body, "dev").unwrap_err();
        assert!(matches!(err, GatewayError::MalformedPayload(_)));
    }

    #[test]
    fn issue_action_dispatches_snapshot() {
        let body = include_bytes!("../tests/fixtures/issue-label-snapshot.json");
        let outcome = route("issues", body, "dev").unwrap();
        match outcome {
            RouteOutcome::Dispatch(CiEvent::IssueSnapshot(ev)) => {
                assert_eq!(ev.action, IssueAction::Edited);
                assert_eq!(ev.issue_number, 108);
                assert_eq!(ev.labels, vec!["masterplan:IP-108", "sync-state:ready"]);
                assert!(ev.snapshot_id.starts_with("issue:sha256:"));
                assert_eq!(ev.snapshot_id.len(), "issue:sha256:".len() + 64);
            }
            other => panic!("expected issue snapshot, got {other:?}"),
        }
    }

    #[test]
    fn issue_snapshot_content_address_is_idempotent() {
        let body = include_bytes!("../tests/fixtures/issue-label-snapshot.json");
        let first = route("issues", body, "dev").unwrap();
        let second = route("issues", body, "dev").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn issue_snapshot_content_address_is_label_order_stable() {
        let first = route(
            "issues",
            br#"{"action":"edited",
                "issue":{"number":108,"title":"Board sync labels","state":"open",
                  "updated_at":"2026-05-27T19:00:00Z",
                  "labels":[{"name":"sync-state:ready"},{"name":"masterplan:IP-108"}]},
                "repository":{"full_name":"owner/repo"}}"#,
            "dev",
        )
        .unwrap();
        let second = route(
            "issues",
            br#"{"action":"edited",
                "issue":{"number":108,"title":"Board sync labels","state":"open",
                  "updated_at":"2026-05-27T19:00:00Z",
                  "labels":[{"name":"masterplan:IP-108"},{"name":"sync-state:ready"},{"name":"sync-state:ready"}]},
                "repository":{"full_name":"owner/repo"}}"#,
            "dev",
        )
        .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn unsupported_issue_action_is_ignored() {
        let body = br#"{"action":"pinned",
            "issue":{"number":108,"title":"board sync","state":"open"},
            "repository":{"full_name":"owner/repo"}}"#;
        let outcome = route("issue", body, "dev").unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }

    #[test]
    fn push_to_claim_ref_dispatches_snapshot() {
        let body = include_bytes!("../tests/fixtures/push-snapshot.json");
        let outcome = route("push", body, "dev").unwrap();
        match outcome {
            RouteOutcome::Dispatch(CiEvent::PushSnapshot(ev)) => {
                assert_eq!(ev.reference, "refs/heads/claims/ADR-0377-D2");
                assert_eq!(ev.deliverable_id, "ADR-0377-D2");
                assert_eq!(ev.after_sha, "def456");
                assert_eq!(ev.commits, 1);
                assert!(ev.snapshot_id.starts_with("push:sha256:"));
                assert_eq!(ev.snapshot_id.len(), "push:sha256:".len() + 64);
            }
            other => panic!("expected push snapshot, got {other:?}"),
        }
    }

    #[test]
    fn push_snapshot_content_address_is_idempotent() {
        let body = include_bytes!("../tests/fixtures/push-snapshot.json");
        let first = route("push", body, "dev").unwrap();
        let second = route("push", body, "dev").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn push_to_other_branch_is_ignored() {
        let body = br#"{"ref":"refs/heads/main","before":"abc","after":"def",
            "repository":{"full_name":"owner/repo"}}"#;
        let outcome = route("push", body, "dev").unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }

    #[test]
    fn push_to_dev_branch_is_ignored_for_board_projection() {
        let body = br#"{"ref":"refs/heads/dev","before":"abc","after":"def",
            "repository":{"full_name":"owner/repo"}}"#;
        let outcome = route("push", body, "dev").unwrap();
        assert!(matches!(outcome, RouteOutcome::Ignored { .. }));
    }
}
