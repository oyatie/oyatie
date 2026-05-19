//! `oya gate run-all` — pre-merge gate aggregator. Replaces
//! `scripts/check.sh` per Wave 2 of the shell/python → Rust replacement
//! program (audit
//! `evidence/audits/shell-python-replacement-audit-2026-05-15.md`
//! row B-1).
//!
//! Naming justification: `gate run-all` is the kebab-case subcommand
//! that runs every `gate validate <name>` lane. The Rust handler
//! `run_all_gates` is snake_case (per canonical-naming kernel) and
//! lives in `src/commands/gate/run_all.rs` (snake_case module file
//! under the canonical `gate` subcommand directory; no redundant
//! `_aggregator` suffix). It is dispatched from `commands::gate::run`
//! via native function calls — no self-exec, no `Command::new("oya")`.
//!
//! Surface-all-failures semantics: each gate's handler is invoked with
//! its own default argument set; the resulting `ExitCode` is captured;
//! one failing gate does NOT short-circuit the rest. The aggregator
//! returns `ExitCode::FAILURE` iff any sub-gate failed.
//!
//! Gates not yet wired through this aggregator (because they require
//! repo-specific env vars or live in sibling crates) are listed in
//! `DEFERRED_GATES` for traceability. They were previously invoked from
//! `scripts/check.sh`; they remain runnable directly via
//! `oya gate validate <name>` until follow-up ADRs port them into the
//! native dispatcher.

use std::process::ExitCode;

use oya_foundry_gate_catalog_domain::{
    AGGREGATED_VALIDATE_LANES, BANNED_PRIMITIVES_COMMAND_LOG_CORPUS_ROOT,
};

use super::run as gate_dispatch;

/// Gates that the legacy `scripts/check.sh` ran but which the Rust
/// aggregator deliberately defers (parameterized invocation,
/// not-yet-ported, or already covered by another lane). Documented for
/// audit traceability; future ADRs will fold these in.
const DEFERRED_GATES: &[(&str, &str)] = &[
    (
        "typescript-workspace",
        "requires --lane <typecheck|test>; invoke directly until \
         a default lane is canonicalized.",
    ),
    (
        "release-supply-chain --phase pre-release",
        "phase argument required; invoke directly.",
    ),
    (
        "supply-chain --require-adr0039-evidence",
        "second supply-chain pass with adr-0039 flag; the aggregator \
         already invokes the default supply-chain lane above.",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunAllArgs {
    pub(crate) include_deferred: bool,
}

pub(crate) fn parse_run_all_args(args: Vec<String>) -> Result<RunAllArgs, String> {
    let mut parsed = RunAllArgs {
        include_deferred: false,
    };
    for flag in args {
        match flag.as_str() {
            "--include-deferred" => parsed.include_deferred = true,
            other => {
                return Err(format!(
                    "gate run-all: unknown flag {other:?}; allowed: --include-deferred"
                ));
            }
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LaneOutcome {
    pub(crate) lane: String,
    pub(crate) passed: bool,
}

pub(crate) fn run_all_gates(args: RunAllArgs, usage: &str) -> ExitCode {
    let mut outcomes: Vec<LaneOutcome> = Vec::with_capacity(AGGREGATED_VALIDATE_LANES.len());
    for lane in AGGREGATED_VALIDATE_LANES {
        println!("[gate run-all] starting: {lane}");
        let dispatch_args = dispatch_args_for_lane(lane);
        let exit = gate_dispatch(dispatch_args, usage);
        let passed = is_success(exit);
        outcomes.push(LaneOutcome {
            lane: (*lane).to_string(),
            passed,
        });
        println!(
            "[gate run-all] {} {}",
            if passed { "PASS" } else { "FAIL" },
            lane
        );
    }

    let failures: Vec<&LaneOutcome> = outcomes.iter().filter(|o| !o.passed).collect();
    println!(
        "\n[gate run-all] summary: {}/{} lanes passed",
        outcomes.len() - failures.len(),
        outcomes.len()
    );
    if !failures.is_empty() {
        println!("[gate run-all] failed lanes:");
        for outcome in &failures {
            println!("  - {}", outcome.lane);
        }
    }
    if args.include_deferred && !DEFERRED_GATES.is_empty() {
        println!("[gate run-all] deferred gates (run directly):");
        for (name, reason) in DEFERRED_GATES {
            println!("  - {name}: {reason}");
        }
    }

    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn dispatch_args_for_lane(lane: &str) -> Vec<String> {
    let mut args = vec!["validate".to_string(), lane.to_string()];
    if lane == "banned-primitives" {
        args.push("--require-command-log-corpus".to_string());
        args.push("--command-log-root".to_string());
        args.push(BANNED_PRIMITIVES_COMMAND_LOG_CORPUS_ROOT.to_string());
    }
    args
}

/// `ExitCode` is opaque (no `==` on Linux/macOS). Compare via a thin
/// wrapper that round-trips through `i32`. SUCCESS == 0.
fn is_success(code: ExitCode) -> bool {
    // The only stable way to inspect ExitCode is to format it; SUCCESS
    // prints as `ExitCode(unix_exit_status(0))` on Linux and just `0` on
    // others. Use the Debug-stable property that SUCCESS != FAILURE.
    // Instead, we route via `into_raw` semantics by comparison with the
    // known constructor.
    //
    // Safer: shadow the dispatch path so each gate handler returns an
    // i32 internally. Here we use a Rust-native trick: ExitCode
    // implements `Termination`, and `SUCCESS` is the only value that
    // reports through `Termination::report()` as `ExitCode::SUCCESS`.
    // Since std doesn't expose equality, we rely on the
    // documented-by-codepath invariant that every gate handler returns
    // exactly one of `SUCCESS`, `FAILURE`, or `from(2)`. We treat
    // anything that prints `0` as success via the debug formatter.
    let formatted = format!("{code:?}");
    // Debug output: `ExitCode(unix_exit_status(0))` on unix,
    // `ExitCode(ExitCode(0))` on windows. Both contain "(0)".
    formatted.contains("(0)")
}

#[cfg(test)]
pub(crate) fn deferred_gate_count() -> usize {
    DEFERRED_GATES.len()
}

#[cfg(test)]
pub(crate) fn aggregated_lane_count() -> usize {
    AGGREGATED_VALIDATE_LANES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_defaults() {
        let parsed = parse_run_all_args(vec![]).expect("defaults");
        assert!(!parsed.include_deferred);
    }

    #[test]
    fn parse_args_include_deferred_flag() {
        let parsed =
            parse_run_all_args(vec!["--include-deferred".into()]).expect("include-deferred");
        assert!(parsed.include_deferred);
    }

    #[test]
    fn parse_args_unknown_flag_rejected() {
        let error = parse_run_all_args(vec!["--bogus".into()]).expect_err("unknown flag");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_architecture_boundaries() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"architecture-boundaries"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_adr_citation() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"adr-citation"));
    }

    #[test]
    fn deferred_gates_documented() {
        assert!(deferred_gate_count() > 0);
        for (name, reason) in DEFERRED_GATES {
            assert!(!name.is_empty(), "deferred gate name must be non-empty");
            assert!(
                !reason.is_empty(),
                "deferred gate {name} must carry a reason"
            );
        }
    }

    #[test]
    fn aggregated_lane_count_nontrivial() {
        assert!(aggregated_lane_count() >= 30);
    }

    #[test]
    fn is_success_recognizes_exit_code_success() {
        assert!(is_success(ExitCode::SUCCESS));
        assert!(!is_success(ExitCode::FAILURE));
        assert!(!is_success(ExitCode::from(2)));
    }
}
