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
        git(
            &temp,
            [
                "ls-remote",
                "--exit-code",
                "origin",
                "refs/heads/claims/ADR-0377-D2"
            ]
        )
        .status
        .success(),
        "claim must exist on the remote authority"
    );
    assert!(
        git(
            &temp,
            [
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/claims/ADR-0377-D2"
            ]
        )
        .status
        .success(),
        "local claim ref mirror should be updated after remote CAS wins"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn plan_claim_cas_rejects_remote_only_claim_ref() {
    let temp = temp_repo("plan-claim-remote-only");
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
    assert!(
        git(&temp, ["update-ref", "-d", "refs/heads/claims/ADR-0377-D2"])
            .status
            .success()
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
        "stderr should explain remote-only CAS conflict: {stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn plan_claim_can_recover_expired_remote_claim_with_lease() {
    let temp = temp_repo("plan-claim-recover-stale");
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
            "--lease-seconds",
            "0",
        ])
        .output()
        .expect("first stale plan claim runs");
    assert!(
        first.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let before = remote_oid(&temp, "refs/heads/claims/ADR-0377-D2");

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
            "--recover-stale",
            "--recovery-reason",
            "worker-a lease expired",
        ])
        .output()
        .expect("stale plan claim recovery runs");

    assert!(
        second.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    let after = remote_oid(&temp, "refs/heads/claims/ADR-0377-D2");
    assert_ne!(before, after, "stale recovery must advance the remote ref");
    let commit = git(&temp, ["cat-file", "-p", &after]);
    let commit_text = String::from_utf8(commit.stdout).expect("commit utf8");
    assert!(commit_text.contains("Claimant: worker-b"));
    assert!(commit_text.contains("Recovery-reason: worker-a lease expired"));
    assert!(commit_text.contains("Source-commit:"));
    assert!(commit_text.contains("Lease-expires-at:"));

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
        git(
            &temp,
            [
                "push",
                "origin",
                &format!("{}:refs/heads/claims/ADR-0377-D2", commit.trim())
            ]
        )
        .status
        .success()
    );
    let _ = git(&temp, ["update-ref", "-d", "refs/heads/claims/ADR-0377-D2"])
        .status
        .success();

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
        !git(
            &temp,
            [
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/claims/ADR-0377-D3"
            ]
        )
        .status
        .success(),
        "next must not acquire the returned deliverable"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn plan_reserve_id_rejects_adr_seen_on_inflight_branch() {
    let temp = temp_repo("plan-reserve-id-inflight-adr");
    write_inflight_adr_branch(&temp, "agent/adr0700", "ADR-0700");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "plan",
            "reserve-id",
            "--repo-root",
            temp.to_str().expect("utf8 repo"),
            "--id",
            "ADR-0700",
            "--claimant",
            "worker-b",
        ])
        .output()
        .expect("plan reserve-id command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already in-flight"),
        "stderr should explain in-flight reservation collision: {stderr}"
    );
    assert!(
        stderr.contains("refs/heads/agent/adr0700"),
        "stderr should name the conflicting branch: {stderr}"
    );
    assert!(
        !git(
            &temp,
            [
                "ls-remote",
                "--exit-code",
                "origin",
                "refs/heads/id-reservations/ADR-0700"
            ]
        )
        .status
        .success(),
        "rejected reservation must not publish an id-reservation ref"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn plan_reserve_id_rejects_prd_seen_on_inflight_branch() {
    let temp = temp_repo("plan-reserve-id-inflight-prd");
    write_inflight_prd_branch(&temp, "agent/prd-collision", "PRD-COLLISION");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "plan",
            "reserve-id",
            "--repo-root",
            temp.to_str().expect("utf8 repo"),
            "--id",
            "PRD-COLLISION",
            "--claimant",
            "worker-b",
        ])
        .output()
        .expect("plan reserve-id command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already in-flight"),
        "stderr should explain in-flight PRD reservation collision: {stderr}"
    );
    assert!(
        stderr.contains("refs/heads/agent/prd-collision"),
        "stderr should name the conflicting branch: {stderr}"
    );
    assert!(
        !git(
            &temp,
            [
                "ls-remote",
                "--exit-code",
                "origin",
                "refs/heads/id-reservations/PRD-COLLISION"
            ]
        )
        .status
        .success(),
        "rejected reservation must not publish a PRD id-reservation ref"
    );

    fs::remove_dir_all(temp).ok();
}

fn temp_repo(label: &str) -> PathBuf {
    let path = temp_dir(label);
    let remote = temp_dir(&format!("{label}-remote"));
    fs::create_dir_all(&path).expect("temp repo dir created");
    fs::create_dir_all(&remote).expect("remote repo dir created");
    assert!(
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&remote)
            .output()
            .expect("git init --bare runs")
            .status
            .success()
    );
    assert!(git(&path, ["init"]).status.success());
    assert!(
        git(
            &path,
            [
                "remote",
                "add",
                "origin",
                remote.to_str().expect("utf8 remote")
            ]
        )
        .status
        .success()
    );
    assert!(
        git_with_env(&path, ["commit", "--allow-empty", "-m", "initial"])
            .status
            .success()
    );
    path
}

fn write_inflight_adr_branch(repo: &Path, branch: &str, adr_id: &str) {
    assert!(git(repo, ["checkout", "-b", branch]).status.success());
    let file_name = format!("docs/decisions/{adr_id}-in-flight.md");
    let file_path = repo.join(&file_name);
    fs::create_dir_all(file_path.parent().expect("adr parent")).expect("adr dir created");
    fs::write(
        &file_path,
        format!(
            "---
id: {adr_id}
status: Proposed
---
# {adr_id}: In-flight reservation fixture
"
        ),
    )
    .expect("adr fixture written");
    assert!(git(repo, ["add", &file_name]).status.success());
    assert!(
        git_with_env(repo, ["commit", "-m", "add in-flight adr fixture"])
            .status
            .success()
    );
    assert!(
        git(
            repo,
            ["push", "origin", &format!("HEAD:refs/heads/{branch}")]
        )
        .status
        .success()
    );
    assert!(git(repo, ["checkout", "-"]).status.success());
}

fn write_inflight_prd_branch(repo: &Path, branch: &str, prd_id: &str) {
    assert!(git(repo, ["checkout", "-b", branch]).status.success());
    let file_name = "specs/products/collision.json";
    let file_path = repo.join(file_name);
    fs::create_dir_all(file_path.parent().expect("prd parent")).expect("prd dir created");
    fs::write(
        &file_path,
        format!(
            r#"{{
  "_meta": {{
    "doc_class": "Machine-Readable-Spec",
    "spec_id": "{prd_id}",
    "status": "Proposed"
  }}
}}
"#
        ),
    )
    .expect("prd fixture written");
    assert!(git(repo, ["add", file_name]).status.success());
    assert!(
        git_with_env(repo, ["commit", "-m", "add in-flight prd fixture"])
            .status
            .success()
    );
    assert!(
        git(
            repo,
            ["push", "origin", &format!("HEAD:refs/heads/{branch}")]
        )
        .status
        .success()
    );
    assert!(git(repo, ["checkout", "-"]).status.success());
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

fn remote_oid(repo: &Path, claim_ref: &str) -> String {
    let output = git(repo, ["ls-remote", "--exit-code", "origin", claim_ref]);
    assert!(
        output.status.success(),
        "ls-remote failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("ls-remote utf8")
        .split_whitespace()
        .next()
        .expect("remote oid")
        .to_owned()
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
