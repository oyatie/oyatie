use std::fs;
use std::path::{Path, PathBuf};

use oya_governance_substance_bar::{
    EnforcementStatus, RULE_ID, SubstanceViolationKind, enforce_substance_bar,
};

#[test]
fn accepts_standard_at_line_floor() {
    let root = fixture_root("substance-pass-standard");
    write(
        &root,
        "docs/standards/example.md",
        &doc_with_lines("Standard", 250),
    );

    let outcome = enforce_substance_bar(&root).expect("check should run");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.docs_with_doc_class, 1);
    assert_eq!(outcome.observations[0].required_lines, 250);
}

#[test]
fn rejects_standard_below_line_floor() {
    let root = fixture_root("substance-fail-standard");
    write(
        &root,
        "docs/standards/example.md",
        &doc_with_lines("Standard", 20),
    );

    let outcome = enforce_substance_bar(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.violations.len(), 1);
    assert_eq!(
        outcome.violations[0].kind,
        SubstanceViolationKind::BelowLineFloor
    );
    assert_eq!(outcome.violations[0].line, 2);
}

#[test]
fn accepts_markdown_without_doc_class_as_out_of_scope() {
    let root = fixture_root("substance-pass-no-frontmatter");
    write(&root, "docs/readme.md", "# No frontmatter\n");

    let outcome = enforce_substance_bar(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.docs_with_doc_class, 0);
}

#[test]
fn rejects_unknown_doc_class_floor() {
    let root = fixture_root("substance-fail-unknown");
    write(
        &root,
        "docs/custom.md",
        "---\ndoc_class: CustomThing\n---\n# Body\n",
    );

    let outcome = enforce_substance_bar(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(
        outcome.violations[0].kind,
        SubstanceViolationKind::UnknownDocClassFloor
    );
}

#[test]
fn applies_spec_floor_to_machine_readable_spec_alias() {
    let root = fixture_root("substance-pass-spec");
    write(
        &root,
        "specs/example.md",
        &doc_with_lines("Machine-Readable-Spec", 600),
    );

    let outcome = enforce_substance_bar(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.observations[0].required_lines, 600);
}

fn doc_with_lines(doc_class: &str, total_lines: usize) -> String {
    let mut doc = format!("---\ndoc_class: {doc_class}\n---\n");
    while doc.lines().count() < total_lines {
        let next = doc.lines().count() + 1;
        doc.push_str(&format!("line {next}: required substance evidence\n"));
    }
    doc
}

fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-governance-substance-{}-{}",
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
