//! Gathering commit facts from a repository. The impure half.
//!
//! The core decides what a fact MEANS; this decides what the facts ARE, and
//! keeps every subprocess and every parse on this side of the seam.

use std::path::Path;
use std::process::Command;

use pipeline_admission::{CommitFact, SignatureState};

/// Why facts could not be gathered at all. Distinct from any per-commit
/// verdict: a range that could not be listed is not a range of good commits.
#[derive(Debug)]
pub struct GatherError(pub String);

impl std::fmt::Display for GatherError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

fn git(args: &[&str]) -> Result<std::process::Output, GatherError> {
    Command::new("git")
        .args(args)
        .output()
        .map_err(|error| GatherError(format!("could not run git: {error}")))
}

/// The lane's own commits: everything reachable from `head` but not `base`.
pub fn range(base: &str, head: &str) -> Result<Vec<String>, GatherError> {
    let output = git(&["rev-list", &format!("{base}..{head}")])?;
    if !output.status.success() {
        return Err(GatherError(format!(
            "could not list {base}..{head}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect())
}

/// Does the commit object carry a signature header at all?
///
/// Read from the object bytes rather than from a verifier, because the
/// unsigned case is SILENT: `git verify-commit` exits non-zero and prints
/// nothing, so an unsigned commit and a broken verifier are indistinguishable
/// downstream. The object always knows.
fn carries_signature(sha: &str) -> Result<bool, GatherError> {
    let output = git(&["cat-file", "commit", sha])?;
    if !output.status.success() {
        return Err(GatherError(format!("could not read commit {sha}")));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        // Headers end at the first blank line; a message body line beginning
        // with "gpgsig" must not be mistaken for one.
        .take_while(|line| !line.is_empty())
        .any(|line| line.starts_with("gpgsig")))
}

/// Verify one commit against the authority rendered at `allowed_signers`.
///
/// The exit code is the verdict and the text is only the reason. That is not
/// stylistic: an untrusted key still prints `Good "git" signature`, so a
/// verifier read by grepping for "Good" admits exactly the commits this gate
/// exists to refuse.
fn verify(sha: &str, allowed_signers: &Path) -> Result<SignatureState, GatherError> {
    if !carries_signature(sha)? {
        return Ok(SignatureState::Unsigned);
    }
    let signers = allowed_signers.to_string_lossy().into_owned();
    let output = git(&[
        "-c",
        &format!("gpg.ssh.allowedSignersFile={signers}"),
        "verify-commit",
        sha,
    ])?;
    let reason = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if output.status.success() {
        return Ok(SignatureState::SignedBy(principal_of(&reason)));
    }
    if reason.contains("No principal matched") {
        return Ok(SignatureState::UntrustedKey);
    }
    Ok(SignatureState::Unverifiable(if reason.is_empty() {
        "the verifier reported nothing".to_owned()
    } else {
        reason.replace('\n', "; ")
    }))
}

/// Pull the principal out of `Good "git" signature for <principal> with ...`.
fn principal_of(reason: &str) -> String {
    reason
        .split_once(" for ")
        .and_then(|(_, rest)| rest.split_once(" with "))
        .map(|(principal, _)| principal.to_owned())
        .unwrap_or_else(|| "an unnamed principal".to_owned())
}

/// Every commit in the range, with what verification concluded about each.
pub fn facts(
    base: &str,
    head: &str,
    allowed_signers: &Path,
) -> Result<Vec<CommitFact>, GatherError> {
    range(base, head)?
        .into_iter()
        .map(|sha| {
            let signature = verify(&sha, allowed_signers)?;
            Ok(CommitFact { sha, signature })
        })
        .collect()
}
