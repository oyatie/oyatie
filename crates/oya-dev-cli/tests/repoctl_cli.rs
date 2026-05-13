use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn shared_cli_entrypoint_remains_public_library_api() {
    let _entrypoint: fn() -> std::process::ExitCode = oya_tooling_cli_dev_runtime::run_cli_from_env;
}

#[test]
fn repoctl_pre_push_runs_check_script_and_reports_text_success() {
    let temp = temp_dir("repoctl-pre-push-success");
    let script = write_file(&temp, "check.sh", "echo repoctl-ok\n");

    let output = Command::new(env!("CARGO_BIN_EXE_repoctl"))
        .args([
            "pre-push",
            "--check-script",
            script.to_str().expect("utf8 script"),
        ])
        .output()
        .expect("repoctl pre-push command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("repoctl-ok"), "stdout={stdout}");
    assert!(
        stdout.contains("repoctl pre-push passed:"),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn repoctl_pre_push_replays_script_stderr_and_fails_closed() {
    let temp = temp_dir("repoctl-pre-push-failure");
    let script = write_file(&temp, "check.sh", "echo repoctl-bad >&2\nexit 7\n");

    let output = Command::new(env!("CARGO_BIN_EXE_repoctl"))
        .args([
            "pre-push",
            "--check-script",
            script.to_str().expect("utf8 script"),
        ])
        .output()
        .expect("repoctl pre-push command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("repoctl-bad"), "stderr={stderr}");
    assert!(
        stderr.contains("repoctl pre-push failed:") && stderr.contains("exit code 7"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn repoctl_pre_push_json_mode_emits_structured_evidence_without_replaying_text() {
    let temp = temp_dir("repoctl-pre-push-json");
    let script = write_file(&temp, "check.sh", "echo repoctl-json-ok\n");

    let output = Command::new(env!("CARGO_BIN_EXE_repoctl"))
        .args([
            "pre-push",
            "--check-script",
            script.to_str().expect("utf8 script"),
            "--format",
            "json",
        ])
        .output()
        .expect("repoctl pre-push command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("{\"command\":\"repoctl pre-push\""),
        "stdout={stdout}"
    );
    assert!(stdout.contains("\"status\":\"passed\""), "stdout={stdout}");
    assert!(
        stdout.contains("\"stdout\":\"repoctl-json-ok\\n\""),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn repoctl_pre_push_scrubs_cargo_package_metadata_for_nested_cargo_plugins() {
    let temp = temp_dir("repoctl-pre-push-cargo-env");
    let script = write_file(
        &temp,
        "check.sh",
        "test -z \"${CARGO_MANIFEST_DIR:-}\"\n\
         test -z \"${CARGO_MANIFEST_PATH:-}\"\n\
         test -z \"${CARGO_PKG_NAME:-}\"\n\
         echo cargo-env-clean\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_repoctl"))
        .args([
            "pre-push",
            "--check-script",
            script.to_str().expect("utf8 script"),
        ])
        .output()
        .expect("repoctl pre-push command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("cargo-env-clean"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn repoctl_pre_push_contract_accepts_wired_fixture() {
    let temp = temp_dir("repoctl-pre-push-contract");
    let agents = write_file(
        &temp,
        "AGENTS.md",
        "## Done-Definition\n- [ ] D12 `repoctl pre-push` passes.\n",
    );
    let check_script = write_file(
        &temp,
        "check.sh",
        "cargo run -p oya-tooling-cli-dev-runtime --bin repoctl -- pre-push --verify-contract\n",
    );
    let manifest = write_file(
        &temp,
        "Cargo.toml",
        "[[bin]]\nname = \"repoctl\"\npath = \"src/main.rs\"\n",
    );
    let hook = write_file(
        &temp,
        "pre-push.sh",
        "cargo run -p oya-tooling-cli-dev-runtime --bin repoctl -- pre-push \"$@\"\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_repoctl"))
        .args([
            "pre-push",
            "--verify-contract",
            "--agents-doc",
            agents.to_str().expect("utf8 agents"),
            "--check-script",
            check_script.to_str().expect("utf8 check"),
            "--cli-manifest",
            manifest.to_str().expect("utf8 manifest"),
            "--hook-script",
            hook.to_str().expect("utf8 hook"),
        ])
        .output()
        .expect("repoctl pre-push contract command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("repoctl pre-push contract validation passed:"),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn oya_binary_can_route_repoctl_command_for_local_harnesses() {
    let temp = temp_dir("oya-repoctl-route");
    let script = write_file(&temp, "check.sh", "echo oya-route-ok\n");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "repoctl",
            "pre-push",
            "--check-script",
            script.to_str().expect("utf8 script"),
        ])
        .output()
        .expect("oya repoctl command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("oya-route-ok"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn repoctl_and_oya_binaries_share_library_entrypoint_without_textual_include() {
    let repoctl_src = include_str!("../src/repoctl.rs");
    assert!(
        !repoctl_src.contains("include!"),
        "repoctl binary must not textually include the command host"
    );
    assert!(
        repoctl_src.contains("run_cli_from_env"),
        "repoctl binary must call the shared CLI entrypoint"
    );

    let main_src = include_str!("../src/main.rs");
    assert!(
        !main_src.contains("include!"),
        "oya binary must remain a thin wrapper without textual includes"
    );
    assert!(
        main_src.contains("run_cli_from_env"),
        "oya binary must call the shared CLI entrypoint"
    );

    let lib_src = include_str!("../src/lib.rs");
    assert!(
        lib_src.contains("pub fn run_cli_from_env() -> ExitCode"),
        "shared CLI entrypoint must remain explicit"
    );
}

fn write_file(root: &Path, name: &str, contents: &str) -> PathBuf {
    fs::create_dir_all(root).expect("temp dir created");
    let path = root.join(name);
    fs::write(&path, contents).expect("file written");
    path
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
