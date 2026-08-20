// The contract-slice-policy fixer: regenerates the GENERATED aggregate
// `contract-slice-policy.json` from the committed `slices/*.json` fragments.
// Mirrors the repo's `check_equals_fix` generated-face doctrine (ADR-0539) — the
// gate (the byte-parity test) and this one-command fixer share the exact same
// `aggregate_policy`/`render_policy_json` functions, so the fixer can never
// disagree with the gate. Refuses (nonzero exit, no write) on a fail-closed
// fragment finding rather than silently materializing a partial policy.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use ci_contract_slice_conformance::{aggregate_policy, load_slice_fragments, render_policy_json};

fn main() -> ExitCode {
    let mut repo_root = PathBuf::from(".");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = args.next() else {
                    eprintln!("--repo-root requires a path");
                    return ExitCode::from(2);
                };
                repo_root = PathBuf::from(value);
            }
            other => {
                eprintln!(
                    "unknown argument {other:?}; usage: oya-cloud-ci-materialize-contract-slice-policy [--repo-root <path>]"
                );
                return ExitCode::from(2);
            }
        }
    }

    let gate_dir = repo_root.join("ci/facade/contract-slice-conformance");
    let load = load_slice_fragments(&gate_dir.join("slices"));
    if !load.findings.is_empty() {
        eprintln!("refusing to materialize contract-slice-policy.json: fragment findings present:");
        for finding in &load.findings {
            eprintln!("  - {} {}", finding.code, finding.key);
        }
        return ExitCode::FAILURE;
    }

    let policy = aggregate_policy(&load);
    let rendered = render_policy_json(&policy);
    let out_path = gate_dir.join("contract-slice-policy.json");
    if let Err(error) = std::fs::write(&out_path, rendered) {
        eprintln!("write {}: {error}", out_path.display());
        return ExitCode::FAILURE;
    }
    println!("materialized {}", out_path.display());
    ExitCode::SUCCESS
}
