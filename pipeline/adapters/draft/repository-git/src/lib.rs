//! Git implementation of the pipeline-local repository read port.

use std::process::{Command, Output};

use pipeline_repository_draft::RepositoryRead;

#[derive(Clone, Copy, Debug, Default)]
pub struct GitRepository;

impl RepositoryRead for GitRepository {
    fn merge_base(&self, left: &str, right: &str) -> Result<String, String> {
        git_text(&["merge-base", left, right])
    }

    fn changed_name_status(&self, base: &str, head: &str) -> Result<Vec<u8>, String> {
        git_output(&["diff", "--name-status", "-z", "-M", base, head, "--"])
            .map(|output| output.stdout)
    }

    fn blob_text(&self, commit: &str, path: &str) -> Result<String, String> {
        let object = format!("{commit}:{path}");
        let output = git_output(&["cat-file", "blob", &object])?;
        String::from_utf8(output.stdout)
            .map_err(|_| format!("git cat-file returned non-UTF-8 for {path}"))
    }

    fn path_exists(&self, commit: &str, path: &str) -> Result<bool, String> {
        exact_tree_path(commit, path, false)
    }

    fn directory_exists(&self, commit: &str, path: &str) -> Result<bool, String> {
        exact_tree_path(commit, path, true)
    }
}

fn exact_tree_path(commit: &str, path: &str, directory: bool) -> Result<bool, String> {
    let mut args = vec!["ls-tree"];
    if directory {
        args.push("-d");
    }
    args.extend(["--name-only", commit, "--", path]);
    let output = git_output(&args)?;
    let result = String::from_utf8(output.stdout)
        .map_err(|_| format!("git ls-tree returned non-UTF-8 for {path}"))?;
    match result.trim_end() {
        "" => Ok(false),
        present if present == path => Ok(true),
        other => Err(format!(
            "unexpected git ls-tree output for {path}: {other:?}"
        )),
    }
}

fn git_text(args: &[&str]) -> Result<String, String> {
    let output = git_output(args)?;
    let text = String::from_utf8(output.stdout)
        .map_err(|_| format!("git {} returned non-UTF-8", args.join(" ")))?;
    let text = text.trim().to_owned();
    if text.is_empty() {
        return Err(format!("git {} returned empty output", args.join(" ")));
    }
    Ok(text)
}

fn git_output(args: &[&str]) -> Result<Output, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("spawn git {}: {error}", args.join(" ")))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}
