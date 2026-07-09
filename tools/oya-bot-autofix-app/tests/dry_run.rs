use oya_bot_autofix_app::{Action, BotPolicy, DeliveryMode, DryRunInput, render_dry_run};
use oya_ci_gate_contract::{ByteRange, Edit, Remediation};

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
