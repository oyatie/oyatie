#![forbid(unsafe_code)]

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use checkout_guard_app::{
    Decision, DecisionInput, decide, default_canonical_checkout, extract_command_from_hook_payload,
};

fn main() -> ExitCode {
    match run() {
        Ok(Decision::Allow) => ExitCode::SUCCESS,
        Ok(Decision::Deny { reason }) => {
            eprintln!("{reason}");
            ExitCode::from(2)
        }
        Err(err) => {
            eprintln!("main-checkout-guard warning: {err}; allowing command");
            ExitCode::SUCCESS
        }
    }
}

fn run() -> Result<Decision, String> {
    let mut payload = String::new();
    std::io::stdin()
        .read_to_string(&mut payload)
        .map_err(|err| format!("failed to read hook payload from stdin: {err}"))?;

    let Some(command) = extract_command_from_hook_payload(&payload) else {
        return Ok(Decision::Allow);
    };
    let session_cwd = std::env::current_dir()
        .map_err(|err| format!("failed to resolve current directory: {err}"))?;
    let canonical_checkout = configured_canonical_checkout(&session_cwd);

    Ok(decide(DecisionInput {
        command,
        session_cwd,
        canonical_checkout,
        process_env: std::env::vars().collect(),
    }))
}

fn configured_canonical_checkout(session_cwd: &Path) -> Option<PathBuf> {
    if let Ok(value) = std::env::var("OYA_CANONICAL_CHECKOUT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            let path = Path::new(trimmed);
            return if path.is_absolute() {
                Some(path.to_path_buf())
            } else {
                Some(session_cwd.join(path))
            };
        }
    }

    let repo_root = git_rev_parse("--show-toplevel")?;
    let git_common_dir = git_rev_parse("--git-common-dir")?;
    default_canonical_checkout(&repo_root, &git_common_dir)
}

fn git_rev_parse(argument: &str) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", argument])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}
