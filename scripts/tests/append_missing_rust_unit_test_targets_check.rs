#![allow(dead_code)]

#[path = "../ci/append-missing-rust-unit-test-targets.rs"]
mod tool;

#[test]
fn appends_mirrored_rust_test_for_plain_library() {
    let input = r#"rust_library(
    name = "oya-widget",
    srcs = ["src/lib.rs"],
    crate = "oya_widget",
    deps = ["//libs/dep:dep"],
)
"#;

    let result = tool::process_text(input);

    assert_eq!(result.added, 1);
    assert!(result.text.contains("rust_library("));
    assert!(result.text.contains("rust_test("));
    assert!(result.text.contains("name = \"oya-widget-unittest\""));
    assert!(result.text.contains("deps = [\"//libs/dep:dep\"]"));
}

#[test]
fn does_not_append_when_rust_test_already_exists() {
    let input = r#"rust_library(
    name = "oya-widget",
)

rust_test(
    name = "oya-widget-unittest",
)
"#;

    let result = tool::process_text(input);

    assert_eq!(result.added, 0);
    assert_eq!(result.text, input);
}

#[test]
fn skips_proc_macro_libraries() {
    let input = r#"rust_library(
    name = "oya-macros",
    proc_macro = True,
)
"#;

    let result = tool::process_text(input);

    assert_eq!(result.added, 0);
    assert_eq!(result.text, input);
}

#[test]
fn handles_multiple_libraries_in_append_only_order() {
    let input = r#"rust_library(
    name = "first",
)

rust_library(
    name = "second",
)
"#;

    let result = tool::process_text(input);

    assert_eq!(result.added, 2);
    let first_pos = result.text.find("name = \"first-unittest\"").unwrap();
    let second_pos = result.text.find("name = \"second-unittest\"").unwrap();
    assert!(first_pos < second_pos);
}

#[test]
fn extracts_only_top_level_rule_blocks() {
    let input = r#"# rust_library(
#   name = "commented",
# )

some_macro(rust_library(
    name = "nested",
))

rust_library(
    name = "real",
)
"#;

    let blocks = tool::rust_rule_blocks(input, "rust_library");

    assert_eq!(blocks.len(), 1);
    assert_eq!(tool::library_name(blocks[0]).as_deref(), Some("real"));
}

#[test]
fn extracts_only_the_name_attribute() {
    let input = r#"rust_library(
    name_alias = "not-the-name",
    name = "actual",
)
"#;

    assert_eq!(tool::library_name(input).as_deref(), Some("actual"));
}

#[test]
fn skips_generated_vendored_dot_and_known_failing_paths() {
    assert!(tool::is_skipped_rel("third-party/rust/BUCK"));
    assert!(tool::is_skipped_rel(".github/workflows/BUCK"));
    assert!(tool::is_skipped_rel("libs/oya-check-dependency-seam/BUCK"));
    assert!(!tool::is_skipped_rel("libs/ordinary-crate/BUCK"));
}
