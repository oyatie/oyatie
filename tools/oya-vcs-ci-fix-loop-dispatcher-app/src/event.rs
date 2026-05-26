//! Event shape posted to the agent dispatch queue.
//!
//! The dispatch queue today is the JSON registry file
//! `registry/ci-fix-loop-retry-budget.json::entries`.
//! Each emitted [`DispatchEvent`] is appended; the agent-runtime follow-up
//! (subagent_runtime_pending) will tail the file and claim each entry via
//! `oya claim --agent ci-fix-loop --intent fix-<source>-PR-<N>`.

use std::fmt;

/// Which dual-source path produced the bundle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixLoopSource {
    /// `workflow_run` failure on a PR.
    CiFailure,
    /// `repository_dispatch: pr-review-fix-requested` emitted by IP-004.
    PrReviewFixRequested,
}

impl FixLoopSource {
    /// Wire string used by the workflow + the schema enum.
    pub fn as_wire(&self) -> &'static str {
        match self {
            FixLoopSource::CiFailure => "ci-failure",
            FixLoopSource::PrReviewFixRequested => "pr-review-fix-requested",
        }
    }

    /// Intent slug used by the agent claim command.
    pub fn intent_slug(&self) -> &'static str {
        match self {
            FixLoopSource::CiFailure => "fix-CI-failure",
            FixLoopSource::PrReviewFixRequested => "fix-review-request",
        }
    }

    pub fn from_wire(value: &str) -> Result<Self, FixLoopSourceParseError> {
        match value {
            "ci-failure" => Ok(FixLoopSource::CiFailure),
            "pr-review-fix-requested" => Ok(FixLoopSource::PrReviewFixRequested),
            other => Err(FixLoopSourceParseError {
                value: other.to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixLoopSourceParseError {
    pub value: String,
}

impl fmt::Display for FixLoopSourceParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown FixLoopSource wire value '{}'; expected one of [ci-failure, pr-review-fix-requested]",
            self.value
        )
    }
}

impl std::error::Error for FixLoopSourceParseError {}

/// One agent-dispatch-queue entry.
///
/// The fields use the same names as the JSON registry to keep the
/// serialization mapping trivial (we don't depend on serde to keep the
/// crate's dependency footprint at zero).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchEvent {
    pub pr_number: u64,
    pub source: FixLoopSource,
    pub attempt: u32,
    pub bundle_path: String,
    pub emitted_at_epoch: u64,
}

impl DispatchEvent {
    /// Render the canonical agent-claim command-line for this dispatch
    /// event. The follow-up subagent runtime invokes the rendered command
    /// (or the equivalent grit-scaffold fallback per ADR-0054 during the
    /// transition).
    pub fn agent_claim_command(&self) -> String {
        format!(
            "oya claim --agent ci-fix-loop --intent \"{intent}-PR-{pr}\" --bundle {bundle}",
            intent = self.source.intent_slug(),
            pr = self.pr_number,
            bundle = self.bundle_path,
        )
    }

    /// JSON serialization without pulling serde. The output is stable
    /// (BTreeMap-like key order: alphabetical) so the registry file is
    /// diff-friendly.
    pub fn to_json_object(&self) -> String {
        format!(
            "{{\"attempt\":{attempt},\"bundle_path\":{bundle},\"emitted_at_epoch\":{epoch},\"pr_number\":{pr},\"source\":{source}}}",
            attempt = self.attempt,
            bundle = json_string(&self.bundle_path),
            epoch = self.emitted_at_epoch,
            pr = self.pr_number,
            source = json_string(self.source.as_wire()),
        )
    }
}

/// Minimal JSON-string escaper.
pub(crate) fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_loop_source_round_trips_wire_value() {
        for source in [
            FixLoopSource::CiFailure,
            FixLoopSource::PrReviewFixRequested,
        ] {
            assert_eq!(FixLoopSource::from_wire(source.as_wire()), Ok(source));
        }
    }

    #[test]
    fn fix_loop_source_rejects_unknown_wire_value() {
        assert!(FixLoopSource::from_wire("merge-queue-failure").is_err());
    }

    #[test]
    fn agent_claim_command_includes_source_pr_and_bundle() {
        let event = DispatchEvent {
            pr_number: 42,
            source: FixLoopSource::CiFailure,
            attempt: 2,
            bundle_path: "evidence/pipeline-maturity-glue/ip-005-fix-loop/42/2.json".to_string(),
            emitted_at_epoch: 1_715_000_000,
        };
        let cmd = event.agent_claim_command();
        assert!(cmd.contains("fix-CI-failure-PR-42"));
        assert!(cmd.contains("evidence/pipeline-maturity-glue/ip-005-fix-loop/42/2.json"));
    }

    #[test]
    fn dispatch_event_to_json_object_is_deterministic_and_sorted() {
        let event = DispatchEvent {
            pr_number: 7,
            source: FixLoopSource::PrReviewFixRequested,
            attempt: 1,
            bundle_path: "evidence/ip-005-fix-loop/7/1.json".to_string(),
            emitted_at_epoch: 1_715_000_000,
        };
        let json = event.to_json_object();
        assert!(json.starts_with("{\"attempt\":1,"));
        assert!(json.contains("\"source\":\"pr-review-fix-requested\""));
        assert!(json.ends_with("}"));
    }

    #[test]
    fn json_string_escapes_quotes_and_control_chars() {
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_string("a\nb"), "\"a\\nb\"");
    }
}
