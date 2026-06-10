#![forbid(unsafe_code)]

use std::process::ExitCode;

use oya_cargo_lock_merge_driver_app::{MergeError, MergeErrorKind, merge_lockfiles};

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
    let merged = merge_lockfiles(&base, &current, &other)?;
    std::fs::write(&current_path, merged).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Io,
            format!("failed to write merged lockfile {current_path}: {err}"),
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

fn usage<T>() -> Result<T, MergeError> {
    Err(MergeError::new(
        MergeErrorKind::Usage,
        "usage: oya-cargo-lock-merge-driver <base> <current> <other>",
    ))
}
