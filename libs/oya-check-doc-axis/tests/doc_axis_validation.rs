//! Integration tests for the doc-axis enforcement kernel (ADR-0388).
//!
//! Each test sets up a synthetic repo using `tempfile::TempDir`, runs
//! `oya_check_doc_axis::validate`, and asserts the expected outcome.

use std::fs;
use std::path::Path;

use oya_check_doc_axis::{DocAxisRule, validate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a file and all parent directories, writing `contents`.
fn write_file(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

/// Set OYA_TODAY so date-sensitive tests are reproducible.
/// SAFETY (Rust 2024): set_var is unsafe because it can race with reads from
/// other threads. These tests are single-threaded per `cargo test --test-threads=1`
/// expectation; the env-var is only read by the gate validator we call below.
fn set_today(date: &str) {
    unsafe {
        std::env::set_var("OYA_TODAY", date);
    }
}

// ---------------------------------------------------------------------------
// Rule 1 — ADR status casing
// ---------------------------------------------------------------------------

#[test]
fn rule1_clean_adr_passes_non_strict() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "docs/decisions/ADR-0001-test.md",
        "---\nid: ADR-0001\nstatus: Accepted\n---\n# body\n",
    );
    let result = validate(root, false);
    assert!(result.is_ok(), "clean ADR should pass: {result:?}");
    let report = result.unwrap();
    assert_eq!(report.adrs_checked, 1);
}

#[test]
fn rule1_bad_casing_is_warning_in_non_strict_mode() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "docs/decisions/ADR-0001-test.md",
        "---\nid: ADR-0001\nstatus: accepted\n---\n# body\n",
    );
    // Non-strict: bad casing is a warning, not a blocking error.
    let result = validate(root, false);
    // Should still pass (no blocking findings).
    assert!(
        result.is_ok(),
        "non-strict should pass with warning: {result:?}"
    );
    let report = result.unwrap();
    assert_eq!(report.warnings, 1, "should have 1 warning for bad casing");
}

#[test]
fn rule1_bad_casing_blocks_in_strict_mode() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "docs/decisions/ADR-0001-test.md",
        "---\nid: ADR-0001\nstatus: accepted\n---\n# body\n",
    );
    let result = validate(root, true);
    assert!(result.is_err(), "strict mode should block on bad casing");
    let findings = result.unwrap_err();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_violated, DocAxisRule::AdrStatusCasing);
    assert!(findings[0].blocking);
    assert_eq!(findings[0].line, Some(3));
}

// ---------------------------------------------------------------------------
// Rule 2 — No shadow ideas
// ---------------------------------------------------------------------------

#[test]
fn rule2_fresh_idea_passes() {
    set_today("2026-05-28");
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // File dated today — 0 days old, within the 14-day window.
    write_file(
        root,
        "docs/ideas/my-idea-2026-05-28.md",
        "---\ntitle: test\n---\n# idea\n",
    );
    let result = validate(root, false);
    assert!(result.is_ok(), "fresh idea should pass: {result:?}");
}

#[test]
fn rule2_stale_idea_without_promotion_blocks() {
    set_today("2026-06-15");
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // File dated 2026-05-28 — 18 days old at 2026-06-15, over the limit.
    write_file(
        root,
        "docs/ideas/my-idea-2026-05-28.md",
        "---\ntitle: test\n---\n# no superseded_by\n",
    );
    let result = validate(root, false);
    assert!(result.is_err(), "stale idea without promotion should block");
    let findings = result.unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.rule_violated == DocAxisRule::ShadowIdea),
        "should have a ShadowIdea finding"
    );
}

#[test]
fn rule2_stale_idea_with_valid_superseded_by_still_blocks_until_deleted_from_head() {
    set_today("2026-06-15");
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // The idea-pager cites ADR-0389, which actually exists.
    write_file(
        root,
        "docs/decisions/ADR-0389-promoted.md",
        "---\nid: ADR-0389\nstatus: Accepted\n---\n# body\n",
    );
    write_file(
        root,
        "docs/ideas/my-idea-2026-05-28.md",
        "---\ntitle: test\nsuperseded_by: ADR-0389\n---\n# promoted\n",
    );
    let result = validate(root, false);
    assert!(
        result.is_err(),
        "a promoted idea still present in candidate HEAD must block: {result:?}"
    );
    assert!(
        result
            .unwrap_err()
            .iter()
            .any(|finding| finding.rule_violated == DocAxisRule::ShadowIdea)
    );
}

// ---------------------------------------------------------------------------
// Rule 3 — No docs proliferation
// ---------------------------------------------------------------------------

#[test]
fn rule3_file_directly_under_docs_blocks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_file(root, "docs/STRAY.md", "# stray file\n");
    let result = validate(root, false);
    assert!(result.is_err(), "stray md under docs/ should block");
    let findings = result.unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.rule_violated == DocAxisRule::DocsProliferation),
        "should have a DocsProliferation finding"
    );
}

#[test]
fn rule3_unknown_subdir_under_docs_blocks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_file(root, "docs/shadow-zone/something.md", "# shadow\n");
    let result = validate(root, false);
    assert!(result.is_err(), "unknown subdir should block");
    let findings = result.unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.rule_violated == DocAxisRule::DocsProliferation),
        "should have a DocsProliferation finding for unknown subdir"
    );
}

#[test]
fn rule3_readable_archive_directory_under_canonical_axis_blocks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "docs/ideas/archive/promoted-idea.md",
        "superseded current-tree copy\n",
    );
    let result = validate(root, false);
    assert!(result.is_err(), "readable archive directory must block");
    assert!(
        result
            .unwrap_err()
            .iter()
            .any(|finding| finding.rule_violated == DocAxisRule::ReadableArchiveDirectory)
    );
}

#[test]
fn rule3_legacy_root_allowlist_passes_but_new_root_docs_block() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_file(root, "docs/AGENTS.md", "# existing contract\n");
    write_file(
        root,
        "docs/architecture/overview.md",
        "# legacy directory\n",
    );

    let result = validate(root, false);
    assert!(
        result.is_ok(),
        "legacy docs allowlist should keep existing corpus green: {result:?}"
    );

    write_file(root, "docs/NEW-ROOT-DOC.md", "# new root doc\n");
    let result = validate(root, false);
    assert!(result.is_err(), "new root docs should still block");
    let findings = result.unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.rule_violated == DocAxisRule::DocsProliferation),
        "new root doc should have a DocsProliferation finding"
    );
}

#[test]
fn rule3_canonical_subdirs_pass() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // Place files in each canonical subdir — all should pass rule 3.
    write_file(
        root,
        "docs/decisions/ADR-0001-test.md",
        "---\nid: ADR-0001\nstatus: Accepted\n---\n",
    );
    write_file(root, "docs/conventions/naming.md", "# naming\n");
    write_file(root, "docs/products/my-product.md", "# product\n");
    let result = validate(root, false);
    // Rule 3 should produce no findings; only check that rule 3 violations are absent.
    if let Err(findings) = &result {
        for f in findings {
            assert_ne!(
                f.rule_violated,
                DocAxisRule::DocsProliferation,
                "canonical subdirs should not trigger DocsProliferation"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Rule 4 — Catalog/manifest crate-claim consistency
// ---------------------------------------------------------------------------

#[test]
fn rule4_manifest_crate_missing_from_catalog_blocks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let manifest = r#"{
  "name": "my-service",
  "bounded_contexts": [
    { "name": "core", "crates": ["oya-check-doc-axis"] }
  ]
}"#;
    write_file(root, "microservices/my-service/manifest.json", manifest);
    // Do NOT create registry/catalog/oya-check-doc-axis.yaml.
    let result = validate(root, false);
    assert!(result.is_err(), "missing catalog entry should block");
    let findings = result.unwrap_err();
    assert!(
        findings
            .iter()
            .any(|f| f.rule_violated == DocAxisRule::CatalogManifestDrift),
        "should have a CatalogManifestDrift finding"
    );
}

#[test]
fn rule4_manifest_with_matching_catalog_passes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let manifest = r#"{
  "name": "my-service",
  "bounded_contexts": [
    { "name": "core", "crates": ["oya-check-doc-axis"] }
  ]
}"#;
    write_file(root, "microservices/my-service/manifest.json", manifest);
    write_file(
        root,
        "registry/catalog/oya-check-doc-axis.yaml",
        "context: governance\nrole: gate\n",
    );
    let result = validate(root, false);
    // Rule 4 should produce no findings.
    if let Err(findings) = &result {
        for f in findings {
            assert_ne!(
                f.rule_violated,
                DocAxisRule::CatalogManifestDrift,
                "matched catalog should not trigger CatalogManifestDrift"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Clean empty repo passes
// ---------------------------------------------------------------------------

#[test]
fn empty_repo_passes() {
    let dir = tempfile::tempdir().unwrap();
    let result = validate(dir.path(), false);
    assert!(result.is_ok(), "empty repo should pass: {result:?}");
}
