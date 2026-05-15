//! F3 adversarial fixtures for the `seam-imports` sub-check.
//!
//! Each test builds a synthetic workspace skeleton in `tempdir` (no fake
//! crate compilation needed — `check_seam_imports` only reads Cargo.toml
//! contents + the dependency-rationales registry) and asserts the lane
//! correctly distinguishes allowed vs violating crates.
//!
//! The point of these tests: prove the lane CATCHES violations, not just
//! that the live workspace happens to be clean today (which only proves
//! Phase 2 worked, not that the lane works).

use std::fs;
use std::path::{Path, PathBuf};

use oya_check_dependency_seam::{
    SubCheckStatus, WorkspaceContext, cargo_toml_declares_dep, check_seam_imports,
    extract_package_name, parse_isolated_in_crate, read_allowed_isolation,
};

use std::sync::atomic::{AtomicU64, Ordering};
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn make_tmp_workspace() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("oya-seam-test-{}-{}-{}", pid, stamp, seq));
    fs::create_dir_all(base.join("crates")).unwrap();
    fs::create_dir_all(base.join("registries/cross-cutting")).unwrap();
    base
}

fn write_crate(root: &Path, name: &str, deps: &[&str]) {
    let dir = root.join("crates").join(name);
    fs::create_dir_all(&dir).unwrap();
    let mut cargo = String::new();
    cargo.push_str("[package]\n");
    cargo.push_str(&format!("name = \"{}\"\n", name));
    cargo.push_str("\n[dependencies]\n");
    for d in deps {
        cargo.push_str(&format!("{}.workspace = true\n", d));
    }
    fs::write(dir.join("Cargo.toml"), cargo).unwrap();
}

fn write_registry(root: &Path, isolated_in_crate: &str) {
    let body = format!(
        r#"{{
  "entries": {{
    "hyper": {{ "isolated_in_crate": "{0}" }},
    "hyper-util": {{ "isolated_in_crate": "{0}" }},
    "http-body-util": {{ "isolated_in_crate": "{0}" }},
    "bytes": {{ "isolated_in_crate": "{0}" }}
  }}
}}
"#,
        isolated_in_crate
    );
    fs::write(
        root.join("registries/cross-cutting/dependency-rationales.json"),
        body,
    )
    .unwrap();
}

// F3 PASSING fixture: a clean workspace where the only hyper-family-declaring
// crate IS the registered isolation crate. Lane MUST report Pass.
#[test]
fn lane_passes_when_only_isolated_crate_declares_hyper() {
    let root = make_tmp_workspace();
    write_crate(&root, "oya-http-runtime-hyper-adapter", &["hyper", "bytes"]);
    write_crate(&root, "oya-domain-something", &[]);
    write_registry(&root, "oya-http-runtime-hyper-adapter");

    let ctx = WorkspaceContext::new(&root);
    let result = check_seam_imports(&ctx);
    assert_eq!(result.status, SubCheckStatus::Pass, "{:?}", result.findings);
}

// F3 FAILING fixture: a non-isolated crate declares `bytes`. Lane MUST
// detect the violation and report Fail with the exact crate name + dep.
#[test]
fn lane_fails_when_non_isolated_crate_declares_bytes() {
    let root = make_tmp_workspace();
    write_crate(&root, "oya-http-runtime-hyper-adapter", &["hyper", "bytes"]);
    // VIOLATION: a kernel crate that declares bytes.
    write_crate(&root, "oya-http-middleware-kernel-stale", &["bytes"]);
    write_registry(&root, "oya-http-runtime-hyper-adapter");

    let ctx = WorkspaceContext::new(&root);
    let result = check_seam_imports(&ctx);
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    assert!(joined.contains("oya-http-middleware-kernel-stale"));
    assert!(joined.contains("bytes"));
    assert!(joined.contains("outside the allowed-isolation set"));
}

// F3: multiple violations are ALL reported, not just the first.
#[test]
fn lane_reports_every_violation_not_just_first() {
    let root = make_tmp_workspace();
    write_crate(&root, "oya-http-runtime-hyper-adapter", &["hyper", "bytes"]);
    write_crate(&root, "violator-a", &["hyper"]);
    write_crate(&root, "violator-b", &["bytes", "hyper-util"]);
    write_registry(&root, "oya-http-runtime-hyper-adapter");

    let ctx = WorkspaceContext::new(&root);
    let result = check_seam_imports(&ctx);
    assert_eq!(result.status, SubCheckStatus::Fail);
    let joined = result.findings.join("\n");
    assert!(joined.contains("violator-a"));
    assert!(joined.contains("violator-b"));
    // violator-b declares two tracked deps — both must surface.
    assert!(joined.contains("hyper-util"));
}

// F3 boundary: a crate whose name PREFIX-MATCHES a tracked dep (e.g.
// `hyper-something`) MUST NOT match `hyper`. cargo_toml_declares_dep
// requires a separator after the dep name.
#[test]
fn dep_name_match_requires_separator_no_prefix_collision() {
    let raw = "[dependencies]\nhyper-something.workspace = true\n";
    assert!(!cargo_toml_declares_dep(raw, "hyper"));
    let raw_ok = "[dependencies]\nhyper.workspace = true\n";
    assert!(cargo_toml_declares_dep(raw_ok, "hyper"));
}

// F3: comments after `#` MUST NOT trigger detection.
#[test]
fn commented_dep_does_not_count() {
    let raw = "[dependencies]\n# hyper.workspace = true (commented out)\n";
    assert!(!cargo_toml_declares_dep(raw, "hyper"));
}

// F3: dotted-syntax variants also detected.
#[test]
fn dotted_syntax_dep_form_detected() {
    let raw = "[dependencies]\nhyper.workspace = true\n";
    assert!(cargo_toml_declares_dep(raw, "hyper"));
    let raw2 = "[dependencies]\nhyper.version = \"1\"\n";
    assert!(cargo_toml_declares_dep(raw2, "hyper"));
    let raw3 = "[dependencies]\nhyper = { workspace = true }\n";
    assert!(cargo_toml_declares_dep(raw3, "hyper"));
}

// F3: extract_package_name returns the value inside [package].
#[test]
fn extract_package_name_finds_name() {
    let raw =
        "[package]\nname = \"oya-foo-bar\"\nversion = \"0.1\"\n[lib]\nname = \"oya_foo_bar\"\n";
    assert_eq!(extract_package_name(raw).as_deref(), Some("oya-foo-bar"));
}

// F3: extract_package_name ignores non-[package] name fields (e.g., [lib] name).
#[test]
fn extract_package_name_ignores_lib_name_field() {
    let raw = "[lib]\nname = \"wrong\"\n[package]\nname = \"right\"\n";
    assert_eq!(extract_package_name(raw).as_deref(), Some("right"));
}

// F3: parse_isolated_in_crate handles the free-form value shapes that
// appear in dependency-rationales.json today.
#[test]
fn parse_isolated_in_crate_handles_realistic_shapes() {
    let v = "oya-http-runtime-hyper-adapter";
    assert_eq!(
        parse_isolated_in_crate(v),
        vec!["oya-http-runtime-hyper-adapter"]
    );

    let v = "oya-http-runtime-hyper-adapter (primary); cell-runtime main.rs entry";
    let parsed = parse_isolated_in_crate(v);
    assert!(parsed.contains(&"oya-http-runtime-hyper-adapter".to_string()));

    let v = "tooling binaries";
    let parsed = parse_isolated_in_crate(v);
    assert!(parsed.iter().any(|s| s == "tooling"));
}

// F3: read_allowed_isolation honors per-dep `isolated_in_crate`.
#[test]
fn read_allowed_isolation_keys_each_tracked_dep() {
    let root = make_tmp_workspace();
    write_registry(&root, "oya-http-runtime-hyper-adapter");
    let map = read_allowed_isolation(&root);
    assert_eq!(map.len(), 4);
    assert!(map.contains_key("hyper"));
    assert!(map.contains_key("bytes"));
    assert!(map.contains_key("hyper-util"));
    assert!(map.contains_key("http-body-util"));
    for v in map.values() {
        assert_eq!(v, &vec!["oya-http-runtime-hyper-adapter".to_string()]);
    }
}
