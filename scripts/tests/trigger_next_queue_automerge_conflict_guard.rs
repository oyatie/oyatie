use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap())
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

#[cfg(unix)]
fn mark_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn mark_executable(_path: &Path) {}

fn copy_executable(from: &Path, to: &Path) {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::copy(from, to).unwrap_or_else(|error| {
        panic!(
            "failed to copy {} to {}: {error}",
            from.display(),
            to.display()
        )
    });
    mark_executable(to);
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

fn rustc_compile(src: &Path, out: &Path) {
    let output = Command::new("rustc")
        .args(["--edition=2021", "-D", "warnings"])
        .arg(src)
        .arg("-o")
        .arg(out)
        .output()
        .unwrap_or_else(|error| panic!("rustc failed to spawn for {}: {error}", src.display()));
    assert!(
        output.status.success(),
        "rustc {} failed\nstdout:\n{}\nstderr:\n{}",
        src.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    mark_executable(out);
}

fn fake_gh_source() -> &'static str {
    r##"
use std::env;
use std::fs;
use std::io::Write;
use std::process;

fn read_env_file(name: &str) -> String {
    fs::read_to_string(env::var(name).unwrap_or_else(|_| panic!("{name} is required"))).unwrap()
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let log_path = env::var("OYA_TEST_GH_LOG").expect("OYA_TEST_GH_LOG is required");
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap()
        .write_all(format!("{}\n", args.join(" ")).as_bytes())
        .unwrap();

    if args.first().map(String::as_str) == Some("repo")
        && args.get(1).map(String::as_str) == Some("view")
    {
        println!("jason931225/oyatie");
        return;
    }

    if args.first().map(String::as_str) == Some("api")
        && args
            .get(1)
            .is_some_and(|value| value.starts_with("repos/") && value.ends_with("/branches/dev/protection/required_status_checks"))
    {
        print!("{}", read_env_file("OYA_TEST_LIVE_CONTEXTS"));
        return;
    }

    if args.first().map(String::as_str) == Some("api")
        && args
            .get(1)
            .is_some_and(|value| value.starts_with("repos/") && value.contains("/commits/"))
    {
        println!(r#"{{"verified":true,"reason":"valid"}}"#);
        return;
    }

    if args.first().map(String::as_str) == Some("pr")
        && args.get(1).map(String::as_str) == Some("list")
    {
        print!("{}", read_env_file("OYA_TEST_PRS"));
        return;
    }

    if args.first().map(String::as_str) == Some("pr")
        && args.get(1).map(String::as_str) == Some("view")
    {
        print!("{}", read_env_file("OYA_TEST_PR_STATE"));
        return;
    }

    if args.first().map(String::as_str) == Some("pr")
        && args.get(1).map(String::as_str) == Some("checks")
    {
        print!("{}", read_env_file("OYA_TEST_CHECKS"));
        return;
    }

    if args.first().map(String::as_str) == Some("pr")
        && args.get(1).map(String::as_str) == Some("merge")
    {
        let marker = env::var("OYA_TEST_GUARD_MARKER").unwrap_or_default();
        if !marker.is_empty() {
            match fs::metadata(&marker) {
                Ok(metadata) if metadata.len() > 0 => {}
                _ => {
                    eprintln!(
                        "gh pr merge reached before sequential conflict guard marker: {}",
                        args.join(" ")
                    );
                    process::exit(98);
                }
            }
        }
        let mut body = String::new();
        if !marker.is_empty() {
            body.push_str(&format!(
                "guard_marker={}\n",
                fs::read_to_string(&marker).unwrap()
            ));
        }
        body.push_str(&format!("{}\n", args.join(" ")));
        fs::write(env::var("OYA_TEST_MERGE_CALLED").expect("merge-called path"), body).unwrap();
        return;
    }

    eprintln!("unexpected gh invocation: {}", args.join(" "));
    process::exit(99);
}
"##
}

fn guard_wrapper_source() -> &'static str {
    r#"
use std::env;
use std::fs;
use std::process::{self, Command};

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let current_exe = env::current_exe().expect("current exe");
    let script_dir = current_exe.parent().expect("script dir");
    let real_guard = script_dir.join("check-sequential-pr-merge-conflicts.real.sh");
    let status = Command::new(&real_guard)
        .args(&args)
        .status()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", real_guard.display()));
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }
    if let Ok(marker) = env::var("OYA_TEST_GUARD_MARKER") {
        if !marker.is_empty() {
            fs::write(marker, format!("guard passed: {}\n", args.join(" "))).unwrap();
        }
    }
}
"#
}

fn compile_fake_gh(tmp: &Path) -> PathBuf {
    let bin_dir = tmp.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let src = tmp.join("fake-gh.rs");
    let bin = bin_dir.join("gh");
    fs::write(&src, fake_gh_source()).unwrap();
    rustc_compile(&src, &bin);
    bin_dir
}

fn compile_guard_wrapper(work: &Path) {
    // Local sequencing regression guard: the compiled wrapper delegates to the
    // real compatibility guard and writes the marker only after that guard exits
    // successfully. The fake gh merge path refuses a non-dry-run merge without
    // this marker. This is local harness evidence, not live cloud-ci/oya-ci
    // authority proof.
    let src = work.join("scripts/check-sequential-pr-merge-conflicts-wrapper.rs");
    let bin = work.join("scripts/check-sequential-pr-merge-conflicts.sh");
    fs::write(&src, guard_wrapper_source()).unwrap();
    rustc_compile(&src, &bin);
    fs::remove_file(src).unwrap();
}

fn setup_queue_repo(root: &Path, tmp: &Path, scenario: &str) -> PathBuf {
    let work = tmp.join(format!("work-{scenario}"));
    let mirror = tmp.join(format!("github-mirror-{scenario}.git"));

    run_git(tmp, &["init", "-q", work.to_str().unwrap()]);
    run_git(
        &work,
        &["config", "user.email", "queue-test@example.invalid"],
    );
    run_git(&work, &["config", "user.name", "queue-test"]);

    fs::create_dir_all(work.join("scripts/ci")).unwrap();
    fs::create_dir_all(work.join("infra/branch-protection")).unwrap();
    copy_executable(
        &root.join("scripts/trigger-next-queue-automerge.sh"),
        &work.join("scripts/trigger-next-queue-automerge.sh"),
    );
    copy_executable(
        &root.join("scripts/check-sequential-pr-merge-conflicts.sh"),
        &work.join("scripts/check-sequential-pr-merge-conflicts.real.sh"),
    );
    fs::copy(
        root.join("scripts/check-sequential-pr-merge-conflicts.rs"),
        work.join("scripts/check-sequential-pr-merge-conflicts.rs"),
    )
    .unwrap();
    fs::copy(
        root.join("scripts/ci/assert-result-bundle-output.rs"),
        work.join("scripts/ci/assert-result-bundle-output.rs"),
    )
    .unwrap();
    fs::copy(
        root.join("infra/branch-protection/dev.json"),
        work.join("infra/branch-protection/dev.json"),
    )
    .unwrap();
    compile_guard_wrapper(&work);

    fs::write(work.join("shared.txt"), "base\n").unwrap();
    run_git(&work, &["add", "shared.txt"]);
    run_git(&work, &["commit", "-q", "-m", "base"]);
    let base_commit = run_git(&work, &["rev-parse", "HEAD"]);

    run_git(&work, &["checkout", "-q", "-b", "pr-455", &base_commit]);
    if scenario == "conflict" {
        fs::write(work.join("shared.txt"), "pr-side\n").unwrap();
        run_git(&work, &["add", "shared.txt"]);
    } else {
        fs::write(work.join("pr.txt"), "pr-only\n").unwrap();
        run_git(&work, &["add", "pr.txt"]);
    }
    run_git(&work, &["commit", "-q", "-m", "pr-455"]);
    let head_commit = run_git(&work, &["rev-parse", "HEAD"]);

    run_git(&work, &["checkout", "-q", "-b", "dev", &base_commit]);
    fs::write(work.join("shared.txt"), "dev-side\n").unwrap();
    run_git(&work, &["add", "shared.txt"]);
    run_git(&work, &["commit", "-q", "-m", "dev advances"]);

    run_git(tmp, &["init", "-q", "--bare", mirror.to_str().unwrap()]);
    run_git(
        &work,
        &[
            "remote",
            "add",
            "origin",
            tmp.join(format!("forgejo-origin-{scenario}.git"))
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
        &["push", "-q", "github-mirror", "dev:refs/heads/dev"],
    );
    run_git(
        &work,
        &[
            "push",
            "-q",
            "github-mirror",
            &format!("{head_commit}:refs/pull/455/head"),
        ],
    );

    fs::write(
        tmp.join(format!("{scenario}-prs.json")),
        format!(
            r#"[
  {{
    "number": 455,
    "headRefName": "feat/conflict-guard-{scenario}",
    "headRefOid": "{head_commit}",
    "isDraft": false,
    "title": "conflict guard {scenario}"
  }}
]
"#
        ),
    )
    .unwrap();

    fs::write(
        tmp.join(format!("{scenario}-state.json")),
        format!(
            r#"{{
  "isDraft": false,
  "mergeable": "MERGEABLE",
  "mergeStateStatus": "CLEAN",
  "reviewDecision": "APPROVED",
  "headRefOid": "{head_commit}"
}}
"#
        ),
    )
    .unwrap();

    work
}

struct TriggerRun {
    status: i32,
    stdout: String,
    stderr: String,
    gh_log: PathBuf,
    merge_called: PathBuf,
    guard_marker: PathBuf,
}

fn run_trigger(
    tmp: &Path,
    fake_bin: &Path,
    scenario: &str,
    work: &Path,
    dry_run: bool,
) -> TriggerRun {
    let out = tmp.join(format!("{scenario}.out"));
    let err = tmp.join(format!("{scenario}.err"));
    let gh_log = tmp.join(format!("{scenario}-gh.log"));
    let merge_called = tmp.join(format!("{scenario}-merge-called"));
    let guard_marker = tmp.join(format!("{scenario}-guard-passed"));
    for path in [&gh_log, &merge_called, &guard_marker] {
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }

    let mut args = vec![
        "--base-branch",
        "dev",
        "--base-ref",
        "dev",
        "--start-pr",
        "455",
        "--limit",
        "20",
        "--required-contexts-config",
        "infra/branch-protection/dev.json",
        "--fetch-remote",
        "github-mirror",
    ];
    if dry_run {
        args.push("--dry-run");
    }

    let path = format!(
        "{}:{}",
        fake_bin.display(),
        env::var("PATH").unwrap_or_default()
    );
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new(work.join("scripts/trigger-next-queue-automerge.sh"))
        .args(args)
        .current_dir(work)
        .env("PATH", path)
        .env("OYA_TEST_GH_LOG", &gh_log)
        .env("OYA_TEST_LIVE_CONTEXTS", tmp.join("live-contexts.json"))
        .env("OYA_TEST_PRS", tmp.join(format!("{scenario}-prs.json")))
        .env(
            "OYA_TEST_PR_STATE",
            tmp.join(format!("{scenario}-state.json")),
        )
        .env("OYA_TEST_CHECKS", tmp.join("checks.json"))
        .env("OYA_TEST_MERGE_CALLED", &merge_called)
        .env("OYA_TEST_GUARD_MARKER", &guard_marker)
        .output()
        .unwrap_or_else(|error| panic!("trigger failed to spawn: {error}"));

    fs::write(&out, &stdout).unwrap();
    fs::write(&err, &stderr).unwrap();

    TriggerRun {
        status: status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&stdout).to_string(),
        stderr: String::from_utf8_lossy(&stderr).to_string(),
        gh_log,
        merge_called,
        guard_marker,
    }
}

fn assert_contains(haystack: &str, needle: &str, label: &str) {
    assert!(
        haystack.contains(needle),
        "{label} missing {needle:?}; observed:\n{haystack}"
    );
}

fn assert_file_contains(path: &Path, needle: &str, label: &str) {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert_contains(&text, needle, label);
}

#[test]
fn trigger_runs_sequential_conflict_guard_before_automerge() {
    let root = repo_root();
    let tmp = temp_dir("trigger-conflict-guard");
    let fake_bin = compile_fake_gh(&tmp);

    fs::write(
        tmp.join("live-contexts.json"),
        r#"{
  "strict": false,
  "contexts": [
    "github-lane-unlocker-required"
  ]
}
"#,
    )
    .unwrap();
    fs::write(
        tmp.join("checks.json"),
        r#"[
  {
    "name": "oya-pr-review",
    "bucket": "pass",
    "state": "SUCCESS",
    "workflow": "review"
  }
]
"#,
    )
    .unwrap();

    let clean_dry_work = setup_queue_repo(&root, &tmp, "clean-dry");
    let clean_dry = run_trigger(&tmp, &fake_bin, "clean-dry", &clean_dry_work, true);
    assert_eq!(
        clean_dry.status, 0,
        "expected clean queue candidate to reach dry-run auto-merge after conflict guard\nstdout:\n{}\nstderr:\n{}",
        clean_dry.stdout, clean_dry.stderr
    );
    assert_contains(
        &clean_dry.stdout,
        "sequential PR merge simulation passed: 1 PRs modeled",
        "clean dry stdout",
    );
    assert_contains(
        &clean_dry.stdout,
        "dry-run: gh pr merge 455 --squash --auto --match-head-commit",
        "clean dry stdout",
    );
    assert_file_contains(
        &clean_dry.guard_marker,
        "guard passed:",
        "clean dry guard marker",
    );
    assert!(
        !clean_dry.merge_called.exists(),
        "dry-run clean scenario must not invoke gh pr merge"
    );

    let clean_real_work = setup_queue_repo(&root, &tmp, "clean-real");
    let clean_real = run_trigger(&tmp, &fake_bin, "clean-real", &clean_real_work, false);
    assert_eq!(
        clean_real.status, 0,
        "expected clean queue candidate to invoke fake gh pr merge after conflict guard\nstdout:\n{}\nstderr:\n{}",
        clean_real.stdout, clean_real.stderr
    );
    assert_contains(
        &clean_real.stdout,
        "sequential PR merge simulation passed: 1 PRs modeled",
        "clean real stdout",
    );
    assert_contains(
        &clean_real.stdout,
        "auto-merge enabled for bottom-most queue PR #455",
        "clean real stdout",
    );
    assert_file_contains(
        &clean_real.merge_called,
        "guard_marker=guard passed:",
        "clean real merge call",
    );
    assert_file_contains(
        &clean_real.merge_called,
        "pr merge 455 --squash --auto --match-head-commit",
        "clean real merge call",
    );
    assert!(
        !clean_real.stdout.contains("dry-run: gh pr merge"),
        "non-dry-run clean scenario unexpectedly used dry-run path"
    );

    let conflict_work = setup_queue_repo(&root, &tmp, "conflict");
    let conflict = run_trigger(&tmp, &fake_bin, "conflict", &conflict_work, false);
    assert_ne!(
        conflict.status, 0,
        "expected conflicting queue candidate to fail before auto-merge\nstdout:\n{}\nstderr:\n{}",
        conflict.stdout, conflict.stderr
    );
    assert_contains(
        &conflict.stdout,
        "queue candidate: PR #455",
        "conflict stdout",
    );
    assert_contains(&conflict.stdout, "checking PR #455", "conflict stdout");
    assert_contains(
        &conflict.stderr,
        "::error::sequential merge conflict at PR #455",
        "conflict stderr",
    );
    assert!(
        !conflict.merge_called.exists(),
        "conflict scenario invoked gh pr merge despite sequential guard failure"
    );
    assert!(
        !conflict.guard_marker.exists(),
        "conflict scenario recorded a sequential guard pass despite merge-tree conflict"
    );
    if conflict.gh_log.exists() {
        let gh_log = fs::read_to_string(&conflict.gh_log).unwrap();
        assert!(
            !gh_log.contains("pr merge"),
            "conflict scenario logged gh pr merge despite sequential guard failure\n{gh_log}"
        );
    }

    fs::remove_dir_all(tmp).unwrap();
}
