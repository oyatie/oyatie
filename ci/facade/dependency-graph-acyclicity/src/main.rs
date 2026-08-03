//! cloud-ci substrate graph-v2 gate binary (ADR-0280 §D-3, amended by ADR-0635).
//!
//! Loads the policy-declared substrate dependency DAG, runs the full coherence evaluation
//! (closed runtime-face-aware shape + graph-3 Tarjan acyclicity + forbidden-edge honouring + valid Kahn
//! order + exact max-min reverse failure closure), prints the report, and exits 0/1/2. LOCAL BRIDGE
//! feedback only (founder CLI-retirement directive): merge authority lives in the buck2 gate test
//! behind oya-ci-required + `//ci/facade/dependency-graph-acyclicity:ci-dependency-graph-acyclicity-gate`, never in this binary.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use ci_dependency_graph_acyclicity::{
    DEFAULT_POLICY_PATH, Verdict, evaluate_with_raw, load_dag, load_json, load_policy,
    render_findings,
};

struct Args {
    repo_root: PathBuf,
    policy_path: String,
    dag_path: Option<String>,
}

const USAGE: &str = "usage: oya-cloud-ci-substrate-dependency-dag-acyclicity \
    [--repo-root <path>] [--policy <path>] [--dag <path>]";

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

    let policy = match load_policy(&args.repo_root, &args.policy_path) {
        Ok(policy) => policy,
        Err(error) => {
            eprintln!("substrate-dependency-dag gate policy error: {error}");
            return ExitCode::from(2);
        }
    };
    let dag_path = args.dag_path.unwrap_or_else(|| policy.dag_path.clone());

    let schema = match load_json(&args.repo_root, &policy.schema_path) {
        Ok(schema) => schema,
        Err(error) => {
            eprintln!("substrate-dependency-dag schema error: {error}");
            return ExitCode::from(2);
        }
    };
    if schema.get("$schema").and_then(serde_json::Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        eprintln!("substrate-dependency-dag schema error: expected Draft 2020-12 schema");
        return ExitCode::from(2);
    }
    let capability_registry = match load_json(&args.repo_root, &policy.capability_registry_path) {
        Ok(registry) => registry,
        Err(error) => {
            eprintln!("substrate-dependency-dag capability registry error: {error}");
            return ExitCode::from(2);
        }
    };

    let dag = match load_dag(&args.repo_root, &dag_path) {
        Ok(dag) => dag,
        Err(error) => {
            eprintln!("substrate-dependency-dag gate parse error: {error}");
            return ExitCode::from(2);
        }
    };

    // Re-read the raw document for the field-completeness pass (the binary always runs the full
    // schema check, not just the graph-only projection).
    let full = args.repo_root.join(&dag_path);
    let raw_bytes = match std::fs::read_to_string(&full) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!(
                "substrate-dependency-dag gate io error: {}: {error}",
                full.display()
            );
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

    let report = evaluate_with_raw(&dag, &raw, &schema, &capability_registry, &policy);
    println!("{}", render_findings(&report.findings));
    if let Some(order) = &report.derived_bootstrap_order {
        println!(
            "derived graph-3 bootstrap_order (Kahn): {}",
            order.join(" -> ")
        );
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
    let mut policy_path = DEFAULT_POLICY_PATH.to_owned();
    let mut dag_path = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| format!("--repo-root requires a path; {USAGE}"))?,
                );
            }
            "--policy" => {
                policy_path = iter
                    .next()
                    .ok_or_else(|| format!("--policy requires a path; {USAGE}"))?;
            }
            "--dag" => {
                dag_path = Some(
                    iter.next()
                        .ok_or_else(|| format!("--dag requires a path; {USAGE}"))?,
                );
            }
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument {other:?}; {USAGE}")),
        }
    }
    Ok(Some(Args {
        repo_root,
        policy_path,
        dag_path,
    }))
}
