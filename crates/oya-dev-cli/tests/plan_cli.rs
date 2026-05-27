// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn plan_claim_next_acquires_git_ref_and_projects_exclusive_labels() {
    let temp = temp_repo("plan-claim-next");
    let master_plan = write_master_plan(&temp);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "plan",
            "claim/next",
            "--repo-root",
            temp.to_str().expect("utf8 repo"),
            "--master-plan",
            master_plan.to_str().expect("utf8 master plan"),
            "--claimant",
            "Worker 1",
        ])
        .output()
        .expect("plan claim command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("plan claimed: ADR-0377-D2"));
    assert!(stdout.contains("refs/heads/claims/ADR-0377-D2"));
    assert!(stdout.contains("state/claimed"));
    assert!(stdout.contains("owner/worker-1"));
    assert!(stdout.contains("deliverable/adr-0377-d2"));

    assert!(
        git(&temp, ["show-ref", "--verify", "--quiet", "refs/heads/claims/ADR-0377-D2"])
            .status
            .success()
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn plan_claim_cas_rejects_existing_claim_ref() {
    let temp = temp_repo("plan-claim-cas");
    let master_plan = write_master_plan(&temp);

    let first = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "plan",
            "claim",
            "--repo-root",
            temp.to_str().expect("utf8 repo"),
            "--master-plan",
            master_plan.to_str().expect("utf8 master plan"),
            "--deliverable",
            "ADR-0377-D2",
            "--claimant",
            "worker-a",
        ])
        .output()
        .expect("first plan claim runs");
    assert!(
        first.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );

    let second = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "plan",
            "claim",
            "--repo-root",
            temp.to_str().expect("utf8 repo"),
            "--master-plan",
            master_plan.to_str().expect("utf8 master plan"),
            "--deliverable",
            "ADR-0377-D2",
            "--claimant",
            "worker-b",
        ])
        .output()
        .expect("second plan claim runs");

    assert!(!second.status.success());
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        stderr.contains("already claimed"),
        "stderr should explain CAS conflict: {stderr}"
    );
    assert!(
        stderr.contains("refs/heads/claims/ADR-0377-D2"),
        "stderr should name claim ref: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn plan_next_skips_existing_claim_without_mutating_next_ref() {
    let temp = temp_repo("plan-next-skip");
    let master_plan = write_master_plan(&temp);
    let empty = git(&temp, ["mktree"]).stdout;
    let empty_tree = String::from_utf8(empty).expect("utf8 tree");
    let commit = git_with_env(
        &temp,
        ["commit-tree", empty_tree.trim(), "-m", "existing claim"],
    );
    let commit = String::from_utf8(commit.stdout).expect("utf8 commit");
    assert!(
        git(&temp, ["update-ref", "refs/heads/claims/ADR-0377-D2", commit.trim()])
            .status
            .success()
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "plan",
            "next",
            "--repo-root",
            temp.to_str().expect("utf8 repo"),
            "--master-plan",
            master_plan.to_str().expect("utf8 master plan"),
            "--claimant",
            "worker-a",
        ])
        .output()
        .expect("plan next runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("plan next: ADR-0377-D3"));
    assert!(!stdout.contains("oya git"));
    assert!(!stdout.contains("oya vcs"));
    assert!(
        !git(&temp, ["show-ref", "--verify", "--quiet", "refs/heads/claims/ADR-0377-D3"])
            .status
            .success(),
        "next must not acquire the returned deliverable"
    );

    fs::remove_dir_all(temp).ok();
}

fn temp_repo(label: &str) -> PathBuf {
    let path = temp_dir(label);
    fs::create_dir_all(&path).expect("temp repo dir created");
    assert!(git(&path, ["init"]).status.success());
    path
}

fn write_master_plan(root: &Path) -> PathBuf {
    let path = root.join("masterplan.generated.json");
    fs::write(
        &path,
        r#"{
  "milestones": [{
    "adrs": [{
      "id": "ADR-0377",
      "deliverables": [
        {"id": "ADR-0377-D2", "description": "claim next"},
        {"id": "ADR-0377-D3", "description": "board sync"}
      ]
    }]
  }]
}"#,
    )
    .expect("master plan written");
    path
}

fn git<const N: usize>(repo: &Path, args: [&str; N]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git command runs")
}

fn git_with_env<const N: usize>(repo: &Path, args: [&str; N]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(repo)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .args(args)
        .output()
        .expect("git command runs")
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
