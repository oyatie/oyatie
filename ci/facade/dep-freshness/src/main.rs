//! `oya-ci-dep-freshness-report` — the advisory reporter for the dependency-freshness gate.
//!
//! Prints the stale tail of the committed mirror and ALWAYS exits 0. That is the contract, not an
//! oversight: `oya-deps.toml` declares `enforcement = "advisory"`, and a reporter that sometimes
//! exits non-zero would become load-bearing the first time someone wired it into a required check.
//! If a blocking form is ever wanted it must be the change-driven one (a diff that ADDS or bumps TO
//! a stale dependency), which needs diff context this binary does not take.
//!
//! Usage:
//!   oya-ci-dep-freshness-report [--repo-root <dir>] [--json]
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

use ci_dep_freshness::{Policy, mirror, owner_index, snapshot_date, stale_entries};

const STEWARDSHIP_REGISTRY: &str = "specs/oss-stewardship-registry.json";
const DEPS_POLICY: &str = "oya-deps.toml";

fn main() -> ExitCode {
    let mut repo_root = PathBuf::from(".");
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => match args.next() {
                Some(value) => repo_root = PathBuf::from(value),
                None => {
                    eprintln!("--repo-root needs a value");
                    return ExitCode::from(2);
                }
            },
            "--json" => json = true,
            "-h" | "--help" => {
                println!("usage: oya-ci-dep-freshness-report [--repo-root <dir>] [--json]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unrecognized argument {other}");
                return ExitCode::from(2);
            }
        }
    }
    match report(&repo_root, json) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            // A reporter that cannot read its own inputs must say so loudly. This is the one
            // non-zero path, and it means "the gate did not run", never "the corpus is clean".
            eprintln!("dep-freshness report failed: {message}");
            ExitCode::from(2)
        }
    }
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);
    fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))
}

fn report(root: &Path, json: bool) -> Result<(), String> {
    let policy = Policy::from_toml(&read(root, DEPS_POLICY)?).map_err(|e| e.to_string())?;
    let manifest = read(root, &policy.manifest)?;
    let as_of = snapshot_date(&manifest)
        .ok_or("mirror manifest has no source.snapshot_date; cannot date the corpus")?;
    let releases = mirror(&read(root, &policy.mirror)?)?;
    // The registry is optional enrichment; its absence must not fail the report.
    let owners = read(root, STEWARDSHIP_REGISTRY)
        .map(|text| owner_index(&text))
        .unwrap_or_default();
    let entries = stale_entries(&releases, &policy, &as_of, &owners);

    if json {
        let rows: Vec<String> = entries
            .iter()
            .map(|e| {
                format!(
                    "{{\"name\":\"{}\",\"latest_stable\":\"{}\",\"last_release_date\":\"{}\",\
                     \"days_since_release\":{},\"owner_team\":{}}}",
                    e.name,
                    e.latest_stable,
                    e.last_release_date,
                    e.days_since_release,
                    e.owner_team
                        .as_ref()
                        .map_or("null".to_string(), |o| format!("\"{o}\""))
                )
            })
            .collect();
        println!(
            "{{\"code\":\"DEP-FRESHNESS-STALE\",\"enforcement\":\"{}\",\"stale_after_days\":{},\
             \"as_of\":\"{}\",\"corpus\":{},\"stale\":{},\"entries\":[{}]}}",
            policy.enforcement,
            policy.stale_after_days,
            as_of,
            releases.len(),
            entries.len(),
            rows.join(",")
        );
        return Ok(());
    }

    println!(
        "DEP-FRESHNESS-STALE (advisory) — {} of {} direct dependencies have had no stable release \
         in {} days, as of {}",
        entries.len(),
        releases.len(),
        policy.stale_after_days,
        as_of
    );
    if entries.is_empty() {
        return Ok(());
    }
    println!(
        "{:>7}  {:<28} {:<22} {:<12} owner",
        "days", "crate", "latest_stable", "released"
    );
    for entry in &entries {
        println!(
            "{:>7}  {:<28} {:<22} {:<12} {}",
            entry.days_since_release,
            entry.name,
            entry.latest_stable,
            entry.last_release_date,
            entry.owner_team.as_deref().unwrap_or("-")
        );
    }
    println!(
        "\nAdvisory only: staleness is time-driven, so this never blocks a merge. Staleness is also \
         not abandonment — a mature crate may simply be finished. Investigate the tail, not the count."
    );
    Ok(())
}
