#![forbid(unsafe_code)]

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::{Parser, Subcommand};
use lane_supervisor_app::{
    Clock, DispatchRowInput, LaneObservation, PrPresence, ReapOptions, WaitFile, derive_lane_id,
    dispatch_registration_row, dispatch_row, event_row_for_decision, is_unhealthy_reap_decision,
    iso8601_from_unix_seconds, parse_jsonl, prompt_from_brief_pointer, render_jsonl_row,
    summarize_lanes, terminal_status_requires_failed_reap, unix_seconds_from_datetime,
};
use serde_json::{Value, json};
use std::fs::{File, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_LEDGER: &str = ".omc/ultragoal/dispatch-ledger.jsonl";
const START_GATE_TIMEOUT_SECONDS: u64 = 300;

#[derive(Debug, Parser)]
#[command(name = "lane-supervisor")]
#[command(
    about = "Local bridge lane liveness supervisor; merge authority remains in cloud-ci/ci."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Dispatch {
        #[arg(long)]
        brief: PathBuf,
        #[arg(long)]
        worktree: PathBuf,
        #[arg(long)]
        branch: String,
        #[arg(long, default_value = "origin/dev@dispatch")]
        base: String,
        #[arg(long = "expected-hard")]
        expected_hard: Vec<String>,
        #[arg(long = "expected-soft")]
        expected_soft: Vec<String>,
        #[arg(long)]
        log: Option<PathBuf>,
        #[arg(long, default_value = DEFAULT_LEDGER)]
        ledger: PathBuf,
    },
    Reap {
        #[arg(long, default_value = DEFAULT_LEDGER)]
        ledger: PathBuf,
        #[arg(long, default_value_t = 30)]
        stall_minutes: i64,
    },
    Status {
        #[arg(long, default_value = DEFAULT_LEDGER)]
        ledger: PathBuf,
        #[arg(long)]
        json: bool,
    },
    #[command(hide = true)]
    InternalRunWorker {
        #[arg(long)]
        brief: PathBuf,
        #[arg(long)]
        worktree: PathBuf,
        #[arg(long)]
        log: PathBuf,
        #[arg(long)]
        wait_file: PathBuf,
        #[arg(long)]
        start_file: PathBuf,
        #[arg(long)]
        run_id: String,
    },
}

struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_seconds(&self) -> i64 {
        unix_seconds_from_datetime(Utc::now())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    match Cli::parse().command {
        Commands::Dispatch {
            brief,
            worktree,
            branch,
            base,
            expected_hard,
            expected_soft,
            log,
            ledger,
        } => dispatch(DispatchCommand {
            brief: &brief,
            worktree: &worktree,
            branch: &branch,
            base: &base,
            expected_hard: &expected_hard,
            expected_soft: &expected_soft,
            log: log.as_deref(),
            ledger: &ledger,
        }),
        Commands::Reap {
            ledger,
            stall_minutes,
        } => reap(&ledger, stall_minutes),
        Commands::Status { ledger, json } => status(&ledger, json),
        Commands::InternalRunWorker {
            brief,
            worktree,
            log,
            wait_file,
            start_file,
            run_id,
        } => internal_run_worker(&brief, &worktree, &log, &wait_file, &start_file, &run_id),
    }
}

struct DispatchCommand<'a> {
    brief: &'a Path,
    worktree: &'a Path,
    branch: &'a str,
    base: &'a str,
    expected_hard: &'a [String],
    expected_soft: &'a [String],
    log: Option<&'a Path>,
    ledger: &'a Path,
}

fn dispatch(command: DispatchCommand<'_>) -> Result<ExitCode> {
    let DispatchCommand {
        brief,
        worktree,
        branch,
        base,
        expected_hard,
        expected_soft,
        log,
        ledger,
    } = command;
    let repo_root = std::env::current_dir().context("failed to resolve current directory")?;
    let brief = resolve_brief_path(brief, &repo_root)?;
    let log_path = match log {
        Some(path) => path.to_path_buf(),
        None => default_log_path(branch),
    };
    let started_at = iso8601_from_unix_seconds(SystemClock.now_unix_seconds())?;
    let run_id = dispatch_run_id(branch)?;
    let wait_file = wait_file_for_log(&log_path, &run_id);
    let start_file = start_file_for_log(&log_path, &run_id);
    ensure_parent_dir(&log_path)?;
    ensure_parent_dir(&wait_file)?;
    ensure_parent_dir(&start_file)?;
    ensure_parent_dir(ledger)?;
    let brief_display = brief.to_string_lossy();
    let lane_id = derive_lane_id(branch, &brief_display);
    let registration_row = dispatch_registration_row(DispatchRowInput {
        lane_id: &lane_id,
        brief: &brief_display,
        worktree: &worktree.to_string_lossy(),
        branch,
        base,
        expected_hard_surfaces: expected_hard,
        expected_soft_surfaces: expected_soft,
        log: &log_path.to_string_lossy(),
        wait_file: &wait_file.to_string_lossy(),
        start_file: &start_file.to_string_lossy(),
        run_id: &run_id,
        at: started_at.clone(),
    });
    append_row(ledger, &registration_row)?;

    let executable = std::env::current_exe().context("failed to resolve current executable")?;
    let mut command = Command::new(executable);
    command
        .arg("internal-run-worker")
        .arg("--brief")
        .arg(&brief)
        .arg("--worktree")
        .arg(worktree)
        .arg("--log")
        .arg(&log_path)
        .arg("--wait-file")
        .arg(&wait_file)
        .arg("--start-file")
        .arg(&start_file)
        .arg("--run-id")
        .arg(&run_id)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    detach_command(&mut command);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            let reason = format!("failed to spawn lane worker wrapper: {err}");
            append_failed_row(ledger, &lane_id, &reason)?;
            return Err(anyhow!(reason));
        }
    };

    let pid = child.id();
    let row = dispatch_row(
        DispatchRowInput {
            lane_id: &lane_id,
            brief: &brief_display,
            worktree: &worktree.to_string_lossy(),
            branch,
            base,
            expected_hard_surfaces: expected_hard,
            expected_soft_surfaces: expected_soft,
            log: &log_path.to_string_lossy(),
            wait_file: &wait_file.to_string_lossy(),
            start_file: &start_file.to_string_lossy(),
            run_id: &run_id,
            at: started_at,
        },
        pid,
    );
    if let Err(err) = append_row(ledger, &row) {
        terminate_child(&mut child);
        return Err(err).context("failed to record dispatched lane after spawning wrapper");
    }
    if let Err(err) = release_start_gate(&start_file, &run_id) {
        terminate_child(&mut child);
        let reason = format!("failed to release lane worker start gate: {err}");
        let _ = append_failed_row(ledger, &lane_id, &reason);
        return Err(err).context("failed to release lane worker start gate");
    }
    println!(
        "dispatched lane_id={} branch={} supervisor_pid={} log={} wait_file={} start_file={} run_id={}",
        lane_id,
        branch,
        pid,
        log_path.display(),
        wait_file.display(),
        start_file.display(),
        run_id
    );
    Ok(ExitCode::SUCCESS)
}

fn reap(ledger: &Path, stall_minutes: i64) -> Result<ExitCode> {
    if stall_minutes <= 0 {
        bail!("--stall-minutes must be positive");
    }
    let rows = read_ledger(ledger)?;
    let summaries = summarize_lanes(&rows);
    let options = ReapOptions {
        stall_seconds: stall_minutes.saturating_mul(60),
    };
    let clock = SystemClock;
    let mut unhealthy = false;
    let mut appended = 0_u64;

    for lane in summaries.values() {
        if terminal_status_requires_failed_reap(&lane.status) {
            unhealthy = true;
        }

        let observation = observe_lane(lane)?;
        let decision = lane_supervisor_app::evaluate_reap(lane, &observation, options, &clock);
        if is_unhealthy_reap_decision(&decision) {
            unhealthy = true;
        }

        if let Some(row) = event_row_for_decision(
            lane,
            &decision,
            iso8601_from_unix_seconds(clock.now_unix_seconds())?,
        ) {
            append_row(ledger, &row)?;
            let status = row.status().unwrap_or("unknown-status");
            println!("{} -> {}", lane.lane_id, status);
            appended = appended.saturating_add(1);
        }
    }

    println!("reap appended {appended} ledger rows");
    if unhealthy {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn status(ledger: &Path, output_json: bool) -> Result<ExitCode> {
    let rows = read_ledger(ledger)?;
    let summaries = summarize_lanes(&rows);
    if output_json {
        let lanes: Vec<Value> = summaries
            .values()
            .map(|lane| {
                json!({
                    "lane_id": lane.lane_id,
                    "status": lane.status,
                    "branch": lane.branch,
                    "brief": lane.brief,
                    "worktree": lane.worktree,
                    "log": lane.log,
                    "wait_file": lane.wait_file,
                    "pid": lane.pid,
                    "at": lane.at,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "lanes": lanes }))?
        );
    } else {
        for lane in summaries.values() {
            let branch = lane.branch.as_deref().unwrap_or("-");
            let log = lane.log.as_deref().unwrap_or("-");
            println!("{}\t{}\t{}\t{}", lane.lane_id, lane.status, branch, log);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn internal_run_worker(
    brief: &Path,
    worktree: &Path,
    log: &Path,
    wait_file: &Path,
    start_file: &Path,
    run_id: &str,
) -> Result<ExitCode> {
    ensure_parent_dir(log)?;
    ensure_parent_dir(wait_file)?;
    if let Err(err) = wait_for_start_gate(
        start_file,
        run_id,
        Duration::from_secs(START_GATE_TIMEOUT_SECONDS),
    ) {
        write_wait_file(wait_file, run_id, 124)?;
        return Err(err).context("lane worker start gate was not released");
    }
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .with_context(|| format!("failed to open log {}", log.display()))?;
    let err_file = log_file
        .try_clone()
        .with_context(|| format!("failed to clone log {}", log.display()))?;
    let prompt = prompt_from_brief_pointer(&brief.to_string_lossy());
    let mut command = Command::new("codex");
    command
        .arg("exec")
        .arg("--dangerously-bypass-approvals-and-sandbox")
        .arg("-C")
        .arg(worktree)
        .arg(prompt)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(err_file));
    detach_command(&mut command);
    let status = command
        .status()
        .context("failed to run codex exec lane worker")?;

    let code = status.code().unwrap_or(125);
    write_wait_file(wait_file, run_id, i64::from(code))?;
    if code == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}

fn observe_lane(lane: &lane_supervisor_app::LaneSummary) -> Result<LaneObservation> {
    let (pr, pr_lookup_error) = match lane.branch.as_deref() {
        Some(branch) => match query_pr(branch)? {
            PrLookup::Known(pr) => (pr, None),
            PrLookup::Indeterminate(reason) => (None, Some(reason)),
        },
        None => (None, None),
    };
    let process_alive = match lane.pid {
        Some(pid) => Some(process_alive(pid)?),
        None => None,
    };
    let log_mtime_unix_seconds = match lane.log.as_deref() {
        Some(path) => modified_unix_seconds(Path::new(path))?,
        None => None,
    };
    let wait_exit_status = match lane.wait_file.as_deref() {
        Some(path) => read_wait_file(Path::new(path), lane.latest_row.get_str("run_id"))?,
        None => None,
    };
    Ok(LaneObservation {
        process_alive,
        log_mtime_unix_seconds,
        wait_exit_status,
        pr,
        pr_lookup_error,
    })
}

enum PrLookup {
    Known(Option<PrPresence>),
    Indeterminate(String),
}

fn query_pr(branch: &str) -> Result<PrLookup> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("list")
        .arg("--head")
        .arg(branch)
        .arg("--json")
        .arg("number")
        .arg("--limit")
        .arg("1")
        .output();
    let output = match output {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PrLookup::Indeterminate(
                "gh executable not found while checking PR state".to_owned(),
            ));
        }
        Err(err) => return Err(anyhow!("failed to run gh pr list: {err}")),
    };
    if !output.status.success() {
        return Ok(PrLookup::Indeterminate(format!(
            "gh pr list exited with status {} while checking branch {branch}",
            output.status
        )));
    }
    let value: Value = serde_json::from_slice(&output.stdout).context("failed to parse gh JSON")?;
    let Some(items) = value.as_array() else {
        return Ok(PrLookup::Indeterminate(
            "gh pr list returned non-array JSON".to_owned(),
        ));
    };
    let number = items
        .first()
        .and_then(|item| item.get("number"))
        .and_then(Value::as_i64);
    Ok(PrLookup::Known(number.map(|number| PrPresence { number })))
}

fn process_alive(pid: i64) -> Result<bool> {
    if pid <= 0 {
        return Ok(false);
    }
    let status = Command::new("/bin/kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(status) => Ok(status.success()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let fallback = Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .context("failed to run kill -0")?;
            Ok(fallback.success())
        }
        Err(err) => Err(anyhow!("failed to run kill -0: {err}")),
    }
}

fn read_wait_file(path: &Path, expected_run_id: Option<&str>) -> Result<Option<i64>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read wait file {}", path.display()))?;
    let wait: WaitFile = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse wait file {}", path.display()))?;
    if let Some(expected_run_id) = expected_run_id
        && wait.run_id.as_deref() != Some(expected_run_id)
    {
        return Ok(None);
    }
    Ok(Some(wait.exit_status))
}

fn modified_unix_seconds(path: &Path) -> Result<Option<i64>> {
    if !path.exists() {
        return Ok(None);
    }
    let modified = std::fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .modified()
        .with_context(|| format!("failed to read mtime for {}", path.display()))?;
    let duration = modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .with_context(|| format!("mtime before epoch for {}", path.display()))?;
    let seconds = i64::try_from(duration.as_secs())
        .with_context(|| format!("mtime does not fit i64 for {}", path.display()))?;
    Ok(Some(seconds))
}

fn read_ledger(path: &Path) -> Result<Vec<lane_supervisor_app::LedgerRow>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read ledger {}", path.display()))?;
    parse_jsonl(&content).map_err(Into::into)
}

fn append_row(path: &Path, row: &lane_supervisor_app::LedgerRow) -> Result<()> {
    ensure_parent_dir(path)?;
    let rendered = render_jsonl_row(row)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open ledger {}", path.display()))?;
    writeln!(file, "{rendered}")
        .with_context(|| format!("failed to append ledger {}", path.display()))
}

fn append_failed_row(ledger: &Path, lane_id: &str, reason: &str) -> Result<()> {
    let mut fields = serde_json::Map::new();
    fields.insert("lane_id".to_owned(), Value::String(lane_id.to_owned()));
    fields.insert("status".to_owned(), Value::String("failed".to_owned()));
    fields.insert("reason".to_owned(), Value::String(reason.to_owned()));
    fields.insert(
        "at".to_owned(),
        Value::String(iso8601_from_unix_seconds(SystemClock.now_unix_seconds())?),
    );
    append_row(
        ledger,
        &lane_supervisor_app::LedgerRow::from_fields(fields),
    )
}

fn release_start_gate(path: &Path, run_id: &str) -> Result<()> {
    ensure_parent_dir(path)?;
    let mut file = File::create(path)
        .with_context(|| format!("failed to create start gate {}", path.display()))?;
    writeln!(file, "{}", json!({ "run_id": run_id }))
        .with_context(|| format!("failed to write start gate {}", path.display()))
}

fn wait_for_start_gate(path: &Path, run_id: &str, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read start gate {}", path.display()))?;
            let value: Value = serde_json::from_str(&content)
                .with_context(|| format!("failed to parse start gate {}", path.display()))?;
            if value.get("run_id").and_then(Value::as_str) != Some(run_id) {
                bail!("start gate {} belongs to a different run", path.display());
            }
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("timed out waiting for start gate {}", path.display());
        }
        sleep(Duration::from_millis(100));
    }
}

fn write_wait_file(wait_file: &Path, run_id: &str, code: i64) -> Result<()> {
    let wait = WaitFile {
        run_id: Some(run_id.to_owned()),
        exit_status: code,
        exited_at: iso8601_from_unix_seconds(SystemClock.now_unix_seconds())?,
    };
    let rendered = serde_json::to_string(&wait).context("failed to render wait file")?;
    let mut file = File::create(wait_file)
        .with_context(|| format!("failed to create wait file {}", wait_file.display()))?;
    writeln!(file, "{rendered}")
        .with_context(|| format!("failed to write wait file {}", wait_file.display()))
}

fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn detach_command(command: &mut Command) {
    #[cfg(unix)]
    {
        command.process_group(0);
    }
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Ok(())
}

fn resolve_brief_path(brief: &Path, repo_root: &Path) -> Result<PathBuf> {
    let candidate = if brief.is_absolute() {
        brief.to_path_buf()
    } else {
        repo_root.join(brief)
    };
    let canonical = candidate
        .canonicalize()
        .with_context(|| format!("brief path {} does not exist", candidate.display()))?;
    if !canonical.is_file() {
        bail!("brief path {} is not a file", canonical.display());
    }
    Ok(canonical)
}

fn default_log_path(branch: &str) -> PathBuf {
    let safe_branch = branch.replace('/', "-");
    PathBuf::from(format!("/tmp/lane-{safe_branch}.log"))
}

fn wait_file_for_log(log: &Path, run_id: &str) -> PathBuf {
    let file_name = log
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "lane.log".to_owned());
    log.with_file_name(format!("{file_name}.{run_id}.wait.json"))
}

fn start_file_for_log(log: &Path, run_id: &str) -> PathBuf {
    let file_name = log
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "lane.log".to_owned());
    log.with_file_name(format!("{file_name}.{run_id}.start.json"))
}

fn dispatch_run_id(branch: &str) -> Result<String> {
    let safe_branch = branch.replace('/', "-");
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?;
    Ok(format!(
        "{}-{}-{}",
        safe_branch,
        std::process::id(),
        duration.as_nanos()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn dispatch_fails_before_spawn_when_registration_cannot_be_recorded() {
        let root = std::env::temp_dir().join(format!(
            "lane-supervisor-registration-fail-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("test temp root should be created");
        let ledger = root.join("dispatch-ledger.jsonl");
        fs::create_dir(&ledger).expect("ledger path should be an unwritable directory");
        let log = root.join("lane.log");

        let result = dispatch(DispatchCommand {
            brief: &root.join("BRIEF.md"),
            worktree: &root.join("worktree"),
            branch: "agent/test-lane",
            base: "origin/dev",
            expected_hard: &[],
            expected_soft: &[],
            log: Some(&log),
            ledger: &ledger,
        });

        assert!(result.is_err());
        assert!(!log.exists());
        let created_files = fs::read_dir(&root)
            .expect("test temp root should be readable")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .count();
        assert_eq!(created_files, 0);
        fs::remove_dir_all(&root).expect("test temp root should be removed");
    }

    #[test]
    fn relative_brief_is_resolved_to_existing_absolute_path() {
        let root = std::env::temp_dir().join(format!(
            "lane-supervisor-brief-resolve-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock should be after epoch")
                .as_nanos()
        ));
        let brief = root.join(".omc/ultragoal/BRIEF-g011-lane-supervisor.md");
        fs::create_dir_all(brief.parent().expect("brief should have a parent"))
            .expect("brief parent should be created");
        fs::write(&brief, "# brief\n").expect("brief should be written");

        let resolved = resolve_brief_path(
            Path::new(".omc/ultragoal/BRIEF-g011-lane-supervisor.md"),
            &root,
        )
        .expect("relative brief should resolve from the repo root");

        assert!(resolved.is_absolute());
        assert_eq!(
            resolved,
            brief.canonicalize().expect("brief should canonicalize")
        );
        fs::remove_dir_all(&root).expect("test temp root should be removed");
    }
}
