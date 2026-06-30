//! Engineering Excellence Council merge-gate kernel (M01-P14-IP-003).
//!
//! Pure I/O-free Rust replacement for the earlier Node.js merge-gate
//! script (retired 2026-05-14 per user directive "no shellscript no mjs
//! etc all rust"). Runners feed a PR body string to [`parse_council_signature`]
//! and check [`CouncilSignature::is_approved`] before allowing the merge
//! queue to admit a ChangeSet.
//!
//! Approved verdicts: `APPROVE` and `APPROVE_WITH_NITS`. Both an
//! `Architect:` line and a `Critic:` line must carry an approved verdict
//! for the gate to pass.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CouncilVerdict {
    Approve,
    ApproveWithNits,
    Reject,
    Defer,
}

impl CouncilVerdict {
    pub fn parse(token: &str) -> Option<Self> {
        match token.to_ascii_uppercase().as_str() {
            "APPROVE" => Some(Self::Approve),
            "APPROVE_WITH_NITS" => Some(Self::ApproveWithNits),
            "REJECT" => Some(Self::Reject),
            "DEFER" => Some(Self::Defer),
            _ => None,
        }
    }

    pub fn is_approved(self) -> bool {
        matches!(self, Self::Approve | Self::ApproveWithNits)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "APPROVE",
            Self::ApproveWithNits => "APPROVE_WITH_NITS",
            Self::Reject => "REJECT",
            Self::Defer => "DEFER",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CouncilEntry {
    pub name: String,            // data_class: INTERNAL_ONLY
    pub verdict: CouncilVerdict, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CouncilSignature {
    pub architect: Option<CouncilEntry>, // data_class: INTERNAL_ONLY
    pub critic: Option<CouncilEntry>,    // data_class: INTERNAL_ONLY
}

impl CouncilSignature {
    pub fn is_approved(&self) -> bool {
        match (&self.architect, &self.critic) {
            (Some(a), Some(c)) => a.verdict.is_approved() && c.verdict.is_approved(),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MergeGateError {
    MissingArchitect,
    MissingCritic,
    ArchitectNotApproved { verdict: CouncilVerdict },
    CriticNotApproved { verdict: CouncilVerdict },
}

impl MergeGateError {
    pub fn message(&self) -> String {
        match self {
            Self::MissingArchitect => "missing Architect signature".to_owned(),
            Self::MissingCritic => "missing Critic signature".to_owned(),
            Self::ArchitectNotApproved { verdict } => {
                format!("Architect verdict not approved: {}", verdict.as_str())
            }
            Self::CriticNotApproved { verdict } => {
                format!("Critic verdict not approved: {}", verdict.as_str())
            }
        }
    }
}

/// Parse a PR body and extract the Council signature block.
///
/// Block boundary: starts at a line matching `## Council Signature`
/// (case-insensitive), ends at the next `## ` heading or EOF. Within
/// the block, lines matching `Architect:` or `Critic:` followed by a
/// name and `(VERDICT)` populate the corresponding entry.
pub fn parse_council_signature(body: &str) -> CouncilSignature {
    let mut in_block = false;
    let mut sig = CouncilSignature::default();

    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if is_council_header(line) {
            in_block = true;
            continue;
        }
        if line.starts_with("## ") && in_block {
            break;
        }
        if !in_block {
            continue;
        }

        let stripped = strip_bullet_prefix(line);
        let Some((role, tail)) = split_role(stripped) else {
            continue;
        };
        let Some((name, verdict)) = parse_verdict_tail(tail) else {
            continue;
        };
        let entry = CouncilEntry {
            name: name.to_owned(),
            verdict,
        };
        match role {
            Role::Architect => sig.architect = Some(entry),
            Role::Critic => sig.critic = Some(entry),
        }
    }
    sig
}

/// Run the gate: parse + validate. Returns `Ok(signature)` if both
/// roles are present and approved; otherwise the first reason for
/// failure.
pub fn evaluate(body: &str) -> Result<CouncilSignature, MergeGateError> {
    let sig = parse_council_signature(body);
    match (&sig.architect, &sig.critic) {
        (None, _) => Err(MergeGateError::MissingArchitect),
        (Some(_), None) => Err(MergeGateError::MissingCritic),
        (Some(a), Some(c)) => {
            if !a.verdict.is_approved() {
                Err(MergeGateError::ArchitectNotApproved { verdict: a.verdict })
            } else if !c.verdict.is_approved() {
                Err(MergeGateError::CriticNotApproved { verdict: c.verdict })
            } else {
                Ok(sig)
            }
        }
    }
}

/// Conflict-avoidance instruction emitted whenever a premature merge is detected.
pub const CONFLICT_AVOIDANCE_AFTER_PREMATURE_MERGE: &str =
    "preserve_wip_create_fresh_branch_from_current_dev";

/// Complete, adapter-neutral merge-hold input for one PR head.
///
/// Adapters (GitHub GraphQL/REST today, owned cloud-ci/API later) normalize their
/// observations into this shape. The kernel remains pure and decides only from
/// the data supplied here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeHoldPreflight {
    pub pr_number: u64,                      // data_class: INTERNAL_ONLY
    pub head_sha: String,                    // data_class: INTERNAL_ONLY
    pub observed_at: String,                 // data_class: INTERNAL_ONLY
    pub team_tasks: Vec<TeamTask>,           // data_class: INTERNAL_ONLY
    pub native_review: NativeReviewState,    // data_class: INTERNAL_ONLY
    pub required_contexts: Vec<String>,      // data_class: INTERNAL_ONLY
    pub required_checks: Vec<RequiredCheck>, // data_class: INTERNAL_ONLY
}

/// PR-linked team task state observed by the merge preflight.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamTask {
    pub id: String,     // data_class: INTERNAL_ONLY
    pub status: String, // data_class: INTERNAL_ONLY
}

/// Native PR review state observed for the exact PR head.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativeReviewState {
    pub review_decision: Option<String>, // data_class: INTERNAL_ONLY
    pub latest_reviews: Vec<ReviewRecord>, // data_class: INTERNAL_ONLY
    pub unresolved_requested_changes_threads: Vec<String>, // data_class: INTERNAL_ONLY
    pub newer_block_comments: Vec<String>, // data_class: INTERNAL_ONLY
}

/// One native review record from the SCM provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewRecord {
    pub author: String,       // data_class: INTERNAL_ONLY
    pub state: String,        // data_class: INTERNAL_ONLY
    pub submitted_at: String, // data_class: INTERNAL_ONLY
    pub commit_sha: String,   // data_class: INTERNAL_ONLY
}

/// One required status/check context observed for a PR.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredCheck {
    pub context: String,              // data_class: INTERNAL_ONLY
    pub head_sha: String,             // data_class: INTERNAL_ONLY
    pub status: String,               // data_class: INTERNAL_ONLY
    pub conclusion: Option<String>,   // data_class: INTERNAL_ONLY
    pub completed_at: Option<String>, // data_class: INTERNAL_ONLY
}

/// Machine-readable merge-hold packet. The same shape represents failure and success:
/// failure lists are empty only when `status == "merge_ready"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeHoldPacket {
    pub schema: &'static str,                    // data_class: INTERNAL_ONLY
    pub status: &'static str,                    // data_class: INTERNAL_ONLY
    pub pr_number: u64,                          // data_class: INTERNAL_ONLY
    pub head_sha: String,                        // data_class: INTERNAL_ONLY
    pub observed_at: String,                     // data_class: INTERNAL_ONLY
    pub non_terminal_team_task_ids: Vec<String>, // data_class: INTERNAL_ONLY
    pub native_review_blockers: Vec<String>,     // data_class: INTERNAL_ONLY
    pub non_green_or_stale_check_contexts: Vec<String>, // data_class: INTERNAL_ONLY
    pub terminal_team_task_ids: Vec<String>,     // data_class: INTERNAL_ONLY
    pub native_review_terminal_evidence: Vec<String>, // data_class: INTERNAL_ONLY
    pub green_check_contexts: Vec<String>,       // data_class: INTERNAL_ONLY
    pub oya_ci_required_green: bool,             // data_class: INTERNAL_ONLY
    pub conflict_avoidance: &'static str,        // data_class: INTERNAL_ONLY
}

impl MergeHoldPacket {
    pub fn is_merge_ready(&self) -> bool {
        self.status == "merge_ready"
    }

    /// Minimal JSON serialization so adapters can emit a concise packet without pulling
    /// serialization dependencies into the kernel.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema\":{},\"status\":{},\"pr_number\":{},\"head_sha\":{},\"observed_at\":{},\"non_terminal_team_task_ids\":{},\"native_review_blockers\":{},\"non_green_or_stale_check_contexts\":{},\"terminal_team_task_ids\":{},\"native_review_terminal_evidence\":{},\"green_check_contexts\":{},\"oya_ci_required_green\":{},\"conflict_avoidance\":{}}}",
            json_string(self.schema),
            json_string(self.status),
            self.pr_number,
            json_string(&self.head_sha),
            json_string(&self.observed_at),
            json_array(&self.non_terminal_team_task_ids),
            json_array(&self.native_review_blockers),
            json_array(&self.non_green_or_stale_check_contexts),
            json_array(&self.terminal_team_task_ids),
            json_array(&self.native_review_terminal_evidence),
            json_array(&self.green_check_contexts),
            self.oya_ci_required_green,
            json_string(self.conflict_avoidance)
        )
    }
}

/// Evaluate the merge hold for a single PR head.
///
/// Merge is ready only when all three planes agree on the same unchanged head:
/// terminal team tasks, terminal native review, and every required check green on
/// `input.head_sha`.
pub fn evaluate_merge_hold(input: &MergeHoldPreflight) -> MergeHoldPacket {
    let mut terminal_team_task_ids = Vec::new();
    let mut non_terminal_team_task_ids = Vec::new();
    for task in &input.team_tasks {
        if is_terminal_team_status(&task.status) {
            terminal_team_task_ids.push(task.id.clone());
        } else {
            non_terminal_team_task_ids.push(task.id.clone());
        }
    }

    let native_review_blockers = native_review_blockers(&input.native_review, &input.head_sha);
    let native_review_terminal_evidence = if native_review_blockers.is_empty() {
        native_review_terminal_evidence(&input.native_review, &input.head_sha)
    } else {
        Vec::new()
    };

    let mut green_check_contexts = Vec::new();
    let mut non_green_or_stale_check_contexts = Vec::new();
    for context in &input.required_contexts {
        if required_context_green_on_head(context, &input.head_sha, &input.required_checks) {
            green_check_contexts.push(context.clone());
        } else {
            non_green_or_stale_check_contexts.push(check_blocker_reason(
                context,
                &input.head_sha,
                &input.required_checks,
            ));
        }
    }
    if !input
        .required_contexts
        .iter()
        .any(|context| context == "oya-ci-required")
    {
        non_green_or_stale_check_contexts
            .push("oya-ci-required:missing-required-context".to_owned());
    }

    let oya_ci_required_green = green_check_contexts
        .iter()
        .any(|context| context == "oya-ci-required");
    let ready = non_terminal_team_task_ids.is_empty()
        && native_review_blockers.is_empty()
        && non_green_or_stale_check_contexts.is_empty()
        && oya_ci_required_green;

    MergeHoldPacket {
        schema: "oyatie.merge-hold-preflight.v1",
        status: if ready { "merge_ready" } else { "blocked" },
        pr_number: input.pr_number,
        head_sha: input.head_sha.clone(),
        observed_at: input.observed_at.clone(),
        non_terminal_team_task_ids,
        native_review_blockers,
        non_green_or_stale_check_contexts,
        terminal_team_task_ids,
        native_review_terminal_evidence,
        green_check_contexts,
        oya_ci_required_green,
        conflict_avoidance: CONFLICT_AVOIDANCE_AFTER_PREMATURE_MERGE,
    }
}

fn is_terminal_team_status(status: &str) -> bool {
    let normalized = status.trim().to_ascii_lowercase();
    normalized == "completed"
        || normalized == "closed_out_of_scope"
        || normalized
            .strip_prefix("closed_handed_to_fixuptask:")
            .is_some_and(|fixuptask_id| !fixuptask_id.trim().is_empty())
}

fn native_review_blockers(review: &NativeReviewState, head_sha: &str) -> Vec<String> {
    let mut blockers = Vec::new();
    match review.review_decision.as_deref().map(normalize_token) {
        Some(decision) if decision == "approved" => {}
        Some(decision) if decision.is_empty() => {
            blockers.push("reviewDecision:empty".to_owned());
        }
        Some(decision) => blockers.push(format!("reviewDecision:{decision}")),
        None => blockers.push("reviewDecision:missing".to_owned()),
    }

    if review.latest_reviews.is_empty() {
        blockers.push("latestReviews:empty".to_owned());
    } else if !review
        .latest_reviews
        .iter()
        .any(|record| normalize_token(&record.state) == "approved" && record.commit_sha == head_sha)
    {
        blockers.push("latestReviews:no-approved-record-on-head".to_owned());
    }

    blockers.extend(
        review
            .unresolved_requested_changes_threads
            .iter()
            .map(|id| format!("requested_changes_thread:{id}")),
    );
    blockers.extend(
        review
            .newer_block_comments
            .iter()
            .map(|id| format!("newer_block_comment:{id}")),
    );
    blockers
}

fn native_review_terminal_evidence(review: &NativeReviewState, head_sha: &str) -> Vec<String> {
    let mut evidence = Vec::new();
    if let Some(decision) = &review.review_decision {
        evidence.push(format!("reviewDecision:{}", normalize_token(decision)));
    }
    evidence.extend(
        review
            .latest_reviews
            .iter()
            .filter(|record| {
                normalize_token(&record.state) == "approved" && record.commit_sha == head_sha
            })
            .map(|record| {
                format!(
                    "approved_review:{}@{}:{}",
                    record.author, record.commit_sha, record.submitted_at
                )
            }),
    );
    evidence
}

fn required_context_green_on_head(context: &str, head_sha: &str, checks: &[RequiredCheck]) -> bool {
    checks.iter().any(|check| {
        check.context == context
            && check.head_sha == head_sha
            && normalize_token(&check.status) == "completed"
            && check
                .conclusion
                .as_deref()
                .is_some_and(|conclusion| normalize_token(conclusion) == "success")
    })
}

fn check_blocker_reason(context: &str, head_sha: &str, checks: &[RequiredCheck]) -> String {
    let matching: Vec<&RequiredCheck> = checks
        .iter()
        .filter(|check| check.context == context)
        .collect();
    if matching.is_empty() {
        return format!("{context}:missing");
    }
    if matching.iter().any(|check| {
        check.head_sha != head_sha
            && check
                .conclusion
                .as_deref()
                .is_some_and(|c| normalize_token(c) == "success")
    }) {
        return format!("{context}:stale-head");
    }
    let check = matching[0];
    let status = normalize_token(&check.status);
    if status != "completed" {
        return format!("{context}:status:{status}");
    }
    let conclusion = check
        .conclusion
        .as_deref()
        .map(normalize_token)
        .unwrap_or_else(|| "missing".to_owned());
    format!("{context}:conclusion:{conclusion}")
}

fn normalize_token(token: &str) -> String {
    token.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn json_array(values: &[String]) -> String {
    let mut out = String::from("[");
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&json_string(value));
    }
    out.push(']');
    out
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Role {
    Architect,
    Critic,
}

fn is_council_header(line: &str) -> bool {
    let normalized = line.trim_start_matches('#').trim().to_ascii_lowercase();
    normalized == "council signature" && line.starts_with("##") && !line.starts_with("###")
}

fn strip_bullet_prefix(line: &str) -> &str {
    line.trim_start()
        .strip_prefix("- ")
        .or_else(|| line.trim_start().strip_prefix("* "))
        .unwrap_or(line)
}

fn split_role(line: &str) -> Option<(Role, &str)> {
    let colon = line.find(':')?;
    let head = line[..colon].trim().to_ascii_lowercase();
    let tail = line[colon + 1..].trim();
    match head.as_str() {
        "architect" => Some((Role::Architect, tail)),
        "critic" => Some((Role::Critic, tail)),
        _ => None,
    }
}

fn parse_verdict_tail(tail: &str) -> Option<(&str, CouncilVerdict)> {
    let open = tail.rfind('(')?;
    let close = tail.rfind(')')?;
    if close < open + 1 {
        return None;
    }
    let verdict_token = tail[open + 1..close].trim();
    let verdict = CouncilVerdict::parse(verdict_token)?;
    let name = tail[..open].trim();
    if name.is_empty() {
        return None;
    }
    Some((name, verdict))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_passes() {
        let body =
            "## Council Signature\nArchitect: alice (APPROVE)\nCritic: bob (APPROVE_WITH_NITS)\n";
        let r = evaluate(body).unwrap();
        assert!(r.is_approved());
    }

    #[test]
    fn missing_critic_errors() {
        let body = "## Council Signature\nArchitect: alice (APPROVE)\n";
        assert!(matches!(evaluate(body), Err(MergeGateError::MissingCritic)));
    }

    #[test]
    fn missing_architect_errors() {
        let body = "## Council Signature\nCritic: bob (APPROVE)\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn rejected_verdict_flagged() {
        let body = "## Council Signature\nArchitect: alice (APPROVE)\nCritic: bob (REJECT)\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::CriticNotApproved {
                verdict: CouncilVerdict::Reject
            })
        ));
    }

    #[test]
    fn no_signature_block_errors() {
        let body = "## Summary\nstuff\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn block_terminates_at_next_heading() {
        let body =
            "## Council Signature\nArchitect: alice (APPROVE)\n\n## Other\nCritic: bob (APPROVE)\n";
        assert!(matches!(evaluate(body), Err(MergeGateError::MissingCritic)));
    }

    #[test]
    fn case_insensitive_header_and_verdict() {
        let body = "## council signature\narchitect: alice (approve)\ncritic: bob (approve)\n";
        assert!(evaluate(body).unwrap().is_approved());
    }

    #[test]
    fn bulleted_lines_accepted() {
        let body = "## Council Signature\n- Architect: alice (APPROVE)\n- Critic: bob (APPROVE_WITH_NITS)\n";
        assert!(evaluate(body).unwrap().is_approved());
    }

    #[test]
    fn defer_verdict_blocks_merge() {
        let body = "## Council Signature\nArchitect: alice (DEFER)\nCritic: bob (APPROVE)\n";
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::ArchitectNotApproved {
                verdict: CouncilVerdict::Defer
            })
        ));
    }

    #[test]
    fn nested_h3_council_header_not_recognized() {
        let body = "### Council Signature\nArchitect: alice (APPROVE)\nCritic: bob (APPROVE)\n";
        // H3 is not a top-level signature block; gate must fail.
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn empty_name_invalidates_line() {
        let body = "## Council Signature\nArchitect: (APPROVE)\nCritic: bob (APPROVE)\n";
        // Empty name → role line is dropped → MissingArchitect.
        assert!(matches!(
            evaluate(body),
            Err(MergeGateError::MissingArchitect)
        ));
    }

    #[test]
    fn verdict_parse_round_trips_known_tokens() {
        for v in [
            CouncilVerdict::Approve,
            CouncilVerdict::ApproveWithNits,
            CouncilVerdict::Reject,
            CouncilVerdict::Defer,
        ] {
            assert_eq!(CouncilVerdict::parse(v.as_str()), Some(v));
        }
    }
    fn ready_preflight() -> MergeHoldPreflight {
        MergeHoldPreflight {
            pr_number: 902,
            head_sha: "abc123".to_owned(),
            observed_at: "2026-06-30T03:00:00Z".to_owned(),
            team_tasks: vec![
                TeamTask {
                    id: "task-13".to_owned(),
                    status: "completed".to_owned(),
                },
                TeamTask {
                    id: "task-15".to_owned(),
                    status: "closed_handed_to_fixuptask:F-PR5-06".to_owned(),
                },
            ],
            native_review: NativeReviewState {
                review_decision: Some("APPROVED".to_owned()),
                latest_reviews: vec![ReviewRecord {
                    author: "reviewer-agent".to_owned(),
                    state: "APPROVED".to_owned(),
                    submitted_at: "2026-06-30T02:59:00Z".to_owned(),
                    commit_sha: "abc123".to_owned(),
                }],
                unresolved_requested_changes_threads: Vec::new(),
                newer_block_comments: Vec::new(),
            },
            required_contexts: vec!["oya-ci-required".to_owned(), "buck2".to_owned()],
            required_checks: vec![
                RequiredCheck {
                    context: "oya-ci-required".to_owned(),
                    head_sha: "abc123".to_owned(),
                    status: "COMPLETED".to_owned(),
                    conclusion: Some("SUCCESS".to_owned()),
                    completed_at: Some("2026-06-30T03:00:00Z".to_owned()),
                },
                RequiredCheck {
                    context: "buck2".to_owned(),
                    head_sha: "abc123".to_owned(),
                    status: "COMPLETED".to_owned(),
                    conclusion: Some("SUCCESS".to_owned()),
                    completed_at: Some("2026-06-30T03:00:00Z".to_owned()),
                },
            ],
        }
    }

    #[test]
    fn merge_hold_success_packet_requires_all_three_planes_on_head() {
        let packet = evaluate_merge_hold(&ready_preflight());

        assert!(packet.is_merge_ready());
        assert_eq!(packet.status, "merge_ready");
        assert_eq!(packet.pr_number, 902);
        assert_eq!(packet.head_sha, "abc123");
        assert_eq!(
            packet.terminal_team_task_ids,
            vec!["task-13".to_owned(), "task-15".to_owned()]
        );
        assert_eq!(
            packet.native_review_terminal_evidence,
            vec![
                "reviewDecision:approved".to_owned(),
                "approved_review:reviewer-agent@abc123:2026-06-30T02:59:00Z".to_owned(),
            ]
        );
        assert_eq!(
            packet.green_check_contexts,
            vec!["oya-ci-required".to_owned(), "buck2".to_owned()]
        );
        assert!(packet.oya_ci_required_green);
        assert!(packet.to_json().contains(
            "\"conflict_avoidance\":\"preserve_wip_create_fresh_branch_from_current_dev\""
        ));
    }

    #[test]
    fn merge_hold_blocks_nonterminal_tasks_review_and_checks() {
        let input = MergeHoldPreflight {
            pr_number: 893,
            head_sha: "new-head".to_owned(),
            observed_at: "2026-06-30T03:10:00Z".to_owned(),
            team_tasks: vec![
                TeamTask {
                    id: "task-14".to_owned(),
                    status: "in_progress".to_owned(),
                },
                TeamTask {
                    id: "task-16".to_owned(),
                    status: "closed_out_of_scope".to_owned(),
                },
            ],
            native_review: NativeReviewState {
                review_decision: None,
                latest_reviews: Vec::new(),
                unresolved_requested_changes_threads: vec!["thread-1".to_owned()],
                newer_block_comments: vec!["comment-99".to_owned()],
            },
            required_contexts: vec![
                "oya-ci-required".to_owned(),
                "buck2".to_owned(),
                "gate-live-postgres".to_owned(),
            ],
            required_checks: vec![
                RequiredCheck {
                    context: "oya-ci-required".to_owned(),
                    head_sha: "old-head".to_owned(),
                    status: "COMPLETED".to_owned(),
                    conclusion: Some("SUCCESS".to_owned()),
                    completed_at: Some("2026-06-30T03:09:00Z".to_owned()),
                },
                RequiredCheck {
                    context: "buck2".to_owned(),
                    head_sha: "new-head".to_owned(),
                    status: "COMPLETED".to_owned(),
                    conclusion: Some("FAILURE".to_owned()),
                    completed_at: Some("2026-06-30T03:09:30Z".to_owned()),
                },
                RequiredCheck {
                    context: "gate-live-postgres".to_owned(),
                    head_sha: "new-head".to_owned(),
                    status: "IN_PROGRESS".to_owned(),
                    conclusion: None,
                    completed_at: None,
                },
            ],
        };

        let packet = evaluate_merge_hold(&input);

        assert!(!packet.is_merge_ready());
        assert_eq!(packet.status, "blocked");
        assert_eq!(packet.non_terminal_team_task_ids, vec!["task-14"]);
        assert_eq!(
            packet.native_review_blockers,
            vec![
                "reviewDecision:missing".to_owned(),
                "latestReviews:empty".to_owned(),
                "requested_changes_thread:thread-1".to_owned(),
                "newer_block_comment:comment-99".to_owned(),
            ]
        );
        assert_eq!(
            packet.non_green_or_stale_check_contexts,
            vec![
                "oya-ci-required:stale-head".to_owned(),
                "buck2:conclusion:failure".to_owned(),
                "gate-live-postgres:status:in_progress".to_owned(),
            ]
        );
        assert!(!packet.oya_ci_required_green);
    }

    #[test]
    fn merge_hold_re_review_requested_and_stale_approval_are_not_ready() {
        let mut input = ready_preflight();
        input.native_review.review_decision = Some("REVIEW_REQUIRED".to_owned());
        input.native_review.latest_reviews[0].commit_sha = "old-head".to_owned();

        let packet = evaluate_merge_hold(&input);

        assert_eq!(
            packet.native_review_blockers,
            vec![
                "reviewDecision:review_required".to_owned(),
                "latestReviews:no-approved-record-on-head".to_owned(),
            ]
        );
        assert!(!packet.is_merge_ready());
    }

    #[test]
    fn merge_hold_requires_oya_ci_required_as_required_context() {
        let mut input = ready_preflight();
        input.required_contexts = vec!["buck2".to_owned()];

        let packet = evaluate_merge_hold(&input);

        assert_eq!(
            packet.non_green_or_stale_check_contexts,
            vec!["oya-ci-required:missing-required-context".to_owned()]
        );
        assert!(!packet.is_merge_ready());
    }
    #[test]
    fn merge_hold_rejects_noncanonical_terminal_task_aliases() {
        for status in [
            "complete",
            "done",
            "closed",
            "closed_handed_to_fixuptask:",
            "closed_handed_to_fixuptask:   ",
        ] {
            let mut input = ready_preflight();
            input.team_tasks = vec![TeamTask {
                id: format!("task-status-{status}"),
                status: status.to_owned(),
            }];

            let packet = evaluate_merge_hold(&input);

            assert_eq!(
                packet.non_terminal_team_task_ids,
                vec![format!("task-status-{status}")]
            );
            assert!(!packet.is_merge_ready());
        }
    }
}
