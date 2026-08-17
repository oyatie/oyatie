//! Embedded-asset hermeticity fixer (ADR-0545) — the automation-default surface.
//!
//! Founder directive 2026-06-11: "gate should prioritize automation where possible; automation
//! should be the default; enforcement is the extra layer." So this gate ships a `--fix` mode (the
//! default developer/agent path) that DERIVES and APPLIES the correct `mapped_srcs` entry for an
//! unmapped embedded asset, plus a `--check` mode that reports without writing. The blocking
//! `*-gate` rust_test is the backstop for what `--fix` cannot safely auto-derive — and the gate's
//! failure detail prints the exact `--fix` command to run. Precedent: the cloud-ci face-settle tool
//! (`--settle --commit` is the default; the freshness gate is the backstop).
//!
//! Modes:
//!   --check   (default) collect + report unmapped includes and the derivable fixes; exit 1 if any
//!             blocking unmapped include remains, 0 if clean.
//!   --fix     apply every auto-derivable mapped_srcs entry to its BUCK file in place; report any
//!             site that needs a manual fix (the backstop); exit 0 if all unmapped sites were fixed
//!             or none existed, 1 if manual fixes remain.
//!
//! Usage: oya-cloud-ci-embedded-asset-hermeticity-fixer [--check|--fix] [--repo-root <path>]
//!
//! ADR-0083 Tier-3: a thin CLI shell; the verdict + remediation logic live in the pure kernel.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_embedded_asset_hermeticity::{
    Remediation, apply_remediation, collect_observed, derive_all_remediations, evaluate_keyed,
};
use serde_json::Value;

const POLICY_REL: &str =
    "ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json";

enum Mode {
    Check,
    Fix,
}

fn main() -> ExitCode {
    let mut mode = Mode::Check;
    let mut repo_root: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => mode = Mode::Check,
            "--fix" => mode = Mode::Fix,
            "--repo-root" => {
                repo_root = args.next().map(PathBuf::from);
            }
            "--help" | "-h" => {
                eprintln!(
                    "oya-cloud-ci-embedded-asset-hermeticity-fixer [--check|--fix] [--repo-root <path>]"
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let root = match repo_root.or_else(discover_repo_root) {
        Some(r) => r,
        None => {
            eprintln!(
                "error: could not locate repo root (pass --repo-root); expected an ancestor with specs/root-hub-pointers.json"
            );
            return ExitCode::from(2);
        }
    };

    let policy: Value = match std::fs::read_to_string(root.join(POLICY_REL)) {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: parse policy {POLICY_REL}: {e}");
                return ExitCode::from(2);
            }
        },
        Err(e) => {
            eprintln!("error: read policy {POLICY_REL}: {e}");
            return ExitCode::from(2);
        }
    };

    let observed = match collect_observed(&root, &policy) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: collect: {e}");
            return ExitCode::from(2);
        }
    };

    let unmapped: Vec<&Value> = observed["sites"]
        .as_array()
        .map(|sites| sites.iter().filter(|s| s["status"] == "unmapped").collect())
        .unwrap_or_default();

    if unmapped.is_empty() {
        let findings = evaluate_keyed(&policy, &observed);
        let blocking = findings
            .iter()
            .filter(|f| {
                f.code == "embedded_asset_unmapped_include"
                    || f.code == "embedded_asset_policy_gate_id_mismatch"
            })
            .count();
        println!(
            "embedded-asset hermeticity: 0 unmapped includes ({blocking} blocking findings). Repo is hermetic."
        );
        return ExitCode::SUCCESS;
    }

    let remediations = derive_all_remediations(&root, &observed);
    match mode {
        Mode::Check => report_check(&unmapped, &remediations),
        Mode::Fix => {
            let patch_exit = apply_fixes(&root, &remediations);
            // After patching, re-collect and re-check. Any covering target that --fix could not
            // patch (refused, manual, comment-bearing) must appear as a persistent warning — the
            // state must never be silently hermetic when a covering target remains unmapped.
            // This closes the gate-GREEN/build-RED gap for the lib+unittest+bin shape (F3/F2).
            let observed2 = match collect_observed(&root, &policy) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("error: re-collect after --fix: {e}");
                    return patch_exit;
                }
            };
            let still_unmapped: Vec<&Value> = observed2["sites"]
                .as_array()
                .map(|sites| sites.iter().filter(|s| s["status"] == "unmapped").collect())
                .unwrap_or_default();
            if still_unmapped.is_empty() {
                println!(
                    "embedded-asset --fix: repo is now hermetic (all unmapped includes resolved)."
                );
                ExitCode::SUCCESS
            } else {
                println!(
                    "\n[WARNING] {} unmapped include(s) remain after --fix (covering target(s) not patched — \
                     see [manual] lines above; these targets will fail to build hermetically):",
                    still_unmapped.len()
                );
                for s in &still_unmapped {
                    println!("  - {} :: {}", s["key"], s["detail"]);
                }
                println!(
                    "Re-run --check to see the full report, or fix the remaining targets by hand."
                );
                ExitCode::FAILURE
            }
        }
    }
}

fn report_check(unmapped: &[&Value], remediations: &[Remediation]) -> ExitCode {
    println!(
        "embedded-asset hermeticity: {} unmapped include(s) detected:",
        unmapped.len()
    );
    for site in unmapped {
        println!("  - {} :: {}", site["key"], site["detail"]);
    }
    let auto: Vec<&Remediation> = remediations.iter().filter(|r| r.applicable).collect();
    let manual: Vec<&Remediation> = remediations.iter().filter(|r| !r.applicable).collect();
    println!(
        "\n{} auto-fixable, {} need manual handling.",
        auto.len(),
        manual.len()
    );
    for r in &auto {
        println!("  [auto] {} :: {}", r.buck_path, r.note);
    }
    for r in &manual {
        println!("  [manual] {} :: {}", r.buck_path, r.note);
    }
    println!(
        "\nRun the auto-fixer:\n  buck2 run //ci/facade/embedded-asset-hermeticity:oya-cloud-ci-embedded-asset-hermeticity-fixer -- --fix"
    );
    ExitCode::FAILURE
}

fn apply_fixes(root: &Path, remediations: &[Remediation]) -> ExitCode {
    let mut applied = 0usize;
    let mut manual = 0usize;
    for rem in remediations {
        if !rem.applicable {
            manual += 1;
            println!("[manual] {} :: {}", rem.buck_path, rem.note);
            continue;
        }
        let buck_abs = root.join(&rem.buck_path);
        let text = match std::fs::read_to_string(&buck_abs) {
            Ok(t) => t,
            Err(e) => {
                println!("[skip] read {}: {e}", rem.buck_path);
                manual += 1;
                continue;
            }
        };
        match apply_remediation(&text, rem) {
            Ok(patched) => {
                if patched == text {
                    println!("[noop] {} already maps {}", rem.buck_path, rem.mapped_value);
                    continue;
                }
                if let Err(e) = std::fs::write(&buck_abs, patched) {
                    println!("[error] write {}: {e}", rem.buck_path);
                    manual += 1;
                    continue;
                }
                applied += 1;
                println!(
                    "[fixed] {} :: mapped_srcs[{}] = \"{}\" in target `{}`",
                    rem.buck_path, rem.mapped_key, rem.mapped_value, rem.target
                );
            }
            Err(e) => {
                manual += 1;
                println!("[manual] {} :: {e}", rem.buck_path);
            }
        }
    }
    println!(
        "\nembedded-asset --fix: {applied} BUCK file(s) patched, {manual} need manual handling."
    );
    if manual == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn discover_repo_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..32 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
