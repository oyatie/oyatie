//! Runner for the facade->core layering gate.
//!
//! `--emit-baseline` prints the frozen-baseline block for the CURRENT tree. It exists to author
//! the initial baseline and to shrink it after a repair — never to absorb a regression. Growth
//! must go through review, which is why the runner does not write the policy file itself.

use std::path::PathBuf;
use std::process::ExitCode;

use ci_facade_core_layering::{DECLARED_CODES, collect, evaluate_keyed};
use serde_json::{Value, json};

const POLICY: &str = include_str!("../facade-core-layering-policy.json");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let repo_root = args
        .iter()
        .position(|a| a == "--repo-root")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let emit_baseline = args.iter().any(|a| a == "--emit-baseline");

    let policy: Value = match serde_json::from_str(POLICY) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("facade-core-layering: policy is not valid JSON: {e}");
            return ExitCode::FAILURE;
        }
    };

    let observed = match collect(&repo_root, &policy) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("facade-core-layering: scan failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    if observed["facade_packages_scanned"].as_u64().unwrap_or(0) == 0 {
        eprintln!(
            "facade-core-layering: scanned ZERO facade packages under {} — refusing to report \
             green, an empty scan is indistinguishable from a clean tree",
            repo_root.display()
        );
        return ExitCode::FAILURE;
    }

    if emit_baseline {
        let mut out = serde_json::Map::new();
        for code in DECLARED_CODES {
            let keys: Vec<&str> = observed["violations"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .filter(|r| r["code"].as_str() == Some(code))
                .filter_map(|r| r["key"].as_str())
                .collect();
            out.insert((*code).to_owned(), json!(keys));
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "frozen_baseline": out }))
                .unwrap_or_else(|e| format!("<serialize failed: {e}>"))
        );
        return ExitCode::SUCCESS;
    }

    let findings = evaluate_keyed(&policy, &observed);
    if findings.is_empty() {
        println!(
            "facade-core-layering: GREEN ({} capabilities, {} facade packages scanned)",
            observed["capabilities_scanned"], observed["facade_packages_scanned"]
        );
        return ExitCode::SUCCESS;
    }

    eprintln!("facade-core-layering: {} finding(s)", findings.len());
    for f in &findings {
        eprintln!("  [{}] {} — {}", f.code, f.key, f.detail);
    }
    eprintln!(
        "\nADR-0562: a `facade` crate reaches its own capability's `core` only through `ports`.\n\
         Fix the edge, or — if this capability has no ports layer yet — introduce one.\n\
         The baseline is shrink-only; it must not grow to absorb a new violation."
    );
    ExitCode::FAILURE
}
