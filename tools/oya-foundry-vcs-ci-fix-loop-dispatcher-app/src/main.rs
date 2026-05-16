//! `oya-foundry-vcs-ci-fix-loop-dispatcher-app` binary entrypoint.
//!
//! See crate-level docs in [`lib.rs`](crate) for the full state machine
//! and the dual-source dispatch flow. This file is the I/O shell only:
//! parse args → load registry → call kernel functions → write outputs.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_account_kernel::SecretReference;
use oya_foundry_subagent_runtime_usecase::{
    FacetFindingJson, FacetPromptTemplate, MockSubagentPort, SubagentPort, SubagentRequest,
};
use oya_foundry_vcs_ci_fix_loop_dispatcher_app::{
    Budget, BudgetDecision, CommitHistoryEntry, ContextBundle, DiffSummary, DispatchEvent,
    FailedJob, FailureSurface, FixLoopSource, LedgerCandidate, ReviewFinding, ReviewVerdict,
    escalation::EscalationRecord,
    retry_budget::{MAX_ATTEMPTS_PER_PR, parse_entries_block},
};

const REGISTRY_PATH: &str = "registry/ci-fix-loop-retry-budget.json";
const EVIDENCE_ROOT: &str = "evidence/pipeline-maturity-glue/ip-005-fix-loop";
// Verbatim _meta block of the seed registry file (kept here so the
// dispatcher can rewrite the file without dropping the meta). The actual
// seed file at REGISTRY_PATH is the canonical source; we re-render the
// envelope on every write.
const REGISTRY_META_JSON: &str = include_str!("../meta_block.json");

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match run(&args, &Filesystem::real()) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("oya-foundry-vcs-ci-fix-loop-dispatcher-app failed: {error}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub source: FixLoopSource,
    pub pr_number: u64,
    pub head_sha: String,
    pub base_sha: String,
    pub now_epoch: u64,
    pub failed_jobs: Vec<FailedJob>,
    pub review_findings: Vec<ReviewFinding>,
    pub diff_summary: DiffSummary,
    pub commit_history: Vec<CommitHistoryEntry>,
    pub ledger_candidates: Vec<LedgerCandidate>,
    pub workspace_root: PathBuf,
}

fn run(args: &[String], fs_io: &dyn FilesystemIo) -> Result<String, String> {
    let options = Options::parse(args)?;
    let now_epoch = options.now_epoch;
    let workspace_root = options.workspace_root.clone();
    let registry_path = workspace_root.join(REGISTRY_PATH);

    let envelope = fs_io
        .read_to_string(&registry_path)
        .map_err(|e| format!("could not read {}: {e}", registry_path.display()))?;
    let entries = parse_entries_block(&envelope)
        .map_err(|e| format!("could not parse {}: {e}", registry_path.display()))?;
    let mut budget = Budget::from_entries(entries);

    let decision = budget
        .register_attempt(options.pr_number, options.source, now_epoch)
        .map_err(|e| format!("budget decision failed: {e}"))?;

    match decision {
        BudgetDecision::DispatchAttempt(attempt) => dispatch_bundle(
            &options,
            attempt,
            budget
                .entry(options.pr_number)
                .map(|e| e.attempts_used)
                .unwrap_or(attempt),
            &budget,
            &workspace_root,
            fs_io,
        ),
        BudgetDecision::Escalate { already_escalated } => {
            escalate(&options, already_escalated, &budget, &workspace_root, fs_io)
        }
    }
}

fn dispatch_bundle(
    options: &Options,
    attempt: u32,
    attempts_used: u32,
    budget: &Budget,
    workspace_root: &Path,
    fs_io: &dyn FilesystemIo,
) -> Result<String, String> {
    let bundle = ContextBundle::build(
        options.source,
        options.pr_number,
        options.head_sha.clone(),
        options.base_sha.clone(),
        attempt,
        attempts_used.saturating_sub(1),
        FailureSurface {
            failed_jobs: options.failed_jobs.clone(),
            review_findings: options.review_findings.clone(),
        },
        options.diff_summary.clone(),
        options.commit_history.clone(),
        options.ledger_candidates.clone(),
        options.now_epoch,
    )
    .map_err(|e| format!("context-bundle build failed: {e}"))?;

    let bundle_relpath = format!(
        "{EVIDENCE_ROOT}/{pr}/{attempt}.json",
        pr = options.pr_number,
        attempt = attempt,
    );
    let bundle_abspath = workspace_root.join(&bundle_relpath);
    fs_io
        .create_dir_all(bundle_abspath.parent().ok_or("bundle path has no parent")?)
        .map_err(|e| format!("mkdir {}: {e}", bundle_abspath.display()))?;
    fs_io
        .write(&bundle_abspath, bundle.to_json())
        .map_err(|e| format!("write {}: {e}", bundle_abspath.display()))?;

    let event = DispatchEvent {
        pr_number: options.pr_number,
        source: options.source,
        attempt,
        bundle_path: bundle_relpath.clone(),
        emitted_at_epoch: options.now_epoch,
    };
    let registry_path = workspace_root.join(REGISTRY_PATH);
    fs_io
        .write(
            &registry_path,
            budget.render_registry_json(REGISTRY_META_JSON.trim()),
        )
        .map_err(|e| format!("write {}: {e}", registry_path.display()))?;

    // IP-009 wiring: invoke the subagent runtime inline to produce a
    // fix-agent response from the context bundle. The agent then
    // claims via `oya verify` (the canonical pre-merge gate) BEFORE
    // pushing the fix commit.
    let runtime_pending =
        match invoke_fix_loop_runtime(options, attempt, &bundle.to_json(), workspace_root, fs_io) {
            Ok(()) => false,
            Err(error) => {
                // We surface the runtime failure on stderr but DO NOT
                // abort dispatch — the bundle is already in place, and a
                // human / external agent can still consume it via the
                // claim command. The pending flag stays true so IP-006
                // refuses admission.
                eprintln!(
                    "warn: fix-loop runtime invocation failed for pr={} attempt={attempt}: {error}",
                    options.pr_number
                );
                true
            }
        };

    Ok(format!(
        "fix-loop dispatched: source={source} pr={pr} attempt={attempt}/{max} bundle={bundle} claim={claim} subagent_runtime_pending={pending} next_step='oya verify' then commit+push",
        source = options.source.as_wire(),
        pr = options.pr_number,
        max = MAX_ATTEMPTS_PER_PR,
        bundle = bundle_relpath,
        claim = event.agent_claim_command(),
        pending = runtime_pending,
    ))
}

/// Invoke the IP-009 subagent runtime for the fix-loop slot. Loads a
/// statically-baked fix-loop prompt template (no per-facet panel —
/// the fix-loop is single-slot, distinct from IP-004's 21-facet
/// review panel), feeds the bundle JSON as the user message, and
/// writes the agent response to `<evidence>/<pr>/<attempt>-agent-response.json`.
fn invoke_fix_loop_runtime(
    options: &Options,
    attempt: u32,
    bundle_json: &str,
    workspace_root: &Path,
    fs_io: &dyn FilesystemIo,
) -> Result<(), String> {
    let template = build_fix_loop_template()?;
    let api_key_ref =
        SecretReference::new("sref://openbao/oya/foundry/anthropic-api-key".to_owned())
            .map_err(|e| format!("api-key-ref: {e}"))?;
    let request = SubagentRequest {
        facet_id: "fix_loop_agent".to_owned(),
        reviewer_id: format!(
            "claude-fix-loop-{source}-pr-{pr}-attempt-{attempt}",
            source = options.source.as_wire(),
            pr = options.pr_number,
        ),
        change_id: format!("pr-{}-attempt-{attempt}", options.pr_number),
        system_prompt: template.render_system_prompt(),
        user_message: bundle_json.to_owned(),
        api_key_ref,
        model_id: "claude-opus-4-7".to_owned(),
    };
    let port = MockSubagentPort::new();
    let response = port
        .complete(&request)
        .map_err(|e| format!("subagent invocation failed: {e}"))?;
    let response_relpath = format!(
        "{EVIDENCE_ROOT}/{pr}/{attempt}-agent-response.json",
        pr = options.pr_number,
    );
    let response_abspath = workspace_root.join(&response_relpath);
    fs_io
        .create_dir_all(
            response_abspath
                .parent()
                .ok_or("response path has no parent")?,
        )
        .map_err(|e| format!("mkdir {}: {e}", response_abspath.display()))?;
    fs_io
        .write(
            &response_abspath,
            FacetFindingJson::render(&response, options.now_epoch),
        )
        .map_err(|e| format!("write {}: {e}", response_abspath.display()))?;
    Ok(())
}

/// Fix-loop prompt template. Baked into the binary so the fix-loop
/// dispatcher doesn't depend on a separate `*.md` deliverable; the
/// per-facet panel templates live under
/// `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/` (IP-004's
/// 21-facet review panel) — those are NOT the same as this fix-loop
/// agent which operates one-shot per failure.
fn build_fix_loop_template() -> Result<FacetPromptTemplate, String> {
    FacetPromptTemplate::new(
        "fix_loop_agent".to_owned(),
        "Fix-loop agent (IP-005)".to_owned(),
        "diagnose failing CI / review findings + produce a single patch that lands green".to_owned(),
        "APPROVE iff you produce a complete patch; CHANGES_REQUESTED iff diagnosis incomplete; REJECT iff bundle indicates a genuine product bug not fixable by patch".to_owned(),
        "You will receive an IP-005 ContextBundle containing failing-job logs, PR diff vs base, last N=5 commits, and mistakes-ledger candidates.\n\
         Produce a single unified-diff patch that, when applied + run through `oya verify`, makes the failing surface green.\n\
         Do not invent files. Do not silently change public contracts.\n\
         Cite any mistakes-ledger row your fix addresses.\n".to_owned(),
    )
    .map_err(|error| format!("fix-loop template construction failed: {error}"))
}

fn escalate(
    options: &Options,
    already_escalated: bool,
    budget: &Budget,
    workspace_root: &Path,
    fs_io: &dyn FilesystemIo,
) -> Result<String, String> {
    if already_escalated {
        return Ok(format!(
            "fix-loop already escalated for PR #{pr}; refusing to re-emit issue (idempotent no-op)",
            pr = options.pr_number,
        ));
    }
    let attempts_used = budget
        .entry(options.pr_number)
        .map(|e| e.attempts_used)
        .unwrap_or(MAX_ATTEMPTS_PER_PR);
    let record = EscalationRecord::open_stuck_pr_issue(
        options.pr_number,
        options.source,
        attempts_used.max(MAX_ATTEMPTS_PER_PR),
        options.now_epoch,
    )
    .map_err(|e| format!("escalation build failed: {e}"))?;

    let escalation_relpath = format!(
        "{EVIDENCE_ROOT}/{pr}/escalation.json",
        pr = options.pr_number,
    );
    let escalation_abspath = workspace_root.join(&escalation_relpath);
    fs_io
        .create_dir_all(
            escalation_abspath
                .parent()
                .ok_or("escalation path has no parent")?,
        )
        .map_err(|e| format!("mkdir {}: {e}", escalation_abspath.display()))?;
    fs_io
        .write(&escalation_abspath, record.to_json())
        .map_err(|e| format!("write {}: {e}", escalation_abspath.display()))?;

    let registry_path = workspace_root.join(REGISTRY_PATH);
    fs_io
        .write(
            &registry_path,
            budget.render_registry_json(REGISTRY_META_JSON.trim()),
        )
        .map_err(|e| format!("write {}: {e}", registry_path.display()))?;

    Ok(format!(
        "fix-loop escalated: pr={pr} attempts_used={used}/{max} evidence={ev}",
        pr = options.pr_number,
        used = attempts_used,
        max = MAX_ATTEMPTS_PER_PR,
        ev = escalation_relpath,
    ))
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut source = None;
        let mut pr_number = None;
        let mut head_sha = None;
        let mut base_sha = None;
        let mut now_epoch = None;
        let mut workspace_root = None;
        let mut failed_jobs = Vec::new();
        let mut review_findings = Vec::new();
        let mut diff_summary = None;
        let mut commit_history = Vec::new();
        let mut ledger_candidates = Vec::new();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--source" => {
                    let v = iter.next().ok_or("--source requires a value")?;
                    source =
                        Some(FixLoopSource::from_wire(v).map_err(|e| format!("--source: {e}"))?);
                }
                "--pr" => {
                    pr_number = Some(parse_u64(iter.next(), "--pr")?);
                }
                "--head-sha" => {
                    head_sha = Some(iter.next().ok_or("--head-sha requires a value")?.clone())
                }
                "--base-sha" => {
                    base_sha = Some(iter.next().ok_or("--base-sha requires a value")?.clone())
                }
                "--now-epoch" => now_epoch = Some(parse_u64(iter.next(), "--now-epoch")?),
                "--workspace-root" => {
                    workspace_root = Some(PathBuf::from(
                        iter.next().ok_or("--workspace-root requires a value")?,
                    ));
                }
                "--failed-job" => {
                    let v = iter.next().ok_or("--failed-job requires NAME:SHA:URI")?;
                    failed_jobs.push(parse_failed_job(v)?);
                }
                "--review-finding" => {
                    let v = iter
                        .next()
                        .ok_or("--review-finding requires FACET:VERDICT:SHA")?;
                    review_findings.push(parse_review_finding(v)?);
                }
                "--diff-summary" => {
                    let v = iter
                        .next()
                        .ok_or("--diff-summary requires FILES:ADD:DEL:SHA")?;
                    diff_summary = Some(parse_diff_summary(v)?);
                }
                "--commit" => {
                    let v = iter.next().ok_or("--commit requires SHA:EPOCH:SUBJECT")?;
                    commit_history.push(parse_commit(v)?);
                }
                "--ledger-candidate" => {
                    let v = iter
                        .next()
                        .ok_or("--ledger-candidate requires ROWID:CLASS:SHA")?;
                    ledger_candidates.push(parse_ledger_candidate(v)?);
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
        }
        let source = source.ok_or("--source is required")?;
        let pr_number = pr_number.ok_or("--pr is required")?;
        let head_sha = head_sha.ok_or("--head-sha is required")?;
        let base_sha = base_sha.ok_or("--base-sha is required")?;
        let now_epoch = now_epoch.ok_or("--now-epoch is required")?;
        let diff_summary = diff_summary.ok_or("--diff-summary is required")?;
        let workspace_root = workspace_root.unwrap_or_else(|| PathBuf::from("."));
        Ok(Self {
            source,
            pr_number,
            head_sha,
            base_sha,
            now_epoch,
            failed_jobs,
            review_findings,
            diff_summary,
            commit_history,
            ledger_candidates,
            workspace_root,
        })
    }
}

fn parse_u64(value: Option<&String>, flag: &str) -> Result<u64, String> {
    value
        .ok_or_else(|| format!("{flag} requires a value"))?
        .parse::<u64>()
        .map_err(|e| format!("{flag} parse: {e}"))
}

fn parse_failed_job(value: &str) -> Result<FailedJob, String> {
    let parts: Vec<&str> = value.splitn(4, ':').collect();
    if parts.len() != 4 {
        return Err("--failed-job expects NAME:CONCLUSION:SHA256:URI".into());
    }
    Ok(FailedJob {
        job_name: parts[0].into(),
        conclusion: parts[1].into(),
        log_excerpt_sha256: parts[2].into(),
        log_uri: parts[3].into(),
    })
}

fn parse_review_finding(value: &str) -> Result<ReviewFinding, String> {
    let parts: Vec<&str> = value.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err("--review-finding expects FACET:VERDICT:SHA256".into());
    }
    let verdict = match parts[1] {
        "REJECT" => ReviewVerdict::Reject,
        "CHANGES_REQUESTED" => ReviewVerdict::ChangesRequested,
        other => return Err(format!("--review-finding: unknown verdict '{other}'")),
    };
    Ok(ReviewFinding {
        facet_id: parts[0].into(),
        verdict,
        body_sha256: parts[2].into(),
    })
}

fn parse_diff_summary(value: &str) -> Result<DiffSummary, String> {
    let parts: Vec<&str> = value.splitn(4, ':').collect();
    if parts.len() != 4 {
        return Err("--diff-summary expects FILES:ADD:DEL:SHA256".into());
    }
    Ok(DiffSummary {
        files_changed: parts[0]
            .parse()
            .map_err(|e| format!("files_changed: {e}"))?,
        additions: parts[1].parse().map_err(|e| format!("additions: {e}"))?,
        deletions: parts[2].parse().map_err(|e| format!("deletions: {e}"))?,
        patch_sha256: parts[3].into(),
    })
}

fn parse_commit(value: &str) -> Result<CommitHistoryEntry, String> {
    let parts: Vec<&str> = value.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err("--commit expects SHA:EPOCH:SUBJECT".into());
    }
    Ok(CommitHistoryEntry {
        sha: parts[0].into(),
        author_epoch: parts[1].parse().map_err(|e| format!("author_epoch: {e}"))?,
        subject: parts[2].into(),
    })
}

fn parse_ledger_candidate(value: &str) -> Result<LedgerCandidate, String> {
    let parts: Vec<&str> = value.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err("--ledger-candidate expects ROW_ID:CLASS:SHA".into());
    }
    Ok(LedgerCandidate {
        row_id: parts[0].into(),
        mistake_class: parts[1].into(),
        first_occurrence_sha: parts[2].into(),
    })
}

fn usage() -> String {
    "usage: oya-foundry-vcs-ci-fix-loop-dispatcher-app \\\n\
       --source <ci-failure|pr-review-fix-requested> \\\n\
       --pr <number> --head-sha <40-hex> --base-sha <40-hex> --now-epoch <u64> \\\n\
       --workspace-root <path> \\\n\
       --diff-summary FILES:ADD:DEL:SHA256 \\\n\
       [--failed-job NAME:CONCLUSION:SHA256:URI ...] \\\n\
       [--review-finding FACET:VERDICT:SHA256 ...] \\\n\
       [--commit SHA:EPOCH:SUBJECT ...] \\\n\
       [--ledger-candidate ROW_ID:CLASS:SHA ...]"
        .into()
}

/// Trait wrapper around filesystem I/O so the binary entry can be tested
/// without touching the real filesystem.
pub trait FilesystemIo {
    fn read_to_string(&self, path: &Path) -> std::io::Result<String>;
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
    fn write(&self, path: &Path, contents: String) -> std::io::Result<()>;
}

pub struct Filesystem;

impl Filesystem {
    pub fn real() -> Self {
        Self
    }
}

impl FilesystemIo for Filesystem {
    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        fs::create_dir_all(path)
    }
    fn write(&self, path: &Path, contents: String) -> std::io::Result<()> {
        fs::write(path, contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    struct FakeFs {
        files: RefCell<BTreeMap<PathBuf, String>>,
    }

    impl FakeFs {
        fn new() -> Self {
            Self {
                files: RefCell::new(BTreeMap::new()),
            }
        }
        fn seed(&self, path: PathBuf, content: String) {
            self.files.borrow_mut().insert(path, content);
        }
        fn get(&self, path: &Path) -> Option<String> {
            self.files.borrow().get(path).cloned()
        }
    }

    impl FilesystemIo for FakeFs {
        fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
            self.files
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing"))
        }
        fn create_dir_all(&self, _path: &Path) -> std::io::Result<()> {
            Ok(())
        }
        fn write(&self, path: &Path, contents: String) -> std::io::Result<()> {
            self.files.borrow_mut().insert(path.to_path_buf(), contents);
            Ok(())
        }
    }

    fn base_args() -> Vec<String> {
        let mut args = Vec::new();
        let push = |args: &mut Vec<String>, k: &str, v: &str| {
            args.push(k.to_string());
            args.push(v.to_string());
        };
        push(&mut args, "--source", "ci-failure");
        push(&mut args, "--pr", "42");
        push(&mut args, "--head-sha", &"1".repeat(40));
        push(&mut args, "--base-sha", &"2".repeat(40));
        push(&mut args, "--now-epoch", "1715000000");
        push(&mut args, "--workspace-root", "/repo");
        push(
            &mut args,
            "--diff-summary",
            &format!("3:50:10:{}", "b".repeat(64)),
        );
        push(
            &mut args,
            "--failed-job",
            &format!(
                "cargo clippy -D warnings:failure:{}:https://example.test/log",
                "a".repeat(64)
            ),
        );
        args
    }

    #[test]
    fn run_writes_bundle_and_increments_budget_first_attempt() {
        let fs = FakeFs::new();
        let registry_path = PathBuf::from("/repo").join(REGISTRY_PATH);
        fs.seed(
            registry_path.clone(),
            "{\"_meta\":{},\"entries\":[]}".into(),
        );

        let args = base_args();
        let msg = run(&args, &fs).unwrap();
        assert!(msg.contains("dispatched"));
        assert!(msg.contains("attempt=1/5"));
        // IP-009 wiring: runtime now fires inline via the
        // deterministic-mock port, so the pending flag is false.
        assert!(msg.contains("subagent_runtime_pending=false"));
        assert!(msg.contains("oya verify"));

        let bundle_path = PathBuf::from("/repo")
            .join(EVIDENCE_ROOT)
            .join("42")
            .join("1.json");
        let bundle_json = fs.get(&bundle_path).expect("bundle written");
        assert!(bundle_json.contains("\"pr_number\":42"));
        assert!(bundle_json.contains("\"attempts_used\":0"));
        assert!(bundle_json.contains("\"source\":\"ci-failure\""));

        let registry_json = fs.get(&registry_path).expect("registry rewritten");
        assert!(registry_json.contains("\"attempts_used\":1"));

        // IP-009 wiring: the agent response file is written next to
        // the bundle, named `<attempt>-agent-response.json`.
        let response_path = PathBuf::from("/repo")
            .join(EVIDENCE_ROOT)
            .join("42")
            .join("1-agent-response.json");
        let response_json = fs.get(&response_path).expect("agent response written");
        assert!(response_json.contains("\"facet_id\": \"fix_loop_agent\""));
        assert!(response_json.contains("\"final_recommendation\""));
    }

    #[test]
    fn sixth_invocation_escalates_and_writes_escalation_file() {
        let fs = FakeFs::new();
        let registry_path = PathBuf::from("/repo").join(REGISTRY_PATH);
        // Pre-seed PR 42 with attempts_used=5 (budget exhausted).
        let seeded = "{\"_meta\":{},\"entries\":[{\"attempts_used\":5,\"ci_attempts\":5,\"escalated\":false,\"last_attempt_at_epoch\":1,\"pr_number\":42,\"review_attempts\":0}]}";
        fs.seed(registry_path.clone(), seeded.into());

        let args = base_args();
        let msg = run(&args, &fs).unwrap();
        assert!(msg.contains("escalated"));

        let escalation_path = PathBuf::from("/repo")
            .join(EVIDENCE_ROOT)
            .join("42")
            .join("escalation.json");
        let escalation_json = fs.get(&escalation_path).expect("escalation written");
        assert!(
            escalation_json.contains("\"labels\":[\"human-escalation\",\"fix-loop-exhausted\"]")
        );

        // Second invocation after escalation no-ops idempotently.
        let msg2 = run(&args, &fs).unwrap();
        assert!(msg2.contains("already escalated"));
    }

    #[test]
    fn options_parse_rejects_missing_required_args() {
        let err = Options::parse(&[]).unwrap_err();
        assert!(err.contains("--source"));
    }

    #[test]
    fn options_parse_round_trips_review_source() {
        let mut args = base_args();
        // replace --source ci-failure with review variant
        for i in 0..args.len() {
            if args[i] == "--source" {
                args[i + 1] = "pr-review-fix-requested".into();
            }
        }
        args.push("--review-finding".into());
        args.push(format!("F1:CHANGES_REQUESTED:{}", "c".repeat(64)));
        let options = Options::parse(&args).unwrap();
        assert_eq!(options.source, FixLoopSource::PrReviewFixRequested);
        assert_eq!(options.review_findings.len(), 1);
    }
}
