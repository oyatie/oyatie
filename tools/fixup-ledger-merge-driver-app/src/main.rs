//! Git merge-driver entrypoint: `fixup-ledger-merge-driver %O %A %B`.
//!
//! Exit 0 = merged cleanly. Exit 1 = merged WITH conflict markers, which a human must resolve.
//! Exit 2 = unmodelled input or I/O; `%A` is left byte-untouched.
//!
//! Exit 1 still WRITES `%A`, and that is the whole point. Git does not re-run its own text merge
//! when a driver exits nonzero — it takes whatever the driver left in `%A` as the conflicted
//! working tree. A driver that exits 1 without writing therefore leaves `ours` standing alone with
//! no markers and the other side's rows simply absent: the file looks clean and complete, so a
//! reflexive `git add` loses rows silently. Verified with a real `git merge`. So on conflict the
//! driver writes a file containing EVERY row from every side, with diff3 markers around the
//! regions that need a human.
#![forbid(unsafe_code)]

use std::process::ExitCode;

use fixup_ledger_merge_driver_app::{MergeError, MergeErrorKind, merge_ledgers};

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
    // Write first, THEN signal. The content is complete in both cases; the exit code only tells
    // git whether a human still has to look at it.
    write_atomic(current_path, &merged.content)?;
    if merged.conflicted {
        return Err(MergeError::new(
            MergeErrorKind::Conflict,
            "left conflict markers in the ledger; resolve them and `git add` the file",
        ));
    }
    Ok(())
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
