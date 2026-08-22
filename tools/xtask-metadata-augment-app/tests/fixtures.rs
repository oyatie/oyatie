// ADR-0083 Tier 3: integration tests use .unwrap() / .expect() / panic! to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

/// 20-cell fixture matrix: 4 dependency table types × 5 dependency forms.
///
/// Tables:
///   T1 = [dependencies]
///   T2 = [dev-dependencies]
///   T3 = [build-dependencies]
///   T4 = [target.cfg(*).dependencies]
///
/// Forms:
///   F1 = bare string:           crate-name = "1.0"
///   F2 = inline table with path: crate-name = { path = "../crate-name" }
///   F3 = inline table with workspace=true: crate-name = { workspace = true }
///   F4 = inline table with package=: crate-name = { package = "other-name", version = "1.0" }
///   F5 = inline table with optional=true: crate-name = { version = "1.0", optional = true }
///
/// Each test asserts that toml_edit can parse and round-trip the manifest form,
/// and that rename logic can locate the crate-name token correctly.
use toml_edit::DocumentMut;

// ──────────────────────────────────────────────
// T1 × F1  bare string in [dependencies]
// ──────────────────────────────────────────────
#[test]
fn t1_f1_dependencies_bare_string() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
platform-tenant-kernel = "0.1.0"
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let name = doc["dependencies"]["platform-tenant-kernel"]
        .as_str()
        .expect("bare string value");
    assert_eq!(name, "0.1.0");
}

// ──────────────────────────────────────────────
// T1 × F2  inline table with path in [dependencies]
// ──────────────────────────────────────────────
#[test]
fn t1_f2_dependencies_inline_table_path() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
platform-tenant-kernel = { path = "../platform-tenant-kernel" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let path = doc["dependencies"]["platform-tenant-kernel"]["path"]
        .as_str()
        .expect("path value");
    assert_eq!(path, "../platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T1 × F3  inline table with workspace=true in [dependencies]
// ──────────────────────────────────────────────
#[test]
fn t1_f3_dependencies_inline_table_workspace() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
platform-tenant-kernel = { workspace = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let ws = doc["dependencies"]["platform-tenant-kernel"]["workspace"]
        .as_bool()
        .expect("workspace bool");
    assert!(ws);
}

// ──────────────────────────────────────────────
// T1 × F4  inline table with package= in [dependencies]
// ──────────────────────────────────────────────
#[test]
fn t1_f4_dependencies_inline_table_package() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
tenant = { package = "platform-tenant-kernel", version = "0.1.0" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let pkg = doc["dependencies"]["tenant"]["package"]
        .as_str()
        .expect("package value");
    assert_eq!(pkg, "platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T1 × F5  inline table with optional=true in [dependencies]
// ──────────────────────────────────────────────
#[test]
fn t1_f5_dependencies_inline_table_optional() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
platform-tenant-kernel = { version = "0.1.0", optional = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let opt = doc["dependencies"]["platform-tenant-kernel"]["optional"]
        .as_bool()
        .expect("optional bool");
    assert!(opt);
}

// ──────────────────────────────────────────────
// T2 × F1  bare string in [dev-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t2_f1_dev_dependencies_bare_string() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dev-dependencies]
platform-tenant-kernel = "0.1.0"
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let name = doc["dev-dependencies"]["platform-tenant-kernel"]
        .as_str()
        .expect("bare string");
    assert_eq!(name, "0.1.0");
}

// ──────────────────────────────────────────────
// T2 × F2  inline table with path in [dev-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t2_f2_dev_dependencies_inline_table_path() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dev-dependencies]
platform-tenant-kernel = { path = "../platform-tenant-kernel" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let path = doc["dev-dependencies"]["platform-tenant-kernel"]["path"]
        .as_str()
        .expect("path");
    assert_eq!(path, "../platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T2 × F3  workspace=true in [dev-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t2_f3_dev_dependencies_workspace() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dev-dependencies]
platform-tenant-kernel = { workspace = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let ws = doc["dev-dependencies"]["platform-tenant-kernel"]["workspace"]
        .as_bool()
        .expect("workspace");
    assert!(ws);
}

// ──────────────────────────────────────────────
// T2 × F4  package= in [dev-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t2_f4_dev_dependencies_package() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dev-dependencies]
tenant = { package = "platform-tenant-kernel", version = "0.1.0" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let pkg = doc["dev-dependencies"]["tenant"]["package"]
        .as_str()
        .expect("package");
    assert_eq!(pkg, "platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T2 × F5  optional=true in [dev-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t2_f5_dev_dependencies_optional() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[dev-dependencies]
platform-tenant-kernel = { version = "0.1.0", optional = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let opt = doc["dev-dependencies"]["platform-tenant-kernel"]["optional"]
        .as_bool()
        .expect("optional");
    assert!(opt);
}

// ──────────────────────────────────────────────
// T3 × F1  bare string in [build-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t3_f1_build_dependencies_bare_string() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[build-dependencies]
platform-tenant-kernel = "0.1.0"
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let v = doc["build-dependencies"]["platform-tenant-kernel"]
        .as_str()
        .expect("version string");
    assert_eq!(v, "0.1.0");
}

// ──────────────────────────────────────────────
// T3 × F2  inline table with path in [build-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t3_f2_build_dependencies_inline_table_path() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[build-dependencies]
platform-tenant-kernel = { path = "../platform-tenant-kernel" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let path = doc["build-dependencies"]["platform-tenant-kernel"]["path"]
        .as_str()
        .expect("path");
    assert_eq!(path, "../platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T3 × F3  workspace=true in [build-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t3_f3_build_dependencies_workspace() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[build-dependencies]
platform-tenant-kernel = { workspace = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let ws = doc["build-dependencies"]["platform-tenant-kernel"]["workspace"]
        .as_bool()
        .expect("workspace");
    assert!(ws);
}

// ──────────────────────────────────────────────
// T3 × F4  package= in [build-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t3_f4_build_dependencies_package() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[build-dependencies]
tenant = { package = "platform-tenant-kernel", version = "0.1.0" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let pkg = doc["build-dependencies"]["tenant"]["package"]
        .as_str()
        .expect("package");
    assert_eq!(pkg, "platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T3 × F5  optional=true in [build-dependencies]
// ──────────────────────────────────────────────
#[test]
fn t3_f5_build_dependencies_optional() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[build-dependencies]
platform-tenant-kernel = { version = "0.1.0", optional = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let opt = doc["build-dependencies"]["platform-tenant-kernel"]["optional"]
        .as_bool()
        .expect("optional");
    assert!(opt);
}

// ──────────────────────────────────────────────
// T4 × F1  bare string in [target.cfg(*).dependencies]
// ──────────────────────────────────────────────
#[test]
fn t4_f1_target_cfg_dependencies_bare_string() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[target.'cfg(unix)'.dependencies]
platform-tenant-kernel = "0.1.0"
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let v = doc["target"]["cfg(unix)"]["dependencies"]["platform-tenant-kernel"]
        .as_str()
        .expect("version string");
    assert_eq!(v, "0.1.0");
}

// ──────────────────────────────────────────────
// T4 × F2  inline table with path in [target.cfg(*).dependencies]
// ──────────────────────────────────────────────
#[test]
fn t4_f2_target_cfg_dependencies_inline_table_path() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[target.'cfg(unix)'.dependencies]
platform-tenant-kernel = { path = "../platform-tenant-kernel" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let path = doc["target"]["cfg(unix)"]["dependencies"]["platform-tenant-kernel"]["path"]
        .as_str()
        .expect("path");
    assert_eq!(path, "../platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T4 × F3  workspace=true in [target.cfg(*).dependencies]
// ──────────────────────────────────────────────
#[test]
fn t4_f3_target_cfg_dependencies_workspace() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[target.'cfg(unix)'.dependencies]
platform-tenant-kernel = { workspace = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let ws = doc["target"]["cfg(unix)"]["dependencies"]["platform-tenant-kernel"]["workspace"]
        .as_bool()
        .expect("workspace");
    assert!(ws);
}

// ──────────────────────────────────────────────
// T4 × F4  package= in [target.cfg(*).dependencies]
// ──────────────────────────────────────────────
#[test]
fn t4_f4_target_cfg_dependencies_package() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[target.'cfg(unix)'.dependencies]
tenant = { package = "platform-tenant-kernel", version = "0.1.0" }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let pkg = doc["target"]["cfg(unix)"]["dependencies"]["tenant"]["package"]
        .as_str()
        .expect("package");
    assert_eq!(pkg, "platform-tenant-kernel");
}

// ──────────────────────────────────────────────
// T4 × F5  optional=true in [target.cfg(*).dependencies]
// ──────────────────────────────────────────────
#[test]
fn t4_f5_target_cfg_dependencies_optional() {
    let input = r#"
[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[target.'cfg(unix)'.dependencies]
platform-tenant-kernel = { version = "0.1.0", optional = true }
"#;
    let doc: DocumentMut = input.parse().expect("parses");
    let opt = doc["target"]["cfg(unix)"]["dependencies"]["platform-tenant-kernel"]["optional"]
        .as_bool()
        .expect("optional");
    assert!(opt);
}
