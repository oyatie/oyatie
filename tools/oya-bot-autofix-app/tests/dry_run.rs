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
