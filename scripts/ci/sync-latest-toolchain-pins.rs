//! Mechanical updater for Rust stable and Buck2 release pins.
//!
//! This is intentionally std-only so Buck2 can compile-check it without first
//! resolving Cargo dependencies. The normal repo-hygiene gate validates the
//! checked-in pins; this helper is for scheduled/manual updater lanes that open
//! an ordinary PR when upstream publishes a newer stable release.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const RUST_PIN_FILES: &[&str] = &[
    "rust-toolchain.toml",
    "Cargo.toml",
    "docs/standards/code-style-rust.md",
    "docs/standards/lts-versions-verified.md",
    "docs/standards/dependency-policy.md",
    "docs/standards/observability-slo.md",
    "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
    "specs/github-lane-unlocker-bridge.json",
    "specs/buck2-authority-policy.json",
    "specs/repo-hygiene-automation.json",
];

const BUCK2_PIN_FILES: &[&str] = &[
    ".github/workflows/github-lane-unlocker-ci-cd.yml",
    "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
    "specs/repo-hygiene-automation.json",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Check,
    Write,
    PrintCurrent,
    Version,
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("sync-latest-toolchain-pins: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    let mode = parse_mode()?;
    if mode == Mode::Version {
        println!("sync-latest-toolchain-pins 0.1.0");
        return Ok(());
    }

    let root = env::var("OYA_REPO_ROOT").unwrap_or_else(|_| ".".to_owned());
    let rust_version = current_stable_rust_version()?;
    let buck2_release = current_buck2_release()?;

    match mode {
        Mode::PrintCurrent => {
            println!("rust={rust_version}");
            println!("buck2={buck2_release}");
            Ok(())
        }
        Mode::Version => unreachable!("handled before upstream discovery"),
        Mode::Check => {
            check_pins(Path::new(&root), &rust_version, &buck2_release)?;
            println!("toolchain pins match latest known stable releases");
            Ok(())
        }
        Mode::Write => {
            rewrite_pins(Path::new(&root), &rust_version, &buck2_release)?;
            println!("updated toolchain pins to rust={rust_version} buck2={buck2_release}");
            Ok(())
        }
    }
}

fn parse_mode() -> Result<Mode, String> {
    let mut mode = Mode::Check;
    for arg in env::args().skip(1) {
        mode = match arg.as_str() {
            "--check" => Mode::Check,
            "--write" => Mode::Write,
            "--print-current" => Mode::PrintCurrent,
            "--version" => Mode::Version,
            other => return Err(format!("unknown argument {other}")),
        };
    }
    Ok(mode)
}

fn command_stdout(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("run {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {:?} exited with {}: {}",
            args,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("{program} output utf8: {error}"))
}

fn current_stable_rust_version() -> Result<String, String> {
    let stdout = command_stdout("rustc", &["+stable", "--version"])?;
    stdout
        .split_whitespace()
        .nth(1)
        .map(str::to_owned)
        .ok_or_else(|| format!("could not parse rustc version from {stdout:?}"))
}

fn current_buck2_release() -> Result<String, String> {
    let stdout = command_stdout(
        "git",
        &[
            "ls-remote",
            "--tags",
            "https://github.com/facebook/buck2.git",
            "refs/tags/[0-9]*",
        ],
    )?;
    stdout
        .lines()
        .filter_map(|line| line.rsplit_once("refs/tags/").map(|(_, tag)| tag.trim()))
        .filter(|tag| is_date_tag(tag))
        .max()
        .map(str::to_owned)
        .ok_or_else(|| "no Buck2 date tags found".to_owned())
}

fn is_date_tag(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn check_pins(root: &Path, rust_version: &str, buck2_release: &str) -> Result<(), String> {
    let mut failures = Vec::new();
    for rel in RUST_PIN_FILES {
        let text = read(root, rel)?;
        if !text.contains(rust_version) {
            failures.push(format!("{rel}: missing Rust stable pin {rust_version}"));
        }
    }
    for rel in BUCK2_PIN_FILES {
        let text = read(root, rel)?;
        if !text.contains(buck2_release) {
            failures.push(format!("{rel}: missing Buck2 release pin {buck2_release}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn rewrite_pins(root: &Path, rust_version: &str, buck2_release: &str) -> Result<(), String> {
    for rel in RUST_PIN_FILES {
        let text = read(root, rel)?;
        let text = replace_known_rust_versions(&text, rust_version);
        fs::write(root.join(rel), text).map_err(|error| format!("write {rel}: {error}"))?;
    }
    for rel in BUCK2_PIN_FILES {
        let text = read(root, rel)?;
        let text = replace_known_buck2_releases(&text, buck2_release);
        fs::write(root.join(rel), text).map_err(|error| format!("write {rel}: {error}"))?;
    }
    Ok(())
}

fn read(root: &Path, rel: &str) -> Result<String, String> {
    fs::read_to_string(root.join(rel)).map_err(|error| format!("read {rel}: {error}"))
}

fn replace_known_rust_versions(text: &str, rust_version: &str) -> String {
    text.replace("1.95.0", rust_version)
        .replace("1.96.0", rust_version)
}

fn replace_known_buck2_releases(text: &str, buck2_release: &str) -> String {
    text.replace("2026-05-18", buck2_release)
        .replace("2026-06-01", buck2_release)
}
