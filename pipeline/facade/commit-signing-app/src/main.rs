//! Commit signing admission for the protected `presubmit` graph.
//!
//! ```text
//! pipeline-commit-signing-app <base-sha> <head-sha>
//! pipeline-commit-signing-app --emit-allowed-signers <path>
//! ```
//!
//! The delivery contract requires an SSH-signed commit on every lane. That
//! requirement was enforced only by a protected-branch ruleset which emits no
//! check run, so a lane carrying one unsigned commit showed every check green
//! and sat `BLOCKED` with nothing naming the cause. This binary is the missing
//! diagnostic: it refuses the same lanes the ruleset refuses, and says which
//! commit and what to do.
//!
//! The second form emits the authority for local use, so that
//! `gpg.ssh.allowedSignersFile` points at what CI enforces:
//!
//! ```text
//! pipeline-commit-signing-app --emit-allowed-signers .git/allowed_signers
//! git config gpg.ssh.allowedSignersFile "$PWD/.git/allowed_signers"
//! ```
//!
//! Without it `git verify-commit` reports every commit unverifiable, which
//! reads exactly like every commit being unsigned.

mod verify;

use std::path::PathBuf;
use std::process::ExitCode;

use pipeline_admission::{allowed_signers, signing_violations};

const USAGE: &str = "usage: pipeline-commit-signing-app <base-sha> <head-sha>\n   \
                     or: pipeline-commit-signing-app --emit-allowed-signers <path>";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [flag, path] if flag == "--emit-allowed-signers" => emit(PathBuf::from(path)),
        [base, head] => admit(base, head),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn emit(path: PathBuf) -> ExitCode {
    match std::fs::write(&path, allowed_signers()) {
        Ok(()) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("could not write {}: {error}", path.display());
            ExitCode::FAILURE
        }
    }
}

fn admit(base: &str, head: &str) -> ExitCode {
    // The authority is rendered from the protected source this binary was
    // built from, never read out of the candidate tree. A lane that could
    // supply its own key file would authorize itself in the same commit the
    // gate is judging.
    let signers =
        std::env::temp_dir().join(format!("oyatie-allowed-signers-{}", std::process::id()));
    if let Err(error) = std::fs::write(&signers, allowed_signers()) {
        eprintln!("could not stage the signing authority: {error}");
        return ExitCode::FAILURE;
    }
    let gathered = verify::facts(base, head, &signers);
    // Best effort: a stale temp file must never turn a verdict into an error.
    let _ = std::fs::remove_file(&signers);

    let facts = match gathered {
        Ok(facts) => facts,
        Err(error) => {
            // FAIL CLOSED. Not gathering the facts is not the same as there
            // being no bad facts.
            eprintln!("commit signing admission could not read the range: {error}");
            return ExitCode::FAILURE;
        }
    };
    let violations = signing_violations(&facts);
    if violations.is_empty() {
        return ExitCode::SUCCESS;
    }
    eprintln!(
        "commit signing admission refused {} of {} commits in {base}..{head}:",
        violations.len(),
        facts.len()
    );
    for violation in &violations {
        eprintln!("  {violation}");
    }
    eprintln!(
        "\nRe-sign a lane by replaying it onto its base with each commit signed:\n  \
         git rebase --onto <base> <base> <lane> --exec 'git commit --amend --no-edit -S'\n\
         then force-push the lane. Verify locally first:\n  \
         pipeline-commit-signing-app --emit-allowed-signers .git/allowed_signers\n  \
         git config gpg.ssh.allowedSignersFile \"$PWD/.git/allowed_signers\""
    );
    ExitCode::FAILURE
}
