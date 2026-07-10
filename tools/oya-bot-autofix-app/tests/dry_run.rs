use std::io::Write;
use std::process::{Command, Stdio};

use oya_bot_autofix_app::{
    Action, AutofixError, BotPolicy, DeliveryMode, DryRunInput, render_dry_run,
};
use oya_ci_gate_contract::{ByteRange, Edit, NewFile, Remediation};

/// Proves a rendered diff is a real, applicable patch — not just a diff that
/// "looks right" by substring — by running it through a real `git apply
/// --check` against a scratch repo where none of the referenced paths exist
/// yet (matching a brand-new-file review).
fn git_apply_check(diff: &str, tag: &str) -> std::process::Output {
    let root = std::env::temp_dir().join(format!(
        "oya-bot-autofix-git-apply-{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create scratch git repo dir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .expect("run git init")
            .success()
    );

    let mut apply = Command::new("git")
        .args(["apply", "--check", "-"])
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn git apply --check");
    apply
        .stdin
        .take()
        .expect("git apply stdin")
        .write_all(diff.as_bytes())
        .expect("write diff to git apply stdin");
    apply.wait_with_output().expect("run git apply --check")
}

#[test]
fn dry_run_renders_diff_without_writes() {
    let original = "[package]\nname = \"example\"\n";
    let remediation = Remediation::AutoFix(Edit::new(
        "libs/example/Cargo.toml",
        ByteRange::new(original.len(), original.len()).expect("valid insertion range"),
        "license = \"Apache-2.0\"\n",
    ));

    let report = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: original,
    })
    .expect("dry-run should render a reviewable diff");

    assert_eq!(report.safety.mode, DeliveryMode::DryRun);
    assert!(!report.safety.writes_performed);
    assert!(!report.safety.can_merge);
    assert!(!report.safety.can_bypass_gates);
    assert!(
        report
            .diff
            .contains("diff --git a/libs/example/Cargo.toml b/libs/example/Cargo.toml")
    );
    assert!(report.diff.contains("+license = \"Apache-2.0\""));
    assert_eq!(original, "[package]\nname = \"example\"\n");
}

#[test]
fn dry_run_renders_scoped_hunk_instead_of_whole_file() {
    let original = [
        "line 1\n", "line 2\n", "line 3\n", "line 4\n", "line 5\n", "line 6\n", "line 7\n",
        "line 8\n", "line 9\n",
    ]
    .concat();
    let start = original.find("line 5").expect("line exists");
    let end = start + "line 5".len();
    let remediation = Remediation::AutoFix(Edit::new(
        "fixture.txt",
        ByteRange::new(start, end).expect("valid replacement range"),
        "LINE 5".to_owned(),
    ));

    let report = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: &original,
    })
    .expect("dry-run should render scoped diff");

    assert!(report.diff.contains("-line 5"));
    assert!(report.diff.contains("+LINE 5"));
    assert!(report.diff.contains(" line 2"));
    assert!(report.diff.contains(" line 8"));
    assert!(!report.diff.contains("-line 1"));
    assert!(!report.diff.contains("+line 9"));
}

#[test]
fn dry_run_marks_missing_final_newline() {
    let original = "alpha\nbeta";
    let start = original.find("beta").expect("line exists");
    let end = start + "beta".len();
    let remediation = Remediation::AutoFix(Edit::new(
        "fixture.txt",
        ByteRange::new(start, end).expect("valid replacement range"),
        "BETA".to_owned(),
    ));

    let report = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: original,
    })
    .expect("dry-run should render newline marker");

    assert!(report.diff.contains("-beta\n\\ No newline at end of file"));
    assert!(report.diff.contains("+BETA\n\\ No newline at end of file"));
}

#[test]
fn policy_is_propose_only_and_cannot_merge_or_bypass_gates() {
    let policy = BotPolicy::propose_only();

    policy
        .authorize(Action::DryRun)
        .expect("dry-run is always allowed");
    policy
        .authorize(Action::ProposeReviewablePullRequest)
        .expect("reviewable PR proposal is allowed");

    assert!(policy.authorize(Action::MergePullRequest).is_err());
    assert!(policy.authorize(Action::BypassGates).is_err());
}

#[test]
fn dry_run_rejects_out_of_bounds_ranges() {
    let original = "[package]\n";
    let remediation = Remediation::AutoFix(Edit::new(
        "libs/example/Cargo.toml",
        ByteRange::new(0, original.len() + 1).expect("contract permits range construction"),
        "",
    ));

    let error = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: original,
    })
    .expect_err("dry-run must reject byte ranges outside the original text");

    assert!(matches!(error, AutofixError::ByteRangeOutOfBounds { .. }));
}

#[test]
fn dry_run_rejects_non_utf8_boundary_ranges() {
    let original = "name = \"é\"\n";
    let remediation = Remediation::AutoFix(Edit::new(
        "libs/example/Cargo.toml",
        ByteRange::new(9, 9).expect("contract permits byte offsets"),
        "e",
    ));

    let error = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: original,
    })
    .expect_err("dry-run must reject ranges that split UTF-8 code points");

    assert!(matches!(
        error,
        AutofixError::ByteRangeNotUtf8Boundary { .. }
    ));
}

#[test]
fn dry_run_rejects_unreviewable_paths() {
    let original = "[package]\n";
    let remediation = Remediation::AutoFix(Edit::new(
        "../Cargo.toml\n+++ b/forged",
        ByteRange::new(0, 0).expect("valid insertion range"),
        "license = \"Apache-2.0\"\n",
    ));

    let error = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: original,
    })
    .expect_err("dry-run must reject paths that can forge diff headers");

    assert!(matches!(error, AutofixError::InvalidPath { .. }));
}

#[test]
fn dry_run_rejects_parent_dir_component() {
    let remediation = Remediation::AutoFix(Edit::new(
        "../Cargo.toml",
        ByteRange::new(0, 0).expect("valid insertion range"),
        "x",
    ));

    let error = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: "",
    })
    .expect_err("dry-run must reject a '..' path component");

    assert!(matches!(error, AutofixError::InvalidPath { .. }));
}

#[test]
fn dry_run_rejects_absolute_path() {
    let remediation = Remediation::AutoFix(Edit::new(
        "/etc/passwd",
        ByteRange::new(0, 0).expect("valid insertion range"),
        "x",
    ));

    let error = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: "",
    })
    .expect_err("dry-run must reject an absolute path");

    assert!(matches!(error, AutofixError::InvalidPath { .. }));
}

#[test]
fn dry_run_rejects_cur_dir_component() {
    let remediation = Remediation::AutoFix(Edit::new(
        "./x",
        ByteRange::new(0, 0).expect("valid insertion range"),
        "x",
    ));

    let error = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: "",
    })
    .expect_err("dry-run must reject a '.' path component");

    assert!(matches!(error, AutofixError::InvalidPath { .. }));
}

#[test]
fn dry_run_rejects_empty_path() {
    let remediation = Remediation::AutoFix(Edit::new(
        "",
        ByteRange::new(0, 0).expect("valid insertion range"),
        "x",
    ));

    let error = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: "",
    })
    .expect_err("dry-run must reject an empty path");

    assert!(matches!(error, AutofixError::InvalidPath { .. }));
}

#[test]
fn dry_run_rejects_control_characters_in_path() {
    for bad_char in ['\n', '\r', '\0'] {
        let path = format!("src/bad{bad_char}name.rs");
        let remediation = Remediation::AutoFix(Edit::new(
            path,
            ByteRange::new(0, 0).expect("valid insertion range"),
            "x",
        ));

        let error = render_dry_run(DryRunInput {
            remediation: &remediation,
            original_text: "",
        })
        .expect_err("dry-run must reject control characters in the path");

        assert!(matches!(error, AutofixError::InvalidPath { .. }));
    }
}

#[test]
fn dry_run_rejects_no_remediation() {
    let error = render_dry_run(DryRunInput {
        remediation: &Remediation::None,
        original_text: "",
    })
    .expect_err("dry-run must reject when there is no remediation to render");

    assert!(matches!(error, AutofixError::NoRemediation));
}

#[test]
fn dry_run_new_file_diff_is_git_apply_clean_when_non_empty() {
    let remediation =
        Remediation::AutoGenerate(NewFile::new("libs/example/NEW_FILE.md", "hello\n"));

    let report = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: "",
    })
    .expect("dry-run should render a new-file diff");

    assert!(
        report
            .diff
            .contains("diff --git a/libs/example/NEW_FILE.md b/libs/example/NEW_FILE.md")
    );
    assert!(report.diff.contains("new file mode 100644"));
    assert!(report.diff.contains("@@ -0,0 +1,1 @@"));
    assert!(report.diff.contains("+hello"));

    let output = git_apply_check(&report.diff, "non-empty");
    assert!(
        output.status.success(),
        "git apply --check rejected the non-empty new-file diff:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn dry_run_new_file_diff_is_git_apply_clean_when_empty() {
    let remediation = Remediation::AutoGenerate(NewFile::new("x/.gitkeep", ""));

    let report = render_dry_run(DryRunInput {
        remediation: &remediation,
        original_text: "",
    })
    .expect("dry-run should render a diff for an empty new file");

    // An empty new file has no lines to hunk over, so `---`/`+++`/`@@` are
    // omitted entirely, matching what `git diff` itself renders for a newly
    // added empty file — its section is just `diff --git` + `new file mode`.
    assert!(report.diff.contains("diff --git a/x/.gitkeep b/x/.gitkeep"));
    assert!(report.diff.contains("new file mode 100644"));
    assert!(!report.diff.contains("@@"));

    let output = git_apply_check(&report.diff, "empty");
    assert!(
        output.status.success(),
        "git apply --check rejected the empty-new-file diff:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
