//! Hermetic, zero-manual-step activation for the born-accounting faces merge driver + hooks.
//!
//! Subcommands:
//!
//!   oya-repo-hooks-bootstrap [bootstrap] [--repo-root <path>] [--driver-bin <path>]
//!     Idempotently install `merge.oya-faces.{name,driver}` git config, `core.hooksPath .githooks`,
//!     the `.githooks/{post-merge,post-rewrite,post-checkout}` shims, and the `.gitattributes`
//!     generated glob block. Default subcommand. Driver path resolves explicit > env > PATH.
//!
//!   oya-repo-hooks-bootstrap gitattributes [--repo-root <path>] [--check]
//!     Regenerate (or, with --check, verify) the `.gitattributes` generated glob block from the
//!     control-plane. `--check` exits 1 if the block is stale (CI drift guard).
//!
//! Exit codes: 0 success; 1 `gitattributes --check` found drift; 2 usage / bootstrap / IO failure.
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use oya_faces_merge_driver_app::ControlPlane;
use oya_repo_hooks_bootstrap_app::{
    BootstrapError, GITATTRIBUTES_PATH, bootstrap, render_gitattributes_block, resolve_driver_bin,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("oya-repo-hooks-bootstrap: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, BootstrapError> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let subcommand = match args.first().map(String::as_str) {
        Some("gitattributes") => {
            args.remove(0);
            "gitattributes"
        }
        Some("bootstrap") => {
            args.remove(0);
            "bootstrap"
        }
        // Default subcommand is bootstrap (the common case the post-checkout hook + CI step call).
        _ => "bootstrap",
    };
    match subcommand {
        "bootstrap" => run_bootstrap(args),
        "gitattributes" => run_gitattributes(args),
        other => Err(BootstrapError::new(format!("unknown subcommand {other:?}"))),
    }
}

fn run_bootstrap(args: Vec<String>) -> Result<ExitCode, BootstrapError> {
    let mut repo_root: Option<PathBuf> = None;
    let mut driver_bin: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = args.get(i).map(PathBuf::from);
            }
            "--driver-bin" => {
                i += 1;
                driver_bin = args.get(i).cloned();
            }
            other => return Err(BootstrapError::new(format!("bootstrap: unknown argument {other:?}"))),
        }
        i += 1;
    }
    let repo_root = repo_root.unwrap_or_else(|| PathBuf::from("."));
    let driver = resolve_driver_bin(driver_bin.as_deref());
    let report = bootstrap(&repo_root, &driver)?;
    if report.is_noop() {
        println!("oya-repo-hooks-bootstrap: already activated (no-op)");
    } else {
        println!(
            "oya-repo-hooks-bootstrap: activated (config={} hooksPath={} hooks={:?} gitattributes={})",
            report.merge_driver_configured,
            report.hooks_path_configured,
            report.hooks_installed,
            report.gitattributes_updated
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn run_gitattributes(args: Vec<String>) -> Result<ExitCode, BootstrapError> {
    let mut repo_root: Option<PathBuf> = None;
    let mut check = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = args.get(i).map(PathBuf::from);
            }
            "--check" => check = true,
            other => {
                return Err(BootstrapError::new(format!(
                    "gitattributes: unknown argument {other:?}"
                )));
            }
        }
        i += 1;
    }
    let repo_root = repo_root.unwrap_or_else(|| PathBuf::from("."));
    let control_plane = ControlPlane::load(&repo_root)?;
    let desired = render_gitattributes_block(&control_plane);
    let path = repo_root.join(GITATTRIBUTES_PATH);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    if check {
        if existing.contains(&desired) {
            println!("oya-repo-hooks-bootstrap: .gitattributes faces glob is current");
            Ok(ExitCode::SUCCESS)
        } else {
            eprintln!(
                "oya-repo-hooks-bootstrap: .gitattributes faces glob is STALE; run `oya-repo-hooks-bootstrap gitattributes` to regenerate"
            );
            Ok(ExitCode::from(1))
        }
    } else {
        // Regenerate via the same idempotent bootstrap path (only the .gitattributes step changes).
        let driver = resolve_driver_bin(None);
        let report = bootstrap(&repo_root, &driver)?;
        println!(
            "oya-repo-hooks-bootstrap: .gitattributes faces glob {}",
            if report.gitattributes_updated { "updated" } else { "already current" }
        );
        Ok(ExitCode::SUCCESS)
    }
}
