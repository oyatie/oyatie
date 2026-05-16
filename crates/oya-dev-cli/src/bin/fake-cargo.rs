//! Test fixture: emulates the subset of `cargo` behavior the
//! `crates/oya-dev-cli/tests/doc_cli.rs` integration tests need.
//!
//! Why this exists: those tests previously wrote tiny shell scripts
//! (`fake-cargo.sh`) into a tempdir and `chmod 0755`'d them. Linux's
//! `exec()` rejects scripts without a shebang with ENOEXEC; macOS
//! silently falls back to `/bin/sh`, so the tests passed locally and
//! failed on CI. Per the no-exceptions canonical posture + the user's
//! "fully replace shell scripts and python scripts" directive, the
//! canonical fix is to remove the shell script entirely and have the
//! tests invoke a Rust binary instead.
//!
//! Behavior is driven by env vars the caller sets on the `oya` command
//! it invokes; `std::process::Command` inherits env into child processes
//! by default, so the env vars propagate down to this binary when the
//! `oya doc rustdoc` runner spawns it as the `--cargo` executable.
//!
//! - `FAKE_CARGO_STDOUT`  literal string to write to stdout (one line)
//! - `FAKE_CARGO_STDERR`  literal string to write to stderr (one line)
//! - `FAKE_CARGO_EXIT`    numeric exit code (default `0`)
//! - `FAKE_CARGO_PRINT_ARGS=1` also print `cargo-args:<args>`,
//!   `rustdoc:<RUSTDOC>`, `target:<CARGO_TARGET_DIR>` (mirrors what
//!   the success-path test expects from the rustdoc invocation echo).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if let Ok(out) = env::var("FAKE_CARGO_STDOUT") {
        println!("{out}");
    }
    if let Ok(err) = env::var("FAKE_CARGO_STDERR") {
        let stderr = std::io::stderr();
        let mut lock = stderr.lock();
        let _ = writeln!(lock, "{err}");
    }
    if env::var("FAKE_CARGO_PRINT_ARGS").as_deref() == Ok("1") {
        println!("cargo-args:{}", args.join(" "));
        println!("rustdoc:{}", env::var("RUSTDOC").unwrap_or_default());
        println!(
            "target:{}",
            env::var("CARGO_TARGET_DIR").unwrap_or_default()
        );
    }

    let exit = env::var("FAKE_CARGO_EXIT")
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0);
    ExitCode::from(exit)
}
