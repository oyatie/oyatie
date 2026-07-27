//! Git merge-driver entrypoint: `fixup-ledger-merge-driver %O %A %B`.
//!
//! Exit 0 = merged, canonical result written over `%A`. Exit 1 = the driver declines (the sides
//! disagree in a way it will not guess at). Exit 2 = unmodelled input, failed self-validation, or
//! I/O. On ANY nonzero exit `%A` is left byte-untouched, so git falls back to a normal conflict —
//! the driver never writes a partially-merged ledger.
#![forbid(unsafe_code)]

use std::process::ExitCode;

use ci_fixup_ledger_merge_driver::{MergeError, MergeErrorKind, merge_ledgers};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("fixup-ledger-merge-driver: {err}");
            match err.kind() {
                MergeErrorKind::Conflict => ExitCode::from(1),
                MergeErrorKind::Parse | MergeErrorKind::Validate => ExitCode::from(2),
            }
        }
    }
}

fn run() -> Result<(), MergeError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [base_path, current_path, other_path] = args.as_slice() else {
        return Err(MergeError::new(
            MergeErrorKind::Parse,
            "usage: fixup-ledger-merge-driver <base> <current> <other>",
        ));
    };

    let base = read(base_path)?;
    let current = read(current_path)?;
    let other = read(other_path)?;
    let merged = merge_ledgers(&base, &current, &other)?;
    write_atomic(current_path, &merged)
}

/// Replace `%A` atomically: write a sibling temp file, then rename over the target. A crash
/// mid-merge therefore leaves `%A` byte-untouched, so a partially-written ledger standing in the
/// working tree is structurally impossible rather than merely unlikely.
fn write_atomic(path: &str, contents: &str) -> Result<(), MergeError> {
    let target = std::path::Path::new(path);
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("merge-result");
    let temp = target.with_file_name(format!(".{name}.merge-tmp-{}", std::process::id()));
    std::fs::write(&temp, contents).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Validate,
            format!("failed to write merge temp file {}: {err}", temp.display()),
        )
    })?;
    std::fs::rename(&temp, target).map_err(|err| {
        let _ = std::fs::remove_file(&temp);
        MergeError::new(
            MergeErrorKind::Validate,
            format!("failed to move merged ledger into {path}: {err}"),
        )
    })
}

fn read(path: &str) -> Result<String, MergeError> {
    std::fs::read_to_string(path).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Validate,
            format!("failed to read ledger {path}: {err}"),
        )
    })
}
