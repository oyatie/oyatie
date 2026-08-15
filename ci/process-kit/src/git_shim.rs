//! Git argv refuse surface (guardrails-env-escape / --no-verify deny).

/// Verbs where `--no-verify` / `-n` means hook bypass.
pub fn is_mutating_git_verb(verb: &str) -> bool {
    matches!(
        verb,
        "commit"
            | "commit-tree"
            | "merge"
            | "rebase"
            | "cherry-pick"
            | "am"
            | "revert"
            | "push"
            | "pull"
            | "apply"
    )
}

/// Return true when argv requests hook bypass (`--no-verify` / `-n` as verify-skip).
pub fn denies_no_verify(args: &[&str]) -> bool {
    for a in args {
        if *a == "--" {
            break;
        }
        if *a == "--no-verify" {
            return true;
        }
        if *a == "-n" {
            return true;
        }
        if let Some(rest) = a.strip_prefix('-') {
            if !rest.starts_with('-') && rest.contains('n') && !rest.contains('=') {
                return true;
            }
        }
    }
    false
}

/// Fail-closed refusal for verification bypass on mutating git commands.
pub fn refuse_no_verify(args: &[&str]) -> Result<(), String> {
    let verb = args.first().copied().unwrap_or("");
    if is_mutating_git_verb(verb) && denies_no_verify(&args[1..]) {
        return Err(
            "git-shim: REFUSE — --no-verify/-n denied by the operating contract".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_long_and_short_on_commit() {
        assert!(refuse_no_verify(&["commit", "--no-verify", "-m", "x"]).is_err());
        assert!(refuse_no_verify(&["commit", "-n", "-m", "x"]).is_err());
        assert!(refuse_no_verify(&["commit", "-an", "-m", "x"]).is_err());
        assert!(refuse_no_verify(&["commit", "-m", "x"]).is_ok());
        assert!(refuse_no_verify(&["status", "-n"]).is_ok());
    }
}
