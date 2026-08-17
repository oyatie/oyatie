//! owners-from-envelopes materializer / freshness check.
//!
//! Reads `specs/integ-branch-envelopes.json`, emits CODEOWNERS + OWNERS-by-prefix faces
//! under `ci/facade/affected-target-set/owners-from-envelopes/`.
//!
//! Modes:
//! - `--write` — materialize faces (local / hub land).
//! - `--check` — refuse when committed faces drift from generation (CI freshness).
//!
//! BAN Cargo.lock / live `.github/CODEOWNERS` flag-day replace in this binary.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_affected_target_set::owners_from_envelopes::{
    EMIT_CODEOWNERS_RELPATH, EMIT_OWNERS_MAP_RELPATH, ENVELOPES_RELPATH, generate_owners,
    owners_map_json,
};
use serde_json::Value;

const LOG: &str = "owners-from-envelopes";

struct Args {
    repo_root: PathBuf,
    envelopes_path: PathBuf,
    write: bool,
    check: bool,
}

fn parse_args(mut argv: std::env::Args) -> Result<Args, String> {
    let _bin = argv.next();
    let mut repo_root = PathBuf::from(".");
    let mut envelopes_path = None;
    let mut write = false;
    let mut check = false;
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(argv.next().ok_or("--repo-root needs a value")?)
            }
            "--envelopes" => {
                envelopes_path = Some(PathBuf::from(argv.next().ok_or("--envelopes needs a value")?))
            }
            "--write" => write = true,
            "--check" => check = true,
            "--help" | "-h" => {
                return Err(
                    "usage: oya-cloud-ci-owners-from-envelopes [--repo-root DIR] [--envelopes PATH] (--write|--check)"
                        .into(),
                )
            }
            other => return Err(format!("unknown arg {other}")),
        }
    }
    if write == check {
        return Err("exactly one of --write or --check is required".into());
    }
    let envelopes_path = envelopes_path.unwrap_or_else(|| repo_root.join(ENVELOPES_RELPATH));
    Ok(Args {
        repo_root,
        envelopes_path,
        write,
        check,
    })
}

fn load_envelopes(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: parse: {e}", path.display()))
}

fn materialize(repo_root: &Path, codeowners: &str, owners_map: &str) -> Result<(), String> {
    let codeowners_path = repo_root.join(EMIT_CODEOWNERS_RELPATH);
    let owners_map_path = repo_root.join(EMIT_OWNERS_MAP_RELPATH);
    if let Some(parent) = codeowners_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&codeowners_path, codeowners)
        .map_err(|e| format!("{}: {e}", codeowners_path.display()))?;
    fs::write(&owners_map_path, owners_map)
        .map_err(|e| format!("{}: {e}", owners_map_path.display()))?;
    eprintln!("{LOG}: wrote {EMIT_CODEOWNERS_RELPATH}");
    eprintln!("{LOG}: wrote {EMIT_OWNERS_MAP_RELPATH}");
    Ok(())
}

fn check_fresh(repo_root: &Path, codeowners: &str, owners_map: &str) -> Result<(), String> {
    let codeowners_path = repo_root.join(EMIT_CODEOWNERS_RELPATH);
    let owners_map_path = repo_root.join(EMIT_OWNERS_MAP_RELPATH);
    let on_disk_co = fs::read_to_string(&codeowners_path)
        .map_err(|e| format!("{}: {e} (run --write)", codeowners_path.display()))?;
    let on_disk_om = fs::read_to_string(&owners_map_path)
        .map_err(|e| format!("{}: {e} (run --write)", owners_map_path.display()))?;
    let mut drift = Vec::new();
    if on_disk_co != codeowners {
        drift.push(EMIT_CODEOWNERS_RELPATH);
    }
    if on_disk_om != owners_map {
        drift.push(EMIT_OWNERS_MAP_RELPATH);
    }
    if drift.is_empty() {
        eprintln!("{LOG}: freshness OK");
        Ok(())
    } else {
        Err(format!(
            "owners-from-envelopes drift: {} (regenerate with --write)",
            drift.join(", ")
        ))
    }
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{LOG}: {e}");
            return ExitCode::from(2);
        }
    };
    let envelopes = match load_envelopes(&args.envelopes_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{LOG}: {e}");
            return ExitCode::from(1);
        }
    };
    let generated = match generate_owners(&envelopes) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{LOG}: {e}");
            return ExitCode::from(1);
        }
    };
    let owners_map = match owners_map_json(&generated.owners_by_prefix) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{LOG}: {e}");
            return ExitCode::from(1);
        }
    };
    let result = if args.write {
        materialize(&args.repo_root, &generated.codeowners, &owners_map)
    } else {
        check_fresh(&args.repo_root, &generated.codeowners, &owners_map)
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{LOG}: {e}");
            ExitCode::from(1)
        }
    }
}
