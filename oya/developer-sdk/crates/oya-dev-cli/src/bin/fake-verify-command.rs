//! Test fixture for `oya verify --ci-required`.
//!
//! Integration tests copy this binary into a temporary PATH as `cargo`,
//! `oya`, and `git`. It records each invocation to `FAKE_VERIFY_LOG`,
//! emits `FAKE_VERIFY_GIT_ROOT` / `FAKE_VERIFY_GIT_DIFF` for the git
//! calls the verifier uses, and fails any invocation whose command line
//! contains a pattern listed in `FAKE_VERIFY_FAILURES`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let argv0 = env::args().next().unwrap_or_default();
    let name = Path::new(&argv0)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("fake-verify-command")
        .to_string();
    let args: Vec<String> = env::args().skip(1).collect();
    let signature = if args.is_empty() {
        name.clone()
    } else {
        format!("{} {}", name, args.join(" "))
    };

    if let Ok(path) = env::var("FAKE_VERIFY_LOG") {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("fake verify log opens");
        writeln!(file, "{signature}").expect("fake verify log writes");
    }

    match (name.as_str(), args.as_slice()) {
        ("git", [cmd, flag]) if cmd == "rev-parse" && flag == "--show-toplevel" => {
            println!(
                "{}",
                env::var("FAKE_VERIFY_GIT_ROOT").unwrap_or_else(|_| ".".into())
            );
            return ExitCode::SUCCESS;
        }
        ("git", [cmd, rev]) if cmd == "rev-parse" && rev == "HEAD" => {
            println!(
                "{}",
                env::var("FAKE_VERIFY_GIT_HEAD")
                    .unwrap_or_else(|_| "fixture-head-1234567890abcdef".into())
            );
            return ExitCode::SUCCESS;
        }
        ("git", [cmd, short, untracked])
            if cmd == "status" && short == "--short" && untracked == "--untracked-files=all" =>
        {
            if let Ok(status) = env::var("FAKE_VERIFY_GIT_STATUS") {
                print!("{status}");
                if !status.ends_with('\n') {
                    println!();
                }
            }
            return ExitCode::SUCCESS;
        }
        ("git", [cmd, name_only, diff_filter, range, sep, pathspec])
            if cmd == "diff"
                && name_only == "--name-only"
                && diff_filter == "--diff-filter=A"
                && range == "origin/dev...HEAD"
                && sep == "--"
                && pathspec == "docs/decisions/ADR-*.md" =>
        {
            if let Ok(diff) = env::var("FAKE_VERIFY_GIT_DIFF") {
                print!("{diff}");
                if !diff.ends_with('\n') {
                    println!();
                }
            }
            return ExitCode::SUCCESS;
        }
        _ => {}
    }

    let should_fail = env::var("FAKE_VERIFY_FAILURES")
        .unwrap_or_default()
        .split(';')
        .filter(|pattern| !pattern.trim().is_empty())
        .any(|pattern| signature.contains(pattern.trim()));
    if should_fail {
        eprintln!("fake failure: {signature}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
