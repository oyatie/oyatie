use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

pub(crate) fn run(args: Vec<String>, _usage: &str) -> ExitCode {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let effective_cwd = effective_git_cwd(&cwd, &args);
    let started_unix = unix_now();

    let status = match Command::new("git").args(&args).status() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("oya git failed to invoke git: {error}");
            return ExitCode::from(127);
        }
    };
    let exit_code = process_exit_code(status);

    if let Some(context) = git_repo_context(&effective_cwd) {
        let _ = append_ledger_event(
            &context,
            &cwd,
            &effective_cwd,
            &args,
            started_unix,
            exit_code,
        );
    }

    ExitCode::from(clamp_exit_code(exit_code))
}

struct GitRepoContext {
    root: PathBuf,
    git_dir: PathBuf,
}

fn git_repo_context(cwd: &Path) -> Option<GitRepoContext> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel", "--absolute-git-dir"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut lines = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let root = PathBuf::from(lines.next()?);
    let git_dir = PathBuf::from(lines.next()?);
    Some(GitRepoContext { root, git_dir })
}

fn append_ledger_event(
    context: &GitRepoContext,
    cwd: &Path,
    git_cwd: &Path,
    args: &[String],
    started_unix: u64,
    exit_code: i32,
) -> Result<(), String> {
    let ledger_dir = context.git_dir.join("oya").join("ledger");
    std::fs::create_dir_all(&ledger_dir)
        .map_err(|error| format!("oya git ledger dir unavailable: {error}"))?;
    let ledger_path = ledger_dir.join("audit-chain.jsonl");
    let event = json!({
        "schema_version": 1,
        "event_type": "oya_git_command",
        "timestamp_unix": unix_now(),
        "started_unix": started_unix,
        "cwd": repo_path_label(cwd, &context.root),
        "git_cwd": repo_path_label(git_cwd, &context.root),
        "repo_root": "repo-root",
        "ledger_scope": "git-metadata",
        "command": "oya git",
        "verb": ledger_verb(args),
        "arg_count": args.len(),
        "exit_code": exit_code,
        "success": exit_code == 0
    });
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ledger_path)
        .map_err(|error| format!("oya git ledger unavailable: {error}"))?;
    writeln!(file, "{event}").map_err(|error| format!("oya git ledger append failed: {error}"))
}

fn repo_path_label(path: &Path, repo_root: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let repo_root = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    if path == repo_root {
        return ".".to_string();
    }
    if let Ok(relative) = path.strip_prefix(&repo_root) {
        let label = relative.display().to_string();
        if label.is_empty() {
            ".".to_string()
        } else {
            label
        }
    } else {
        "outside-repo".to_string()
    }
}

fn git_verb(args: &[String]) -> Option<&str> {
    let mut skip_next = false;
    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        match arg.as_str() {
            "--" => continue,
            "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace" => {
                skip_next = true;
                continue;
            }
            value if value.starts_with("-C") && value.len() > 2 => continue,
            value
                if value.starts_with("--git-dir=")
                    || value.starts_with("--work-tree=")
                    || value.starts_with("--namespace=")
                    || value.starts_with("--exec-path=")
                    || value.starts_with("--config-env=") =>
            {
                continue;
            }
            value if value.starts_with('-') => continue,
            value => return Some(value),
        }
    }
    None
}

fn effective_git_cwd(cwd: &Path, args: &[String]) -> PathBuf {
    let mut effective = cwd.to_path_buf();
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--" => break,
            "-C" => {
                let Some(path) = args.get(index + 1) else {
                    break;
                };
                effective = apply_git_cwd(&effective, path);
                index += 2;
            }
            value if value.starts_with("-C") && value.len() > 2 => {
                effective = apply_git_cwd(&effective, &value[2..]);
                index += 1;
            }
            "-c" | "--git-dir" | "--work-tree" | "--namespace" => {
                index += 2;
            }
            value
                if value.starts_with("--git-dir=")
                    || value.starts_with("--work-tree=")
                    || value.starts_with("--namespace=")
                    || value.starts_with("--exec-path=")
                    || value.starts_with("--config-env=") =>
            {
                index += 1;
            }
            value if value.starts_with('-') => {
                index += 1;
            }
            _ => break,
        }
    }
    effective
}

fn apply_git_cwd(current: &Path, next: &str) -> PathBuf {
    if next.is_empty() {
        return current.to_path_buf();
    }
    let next = PathBuf::from(next);
    if next.is_absolute() {
        next
    } else {
        current.join(next)
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn process_exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        return 128 + signal;
    }

    1
}

fn clamp_exit_code(code: i32) -> u8 {
    if code < 0 || code > u8::MAX as i32 {
        1
    } else {
        code as u8
    }
}

fn ledger_verb(args: &[String]) -> Option<String> {
    git_verb(args).map(sanitize_ledger_token)
}

fn sanitize_ledger_token(value: &str) -> String {
    if value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        value.to_string()
    } else {
        "redacted".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_verb_skips_common_global_options() {
        let args = vec![
            "-C".to_string(),
            "/tmp/repo".to_string(),
            "-c".to_string(),
            "status.short=true".to_string(),
            "status".to_string(),
            "--short".to_string(),
        ];

        assert_eq!(git_verb(&args), Some("status"));
    }

    #[test]
    fn clamp_exit_code_preserves_normal_codes() {
        assert_eq!(clamp_exit_code(0), 0);
        assert_eq!(clamp_exit_code(2), 2);
        assert_eq!(clamp_exit_code(256), 1);
        assert_eq!(clamp_exit_code(-1), 1);
    }

    #[test]
    fn ledger_verb_redacts_unusual_tokens() {
        let args = vec!["https://token:secret@example.com/owner/repo.git".to_string()];

        assert_eq!(ledger_verb(&args), Some("redacted".to_string()));
    }

    #[test]
    fn effective_git_cwd_applies_repeated_c_options() {
        let args = vec![
            "-C".to_string(),
            "/tmp".to_string(),
            "-C".to_string(),
            "repo".to_string(),
            "status".to_string(),
        ];

        assert_eq!(
            effective_git_cwd(Path::new("/home/example"), &args),
            PathBuf::from("/tmp/repo")
        );
    }

    #[test]
    fn repo_path_label_redacts_outside_paths() {
        assert_eq!(
            repo_path_label(Path::new("/tmp/caller"), Path::new("/tmp/repo")),
            "outside-repo"
        );
    }
}
