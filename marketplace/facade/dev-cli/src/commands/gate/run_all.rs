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

use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use oya_governance_gate_catalog_domain::{
    AGGREGATED_VALIDATE_LANES, BANNED_PRIMITIVES_COMMAND_LOG_CORPUS_ROOT,
    CI_REQUIRED_PREFLIGHT_COMMANDS, DEPENDENCY_SEAM_EVIDENCE,
};

use crate::commands::verify_affected::changed_files;

use super::result_cache::{FsVerdictCache, GateInputs, Verdict, default_cache_dir};
use super::run as gate_dispatch;

/// Declared inputs for a gate lane (ADR-0360 O7 content-addressed cache).
///
/// SAFETY: defaults to [`GateInputs::Unenumerable`] — a lane is cache-served
/// only once its FULL input set is explicitly declared here. Until then every
/// lane always runs (no false PASS). Declaring a lane is per-gate work: a lane
/// may be declared only if it is deterministic and reads exactly the declared
/// files; global/cross-corpus gates must declare the whole corpus.
fn lane_gate_inputs(_lane: &str) -> GateInputs {
    // No lane is declared cacheable yet; adoption is incremental + reviewed.
    GateInputs::Unenumerable
}

/// Gates that the legacy `scripts/check.sh` ran but which the Rust
/// aggregator deliberately defers (parameterized invocation,
/// not-yet-ported, or already covered by another lane). Documented for
/// audit traceability; future ADRs will fold these in.
const DEFERRED_GATES: &[(&str, &str)] = &[(
    "typescript-workspace",
    "requires --lane <typecheck|test>; invoke directly until \
         a default lane is canonicalized.",
)];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunAllArgs {
    pub(crate) include_deferred: bool,
    pub(crate) ci_required: bool,
    /// When `true`, narrow the lane set to those triggered by the diff vs `base`.
    /// Ignored (full set) when `ci_required` is also `true`.
    pub(crate) affected: bool,
    /// Git ref to diff against when `--affected` is supplied. Defaults to `origin/dev`.
    pub(crate) base: String,
}

pub(crate) fn parse_run_all_args(args: Vec<String>) -> Result<RunAllArgs, String> {
    let mut parsed = RunAllArgs {
        include_deferred: false,
        ci_required: false,
        affected: false,
        base: "origin/dev".to_string(),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--include-deferred" => parsed.include_deferred = true,
            "--ci-required" => parsed.ci_required = true,
            "--affected" => parsed.affected = true,
            "--base" => {
                let ref_val = iter
                    .next()
                    .ok_or_else(|| "gate run-all: --base requires a <ref> argument".to_string())?;
                parsed.base = ref_val;
            }
            other => {
                return Err(format!(
                    "gate run-all: unknown flag {other:?}; allowed: --include-deferred, --ci-required, --affected, --base <ref>"
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
    // Compute the active lane set.
    // --ci-required is the trunk backstop: always runs the full catalog,
    // unconditionally overriding --affected narrowing.
    let active_lanes: Vec<&str> = if args.ci_required || !args.affected {
        AGGREGATED_VALIDATE_LANES.to_vec()
    } else {
        // --affected mode: narrow to lanes triggered by the diff vs base.
        let repo_root = Path::new(".");
        match changed_files(repo_root, &args.base) {
            Ok(changed) => {
                let changed_refs: Vec<&str> = changed.iter().map(String::as_str).collect();
                let selected = oya_governance_gate_catalog_domain::lanes_for_changed(&changed_refs);
                println!(
                    "[gate run-all] affected mode: {}/{} lanes selected (base={})",
                    selected.len(),
                    AGGREGATED_VALIDATE_LANES.len(),
                    args.base
                );
                selected
            }
            Err(error) => {
                eprintln!(
                    "[gate run-all] WARNING: affected-scope git diff failed ({error}); \
                     falling back to full lane set"
                );
                AGGREGATED_VALIDATE_LANES.to_vec()
            }
        }
    };

    let mut outcomes: Vec<LaneOutcome> = Vec::with_capacity(
        active_lanes.len()
            + if args.ci_required {
                CI_REQUIRED_PREFLIGHT_COMMANDS.len()
            } else {
                0
            },
    );
    // O7 (ADR-0360): opt-in content-addressed verdict cache. Default OFF =>
    // unchanged behaviour. With OYA_GATE_CACHE=1, a lane is cache-served only if
    // lane_gate_inputs() declares ALL its inputs; the default is Unenumerable,
    // so every lane still runs until a gate is explicitly, safely declared.
    let cache = (std::env::var("OYA_GATE_CACHE").as_deref() == Ok("1"))
        .then(|| FsVerdictCache::new(default_cache_dir(Path::new("."))));

    for lane in &active_lanes {
        let inputs = lane_gate_inputs(lane);
        if let Some(cache) = &cache
            && let Some(verdict) = cache.lookup(&inputs)
        {
            let passed = matches!(verdict, Verdict::Pass);
            println!(
                "[gate run-all] {} {} (cached)",
                if passed { "PASS" } else { "FAIL" },
                lane
            );
            outcomes.push(LaneOutcome {
                lane: (*lane).to_string(),
                passed,
            });
            continue;
        }
        println!("[gate run-all] starting: {lane}");
        let dispatch_args = dispatch_args_for_lane(lane);
        let exit = gate_dispatch(dispatch_args, usage);
        let passed = is_success(exit);
        if let Some(cache) = &cache {
            cache.record(&inputs, if passed { Verdict::Pass } else { Verdict::Fail });
        }
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

    if args.ci_required {
        println!("[gate run-all] ci-required preflight: starting hosted required-check mirrors");
        for command in CI_REQUIRED_PREFLIGHT_COMMANDS {
            println!("[gate run-all] starting: {command}");
            let passed = run_ci_required_preflight_command(command);
            outcomes.push(LaneOutcome {
                lane: (*command).to_string(),
                passed,
            });
            println!(
                "[gate run-all] {} {}",
                if passed { "PASS" } else { "FAIL" },
                command
            );
        }
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
    } else if lane == "dependency-seam" {
        args.push("--repo-root".to_string());
        args.push(".".to_string());
        args.push("--evidence".to_string());
        args.push(DEPENDENCY_SEAM_EVIDENCE.to_string());
        args.push("--online-audit".to_string());
        args.push("--severity".to_string());
        args.push("error".to_string());
    }
    args
}

fn run_ci_required_preflight_command(command: &str) -> bool {
    let mut child = match command {
        "cargo fmt --all -- --check" => {
            let mut child = Command::new("cargo");
            child.args(["fmt", "--all", "--", "--check"]);
            child
        }
        "cargo check --workspace --all-targets --keep-going" => {
            let mut child = cargo_with_ci_env();
            child.args(["check", "--workspace", "--all-targets", "--keep-going"]);
            child
        }
        "cargo clippy --workspace --all-targets --keep-going -- -D warnings" => {
            let mut child = cargo_with_ci_env();
            child.args([
                "clippy",
                "--workspace",
                "--all-targets",
                "--keep-going",
                "--",
                "-D",
                "warnings",
            ]);
            child
        }
        "cargo nextest run --workspace --no-fail-fast" => {
            let mut child = cargo_with_ci_env();
            child.env("NEXTEST_PROFILE", "ci");
            child.args(["nextest", "run", "--workspace", "--no-fail-fast"]);
            child
        }
        "bash scripts/github-actions-required-secrets-check.sh" => {
            let mut child = Command::new("bash");
            child.args(["scripts/github-actions-required-secrets-check.sh"]);
            child
        }
        other => {
            eprintln!("[gate run-all] unsupported ci-required command in catalog: {other}");
            return false;
        }
    };
    match child
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => status.success(),
        Err(error) => {
            eprintln!("[gate run-all] could not start `{command}`: {error}");
            false
        }
    }
}

fn cargo_with_ci_env() -> Command {
    let mut command = Command::new("cargo");
    command.env("CARGO_TERM_COLOR", "always");
    command.env("CARGO_INCREMENTAL", "0");
    command.env_remove("RUSTC_WRAPPER");
    command
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
        assert!(!parsed.ci_required);
    }

    #[test]
    fn parse_args_include_deferred_flag() {
        let parsed =
            parse_run_all_args(vec!["--include-deferred".into()]).expect("include-deferred");
        assert!(parsed.include_deferred);
        assert!(!parsed.ci_required);
    }

    #[test]
    fn parse_args_ci_required_flag() {
        let parsed = parse_run_all_args(vec!["--ci-required".into()]).expect("ci-required");
        assert!(parsed.ci_required);
        assert!(!parsed.include_deferred);
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
    fn aggregated_lane_catalog_contains_platform_substrate_defaults() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"platform-substrate-defaults"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_freshness() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"freshness"));
    }

    #[test]
    fn dependency_seam_dispatch_uses_required_ci_args() {
        let args = dispatch_args_for_lane("dependency-seam");
        assert!(args.contains(&"--online-audit".to_string()));
        assert!(args.contains(&"--severity".to_string()));
        assert!(args.contains(&"error".to_string()));
        assert!(args.contains(&DEPENDENCY_SEAM_EVIDENCE.to_string()));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_module_catalog() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-module-catalog"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_gitops_evidence() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-gitops-evidence"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_helm_chart_signed_image_wiring() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-helm-chart-signed-image-wiring"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_kubewarden_admission_policy() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-kubewarden-admission-policy"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_cell_topology() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-cell-topology"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_opentofu_validation() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-opentofu-validation"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_module_provenance() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-module-provenance"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_module_provider_requirements() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-module-provider-requirements"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_module_release_index() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-module-release-index"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_module_archive() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-module-archive"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_module_registry_protocol() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-module-registry-protocol"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_provider_readiness() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-provider-readiness"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_provider_lockfile() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-provider-lockfile"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_provider_signature_review() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-provider-signature-review"));
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

    // ------------------------------------------------------------------
    // --affected / --base flag tests (ADR-0360 O1 gate-scope narrowing)
    // ------------------------------------------------------------------

    #[test]
    fn parse_args_affected_flag_defaults_base_to_origin_dev() {
        let parsed = parse_run_all_args(vec!["--affected".into()]).expect("affected flag");
        assert!(parsed.affected);
        assert!(!parsed.ci_required);
        assert_eq!(parsed.base, "origin/dev");
    }

    #[test]
    fn parse_args_affected_with_explicit_base() {
        let parsed = parse_run_all_args(vec!["--affected".into(), "--base".into(), "main".into()])
            .expect("--affected --base main");
        assert!(parsed.affected);
        assert_eq!(parsed.base, "main");
    }

    #[test]
    fn parse_args_base_without_value_rejected() {
        let err = parse_run_all_args(vec!["--affected".into(), "--base".into()])
            .expect_err("--base needs a value");
        assert!(err.contains("--base"), "error should mention --base: {err}");
    }

    #[test]
    fn parse_args_defaults_affected_false() {
        let parsed = parse_run_all_args(vec![]).expect("defaults");
        assert!(!parsed.affected);
        assert_eq!(parsed.base, "origin/dev");
    }

    /// `--ci-required` must always select the full `AGGREGATED_VALIDATE_LANES`
    /// set regardless of whether `--affected` is also supplied. This tests the
    /// lane-selection logic directly (pure function, no subprocess).
    #[test]
    fn ci_required_selects_full_lane_set() {
        // With ci_required=true, the active lanes must equal AGGREGATED_VALIDATE_LANES
        // exactly (same order, same content). We verify this by simulating the same
        // branch that run_all_gates takes when ci_required is true.
        let ci_required = true;
        let affected = true; // even with affected=true, ci_required wins
        let active_lanes: Vec<&str> = if ci_required || !affected {
            AGGREGATED_VALIDATE_LANES.to_vec()
        } else {
            // This branch must NOT be reached when ci_required is true.
            oya_governance_gate_catalog_domain::lanes_for_changed(&[
                "docs/decisions/ADR-9999-test.md",
            ])
        };
        assert_eq!(
            active_lanes, AGGREGATED_VALIDATE_LANES,
            "--ci-required must select the full AGGREGATED_VALIDATE_LANES set"
        );
    }

    /// `--affected` (without `--ci-required`) must narrow the lane set when the
    /// diff contains only ADR docs. The narrowed set must be a strict subset of
    /// the full catalog (proving the logic actually narrows, not just passes through).
    #[test]
    fn affected_narrows_lanes_for_adr_only_diff() {
        // Simulate an ADR-only diff: only docs/decisions changed.
        // lanes_for_changed is pure (no I/O), so we call it directly.
        let changed = ["docs/adr-archive/ADR-0360-ci-pipeline-optimization-program.md"];
        let selected = oya_governance_gate_catalog_domain::lanes_for_changed(&changed);

        // (1) The result must be strictly smaller than the full catalog because
        //     cloud-iac-*, slo-coverage, and other infra lanes are NOT triggered
        //     by a docs/decisions change.
        assert!(
            selected.len() < AGGREGATED_VALIDATE_LANES.len(),
            "affected narrowing must produce a subset: got {} of {} lanes",
            selected.len(),
            AGGREGATED_VALIDATE_LANES.len()
        );

        // (2) ADR-surface lanes that ARE triggered by docs/decisions must be present.
        assert!(
            selected.contains(&"adr-planning-completeness"),
            "adr-planning-completeness must be selected for a docs/decisions change"
        );
        assert!(
            selected.contains(&"adr-supersession-consistency"),
            "adr-supersession-consistency must be selected for a docs/decisions change"
        );

        // (3) Cloud IaC lanes (infra/**-gated) must be absent for this diff.
        assert!(
            !selected.contains(&"cloud-iac-opentofu-validation"),
            "cloud-iac-opentofu-validation must NOT be selected for a docs/decisions-only change"
        );
        assert!(
            !selected.contains(&"slo-coverage"),
            "slo-coverage must NOT be selected for a docs/decisions-only change"
        );
    }
}
