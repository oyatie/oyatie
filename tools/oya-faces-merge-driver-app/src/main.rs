//! Git merge-driver + post-merge settle entrypoint for the born-accounting generated faces.
//!
//! Two subcommands:
//!
//!   oya-faces-merge-driver driver <%O> <%A> <%B> <%P>
//!     The per-file merge driver git invokes via `.gitattributes <faces-glob> merge=oya-faces`
//!     plus the `merge.oya-faces.driver` git config the bootstrap installs. COSMETIC: takes theirs
//!     (%B) into %A so git records the face resolved, exit 0. The post-merge settle is authoritative.
//!
//!   oya-faces-merge-driver settle [--repo-root <path>]
//!     The authoritative regeneration run by the checked-in `.githooks/post-merge` +
//!     `.githooks/post-rewrite` hooks AFTER the merge/rebase commit exists. Regenerates ALL faces
//!     from the committed merged tree, byte-rediff + determinism self-check, then settles.
//!
//! Exit codes (mirroring the cargo-lock / friction-ledger precedents):
//!   0 = success.
//!   1 = the driver declines this merge (e.g. a non-declared face) — git keeps the conflict.
//!   2 = control-plane / regen / drift / determinism / settle / IO / usage failure. On ANY nonzero
//!       exit no guessed or partial face is written (fail closed).
#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use oya_faces_merge_driver_app::{
    Buck2RegenAdapter, FacesMergeError, FacesMergeErrorKind, run_merge_driver, run_post_merge_settle,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("oya-faces-merge-driver: {err}");
            match err.kind() {
                FacesMergeErrorKind::Conflict => ExitCode::from(1),
                FacesMergeErrorKind::ControlPlane
                | FacesMergeErrorKind::Regen
                | FacesMergeErrorKind::Drift
                | FacesMergeErrorKind::Settle
                | FacesMergeErrorKind::Io
                | FacesMergeErrorKind::Usage => ExitCode::from(2),
            }
        }
    }
}

fn run() -> Result<(), FacesMergeError> {
    let mut args = std::env::args();
    let _program = args.next();
    let Some(subcommand) = args.next() else {
        return usage();
    };
    match subcommand.as_str() {
        "driver" => run_driver(args.collect()),
        "settle" => run_settle(args.collect()),
        other => Err(FacesMergeError::new(
            FacesMergeErrorKind::Usage,
            format!("unknown subcommand {other:?}; {USAGE}"),
        )),
    }
}

/// `driver <%O> <%A> <%B> <%P>` — the per-file merge driver. The repo root is discovered up-tree
/// from the merge-target path's directory (git invokes the driver with the working tree as cwd, but
/// %A is a temp file, so root discovery walks up from cwd).
fn run_driver(args: Vec<String>) -> Result<(), FacesMergeError> {
    if args.len() != 4 {
        return Err(FacesMergeError::new(
            FacesMergeErrorKind::Usage,
            format!("driver expects exactly %O %A %B %P (4 args), got {}; {USAGE}", args.len()),
        ));
    }
    let ancestor = PathBuf::from(&args[0]);
    let ours = PathBuf::from(&args[1]);
    let theirs = PathBuf::from(&args[2]);
    let pathname = &args[3];

    // git runs merge drivers with the repo working tree as cwd, so discover the root from cwd.
    let repo_root = discover_repo_root()?;
    run_merge_driver(&repo_root, &ancestor, &ours, &theirs, pathname)
}

/// `settle [--repo-root <path>]` — the authoritative post-merge / post-rewrite settle.
fn run_settle(args: Vec<String>) -> Result<(), FacesMergeError> {
    let mut repo_root: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = args.get(i).map(PathBuf::from);
            }
            other => {
                return Err(FacesMergeError::new(
                    FacesMergeErrorKind::Usage,
                    format!("settle: unknown argument {other:?}; {USAGE}"),
                ));
            }
        }
        i += 1;
    }
    let repo_root = match repo_root {
        Some(root) => root,
        None => discover_repo_root()?,
    };
    let regen = Buck2RegenAdapter;
    let faces = run_post_merge_settle(&repo_root, &regen)?;
    if faces.is_empty() {
        println!("oya-faces-merge-driver: no faces to settle");
    } else {
        println!(
            "oya-faces-merge-driver: settled {} generated faces from the merged tree",
            faces.len()
        );
    }
    Ok(())
}

/// Walk up from cwd to the repo root (the dir holding `specs/root-hub-pointers.json`), matching the
/// producer/emitter root-discovery convention. Used by both subcommands.
fn discover_repo_root() -> Result<PathBuf, FacesMergeError> {
    const MARKER: &str = "specs/root-hub-pointers.json";
    let mut dir = std::env::current_dir().map_err(|e| {
        FacesMergeError::new(FacesMergeErrorKind::Io, format!("current dir: {e}"))
    })?;
    for _ in 0..32 {
        if dir.join(MARKER).is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(FacesMergeError::new(
        FacesMergeErrorKind::Io,
        format!("failed to locate repo root (no {MARKER} up-tree from cwd)"),
    ))
}

const USAGE: &str =
    "usage: oya-faces-merge-driver driver <%O> <%A> <%B> <%P> | settle [--repo-root <path>]";

fn usage() -> Result<(), FacesMergeError> {
    Err(FacesMergeError::new(FacesMergeErrorKind::Usage, USAGE))
}
