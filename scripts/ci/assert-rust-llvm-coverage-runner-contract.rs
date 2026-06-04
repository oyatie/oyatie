//! Rust LLVM coverage-runner contract checker.
//!
//! Local/static contract evidence only: validates the Buck2-native LLVM
//! source-based coverage target shape, explicit Tarpaulin non-authority, and
//! non-claim boundary. It does not run coverage, post statuses, mutate branch
//! protection, or prove live required-context authority.

use std::env;
use std::fs;

const DEFAULT_SPEC: &str = "specs/rust-llvm-coverage-runner-contract.json";
const FALSE_CLAIMS: &[&str] = &[
    "coverage_report_generated",
    "coverage_budget_enforced",
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];
const REQUIRED_OFFICIAL_URLS: &[&str] = &[
    "https://doc.rust-lang.org/rustc/instrument-coverage.html",
    "https://clang.llvm.org/docs/SourceBasedCodeCoverage.html",
    "https://buck2.build/docs/users/commands/",
];
const REQUIRED_EVIDENCE_FIELDS: &[&str] = &[
    "Buck2 target",
    "Buck2 Build ID",
    "report path",
    "changed-file delta",
    "excluded generated paths",
    "coverage budget result",
];
const REQUIRED_TOOLCHAIN_TOOLS: &[&str] = &["rustc", "llvm-profdata", "llvm-cov"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn contains_json_string(text: &str, value: &str) -> bool {
    text.contains(&json_string(value))
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn has_string_value(text: &str, key: &str, value: &str) -> bool {
    let key_token = json_string(key);
    let value_token = json_string(value);
    let mut cursor = 0usize;
    while let Some(relative) = text[cursor..].find(&key_token) {
        cursor += relative + key_token.len();
        let mut rest = &text[cursor..];
        rest = rest.trim_start();
        let Some(after_colon) = rest.strip_prefix(':') else {
            continue;
        };
        if after_colon.trim_start().starts_with(&value_token) {
            return true;
        }
    }
    false
}

fn require(condition: bool, failures: &mut Vec<String>, failure: &str) {
    if !condition {
        failures.push(failure.to_owned());
    }
}

fn raw_cargo_command_present(text: &str) -> bool {
    for raw_line in text.lines() {
        let line = raw_line.to_ascii_lowercase();
        let mut index = 0usize;
        while let Some(pos) = line[index..].find("cargo") {
            let start = index + pos;
            let before = line[..start].chars().next_back();
            let after = start + "cargo".len();
            let before_ok = before
                .map(|ch| ch.is_whitespace() || matches!(ch, ';' | '&' | '|' | '(' | '`'))
                .unwrap_or(true);
            let after_ok = line[after..]
                .chars()
                .next()
                .map(|ch| ch.is_whitespace())
                .unwrap_or(false);
            if before_ok && after_ok {
                let mut cursor = after;
                while line[cursor..]
                    .chars()
                    .next()
                    .map(|ch| ch.is_whitespace())
                    .unwrap_or(false)
                {
                    cursor += line[cursor..].chars().next().unwrap().len_utf8();
                }
                let command_start = cursor;
                while cursor < line.len() {
                    let ch = line[cursor..].chars().next().unwrap();
                    if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
                        break;
                    }
                    cursor += ch.len_utf8();
                }
                let terminated = line[cursor..]
                    .chars()
                    .next()
                    .map(|ch| ch.is_whitespace())
                    .unwrap_or(true);
                if terminated && cursor > command_start {
                    return true;
                }
            }
            index = after;
        }
    }
    false
}

pub fn evaluate_text(spec: &str) -> Evaluation {
    let mut failures = Vec::new();

    require(
        has_bool(spec, "coverage_runner_contract_proven", true),
        &mut failures,
        "coverage_runner_contract_not_proven",
    );
    for claim in FALSE_CLAIMS {
        require(
            has_bool(spec, claim, false),
            &mut failures,
            &format!("forbidden_true_or_missing_claim_{claim}"),
        );
    }

    require(
        has_string_value(
            spec,
            "canonical_surface",
            "Buck2-native LLVM source-based coverage",
        ),
        &mut failures,
        "missing_buck2_native_llvm_canonical_surface",
    );
    require(
        spec.contains("Tarpaulin") && spec.contains("not required CI/PR"),
        &mut failures,
        "tarpaulin_noncanonical_boundary_missing",
    );
    require(
        has_string_value(
            spec,
            "buck2_contract_target",
            "//:rust-llvm-coverage-runner-contract-check",
        ),
        &mut failures,
        "wrong_buck2_contract_target",
    );
    require(
        has_string_value(
            spec,
            "future_runner_authority",
            "trusted cloud-ci/oya-ci Buck2 target inventory",
        ),
        &mut failures,
        "wrong_future_runner_authority",
    );

    require(
        has_bool(spec, "ambient_path_llvm_tools_required", false),
        &mut failures,
        "ambient_path_llvm_tools_not_forbidden",
    );
    require(
        spec.contains("rustup llvm-tools component"),
        &mut failures,
        "missing_rustup_llvm_tools_source",
    );
    require(
        spec.contains("trusted cloud-ci/oya-ci Buck2 toolchain inventory"),
        &mut failures,
        "missing_live_runner_toolchain_inventory_source",
    );
    require(
        spec.contains("rustc --print sysroot"),
        &mut failures,
        "missing_sysroot_tool_path_derivation",
    );
    require(
        spec.to_ascii_lowercase().contains("pin"),
        &mut failures,
        "missing_toolchain_pinning_requirement",
    );
    for tool in REQUIRED_TOOLCHAIN_TOOLS {
        require(
            contains_json_string(spec, tool),
            &mut failures,
            &format!("missing_required_toolchain_tool_{}", tool.replace('-', "_")),
        );
    }

    require(
        has_string_value(
            spec,
            "buck2_smoke_target",
            "//:rust-llvm-coverage-smoke-check",
        ),
        &mut failures,
        "missing_buck2_coverage_smoke_target",
    );
    require(
        has_string_value(
            spec,
            "smoke_script",
            "scripts/ci/run-rust-llvm-coverage-smoke.rs",
        ),
        &mut failures,
        "missing_coverage_smoke_script",
    );
    require(
        has_string_value(
            spec,
            "fixture",
            "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs",
        ),
        &mut failures,
        "missing_coverage_smoke_fixture",
    );
    require(
        has_bool(spec, "fixture_report_generated", true),
        &mut failures,
        "fixture_report_generation_not_recorded",
    );
    require(
        has_bool(spec, "production_coverage_report_generated", false),
        &mut failures,
        "production_coverage_false_boundary_missing",
    );
    require(
        spec.to_ascii_lowercase().contains("none"),
        &mut failures,
        "smoke_budget_authority_not_none",
    );

    for field in REQUIRED_EVIDENCE_FIELDS {
        require(
            contains_json_string(spec, field),
            &mut failures,
            &format!(
                "missing_required_evidence_field_{}",
                field
                    .replace(' ', "_")
                    .replace('-', "_")
                    .to_ascii_lowercase()
            ),
        );
    }

    require(
        has_string_value(spec, "rustc_flag", "rustc -C instrument-coverage"),
        &mut failures,
        "missing_instrument_coverage_flag",
    );
    require(
        has_string_value(spec, "buck2_rust_rule_field", "rustc_flags"),
        &mut failures,
        "missing_buck2_rustc_flags_field",
    );
    require(
        has_string_value(spec, "profile_env_var", "LLVM_PROFILE_FILE"),
        &mut failures,
        "missing_llvm_profile_file_env",
    );
    require(
        spec.contains("%m-%p") && spec.contains(".profraw"),
        &mut failures,
        "missing_profile_collision_guard_or_profraw_template",
    );
    require(
        has_string_value(spec, "profile_collision_guard", "%m-%p"),
        &mut failures,
        "missing_profile_collision_guard",
    );
    require(
        has_string_value(spec, "profile_extension", ".profraw"),
        &mut failures,
        "missing_profraw_extension",
    );

    require(
        has_string_value(spec, "tool", "llvm-profdata"),
        &mut failures,
        "missing_llvm_profdata_tool",
    );
    require(
        has_string_value(spec, "operation", "merge"),
        &mut failures,
        "missing_profdata_merge_operation",
    );
    require(
        has_string_value(spec, "mode", "-sparse"),
        &mut failures,
        "missing_sparse_profdata_merge_mode",
    );
    require(
        spec.contains("*.profraw"),
        &mut failures,
        "missing_profraw_merge_input_glob",
    );
    require(
        spec.contains(".profdata"),
        &mut failures,
        "missing_profdata_output",
    );

    require(
        has_string_value(spec, "tool", "llvm-cov"),
        &mut failures,
        "missing_llvm_cov_tool",
    );
    for fmt in ["text", "html", "json"] {
        require(
            contains_json_string(spec, fmt),
            &mut failures,
            &format!("missing_llvm_cov_{fmt}_format"),
        );
    }
    require(
        compact_json_text(spec).contains("\"changed_files_line_coverage_minimum\":80"),
        &mut failures,
        "wrong_changed_file_delta_budget",
    );
    require(
        compact_json_text(spec).contains("\"kernel_domain_absolute_line_coverage_minimum\":70"),
        &mut failures,
        "wrong_kernel_domain_budget",
    );
    require(
        has_bool(spec, "generated_code_excluded", true),
        &mut failures,
        "generated_code_exclusion_missing",
    );

    require(
        spec.contains("Tarpaulin as canonical monorepo coverage evidence"),
        &mut failures,
        "missing_tarpaulin_forbidden_authority",
    );
    require(
        spec.contains("candidate-authored target inventory as coverage authority"),
        &mut failures,
        "missing_candidate_inventory_forbidden_authority",
    );
    for url in REQUIRED_OFFICIAL_URLS {
        let suffix = if url.ends_with('/') {
            url.trim_end_matches('/').rsplit('/').next().unwrap_or(url)
        } else {
            url.rsplit('/').next().unwrap_or(url)
        };
        require(
            contains_json_string(spec, url),
            &mut failures,
            &format!("missing_official_reference_{suffix}"),
        );
    }
    require(
        !raw_cargo_command_present(spec),
        &mut failures,
        "raw_cargo_command_present_in_contract",
    );
    require(
        spec.contains("buck2 build //:rust-llvm-coverage-smoke-check"),
        &mut failures,
        "missing_smoke_target_in_automated_chain",
    );

    failures.sort();
    failures.dedup();
    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
    }
}

pub fn evaluate_file(path: &str) -> Evaluation {
    match fs::read_to_string(path) {
        Ok(text) => evaluate_text(&text),
        Err(error) => Evaluation {
            verdict: "FAIL".to_owned(),
            failures: vec![format!("spec_read_failed:{error}")],
        },
    }
}

fn render_json(evaluation: &Evaluation) -> String {
    let failures = evaluation
        .failures
        .iter()
        .map(|failure| json_string(failure))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"authority_boundary\":\"local/static coverage runner contract only; no coverage report generated and no live required-context authority proven\",\"coverage_budget_enforced\":false,\"coverage_report_generated\":false,\"coverage_runner_contract_proven\":{},\"failures\":[{}],\"hyperscaler_grade\":false,\"live_required_context_execution_proven\":false,\"p0_0_green\":false,\"phase0_complete\":false,\"production_ready\":false,\"protected_branch_authority_proven\":false,\"status_mutation_performed\":false,\"verdict\":{}}}",
        if evaluation.failures.is_empty() {
            "true"
        } else {
            "false"
        },
        failures,
        json_string(&evaluation.verdict),
    )
}

fn parse_args() -> (String, bool) {
    let mut spec = DEFAULT_SPEC.to_owned();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--spec" => spec = args.next().unwrap_or_else(|| spec.clone()),
            "--json" => json = true,
            _ => {}
        }
    }
    (spec, json)
}

fn main() {
    let (spec, json) = parse_args();
    let evaluation = evaluate_file(&spec);
    let rendered = render_json(&evaluation);
    if json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict != "PASS" {
        std::process::exit(1);
    }
}
