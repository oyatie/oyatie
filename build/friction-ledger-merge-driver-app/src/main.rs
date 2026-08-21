//! Git merge-driver entrypoint: `oya-friction-ledger-merge-driver %O %A %B`.
//!
//! Exit 0 = merged (canonical result written over `%A`). Exit 1 = the driver declines the merge
//! (e.g. base already corrupt). Exit 2 = unparseable/unmodeled input, I/O, usage, or a failed
//! D7 self-validation. On ANY nonzero exit `%A` is left untouched, so git falls back to a normal
//! conflict — the driver never writes garbage (FRIC-1781370000 incident 2).
#![forbid(unsafe_code)]

use std::process::ExitCode;

use friction_ledger_merge_driver_app::{MergeError, MergeErrorKind, merge_ledgers};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("oya-friction-ledger-merge-driver: {err}");
            match err.kind() {
                MergeErrorKind::Conflict => ExitCode::from(1),
                MergeErrorKind::Parse
                | MergeErrorKind::Validate
                | MergeErrorKind::Io
                | MergeErrorKind::Usage => ExitCode::from(2),
            }
        }
    }
}

fn run() -> Result<(), MergeError> {
    let mut args = std::env::args();
    let _program = args.next();
    let Some(base_path) = args.next() else {
        return usage();
    };
    let Some(current_path) = args.next() else {
        return usage();
    };
    let Some(other_path) = args.next() else {
        return usage();
    };
    if args.next().is_some() {
        return usage();
    }

    let base = read(&base_path)?;
    let current = read(&current_path)?;
    let other = read(&other_path)?;
    let merged = merge_ledgers(&base, &current, &other)?;
    write_atomic(&current_path, &merged)
}

/// Replace `%A` atomically: write a sibling temp file, then rename over the target. A crash
/// mid-merge therefore leaves `%A` byte-untouched — the incident-2 class (partial/garbage bytes
/// standing in the working tree) is structurally impossible, not merely unlikely. (A SIGKILL
/// between write and rename can strand the hidden pid-suffixed temp; cosmetic only — git's
/// `%A`/`%O`/`%B` temp files live outside the tree and the guarantee on `%A` still holds.)
fn write_atomic(path: &str, contents: &str) -> Result<(), MergeError> {
    let target = std::path::Path::new(path);
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("merge-result");
    let temp = target.with_file_name(format!(".{file_name}.oya-merge-tmp-{}", std::process::id()));
    std::fs::write(&temp, contents).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Io,
            format!("failed to write merge temp file {}: {err}", temp.display()),
        )
    })?;
    std::fs::rename(&temp, target).map_err(|err| {
        let _ = std::fs::remove_file(&temp);
        MergeError::new(
            MergeErrorKind::Io,
            format!("failed to move merged ledger into {path}: {err}"),
        )
    })
}

fn read(path: &str) -> Result<String, MergeError> {
    std::fs::read_to_string(path).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Io,
            format!("failed to read ledger {path}: {err}"),
        )
    })
}

fn usage<T>() -> Result<T, MergeError> {
    Err(MergeError::new(
        MergeErrorKind::Usage,
        "usage: oya-friction-ledger-merge-driver <base> <current> <other>",
    ))
}
