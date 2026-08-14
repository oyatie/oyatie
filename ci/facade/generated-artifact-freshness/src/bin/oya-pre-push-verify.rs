//! git pre-push hook: owned-Rust automation layer in front of the CI backstop.
//!
//! Canonical protocol (`docs/oya-ci/gate-catalog.md`, FRIC-1781250000 / ADR-0552):
//! `oya-cloud-ci-face-settle --verify` is the REQUIRED last step before EVERY push.
//! It is read-only (never writes to the repository): against a working tree asserted
//! byte-identical to HEAD it regenerates the generated faces and runs the freshness
//! gate's own full check, exiting nonzero with the stale list and exact remediation.
//!
//! This binary is that hook, per ADR-0548 D6 the automation-default LOCAL check in
//! front of the canonical cloud-ci freshness gate behind `oya-ci-required` (ADR-0515),
//! never a substitute for it. The repository stack invariant requires automation
//! deliverables to be owned Rust, so the hook logic lives here instead of a shell
//! script; the installed artifact is the compiled binary, not a branch-controlled file.
//!
//! Install (writes OUTSIDE the checked-out tree, into the git common-dir hooks dir, so
//! a contribution can never replace the hook git executes):
//!
//! ```text
//! buck2 run //ci/facade/generated-artifact-freshness:oya-pre-push-verify-bin -- install
//! ```
//!
//! Every branch push then runs the read-only `--verify`; a failing verify blocks the
//! push with the tool's own stale list and remediation output. No bypass path exists by
//! design (`docs/AGENTS.md`: a failing hook means fix the faces, not the hook).

#![forbid(unsafe_code)]

use std::env;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use ci_generated_artifact_freshness::{run_face_settle_with_buck2, FaceSettleMode};

const ZERO_SHA: &str = "0000000000000000000000000000000000000000";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("install") {
        return match install(&args[1..]) {
            Ok(message) => {
                println!("{message}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("pre-push install: {message}");
                ExitCode::FAILURE
            }
        };
    }
    match run_hook() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("pre-push: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Hook mode: read the refs git passes on stdin, gate the push, and run the verify.
fn run_hook() -> Result<ExitCode, String> {
    let pushed_shas = read_pushed_branch_shas()?;
    if pushed_shas.is_empty() {
        // Tag push or branch deletion: no face-relevant commit to verify.
        return Ok(ExitCode::SUCCESS);
    }

    // Local-bridge posture: without buck2 there is nothing to run locally; the CI
    // freshness gate remains the merge backstop, so never block a push the local
    // toolchain cannot judge.
    if !buck2_available() {
        println!(
            "pre-push: buck2 not found; skipping local face-settle --verify (the cloud-ci freshness gate in oya-ci-required still applies)."
        );
        return Ok(ExitCode::SUCCESS);
    }

    // The verify certifies the COMMITTED tree (HEAD) only. Every pushed non-deletion
    // local SHA must equal HEAD, otherwise fail closed: a settled checkout must not let
    // a stale `topic` commit pass just because a different branch is checked out.
    let head = head_sha()?;
    for sha in &pushed_shas {
        if sha != &head {
            return Err(format!(
                "refusing to certify push of {sha}: the local face-settle --verify certifies \
                 the committed tree at HEAD ({head}) only; push the checked-out branch, or use \
                 a per-branch worktree whose HEAD is the branch you are pushing"
            ));
        }
    }

    let root = repo_root()?;
    let report = run_face_settle_with_buck2(&root, FaceSettleMode::Verify)
        .map_err(|error| format!("face-settle --verify failed: {error}"))?;
    println!("{}", report.message);
    Ok(if report.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// Parse git's pre-push stdin lines (`<local ref> <local sha> <remote ref> <remote sha>`)
/// and return the local SHA of every branch push that is not a deletion.
fn read_pushed_branch_shas() -> Result<Vec<String>, String> {
    let stdin = io::stdin();
    let mut shas = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("read pre-push stdin: {error}"))?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 4 {
            return Err(format!("malformed pre-push line: {line:?}"));
        }
        let local_sha = fields[1];
        let remote_ref = fields[2];
        if remote_ref.starts_with("refs/heads/") && local_sha != ZERO_SHA {
            shas.push(local_sha.to_owned());
        }
    }
    Ok(shas)
}

fn buck2_available() -> bool {
    Command::new("buck2")
        .arg("--version")
        .output()
        .is_ok()
}

fn head_sha() -> Result<String, String> {
    git_capture(None, &["rev-parse", "HEAD"])
}

fn repo_root() -> Result<PathBuf, String> {
    Ok(PathBuf::from(git_capture(None, &["rev-parse", "--show-toplevel"])?))
}

/// Installer mode: copy this compiled binary into the git common-dir `hooks/` directory
/// (OUTSIDE the checked-out tree) and point `core.hooksPath` at it. Git then executes a
/// pinned verifier — never a branch-controlled file from whichever branch is checked out.
fn install(args: &[String]) -> Result<String, String> {
    let mut repo_root: Option<PathBuf> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err("install: --repo-root requires a path".to_owned());
                };
                repo_root = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return Err(
                    "usage: oya-pre-push-verify install [--repo-root <path>]".to_owned(),
                );
            }
            other => {
                return Err(format!(
                    "install: unknown argument {other:?}; usage: oya-pre-push-verify install [--repo-root <path>]"
                ));
            }
        }
    }
    let root = match repo_root {
        Some(root) => root,
        None => repo_root()?,
    };
    install_into_common_hooks_dir(&root)
}

fn install_into_common_hooks_dir(repo_root: &Path) -> Result<String, String> {
    let common_dir_raw = git_capture(Some(repo_root), &["rev-parse", "--git-common-dir"])?;
    let common_dir = if Path::new(&common_dir_raw).is_absolute() {
        PathBuf::from(common_dir_raw)
    } else {
        repo_root.join(common_dir_raw)
    };
    let hooks_dir = common_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|error| format!("create hooks dir {}: {error}", hooks_dir.display()))?;

    let exe = env::current_exe()
        .map_err(|error| format!("resolve current executable: {error}"))?;
    let dest = hooks_dir.join("pre-push");
    std::fs::copy(&exe, &dest)
        .map_err(|error| format!("copy {} -> {}: {error}", exe.display(), dest.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)
            .map_err(|error| format!("stat {}: {error}", dest.display()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)
            .map_err(|error| format!("chmod {}: {error}", dest.display()))?;
    }

    let hooks_dir_abs = std::fs::canonicalize(&hooks_dir)
        .unwrap_or_else(|_| hooks_dir.clone());
    let hooks_dir_str = hooks_dir_abs
        .to_str()
        .ok_or_else(|| "hooks dir is not valid UTF-8".to_owned())?;
    git_run(
        Some(repo_root),
        &["config", "core.hooksPath", hooks_dir_str],
    )?;

    Ok(format!(
        "installed pinned pre-push verifier at {} (outside the checked-out tree); core.hooksPath set to {}",
        dest.display(),
        hooks_dir_abs.display()
    ))
}

fn git_capture(cwd: Option<&Path>, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command
        .output()
        .map_err(|error| format!("run git {args:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn git_run(cwd: Option<&Path>, args: &[&str]) -> Result<(), String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command
        .output()
        .map_err(|error| format!("run git {args:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}
