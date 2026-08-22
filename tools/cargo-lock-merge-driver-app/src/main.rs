//! Git merge-driver entrypoint: `cargo-lock-merge-driver %O %A %B [...]`.
//!
//! | exit | meaning | contents of `%A` | effect on the merge |
//! |------|---------|------------------|---------------------|
//! | 0 | merged cleanly | the merged lockfile | resolved |
//! | 1 | conflict | EVERY side under diff3 markers | `UU`, a human resolves |
//! | 129 | `%A` unknown or unwritable | untouched | git ABANDONS the merge |
//!
//! **No exit leaves `%A` as unmarked `ours` while git believes a merge happened.** Git does not
//! re-run its own text merge when a driver exits nonzero — it takes whatever the driver left in
//! `%A` as the conflicted working tree. A driver that exits nonzero without writing therefore
//! leaves `ours` alone, with no markers and the other side's packages simply absent: the path is
//! `UU` but the file reads as clean and complete, so a reflexive `git add` loses `theirs` silently.
//! Verified twice with a real `git merge`, each time producing a merge commit identical to `ours`.
//!
//! That state is unreachable here by construction, not by remembering to write on each branch:
//!
//! 1. **The conflict document goes into `%A` BEFORE anything that can fail.** Once that write
//!    lands, every remaining way this process can end — a parse failure, a semantic conflict, a
//!    panic, an abort, a `SIGKILL` — leaves `%A` carrying every side under diff3 markers. Nothing
//!    downstream has to remember anything; a clean merge merely *upgrades* `%A` afterwards. This
//!    is why there is no `catch_unwind` and no error type threaded through: correctness does not
//!    depend on catching anything.
//! 2. **Before that write, any exit is >128.** gitattributes(5) defines >128 as the driver having
//!    crashed, so git fails the merge outright instead of recording a conflict: no `MERGE_HEAD`,
//!    no `UU` path, nothing for a `git add` to commit. A merge that did not happen loses nothing.
//!    A panic hook covers the same window, since a bare panic would otherwise exit 101 — a code
//!    git reads as an ordinary conflict.
//!
//! Exit 2 no longer exists. It was the one code that told git "conflict" while promising nothing
//! about `%A`, which is exactly the shape of the data loss.
#![forbid(unsafe_code)]

use std::process::ExitCode;

use cargo_lock_merge_driver_app::{merge_lockfiles, whole_file_conflict};

/// Nonzero AND >128, which gitattributes(5) defines as the driver having crashed. Git then fails
/// the merge outright instead of recording a conflict, which is the only safe answer while `%A`
/// cannot be given correct contents.
const ABORT_MERGE: u8 = 129;

/// Stands in for a side git handed us that could not be read. It goes INSIDE the markers, so the
/// human sees which side is missing instead of the driver quietly resolving to the side it could
/// read.
const UNREADABLE_SIDE: &str = "# cargo-lock-merge-driver: this side could not be read";

fn main() -> ExitCode {
    // A panic would otherwise exit 101, which git reads as an ordinary conflict — the one way a
    // crash could still present unwritten `%A` as a resolved-looking merge. Exiting >128 instead
    // makes git abandon the merge. After the first write below every outcome is already safe, so
    // this only has to cover the window before it.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("cargo-lock-merge-driver: panicked, abandoning the merge: {info}");
        std::process::exit(i32::from(ABORT_MERGE));
    }));

    let args: Vec<String> = std::env::args().skip(1).collect();
    // `%O %A %B` are read positionally. Git may substitute more — `%L` (conflict-marker size),
    // `%P` (pathname), `%S`/`%X`/`%Y` (conflict labels) — and `merge.faces` in this very repo
    // is already registered as `%O %A %B %P`, so the tail is accepted and ignored.
    //
    // Ignored, not decoded: git substitutes whatever order the config names, so a positional guess
    // at which extra is `%P` would print the conflict-marker size as a filename. Nothing is lost,
    // because git already names the real path in its own `CONFLICT (content): Merge conflict in
    // <path>` line. Demanding exactly three arguments was itself a data-loss bug — a four-
    // placeholder registration took the usage branch, exited without writing, and git kept
    // unmarked `ours`.
    let [base_path, current_path, other_path, ..] = args.as_slice() else {
        eprintln!(
            "cargo-lock-merge-driver: expected at least <base> <current> <other>, got {} \
             argument(s); abandoning the merge rather than resolving it to one side",
            args.len()
        );
        return ExitCode::from(ABORT_MERGE);
    };

    let base = read(base_path);
    let current = read(current_path);
    let other = read(other_path);

    // THE load-bearing line. Put the safe answer in `%A` first; upgrade it only if the merge
    // actually succeeds. Every exit from here down is safe whether or not it is reached
    // deliberately.
    if let Err(err) = write_atomic(
        current_path,
        &whole_file_conflict(side(&base), side(&current), side(&other)),
    ) {
        eprintln!("cargo-lock-merge-driver: {err}");
        // `%A` still holds `ours`. Exiting 1 here would present that as a resolved-looking
        // conflict, so abandon the merge instead.
        return ExitCode::from(ABORT_MERGE);
    }

    let (Ok(base_text), Ok(current_text), Ok(other_text)) = (&base, &current, &other) else {
        return conflict(&read_failure(&[&base, &current, &other]));
    };

    let merged = match merge_lockfiles(base_text, current_text, other_text) {
        Ok(merged) => merged,
        Err(err) => return conflict(&err.to_string()),
    };

    match write_atomic(current_path, &merged) {
        Ok(()) => ExitCode::SUCCESS,
        // The merge succeeded but could not be installed. `%A` still carries every side, so this
        // is an ordinary conflict a human can resolve, not a reason to abandon the merge.
        Err(err) => conflict(&format!(
            "merged cleanly but could not replace the lockfile: {err}"
        )),
    }
}

/// Report a conflict whose document is ALREADY in `%A`. Deliberately does not name the file: `%A`
/// is a git temp path (`.merge_file_XKfHHR`) that will not exist by the time a human looks, and
/// git names the real one on the next line.
fn conflict(reason: &str) -> ExitCode {
    eprintln!(
        "cargo-lock-merge-driver: {reason}; every side is in the file under conflict markers \
         — resolve them by hand, it does not parse as TOML until you do"
    );
    ExitCode::from(1)
}

fn side(contents: &Result<String, String>) -> &str {
    match contents {
        Ok(contents) => contents,
        Err(_) => UNREADABLE_SIDE,
    }
}

fn read_failure(sides: &[&Result<String, String>]) -> String {
    for side in sides {
        if let Err(err) = side {
            return err.clone();
        }
    }
    "internal: no side failed to read but the merge was not attempted".to_owned()
}

/// Replace `%A` atomically: write a sibling temp file, then rename over the target. `%A` is
/// therefore always either its previous contents or the full new ones, so the conflict document
/// cannot land half-written and be accepted by git as the merged result.
fn write_atomic(path: &str, contents: &str) -> Result<(), String> {
    let target = std::path::Path::new(path);
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("merge-result");
    let temp = target.with_file_name(format!(".{name}.merge-tmp-{}", std::process::id()));
    std::fs::write(&temp, contents)
        .map_err(|err| format!("failed to write merge temp file {}: {err}", temp.display()))?;
    std::fs::rename(&temp, target).map_err(|err| {
        let _ = std::fs::remove_file(&temp);
        format!("failed to move merged lockfile into {path}: {err}")
    })
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|err| format!("failed to read lockfile {path}: {err}"))
}
