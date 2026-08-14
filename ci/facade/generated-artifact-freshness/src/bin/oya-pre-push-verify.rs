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
//! Security posture (fail-closed on all three axes):
//! 1. The hook NEVER builds from the active checkout's Buck graph. The generator tools
//!    (emitter/producer/masterplan/architecture-graph) are built ONCE at install time
//!    from the install-time checkout and copied into the hooks dir as PINNED binaries;
//!    at push time the hook executes only those prebuilt tools against the repo tree
//!    consumed as DATA. A malicious branch therefore cannot make the hook execute
//!    checkout-controlled code.
//! 2. Install PRESERVES an existing `core.hooksPath`: if one is configured (e.g.
//!    organization-managed commit-msg/signing hooks) the verifier is installed into
//!    that same directory and the configuration is left untouched; if none is set, the
//!    verifier goes into git's default hooks dir and no configuration is written.
//! 3. A protocol handshake fails closed with an explicit reinstall requirement: the
//!    installed manifest must carry the embedded protocol version, and the repository's
//!    tracked control-plane must still declare exactly the generated faces this
//!    verifier covers — otherwise the hook refuses to run and demands a reinstall.
//!
//! Install (writes OUTSIDE the checked-out tree, into the git hooks dir, so a
//! contribution can never replace the hook git executes):
//!
//! ```text
//! buck2 run //ci/facade/generated-artifact-freshness:oya-pre-push-verify-bin -- install
//! ```

#![forbid(unsafe_code)]

use std::env;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use ci_generated_artifact_freshness::{
    PRE_PUSH_VERIFIER_MANIFEST_FILE, PRE_PUSH_VERIFIER_PROTOCOL_VERSION,
    PRE_PUSH_VERIFIER_TOOLS_DIR, FaceSettleMode, install_pre_push_verifier_tools,
    read_pre_push_verifier_manifest, run_face_settle_with_pinned_tools,
    verify_pre_push_verifier_protocol, write_pre_push_verifier_manifest,
};

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

/// Hook mode: read the refs git passes on stdin, gate the push, and run the verify with
/// the pinned tools installed alongside this binary.
fn run_hook() -> Result<ExitCode, String> {
    let pushed_shas = read_pushed_branch_shas()?;
    if pushed_shas.is_empty() {
        // Tag push or branch deletion: no face-relevant commit to verify.
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
    // The installed hook lives at <hooks-dir>/pre-push, so its own directory is the hooks dir.
    let exe = env::current_exe()
        .map_err(|error| format!("resolve current executable: {error}"))?;
    let hooks_dir = exe
        .parent()
        .ok_or_else(|| "installed hook has no parent directory".to_owned())?
        .to_path_buf();
    let tools_dir = hooks_dir.join(PRE_PUSH_VERIFIER_TOOLS_DIR);

    // Protocol handshake (fail closed with explicit reinstall requirement): the install
    // manifest must exist, match this binary's embedded protocol version, bind to the
    // pinned tool builds actually installed next to the hook, AND bind to the repository's
    // generator source the pinned tools were built from — so a generator change (even one
    // that leaves the Buck label and protocol integer untouched) fails closed instead of
    // silently verifying with stale tools.
    let _ = read_pre_push_verifier_manifest(&hooks_dir, &tools_dir, &root)
        .map_err(|error| error.to_string())?;
    // ...and the repository must still declare exactly the faces this verifier covers.
    verify_pre_push_verifier_protocol(&root).map_err(|error| error.to_string())?;

    // Run the verify with the PINNED tools — never building from the active checkout.
    let report = run_face_settle_with_pinned_tools(&root, FaceSettleMode::Verify, &tools_dir)
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

fn head_sha() -> Result<String, String> {
    git_capture(None, &["rev-parse", "HEAD"])
}

fn repo_root() -> Result<PathBuf, String> {
    Ok(PathBuf::from(git_capture(
        None,
        &["rev-parse", "--show-toplevel"],
    )?))
}

/// Installer mode: pin the generator tools (built once from this checkout), copy this
/// verifier into the git hooks dir (OUTSIDE the checked-out tree, preserving any existing
/// `core.hooksPath`), and write the protocol manifest.
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

    // Preserve an existing configured hooks path (org-managed commit-msg/signing/security
    // hooks): install INTO that same directory and never rewrite core.hooksPath. Only when
    // no LOCAL path is configured do we fall back to git's default hooks dir, and we do NOT
    // set core.hooksPath then either (git already resolves the default). A hooks path
    // configured at global/system scope is refused: installing into a shared directory would
    // run this repository's verifier for every repository owned by the user.
    let hooks_dir = resolve_hooks_dir(&root)?;

    let exe = env::current_exe()
        .map_err(|error| format!("resolve current executable: {error}"))?;
    let dest = hooks_dir.join("pre-push");
    // Allow REPLACING a prior oya installation (the manifest next to it marks it as ours): a
    // protocol bump builds a byte-different verifier, and refusing would block the very reinstall
    // the handshake demands. Refuse only a pre-push that is NOT marked as ours.
    let is_prior_oya_install = hooks_dir.join(PRE_PUSH_VERIFIER_MANIFEST_FILE).exists();
    if dest.exists()
        && !is_prior_oya_install
        && !files_equal(&dest, &exe)?
    {
        return Err(format!(
            "refusing to overwrite {}: an unrelated pre-push hook is installed there; move it aside or remove it first (the oya verifier never replaces unrelated user hook state)",
            dest.display()
        ));
    }

    // Pin the generator tools FIRST (built from this trusted install checkout), then copy
    // the verifier, then write the manifest last so a partial install fails closed.
    let tools_dir = hooks_dir.join(PRE_PUSH_VERIFIER_TOOLS_DIR);
    install_pre_push_verifier_tools(&root, &tools_dir).map_err(|error| error.to_string())?;

    std::fs::create_dir_all(&hooks_dir)
        .map_err(|error| format!("create hooks dir {}: {error}", hooks_dir.display()))?;
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
    write_pre_push_verifier_manifest(&hooks_dir, &tools_dir, &root)
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "installed pinned pre-push verifier at {} (outside the checked-out tree); pinned tools at {}; manifest protocol v{PRE_PUSH_VERIFIER_PROTOCOL_VERSION}",
        dest.display(),
        tools_dir.display()
    ))
}

/// Resolve the hooks dir, preserving a LOCAL `core.hooksPath`. Refuses when the configured path
/// points INSIDE the checked-out tree (a branch-controlled hook location) or comes from global/
/// system scope (a shared directory that would run this verifier for every repo). The git
/// metadata dir (`.git/`) is never part of the checked-out tree, so a configured path inside it
/// (e.g. an explicit `core.hooksPath` equal to git's default) is accepted.
fn resolve_hooks_dir(repo_root: &Path) -> Result<PathBuf, String> {
    let local = git_capture_optional(
        Some(repo_root),
        &["config", "--type=path", "--local", "--get", "core.hooksPath"],
    )?;
    match local {
        Some(existing) if !existing.trim().is_empty() => {
            let existing = PathBuf::from(existing.trim());
            let existing = absolutize(repo_root, &existing);
            let git_common_dir = resolve_git_common_dir(repo_root)?;
            if inside_checked_out_tree(repo_root, &git_common_dir, &existing) {
                return Err(format!(
                    "refusing to install: core.hooksPath ({}) points inside the checked-out tree; a branch-controlled hook would execute with your privileges — point it outside the worktree or unset it, then re-run install",
                    existing.display()
                ));
            }
            Ok(existing)
        }
        _ => {
            // No LOCAL hooks path. A global/system one would be a shared directory — installing
            // there would execute this repository's verifier for every repository the user owns.
            let any_scope = git_capture_optional(
                Some(repo_root),
                &["config", "--type=path", "--get", "core.hooksPath"],
            )?;
            if let Some(existing) = any_scope.filter(|value| !value.trim().is_empty()) {
                return Err(format!(
                    "refusing to install: core.hooksPath is configured at global/system scope ({}) and points into a shared hooks directory; installing there would run this verifier for every repository you own — set it locally (git config --local core.hooksPath <dir>) or unset it, then re-run install",
                    existing.trim()
                ));
            }
            // No configured path anywhere: use git's default hooks dir and leave configuration alone.
            let git_common_dir = resolve_git_common_dir(repo_root)?;
            Ok(git_common_dir.join("hooks"))
        }
    }
}

fn resolve_git_common_dir(repo_root: &Path) -> Result<PathBuf, String> {
    let raw = git_capture(Some(repo_root), &["rev-parse", "--git-common-dir"])?;
    Ok(absolutize(repo_root, &Path::new(&raw)))
}

fn absolutize(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

/// True when `path` lies under the worktree root but NOT under the git metadata dir — i.e.
/// inside the tree git checks out, where a contribution could replace the hook file.
fn inside_checked_out_tree(repo_root: &Path, git_common_dir: &Path, path: &Path) -> bool {
    path.starts_with(repo_root) && !path.starts_with(git_common_dir)
}

fn files_equal(left: &Path, right: &Path) -> Result<bool, String> {
    let left_bytes = std::fs::read(left)
        .map_err(|error| format!("read {}: {error}", left.display()))?;
    let right_bytes = std::fs::read(right)
        .map_err(|error| format!("read {}: {error}", right.display()))?;
    Ok(left_bytes == right_bytes)
}

fn git_capture_optional(cwd: Option<&Path>, args: &[&str]) -> Result<Option<String>, String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command
        .output()
        .map_err(|error| format!("run git {args:?}: {error}"))?;
    if !output.status.success() {
        // `git config --get <key>` exits 1 when the key is absent: treat as None.
        return Ok(None);
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok(if value.is_empty() { None } else { Some(value) })
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
