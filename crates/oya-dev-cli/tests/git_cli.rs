// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// command-line invariants under controlled fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn oya_git_forwards_to_git_version_without_ledger_outside_repo() {
    let temp = temp_dir("oya-git-version");
    fs::create_dir_all(&temp).expect("temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["git", "--version"])
        .current_dir(&temp)
        .output()
        .expect("oya git runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("git version "));
    assert!(!temp.join(".oya").exists());
    fs::remove_dir_all(temp).ok();
}

#[test]
fn oya_git_empty_args_match_git_empty_args() {
    let temp = temp_dir("oya-git-empty");
    fs::create_dir_all(&temp).expect("temp dir");
    let oya = Command::new(env!("CARGO_BIN_EXE_oya"))
        .arg("git")
        .current_dir(&temp)
        .output()
        .expect("oya git runs");
    let git = Command::new("git")
        .current_dir(&temp)
        .output()
        .expect("git runs");

    assert_eq!(oya.status.code(), git.status.code());
    assert_eq!(
        String::from_utf8_lossy(&oya.stdout),
        String::from_utf8_lossy(&git.stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&oya.stderr),
        String::from_utf8_lossy(&git.stderr)
    );
    fs::remove_dir_all(temp).ok();
}

#[test]
fn oya_git_writes_git_metadata_side_channel_ledger_in_repo() {
    let repo = temp_dir("oya-git-ledger");
    fs::create_dir_all(&repo).expect("temp repo dir");
    let init = Command::new("git")
        .arg("init")
        .current_dir(&repo)
        .output()
        .expect("git init runs");
    assert!(
        init.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["git", "status", "--short"])
        .current_dir(&repo)
        .output()
        .expect("oya git status runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(!repo.join(".oya").exists());
    let ledger_path = repo
        .join(".git")
        .join("oya")
        .join("ledger")
        .join("audit-chain.jsonl");
    let ledger = fs::read_to_string(&ledger_path).expect("ledger written");
    assert!(ledger.contains("\"event_type\":\"oya_git_command\""));
    assert!(ledger.contains("\"verb\":\"status\""));
    assert!(ledger.contains("\"arg_count\":2"));
    assert!(ledger.contains("\"success\":true"));
    assert!(ledger.contains("\"ledger_scope\":\"git-metadata\""));
    assert!(ledger.contains("\"repo_root\":\"repo-root\""));
    assert!(!ledger.contains("\"args\""));
    assert!(!ledger.contains(&repo.display().to_string()));

    fs::remove_dir_all(repo).ok();
}

#[test]
fn oya_git_ledger_uses_effective_c_option_repo_and_omits_secrets() {
    let caller = temp_dir("oya-git-caller");
    let target = temp_dir("oya-git-target");
    fs::create_dir_all(&caller).expect("caller dir");
    fs::create_dir_all(&target).expect("target dir");
    let init = Command::new("git")
        .arg("init")
        .current_dir(&target)
        .output()
        .expect("git init runs");
    assert!(
        init.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let secret = "Authorization: Bearer test-secret-token";
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["git", "-C"])
        .arg(&target)
        .args(["-c"])
        .arg(format!("http.extraHeader={secret}"))
        .args(["status", "--short"])
        .current_dir(&caller)
        .output()
        .expect("oya git -C status runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(!target.join(".oya").exists());
    let target_ledger_path = target
        .join(".git")
        .join("oya")
        .join("ledger")
        .join("audit-chain.jsonl");
    let target_ledger = fs::read_to_string(&target_ledger_path).expect("target ledger written");
    assert!(target_ledger.contains("\"verb\":\"status\""));
    assert!(target_ledger.contains("\"cwd\":\"outside-repo\""));
    assert!(target_ledger.contains("\"git_cwd\":\".\""));
    assert!(!target_ledger.contains(secret));
    assert!(!target_ledger.contains("test-secret-token"));
    assert!(!target_ledger.contains(&caller.display().to_string()));
    assert!(!target_ledger.contains(&target.display().to_string()));
    assert!(!target_ledger.contains("\"args\""));
    assert!(!caller.join(".oya").exists());

    fs::remove_dir_all(caller).ok();
    fs::remove_dir_all(target).ok();
}

fn temp_dir(prefix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
