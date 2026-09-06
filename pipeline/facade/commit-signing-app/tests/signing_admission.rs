//! The binary refuses what the protected ruleset refuses, and says which.
//!
//! These drive a throwaway repository rather than this one, because the cases
//! that matter cannot be staged in a repository whose history is already
//! correct: an unsigned commit and a commit signed by a key the authority does
//! not name.

use std::path::Path;
use std::process::{Command, Output};

fn git(repository: &Path, args: &[&str]) -> Output {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .env("GIT_AUTHOR_NAME", "Probe")
        .env("GIT_AUTHOR_EMAIL", "probe@example.invalid")
        .env("GIT_COMMITTER_NAME", "Probe")
        .env("GIT_COMMITTER_EMAIL", "probe@example.invalid")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// A repository with one empty commit as a base, returning that base's sha.
fn repository(case: &str) -> (std::path::PathBuf, String) {
    let root = std::env::temp_dir().join(format!("oyatie-signing-{case}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch repository");
    git(&root, &["init", "-q", "-b", "main"]);
    // The ambient user may sign by default. An "unsigned" fixture that
    // inherited a real signing key would be admitted, and the test asserting
    // it is refused would fail for a reason that looks like a gate defect.
    git(&root, &["config", "commit.gpgsign", "false"]);
    git(&root, &["commit", "-q", "--allow-empty", "-m", "base"]);
    let base = String::from_utf8_lossy(&git(&root, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
    (root, base)
}

fn admit(repository: &Path, base: &str, head: &str) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_pipeline-commit-signing-app"))
        .args([base, head])
        .current_dir(repository)
        .output()
        .expect("the admission binary runs");
    let mut said = String::from_utf8_lossy(&output.stderr).into_owned();
    said.push_str(&String::from_utf8_lossy(&output.stdout));
    (output.status.success(), said)
}

#[test]
fn an_unsigned_commit_is_refused_and_named() {
    let (root, base) = repository("unsigned");
    git(
        &root,
        &["commit", "-q", "--allow-empty", "-m", "unsigned work"],
    );
    let head = String::from_utf8_lossy(&git(&root, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();

    let (admitted, report) = admit(&root, &base, &head);

    assert!(
        !admitted,
        "an unsigned commit must not be admitted: {report}"
    );
    assert!(
        report.contains(&head[..9]),
        "the refusal must name the commit: {report}"
    );
    assert!(report.contains("unsigned"), "{report}");
}

/// THE TRAP. A signature made by a key the authority does not name still makes
/// `git verify-commit` print `Good "git" signature` — it only fails on the
/// separate `No principal matched` line and the exit code. A gate that read
/// the word "Good" would admit precisely the commits this one exists to
/// refuse, so the case is pinned with a real signature from a real key that
/// really is not trusted.
#[test]
fn a_signature_from_an_unenrolled_key_is_refused_as_untrusted_not_admitted() {
    let (root, base) = repository("untrusted");
    let key = root.join("intruder");
    assert!(
        Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "intruder", "-f"])
            .arg(&key)
            .status()
            .expect("ssh-keygen runs")
            .success(),
        "the probe key must be generated"
    );
    git(&root, &["config", "gpg.format", "ssh"]);
    git(
        &root,
        &[
            "config",
            "user.signingkey",
            &format!("{}.pub", key.display()),
        ],
    );
    git(
        &root,
        &[
            "commit",
            "-q",
            "-S",
            "--allow-empty",
            "-m",
            "signed by an intruder",
        ],
    );
    let head = String::from_utf8_lossy(&git(&root, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();

    let (admitted, report) = admit(&root, &base, &head);

    assert!(
        !admitted,
        "a real signature from an unenrolled key must not be admitted: {report}"
    );
    assert!(
        report.contains("enroll"),
        "and it must read as an untrusted key, not as an unsigned commit: {report}"
    );
    assert!(
        !report.contains("the delivery contract requires an SSH-signed commit"),
        "the two failures have different repairs and must not be collapsed: {report}"
    );
}

/// The authority the gate enforces is the one a developer can install, or the
/// local instrument stays blind and every commit reads as unverifiable.
#[test]
fn the_authority_can_be_emitted_for_local_verification() {
    let (root, _) = repository("emit");
    let target = root.join("allowed_signers");

    let output = Command::new(env!("CARGO_BIN_EXE_pipeline-commit-signing-app"))
        .arg("--emit-allowed-signers")
        .arg(&target)
        .output()
        .expect("the admission binary runs");

    assert!(output.status.success());
    let rendered = std::fs::read_to_string(&target).expect("the authority is written");
    assert!(rendered.contains("namespaces=\"git\""), "{rendered}");
    assert!(rendered.contains("ssh-ed25519"), "{rendered}");
}

// THE ADMITTED PATH IS NOT COVERED END TO END, and the reason is worth
// recording rather than leaving as a silence.
//
// Reaching `SignatureState::SignedBy` needs a commit signed by a key the
// authority names, and this suite cannot make one: the enrolled keys are
// public halves, and a private half in the repository would be a far worse
// defect than the gap. The two alternatives are worse still — embedding a
// key, or letting the authority be injected at runtime, which is exactly the
// candidate-supplies-its-own-trust hole the two-checkout discipline exists to
// close.
//
// Three attempts to cover it against this repository's own history all
// failed, each in a different way, and the pattern is the lesson: the thing
// under test is not the thing you are looking at. `HEAD~1..HEAD` passed
// locally and judged the host's forged merge commit under CI; deriving the
// lane from `HEAD^2` fixed that and then died in the workspace test job,
// which checks out shallow so neither parent resolves.
//
// What guards the failure this would have caught — a gate that has begun
// refusing EVERYTHING — is the authority test in `signing_authority.rs`,
// which pins the enrolled set by count and identity, so dropping a key fails
// loudly. `signing_violations` is unit-tested to admit `SignedBy`. The
// uncovered link is `verify()` mapping a good signature onto it.
