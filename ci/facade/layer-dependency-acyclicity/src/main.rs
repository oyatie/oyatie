//! cloud-ci-tier-dependency-acyclicity gate binary (Phase-0 capability-first reorg; ADR-0245/0280/0562).
//!
//! Collects the live crate/dependency/tier corpus (cargo path-deps + buck deps, projected through
//! each service's `manifest.json` tier facet), loads the policy + frozen baseline, evaluates the
//! ADR-0245 tier rules + the ADR-0280 S-rank rule + a Tarjan cycle backstop, prints the report, and
//! exits 0 (GREEN) / 1 (RED, a regression vs the frozen baseline) / 2 (parse/io error). LOCAL BRIDGE
//! feedback only (founder CLI-retirement directive): merge authority lives in the buck2 gate test
//! behind oya-ci-required, never in this binary.
//!
//! `--emit-baseline` re-freezes the baseline: it prints the current live violation set in the
//! baseline document shape (canonical key order) to stdout, for re-snapshotting the known-debt after
//! a deliberate policy/rule change. It never writes files.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use ci_layer_dependency_acyclicity::{
    BASELINE_PATH, Baseline, POLICY_PATH, Status, Verdict, collect_corpus, evaluate, load_json,
    render,
};
use serde_json::{Value, json};

struct Args {
    repo_root: PathBuf,
    policy_path: String,
    baseline_path: String,
    emit_baseline: bool,
}

const USAGE: &str = "usage: oya-cloud-ci-tier-dependency-acyclicity [--repo-root <path>] \
[--policy <path>] [--baseline <path>] [--emit-baseline]";

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

    let policy = match load_json(&args.repo_root, &args.policy_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("tier-dependency-acyclicity gate policy error: {e}");
            return ExitCode::from(2);
        }
    };
    let baseline = match load_json(&args.repo_root, &args.baseline_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("tier-dependency-acyclicity gate baseline error: {e}");
            return ExitCode::from(2);
        }
    };
    let observed = match collect_corpus(&args.repo_root, &policy) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("tier-dependency-acyclicity gate collection error: {e}");
            return ExitCode::from(2);
        }
    };

    let report = evaluate(&policy, &baseline, &observed);

    if args.emit_baseline {
        // Print the CURRENT live violation set as a baseline document (the re-freeze surface).
        // Exclude the diagnostic codes (policy/baseline-malformed + the B3 stale-baseline phantom
        // finding): a re-emit re-freezes the LIVE violation set, so a phantom row must be dropped, not
        // carried forward.
        let mut violations: Vec<Value> = report
            .findings
            .iter()
            .filter(|f| {
                // The NON-BASELINEABLE codes: `evaluate` never consults the baseline for these,
                // so a row here is permanently inert. Worse than useless — once the condition is
                // fixed the row is absent from `live`, so it inflates `burned_down` forever, and
                // if its subject crate later moves, TDA-STALE-BASELINE fires on a row that should
                // never have existed. Excluding them makes the documented re-freeze remedy safe.
                f.code != "TDA-POLICY-MALFORMED"
                    && f.code != "TDA-BASELINE-MALFORMED"
                    && f.code != "TDA-STALE-BASELINE"
                    && f.code != "TDA-UNDECLARED-ROOT"
            })
            .map(|f| json!({ "code": f.code, "subject": f.subject }))
            .collect();
        violations.sort_by(|a, b| {
            let ka = Baseline::key_of(
                a.get("code").and_then(Value::as_str).unwrap_or(""),
                a.get("subject").and_then(Value::as_str).unwrap_or(""),
            );
            let kb = Baseline::key_of(
                b.get("code").and_then(Value::as_str).unwrap_or(""),
                b.get("subject").and_then(Value::as_str).unwrap_or(""),
            );
            ka.cmp(&kb)
        });
        let doc = json!({
            "gate_id": "cloud-ci-tier-dependency-acyclicity",
            "burn_down_target": 0,
            "violations": violations,
        });
        match serde_json::to_string_pretty(&doc) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("tier-dependency-acyclicity gate emit error: {e}");
                return ExitCode::from(2);
            }
        }
        return ExitCode::SUCCESS;
    }

    println!("{}", render(&report));
    let _ = Status::Baselined; // status enum is part of the report contract.
    match report.verdict {
        Verdict::Green => ExitCode::SUCCESS,
        Verdict::Red => ExitCode::FAILURE,
    }
}

/// Parse argv. `Ok(None)` => `--help` (print usage, exit 0); `Ok(Some(_))` runnable; `Err` usage
/// error (exit 2).
fn parse_args(args: Vec<String>) -> Result<Option<Args>, String> {
    let mut repo_root = PathBuf::from(".");
    let mut policy_path = POLICY_PATH.to_owned();
    let mut baseline_path = BASELINE_PATH.to_owned();
    let mut emit_baseline = false;
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
            "--baseline" => {
                baseline_path = iter
                    .next()
                    .ok_or_else(|| format!("--baseline requires a path; {USAGE}"))?;
            }
            "--emit-baseline" => emit_baseline = true,
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument {other:?}; {USAGE}")),
        }
    }
    Ok(Some(Args {
        repo_root,
        policy_path,
        baseline_path,
        emit_baseline,
    }))
}
