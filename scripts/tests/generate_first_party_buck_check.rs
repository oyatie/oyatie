#![allow(dead_code)]

#[path = "../ci/generate-first-party-buck.rs"]
mod tool;

use std::path::Path;

fn resolver() -> tool::ThirdPartyResolver {
    tool::ThirdPartyResolver::from_buck_text(
        r#"alias(
    name = "serde",
    actual = ":serde-1",
    visibility = ["PUBLIC"],
)

rust_library(
    name = "tokio-1",
)
"#,
    )
}

#[test]
fn renders_library_unittest_path_and_registry_dependencies() {
    let model = tool::parse_manifest(
        r#"[package]
name = "oya-widget"
edition = "2024"

[dependencies]
serde = "1"
oya-core = { path = "../oya-core" }
"#,
        Path::new("libs/oya-widget"),
    );

    let output = tool::render_buck_content(
        &model,
        Path::new("libs/oya-widget"),
        Path::new("."),
        &resolver(),
        true,
        false,
    )
    .unwrap();

    assert!(output.contains("rust_library("));
    assert!(output.contains("rust_test("));
    assert!(output.contains("name = \"oya-widget-unittest\""));
    assert!(output.contains("\"//libs/oya-core:oya-core\""));
    assert!(output.contains("\"third-party//:serde\""));
}

#[test]
fn renders_binary_with_bin_suffix_when_library_name_collides() {
    let model = tool::parse_manifest(
        r#"[package]
name = "oya-app"

[[bin]]
name = "oya-app"
path = "src/main.rs"
"#,
        Path::new("oya/oya-app"),
    );

    let output = tool::render_buck_content(
        &model,
        Path::new("oya/oya-app"),
        Path::new("."),
        &resolver(),
        true,
        false,
    )
    .unwrap();

    assert!(output.contains("rust_binary("));
    assert!(output.contains("name = \"oya-app-bin\""));
    assert!(output.contains("\"//oya/oya-app:oya-app\""));
}

#[test]
fn skips_unittest_for_proc_macro_library() {
    let model = tool::parse_manifest(
        r#"[package]
name = "oya-macros"

[lib]
proc-macro = true
"#,
        Path::new("libs/oya-macros"),
    );

    let output = tool::render_buck_content(
        &model,
        Path::new("libs/oya-macros"),
        Path::new("."),
        &resolver(),
        false,
        false,
    )
    .unwrap();

    assert!(output.contains("proc_macro = True"));
    assert!(!output.contains("rust_test("));
}

#[test]
fn preserves_proto_buildscript_override_for_known_crates() {
    let model = tool::parse_manifest(
        r#"[package]
name = "oya-identity-workload-rest"

[dependencies]
tonic-prost-build = "0.14"
"#,
        Path::new("oya/identity/workload-rest"),
    );

    let output = tool::render_buck_content(
        &model,
        Path::new("oya/identity/workload-rest"),
        Path::new("."),
        &resolver(),
        true,
        false,
    )
    .unwrap();

    assert!(output.contains("buildscript_run("));
    assert!(output.contains("name = \"oya-identity-workload-rest-build-script\""));
    assert!(output.contains("OUT_DIR"));
    assert!(output.contains("third-party//:tonic-prost-build-0.14"));
}

#[test]
fn resolves_versioned_third_party_target_when_no_public_alias_exists() {
    let resolver = tool::ThirdPartyResolver::from_buck_text(
        r#"rust_library(
    name = "tonic-prost-build-0.14",
)
"#,
    );

    assert_eq!(
        resolver.resolve("tonic-prost-build", "0.14").as_deref(),
        Some("tonic-prost-build-0.14")
    );
}

#[test]
fn returns_none_when_manifest_has_no_rust_targets() {
    let model = tool::parse_manifest(
        r#"[package]
name = "empty"
"#,
        Path::new("libs/empty"),
    );

    assert!(
        tool::render_buck_content(
            &model,
            Path::new("libs/empty"),
            Path::new("."),
            &resolver(),
            false,
            false,
        )
        .is_none()
    );
}
