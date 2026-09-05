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

fn git(at: &Path, args: &[&str]) -> Result<std::process::Output, GatherError> {
    Command::new("git")
        .args(args)
        .current_dir(at)
        .output()
        .map_err(|error| GatherError(format!("could not run git: {error}")))
}

/// The lane's own commits: everything reachable from `head` but not `base`.
pub fn range(at: &Path, base: &str, head: &str) -> Result<Vec<String>, GatherError> {
    let output = git(at, &["rev-list", &format!("{base}..{head}")])?;
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
fn signature_header(at: &Path, sha: &str) -> Result<Option<String>, GatherError> {
    let output = git(at, &["cat-file", "commit", sha])?;
    if !output.status.success() {
        return Err(GatherError(format!("could not read commit {sha}")));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        // Headers end at the first blank line; a message body line beginning
        // with "gpgsig" must not be mistaken for one.
        .take_while(|line| !line.is_empty())
        .find(|line| line.starts_with("gpgsig"))
        .map(str::to_owned))
}

/// Verify one commit against the authority rendered at `allowed_signers`.
///
/// The exit code is the verdict and the text is only the reason. That is not
/// stylistic: an untrusted key still prints `Good "git" signature`, so a
/// verifier read by grepping for "Good" admits exactly the commits this gate
/// exists to refuse.
fn verify(at: &Path, sha: &str, allowed_signers: &Path) -> Result<SignatureState, GatherError> {
    let Some(header) = signature_header(at, sha)? else {
        return Ok(SignatureState::Unsigned);
    };
    // Recognised from the object rather than from the verifier's complaint:
    // `gpg` may be absent, in which case git reports a broken instrument for
    // what is really a well-formed signature of the wrong kind.
    if header.contains("BEGIN PGP SIGNATURE") {
        return Ok(SignatureState::NotSshSigned);
    }
    let signers = allowed_signers.to_string_lossy().into_owned();
    let output = git(
        at,
        &[
            "-c",
            &format!("gpg.ssh.allowedSignersFile={signers}"),
            "verify-commit",
            sha,
        ],
    )?;
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
    at: &Path,
    base: &str,
    head: &str,
    allowed_signers: &Path,
) -> Result<Vec<CommitFact>, GatherError> {
    range(at, base, head)?
        .into_iter()
        .map(|sha| {
            let signature = verify(at, &sha, allowed_signers)?;
            Ok(CommitFact { sha, signature })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(at: &Path, args: &[&str]) -> std::process::Output {
        let out = Command::new("git")
            .args(args)
            .current_dir(at)
            .env("GIT_AUTHOR_NAME", "Probe")
            .env("GIT_AUTHOR_EMAIL", "probe@example.invalid")
            .env("GIT_COMMITTER_NAME", "Probe")
            .env("GIT_COMMITTER_EMAIL", "probe@example.invalid")
            .output()
            .expect("git runs");
        assert!(out.status.success(), "git {args:?}: {out:?}");
        out
    }

    fn scratch(case: &str) -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!("oyatie-verify-{case}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("a scratch repository");
        git(&at, &["init", "-q", "-b", "main"]);
        git(&at, &["config", "commit.gpgsign", "false"]);
        git(&at, &["commit", "-q", "--allow-empty", "-m", "base"]);
        at
    }

    fn head(at: &Path) -> String {
        String::from_utf8_lossy(&git(at, &["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_owned()
    }

    /// A signature from a key the anchor NAMES is admitted.
    ///
    /// The one link no other test reaches. It needs no enrolled private key
    /// and no change to how production picks its authority: `facts` already
    /// takes the anchor as a parameter, and `main` goes on feeding it from
    /// the protected source. The key is generated here and trusted only here.
    #[test]
    fn a_signature_from_a_key_the_anchor_names_is_admitted() {
        let at = scratch("trusted");
        let base = head(&at);
        let key = at.join("author");
        assert!(
            Command::new("ssh-keygen")
                .args(["-q", "-t", "ed25519", "-N", "", "-C", "author", "-f"])
                .arg(&key)
                .status()
                .expect("ssh-keygen runs")
                .success()
        );
        git(&at, &["config", "gpg.format", "ssh"]);
        git(
            &at,
            &[
                "config",
                "user.signingkey",
                &format!("{}.pub", key.display()),
            ],
        );
        git(
            &at,
            &["commit", "-q", "-S", "--allow-empty", "-m", "signed"],
        );
        let public = std::fs::read_to_string(format!("{}.pub", key.display())).expect("the key");
        let anchor = at.join("allowed_signers");
        std::fs::write(
            &anchor,
            format!("probe@example.invalid namespaces=\"git\" {}", public.trim()),
        )
        .expect("the anchor");

        let facts = facts(&at, &base, &head(&at), &anchor).expect("the range reads");

        assert_eq!(facts.len(), 1);
        assert!(
            matches!(facts[0].signature, SignatureState::SignedBy(_)),
            "a key the anchor names must be admitted: {:?}",
            facts[0].signature
        );
    }

    /// A PGP signature is recognised as ITS OWN case, from the object bytes.
    ///
    /// This is what GitHub's "Update branch" forges, and it is the likeliest
    /// real trip. Read from the object rather than from the verifier's
    /// complaint, because `gpg` may be absent — in which case git reports a
    /// broken instrument for a well-formed signature of the wrong kind, and
    /// sends the author to look at CI instead of rebasing.
    #[test]
    fn a_pgp_signature_is_not_reported_as_a_broken_verifier() {
        let at = scratch("pgp");
        let base = head(&at);
        git(
            &at,
            &["commit", "-q", "--allow-empty", "-m", "to be forged"],
        );
        let raw =
            String::from_utf8_lossy(&git(&at, &["cat-file", "commit", "HEAD"]).stdout).into_owned();
        let (headers, body) = raw.split_once("\n\n").expect("a commit body");
        let forged = format!(
            "{headers}\ngpgsig -----BEGIN PGP SIGNATURE-----\n \n \
             -----END PGP SIGNATURE-----\n\n{body}"
        );
        let sha = {
            use std::io::Write as _;
            let mut child = Command::new("git")
                .args(["hash-object", "-w", "-t", "commit", "--stdin"])
                .current_dir(&at)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .expect("git runs");
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(forged.as_bytes())
                .expect("write");
            let out = child.wait_with_output().expect("git finishes");
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        };

        let facts = facts(&at, &base, &sha, &at.join("absent_anchor")).expect("the range reads");

        assert_eq!(
            facts[0].signature,
            SignatureState::NotSshSigned,
            "a PGP signature is a recognised verdict, not a broken instrument"
        );
    }
}
