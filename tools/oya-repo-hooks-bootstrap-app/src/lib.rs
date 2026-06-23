//! # oya-repo-hooks-bootstrap-app (task #125 v1, ADR-0572)
//!
//! The hermetic, zero-manual-step activation companion for `oya-faces-merge-driver`.
//!
//! `.gitattributes <faces-glob> merge=oya-faces` NAMES a merge driver; it does NOT DEFINE one. A
//! driver is active only if `merge.oya-faces.driver` exists in *that clone's* git config — per-clone
//! local state the repo cannot carry, and absent on fresh CI runners. (The same gap leaves the
//! existing cargo-lock / friction-ledger drivers un-activated: their READMEs document a manual
//! `git config` line — exactly the trap this binary fixes.) The founder bar is AUTOMATED, not
//! flag-only, so activation must be a binary, not a README line.
//!
//! This binary, given a repo root + the built driver binary path, IDEMPOTENTLY:
//!   1. writes `git config --local merge.oya-faces.name` + `merge.oya-faces.driver "<abs> driver
//!      %O %A %B %P"`,
//!   2. sets `git config --local core.hooksPath .githooks`,
//!   3. installs `.githooks/{post-merge,post-rewrite,post-checkout}` shims (the post-merge/
//!      post-rewrite ones run `oya-faces-merge-driver settle`; post-checkout re-runs this bootstrap).
//!
//! Re-running is a no-op (idempotent): a config already at the target value / a hook already present
//! with the expected content is not rewritten.
//!
//! ## Universality (policy-as-data)
//! The faces-glob set comes from `registry/generated-artifact-control-plane.json` (via the
//! re-exported `oya_faces_merge_driver_app::ControlPlane`), so `.gitattributes` cannot drift from the
//! declared policy. The driver name + attribute are the shared `MERGE_ATTRIBUTE` constant.
//!
//! ## Irreducible-glue ledger (ADR-0523)
//! The `git config --local` writes are subprocesses (`git` is the config store; there is no pure-Rust
//! equivalent). The `.githooks` shims are 2-line POSIX shells that invoke the Rust binaries (git
//! requires hooks be executables it spawns). Both are ledgered as the minimal git-integration glue —
//! no other shell. The Rust binary IS the auto-fix; the shim is the git-mandated entrypoint.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_faces_merge_driver_app::{ControlPlane, MERGE_ATTRIBUTE};

/// The git-config merge driver human-readable name.
const DRIVER_NAME: &str = "Oyatie born-accounting faces regenerate-on-merge driver";

/// The checked-in hooks directory git's `core.hooksPath` points at (repo-relative).
pub const HOOKS_DIR: &str = ".githooks";

/// The repo-relative `.gitattributes` the glob lines live in.
pub const GITATTRIBUTES_PATH: &str = ".gitattributes";

/// The marker comment that delimits the generated faces-glob block in `.gitattributes`, so the block
/// can be regenerated in place without disturbing the hand-authored lines around it.
const GITATTRIBUTES_BLOCK_BEGIN: &str = "# BEGIN oya-faces merge driver glob (generated from control-plane; do not hand-edit)";
const GITATTRIBUTES_BLOCK_END: &str = "# END oya-faces merge driver glob";

/// A bootstrap failure. Fail LOUD: every variant carries a diagnosable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapError {
    message: String,
}

impl BootstrapError {
    /// Build a bootstrap error with a diagnostic `message`. Public so the binary entrypoint can
    /// construct usage/argument errors.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for BootstrapError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for BootstrapError {}

impl From<oya_faces_merge_driver_app::FacesMergeError> for BootstrapError {
    fn from(e: oya_faces_merge_driver_app::FacesMergeError) -> Self {
        BootstrapError::new(format!("control-plane: {e}"))
    }
}

/// What the idempotent bootstrap changed (empty fields == already correct, a no-op re-run).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BootstrapReport {
    /// True iff a `git config --local merge.oya-faces.*` value was written (false == already set).
    pub merge_driver_configured: bool,
    /// True iff `core.hooksPath` was written (false == already `.githooks`).
    pub hooks_path_configured: bool,
    /// The hook shim file names written/updated (empty == all already present + current).
    pub hooks_installed: Vec<String>,
    /// True iff the `.gitattributes` generated glob block was written/updated (false == already current).
    pub gitattributes_updated: bool,
}

impl BootstrapReport {
    /// True iff nothing was changed (a fully idempotent no-op re-run).
    #[must_use]
    pub fn is_noop(&self) -> bool {
        !self.merge_driver_configured
            && !self.hooks_path_configured
            && self.hooks_installed.is_empty()
            && !self.gitattributes_updated
    }
}

/// Idempotently activate the faces merge driver + hooks in the clone at `repo_root`, pointing the
/// driver at `driver_bin` (the absolute path to the built `oya-faces-merge-driver`).
///
/// Steps (each idempotent — only writes when the current value differs from the target):
/// 1. `.gitattributes` generated glob block (from the control-plane declared faces).
/// 2. `git config --local merge.oya-faces.{name,driver}`.
/// 3. `git config --local core.hooksPath .githooks`.
/// 4. `.githooks/{post-merge,post-rewrite,post-checkout}` shims.
///
/// # Errors
/// [`BootstrapError`] on a control-plane load failure, a `git config` failure, or an IO failure.
pub fn bootstrap(repo_root: &Path, driver_bin: &Path) -> Result<BootstrapReport, BootstrapError> {
    let control_plane = ControlPlane::load(repo_root)?;
    let mut report = BootstrapReport::default();

    // 1. .gitattributes glob block (universality: derived from the control-plane).
    report.gitattributes_updated = ensure_gitattributes(repo_root, &control_plane)?;

    // 2. merge.oya-faces.{name,driver}. The driver value embeds the absolute binary path + the
    //    `driver %O %A %B %P` subcommand contract.
    let driver_value = format!(
        "{} driver %O %A %B %P",
        driver_bin
            .to_str()
            .ok_or_else(|| BootstrapError::new("driver binary path is not valid UTF-8"))?
    );
    let name_changed = ensure_git_config(
        repo_root,
        &format!("merge.{MERGE_ATTRIBUTE}.name"),
        DRIVER_NAME,
    )?;
    let driver_changed = ensure_git_config(
        repo_root,
        &format!("merge.{MERGE_ATTRIBUTE}.driver"),
        &driver_value,
    )?;
    report.merge_driver_configured = name_changed || driver_changed;

    // 3. core.hooksPath -> .githooks.
    report.hooks_path_configured = ensure_git_config(repo_root, "core.hooksPath", HOOKS_DIR)?;

    // 4. The hook shims.
    report.hooks_installed = ensure_hooks(repo_root)?;

    Ok(report)
}

/// The generated `.gitattributes` glob block for the SETTLE-CAPABLE faces (one exact line per face
/// path, sorted — exact paths cannot drift from the control-plane and avoid wildcard over-match).
/// Scoped to `settle_capable_face_paths()` (NOT the full declared regeneratable set) so the
/// `merge=oya-faces` attribute is only attached to faces the driver+settle can authoritatively
/// re-derive — never a controller-materialized projection the local settle cannot produce
/// (fail-closed). Public so the `gitattributes --check` subcommand can compare against the on-disk file.
#[must_use]
pub fn render_gitattributes_block(control_plane: &ControlPlane) -> String {
    let mut out = String::new();
    out.push_str(GITATTRIBUTES_BLOCK_BEGIN);
    out.push('\n');
    for path in control_plane.settle_capable_face_paths() {
        // Only faces the settle authoritatively regenerates get the regenerate-on-merge attribute.
        out.push_str(&path);
        out.push(' ');
        out.push_str(&format!("merge={MERGE_ATTRIBUTE}"));
        out.push('\n');
    }
    out.push_str(GITATTRIBUTES_BLOCK_END);
    out.push('\n');
    out
}

/// Ensure the `.gitattributes` generated block matches `render_gitattributes_block`, regenerating it
/// in place between the BEGIN/END markers (or appending the block if absent). Returns true iff the
/// file was changed. Idempotent.
fn ensure_gitattributes(
    repo_root: &Path,
    control_plane: &ControlPlane,
) -> Result<bool, BootstrapError> {
    let path = repo_root.join(GITATTRIBUTES_PATH);
    let desired_block = render_gitattributes_block(control_plane);
    let existing = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(BootstrapError::new(format!(
                "read {}: {e}",
                path.display()
            )));
        }
    };

    let updated = replace_block(&existing, &desired_block);
    if updated == existing {
        return Ok(false);
    }
    std::fs::write(&path, &updated)
        .map_err(|e| BootstrapError::new(format!("write {}: {e}", path.display())))?;
    Ok(true)
}

/// Replace the BEGIN..END block in `existing` with `desired_block`, or append it (separated by a
/// blank line) if no block exists. Pure (no IO) so it is unit-testable.
fn replace_block(existing: &str, desired_block: &str) -> String {
    if let (Some(begin), Some(end_line_start)) = (
        existing.find(GITATTRIBUTES_BLOCK_BEGIN),
        existing.find(GITATTRIBUTES_BLOCK_END),
    ) {
        // Extend the end to cover the END marker line + its trailing newline.
        let after_end = existing[end_line_start..]
            .find('\n')
            .map(|offset| end_line_start + offset + 1)
            .unwrap_or(existing.len());
        let mut out = String::with_capacity(existing.len());
        out.push_str(&existing[..begin]);
        out.push_str(desired_block);
        out.push_str(&existing[after_end..]);
        return out;
    }
    // No block yet: append. Ensure exactly one blank line before the block when there is prior text.
    let mut out = existing.to_owned();
    if !out.is_empty() {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(desired_block);
    out
}

/// Set `key` to `value` via `git config --local`, but ONLY if the current value differs (idempotent).
/// Returns true iff a write happened. Fail LOUD on a `git` failure.
fn ensure_git_config(repo_root: &Path, key: &str, value: &str) -> Result<bool, BootstrapError> {
    if current_git_config(repo_root, key)?.as_deref() == Some(value) {
        return Ok(false);
    }
    run_git(repo_root, &["config", "--local", key, value], &format!("git config {key}"))?;
    Ok(true)
}

/// Read `git config --local --get <key>`, returning `None` when unset. A non-zero exit with empty
/// stderr is the "unset" signal (git exit code 1); any other failure is fatal.
fn current_git_config(repo_root: &Path, key: &str) -> Result<Option<String>, BootstrapError> {
    let output = Command::new("git")
        .args(["config", "--local", "--get", key])
        .current_dir(repo_root)
        .output()
        .map_err(|e| BootstrapError::new(format!("git config --get {key}: {e}")))?;
    if output.status.success() {
        let value = String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_owned();
        Ok(Some(value))
    } else if output.status.code() == Some(1) {
        // Exit 1 = the key is not set (the documented git-config "value not found" code).
        Ok(None)
    } else {
        Err(BootstrapError::new(format!(
            "git config --get {key} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

fn run_git(repo_root: &Path, args: &[&str], context: &str) -> Result<(), BootstrapError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| BootstrapError::new(format!("{context}: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(BootstrapError::new(format!(
            "{context} failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// The hook shims to install. Each is a 2-line POSIX shell that runs a Rust binary. post-merge +
/// post-rewrite run the authoritative settle; post-checkout re-runs this bootstrap (idempotent), so a
/// `git checkout` keeps the activation fresh after the first run.
fn hook_shims() -> Vec<(&'static str, String)> {
    let settle = format!(
        "#!/bin/sh\n# Generated by oya-repo-hooks-bootstrap ({}). Settle born-accounting faces from\n\
         # the committed merged tree (authoritative; the per-file driver is cosmetic). Fail-closed.\n\
         exec oya-faces-merge-driver settle\n",
        MERGE_ATTRIBUTE
    );
    let checkout = format!(
        "#!/bin/sh\n# Generated by oya-repo-hooks-bootstrap ({}). Re-run the idempotent bootstrap so\n\
         # the merge driver + hooks stay activated after a checkout. Best-effort (never block checkout).\n\
         oya-repo-hooks-bootstrap >/dev/null 2>&1 || true\n",
        MERGE_ATTRIBUTE
    );
    vec![
        ("post-merge", settle.clone()),
        // post-rewrite receives a command name ($1) on stdin args; the settle is the same.
        ("post-rewrite", settle),
        ("post-checkout", checkout),
    ]
}

/// Install/update the hook shims under `<repo_root>/.githooks`, returning the file names actually
/// written (empty == all already present + current). Idempotent: a hook already at the expected
/// content is not rewritten. The shims are made executable (git spawns hooks).
fn ensure_hooks(repo_root: &Path) -> Result<Vec<String>, BootstrapError> {
    let hooks_dir = repo_root.join(HOOKS_DIR);
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|e| BootstrapError::new(format!("create {}: {e}", hooks_dir.display())))?;
    let mut written = Vec::new();
    for (name, content) in hook_shims() {
        let path = hooks_dir.join(name);
        let current = std::fs::read_to_string(&path).ok();
        if current.as_deref() == Some(content.as_str()) && is_executable(&path) {
            continue;
        }
        std::fs::write(&path, &content)
            .map_err(|e| BootstrapError::new(format!("write {}: {e}", path.display())))?;
        set_executable(&path)?;
        written.push(name.to_owned());
    }
    written.sort();
    Ok(written)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.exists()
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), BootstrapError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)
        .map_err(|e| BootstrapError::new(format!("stat {}: {e}", path.display())))?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms)
        .map_err(|e| BootstrapError::new(format!("chmod {}: {e}", path.display())))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), BootstrapError> {
    Ok(())
}

/// Resolve the built driver binary path for the `bootstrap` subcommand. Resolution order:
///   1. `--driver-bin <path>` (explicit, e.g. the buck2 `$(location)` the CI step injects);
///   2. `OYA_FACES_MERGE_DRIVER` env (the build-time-injected path, mirroring the cli-fixtures
///      precedent);
///   3. the bare name `oya-faces-merge-driver` (resolved from `PATH` by git at merge time).
///
/// Returns the path to embed in `merge.oya-faces.driver`. Never fails — option 3 is the always-valid
/// fallback (git resolves it from PATH, the same way the existing driver READMEs assume).
#[must_use]
pub fn resolve_driver_bin(explicit: Option<&str>) -> PathBuf {
    if let Some(path) = explicit {
        return PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("OYA_FACES_MERGE_DRIVER") {
        return PathBuf::from(path);
    }
    PathBuf::from("oya-faces-merge-driver")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn control_plane() -> ControlPlane {
        // Both faces use the accounting producer target so they are SETTLE-CAPABLE (the glob block is
        // derived from settle-capable faces, not the broader regeneratable set).
        ControlPlane::from_manifest(&json!({
            "artifacts": [
                {
                    "artifact_id": "b",
                    "path": "out/b.generated.json",
                    "merge_policy": "never-manual-merge-regenerate-from-source-tree",
                    "generator": { "generator_target": oya_faces_merge_driver_app::PRODUCER_TARGET }
                },
                {
                    "artifact_id": "a",
                    "path": "out/a.generated.json",
                    "merge_policy": "never-manual-merge-regenerate-from-source-tree",
                    "generator": { "generator_target": oya_faces_merge_driver_app::EMITTER_TARGET }
                }
            ]
        }))
        .expect("parse control plane")
    }

    #[test]
    fn gitattributes_block_is_sorted_and_attribute_tagged() {
        let block = render_gitattributes_block(&control_plane());
        let lines: Vec<&str> = block.lines().collect();
        assert_eq!(lines[0], GITATTRIBUTES_BLOCK_BEGIN);
        assert_eq!(lines[1], "out/a.generated.json merge=oya-faces");
        assert_eq!(lines[2], "out/b.generated.json merge=oya-faces");
        assert_eq!(lines[3], GITATTRIBUTES_BLOCK_END);
    }

    #[test]
    fn replace_block_appends_when_absent() {
        let existing = "Cargo.lock merge=cargo-lock\n";
        let block = render_gitattributes_block(&control_plane());
        let out = replace_block(existing, &block);
        assert!(out.starts_with("Cargo.lock merge=cargo-lock\n\n"));
        assert!(out.contains("out/a.generated.json merge=oya-faces"));
    }

    #[test]
    fn replace_block_is_idempotent_in_place() {
        let block = render_gitattributes_block(&control_plane());
        let existing = format!("# header\nCargo.lock merge=cargo-lock\n\n{block}# trailer\n");
        let once = replace_block(&existing, &block);
        let twice = replace_block(&once, &block);
        assert_eq!(once, twice, "replacing the block again is a no-op");
        assert!(once.contains("# header"));
        assert!(once.contains("# trailer"));
    }

    #[test]
    fn replace_block_updates_in_place_when_faces_change() {
        let old_block = format!(
            "{GITATTRIBUTES_BLOCK_BEGIN}\nout/old.generated.json merge=oya-faces\n{GITATTRIBUTES_BLOCK_END}\n"
        );
        let existing = format!("Cargo.lock merge=cargo-lock\n\n{old_block}");
        let new_block = render_gitattributes_block(&control_plane());
        let out = replace_block(&existing, &new_block);
        assert!(!out.contains("out/old.generated.json"), "stale glob removed");
        assert!(out.contains("out/a.generated.json merge=oya-faces"), "new glob present");
        assert!(out.starts_with("Cargo.lock merge=cargo-lock\n"), "prior lines preserved");
    }

    #[test]
    fn resolve_driver_bin_prefers_explicit_then_env_then_path() {
        assert_eq!(
            resolve_driver_bin(Some("/abs/driver")),
            PathBuf::from("/abs/driver")
        );
        // No explicit, no env set in this test -> bare name from PATH.
        assert_eq!(
            resolve_driver_bin(None),
            PathBuf::from("oya-faces-merge-driver")
        );
    }

    #[test]
    fn hook_shims_settle_and_rebootstrap() {
        let shims = hook_shims();
        let names: Vec<&str> = shims.iter().map(|(n, _)| *n).collect();
        assert_eq!(names, vec!["post-merge", "post-rewrite", "post-checkout"]);
        assert!(shims[0].1.contains("oya-faces-merge-driver settle"));
        assert!(shims[2].1.contains("oya-repo-hooks-bootstrap"));
    }
}
