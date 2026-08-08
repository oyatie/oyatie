//! cloud-ci-workflow-lane-preflight gate binary — the RUNNABLE detector.
//!
//! The kernel next door is pure and was, until this file existed, only ever reachable from
//! hand-written test fixtures. A gate with no caller cannot fail a real run, so its green is not
//! coverage: it is the absence of measurement wearing the costume of a passing check. This binary
//! is the entry point that lets a run actually be judged.
//!
//! It stays faithful to the kernel's split of responsibility: it does NOT shell out to git and does
//! NOT walk the tree looking for lanes. The CALLER runs `git rev-parse` / `git cat-file -e` /
//! `git check-ignore` and hands the ANSWERS in as data, because a gate that performed its own git
//! queries would be trusting its own walk in exactly the place failure 2 hid. The only I/O here is
//! reading two documents and writing one.
//!
//! Usage:
//!   ci-workflow-lane-preflight-bin [--policy <path>] [--lanes <path>]
//!
//! `--lanes` defaults to STDIN. The lanes document is a JSON array of lane declarations:
//!
//! ```json
//! [ { "lane": "reorg-a",
//!     "commit_sha": "1111111111111111111111111111111111111111",
//!     "commit_exists": true,
//!     "paths": [ { "path": "ci/facade/x/src/lib.rs", "git_visible": true } ] } ]
//! ```
//!
//! Exit codes: 0 = green; 1 = blocking findings (EVERY finding blocks — see the kernel docs for why
//! there is no advisory tier); 2 = argument, read or parse error. Note that 2 is also what an empty
//! or malformed lanes document earns: a run that cannot be read has not been shown to be disjoint,
//! and fail-closed is the whole point of the anti-vacuity floors.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use ci_workflow_lane_preflight::{GATE_ID, LaneDeclaration, Policy, evaluate};

const DEFAULT_POLICY: &str =
    "ci/facade/workflow-lane-preflight/workflow-lane-preflight-policy.json";

fn usage() -> String {
    format!(
        "{GATE_ID}\n\
         usage: ci-workflow-lane-preflight-bin [--policy <path>] [--lanes <path>]\n\
         \n\
         --policy  frozen policy JSON (default: {DEFAULT_POLICY})\n\
         --lanes   lane declarations as a JSON array (default: stdin)\n\
         \n\
         exit: 0 green, 1 blocking findings, 2 argument/read/parse error"
    )
}

struct Args {
    policy: PathBuf,
    lanes: Option<PathBuf>,
}

enum ParseOutcome {
    Run(Args),
    Help,
    Error(String),
}

fn parse_args(argv: Vec<String>) -> ParseOutcome {
    let mut policy = PathBuf::from(DEFAULT_POLICY);
    let mut lanes = None;
    let mut it = argv.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => return ParseOutcome::Help,
            "--policy" => match it.next() {
                Some(value) => policy = PathBuf::from(value),
                None => return ParseOutcome::Error("--policy needs a path".to_owned()),
            },
            "--lanes" => match it.next() {
                Some(value) => lanes = Some(PathBuf::from(value)),
                None => return ParseOutcome::Error("--lanes needs a path".to_owned()),
            },
            other => return ParseOutcome::Error(format!("unrecognized argument {other}")),
        }
    }
    ParseOutcome::Run(Args { policy, lanes })
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        ParseOutcome::Run(args) => args,
        ParseOutcome::Help => {
            println!("{}", usage());
            return ExitCode::SUCCESS;
        }
        ParseOutcome::Error(message) => {
            eprintln!("{message}\n\n{}", usage());
            return ExitCode::from(2);
        }
    };

    let policy_text = match std::fs::read_to_string(&args.policy) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("failed to read policy {}: {error}", args.policy.display());
            return ExitCode::from(2);
        }
    };
    let policy: Policy = match serde_json::from_str(&policy_text) {
        Ok(policy) => policy,
        Err(error) => {
            eprintln!("failed to parse policy {}: {error}", args.policy.display());
            return ExitCode::from(2);
        }
    };

    let lanes_text = match args.lanes {
        Some(ref path) => match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("failed to read lanes {}: {error}", path.display());
                return ExitCode::from(2);
            }
        },
        None => {
            let mut buffer = String::new();
            if let Err(error) = std::io::stdin().read_to_string(&mut buffer) {
                eprintln!("failed to read lanes from stdin: {error}");
                return ExitCode::from(2);
            }
            buffer
        }
    };
    let lanes: Vec<LaneDeclaration> = match serde_json::from_str(&lanes_text) {
        Ok(lanes) => lanes,
        Err(error) => {
            eprintln!("failed to parse lane declarations: {error}");
            return ExitCode::from(2);
        }
    };

    let verdict = evaluate(&lanes, &policy);
    match serde_json::to_string_pretty(&verdict) {
        Ok(rendered) => println!("{rendered}"),
        Err(error) => {
            eprintln!("failed to render verdict: {error}");
            return ExitCode::from(2);
        }
    }

    if verdict.failed() {
        eprintln!(
            "{GATE_ID}: {} blocking finding(s) across {} lane(s)",
            verdict.findings.len(),
            verdict.lanes
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_policy_path_is_the_committed_one() {
        // If this drifts, the binary silently judges a run against a policy that is not the frozen
        // one — the shape of failure this whole gate exists to make impossible.
        assert!(DEFAULT_POLICY.ends_with("workflow-lane-preflight-policy.json"));
    }

    #[test]
    fn args_default_to_the_frozen_policy_and_stdin() {
        match parse_args(Vec::new()) {
            ParseOutcome::Run(args) => {
                assert_eq!(args.policy, PathBuf::from(DEFAULT_POLICY));
                assert!(args.lanes.is_none(), "lanes must default to stdin");
            }
            _ => panic!("empty argv should run"),
        }
    }

    #[test]
    fn both_paths_are_overridable() {
        match parse_args(vec![
            "--policy".to_owned(),
            "p.json".to_owned(),
            "--lanes".to_owned(),
            "l.json".to_owned(),
        ]) {
            ParseOutcome::Run(args) => {
                assert_eq!(args.policy, PathBuf::from("p.json"));
                assert_eq!(args.lanes, Some(PathBuf::from("l.json")));
            }
            _ => panic!("both flags should parse"),
        }
    }

    #[test]
    fn a_flag_missing_its_value_is_an_error_not_a_silent_default() {
        // Falling back to the default policy here would judge the run against the wrong document
        // and still exit 0. Fail-closed on the argument, not just on the verdict.
        assert!(matches!(
            parse_args(vec!["--policy".to_owned()]),
            ParseOutcome::Error(_)
        ));
        assert!(matches!(
            parse_args(vec!["--lanes".to_owned()]),
            ParseOutcome::Error(_)
        ));
        assert!(matches!(
            parse_args(vec!["--nonsense".to_owned()]),
            ParseOutcome::Error(_)
        ));
    }

    #[test]
    fn a_lanes_document_round_trips_through_the_wire_format() {
        // The binary's contract with its caller IS this JSON shape; if it stops deserializing, the
        // gate is unreachable no matter how correct the kernel is.
        let lanes: Vec<LaneDeclaration> = serde_json::from_str(
            r#"[{"lane":"a","commit_sha":"aaa","commit_exists":true,
                 "paths":[{"path":"ci/a/x.rs","git_visible":true}]}]"#,
        )
        .expect("wire format parses");
        assert_eq!(lanes.len(), 1);
        assert_eq!(lanes[0].paths[0].path, "ci/a/x.rs");
    }
}
