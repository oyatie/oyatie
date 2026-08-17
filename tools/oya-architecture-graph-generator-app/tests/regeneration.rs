//! Repo-local regeneration proofs (always on, no external golden required):
//!   1. Rendering from the committed template + SSOT + controller-owned
//!      masterplan projection is deterministic (render twice -> identical
//!      bytes). The de-committed `product-graph.html` is no longer a git-tracked
//!      golden.
//!   2. The baked `const GRAPH = {...};` literal parses as JSON and carries the
//!      five dashboard keys in order.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use oya_architecture_graph_generator_app::render;
use serde_json::Value;

mod support;

use support::{resolve_masterplan_input, resolve_masterplan_input_with};

fn repo_root() -> PathBuf {
    std::env::current_dir()
        .expect("current dir readable")
        .ancestors()
        .find(|dir| {
            dir.join("docs/machine-readable/architecture-graph.json")
                .exists()
        })
        .expect("could not locate Oyatie repo root from current dir")
        .to_path_buf()
}

fn render_dashboard() -> String {
    let root = repo_root();
    let masterplan = resolve_masterplan_input(&root).expect("masterplan input resolves");
    render(
        &root.join("docs/machine-readable/architecture-graph.json"),
        masterplan.path(),
        &root.join("docs/architecture/product-graph.template.html"),
    )
    .expect("render succeeds")
}

fn fixture_repo() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("fixture root");
    let decisions = root.path().join("docs/decisions");
    fs::create_dir_all(&decisions).expect("decisions dir");
    fs::write(
        decisions.join("ADR-0001-test.md"),
        r#"---
status: Accepted
planning_impact: true
milestone: M-TEST
depends_on: []
deliverables:
  - id: ADR-0001-D1
    description: deterministic fixture
    exit_criteria: fixture renders
    verified_by: cargo test
---
# Fixture
"#,
    )
    .expect("write planning ADR fixture");
    root
}

fn valid_projection_bytes(root: &Path) -> String {
    ci_generated_artifact_freshness::render_masterplan_projection_from_decisions(
        &root.join("docs/decisions"),
    )
    .expect("fixture projection renders")
    .1
}

#[test]
fn absent_controller_face_materializes_only_a_temporary_projection() {
    let root = fixture_repo();
    let canonical = root
        .path()
        .join("docs/machine-readable/masterplan.generated.json");

    let input = resolve_masterplan_input_with(root.path(), None).expect("fallback materializes");

    assert!(input.is_temporary());
    assert!(input.path().is_file());
    assert!(!input.path().starts_with(root.path()));
    assert!(
        !canonical.exists(),
        "fallback must not write the repository"
    );
}

#[test]
fn declared_resource_has_precedence_over_an_existing_controller_face() {
    let root = fixture_repo();
    let canonical = root
        .path()
        .join("docs/machine-readable/masterplan.generated.json");
    fs::create_dir_all(canonical.parent().expect("canonical parent")).expect("create parent");
    fs::write(&canonical, valid_projection_bytes(root.path())).expect("write canonical face");
    let declared = root.path().join("buck-resource/masterplan.generated.json");
    fs::create_dir_all(declared.parent().expect("declared parent")).expect("create resource dir");
    fs::write(&declared, valid_projection_bytes(root.path())).expect("write declared face");

    let input = resolve_masterplan_input_with(root.path(), Some(declared.as_os_str()))
        .expect("declared resource resolves");

    assert_eq!(input.path(), declared);
    assert!(!input.is_temporary());
}

#[test]
fn declared_or_existing_malformed_inputs_fail_closed_without_fallback() {
    let root = fixture_repo();
    let missing = root.path().join("buck-resource/missing.generated.json");
    let missing_error = resolve_masterplan_input_with(root.path(), Some(missing.as_os_str()))
        .expect_err("missing declared input must fail");
    assert!(missing_error.contains("unavailable"), "{missing_error}");

    let declared = root.path().join("buck-resource/malformed.generated.json");
    fs::create_dir_all(declared.parent().expect("declared parent")).expect("create resource dir");
    fs::write(&declared, "{").expect("write malformed declared face");
    let declared_error = resolve_masterplan_input_with(root.path(), Some(declared.as_os_str()))
        .expect_err("malformed declared input must fail");
    assert!(declared_error.contains("parse masterplan projection"));

    fs::write(&declared, "{}").expect("write wrong-shape declared face");
    let shape_error = resolve_masterplan_input_with(root.path(), Some(declared.as_os_str()))
        .expect_err("wrong-shape declared input must fail");
    assert!(shape_error.contains("validate masterplan projection"));

    let directory = root.path().join("buck-resource/not-a-file");
    fs::create_dir_all(&directory).expect("create declared directory");
    let directory_error = resolve_masterplan_input_with(root.path(), Some(directory.as_os_str()))
        .expect_err("declared directory must fail");
    assert!(directory_error.contains("regular non-symlink file"));

    let canonical = root
        .path()
        .join("docs/machine-readable/masterplan.generated.json");
    fs::create_dir_all(canonical.parent().expect("canonical parent")).expect("create parent");
    fs::write(&canonical, "{").expect("write malformed canonical face");
    let canonical_error = resolve_masterplan_input_with(root.path(), None)
        .expect_err("malformed existing face must fail");
    assert!(canonical_error.contains("parse masterplan projection"));
    assert_eq!(fs::read_to_string(canonical).expect("read face"), "{");

    let empty_error = resolve_masterplan_input_with(root.path(), Some(OsStr::new("")))
        .expect_err("empty binding must fail");
    assert!(empty_error.contains("must not be empty"));
}

#[cfg(unix)]
#[test]
fn declared_symlink_input_fails_closed() {
    use std::os::unix::fs::symlink;

    let root = fixture_repo();
    let target = root.path().join("buck-resource/target.generated.json");
    fs::create_dir_all(target.parent().expect("target parent")).expect("create resource dir");
    fs::write(&target, valid_projection_bytes(root.path())).expect("write target face");
    let declared = root.path().join("buck-resource/symlink.generated.json");
    symlink(&target, &declared).expect("create declared symlink");

    let error = resolve_masterplan_input_with(root.path(), Some(declared.as_os_str()))
        .expect_err("declared symlink must fail");
    assert!(error.contains("regular non-symlink file"));
}

#[test]
fn temporary_projection_is_byte_deterministic() {
    let root = fixture_repo();
    let first = resolve_masterplan_input_with(root.path(), None).expect("first materialization");
    let second = resolve_masterplan_input_with(root.path(), None).expect("second materialization");

    assert_ne!(first.path(), second.path());
    assert_eq!(
        fs::read(first.path()).expect("read first"),
        fs::read(second.path()).expect("read second")
    );
}

#[test]
fn regenerated_product_graph_is_deterministic() {
    let first = render_dashboard();
    let second = render_dashboard();
    assert_eq!(
        first, second,
        "regenerating product-graph.html from identical source inputs must be byte-deterministic; failures indicate nondeterministic generation or source-input drift, not a missing committed HTML golden"
    );
}

#[test]
fn baked_graph_literal_parses_with_dashboard_keys() {
    let rendered = render_dashboard();
    let prefix = "const GRAPH = ";
    let start = rendered.find(prefix).expect("const GRAPH present") + prefix.len();
    // Brace-match to extract the JSON object literal.
    let bytes = rendered.as_bytes();
    let mut depth = 0usize;
    let mut in_str = false;
    let mut esc = false;
    let mut end = start;
    for (offset, &b) in bytes[start..].iter().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end = start + offset + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    let literal = &rendered[start..end];
    let parsed: Value = serde_json::from_str(literal).expect("baked GRAPH parses as JSON");
    let keys: Vec<&str> = parsed
        .as_object()
        .expect("GRAPH is an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        ["_meta", "verticals", "techstack", "masterplan", "lanes"]
    );
}
