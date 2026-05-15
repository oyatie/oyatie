//! `oya-foundry-vcs-merge-queue-fix-loop-app` binary entrypoint.
//!
//! Driven by `.github/workflows/ci-failure-fix-loop.yml` AND on
//! every IP-004 dispatcher run: reads the admission log, drains pending
//! events into the scheduler, runs one or more ticks, persists the
//! tick-log registry, and emits eviction-escalation files when a PR's
//! retry budget exhausts.

// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_vcs_merge_queue_fix_loop_app::{
    AdmissionEventKind, ParkedReason, Scheduler, parse_admission_log_str,
    render_tick_log_registry,
};

const ADMISSION_LOG_PATH: &str = "registries/cross-cutting/merge-queue-admission-log.json";
const TICK_LOG_PATH: &str = "registries/cross-cutting/merge-queue-tick-log.json";
const EVIDENCE_ROOT: &str = "evidence/pipeline-maturity-glue/ip-006-merge-queue";
const TICK_LOG_META: &str = include_str!("../tick_log_meta.json");

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match run(&args, &RealFs) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("oya-foundry-vcs-merge-queue-fix-loop-app failed: {e}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub workspace_root: PathBuf,
    pub initial_head_sha: String,
    pub now_epoch: u64,
    pub max_ticks: u32,
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut workspace_root = None;
        let mut initial_head_sha = None;
        let mut now_epoch = None;
        let mut max_ticks = 10u32;
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--workspace-root" => {
                    workspace_root = Some(PathBuf::from(
                        iter.next().ok_or("--workspace-root requires a value")?,
                    ));
                }
                "--initial-head-sha" => {
                    initial_head_sha = Some(
                        iter.next()
                            .ok_or("--initial-head-sha requires a value")?
                            .clone(),
                    );
                }
                "--now-epoch" => {
                    now_epoch = Some(
                        iter.next()
                            .ok_or("--now-epoch requires a value")?
                            .parse::<u64>()
                            .map_err(|e| format!("--now-epoch: {e}"))?,
                    );
                }
                "--max-ticks" => {
                    max_ticks = iter
                        .next()
                        .ok_or("--max-ticks requires a value")?
                        .parse::<u32>()
                        .map_err(|e| format!("--max-ticks: {e}"))?;
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
        }
        Ok(Self {
            workspace_root: workspace_root.unwrap_or_else(|| PathBuf::from(".")),
            initial_head_sha: initial_head_sha
                .ok_or("--initial-head-sha is required (40-hex)")?,
            now_epoch: now_epoch.ok_or("--now-epoch is required")?,
            max_ticks,
        })
    }
}

pub fn run(args: &[String], fs_io: &dyn FilesystemIo) -> Result<String, String> {
    let opts = Options::parse(args)?;
    let admission_path = opts.workspace_root.join(ADMISSION_LOG_PATH);
    let admission_json = fs_io
        .read_to_string(&admission_path)
        .map_err(|e| format!("read {}: {e}", admission_path.display()))?;
    let events = parse_admission_log_str(&admission_json)
        .map_err(|e| format!("parse admission log: {e}"))?;

    let mut scheduler = Scheduler::new(&opts.initial_head_sha)
        .map_err(|e| format!("scheduler init: {e}"))?;
    let now_epoch = opts.now_epoch;
    let mut summary = SchedulerSummary::default();
    for event in events {
        // IP-009 admission-gate: refuse APPROVE events that still
        // carry `subagent_runtime_pending=true`. The flag is set by
        // IP-004's dispatcher when no real per-facet subagent panel
        // has produced findings — admitting under that condition
        // would let a PR merge without real review. We count
        // refusals separately and emit them in the run summary so an
        // external observer can verify convergence.
        if event.subagent_runtime_pending {
            summary.refused_for_pending_runtime += 1;
            continue;
        }
        match event.kind {
            AdmissionEventKind::PrReviewApproved => {
                scheduler
                    .admit(event.pr_number, event.changeset_id.clone())
                    .map_err(|e| format!("admit pr={}: {e}", event.pr_number))?;
                summary.admitted += 1;
            }
            AdmissionEventKind::PrReviewFixRequested => {
                // The PR may or may not be in the queue yet (event ordering
                // may put fix-requested first if the reviewer rejected
                // before admission). The integration crate only parks PRs
                // already admitted; otherwise it is a no-op (idempotent).
                if let Err(e) = scheduler.admit(event.pr_number, event.changeset_id.clone()) {
                    // Already admitted is fine; other errors are real.
                    if !matches!(
                        e,
                        oya_foundry_vcs_merge_queue_fix_loop_app::SchedulerError::PrAlreadyAdmitted(_)
                    ) {
                        return Err(format!("pre-admit pr={}: {e}", event.pr_number));
                    }
                }
                scheduler
                    .park(
                        event.pr_number,
                        event.head_sha.clone(),
                        ParkedReason::ReviewChangesRequested,
                        event.emitted_at_epoch,
                    )
                    .map_err(|e| format!("park pr={}: {e}", event.pr_number))?;
                summary.parked += 1;
            }
        }
    }
    let mut tick_actions = 0u32;
    while tick_actions < opts.max_ticks {
        let action = scheduler
            .tick(now_epoch + u64::from(tick_actions))
            .map_err(|e| format!("scheduler tick: {e}"))?;
        tick_actions += 1;
        if matches!(
            action,
            oya_foundry_vcs_merge_queue_fix_loop_app::TickAction::Idle
        ) {
            break;
        }
    }

    let tick_log_path = opts.workspace_root.join(TICK_LOG_PATH);
    fs_io
        .create_dir_all(
            tick_log_path
                .parent()
                .ok_or("tick log path has no parent")?,
        )
        .map_err(|e| format!("mkdir {}: {e}", tick_log_path.display()))?;
    fs_io
        .write(
            &tick_log_path,
            render_tick_log_registry(TICK_LOG_META.trim(), scheduler.tick_log()),
        )
        .map_err(|e| format!("write {}: {e}", tick_log_path.display()))?;

    // Per-tick evidence file for each tick (one JSON per tick).
    for tick in scheduler.tick_log() {
        let path = opts
            .workspace_root
            .join(EVIDENCE_ROOT)
            .join(format!("tick-{:04}.json", tick.tick_number));
        fs_io
            .create_dir_all(path.parent().ok_or("tick evidence has no parent")?)
            .map_err(|e| format!("mkdir {}: {e}", path.display()))?;
        fs_io
            .write(
                &path,
                render_tick_log_registry(TICK_LOG_META.trim(), std::iter::once(tick)),
            )
            .map_err(|e| format!("write {}: {e}", path.display()))?;
    }

    // IP-009 admission-gate: the integration layer now refuses
    // pending-flagged APPROVEs, so its own `subagent_runtime_pending`
    // flag in the success message reflects the COMPLETED state of
    // the queue: false iff at least one event was processed AND zero
    // events were refused for pending runtime, OR no events arrived
    // at all (idle pipeline). When refusals occurred, the marker
    // stays true so downstream observers know the queue isn't yet
    // accepting all upstream APPROVE traffic.
    let integration_pending = summary.refused_for_pending_runtime > 0;
    Ok(format!(
        "merge-queue: admitted={a} parked={p} refused_for_pending_runtime={r} ticks={t} queue_depth={qd} parked_count={pc} subagent_runtime_pending={pending}",
        a = summary.admitted,
        p = summary.parked,
        r = summary.refused_for_pending_runtime,
        t = tick_actions,
        qd = scheduler.queue_depth(),
        pc = scheduler.parked_count(),
        pending = integration_pending,
    ))
}

#[derive(Clone, Copy, Debug, Default)]
struct SchedulerSummary {
    admitted: u32,
    parked: u32,
    /// IP-009 admission-gate: count of APPROVE events refused
    /// because they still carried `subagent_runtime_pending=true`.
    refused_for_pending_runtime: u32,
}

fn usage() -> String {
    "usage: oya-foundry-vcs-merge-queue-fix-loop-app \\\n\
       --workspace-root <path> --initial-head-sha <40-hex> --now-epoch <u64> \\\n\
       [--max-ticks <u32>]"
        .into()
}

pub trait FilesystemIo {
    fn read_to_string(&self, path: &Path) -> std::io::Result<String>;
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
    fn write(&self, path: &Path, contents: String) -> std::io::Result<()>;
}

pub struct RealFs;

impl FilesystemIo for RealFs {
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
        fn seed(&self, p: PathBuf, c: String) {
            self.files.borrow_mut().insert(p, c);
        }
        fn get(&self, p: &Path) -> Option<String> {
            self.files.borrow().get(p).cloned()
        }
    }

    impl FilesystemIo for FakeFs {
        fn read_to_string(&self, p: &Path) -> std::io::Result<String> {
            self.files
                .borrow()
                .get(p)
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing"))
        }
        fn create_dir_all(&self, _p: &Path) -> std::io::Result<()> {
            Ok(())
        }
        fn write(&self, p: &Path, c: String) -> std::io::Result<()> {
            self.files.borrow_mut().insert(p.to_path_buf(), c);
            Ok(())
        }
    }

    fn base_args() -> Vec<String> {
        vec![
            "--workspace-root".into(),
            "/repo".into(),
            "--initial-head-sha".into(),
            "0".repeat(40),
            "--now-epoch".into(),
            "1715000000".into(),
            "--max-ticks".into(),
            "10".into(),
        ]
    }

    fn admission_json(events: &[(&str, u64, &str)]) -> String {
        admission_json_with_pending(events, false)
    }

    /// Same as `admission_json` but stamps every entry with the given
    /// `subagent_runtime_pending` flag. Used by the IP-009 admission-
    /// gate refusal test.
    fn admission_json_with_pending(events: &[(&str, u64, &str)], pending: bool) -> String {
        let body = events
            .iter()
            .map(|(kind, pr, cs)| {
                format!(
                    r#"{{"base_sha":"{base}","changeset_id":"{cs}","emitted_at_epoch":1,"head_sha":"{head}","kind":"{kind}","pr_number":{pr},"subagent_runtime_pending":{pending}}}"#,
                    base = "2".repeat(40),
                    head = "1".repeat(40),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"_meta\":{{}},\"entries\":[{body}]}}")
    }

    #[test]
    fn admit_and_tick_writes_tick_log() {
        let fs = FakeFs::new();
        let log = PathBuf::from("/repo").join(ADMISSION_LOG_PATH);
        fs.seed(log, admission_json(&[("pr-review-approved", 42, "cs_a")]));
        let args = base_args();
        let msg = run(&args, &fs).unwrap();
        assert!(msg.contains("admitted=1"));
        // IP-009 wiring: with no pending-flagged events, the
        // integration layer's own `subagent_runtime_pending` reports
        // false — the queue is admitting and no upstream pending
        // events were refused.
        assert!(msg.contains("subagent_runtime_pending=false"));
        assert!(msg.contains("refused_for_pending_runtime=0"));
        let tick_log_path = PathBuf::from("/repo").join(TICK_LOG_PATH);
        let tick_log = fs.get(&tick_log_path).expect("tick log written");
        assert!(tick_log.contains("\"kind\":\"merge\""));
        assert!(tick_log.contains("\"pr_number\":42"));
    }

    #[test]
    fn admission_gate_refuses_pending_approve_events() {
        // IP-009 admission-gate convergence guarantee: when IP-004's
        // dispatcher emits an APPROVE event but the runtime hasn't
        // produced findings (pending=true), IP-006 MUST refuse to
        // admit. Otherwise the PR would merge without real review.
        let fs = FakeFs::new();
        let log = PathBuf::from("/repo").join(ADMISSION_LOG_PATH);
        fs.seed(
            log,
            admission_json_with_pending(&[("pr-review-approved", 42, "cs_a")], true),
        );
        let args = base_args();
        let msg = run(&args, &fs).unwrap();
        assert!(msg.contains("admitted=0"));
        assert!(msg.contains("refused_for_pending_runtime=1"));
        assert!(msg.contains("subagent_runtime_pending=true"));
    }

    #[test]
    fn admission_gate_passes_through_complete_runtime_events() {
        // The complement of the previous test: with pending=false
        // explicitly set, the event flows through into Scheduler::admit.
        let fs = FakeFs::new();
        let log = PathBuf::from("/repo").join(ADMISSION_LOG_PATH);
        fs.seed(
            log,
            admission_json_with_pending(&[("pr-review-approved", 42, "cs_a")], false),
        );
        let args = base_args();
        let msg = run(&args, &fs).unwrap();
        assert!(msg.contains("admitted=1"));
        assert!(msg.contains("refused_for_pending_runtime=0"));
    }

    #[test]
    fn park_pr_review_fix_requested_keeps_in_queue() {
        // IP-006 acceptance: parked PR stays in queue; other admissible PRs continue.
        let fs = FakeFs::new();
        let log = PathBuf::from("/repo").join(ADMISSION_LOG_PATH);
        // A approved + fix-requested → parked; B approved.
        fs.seed(
            log,
            admission_json(&[
                ("pr-review-fix-requested", 101, "cs_a"),
                ("pr-review-approved", 102, "cs_b"),
            ]),
        );
        let msg = run(&base_args(), &fs).unwrap();
        assert!(msg.contains("admitted=1"));
        assert!(msg.contains("parked=1"));
        // After ticks, queue_depth should be 2 (A parked, B merged → Merged state still in queue).
        // parked_count=1 confirms A is parked.
        assert!(msg.contains("parked_count=1"));
        let tick_log_path = PathBuf::from("/repo").join(TICK_LOG_PATH);
        let tick_log = fs.get(&tick_log_path).expect("tick log written");
        // B should merge (PR 102), A should stay parked
        assert!(tick_log.contains("\"pr_number\":102"));
    }

    #[test]
    fn idle_tick_emitted_when_all_parked() {
        let fs = FakeFs::new();
        let log = PathBuf::from("/repo").join(ADMISSION_LOG_PATH);
        fs.seed(
            log,
            admission_json(&[("pr-review-fix-requested", 101, "cs_a")]),
        );
        let msg = run(&base_args(), &fs).unwrap();
        assert!(msg.contains("ticks=1"));
        let tick_log_path = PathBuf::from("/repo").join(TICK_LOG_PATH);
        let tick_log = fs.get(&tick_log_path).expect("tick log written");
        assert!(tick_log.contains("\"kind\":\"idle\""));
    }

    #[test]
    fn options_parse_requires_initial_head_sha() {
        let err = Options::parse(&[
            "--workspace-root".into(),
            "/x".into(),
            "--now-epoch".into(),
            "1".into(),
        ])
        .unwrap_err();
        assert!(err.contains("--initial-head-sha"));
    }
}
