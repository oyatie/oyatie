//! Fail-closed semantic overlay for Reindeer-generated `third-party/BUCK`.
//!
//! Reindeer cannot currently express the Buck2 `$(location ...)` value needed
//! for AWS-LC build-script metadata or the per-OS PSM preprocessor `select()`.
//! This module applies those two exact fragments to a temporary Reindeer output.
//! Generated rule drift, partial prior overlays, and duplicate anchors are
//! rejected before the file boundary writes anything.

use std::fs;
use std::path::Path;

const AWS_RULE: &str = "aws-lc-rs-1-build-script-run";
const AWS_OWNED_KEYS: [&str; 3] = [
    "DEP_AWS_LC_0_41_0_INCLUDE",
    "DEP_AWS_LC_0_41_0_LIBCRYPTO",
    "CARGO_FEATURE_AWS_LC_SYS",
];
const AWS_OVERLAY: [&str; 5] = [
    "        # DEP_* propagated from aws-lc-sys (links = \"aws_lc_0_41_0\") — reindeer",
    "        # cannot emit the $(location) macro, so it is injected post-buckify.",
    "        \"DEP_AWS_LC_0_41_0_INCLUDE\": \"$(location :aws-lc-sys-0.41-build-script-main-run[out_dir])/include\",",
    "        \"DEP_AWS_LC_0_41_0_LIBCRYPTO\": \"aws_lc_0_41_0_crypto\",",
    "        \"CARGO_FEATURE_AWS_LC_SYS\": \"1\",",
];

const PSM_RULE: &str = "psm-0.1-psm_asm";
const PSM_MARKER: &str = "\"prelude//os:linux\": [";
const PSM_BASELINE: [&str; 5] = [
    "    preprocessor_flags = [",
    "        \"-DCFG_TARGET_OS_darwin\",",
    "        \"-DCFG_TARGET_ARCH_aarch64\",",
    "        \"-DCFG_TARGET_ENV_\",",
    "    ],",
];
const PSM_OVERLAY: [&str; 12] = [
    "    preprocessor_flags = select({",
    "        \"prelude//os:linux\": [",
    "            \"-DCFG_TARGET_OS_linux\",",
    "            \"-DCFG_TARGET_ARCH_aarch64\",",
    "            \"-DCFG_TARGET_ENV_\",",
    "        ],",
    "        \"DEFAULT\": [",
    "            \"-DCFG_TARGET_OS_darwin\",",
    "            \"-DCFG_TARGET_ARCH_aarch64\",",
    "            \"-DCFG_TARGET_ENV_\",",
    "        ],",
    "    }),",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThirdPartyOverlay {
    pub text: String,
    pub patches_applied: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThirdPartyOverlayError {
    Contract(String),
    Io(String),
}

impl std::fmt::Display for ThirdPartyOverlayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract(message) | Self::Io(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for ThirdPartyOverlayError {}

pub fn apply_third_party_buck_overlay(
    source: &str,
) -> Result<ThirdPartyOverlay, ThirdPartyOverlayError> {
    let mut lines = source.lines().map(str::to_owned).collect::<Vec<_>>();
    let mut patches_applied = 0;

    apply_aws_overlay(&mut lines, &mut patches_applied)?;
    apply_psm_overlay(&mut lines, &mut patches_applied)?;

    Ok(ThirdPartyOverlay {
        text: format!("{}\n", lines.join("\n")),
        patches_applied,
    })
}

pub fn apply_third_party_buck_overlay_file(path: &Path) -> Result<usize, ThirdPartyOverlayError> {
    let source = fs::read_to_string(path)
        .map_err(|error| ThirdPartyOverlayError::Io(format!("read {}: {error}", path.display())))?;
    let overlay = apply_third_party_buck_overlay(&source)?;
    if overlay.patches_applied > 0 {
        fs::write(path, overlay.text).map_err(|error| {
            ThirdPartyOverlayError::Io(format!("write {}: {error}", path.display()))
        })?;
    }
    Ok(overlay.patches_applied)
}

fn apply_aws_overlay(
    lines: &mut Vec<String>,
    patches_applied: &mut usize,
) -> Result<(), ThirdPartyOverlayError> {
    let rule_start = unique_rule_start(lines, AWS_RULE)?;
    let rule_end = rule_end(lines, rule_start)?;
    let anchor_line = "        \"CARGO_PKG_VERSION_PRE\": \"\",";
    let anchors = lines[rule_start..=rule_end]
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| (line == anchor_line).then_some(rule_start + offset))
        .collect::<Vec<_>>();
    let [anchor] = anchors.as_slice() else {
        return Err(contract_error(format!(
            "expected exactly one CARGO_PKG_VERSION_PRE anchor in {AWS_RULE}, found {}",
            anchors.len()
        )));
    };

    let owned_occurrences = AWS_OWNED_KEYS.map(|key| {
        lines[rule_start..=rule_end]
            .iter()
            .filter(|line| line.contains(key))
            .count()
    });
    if lines_match(lines, *anchor + 1, &AWS_OVERLAY)
        && owned_occurrences.iter().all(|count| *count == 1)
    {
        return Ok(());
    }
    if owned_occurrences.iter().any(|count| *count > 0) {
        return Err(contract_error(format!(
            "incomplete or corrupt DEP overlay in {AWS_RULE}: {owned_occurrences:?}"
        )));
    }

    lines.splice(
        *anchor + 1..*anchor + 1,
        AWS_OVERLAY.iter().map(|line| (*line).to_owned()),
    );
    *patches_applied += 1;
    Ok(())
}

fn apply_psm_overlay(
    lines: &mut Vec<String>,
    patches_applied: &mut usize,
) -> Result<(), ThirdPartyOverlayError> {
    let rule_start = unique_rule_start(lines, PSM_RULE)?;
    let rule_end = rule_end(lines, rule_start)?;
    let window = &lines[rule_start..=rule_end];
    let attribute_count = window
        .iter()
        .filter(|line| line.starts_with("    preprocessor_flags = "))
        .count();
    if attribute_count != 1 {
        return Err(contract_error(format!(
            "expected exactly one preprocessor_flags attribute in {PSM_RULE}, found \
             {attribute_count}"
        )));
    }
    let overlay_starts = window
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| (line == PSM_OVERLAY[0]).then_some(rule_start + offset))
        .collect::<Vec<_>>();
    let marker_count = window
        .iter()
        .filter(|line| line.contains(PSM_MARKER))
        .count();

    if let [overlay_start] = overlay_starts.as_slice() {
        if marker_count == 1 && lines_match(lines, *overlay_start, &PSM_OVERLAY) {
            return Ok(());
        }
        return Err(contract_error(format!(
            "incomplete or corrupt platform overlay in {PSM_RULE}"
        )));
    }
    if !overlay_starts.is_empty() || marker_count > 0 {
        return Err(contract_error(format!(
            "incomplete or corrupt platform overlay in {PSM_RULE}"
        )));
    }

    let baseline_starts = window
        .iter()
        .enumerate()
        .filter_map(|(offset, line)| (line == PSM_BASELINE[0]).then_some(rule_start + offset))
        .collect::<Vec<_>>();
    let [baseline_start] = baseline_starts.as_slice() else {
        return Err(contract_error(format!(
            "unexpected generated preprocessor flags in {PSM_RULE}"
        )));
    };
    if !lines_match(lines, *baseline_start, &PSM_BASELINE) {
        return Err(contract_error(format!(
            "unexpected generated preprocessor flags in {PSM_RULE}"
        )));
    }

    lines.splice(
        *baseline_start..*baseline_start + PSM_BASELINE.len(),
        PSM_OVERLAY.iter().map(|line| (*line).to_owned()),
    );
    *patches_applied += 1;
    Ok(())
}

fn unique_rule_start(lines: &[String], target: &str) -> Result<usize, ThirdPartyOverlayError> {
    let needle = format!("    name = \"{target}\",");
    let matches = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (line == &needle).then_some(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(*index),
        _ => Err(contract_error(format!(
            "expected exactly one generated rule {target}, found {}",
            matches.len()
        ))),
    }
}

fn rule_end(lines: &[String], start: usize) -> Result<usize, ThirdPartyOverlayError> {
    lines[start..]
        .iter()
        .position(|line| line == ")")
        .map(|offset| start + offset)
        .ok_or_else(|| contract_error("unterminated generated rule"))
}

fn lines_match(lines: &[String], start: usize, expected: &[&str]) -> bool {
    lines
        .get(start..start + expected.len())
        .is_some_and(|actual| {
            actual
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())
        })
}

fn contract_error(message: impl Into<String>) -> ThirdPartyOverlayError {
    ThirdPartyOverlayError::Contract(message.into())
}

#[cfg(test)]
mod tests {
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
        let first =
            apply_third_party_buck_overlay(&unpatched_fixture()).expect("apply exact overlay");
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
}
