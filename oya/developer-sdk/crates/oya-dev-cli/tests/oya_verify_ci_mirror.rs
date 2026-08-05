// ADR-0083 Tier 3: integration tests use `.expect()` / `.panic!()` to assert
// verifier CLI invariants with fixture commands.
#![allow(clippy::expect_used, clippy::panic)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn repo_root() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
        .strip_suffix("/crates/oya-dev-cli")
        .expect("manifest dir is under crates/oya-dev-cli")
}

#[test]
fn oya_verify_ci_required_runs_mandatory_mirror_and_advisory_steps() {
    let fixture = VerifyFixture::new("verify-pass");

    let output = fixture.run(&["verify", "--ci-required"], &[], "");

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("oya verify: PASS (mandatory: 5/5, advisory: 2/2)"));

    let log = fixture.log();
    assert_in_order(
        &log,
        &[
            "git rev-parse --show-toplevel",
            "cargo fmt --check",
            "cargo check --workspace --all-targets",
            "cargo clippy --workspace --all-targets -- -D warnings",
            "cargo nextest --version",
            "cargo nextest run --workspace",
            "oya gate run-all --ci-required",
            "oya doc adr-index --write",
            "git diff --name-only --diff-filter=A origin/dev...HEAD -- docs/decisions/ADR-*.md",
        ],
    );
}

#[test]
fn oya_verify_mandatory_failure_exits_one_after_running_later_steps() {
    let fixture = VerifyFixture::new("verify-mandatory-fail");

    let output = fixture.run(
        &["verify", "--ci-required"],
        &[(
            "FAKE_VERIFY_FAILURES",
            "cargo clippy --workspace --all-targets -- -D warnings",
        )],
        "",
    );

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("oya verify: FAIL (mandatory: 4/5, advisory: 2/2)"));

    let log = fixture.log();
    assert!(log.contains("cargo nextest run --workspace"));
    assert!(log.contains("oya gate run-all --ci-required"));
    assert!(log.contains("oya doc adr-index --write"));
}

#[test]
fn oya_verify_advisory_failure_does_not_fail_when_new_adr_lint_is_not_blocking() {
    let fixture = VerifyFixture::new("verify-advisory-fail");

    let output = fixture.run(
        &["verify", "--ci-required"],
        &[("FAKE_VERIFY_FAILURES", "oya doc adr-index --write")],
        "",
    );

    assert_success(&output);
    assert!(stdout(&output).contains("oya verify: PASS (mandatory: 5/5, advisory: 1/2)"));
}

#[test]
fn oya_verify_new_adr_shape_failure_blocks_exit_even_though_it_is_advisory_classified() {
    let fixture = VerifyFixture::new("verify-adr-shape-fail");

    let output = fixture.run(
        &["verify", "--ci-required"],
        &[(
            "FAKE_VERIFY_FAILURES",
            "oya lint adr-shape docs/decisions/ADR-9999-fixture.md",
        )],
        "docs/decisions/ADR-9999-fixture.md\n",
    );

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stdout(&output).contains("oya verify: FAIL (mandatory: 5/5, advisory: 1/2)"));
    assert!(
        fixture
            .log()
            .contains("oya lint adr-shape docs/decisions/ADR-9999-fixture.md")
    );
}

#[test]
fn oya_verify_skip_flags_suppress_their_mandatory_steps() {
    let fixture = VerifyFixture::new("verify-skip-flags");

    let output = fixture.run(
        &[
            "verify",
            "--ci-required",
            "--include-deferred",
            "--skip-fmt",
            "--skip-check",
            "--skip-clippy",
            "--skip-nextest",
            "--skip-gate-run-all",
        ],
        &[],
        "",
    );

    assert_success(&output);
    let log = fixture.log();
    assert!(!log.contains("cargo fmt --check"));
    assert!(!log.contains("cargo check --workspace --all-targets"));
    assert!(!log.contains("cargo clippy --workspace --all-targets -- -D warnings"));
    assert!(!log.contains("cargo nextest run --workspace"));
    assert!(!log.contains("oya gate run-all --ci-required"));
    assert!(stdout(&output).contains("oya verify: PASS (mandatory: 0/5, advisory: 2/2)"));
}

#[test]
fn oya_verify_include_deferred_threads_to_gate_run_all() {
    let fixture = VerifyFixture::new("verify-include-deferred");

    let output = fixture.run(&["verify", "--ci-required", "--include-deferred"], &[], "");

    assert_success(&output);
    assert!(
        fixture
            .log()
            .contains("oya gate run-all --ci-required --include-deferred")
    );
}

#[test]
fn oya_verify_unknown_ci_required_flag_exits_two() {
    let fixture = VerifyFixture::new("verify-invalid-flag");

    let output = fixture.run(&["verify", "--ci-required", "--bogus"], &[], "");

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("unknown flag"));
}

#[test]
fn oya_verify_pre_push_runs_freshness_faces_and_affected_set_before_push() {
    let fixture = VerifyFixture::new("verify-pre-push-pass");

    let output = fixture.run(&["verify", "--pre-push", "--base", "origin/dev"], &[], "");

    assert_success(&output);
    assert!(stdout(&output).contains("oya verify --pre-push: PASS (3/3)"));

    let root = fixture.root.display().to_string();
    let freshness = format!("oya gate validate freshness --repo-root {root}");
    let faces = format!(
        "buck2 run //cloud/cloud-ci/gates/oya-cloud-ci-freshness-app:oya-cloud-ci-face-settle-bin -- --repo-root {root}"
    );
    let log = fixture.log();
    assert_in_order(
        &log,
        &[
            "git rev-parse --show-toplevel",
            &freshness,
            &faces,
            "buck2-affected-gate.sh origin/dev HEAD",
        ],
    );
}

#[test]
fn oya_verify_pre_push_affected_failure_reports_actionable_autofix_guidance() {
    let fixture = VerifyFixture::new("verify-pre-push-affected-fail");

    let output = fixture.run(
        &["verify", "--pre-push", "--base", "origin/dev"],
        &[(
            "FAKE_VERIFY_FAILURES",
            "buck2-affected-gate.sh origin/dev HEAD",
        )],
        "",
    );

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.contains("oya verify --pre-push: FAIL (2/3)"));
    assert!(stdout.contains("Autofix guidance (affected-set)"));
    assert!(stdout.contains("infra/ci/buck2-affected-gate.sh origin/dev HEAD"));

    let log = fixture.log();
    assert!(log.contains("oya gate validate freshness --repo-root"));
    assert!(log.contains("oya-cloud-ci-face-settle-bin"));
    assert!(log.contains("buck2-affected-gate.sh origin/dev HEAD"));
}

#[test]
fn oya_verify_terminal_clean_checkout_emits_machine_readable_slice_result() {
    let fixture = VerifyFixture::new("verify-terminal-clean-checkout-pass");

    let output = fixture.run(
        &["verify", "--terminal-evidence", "clean-checkout"],
        &[],
        "",
    );

    assert_success(&output);
    let result: Value = serde_json::from_str(&stdout(&output)).expect("stdout is json");
    assert_eq!(
        result["schema_version"],
        "g013-terminal-verifier-harness.v1"
    );
    assert_eq!(result["evidence_class"], "clean-checkout");
    assert_eq!(result["claim_scope"], "slice_evidence");
    assert_eq!(result["outcome"], "pass");
    assert_eq!(result["full_platform_terminal_closure_claimed"], false);
    assert_eq!(
        result["dirty_paths"]
            .as_array()
            .expect("dirty_paths array")
            .len(),
        0
    );
    assert_eq!(result["checkout_ref"], "fixture-head-1234567890abcdef");

    let log = fixture.log();
    assert_in_order(
        &log,
        &[
            "git rev-parse --show-toplevel",
            "git rev-parse HEAD",
            "git status --short --untracked-files=all",
        ],
    );
}

#[test]
fn oya_verify_terminal_clean_checkout_fails_dirty_checkout_without_terminal_claim() {
    let fixture = VerifyFixture::new("verify-terminal-clean-checkout-dirty");

    let output = fixture.run(
        &["verify", "--terminal-evidence", "clean-checkout"],
        &[("FAKE_VERIFY_GIT_STATUS", " M src/lib.rs\n?? scratch.txt\n")],
        "",
    );

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let result: Value = serde_json::from_str(&stdout(&output)).expect("stdout is json");
    assert_eq!(result["evidence_class"], "clean-checkout");
    assert_eq!(result["outcome"], "fail");
    assert_eq!(result["full_platform_terminal_closure_claimed"], false);
    let dirty_paths = result["dirty_paths"].as_array().expect("dirty_paths array");
    assert_eq!(dirty_paths.len(), 2);
    assert!(dirty_paths.iter().any(|path| path == "M src/lib.rs"));
    assert!(dirty_paths.iter().any(|path| path == "?? scratch.txt"));
}

struct VerifyFixture {
    root: PathBuf,
    bin: PathBuf,
    log: PathBuf,
}

impl VerifyFixture {
    fn new(name: &str) -> Self {
        let root = temp_dir(name);
        let bin = root.join("bin");
        fs::create_dir_all(&bin).expect("fixture bin created");
        let fake = Path::new(env!("CARGO_BIN_EXE_fake-verify-command"));
        for name in ["cargo", "oya", "git"] {
            install_fake_command(fake, &bin.join(name));
        }
        install_fake_command(fake, &bin.join("buck2"));
        let affected_gate = root.join("infra/ci/buck2-affected-gate.sh");
        fs::create_dir_all(affected_gate.parent().expect("affected gate parent"))
            .expect("affected gate parent created");
        install_fake_command(fake, &affected_gate);
        Self {
            log: root.join("commands.log"),
            root,
            bin,
        }
    }

    fn run(&self, args: &[&str], envs: &[(&str, &str)], git_diff: &str) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_oya"));
        command
            .current_dir(&self.root)
            .args(args)
            // These integration tests are themselves run by
            // `oya verify --ci-required` during the CI mirror. The verifier
            // deliberately exports OYA_VERIFY_RUNNING=1 to child commands so a
            // real nested `oya verify` is refused. The fixture is different:
            // it launches a fresh top-level verifier against fake cargo/oya/git
            // shims, so clear the parent guard before asserting verifier
            // behavior.
            .env_remove("OYA_VERIFY_RUNNING")
            .env("PATH", fixture_path(&self.bin))
            .env("FAKE_VERIFY_LOG", &self.log)
            .env("FAKE_VERIFY_GIT_ROOT", &self.root)
            .env("FAKE_VERIFY_GIT_DIFF", git_diff);
        for (key, value) in envs {
            command.env(key, value);
        }
        command.output().expect("oya command runs")
    }

    fn log(&self) -> String {
        fs::read_to_string(&self.log).unwrap_or_default()
    }
}

impl Drop for VerifyFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn install_fake_command(fake: &Path, dest: &Path) {
    std::os::unix::fs::symlink(fake, dest).expect("fake command symlinked");
}

#[cfg(not(unix))]
fn install_fake_command(fake: &Path, dest: &Path) {
    fs::copy(fake, dest).expect("fake command copied");
    let permissions = fs::metadata(fake).expect("fake metadata").permissions();
    fs::set_permissions(dest, permissions).expect("fake permissions copied");
}

fn temp_dir(name: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "oya-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos()
    ))
}

fn fixture_path(bin: &Path) -> String {
    let current = env::var("PATH").unwrap_or_default();
    format!("{}:{current}", bin.display())
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        stdout(output),
        stderr(output)
    );
}

fn assert_in_order(haystack: &str, needles: &[&str]) {
    let mut cursor = 0;
    for needle in needles {
        let Some(offset) = haystack[cursor..].find(needle) else {
            panic!("{needle:?} not found after byte {cursor} in:\n{haystack}");
        };
        cursor += offset + needle.len();
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}
