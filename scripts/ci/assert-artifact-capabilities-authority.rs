//! Artifact capability registry command-authority check.
//!
//! This is intentionally local/static: it verifies the checked-in registry no
//! longer advertises retired local Oya CLI or direct Cargo command authority.
//! It does not post statuses, mutate branch protection, or claim live
//! Kubernetes/Prow execution.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_PATH: &str = "registry/artifact-capabilities-registry.json";

const FORBIDDEN_AUTHORITY_TERMS: &[&str] = &[
    concat!("oya", " gate"),
    concat!("oya", " check"),
    concat!("oya", " verify"),
    concat!("oya", "-dev-cli"),
    concat!("target/debug", "/oya"),
    concat!("cargo", " check"),
    concat!("cargo", " test"),
    concat!("cargo", " clippy"),
    concat!("cargo", " nextest"),
    concat!("cargo", " deny"),
    concat!("cargo", " machete"),
];

const REQUIRED_BUCK2_COMMANDS: &[&str] = &[
    "buck2 build //:artifact-capabilities-authority-check --show-output",
    "buck2 build //oya/intelligence/crates/oya-intelligence-supervisor-kernel:oya-intelligence-supervisor-kernel --show-output",
    "buck2 build //oya/intelligence/crates/oya-intelligence-settings-template-kernel:oya-intelligence-settings-template-kernel --show-output",
    "buck2 build //:language-discipline-check --show-output",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: &'static str,
    pub registry: &'static str,
    pub forbidden_hit_count: usize,
    pub required_command_count: usize,
    pub failures: Vec<String>,
}

pub fn evaluate_text(text: &str) -> Evaluation {
    let mut failures = Vec::new();

    for term in FORBIDDEN_AUTHORITY_TERMS {
        let count = text.matches(term).count();
        if count > 0 {
            failures.push(format!(
                "retired artifact registry command authority `{term}` appears {count} time(s)"
            ));
        }
    }

    for command in REQUIRED_BUCK2_COMMANDS {
        if !text.contains(command) {
            failures.push(format!(
                "artifact registry missing required Buck2 command `{command}`"
            ));
        }
    }

    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" },
        registry: REGISTRY_PATH,
        forbidden_hit_count: FORBIDDEN_AUTHORITY_TERMS
            .iter()
            .map(|term| text.matches(term).count())
            .sum(),
        required_command_count: REQUIRED_BUCK2_COMMANDS.len(),
        failures,
    }
}

pub fn evaluate(root: &Path) -> Evaluation {
    let path = root.join(REGISTRY_PATH);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    evaluate_text(&text)
}

fn repo_root() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

fn render_json(evaluation: &Evaluation) -> String {
    let failures = evaluation
        .failures
        .iter()
        .map(|failure| format!("\"{}\"", escape_json(failure)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"verdict\":\"{}\",\"registry\":\"{}\",\"forbidden_hit_count\":{},\"required_command_count\":{},\"failures\":[{}]}}",
        evaluation.verdict,
        evaluation.registry,
        evaluation.forbidden_hit_count,
        evaluation.required_command_count,
        failures
    )
}

fn main() {
    let evaluation = evaluate(&repo_root());
    println!("{}", render_json(&evaluation));
    if !evaluation.failures.is_empty() {
        std::process::exit(1);
    }
}
