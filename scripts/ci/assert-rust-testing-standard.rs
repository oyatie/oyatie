//! Rust testing-standard drift checker.
//!
//! local/static documentation-drift evidence only. This validates that the
//! testing standard preserves Buck2-native LLVM source-based coverage,
//! Tarpaulin non-authority, and dual Cargo+Buck2 local mutation boundaries. It
//! never runs coverage, runs mutation testing, posts statuses, mutates branch
//! protection, or claims P0.0 green / Phase-0 completion / production readiness.

use std::env;
use std::fs;
use std::path::Path;

const DEFAULT_DOC: &str = "docs/standards/testing.md";
const LIVE_FALSE_FLAGS: &[&str] = &[
    "coverage_runner_implemented",
    "mutation_lane_implemented",
    "live_required_context_execution_proven",
    "protected_branch_authority_proven",
    "status_mutation_performed",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchor {
    pub id: &'static str,
    pub tokens: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorResult {
    pub id: &'static str,
    pub present: bool,
    pub tokens: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub doc: String,
    pub authority_boundary: String,
    pub anchor_results: Vec<AnchorResult>,
    pub anchor_count: usize,
    pub anchors_present: usize,
    pub standard_contract_proven: bool,
    pub verdict: String,
    pub failures: Vec<String>,
}

const REQUIRED_ANCHORS: &[Anchor] = &[
    Anchor {
        id: "buck2_native_llvm_coverage_policy",
        tokens: &["Buck2-native LLVM source-based coverage"],
    },
    Anchor {
        id: "coverage_generated_through_buck2",
        tokens: &["Coverage is generated natively through Buck2, not Tarpaulin"],
    },
    Anchor {
        id: "tarpaulin_non_canonical",
        tokens: &["Tarpaulin is not the canonical coverage surface"],
    },
    Anchor {
        id: "tarpaulin_not_required_ci",
        tokens: &["MUST NOT be added as required CI/PR evidence"],
    },
    Anchor {
        id: "rustc_instrument_coverage",
        tokens: &["rustc -C instrument-coverage", "-C instrument-coverage"],
    },
    Anchor {
        id: "llvm_profile_file",
        tokens: &["LLVM_PROFILE_FILE"],
    },
    Anchor {
        id: "profraw_profiles",
        tokens: &[".profraw"],
    },
    Anchor {
        id: "llvm_profdata",
        tokens: &["llvm-profdata"],
    },
    Anchor {
        id: "llvm_cov",
        tokens: &["llvm-cov"],
    },
    Anchor {
        id: "buck2_build_id_evidence",
        tokens: &["Buck2 target", "Build ID", "report path"],
    },
    Anchor {
        id: "delta_and_generated_exclusions",
        tokens: &["changed-file delta", "excluded generated paths"],
    },
    Anchor {
        id: "dual_cargo_buck2_harness",
        tokens: &["dual Cargo+Buck2"],
    },
    Anchor {
        id: "cargo_manifests_retained",
        tokens: &["Cargo.toml", "Cargo.lock"],
    },
    Anchor {
        id: "cargo_mutants_local",
        tokens: &["cargo mutants"],
    },
    Anchor {
        id: "cargo_nextest_local",
        tokens: &["cargo nextest"],
    },
    Anchor {
        id: "local_cargo_mutation_advisory",
        tokens: &["Local Cargo mutation output is advisory"],
    },
    Anchor {
        id: "buck2_or_cloud_ci_mutation_capture",
        tokens: &[
            "Buck2 target or trusted cloud-ci/oya-ci lane",
            "captured the mutation run",
        ],
    },
    Anchor {
        id: "buck2_authority",
        tokens: &["Buck2 `BUCK` targets remain the build/test/CI authority"],
    },
    Anchor {
        id: "reindeer_generated_buck",
        tokens: &["reindeer-style generation", "generated-BUCK path"],
    },
    Anchor {
        id: "raw_cargo_not_authority",
        tokens: &["raw Cargo commands are not CI/build/test authority"],
    },
    Anchor {
        id: "buck2_show_output",
        tokens: &["buck2 test //... --show-output"],
    },
    Anchor {
        id: "trusted_cloud_ci_inventory",
        tokens: &["trusted cloud-ci/oya-ci Buck2 target inventory"],
    },
    Anchor {
        id: "anti_pattern_tarpaulin_authority",
        tokens: &["Adding Tarpaulin as the monorepo coverage authority"],
    },
    Anchor {
        id: "anti_pattern_local_cargo_merge_authority",
        tokens: &["Treating local Cargo mutation testing as merge authority"],
    },
    Anchor {
        id: "rustc_source",
        tokens: &["https://doc.rust-lang.org/rustc/instrument-coverage.html"],
    },
    Anchor {
        id: "llvm_source",
        tokens: &["https://clang.llvm.org/docs/SourceBasedCodeCoverage.html"],
    },
    Anchor {
        id: "buck2_commands_source",
        tokens: &["https://buck2.build/docs/users/commands/"],
    },
    Anchor {
        id: "buck2_bootstrapping_source",
        tokens: &["https://buck2.build/docs/about/bootstrapping/"],
    },
    Anchor {
        id: "cargo_workspace_source",
        tokens: &["https://doc.rust-lang.org/cargo/reference/workspaces.html"],
    },
    Anchor {
        id: "reindeer_source",
        tokens: &["https://github.com/facebookincubator/reindeer"],
    },
    Anchor {
        id: "cargo_mutants_source",
        tokens: &["https://mutants.rs/"],
    },
];

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

fn json_string(input: &str) -> String {
    format!("\"{}\"", json_escape(input))
}

fn json_string_array(values: &[&str]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn contains_all(text: &str, tokens: &[&str]) -> bool {
    tokens.iter().all(|token| text.contains(token))
}

pub fn anchor_results(text: &str) -> Vec<AnchorResult> {
    REQUIRED_ANCHORS
        .iter()
        .map(|anchor| AnchorResult {
            id: anchor.id,
            present: contains_all(text, anchor.tokens),
            tokens: anchor.tokens,
        })
        .collect()
}

fn missing_anchor_failures(results: &[AnchorResult]) -> Vec<String> {
    results
        .iter()
        .filter(|result| !result.present)
        .map(|result| format!("missing_{}", result.id))
        .collect()
}

fn tarpaulin_boundary_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    if !text.contains("Tarpaulin") {
        failures.push("missing_tarpaulin_boundary_subject".to_owned());
        return failures;
    }
    if !text.contains("Tarpaulin is not the canonical coverage surface") {
        failures.push("tarpaulin_canonicalized".to_owned());
    }
    if !text.contains("MUST NOT be added as required CI/PR evidence") {
        failures.push("tarpaulin_required_ci_boundary_missing".to_owned());
    }
    for (index, line) in text.lines().enumerate() {
        let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.contains("Tarpaulin is canonical")
            || normalized.contains("Tarpaulin is the canonical")
        {
            failures.push(format!("tarpaulin_canonical_claim_line_{}", index + 1));
        }
        if normalized.contains("Tarpaulin")
            && normalized.contains("required CI/PR evidence")
            && !normalized.contains("MUST NOT")
        {
            failures.push(format!("tarpaulin_required_ci_claim_line_{}", index + 1));
        }
    }
    failures
}

fn local_cargo_mutation_boundary_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    if !text.contains("Local Cargo mutation output is advisory") {
        failures.push("local_cargo_mutation_not_advisory".to_owned());
    }
    if !text.contains("captured by a Buck2 target or trusted cloud-ci/oya-ci lane") {
        failures.push("local_cargo_mutation_capture_boundary_missing".to_owned());
    }
    if !text.contains("Treating local Cargo mutation testing as merge authority") {
        failures.push("local_cargo_mutation_merge_authority_antipattern_missing".to_owned());
    }
    failures
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn has_forbidden_true_claim(text: &str, claim: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let claim = claim.to_ascii_lowercase();
    let mut offset = 0usize;
    while let Some(relative) = lower[offset..].find(&claim) {
        let start = offset + relative;
        let end = start + claim.len();
        let before_ok = lower[..start]
            .chars()
            .next_back()
            .map(|ch| !is_word_char(ch))
            .unwrap_or(true);
        let after_ok = lower[end..]
            .chars()
            .next()
            .map(|ch| !is_word_char(ch))
            .unwrap_or(true);
        if before_ok && after_ok {
            let rest = lower[end..].trim_start();
            if let Some(after_separator) = rest.strip_prefix(':').or_else(|| rest.strip_prefix('='))
            {
                if after_separator.trim_start().starts_with("true") {
                    return true;
                }
            }
        }
        offset = end;
    }
    false
}

fn claim_boundary_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for claim in LIVE_FALSE_FLAGS {
        if has_forbidden_true_claim(text, claim) {
            failures.push(format!("forbidden_true_claim_{claim}"));
        }
    }
    let lower = text.to_ascii_lowercase();
    for (label, phrase) in [
        ("p0_0_green_phrase", "p0.0 is green"),
        ("phase0_complete_phrase", "phase-0 is complete"),
        ("production_ready_phrase", "production-ready now"),
        ("hyperscaler_grade_phrase", "hyperscaler-grade now"),
    ] {
        if lower.contains(phrase) {
            failures.push(format!("forbidden_claim_{label}"));
        }
    }
    failures
}

pub fn evaluate_text(doc: &str, text: &str) -> Evaluation {
    let anchors = anchor_results(text);
    let mut failures = Vec::new();
    failures.extend(missing_anchor_failures(&anchors));
    failures.extend(tarpaulin_boundary_failures(text));
    failures.extend(local_cargo_mutation_boundary_failures(text));
    failures.extend(claim_boundary_failures(text));
    failures.sort();
    failures.dedup();
    let anchors_present = anchors.iter().filter(|anchor| anchor.present).count();
    Evaluation {
        doc: doc.to_owned(),
        authority_boundary: concat!(
            "local/static standards drift evidence only; this checker does not run coverage, ",
            "does not run mutation testing, never posts statuses, never mutates branch protection, ",
            "and cannot prove live Phase-0 exit authority"
        )
        .to_owned(),
        anchor_count: anchors.len(),
        anchors_present,
        standard_contract_proven: failures.is_empty(),
        verdict: if failures.is_empty() {
            "PASS".to_owned()
        } else {
            "FAIL".to_owned()
        },
        anchor_results: anchors,
        failures,
    }
}

pub fn evaluate_file(path: &Path) -> Evaluation {
    match fs::read_to_string(path) {
        Ok(text) => evaluate_text(&path.display().to_string(), &text),
        Err(error) => {
            let mut evaluation = evaluate_text(&path.display().to_string(), "");
            evaluation.verdict = "FAIL".to_owned();
            evaluation.standard_contract_proven = false;
            evaluation.failures.push(format!("read_failed_{error}"));
            evaluation
        }
    }
}

pub fn to_json(evaluation: &Evaluation) -> String {
    let anchor_results = evaluation
        .anchor_results
        .iter()
        .map(|result| {
            format!(
                "{{\"id\":{},\"present\":{},\"tokens\":{}}}",
                json_string(result.id),
                json_bool(result.present),
                json_string_array(result.tokens)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let failures = evaluation
        .failures
        .iter()
        .map(|failure| json_string(failure))
        .collect::<Vec<_>>()
        .join(",");
    let false_flags = LIVE_FALSE_FLAGS
        .iter()
        .map(|flag| format!("\"{flag}\":false"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"doc\":{},",
            "\"authority_boundary\":{},",
            "\"anchor_results\":[{}],",
            "\"anchor_count\":{},",
            "\"anchors_present\":{},",
            "\"standard_contract_proven\":{},",
            "{},",
            "\"verdict\":{},",
            "\"failures\":[{}]",
            "}}"
        ),
        json_string(&evaluation.doc),
        json_string(&evaluation.authority_boundary),
        anchor_results,
        evaluation.anchor_count,
        evaluation.anchors_present,
        json_bool(evaluation.standard_contract_proven),
        false_flags,
        json_string(&evaluation.verdict),
        failures
    )
}

fn print_usage(program: &str) {
    eprintln!("usage: {program} [--doc PATH] [--json]");
}

fn run() -> i32 {
    let mut doc = DEFAULT_DOC.to_owned();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--doc" => {
                let Some(value) = args.next() else {
                    print_usage("assert-rust-testing-standard");
                    return 2;
                };
                doc = value;
            }
            "--json" => json = true,
            "--help" | "-h" => {
                print_usage("assert-rust-testing-standard");
                return 0;
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_usage("assert-rust-testing-standard");
                return 2;
            }
        }
    }
    let evaluation = evaluate_file(Path::new(&doc));
    let rendered = to_json(&evaluation);
    if json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict == "PASS" { 0 } else { 1 }
}

#[cfg(not(test))]
fn main() {
    std::process::exit(run());
}
