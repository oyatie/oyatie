//! cloud-ci-capability-membership gate binary (Phase-0 capability-first reorg; ADR-0562 §6).
//! The MEMBERSHIP lint (the anti-junk-drawer authority): maps EVERY crate in the tree to exactly
//! one registered capability/meta home, fails any NEW unmapped crate or NEW top-level dir outside
//! the closed set, and enforces the base/-admission rule. Born ADVISORY against the frozen unmapped
//! baseline (no regression); flips to BLOCKING when the baseline reaches 0. The blocking buck2
//! `rust_test` gate is the backstop; this binary is the runnable detector.
//!
//! `--emit-legacy-freeze` is the PRODUCER for the policy's `legacy_root_freeze` block: it prints the
//! block for the CURRENT tree, derived from this gate's own `collect`, so the frozen legacy-root
//! census is never hand-typed and can never drift from the corpus the gate walks. It prints; it does
//! NOT write the policy — growth goes through review, the same rule the sibling baseline producers
//! follow. It also REFUSES outright to emit a census that GREW relative to the committed one unless
//! `--allow-new` is passed, because a producer that silently absorbs a newly-born legacy-root crate
//! would defeat the entire freeze.
//!
//! Usage:
//!   oya-cloud-ci-capability-membership-app-bin [--repo-root <path>] [--policy <path>]
//!                                             [--emit-legacy-freeze [--allow-new]]
//!
//! Exit codes: 0 = green (no findings); 1 = red findings remain; 2 = argument or collection error,
//! or an `--emit-legacy-freeze` blocked by census growth (fail-closed).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_module_membership::{
    LEGACY_ROOT_FREEZE_KEY, Verdict, collect, evaluate, evaluate_keyed, legacy_root_census,
    render_findings,
};
use serde_json::{Value, json};

const DEFAULT_POLICY: &str = "ci/facade/module-membership/capability-membership-policy.json";

struct Args {
    repo_root: PathBuf,
    policy: Option<PathBuf>,
    emit_legacy_freeze: bool,
    allow_new: bool,
}

enum ParseOutcome {
    Run(Args),
    Help,
    Error(String),
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        ParseOutcome::Run(args) => args,
        ParseOutcome::Help => {
            println!("{}", usage());
            return ExitCode::SUCCESS;
        }
        ParseOutcome::Error(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("capability-membership gate failed to run: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let policy = load_policy(&args.repo_root, args.policy.as_deref())?;
    let observed = collect(&args.repo_root, &policy).map_err(|e| e.to_string())?;
    if args.emit_legacy_freeze {
        return emit_legacy_freeze(&policy, &observed, args.allow_new);
    }
    let report = evaluate(&policy, &observed);
    let findings = evaluate_keyed(&policy, &observed);
    println!("{}", render_findings(&findings));
    println!(
        "capability-membership: {} crate(s) checked; {} mapped to a home; {} in the frozen unmapped baseline (burn-down target 0); {} still under a FROZEN legacy root (shrink-only, burn-down target 0)",
        report.crates_checked,
        report.mapped_to_home,
        report.frozen_unmapped,
        report.legacy_root_crates
    );
    if report.verdict == Verdict::Green {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

/// PRODUCER for the policy's `legacy_root_freeze` block. Renders the census from this gate's own
/// [`collect`] so it cannot drift from the corpus the gate walks, and REFUSES to emit a census that
/// grew relative to the committed one unless `allow_new` — a regen must never be able to launder a
/// newly-born legacy-root crate into the tolerated set. Prints; never writes the policy.
fn emit_legacy_freeze(
    policy: &Value,
    observed: &Value,
    allow_new: bool,
) -> Result<ExitCode, String> {
    let crates: Vec<String> = observed
        .get("crates")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    let Some(freeze) = policy
        .get(LEGACY_ROOT_FREEZE_KEY)
        .and_then(Value::as_object)
    else {
        return Err(format!(
            "--emit-legacy-freeze: the policy carries no `{LEGACY_ROOT_FREEZE_KEY}` block; the block \
             declares `frozen_roots` (the DATA this producer renders against) and must exist first"
        ));
    };
    let census = legacy_root_census(policy, &crates);
    if census.is_empty() {
        return Err(format!(
            "--emit-legacy-freeze: rendered an EMPTY census over {} collected crate(s); \
             `{LEGACY_ROOT_FREEZE_KEY}.frozen_roots` is empty or matches nothing, and an empty \
             census would freeze nothing while reading as success",
            crates.len()
        ));
    }

    let prior: Vec<String> = freeze
        .get("crates")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let growth: Vec<&String> = census.iter().filter(|c| !prior.contains(c)).collect();
    if !growth.is_empty() && !allow_new {
        eprintln!(
            "--emit-legacy-freeze: REFUSING to grow the legacy-root census. {} crate(s) under a \
             FROZEN legacy root are NOT in the committed census — they were BORN there, which is \
             exactly the accrual this freeze exists to stop:",
            growth.len()
        );
        for dir in &growth {
            eprintln!("  {dir}");
        }
        eprintln!(
            "Move each crate to its capability root, or (for a reviewed, deliberate exception) \
             re-run with --allow-new so the growth lands as a visible policy diff."
        );
        return Ok(ExitCode::from(2));
    }

    // `_provenance` is carried forward UNTOUCHED. It anchors the commit the freeze was
    // established at, which burn-down does not move — the same fixed-anchor shape every sibling
    // policy's `frozen_at`/`frozen_at_ref` uses. An earlier revision also rewrote a
    // `_provenance.crates_total` here, which made the block assert its own contradiction after the
    // first burn-down: `frozen_at` stayed at the freeze commit while the count tracked the CURRENT
    // tree, so it read as "at commit 96da99d1 there were 425 crates" when there were 445. The
    // count was read by nothing and is `crates.len()` anyway, so it is gone rather than repaired.
    let mut block = freeze.clone();
    block.insert("crates".to_owned(), json!(census));
    let rendered = serde_json::to_string_pretty(&json!({ LEGACY_ROOT_FREEZE_KEY: block }))
        .map_err(|e| format!("--emit-legacy-freeze: serialize: {e}"))?;
    println!("{rendered}");
    eprintln!(
        "--emit-legacy-freeze: {} crate(s) under {} frozen legacy root(s); committed census had {} \
         ({} burned down{})",
        census.len(),
        freeze
            .get("frozen_roots")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        prior.len(),
        prior.iter().filter(|p| !census.contains(p)).count(),
        if growth.is_empty() {
            String::new()
        } else {
            format!(", {} GROWN under --allow-new", growth.len())
        }
    );
    Ok(ExitCode::SUCCESS)
}

fn load_policy(repo_root: &Path, policy: Option<&Path>) -> Result<Value, String> {
    let path = match policy {
        Some(path) => path.to_path_buf(),
        None => repo_root.join(DEFAULT_POLICY),
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("read policy {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse policy {}: {e}", path.display()))
}

fn parse_args(args: Vec<String>) -> ParseOutcome {
    let mut repo_root = PathBuf::from(".");
    let mut policy = None;
    let mut emit_legacy_freeze = false;
    let mut allow_new = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "capability-membership: --repo-root requires a path".to_owned(),
                    );
                };
                repo_root = PathBuf::from(value);
            }
            "--policy" => {
                let Some(value) = iter.next() else {
                    return ParseOutcome::Error(
                        "capability-membership: --policy requires a path".to_owned(),
                    );
                };
                policy = Some(PathBuf::from(value));
            }
            "--emit-legacy-freeze" => emit_legacy_freeze = true,
            "--allow-new" => allow_new = true,
            "--help" | "-h" => return ParseOutcome::Help,
            other => {
                return ParseOutcome::Error(format!(
                    "capability-membership: unknown argument {other:?}; {}",
                    usage()
                ));
            }
        }
    }
    if allow_new && !emit_legacy_freeze {
        return ParseOutcome::Error(
            "capability-membership: --allow-new is only meaningful with --emit-legacy-freeze"
                .to_owned(),
        );
    }
    ParseOutcome::Run(Args {
        repo_root,
        policy,
        emit_legacy_freeze,
        allow_new,
    })
}

fn usage() -> String {
    "usage: oya-cloud-ci-capability-membership-app-bin [--repo-root <path>] [--policy <path>] \
     [--emit-legacy-freeze [--allow-new]]"
        .to_owned()
}
