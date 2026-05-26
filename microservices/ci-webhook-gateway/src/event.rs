//! Forgejo webhook event parsing + the closed router table.
//!
//! The gateway acts on `pull_request` events whose base branch is the gated
//! target (default `dev`). Forgejo's `pull_request` payload mirrors Gitea's:
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

/// The outcome of routing a raw delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteOutcome {
    /// A PR event against the target branch that the pipeline should act on.
    Dispatch(PullRequestEvent),
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

    Ok(RouteOutcome::Dispatch(PullRequestEvent {
        action,
        pr_number,
        base_ref: raw.pull_request.base.ref_name,
        head_ref: raw.pull_request.head.ref_name,
        head_sha: raw.pull_request.head.sha,
        draft: raw.pull_request.draft,
    }))
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
            RouteOutcome::Dispatch(ev) => {
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
            RouteOutcome::Dispatch(PullRequestEvent {
                action: PrAction::Synchronized,
                ..
            })
        ));
    }

    #[test]
    fn synchronize_github_spelling_also_dispatches() {
        let outcome = route("pull_request", &pr_body("synchronize", "dev", false), "dev").unwrap();
        assert!(matches!(
            outcome,
            RouteOutcome::Dispatch(PullRequestEvent {
                action: PrAction::Synchronized,
                ..
            })
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
}
