//! cloud-ci-substrate-dependency-dag-acyclicity gate binary (ADR-0280 §D-3).
//!
//! Loads the canonical `specs/substrate-dependency-dag.json`, runs the full coherence evaluation
//! (Tarjan acyclicity + forbidden-edge honouring + Kahn topo-sort == bootstrap_order + schema
//! completeness), prints the report, and exits 0 (GREEN) / 1 (RED) / 2 (parse error). LOCAL BRIDGE
//! feedback only (founder CLI-retirement directive): merge authority lives in the buck2 gate test
//! behind oya-ci-required + the check-substrates lane, never in this binary.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use oya_cloud_ci_substrate_dependency_dag_acyclicity_app::{
    DAG_PATH, evaluate_with_raw, load_dag, render_findings, Verdict,
};

struct Args {
    repo_root: PathBuf,
    dag_path: String,
}

const USAGE: &str =
    "usage: oya-cloud-ci-substrate-dependency-dag-acyclicity [--repo-root <path>] [--dag <path>]";

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Some(args)) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    let dag = match load_dag(&args.repo_root, &args.dag_path) {
        Ok(dag) => dag,
        Err(error) => {
            eprintln!("substrate-dependency-dag gate parse error: {error}");
            return ExitCode::from(2);
        }
    };

    // Re-read the raw document for the field-completeness pass (the binary always runs the full
    // schema check, not just the graph-only projection).
    let full = args.repo_root.join(&args.dag_path);
    let raw_bytes = match std::fs::read_to_string(&full) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("substrate-dependency-dag gate io error: {}: {error}", full.display());
            return ExitCode::from(2);
        }
    };
    let raw: serde_json::Value = match serde_json::from_str(&raw_bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("substrate-dependency-dag gate parse error: {error}");
            return ExitCode::from(2);
        }
    };

    let report = evaluate_with_raw(&dag, &raw);
    println!("{}", render_findings(&report.findings));
    if let Some(order) = &report.derived_bootstrap_order {
        println!("derived bootstrap_order (Kahn topo-sort): {}", order.join(" -> "));
    }
    match report.verdict {
        Verdict::Green => ExitCode::SUCCESS,
        Verdict::Red => ExitCode::FAILURE,
    }
}

/// Parse argv. `Ok(None)` => `--help` (print usage, exit 0); `Ok(Some(_))` runnable; `Err` usage
/// error (exit 2).
fn parse_args(args: Vec<String>) -> Result<Option<Args>, String> {
    let mut repo_root = PathBuf::from(".");
    let mut dag_path = DAG_PATH.to_owned();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| format!("--repo-root requires a path; {USAGE}"))?,
                );
            }
            "--dag" => {
                dag_path = iter
                    .next()
                    .ok_or_else(|| format!("--dag requires a path; {USAGE}"))?;
            }
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument {other:?}; {USAGE}")),
        }
    }
    Ok(Some(Args { repo_root, dag_path }))
}
