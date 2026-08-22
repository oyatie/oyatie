use std::fs;
use std::path::{Path, PathBuf};

use check_no_template_stamping::{EnforcementStatus, RULE_ID, enforce_no_template_stamping};

#[test]
fn rejects_three_adjacent_template_shaped_docs() {
    let root = fixture_root("template-fail-three");
    write_template_doc(&root, "docs/a/001.md", "alpha");
    write_template_doc(&root, "docs/a/002.md", "bravo");
    write_template_doc(&root, "docs/a/003.md", "charlie");

    let outcome = enforce_no_template_stamping(&root).expect("check should run");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.violations.len(), 1);
    assert_eq!(outcome.violations[0].files.len(), 3);
}

#[test]
fn accepts_two_adjacent_template_shaped_docs() {
    let root = fixture_root("template-pass-two");
    write_template_doc(&root, "docs/a/001.md", "alpha");
    write_template_doc(&root, "docs/a/002.md", "bravo");

    let outcome = enforce_no_template_stamping(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert!(outcome.violations.is_empty());
}

#[test]
fn accepts_three_docs_with_different_line_shapes() {
    let root = fixture_root("template-pass-different");
    write(
        &root,
        "docs/a/001.md",
        "# One\n\nParagraph with prose.\n\n```rust\nfn main() {}\n```\n",
    );
    write(
        &root,
        "docs/a/002.md",
        "# Two\n\n| Column | Value |\n|---|---|\n| x | y |\n",
    );
    write(
        &root,
        "docs/a/003.md",
        "# Three\n\n1. first\n2. second\n3. third\n",
    );

    let outcome = enforce_no_template_stamping(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
}

#[test]
fn does_not_join_runs_across_directories() {
    let root = fixture_root("template-pass-directories");
    write_template_doc(&root, "docs/a/001.md", "alpha");
    write_template_doc(&root, "docs/a/002.md", "bravo");
    write_template_doc(&root, "docs/b/003.md", "charlie");

    let outcome = enforce_no_template_stamping(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
}

#[test]
fn reports_run_in_microservice_directory() {
    let root = fixture_root("template-fail-microservice");
    write_template_doc(&root, "microservices/mail/runbooks/001.md", "alpha");
    write_template_doc(&root, "microservices/mail/runbooks/002.md", "bravo");
    write_template_doc(&root, "microservices/mail/runbooks/003.md", "charlie");

    let outcome = enforce_no_template_stamping(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(
        outcome.violations[0].directory,
        PathBuf::from("microservices/mail/runbooks")
    );
}

fn write_template_doc(root: &Path, relative: &str, subject: &str) {
    write(
        root,
        relative,
        &format!(
            "# {subject} runbook\n\n## Trigger\n{subject} event starts.\n\n## Procedure\n1. Check {subject} state.\n2. Emit {subject} audit.\n3. Verify {subject} state.\n\n## Rollback\nRestore {subject} route.\n"
        ),
    );
}

fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "governance-template-{}-{}",
        name,
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture root");
    }
    fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file has parent"))
        .expect("create fixture parent");
    fs::write(path, content).expect("write fixture file");
}
