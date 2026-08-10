//! cloud-ci-hub-exclusivity live producer (ADR-0711 / Swarm Delivery Law).
//!
//! Loads hub authority from `specs/integ-branch-envelopes.json#hubs.paths` (cite pointer;
//! never re-list), consumes optional hermetic open-PR changed-file fixture facts, and runs the
//! pure [`ci_affected_target_set::hub_exclusivity`] evaluator. Multi-own → mechanical REFUSE
//! (exit 1).
//!
//! Wired into the binding `oya-ci-required` affected-set admission path so the required
//! context evaluates concurrent integ PR hub ownership when the admission adapter provides those
//! facts. The hermetic Buck2 binary deliberately skips live discovery when no fixture is supplied.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_affected_target_set::hub_exclusivity::{
    DEFAULT_POLICY_RELPATH, ENVELOPES_RELPATH, GATE_ID, HUBS_PATHS_POINTER, Verdict,
    evaluate_from_producer_docs, open_pr_facts_from_json,
};
use serde_json::Value;

const LOG: &str = "hub-exclusivity";

struct Args {
    policy_path: PathBuf,
    envelopes_path: PathBuf,
    open_prs_fixture: Option<PathBuf>,
}

fn parse_args(mut argv: std::env::Args) -> Result<Args, String> {
    let _bin = argv.next();
    let mut repo_root = PathBuf::from(".");
    let mut policy_path = None;
    let mut envelopes_path = None;
    let mut open_prs_fixture = None;
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(argv.next().ok_or("--repo-root needs a value")?)
            }
            "--policy" => {
                policy_path = Some(PathBuf::from(argv.next().ok_or("--policy needs a value")?))
            }
            "--envelopes" => {
                envelopes_path = Some(PathBuf::from(
                    argv.next().ok_or("--envelopes needs a value")?,
                ))
            }
            "--open-prs-fixture" => {
                open_prs_fixture = Some(PathBuf::from(
                    argv.next().ok_or("--open-prs-fixture needs a value")?,
                ))
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(Args {
        policy_path: policy_path.unwrap_or_else(|| repo_root.join(DEFAULT_POLICY_RELPATH)),
        envelopes_path: envelopes_path.unwrap_or_else(|| repo_root.join(ENVELOPES_RELPATH)),
        open_prs_fixture,
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("cannot parse `{}`: {e}", path.display()))
}

fn load_open_pr_facts(fixture: Option<&Path>) -> Result<Value, String> {
    match fixture {
        Some(path) => read_json(path),
        None => Ok(Value::Array(Vec::new())),
    }
}

fn run(args: &Args) -> ExitCode {
    if !args.envelopes_path.is_file() {
        println!(
            "{LOG}: SKIP — `{}` absent; hub authority pointer {HUBS_PATHS_POINTER} not yet on tip",
            args.envelopes_path.display()
        );
        return ExitCode::SUCCESS;
    }

    let policy_doc = match read_json(&args.policy_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{LOG}: POLICY ERROR: {e}");
            return ExitCode::from(2);
        }
    };
    let envelopes_doc = match read_json(&args.envelopes_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{LOG}: ENVELOPES ERROR: {e}");
            return ExitCode::from(2);
        }
    };

    let open_prs_doc = match load_open_pr_facts(args.open_prs_fixture.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{LOG}: FIXTURE ERROR: {e}");
            return ExitCode::from(2);
        }
    };
    if args.open_prs_fixture.is_none() {
        eprintln!("{LOG}: SKIP — no hermetic open-PR fixture supplied");
    }

    if let Err(finding) = open_pr_facts_from_json(&open_prs_doc) {
        eprintln!(
            "{LOG}: REFUSE — {}: {} ({})",
            finding.code, finding.key, finding.detail
        );
        return ExitCode::from(1);
    }

    let report = evaluate_from_producer_docs(&policy_doc, &envelopes_doc, &open_prs_doc);
    match report.verdict {
        Verdict::Green => {
            println!(
                "{LOG}: GREEN — {GATE_ID}; authority={HUBS_PATHS_POINTER}; open_prs={}",
                open_prs_doc.as_array().map(Vec::len).unwrap_or(0)
            );
            ExitCode::SUCCESS
        }
        Verdict::Refuse => {
            eprintln!("{LOG}: REFUSE — {GATE_ID} findings:");
            for f in &report.findings {
                eprintln!("{LOG}:   [{}] {}: {}", f.code, f.key, f.detail);
            }
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{LOG}: ARGS ERROR: {e}");
            eprintln!(
                "{LOG}: usage: oya-cloud-ci-hub-exclusivity [--repo-root <path>] [--policy <pack.json>] \
                 [--envelopes <envelopes.json>] [--open-prs-fixture <facts.json>]"
            );
            return ExitCode::from(2);
        }
    };
    run(&args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fetch_shape_round_trips_through_open_pr_parser() {
        let emitted = json!([
            {
                "number": 1643,
                "head_ref_name": "integ/os",
                "files": [{ "filename": "Cargo.lock" }]
            },
            {
                "number": 1647,
                "head_ref_name": "integ/build",
                "files": [{ "filename": "Cargo.lock" }, { "filename": "build/x.rs" }]
            }
        ]);
        let facts = open_pr_facts_from_json(&emitted).expect("parse emitted shape");
        assert_eq!(facts.len(), 2);
        assert!(facts[0].files.contains("Cargo.lock"));
        assert!(facts[1].files.contains("build/x.rs"));
    }

    #[test]
    fn absent_fixture_skips_live_open_pr_discovery() {
        assert_eq!(load_open_pr_facts(None).expect("skip live discovery"), json!([]));
    }
}
