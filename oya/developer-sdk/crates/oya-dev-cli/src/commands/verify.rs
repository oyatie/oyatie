//! `oya verify` — single canonical local-developer verification entry
//! point.
//!
//! Per ADR-0346 / Wave 15-ZA, `oya verify --ci-required` is the local CI
//! mirror. It directly runs the five mandatory mirror steps before it
//! returns success:
//! - D-1: cargo fmt --check
//! - D-2: cargo check --workspace --all-targets
//! - D-3: cargo clippy --workspace --all-targets -- -D warnings
//! - D-4: cargo nextest run --workspace
//! - D-5: oya gate run-all --ci-required
//!
//! The two advisory steps are D-6 `oya doc adr-index --write` and D-7
//! `oya lint adr-shape` for new ADRs in `origin/dev...HEAD`.
//!
//! `oya verify` without `--ci-required` intentionally preserves the old
//! thin-alias behavior and delegates to `oya gate run-all`.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus, Stdio};
use std::time::Instant;

use crate::terminal_verifier_harness::{parse_terminal_evidence_args, run_terminal_evidence};

use super::gate;

const MANDATORY_TOTAL: usize = 5;
const ADVISORY_TOTAL: usize = 2;
const PRE_PUSH_TOTAL: usize = 3;
const FACE_SETTLE_TARGET: &str =
    "//cloud/cloud-ci/gates/oya-cloud-ci-freshness-app:oya-cloud-ci-face-settle-bin";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct VerifyArgs {
    ci_required: bool,
    include_deferred: bool,
    skip_fmt: bool,
    skip_check: bool,
    skip_clippy: bool,
    skip_nextest: bool,
    skip_gate_run_all: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum StepState {
    Passed,
    Failed,
    Skipped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MirrorStep {
    CargoFmt,
    CargoCheck,
    CargoClippy,
    CargoNextest,
    GateRunAll,
    AdrIndex,
    AdrShape,
}

impl MirrorStep {
    fn id(self) -> &'static str {
        match self {
            MirrorStep::CargoFmt => "D-1",
            MirrorStep::CargoCheck => "D-2",
            MirrorStep::CargoClippy => "D-3",
            MirrorStep::CargoNextest => "D-4",
            MirrorStep::GateRunAll => "D-5",
            MirrorStep::AdrIndex => "D-6",
            MirrorStep::AdrShape => "D-7",
        }
    }

    fn command_display(self) -> &'static str {
        match self {
            MirrorStep::CargoFmt => "cargo fmt --check",
            MirrorStep::CargoCheck => "cargo check --workspace --all-targets",
            MirrorStep::CargoClippy => "cargo clippy --workspace --all-targets -- -D warnings",
            MirrorStep::CargoNextest => "cargo nextest run --workspace",
            MirrorStep::GateRunAll => "oya gate run-all --ci-required",
            MirrorStep::AdrIndex => "oya doc adr-index --write",
            MirrorStep::AdrShape => "oya lint adr-shape",
        }
    }
}

fn mandatory_steps() -> [MirrorStep; MANDATORY_TOTAL] {
    [
        MirrorStep::CargoFmt,
        MirrorStep::CargoCheck,
        MirrorStep::CargoClippy,
        MirrorStep::CargoNextest,
        MirrorStep::GateRunAll,
    ]
}

fn advisory_steps() -> [MirrorStep; ADVISORY_TOTAL] {
    [MirrorStep::AdrIndex, MirrorStep::AdrShape]
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StepOutcome {
    state: StepState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VerifyInvalid {
    message: String,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    if args.iter().any(|arg| arg == "--terminal-evidence") {
        return run_terminal_evidence_entry(args, usage);
    }
    if args.iter().any(|arg| arg == "--pre-push") {
        return run_pre_push_entry(args, usage);
    }
    if let Some(pos) = args.iter().position(|arg| arg == "--from-results") {
        return run_from_results(args.get(pos + 1).map(String::as_str), usage);
    }
    if args.iter().any(|arg| arg == "--affected") {
        return run_affected_entry(args, usage);
    }
    if !args.iter().any(|arg| arg == "--ci-required") {
        return run_gate_alias(args, usage);
    }

    let args = match parse_verify_args(args, usage) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error.message);
            return ExitCode::from(2);
        }
    };

    if std::env::var("OYA_VERIFY_RUNNING").as_deref() == Ok("1") {
        eprintln!("oya verify: recursive invocation refused");
        return ExitCode::from(2);
    }

    match run_ci_required_mirror(args) {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("{}", error.message);
            ExitCode::from(2)
        }
    }
}

fn run_terminal_evidence_entry(args: Vec<String>, usage: &str) -> ExitCode {
    let args = match parse_terminal_evidence_args(args, usage) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error.message);
            return ExitCode::from(2);
        }
    };
    match run_terminal_evidence(args) {
        Ok(run) => {
            println!("{}", run.stdout_json);
            run.exit
        }
        Err(error) => {
            eprintln!("{}", error.message);
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PrePushArgs {
    base: String,
}

fn run_pre_push_entry(args: Vec<String>, usage: &str) -> ExitCode {
    let args = match parse_pre_push_args(args, usage) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error.message);
            return ExitCode::from(2);
        }
    };
    if std::env::var("OYA_VERIFY_RUNNING").as_deref() == Ok("1") {
        eprintln!("oya verify: recursive invocation refused");
        return ExitCode::from(2);
    }

    match run_pre_push_self_verify(args) {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("{}", error.message);
            ExitCode::from(2)
        }
    }
}

fn parse_pre_push_args(args: Vec<String>, usage: &str) -> Result<PrePushArgs, VerifyInvalid> {
    let mut base = "origin/dev".to_string();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--pre-push" => {}
            "--base" => {
                let Some(value) = iter.next() else {
                    return Err(VerifyInvalid {
                        message: format!("oya verify: --base requires a <ref> argument\n{usage}"),
                    });
                };
                base = value;
            }
            other => {
                return Err(VerifyInvalid {
                    message: format!("oya verify: unknown flag {other:?}\n{usage}"),
                });
            }
        }
    }
    Ok(PrePushArgs { base })
}

fn run_pre_push_self_verify(args: PrePushArgs) -> Result<ExitCode, VerifyInvalid> {
    let workspace_root = workspace_root()?;
    let root_display = workspace_root.display().to_string();
    let affected_script = workspace_root.join("infra/ci/buck2-affected-gate.sh");

    let outcomes = vec![
        run_pre_push_step(
            "P-1",
            "freshness gate (Cargo.lock + generated-face byte drift)",
            "Autofix guidance (freshness): run `cargo metadata >/dev/null` for Cargo.lock drift; run `infra/ci/materialize-cloud-ci-generated-faces.sh .` only after content changes are committed, then create a faces-only settle commit.",
            || {
                run_inherited(
                    "oya",
                    &[
                        "gate",
                        "validate",
                        "freshness",
                        "--repo-root",
                        root_display.as_str(),
                    ],
                    workspace_root.as_path(),
                )
            },
        )?,
        run_pre_push_step(
            "P-2",
            "generated-face settle check",
            "Autofix guidance (faces): commit content changes first; then run `buck2 run //cloud/cloud-ci/gates/oya-cloud-ci-freshness-app:oya-cloud-ci-face-settle-bin -- --repo-root . --settle`; commit only the generated-face paths.",
            || {
                run_inherited(
                    "buck2",
                    &[
                        "run",
                        FACE_SETTLE_TARGET,
                        "--",
                        "--repo-root",
                        root_display.as_str(),
                    ],
                    workspace_root.as_path(),
                )
            },
        )?,
        run_pre_push_step(
            "P-3",
            "Buck2 affected-set build/test",
            &format!(
                "Autofix guidance (affected-set): rerun `infra/ci/buck2-affected-gate.sh {} HEAD`; if owner/rdeps resolution is fatal, add or repair the relevant BUCK target/rust_test wiring; if a target fails, fix that target before pushing.",
                args.base
            ),
            || {
                run_inherited_path(
                    &affected_script,
                    &[args.base.as_str(), "HEAD"],
                    workspace_root.as_path(),
                )
            },
        )?,
    ];

    let passed = outcomes
        .iter()
        .filter(|outcome| outcome.state == StepState::Passed)
        .count();
    let failed = outcomes
        .iter()
        .any(|outcome| outcome.state == StepState::Failed);
    let status = if failed { "FAIL" } else { "PASS" };
    println!("oya verify --pre-push: {status} ({passed}/{PRE_PUSH_TOTAL})");

    if failed {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn run_pre_push_step<F>(
    id: &str,
    label: &str,
    guidance: &str,
    run: F,
) -> Result<StepOutcome, VerifyInvalid>
where
    F: FnOnce() -> Result<ExitStatus, VerifyInvalid>,
{
    println!("=== {id}: {label} ===");
    let start = Instant::now();
    let status = run()?;
    let elapsed = start.elapsed().as_secs_f32();
    if status.success() {
        println!("--- {id}: PASS ({elapsed:.1}s) ---");
        Ok(StepOutcome {
            state: StepState::Passed,
        })
    } else {
        let exit = status
            .code()
            .map(|code| format!("exit {code}"))
            .unwrap_or_else(|| "signal termination".into());
        println!("--- {id}: FAIL ({exit}, {elapsed:.1}s) ---");
        println!("{guidance}");
        Ok(StepOutcome {
            state: StepState::Failed,
        })
    }
}

/// `oya verify --from-results <junit.xml>` — ADR-0360 O2 gate-only overlay:
/// derive the test verdict from the lane's nextest JUnit report instead of
/// re-running cargo. Unparseable/absent report is a FAILURE, never a silent PASS.
fn run_from_results(path: Option<&str>, usage: &str) -> ExitCode {
    let Some(path) = path else {
        eprintln!("oya verify: --from-results requires a <junit.xml> path\n{usage}");
        return ExitCode::from(2);
    };
    let xml = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("oya verify --from-results: cannot read {path:?}: {error}");
            return ExitCode::from(2);
        }
    };
    match super::verify_results::parse_junit_summary(&xml) {
        Ok(summary) => {
            println!(
                "oya verify --from-results {path}: tests={}, failures={}, errors={} -> {}",
                summary.tests,
                summary.failures,
                summary.errors,
                if summary.passed() { "PASS" } else { "FAIL" }
            );
            if summary.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("oya verify --from-results {path}: {message}");
            ExitCode::from(2)
        }
    }
}

fn run_gate_alias(args: Vec<String>, usage: &str) -> ExitCode {
    let mut forwarded = Vec::with_capacity(args.len() + 1);
    forwarded.push("run-all".to_string());
    forwarded.extend(args);
    gate::run(forwarded, usage)
}

/// `oya verify --affected [--base <ref>]` — ADR-0360 O1 presubmit mode. Narrows
/// the cargo mirror to the affected reverse-dependency closure (or skips cargo
/// for non-Rust-only changes); the governance gates always run. `--ci-required`
/// remains the authoritative full mirror (the trunk backstop).
fn run_affected_entry(args: Vec<String>, usage: &str) -> ExitCode {
    use super::verify_affected::{self, BuildScope};

    let mut verify_args = VerifyArgs::default();
    let mut base = "dev".to_string();
    let mut iter = args.into_iter().peekable();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--affected" => {}
            "--base" => match iter.next() {
                Some(value) => base = value,
                None => {
                    eprintln!("oya verify: --base requires a <ref> argument\n{usage}");
                    return ExitCode::from(2);
                }
            },
            "--include-deferred" => verify_args.include_deferred = true,
            "--skip-fmt" => verify_args.skip_fmt = true,
            "--skip-check" => verify_args.skip_check = true,
            "--skip-clippy" => verify_args.skip_clippy = true,
            "--skip-nextest" => verify_args.skip_nextest = true,
            "--skip-gate-run-all" | "--skip-gates" => verify_args.skip_gate_run_all = true,
            other => {
                eprintln!("oya verify: unknown flag {other:?}\n{usage}");
                return ExitCode::from(2);
            }
        }
    }

    let repo_root = match workspace_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("{}", error.message);
            return ExitCode::from(2);
        }
    };

    let changed = match verify_affected::changed_files(repo_root.as_path(), &base) {
        Ok(files) => files,
        Err(message) => {
            eprintln!("oya verify --affected: {message}");
            return ExitCode::from(2);
        }
    };
    let (members, rdeps) = match verify_affected::workspace_graph(repo_root.as_path()) {
        Ok(graph) => graph,
        Err(message) => {
            eprintln!("oya verify --affected: {message}");
            return ExitCode::from(2);
        }
    };

    let scope = verify_affected::classify(&changed, &members, &rdeps);
    let targeting = match &scope {
        BuildScope::Full => {
            println!(
                "oya verify --affected (base={base}): {} changed file(s) -> FULL workspace mirror (full-build trigger)",
                changed.len()
            );
            CargoTargeting::Workspace
        }
        BuildScope::NoRust => {
            println!(
                "oya verify --affected (base={base}): {} changed file(s), no Rust impact -> SKIP cargo; gates only",
                changed.len()
            );
            CargoTargeting::Skip
        }
        BuildScope::Crates(crates) => {
            println!(
                "oya verify --affected (base={base}): {} changed file(s) -> {} affected crate(s): {}",
                changed.len(),
                crates.len(),
                crates.join(", ")
            );
            CargoTargeting::Packages(crates.clone())
        }
    };

    if std::env::var("OYA_VERIFY_RUNNING").as_deref() == Ok("1") {
        eprintln!("oya verify: recursive invocation refused");
        return ExitCode::from(2);
    }

    match run_mirror(verify_args, targeting) {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("{}", error.message);
            ExitCode::from(2)
        }
    }
}

fn parse_verify_args(args: Vec<String>, usage: &str) -> Result<VerifyArgs, VerifyInvalid> {
    let mut parsed = VerifyArgs::default();
    for flag in args {
        match flag.as_str() {
            "--ci-required" => parsed.ci_required = true,
            "--include-deferred" => parsed.include_deferred = true,
            "--skip-fmt" => parsed.skip_fmt = true,
            "--skip-check" => parsed.skip_check = true,
            "--skip-clippy" => parsed.skip_clippy = true,
            "--skip-nextest" => parsed.skip_nextest = true,
            "--skip-gate-run-all" | "--skip-gates" => parsed.skip_gate_run_all = true,
            other => {
                return Err(VerifyInvalid {
                    message: format!("oya verify: unknown flag {other:?}\n{usage}"),
                });
            }
        }
    }
    if !parsed.ci_required {
        return Err(VerifyInvalid {
            message: format!("oya verify: --ci-required is required for CI mirror mode\n{usage}"),
        });
    }
    Ok(parsed)
}

/// How the cargo mirror steps are targeted. `--ci-required` always uses
/// [`CargoTargeting::Workspace`] (the authoritative full mirror); `--affected`
/// narrows to [`CargoTargeting::Packages`] or skips cargo entirely
/// ([`CargoTargeting::Skip`]) per ADR-0360 O1.
#[derive(Clone, Debug, Eq, PartialEq)]
enum CargoTargeting {
    Workspace,
    Packages(Vec<String>),
    Skip,
}

impl CargoTargeting {
    fn package_flags(&self) -> Vec<String> {
        match self {
            CargoTargeting::Packages(crates) => crates
                .iter()
                .flat_map(|c| ["-p".to_string(), c.clone()])
                .collect(),
            _ => Vec::new(),
        }
    }

    fn check_args(&self) -> Vec<String> {
        let mut a = vec!["check".to_string()];
        match self {
            CargoTargeting::Packages(_) => a.extend(self.package_flags()),
            _ => a.push("--workspace".to_string()),
        }
        a.push("--all-targets".to_string());
        a
    }

    fn clippy_args(&self) -> Vec<String> {
        let mut a = vec!["clippy".to_string()];
        match self {
            CargoTargeting::Packages(_) => a.extend(self.package_flags()),
            _ => a.push("--workspace".to_string()),
        }
        a.extend(["--all-targets", "--", "-D", "warnings"].map(String::from));
        a
    }

    fn nextest_args(&self) -> Vec<String> {
        let mut a = vec!["nextest".to_string(), "run".to_string()];
        match self {
            CargoTargeting::Packages(_) => a.extend(self.package_flags()),
            _ => a.push("--workspace".to_string()),
        }
        a
    }
}

fn run_ci_required_mirror(args: VerifyArgs) -> Result<ExitCode, VerifyInvalid> {
    run_mirror(args, CargoTargeting::Workspace)
}

fn run_mirror(args: VerifyArgs, targeting: CargoTargeting) -> Result<ExitCode, VerifyInvalid> {
    let workspace_root = workspace_root()?;
    warn_for_skip_flags(&args);

    let skip_cargo = matches!(targeting, CargoTargeting::Skip);
    let check_args = targeting.check_args();
    let clippy_args = targeting.clippy_args();
    let nextest_args = targeting.nextest_args();

    let [fmt, check, clippy, nextest, gate_run_all] = mandatory_steps();
    let mut mandatory = Vec::with_capacity(MANDATORY_TOTAL);
    mandatory.push(run_or_skip(fmt, args.skip_fmt || skip_cargo, || {
        run_inherited("cargo", &["fmt", "--check"], workspace_root.as_path())
    })?);
    mandatory.push(run_or_skip(check, args.skip_check || skip_cargo, || {
        let refs: Vec<&str> = check_args.iter().map(String::as_str).collect();
        run_inherited("cargo", &refs, workspace_root.as_path())
    })?);
    mandatory.push(run_or_skip(clippy, args.skip_clippy || skip_cargo, || {
        let refs: Vec<&str> = clippy_args.iter().map(String::as_str).collect();
        run_inherited("cargo", &refs, workspace_root.as_path())
    })?);
    mandatory.push(run_or_skip(
        nextest,
        args.skip_nextest || skip_cargo,
        || {
            ensure_cargo_nextest(workspace_root.as_path())?;
            let refs: Vec<&str> = nextest_args.iter().map(String::as_str).collect();
            run_inherited("cargo", &refs, workspace_root.as_path())
        },
    )?);
    mandatory.push(run_or_skip(gate_run_all, args.skip_gate_run_all, || {
        let mut gate_args = vec!["gate", "run-all", "--ci-required"];
        if args.include_deferred {
            gate_args.push("--include-deferred");
        }
        run_inherited("oya", &gate_args, workspace_root.as_path())
    })?);

    let [adr_index, _adr_shape] = advisory_steps();
    let mut advisory = Vec::with_capacity(ADVISORY_TOTAL);
    advisory.push(run_step(adr_index, || {
        run_inherited(
            "oya",
            &["doc", "adr-index", "--write"],
            workspace_root.as_path(),
        )
    })?);
    let d7 = run_adr_shape_advisory(workspace_root.as_path())?;
    let d7_blocks = d7.blocks_on_failure;
    advisory.push(d7.outcome);

    let mandatory_passed = mandatory
        .iter()
        .filter(|outcome| outcome.state == StepState::Passed)
        .count();
    let advisory_passed = advisory
        .iter()
        .filter(|outcome| outcome.state == StepState::Passed)
        .count();
    let mandatory_failed = mandatory
        .iter()
        .any(|outcome| outcome.state == StepState::Failed);
    let d7_failed_blocker = d7_blocks
        && advisory
            .last()
            .is_some_and(|outcome| outcome.state == StepState::Failed);
    let failed = mandatory_failed || d7_failed_blocker;
    let status = if failed { "FAIL" } else { "PASS" };

    println!(
        "oya verify: {status} (mandatory: {mandatory_passed}/{MANDATORY_TOTAL}, advisory: {advisory_passed}/{ADVISORY_TOTAL})"
    );

    if failed {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn workspace_root() -> Result<PathBuf, VerifyInvalid> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| invalid_start("git", &error))?;
    if !output.status.success() {
        return Err(VerifyInvalid {
            message: "oya verify: must be run inside the oyatie git repository".into(),
        });
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        return Err(VerifyInvalid {
            message: "oya verify: git did not return a workspace root".into(),
        });
    }
    Ok(PathBuf::from(root))
}

fn warn_for_skip_flags(args: &VerifyArgs) {
    let skipped = [
        (args.skip_fmt, "D-1 cargo fmt --check"),
        (args.skip_check, "D-2 cargo check --workspace --all-targets"),
        (
            args.skip_clippy,
            "D-3 cargo clippy --workspace --all-targets -- -D warnings",
        ),
        (args.skip_nextest, "D-4 cargo nextest run --workspace"),
        (args.skip_gate_run_all, "D-5 oya gate run-all --ci-required"),
    ];
    for (_, label) in skipped.iter().filter(|(skip, _)| *skip) {
        eprintln!("oya verify: warning: skipping {label}");
    }
    if std::env::var("CI").as_deref() == Ok("true") && skipped.iter().any(|(skip, _)| *skip) {
        eprintln!("oya verify: warning: skip flags are not intended for CI=true environments");
    }
}

fn run_or_skip<F>(step: MirrorStep, skip: bool, run: F) -> Result<StepOutcome, VerifyInvalid>
where
    F: FnOnce() -> Result<ExitStatus, VerifyInvalid>,
{
    let id = step.id();
    let command_display = step.command_display();
    if skip {
        println!("=== {id}: {command_display} ===");
        println!("--- {id}: SKIP (requested by flag) ---");
        return Ok(StepOutcome {
            state: StepState::Skipped,
        });
    }
    run_step(step, run)
}

fn run_step<F>(step: MirrorStep, run: F) -> Result<StepOutcome, VerifyInvalid>
where
    F: FnOnce() -> Result<ExitStatus, VerifyInvalid>,
{
    let id = step.id();
    let command_display = step.command_display();
    println!("=== {id}: {command_display} ===");
    let start = Instant::now();
    let status = run()?;
    let elapsed = start.elapsed().as_secs_f32();
    if status.success() {
        println!("--- {id}: PASS ({elapsed:.1}s) ---");
        Ok(StepOutcome {
            state: StepState::Passed,
        })
    } else {
        let exit = status
            .code()
            .map(|code| format!("exit {code}"))
            .unwrap_or_else(|| "signal termination".into());
        println!("--- {id}: FAIL ({exit}, {elapsed:.1}s) ---");
        Ok(StepOutcome {
            state: StepState::Failed,
        })
    }
}

fn run_inherited(program: &str, args: &[&str], cwd: &Path) -> Result<ExitStatus, VerifyInvalid> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env("OYA_VERIFY_RUNNING", "1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| invalid_start(program, &error))
}

fn run_inherited_path(
    program: &Path,
    args: &[&str],
    cwd: &Path,
) -> Result<ExitStatus, VerifyInvalid> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env("OYA_VERIFY_RUNNING", "1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| invalid_start(&program.display().to_string(), &error))
}

fn ensure_cargo_nextest(cwd: &Path) -> Result<(), VerifyInvalid> {
    let status = run_silent("cargo", &["nextest", "--version"], cwd)?;
    if status.success() {
        Ok(())
    } else {
        Err(VerifyInvalid {
            message:
                "oya verify: missing required tool cargo-nextest (`cargo nextest --version` failed)"
                    .into(),
        })
    }
}

fn run_silent(program: &str, args: &[&str], cwd: &Path) -> Result<ExitStatus, VerifyInvalid> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env("OYA_VERIFY_RUNNING", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| invalid_start(program, &error))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AdrShapeOutcome {
    outcome: StepOutcome,
    blocks_on_failure: bool,
}

fn run_adr_shape_advisory(cwd: &Path) -> Result<AdrShapeOutcome, VerifyInvalid> {
    let step = MirrorStep::AdrShape;
    let id = step.id();
    println!("=== {id}: {} ===", step.command_display());
    let start = Instant::now();
    let output = Command::new("git")
        .args([
            "diff",
            "--name-only",
            "--diff-filter=A",
            "origin/dev...HEAD",
            "--",
            "docs/decisions/ADR-*.md",
        ])
        .current_dir(cwd)
        .env("OYA_VERIFY_RUNNING", "1")
        .output()
        .map_err(|error| invalid_start("git", &error))?;

    if !output.status.success() {
        let elapsed = start.elapsed().as_secs_f32();
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        println!("--- {id}: FAIL (git diff failed, {elapsed:.1}s) ---");
        return Ok(AdrShapeOutcome {
            outcome: StepOutcome {
                state: StepState::Failed,
            },
            blocks_on_failure: false,
        });
    }

    let paths: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if paths.is_empty() {
        let elapsed = start.elapsed().as_secs_f32();
        println!("{id}: no newly added ADRs in origin/dev...HEAD");
        println!("--- {id}: PASS ({elapsed:.1}s) ---");
        return Ok(AdrShapeOutcome {
            outcome: StepOutcome {
                state: StepState::Passed,
            },
            blocks_on_failure: false,
        });
    }

    let mut failed = false;
    for path in &paths {
        let status = run_inherited("oya", &["lint", "adr-shape", path], cwd)?;
        failed |= !status.success();
    }
    let elapsed = start.elapsed().as_secs_f32();
    if failed {
        println!("--- {id}: FAIL (new ADR shape blocker, {elapsed:.1}s) ---");
        Ok(AdrShapeOutcome {
            outcome: StepOutcome {
                state: StepState::Failed,
            },
            blocks_on_failure: true,
        })
    } else {
        println!("--- {id}: PASS ({elapsed:.1}s) ---");
        Ok(AdrShapeOutcome {
            outcome: StepOutcome {
                state: StepState::Passed,
            },
            blocks_on_failure: true,
        })
    }
}

fn invalid_start(program: &str, error: &std::io::Error) -> VerifyInvalid {
    let message = if error.kind() == ErrorKind::NotFound {
        format!("oya verify: missing required tool {program:?}")
    } else {
        format!("oya verify: could not start {program:?}: {error}")
    };
    VerifyInvalid { message }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ci_required_flags() {
        let args = parse_verify_args(
            vec![
                "--ci-required".into(),
                "--include-deferred".into(),
                "--skip-fmt".into(),
                "--skip-check".into(),
                "--skip-clippy".into(),
                "--skip-nextest".into(),
                "--skip-gate-run-all".into(),
            ],
            "usage",
        )
        .expect("valid flags");

        assert!(args.ci_required);
        assert!(args.include_deferred);
        assert!(args.skip_fmt);
        assert!(args.skip_check);
        assert!(args.skip_clippy);
        assert!(args.skip_nextest);
        assert!(args.skip_gate_run_all);
    }

    #[test]
    fn parse_ci_required_accepts_adr_skip_gates_alias() {
        let args = parse_verify_args(vec!["--ci-required".into(), "--skip-gates".into()], "usage")
            .expect("alias");

        assert!(args.skip_gate_run_all);
    }

    #[test]
    fn parse_ci_required_rejects_unknown_flags() {
        let error = parse_verify_args(vec!["--ci-required".into(), "--bogus".into()], "usage")
            .expect_err("unknown flag");

        assert!(error.message.contains("unknown flag"));
    }
}
