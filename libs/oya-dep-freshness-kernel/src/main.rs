//! `oya-dep-freshness-producer` (data-gen, OPERATIONAL — not the hermetic gate).
//!
//! Reads a local checkout of crates.io sparse-index files and distills them into the vendored
//! snapshot the hermetic freshness gate consumes:
//!   <out-dir>/freshness.json          — the distilled [`CrateRelease`] records, sorted by name
//!   <out-dir>/freshness-manifest.json — { schema, source:{index,snapshot_date}, content_hash, crate_count }
//!
//! NO NETWORK, NO CLOCK, NO SUBPROCESS — the same contract as `oya-advisory-mirror-producer`, and
//! for the same reason. The snapshot date is passed as a CLI argument rather than read from the
//! system clock, so re-running the producer on the same checkout reproduces the same bytes. Whatever
//! fetches or refreshes the index checkout is a separate concern and never this binary: the moment a
//! producer reads a clock or a socket, its output stops being reproducible and the gate that consumes
//! it stops being cacheable.
//!
//! Each input file is one crate, named for that crate, containing newline-delimited JSON — one
//! object per published version. That is the canonical `index.crates.io` layout. Note that cargo's
//! own `~/.cargo/registry/index/*/.cache/` files are NUL-delimited with an etag header and are NOT
//! this format; convert them before use rather than pointing this at the cache.
//!
//! Usage:
//!   oya-dep-freshness-producer --index-dir <dir> --snapshot-date <iso-date> --out-dir <dir>
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{fs, io};

use oya_dep_freshness_kernel::{FRESHNESS_SCHEMA, canonical_hash, distill};

struct Args {
    index_dir: PathBuf,
    snapshot_date: String,
    out_dir: PathBuf,
}

fn usage() -> String {
    "usage: oya-dep-freshness-producer --index-dir <dir> --snapshot-date <YYYY-MM-DD> \
     --out-dir <dir>"
        .to_string()
}

fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1).collect()) {
        Ok(Some(args)) => match run(&args) {
            Ok(count) => {
                println!("distilled {count} crates into {}", args.out_dir.display());
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("dep-freshness producer failed: {message}");
                ExitCode::from(2)
            }
        },
        Ok(None) => {
            println!("{}", usage());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}\n{}", usage());
            ExitCode::from(2)
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<Option<Args>, String> {
    let (mut index_dir, mut snapshot_date, mut out_dir) = (None, None, None);
    let mut rest = args.into_iter();
    while let Some(flag) = rest.next() {
        match flag.as_str() {
            "-h" | "--help" => return Ok(None),
            "--index-dir" => index_dir = Some(PathBuf::from(value(&mut rest, &flag)?)),
            "--snapshot-date" => snapshot_date = Some(value(&mut rest, &flag)?),
            "--out-dir" => out_dir = Some(PathBuf::from(value(&mut rest, &flag)?)),
            other => return Err(format!("unrecognized argument {other}")),
        }
    }
    let snapshot_date = snapshot_date.ok_or("missing --snapshot-date")?;
    // Validated here rather than trusted: a malformed date would silently make every staleness
    // computation in the gate return None, which reads as "nothing is stale".
    if oya_dep_freshness_kernel::days_between(&snapshot_date, &snapshot_date).is_none() {
        return Err(format!(
            "--snapshot-date {snapshot_date} is not a YYYY-MM-DD calendar date"
        ));
    }
    Ok(Some(Args {
        index_dir: index_dir.ok_or("missing --index-dir")?,
        snapshot_date,
        out_dir: out_dir.ok_or("missing --out-dir")?,
    }))
}

fn value(rest: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    rest.next().ok_or_else(|| format!("{flag} needs a value"))
}

fn run(args: &Args) -> Result<usize, String> {
    let inputs = read_index_dir(&args.index_dir)
        .map_err(|e| format!("read {}: {e}", args.index_dir.display()))?;
    if inputs.is_empty() {
        // Fail rather than emit an empty mirror. An empty snapshot would make the gate report
        // every dependency as UNKNOWN, or — worse, if a future gate treats "no record" as "fine" —
        // report nothing at all. Refusing here keeps that ambiguity out of the committed data.
        return Err(format!(
            "no index files under {}; refusing to write an empty mirror",
            args.index_dir.display()
        ));
    }
    let releases = distill(&inputs);
    let manifest = format!(
        "{{\n  \"schema\": \"{}\",\n  \"source\": {{\n    \"index\": \"index.crates.io\",\n    \
         \"snapshot_date\": \"{}\"\n  }},\n  \"content_hash\": \"{}\",\n  \"crate_count\": {}\n}}\n",
        FRESHNESS_SCHEMA,
        args.snapshot_date,
        canonical_hash(&releases),
        releases.len()
    );
    let snapshot = serde_json::to_string_pretty(&releases)
        .map_err(|e| format!("serialize freshness.json: {e}"))?;

    fs::create_dir_all(&args.out_dir)
        .map_err(|e| format!("create {}: {e}", args.out_dir.display()))?;
    write(
        &args.out_dir.join("freshness.json"),
        &format!("{snapshot}\n"),
    )?;
    write(&args.out_dir.join("freshness-manifest.json"), &manifest)?;
    Ok(releases.len())
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Every regular file under `dir`, as `(file stem = crate name, contents)`.
///
/// Recursive because the canonical index is sharded by name prefix (`se/rd/serde`). Directory
/// entries are walked; anything unreadable is an error rather than a skip, so a permission problem
/// cannot quietly shrink the mirror.
fn read_index_dir(dir: &Path) -> io::Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                stack.push(path);
                continue;
            }
            // The index has no config file at leaf level, but a checkout may carry `config.json`
            // at its root; it is not a crate record and must not become one.
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with('.') || name == "config.json" {
                continue;
            }
            out.push((name.to_string(), fs::read_to_string(&path)?));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_malformed_snapshot_date_is_rejected_at_parse_time() {
        let args = |date: &str| {
            parse_args(vec![
                "--index-dir".into(),
                "i".into(),
                "--out-dir".into(),
                "o".into(),
                "--snapshot-date".into(),
                date.into(),
            ])
        };
        assert!(args("2026-08-17").is_ok());
        assert!(args("17-08-2026").is_err());
        assert!(args("2026-13-01").is_err());
        assert!(args("not-a-date").is_err());
    }

    #[test]
    fn missing_required_flags_fail_closed() {
        assert!(parse_args(vec!["--index-dir".into(), "i".into()]).is_err());
        assert!(parse_args(vec!["--snapshot-date".into(), "2026-08-17".into()]).is_err());
        assert!(parse_args(vec!["--nonsense".into()]).is_err());
    }

    #[test]
    fn help_is_not_an_error() {
        assert!(matches!(parse_args(vec!["--help".into()]), Ok(None)));
    }
}
