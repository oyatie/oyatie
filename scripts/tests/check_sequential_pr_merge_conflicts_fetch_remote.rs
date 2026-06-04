use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap())
}

fn checker_bin() -> PathBuf {
    env::var_os("OYA_SEQ_CONFLICT_CHECKER_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root().join("scripts/check-sequential-pr-merge-conflicts"))
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("oyatie-{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn run_git(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|error| panic!("git {} failed to spawn: {error}", args.join(" ")));
    assert!(
        output.status.success(),
        "git {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run_checker(cwd: &Path, args: &[String]) -> std::process::Output {
    Command::new(checker_bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|error| panic!("checker failed to spawn: {error}"))
}

#[test]
fn fetch_remote_selects_github_mirror_and_default_origin_fails_closed() {
    let tmp = temp_dir("check-sequential-fetch-remote");
    let work = tmp.join("work");
    let mirror = tmp.join("github-mirror.git");

    run_git(&tmp, &["init", "-q", work.to_str().unwrap()]);
    run_git(
        &work,
        &["config", "user.email", "queue-test@example.invalid"],
    );
    run_git(&work, &["config", "user.name", "queue-test"]);

    fs::write(work.join("shared.txt"), "base\n").unwrap();
    run_git(&work, &["add", "shared.txt"]);
    run_git(&work, &["commit", "-q", "-m", "base"]);
    let base_commit = run_git(&work, &["rev-parse", "HEAD"]);

    run_git(&tmp, &["init", "-q", "--bare", mirror.to_str().unwrap()]);
    run_git(
        &work,
        &[
            "remote",
            "add",
            "origin",
            tmp.join("forgejo-origin-does-not-exist.git")
                .to_str()
                .unwrap(),
        ],
    );
    run_git(
        &work,
        &["remote", "add", "github-mirror", mirror.to_str().unwrap()],
    );
    run_git(
        &work,
        &["push", "-q", "github-mirror", "HEAD:refs/heads/dev"],
    );

    run_git(&work, &["checkout", "-q", "-b", "pr-455"]);
    fs::write(work.join("pr.txt"), "pr\n").unwrap();
    run_git(&work, &["add", "pr.txt"]);
    run_git(&work, &["commit", "-q", "-m", "pr"]);
    let head_commit = run_git(&work, &["rev-parse", "HEAD"]);
    run_git(
        &work,
        &["push", "-q", "github-mirror", "HEAD:refs/pull/455/head"],
    );

    let prs_json = tmp.join("prs.json");
    fs::write(
        &prs_json,
        format!(
            r#"[
  {{
    "number": 455,
    "headRefName": "feat/pinned-head",
    "headRefOid": "{head_commit}",
    "isDraft": false,
    "title": "remote-select guard"
  }}
]
"#
        ),
    )
    .unwrap();

    let common = vec![
        "--base-branch".to_string(),
        "dev".to_string(),
        "--base-ref".to_string(),
        base_commit.clone(),
        "--start-pr".to_string(),
        "455".to_string(),
        "--end-pr".to_string(),
        "455".to_string(),
        "--pr-json".to_string(),
        prs_json.to_string_lossy().to_string(),
    ];

    let mut pass_args = common.clone();
    // Buck2 policy anchor: --fetch-remote github-mirror must pass when origin is
    // a non-GitHub/Forgejo remote and the GitHub mirror owns refs/pull/*.
    pass_args.extend(["--fetch-remote".to_string(), "github-mirror".to_string()]);
    let pass = run_checker(&work, &pass_args);
    assert!(
        pass.status.success(),
        "explicit github-mirror fetch should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&pass.stdout),
        String::from_utf8_lossy(&pass.stderr)
    );
    let pass_stdout = String::from_utf8_lossy(&pass.stdout);
    assert!(pass_stdout.contains("fetch_remote=github-mirror"));
    assert!(pass_stdout.contains("sequential PR merge simulation passed: 1 PRs modeled"));

    let fail = run_checker(&work, &common);
    assert!(
        !fail.status.success(),
        "default origin fetch should fail when origin is non-GitHub Forgejo remote"
    );
    let fail_stderr = String::from_utf8_lossy(&fail.stderr);
    assert!(fail_stderr.contains("failed to fetch PR #455 head from remote origin"));
    assert!(
        fail_stderr.contains("pass --fetch-remote for the GitHub mirror when origin is Forgejo")
    );

    fs::remove_dir_all(tmp).unwrap();
}
