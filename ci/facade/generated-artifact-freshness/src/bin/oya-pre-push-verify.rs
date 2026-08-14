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
//! buck2 run //ci/facade/generated-artifact-freshness:oya-pre-push-verify-bin -- reconcile
//! ```

#![forbid(unsafe_code)]

use std::env;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use ci_generated_artifact_freshness::{
    FaceSettleMode, PRE_PUSH_GENERATION_DIR_PREFIX, PRE_PUSH_VERIFIER_MANIFEST_FILE,
    PRE_PUSH_VERIFIER_PROTOCOL_VERSION, PRE_PUSH_VERIFIER_TOOLS_DIR,
    any_generation_manifest_owns_hook, assert_committed_tree_clean, fnv1a64_hex,
    install_pre_push_verifier_tools, read_pre_push_verifier_manifest,
    read_pre_push_verifier_wiring, run_face_settle_with_pinned_tools,
    verify_pre_push_verifier_protocol, write_pre_push_verifier_manifest,
};

const ZERO_SHA: &str = "0000000000000000000000000000000000000000";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if is_reconcile_invocation(&args) {
        return match reconcile(&args[1..]) {
            Ok(message) => {
                println!("{message}");
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("pre-push reconcile: {message}");
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

/// True only for an EXPLICIT reconciler invocation (`reconcile`, optionally with `--repo-root` or
/// `--help`). Git invokes the pre-push hook as `pre-push <remote-name> <remote-url>`; when the
/// remote is literally named `reconcile` the first argument is the remote name, not the
/// subcommand, so the subcommand must not be guessed from argv[0] alone — a two-argument
/// invocation whose second argument is not a recognized flag is a hook invocation, not the
/// reconciler, and must not enter reconciler mode (which would reject the URL and permanently
/// block pushes to that remote).
fn is_reconcile_invocation(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("reconcile") {
        return false;
    }
    match &args[1..] {
        [] => true,
        [flag] if flag == "--help" || flag == "-h" => true,
        [flag, _value] if flag == "--repo-root" => true,
        // `reconcile <url>`: a push to a remote literally named `reconcile`.
        _ => false,
    }
}

/// Hook mode: read the refs git passes on stdin, gate the push, and run the verify with
/// the pinned tools installed alongside this binary.
fn run_hook() -> Result<ExitCode, String> {
    let pushed_commits = read_pushed_face_commits()?;
    if pushed_commits.is_empty() {
        // Deletion-only push (branch or tag deletion): no face-relevant commit is introduced.
        return Ok(ExitCode::SUCCESS);
    }

    // The verify certifies the COMMITTED tree (HEAD) only. Every pushed non-deletion commit
    // (branch head or tag-peeled commit) must equal HEAD, otherwise fail closed: a settled
    // checkout must not let a stale `topic` commit pass just because a different branch is
    // checked out — and a tag pushed from a different commit is equally uncertified.
    let head = head_sha()?;
    for sha in &pushed_commits {
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
    let exe = env::current_exe().map_err(|error| format!("resolve current executable: {error}"))?;
    let hooks_dir = exe
        .parent()
        .ok_or_else(|| "installed hook has no parent directory".to_owned())?
        .to_path_buf();
    // Per-worktree generation: linked worktrees share git's common-dir hooks directory, so the
    // pinned tools + manifest for THIS checkout live under a worktree-keyed subdirectory. Two
    // worktrees at different generator sources can no longer clobber each other's sole
    // generation; each push dispatches to the generation matching its own checkout.
    let generation_dir = hooks_dir.join(format!(
        "{PRE_PUSH_GENERATION_DIR_PREFIX}{}",
        pre_push_generation_key(&root)?
    ));
    let tools_dir = generation_dir.join(PRE_PUSH_VERIFIER_TOOLS_DIR);

    // Verify certifies the COMMITTED tree (HEAD) only. Assert cleanliness BEFORE the manifest
    // handshake so a dirty generator source fails with the tree-clean remediation ("commit or
    // remove these changes first") instead of a misleading "generator source changed — reinstall":
    // reinstalling pins tools built from the SAME dirty tree and still fails, whereas the real
    // contract is that the working tree must match HEAD.
    assert_committed_tree_clean(&root).map_err(|error| error.to_string())?;

    // Protocol handshake (fail closed with explicit reinstall requirement): the install
    // manifest must exist, match this binary's embedded protocol version, bind to the
    // pinned tool builds actually installed next to the hook, AND bind to the repository's
    // generator source the pinned tools were built from — so a generator change (even one
    // that leaves the Buck label and protocol integer untouched) fails closed instead of
    // silently verifying with stale tools.
    let _ = read_pre_push_verifier_manifest(&generation_dir, &tools_dir, &root)
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

/// Parse git's pre-push stdin lines (`<local ref> <local sha> <remote ref> <remote sha>`) and
/// return the local COMMIT of every non-deletion branch or tag push. Tag objects are peeled to
/// their commit (`^{commit}`) so the HEAD-equality certification applies to the commits a tag
/// introduces — a tag-only push can no longer skip verification.
fn read_pushed_face_commits() -> Result<Vec<String>, String> {
    read_pushed_face_commits_from(io::stdin().lock())
}

/// Parse git's pre-push stdin lines and return the local COMMIT of every non-deletion branch or
/// tag push (see [`read_pushed_face_commits`]); split out so the protocol surface is unit-testable
/// with an injected reader.
fn read_pushed_face_commits_from(mut reader: impl BufRead) -> Result<Vec<String>, String> {
    let mut commits = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|error| format!("read pre-push stdin: {error}"))?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 4 {
            return Err(format!("malformed pre-push line: {line:?}"));
        }
        let local_sha = fields[1];
        let remote_ref = fields[2];
        let is_face_ref =
            remote_ref.starts_with("refs/heads/") || remote_ref.starts_with("refs/tags/");
        if is_face_ref && local_sha != ZERO_SHA {
            if remote_ref.starts_with("refs/tags/") {
                commits.push(peel_commit(local_sha)?);
            } else {
                commits.push(local_sha.to_owned());
            }
        }
    }
    Ok(commits)
}

/// Peel an annotated tag object to its commit so the HEAD-equality certification applies to the
/// commit the tag introduces (lightweight tags already point at a commit).
fn peel_commit(sha: &str) -> Result<String, String> {
    let arg = format!("{sha}^{{commit}}");
    git_capture(None, &["rev-parse", arg.as_str()])
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

/// Reconciler mode: converge the INSTALLED hook state (pinned generator tools built once from this
/// checkout, the verifier copy in the git hooks dir OUTSIDE the checked-out tree preserving any
/// existing `core.hooksPath`, and the protocol manifest) toward the DECLARED wiring state in the
/// repository (`tools/hooks/pre-push-verifier.wiring.json`). Declarative-state-driven and
/// idempotent: the reconciler fails closed when the binary disagrees with the declaration (declared
/// hook name, protocol, pinned tools, or generator source dirs drift) instead of installing stale.
fn reconcile(args: &[String]) -> Result<String, String> {
    let mut cli_repo_root: Option<PathBuf> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err("reconcile: --repo-root requires a path".to_owned());
                };
                cli_repo_root = Some(PathBuf::from(value));
            }
            "--help" | "-h" => {
                return Err("usage: oya-pre-push-verify reconcile [--repo-root <path>]".to_owned());
            }
            other => {
                return Err(format!(
                    "reconcile: unknown argument {other:?}; usage: oya-pre-push-verify reconcile [--repo-root <path>]"
                ));
            }
        }
    }
    let root = match cli_repo_root {
        Some(root) => root,
        None => repo_root()?,
    };

    // The reconciler converges toward the repository's DECLARED wiring state and fails closed when
    // this binary disagrees with it (declared hook name / protocol / pinned tools / source dirs).
    read_pre_push_verifier_wiring(&root).map_err(|error| error.to_string())?;

    // Preserve an existing configured hooks path (org-managed commit-msg/signing/security
    // hooks): install INTO that same directory and never rewrite core.hooksPath. Only when
    // no LOCAL path is configured do we fall back to git's default hooks dir, and we do NOT
    // set core.hooksPath then either (git already resolves the default). A hooks path
    // configured at global/system scope is refused: installing into a shared directory would
    // run this repository's verifier for every repository owned by the user.
    let hooks_dir = resolve_hooks_dir(&root)?;
    // Per-worktree generation: the pinned tools + manifest for THIS checkout live under a
    // worktree-keyed subdirectory of the (possibly shared) hooks dir, so linked worktrees at
    // different generator sources never replace each other's sole generation.
    let generation_dir = hooks_dir.join(format!(
        "{PRE_PUSH_GENERATION_DIR_PREFIX}{}",
        pre_push_generation_key(&root)?
    ));

    let exe = env::current_exe().map_err(|error| format!("resolve current executable: {error}"))?;
    let dest = hooks_dir.join("pre-push");
    // Replacement permission is bound to the INSTALLED HOOK's identity, not the manifest's mere
    // existence: a manifest records the hook binary's fingerprint, so a manifest left behind after
    // the user swapped in another tool's hook cannot authorize an overwrite of unrelated user hook
    // state. Linked worktrees share this hooks dir, so ownership is checked across EVERY
    // per-worktree generation manifest — a second worktree reconciling over the first's shared
    // `pre-push` must recognize it as our own verifier rather than refusing it as unrelated. A
    // protocol bump builds a byte-different verifier, and the recorded fingerprint matches the
    // CURRENT hook (our own binary), so reinstalls keep working.
    let is_prior_oya_install =
        any_generation_manifest_owns_hook(&hooks_dir, &dest).unwrap_or(false);
    if dest.exists() && !is_prior_oya_install && !files_equal(&dest, &exe)? {
        return Err(format!(
            "refusing to overwrite {}: an unrelated pre-push hook is installed there; move it aside or remove it first (the oya verifier never replaces unrelated user hook state)",
            dest.display()
        ));
    }

    // Pin the generator tools FIRST (built from this trusted install checkout), then copy
    // the verifier, then write the manifest LAST. To make a reinstall ATOMIC (wave-4): the
    // old manifest is invalidated BEFORE any pinned tool is overwritten, so an interruption
    // mid-generation leaves no valid manifest and the hook fails closed with a reinstall
    // requirement instead of executing a mixed old/new tool set against a stale manifest.
    let manifest_path = generation_dir.join(PRE_PUSH_VERIFIER_MANIFEST_FILE);
    let _ = std::fs::remove_file(&manifest_path);
    let tools_dir = generation_dir.join(PRE_PUSH_VERIFIER_TOOLS_DIR);
    install_pre_push_verifier_tools(&root, &tools_dir).map_err(|error| error.to_string())?;

    std::fs::create_dir_all(&generation_dir).map_err(|error| {
        format!(
            "create generation dir {}: {error}",
            generation_dir.display()
        )
    })?;
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
    write_pre_push_verifier_manifest(&generation_dir, &tools_dir, &root, &dest)
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "reconciled pinned pre-push verifier at {} (outside the checked-out tree) toward the declared wiring (tools/hooks/pre-push-verifier.wiring.json); pinned tools at {}; manifest protocol v{PRE_PUSH_VERIFIER_PROTOCOL_VERSION}",
        dest.display(),
        tools_dir.display()
    ))
}

/// Per-worktree generation key: FNV-1a of the canonical worktree root. Linked worktrees of one
/// repository resolve to DIFFERENT canonical roots, so each worktree gets its own pinned-tool
/// generation + manifest instead of all worktrees fighting over one shared mutable installation.
fn pre_push_generation_key(repo_root: &Path) -> Result<String, String> {
    let canonical = std::fs::canonicalize(repo_root)
        .map_err(|error| format!("canonicalize repo root {}: {error}", repo_root.display()))?;
    Ok(fnv1a64_hex(canonical.to_string_lossy().as_bytes()))
}

/// Resolve the hooks dir, preserving a LOCAL or WORKTREE-scoped `core.hooksPath`. Refuses when the
/// configured path points INSIDE the checked-out tree (a branch-controlled hook location) or comes
/// from global/system scope (a shared directory that would run this verifier for every repo). The
/// git metadata dir (`.git/`) is never part of the checked-out tree, so a configured path inside it
/// (e.g. an explicit `core.hooksPath` equal to git's default) is accepted.
fn resolve_hooks_dir(repo_root: &Path) -> Result<PathBuf, String> {
    let local = git_capture_optional(
        Some(repo_root),
        &[
            "config",
            "--type=path",
            "--local",
            "--get",
            "core.hooksPath",
        ],
    )?;
    // With `extensions.worktreeConfig` enabled, `core.hooksPath` can be configured with
    // `git config --worktree`; Git honors that per-worktree value, so it is repository-owned just
    // like `--local`. git_capture_optional returns None when the `--worktree` scope is unavailable
    // (the extension is unset), in which case we fall through exactly as before.
    let worktree = git_capture_optional(
        Some(repo_root),
        &[
            "config",
            "--type=path",
            "--worktree",
            "--get",
            "core.hooksPath",
        ],
    )?;
    // Git config precedence is system → global → local → worktree (later wins): when BOTH scopes
    // are set, git executes hooks from the WORKTREE path, so we must install there too. Preferring
    // `local` here would install into a directory git never reads, silently leaving every push
    // unverified (a fail-open hole).
    let configured = worktree.or(local);
    match configured {
        Some(existing) if !existing.trim().is_empty() => {
            let existing = PathBuf::from(existing.trim());
            let existing = absolutize(repo_root, &existing);
            // Canonicalize so `..` components and symlinks resolve BEFORE the inside-checkout
            // check: a lexical `starts_with` on an unnormalized path (e.g. `<repo>/.git/../tracked-hooks`)
            // could otherwise be misclassified as git metadata and accepted, letting a checkout
            // replace the installed hook. `canonicalize` fails if the directory does not exist
            // yet; hooks dirs are created at install, so require a resolvable existing ancestor.
            let existing = canonicalize_or_ancestor(&existing)?;
            let git_common_dir = resolve_git_common_dir(repo_root)?;
            let git_common_dir = canonicalize_or_ancestor(&git_common_dir)?;
            let repo_root = canonicalize_or_ancestor(repo_root)?;
            if inside_checked_out_tree(&repo_root, &git_common_dir, &existing) {
                return Err(format!(
                    "refusing to install: core.hooksPath ({}) resolves inside the checked-out tree; a branch-controlled hook would execute with your privileges — point it outside the worktree or unset it, then re-run install",
                    existing.display()
                ));
            }
            Ok(existing)
        }
        _ => {
            // No LOCAL or WORKTREE hooks path. A global/system one would be a shared directory —
            // installing there would execute this repository's verifier for every repository the
            // user owns.
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

/// Resolve `path` with `std::fs::canonicalize` when it exists; otherwise walk up to the nearest
/// existing ancestor, canonicalize it, and re-append the missing suffix with `..`/`.` components
/// normalized against the canonical ancestor. This fully normalizes `..` and symlinks so the
/// inside-checked-out-tree classification is based on real directories, never lexical path text:
/// e.g. `<repo>/.git/../tracked-hooks` becomes `<repo>/tracked-hooks`, which is correctly seen
/// as checkout-controlled.
fn canonicalize_or_ancestor(path: &Path) -> Result<PathBuf, String> {
    let mut current = path.to_path_buf();
    let mut missing: Vec<std::ffi::OsString> = Vec::new();
    while !current.exists() {
        let Some(name) = current.file_name() else {
            return Err(format!(
                "cannot canonicalize {}: no existing ancestor",
                path.display()
            ));
        };
        missing.push(name.to_os_string());
        if !current.pop() {
            return Err(format!(
                "cannot canonicalize {}: no existing ancestor",
                path.display()
            ));
        }
    }
    let mut canonical = std::fs::canonicalize(&current)
        .map_err(|error| format!("canonicalize {}: {error}", current.display()))?;
    // Re-apply the missing suffix, resolving `..` (and dropping `.`) against the canonical
    // ancestor so the final path contains no traversal components.
    for component in missing.iter().rev() {
        if component == ".." {
            if !canonical.pop() {
                return Err(format!(
                    "cannot canonicalize {}: traversal above filesystem root",
                    path.display()
                ));
            }
        } else if component != "." {
            canonical.push(component);
        }
    }
    Ok(canonical)
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
    let left_bytes =
        std::fs::read(left).map_err(|error| format!("read {}: {error}", left.display()))?;
    let right_bytes =
        std::fs::read(right).map_err(|error| format!("read {}: {error}", right.display()))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn run_git(root: &Path, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("run git")
    }

    #[test]
    fn is_reconcile_invocation_disambiguates_remote_named_reconcile() {
        assert!(is_reconcile_invocation(&["reconcile".to_owned()]));
        assert!(is_reconcile_invocation(&[
            "reconcile".to_owned(),
            "--repo-root".to_owned(),
            ".".to_owned(),
        ]));
        assert!(is_reconcile_invocation(&[
            "reconcile".to_owned(),
            "--help".to_owned(),
        ]));
        // A push to a remote literally named `reconcile` is a HOOK invocation, never the reconciler:
        // guessing the subcommand from argv[0] alone would reject the URL and block pushes forever.
        assert!(!is_reconcile_invocation(&[
            "reconcile".to_owned(),
            "https://example.com/repo.git".to_owned(),
        ]));
        assert!(!is_reconcile_invocation(&["not-reconcile".to_owned()]));
        assert!(!is_reconcile_invocation(&[]));
    }

    #[test]
    fn pre_push_protocol_parses_branches_skips_deletions_and_non_face_refs() {
        let input = concat!(
            "refs/heads/topic aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa refs/heads/topic bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n",
            "refs/heads/topic 0000000000000000000000000000000000000000 refs/heads/topic bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n",
            "refs/tags/v1 0000000000000000000000000000000000000000 refs/tags/v1 0000000000000000000000000000000000000000\n",
            "refs/notes/commits aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa refs/notes/commits bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n",
        );
        let commits =
            read_pushed_face_commits_from(Cursor::new(input.as_bytes())).expect("parse stdin");
        assert_eq!(
            commits,
            vec!["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()],
            "only the non-deletion branch push is a face-relevant commit"
        );
    }

    #[test]
    fn pre_push_protocol_rejects_malformed_line() {
        let input = "refs/heads/topic aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa refs/heads/topic\n";
        let error = read_pushed_face_commits_from(Cursor::new(input.as_bytes()))
            .expect_err("three fields must fail closed");
        assert!(error.contains("malformed"), "{error}");
    }

    #[test]
    fn resolve_hooks_dir_prefers_worktree_scope_over_local_scope() {
        // Git config precedence is system → global → local → worktree (later wins). When both
        // `--local` and `--worktree` set `core.hooksPath`, git executes hooks from the worktree
        // path; the installer must target that same directory or the hook silently never runs.
        let base = std::env::temp_dir().join(format!(
            "oya-pre-push-hooks-precedence-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos(),
        ));
        let repo = base.join("repo");
        std::fs::create_dir_all(&repo).expect("create repo");
        if !run_git(&repo, &["init", "-q"]).status.success()
            || !run_git(&repo, &["config", "extensions.worktreeConfig", "true"])
                .status
                .success()
        {
            // Worktree-scoped config unsupported on this git: nothing to assert, skip.
            let _ = std::fs::remove_dir_all(&base);
            return;
        }
        let local_dir = base.join("local-hooks");
        let worktree_dir = base.join("worktree-hooks");
        std::fs::create_dir_all(&local_dir).expect("create local hooks dir");
        std::fs::create_dir_all(&worktree_dir).expect("create worktree hooks dir");
        let set_local = run_git(
            &repo,
            &[
                "config",
                "--local",
                "core.hooksPath",
                local_dir.to_str().expect("local utf-8"),
            ],
        );
        assert!(set_local.status.success(), "set local core.hooksPath");
        let set_worktree = run_git(
            &repo,
            &[
                "config",
                "--worktree",
                "core.hooksPath",
                worktree_dir.to_str().expect("worktree utf-8"),
            ],
        );
        assert!(set_worktree.status.success(), "set worktree core.hooksPath");

        let resolved = resolve_hooks_dir(&repo).expect("resolve hooks dir");
        assert_eq!(
            resolved,
            std::fs::canonicalize(&worktree_dir).expect("canonical worktree hooks dir"),
            "worktree-scoped core.hooksPath must win over local scope"
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
