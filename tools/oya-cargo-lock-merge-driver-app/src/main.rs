//! Git merge-driver entrypoint: `oya-cargo-lock-merge-driver %O %A %B`.
//!
//! | exit | meaning | effect on `%A` |
//! |------|---------|----------------|
//! | 0 | merged cleanly | replaced atomically with the merged lockfile |
//! | 1 | conflict | replaced atomically with EVERY side under diff3 markers |
//! | 2 | `%A` is unknown (bad driver argument list) or unwritable | untouched |
//!
//! **Exit 1 still writes, and that is the point.** Git does not re-run its own text merge when a
//! driver exits nonzero — it takes whatever the driver left in `%A` as the conflicted working
//! tree. A driver that exits 1 without writing leaves `ours` alone, with no markers and the other
//! side's packages simply absent: the path is `UU` but the file reads as clean and complete, so a
//! reflexive `git add` loses `theirs` silently. Verified with a real `git merge` — a merge commit
//! byte-identical to `ours`, with `theirs`' package gone and nothing to make a human look.
//!
//! So an unreadable side, a lockfile that does not parse, and a genuine semantic conflict all take
//! the same path: write every side under markers, then signal. Exit 2 is reserved for the two
//! states where writing is not possible at all — a driver argument list that never named `%A`, and
//! a filesystem that refused the write.
#![forbid(unsafe_code)]

use std::process::ExitCode;

use oya_cargo_lock_merge_driver_app::{
    MergeError, MergeErrorKind, merge_lockfiles, whole_file_conflict,
};

/// Stands in for a side git handed us that could not be read. It goes INSIDE the markers, so the
/// human sees which side is missing instead of the driver quietly resolving to the side it could
/// read.
const UNREADABLE_SIDE: &str = "# oya-cargo-lock-merge-driver: this side could not be read";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            match err.kind() {
                MergeErrorKind::Conflict => ExitCode::from(1),
                MergeErrorKind::Parse | MergeErrorKind::Io | MergeErrorKind::Usage => {
                    ExitCode::from(2)
                }
            }
        }
    }
}

fn run() -> Result<(), MergeError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The one state that cannot be marked: with no `%A` argument there is no file to write to.
    let [base_path, current_path, other_path] = args.as_slice() else {
        return Err(MergeError::new(
            MergeErrorKind::Usage,
            "usage: oya-cargo-lock-merge-driver <base> <current> <other>",
        ));
    };

    let base = read(base_path);
    let current = read(current_path);
    let other = read(other_path);

    let outcome = match (&base, &current, &other) {
        (Ok(base), Ok(current), Ok(other)) => merge_lockfiles(base, current, other),
        _ => Err(first_read_failure(&base, &current, &other)),
    };

    match outcome {
        // Write first, THEN signal. In both branches `%A` already holds everything a human or a
        // build needs; the exit code only tells git whether one still has to look at it.
        Ok(merged) => write_atomic(current_path, &merged),
        Err(err) => {
            write_atomic(
                current_path,
                &whole_file_conflict(side(&base), side(&current), side(&other)),
            )?;
            Err(MergeError::new(
                MergeErrorKind::Conflict,
                format!("{err}; wrote every side into {current_path} under conflict markers"),
            ))
        }
    }
}

fn side(contents: &Result<String, MergeError>) -> &str {
    match contents {
        Ok(contents) => contents,
        Err(_) => UNREADABLE_SIDE,
    }
}

fn first_read_failure(
    base: &Result<String, MergeError>,
    current: &Result<String, MergeError>,
    other: &Result<String, MergeError>,
) -> MergeError {
    for side in [base, current, other] {
        if let Err(err) = side {
            return err.clone();
        }
    }
    MergeError::new(
        MergeErrorKind::Io,
        "internal: no side failed to read but the merge was not attempted",
    )
}

/// Replace `%A` atomically: write a sibling temp file, then rename over the target. A crash
/// mid-merge therefore leaves `%A` byte-untouched, so a half-written lockfile accepted by git as
/// the merged result is structurally impossible rather than merely unlikely.
fn write_atomic(path: &str, contents: &str) -> Result<(), MergeError> {
    let target = std::path::Path::new(path);
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("merge-result");
    let temp = target.with_file_name(format!(".{name}.oya-merge-tmp-{}", std::process::id()));
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
            format!("failed to move merged lockfile into {path}: {err}"),
        )
    })
}

fn read(path: &str) -> Result<String, MergeError> {
    std::fs::read_to_string(path).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Io,
            format!("failed to read lockfile {path}: {err}"),
        )
    })
}
