//! Sequential PR merge-conflict simulator for the temporary GitHub lane unlocker.
//!
//! This is a local/static merge-safety tool. It models open PRs in ascending PR
//! number order using `git merge-tree --write-tree` and `git commit-tree`.
//! It does not mutate branch protection, post statuses, merge PRs, or claim
//! native CI/CD authority.

#[allow(dead_code)]
#[path = "ci/assert-result-bundle-output.rs"]
mod json_support;

use json_support::Json;
use std::collections::{BTreeSet, HashSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Options {
    base_branch: String,
    base_ref: Option<String>,
    start_pr: u64,
    end_pr: Option<u64>,
    skip_prs: BTreeSet<u64>,
    skip_prs_raw: String,
    limit: u64,
    pr_json: Option<PathBuf>,
    fetch_heads: bool,
    fetch_remote: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            base_branch: "dev".to_string(),
            base_ref: None,
            start_pr: 1,
            end_pr: None,
            skip_prs: BTreeSet::new(),
            skip_prs_raw: String::new(),
            limit: 200,
            pr_json: None,
            fetch_heads: true,
            fetch_remote: "origin".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PullRequest {
    number: u64,
    head_ref_name: String,
    head_ref_oid: String,
    is_draft: bool,
    title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

fn usage(program: &str) -> String {
    format!(
        "\
Usage: {program} [options]

Options:
  --base-branch <branch>   GitHub PR base branch to inspect (default: dev)
  --base-ref <ref>         Local git ref used as the starting virtual base
                           (default: origin/<base-branch>)
  --start-pr <number>      First PR number in the numeric merge sequence
                           (default: 1; no PR is excluded by default)
  --end-pr <number>        Last PR number to include in the numeric merge
                           sequence (default: no upper bound)
  --skip-prs <csv>         Explicit one-off PR numbers to skip, e.g. \"109,130\"
                           (default: empty)
  --limit <number>         Maximum open PRs to query from GitHub (default: 200)
  --pr-json <path>         Read PR list JSON from a file instead of gh pr list
  --fetch-remote <remote>  Git remote used to fetch refs/pull/<N>/head when
                           --no-fetch is not supplied (default: origin)
  --no-fetch               Do not fetch refs/pull/<N>/head before simulation

The tool simulates open PRs in ascending PR-number order by repeatedly
running `git merge-tree --write-tree` and materializing a temporary virtual
merge commit with `git commit-tree`. It fails at the first conflict and prints
the conflict file list.
"
    )
}

fn parse_positive_u64(raw: &str, flag: &str) -> Result<u64, String> {
    if raw.is_empty() || raw.chars().any(|ch| !ch.is_ascii_digit()) {
        return Err(format!("{flag} must be a positive integer"));
    }
    raw.parse::<u64>()
        .map_err(|_| format!("{flag} must be a positive integer"))
}

fn parse_skip_prs(raw: &str) -> Result<BTreeSet<u64>, String> {
    let mut values = BTreeSet::new();
    for item in raw
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        values.insert(parse_positive_u64(item, "--skip-prs")?);
    }
    Ok(values)
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing {flag} value"))
}

fn parse_args(args: &[String]) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--base-branch" => options.base_branch = take_value(args, &mut index, "--base-branch")?,
            "--base-ref" => options.base_ref = Some(take_value(args, &mut index, "--base-ref")?),
            "--start-pr" => {
                options.start_pr =
                    parse_positive_u64(&take_value(args, &mut index, "--start-pr")?, "--start-pr")?
            }
            "--end-pr" => {
                options.end_pr = Some(parse_positive_u64(
                    &take_value(args, &mut index, "--end-pr")?,
                    "--end-pr",
                )?)
            }
            "--skip-prs" => {
                options.skip_prs_raw = take_value(args, &mut index, "--skip-prs")?;
                options.skip_prs = parse_skip_prs(&options.skip_prs_raw)?;
            }
            "--limit" => {
                options.limit =
                    parse_positive_u64(&take_value(args, &mut index, "--limit")?, "--limit")?
            }
            "--pr-json" => {
                options.pr_json = Some(PathBuf::from(take_value(args, &mut index, "--pr-json")?))
            }
            "--fetch-remote" => {
                options.fetch_remote = take_value(args, &mut index, "--fetch-remote")?
            }
            "--no-fetch" => options.fetch_heads = false,
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    if let Some(end_pr) = options.end_pr {
        if end_pr < options.start_pr {
            return Err("--end-pr must be greater than or equal to --start-pr".to_string());
        }
    }
    if options.fetch_heads && options.fetch_remote.is_empty() {
        return Err("--fetch-remote must not be empty when PR-head fetch is enabled".to_string());
    }
    Ok(Some(options))
}

fn command_output(cwd: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run {program}: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "{program} {} failed: {}",
            args.join(" "),
            stderr.trim()
        ))
    }
}

fn rev_parse(cwd: &Path, rev: &str) -> Result<String, String> {
    command_output(cwd, "git", &["rev-parse", rev])
}

fn verify_commit(cwd: &Path, rev: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", rev])
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_git_fetch(cwd: &Path, remote: &str, number: u64, pr_ref: &str) -> bool {
    Command::new("git")
        .args([
            "fetch",
            "--no-tags",
            remote,
            &format!("+refs/pull/{number}/head:{pr_ref}"),
        ])
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_merge_tree(cwd: &Path, virtual_head: &str, head_ref: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["merge-tree", "--write-tree", virtual_head, head_ref])
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run git merge-tree: {error}"))?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() {
        Ok(combined)
    } else {
        Err(combined)
    }
}

fn commit_tree(
    cwd: &Path,
    tree_id: &str,
    virtual_head: &str,
    head_commit: &str,
    number: u64,
) -> Result<String, String> {
    let mut args = vec![
        tree_id.to_string(),
        "-p".to_string(),
        virtual_head.to_string(),
    ];
    if head_commit != virtual_head {
        args.push("-p".to_string());
        args.push(head_commit.to_string());
    }
    let mut child = Command::new("git")
        .arg("commit-tree")
        .args(args)
        .current_dir(cwd)
        .env(
            "GIT_AUTHOR_NAME",
            env::var("GIT_AUTHOR_NAME").unwrap_or_else(|_| "oyatie-queue-simulator".to_string()),
        )
        .env(
            "GIT_AUTHOR_EMAIL",
            env::var("GIT_AUTHOR_EMAIL")
                .unwrap_or_else(|_| "queue-simulator@users.noreply.github.com".to_string()),
        )
        .env(
            "GIT_COMMITTER_NAME",
            env::var("GIT_COMMITTER_NAME").unwrap_or_else(|_| {
                env::var("GIT_AUTHOR_NAME").unwrap_or_else(|_| "oyatie-queue-simulator".to_string())
            }),
        )
        .env(
            "GIT_COMMITTER_EMAIL",
            env::var("GIT_COMMITTER_EMAIL").unwrap_or_else(|_| {
                env::var("GIT_AUTHOR_EMAIL")
                    .unwrap_or_else(|_| "queue-simulator@users.noreply.github.com".to_string())
            }),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to spawn git commit-tree: {error}"))?;
    child
        .stdin
        .as_mut()
        .expect("commit-tree stdin")
        .write_all(format!("sequential merge simulation PR #{number}\n").as_bytes())
        .map_err(|error| format!("failed to write commit-tree message: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for git commit-tree: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!(
            "git commit-tree failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn json_object_field<'a>(
    object: &'a std::collections::BTreeMap<String, Json>,
    key: &str,
) -> Option<&'a Json> {
    object.get(key)
}

fn json_string(object: &std::collections::BTreeMap<String, Json>, key: &str) -> String {
    json_object_field(object, key)
        .and_then(Json::as_str)
        .unwrap_or_default()
        .to_string()
}

fn json_bool(object: &std::collections::BTreeMap<String, Json>, key: &str) -> bool {
    json_object_field(object, key)
        .and_then(Json::as_bool)
        .unwrap_or(false)
}

fn parse_prs(text: &str) -> Result<Vec<PullRequest>, String> {
    let json = json_support::parse_json(text)?;
    let Some(items) = json.as_array() else {
        return Err("PR JSON must be an array".to_string());
    };
    let mut prs = Vec::new();
    for item in items {
        let Some(object) = item.as_object() else {
            return Err("PR JSON entries must be objects".to_string());
        };
        let Some(number) = json_object_field(object, "number").and_then(Json::as_i64) else {
            return Err("PR JSON entry missing numeric number".to_string());
        };
        if number < 0 {
            return Err("PR JSON entry has negative number".to_string());
        }
        prs.push(PullRequest {
            number: number as u64,
            head_ref_name: json_string(object, "headRefName"),
            head_ref_oid: json_string(object, "headRefOid"),
            is_draft: json_bool(object, "isDraft"),
            title: json_string(object, "title"),
        });
    }
    prs.sort_by_key(|pr| pr.number);
    Ok(prs)
}

fn load_prs(cwd: &Path, options: &Options) -> Result<Vec<PullRequest>, String> {
    let text = if let Some(path) = &options.pr_json {
        fs::read_to_string(path)
            .map_err(|error| format!("read {} failed: {error}", path.display()))?
    } else {
        let output = Command::new("gh")
            .args([
                "pr",
                "list",
                "--state",
                "open",
                "--base",
                &options.base_branch,
                "--limit",
                &options.limit.to_string(),
                "--json",
                "number,headRefName,headRefOid,isDraft,title",
            ])
            .current_dir(cwd)
            .output()
            .map_err(|error| format!("gh is required unless --pr-json is supplied: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "gh pr list failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    parse_prs(&text)
}

fn short_sha(value: &str) -> &str {
    value.get(..8).unwrap_or(value)
}

fn conflict_files(merge_output: &str) -> Vec<String> {
    let mut files = HashSet::new();
    for line in merge_output.lines() {
        let Some((prefix, path)) = line.split_once('\t') else {
            continue;
        };
        let parts = prefix.split_whitespace().collect::<Vec<_>>();
        if parts.len() == 3
            && parts[0].len() == 6
            && parts[0].chars().all(|ch| ch.is_ascii_digit())
            && parts[1].chars().all(|ch| ch.is_ascii_hexdigit())
            && matches!(parts[2], "1" | "2" | "3")
        {
            files.insert(path.to_string());
        }
    }
    let mut files = files.into_iter().collect::<Vec<_>>();
    files.sort();
    files
}

fn selected_prs(options: &Options, prs: Vec<PullRequest>) -> Vec<PullRequest> {
    prs.into_iter()
        .filter(|pr| pr.number >= options.start_pr)
        .filter(|pr| options.end_pr.is_none_or(|end| pr.number <= end))
        .filter(|pr| !options.skip_prs.contains(&pr.number))
        .collect()
}

pub fn run_in(cwd: &Path, args: &[String]) -> RunResult {
    let program = args
        .first()
        .map(String::as_str)
        .unwrap_or("check-sequential-pr-merge-conflicts");
    let options = match parse_args(args) {
        Ok(Some(options)) => options,
        Ok(None) => {
            return RunResult {
                code: 0,
                stdout: usage(program),
                stderr: String::new(),
            };
        }
        Err(error) => {
            return RunResult {
                code: 2,
                stdout: String::new(),
                stderr: format!("{error}\n{}", usage(program)),
            };
        }
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    let base_ref = options
        .base_ref
        .clone()
        .unwrap_or_else(|| format!("origin/{}", options.base_branch));
    if !verify_commit(cwd, &format!("{base_ref}^{{commit}}")) {
        return RunResult {
            code: 2,
            stdout,
            stderr: format!("base ref is not a commit: {base_ref}\n"),
        };
    }
    let prs = match load_prs(cwd, &options).map(|prs| selected_prs(&options, prs)) {
        Ok(prs) => prs,
        Err(error) => {
            return RunResult {
                code: 2,
                stdout,
                stderr: format!("{error}\n"),
            };
        }
    };
    if prs.is_empty() {
        stdout.push_str(&format!(
            "sequential PR merge simulation: no open PRs matched base={} start_pr={}\n",
            options.base_branch, options.start_pr
        ));
        return RunResult {
            code: 0,
            stdout,
            stderr,
        };
    }

    let mut virtual_head = match rev_parse(cwd, &format!("{base_ref}^{{commit}}")) {
        Ok(value) => value,
        Err(error) => {
            return RunResult {
                code: 2,
                stdout,
                stderr: format!("{error}\n"),
            };
        }
    };
    stdout.push_str("sequential PR merge simulation\n");
    stdout.push_str(&format!("base_branch={}\n", options.base_branch));
    stdout.push_str(&format!("base_ref={base_ref}\n"));
    stdout.push_str(&format!("base_commit={virtual_head}\n"));
    stdout.push_str(&format!("start_pr={}\n", options.start_pr));
    stdout.push_str(&format!(
        "end_pr={}\n",
        options
            .end_pr
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<none>".to_string())
    ));
    stdout.push_str(&format!(
        "skip_prs={}\n",
        if options.skip_prs_raw.is_empty() {
            "<none>"
        } else {
            &options.skip_prs_raw
        }
    ));
    if options.fetch_heads {
        stdout.push_str(&format!("fetch_remote={}\n", options.fetch_remote));
    } else {
        stdout.push_str("fetch_remote=<disabled>\n");
    }

    let mut count = 0usize;
    for pr in prs {
        count += 1;
        let pr_ref = format!("refs/remotes/pr/{}", pr.number);
        let head_ref = if options.fetch_heads {
            if !run_git_fetch(cwd, &options.fetch_remote, pr.number, &pr_ref) {
                stderr.push_str(&format!(
                    "::error::failed to fetch PR #{} head from remote {}; pass --fetch-remote for the GitHub mirror when origin is not GitHub\n",
                    pr.number, options.fetch_remote
                ));
                return RunResult {
                    code: 1,
                    stdout,
                    stderr,
                };
            }
            let fetched_head = match rev_parse(cwd, &format!("{pr_ref}^{{commit}}")) {
                Ok(value) => value,
                Err(error) => {
                    stderr.push_str(&format!("{error}\n"));
                    return RunResult {
                        code: 1,
                        stdout,
                        stderr,
                    };
                }
            };
            if fetched_head != pr.head_ref_oid {
                stderr.push_str(&format!(
                    "::error::PR #{} moved while fetching ({} -> {}); refusing stale queue simulation\n",
                    pr.number, pr.head_ref_oid, fetched_head
                ));
                return RunResult {
                    code: 1,
                    stdout,
                    stderr,
                };
            }
            pr_ref
        } else {
            pr.head_ref_oid.clone()
        };
        if !verify_commit(cwd, &format!("{head_ref}^{{commit}}")) {
            stderr.push_str(&format!(
                "::error::PR #{} head is not available locally: {head_ref}\n",
                pr.number
            ));
            return RunResult {
                code: 1,
                stdout,
                stderr,
            };
        }
        let head_commit = match rev_parse(cwd, &format!("{head_ref}^{{commit}}")) {
            Ok(value) => value,
            Err(error) => {
                stderr.push_str(&format!("{error}\n"));
                return RunResult {
                    code: 1,
                    stdout,
                    stderr,
                };
            }
        };
        stdout.push_str(&format!(
            "checking PR #{}: {} ({}) draft={} {}\n",
            pr.number,
            pr.head_ref_name,
            short_sha(&pr.head_ref_oid),
            pr.is_draft,
            pr.title
        ));
        let merge_output = match run_merge_tree(cwd, &virtual_head, &head_ref) {
            Ok(output) => output,
            Err(output) => {
                stderr.push_str(&format!(
                    "::error::sequential merge conflict at PR #{} ({})\n",
                    pr.number, pr.head_ref_name
                ));
                stderr.push_str("conflict files:\n");
                for file in conflict_files(&output) {
                    stderr.push_str(&format!("{file}\n"));
                }
                stderr.push_str("merge-tree output:\n");
                stderr.push_str(&output);
                if !output.ends_with('\n') {
                    stderr.push('\n');
                }
                return RunResult {
                    code: 1,
                    stdout,
                    stderr,
                };
            }
        };
        let tree_id = merge_output.lines().next().unwrap_or_default().trim();
        if tree_id.is_empty() {
            stderr.push_str(&format!(
                "::error::git merge-tree produced no tree for PR #{}\n",
                pr.number
            ));
            return RunResult {
                code: 1,
                stdout,
                stderr,
            };
        }
        virtual_head = match commit_tree(cwd, tree_id, &virtual_head, &head_commit, pr.number) {
            Ok(value) => value,
            Err(error) => {
                stderr.push_str(&format!("{error}\n"));
                return RunResult {
                    code: 1,
                    stdout,
                    stderr,
                };
            }
        };
    }
    stdout.push_str(&format!(
        "sequential PR merge simulation passed: {count} PRs modeled; virtual_head={virtual_head}\n"
    ));
    RunResult {
        code: 0,
        stdout,
        stderr,
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let result = run_in(&cwd, &args);
    print!("{}", result.stdout);
    eprint!("{}", result.stderr);
    std::process::exit(result.code);
}
