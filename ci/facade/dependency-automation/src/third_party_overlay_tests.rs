use super::*;

fn aws_rule() -> &'static str {
    r#"buildscript_run(
    name = "aws-lc-rs-1-build-script-run",
    env = {
        "CARGO_PKG_VERSION_PRE": "",
    },
)
"#
}

fn psm_rule() -> &'static str {
    r#"cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = [
        "-DCFG_TARGET_OS_darwin",
        "-DCFG_TARGET_ARCH_aarch64",
        "-DCFG_TARGET_ENV_",
    ],
)
"#
}

fn unpatched_fixture() -> String {
    format!("{}{}", aws_rule(), psm_rule())
}

fn patched_fixture() -> &'static str {
    r#"buildscript_run(
    name = "aws-lc-rs-1-build-script-run",
    env = {
        "CARGO_PKG_VERSION_PRE": "",
        # DEP_* propagated from aws-lc-sys (links = "aws_lc_0_41_0") — reindeer
        # cannot emit the $(location) macro, so it is injected post-buckify.
        "DEP_AWS_LC_0_41_0_INCLUDE": "$(location :aws-lc-sys-0.41-build-script-main-run[out_dir])/include",
        "DEP_AWS_LC_0_41_0_LIBCRYPTO": "aws_lc_0_41_0_crypto",
        "CARGO_FEATURE_AWS_LC_SYS": "1",
    },
)
cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = select({
        "prelude//os:linux": [
            "-DCFG_TARGET_OS_linux",
            "-DCFG_TARGET_ARCH_aarch64",
            "-DCFG_TARGET_ENV_",
        ],
        "DEFAULT": [
            "-DCFG_TARGET_OS_darwin",
            "-DCFG_TARGET_ARCH_aarch64",
            "-DCFG_TARGET_ENV_",
        ],
    }),
)
"#
}

fn assert_contract_error(source: &str, expected: &str) {
    let error = apply_third_party_buck_overlay(source)
        .expect_err("invalid generated shape must fail closed");
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?}, got {error}"
    );
}

#[test]
fn applies_exact_golden_overlay_once_and_is_idempotent() {
    let first = apply_third_party_buck_overlay(&unpatched_fixture()).expect("apply exact overlay");
    assert_eq!(first.patches_applied, 2);
    assert_eq!(first.text, patched_fixture());

    let second = apply_third_party_buck_overlay(&first.text).expect("reapply exact overlay");
    assert_eq!(second.patches_applied, 0);
    assert_eq!(second.text, first.text);
}

#[test]
fn refuses_a_missing_generated_rule_or_anchor() {
    assert_contract_error(psm_rule(), "aws-lc-rs-1-build-script-run");
    let missing_anchor =
        aws_rule().replace("        \"CARGO_PKG_VERSION_PRE\": \"\",\n", "") + psm_rule();
    assert_contract_error(&missing_anchor, "CARGO_PKG_VERSION_PRE anchor");
}

#[test]
fn refuses_a_corrupt_aws_dep_overlay() {
    let source = aws_rule().replace(
        "        \"CARGO_PKG_VERSION_PRE\": \"\",",
        r#"        "CARGO_PKG_VERSION_PRE": "",
        "DEP_AWS_LC_0_41_0_INCLUDE": "wrong","#,
    ) + psm_rule();
    assert_contract_error(&source, "incomplete or corrupt DEP overlay");
}

#[test]
fn refuses_a_partial_aws_dep_overlay_without_include() {
    let source = aws_rule().replace(
        "        \"CARGO_PKG_VERSION_PRE\": \"\",",
        r#"        "CARGO_PKG_VERSION_PRE": "",
        "DEP_AWS_LC_0_41_0_LIBCRYPTO": "aws_lc_0_41_0_crypto","#,
    ) + psm_rule();
    assert_contract_error(&source, "incomplete or corrupt DEP overlay");
}

#[test]
fn refuses_unexpected_psm_shape() {
    let unexpected = r#"cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = ["unexpected"],
)
"#;
    assert_contract_error(
        &(aws_rule().to_owned() + unexpected),
        "unexpected generated preprocessor flags",
    );
}

#[test]
fn refuses_a_partial_psm_platform_overlay() {
    let partial = r#"cxx_library(
    name = "psm-0.1-psm_asm",
    preprocessor_flags = select({
        "prelude//os:linux": [],
        "DEFAULT": [],
    }),
)
"#;
    assert_contract_error(
        &(aws_rule().to_owned() + partial),
        "incomplete or corrupt platform overlay",
    );
}

#[test]
fn refuses_duplicate_generated_rules_and_anchors() {
    assert_contract_error(
        &(aws_rule().repeat(2) + psm_rule()),
        "expected exactly one generated rule",
    );
    let duplicate_anchor = aws_rule().replace(
        "        \"CARGO_PKG_VERSION_PRE\": \"\",",
        r#"        "CARGO_PKG_VERSION_PRE": "",
        "CARGO_PKG_VERSION_PRE": "","#,
    ) + psm_rule();
    assert_contract_error(
        &duplicate_anchor,
        "expected exactly one CARGO_PKG_VERSION_PRE anchor",
    );
}

#[test]
fn refuses_unterminated_generated_rule() {
    let unterminated = aws_rule().trim_end_matches(")\n");
    assert_contract_error(unterminated, "unterminated generated rule");
}

#[test]
fn refuses_mixed_psm_baseline_and_overlay_attributes() {
    let mixed = patched_fixture().replace(
        "    preprocessor_flags = select({",
        r#"    preprocessor_flags = [
        "-DCFG_TARGET_OS_darwin",
        "-DCFG_TARGET_ARCH_aarch64",
        "-DCFG_TARGET_ENV_",
    ],
    preprocessor_flags = select({"#,
    );
    assert_contract_error(&mixed, "expected exactly one preprocessor_flags attribute");
}
