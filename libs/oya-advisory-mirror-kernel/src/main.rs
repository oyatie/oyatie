//! oya-advisory-mirror-producer (data-gen, OPERATIONAL — not the hermetic gate).
//!
//! Reads a local `rustsec/advisory-db` checkout, distills every `crates/**/RUSTSEC-*.md` into
//! the vendored snapshot the hermetic `cloud-ci-supply-chain-audit` gate consumes:
//!   <out-dir>/advisories.json      — the distilled [`Advisory`] records (sorted by id)
//!   <out-dir>/mirror-manifest.json — { schema, source:{repo,commit,last_sync}, content_hash, advisory_count }
//!
//! No network, no clock, no subprocess: the pinned commit + last-sync date are passed as CLI
//! args (run `git rev-parse HEAD` / `git show -s --format=%cI` on the checkout by hand). The
//! producer is pure file I/O over a checkout you already pinned; the network/clock-bearing
//! refresh is a separate owned reconciler (deferred Slice D), never this binary.
//!
//! Usage:
//!   oya-advisory-mirror-producer --advisory-db <checkout> --repo <url> --commit <sha> \
//!       --last-sync <iso-date> --out-dir <dir>
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_advisory_mirror_kernel::{Advisory, MIRROR_SCHEMA, canonical_hash, distill};

struct Args {
    advisory_db: PathBuf,
    repo: String,
    commit: String,
    last_sync: String,
    out_dir: PathBuf,
}

fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1).collect()) {
        Ok(Some(args)) => match run(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("advisory-mirror producer failed: {message}");
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

fn run(args: &Args) -> Result<(), String> {
    let crates_dir = args.advisory_db.join("crates");
    let mut texts: Vec<String> = Vec::new();
    collect_advisory_texts(&crates_dir, &mut texts)?;
    if texts.is_empty() {
        return Err(format!(
            "no RUSTSEC-*.md advisories found under {}; is --advisory-db a rustsec/advisory-db checkout?",
            crates_dir.display()
        ));
    }

    let mut advisories: Vec<Advisory> = distill(&texts);
    advisories.sort_by(|a, b| a.id.cmp(&b.id));
    advisories.dedup_by(|a, b| a.id == b.id);

    let content_hash = canonical_hash(&advisories);
    let advisory_count = advisories.len();

    let advisories_json = serde_json::to_string_pretty(&advisories)
        .map_err(|e| format!("serialize advisories.json: {e}"))?;
    let manifest = serde_json::json!({
        "schema": MIRROR_SCHEMA,
        "source": {
            "repo": args.repo,
            "commit": args.commit,
            "last_sync": args.last_sync,
        },
        "content_hash": content_hash,
        "advisory_count": advisory_count,
    });
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("serialize mirror-manifest.json: {e}"))?;

    std::fs::create_dir_all(&args.out_dir)
        .map_err(|e| format!("create out-dir {}: {e}", args.out_dir.display()))?;
    write_file(&args.out_dir.join("advisories.json"), &advisories_json)?;
    write_file(&args.out_dir.join("mirror-manifest.json"), &manifest_json)?;

    println!(
        "advisory-mirror: vendored {advisory_count} advisories (content_hash {content_hash}) from {} @ {} to {}",
        args.repo,
        args.commit,
        args.out_dir.display()
    );
    Ok(())
}

/// Recursively collect every `RUSTSEC-*.md` advisory text under `dir` (read-only). The
/// advisory-db lays them out as `crates/<crate>/RUSTSEC-YYYY-NNNN.md`; symlinked top-level
/// `RUSTSEC-*.md` aliases are skipped via the `crates/` anchor (no double-counting).
fn collect_advisory_texts(dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("read dir {}: {e}", dir.display())),
    };
    for entry in entries {
        let entry = entry.map_err(|e| format!("read entry in {}: {e}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("file_type {}: {e}", path.display()))?;
        if file_type.is_dir() {
            collect_advisory_texts(&path, out)?;
        } else if is_advisory_file(&path) {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            out.push(text);
        }
    }
    Ok(())
}

fn is_advisory_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("md")
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("RUSTSEC-"))
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    let mut body = contents.to_owned();
    body.push('\n');
    std::fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn parse_args(raw: Vec<String>) -> Result<Option<Args>, String> {
    let mut advisory_db: Option<PathBuf> = None;
    let mut repo: Option<String> = None;
    let mut commit: Option<String> = None;
    let mut last_sync: Option<String> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut iter = raw.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--advisory-db" => advisory_db = Some(PathBuf::from(next_value(&mut iter, &arg)?)),
            "--repo" => repo = Some(next_value(&mut iter, &arg)?),
            "--commit" => commit = Some(next_value(&mut iter, &arg)?),
            "--last-sync" => last_sync = Some(next_value(&mut iter, &arg)?),
            "--out-dir" => out_dir = Some(PathBuf::from(next_value(&mut iter, &arg)?)),
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    Ok(Some(Args {
        advisory_db: advisory_db.ok_or("--advisory-db <checkout> is required")?,
        repo: repo.ok_or("--repo <url> is required")?,
        commit: commit.ok_or("--commit <sha> is required")?,
        last_sync: last_sync.ok_or("--last-sync <iso-date> is required")?,
        out_dir: out_dir.ok_or("--out-dir <dir> is required")?,
    }))
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn usage() -> String {
    "usage: oya-advisory-mirror-producer --advisory-db <checkout> --repo <url> --commit <sha> --last-sync <iso-date> --out-dir <dir>".to_owned()
}
