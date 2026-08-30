//! Fail-closed GitHub path occupancy facade and verdict.
//!
//! Success means every open pull request targeting `dev` was enumerated, its
//! Git head was fetched, and its complete NUL-delimited Git path-set was
//! disjoint from the current pull request over AUTHORED paths (paths `.gitattributes`
//! declares structurally mergeable are excluded; see `pipeline_admission::occupancy`). API, fetch, diff, parse, and empty
//! current-set failures are all red.

use std::collections::BTreeSet;
use std::env;
use std::process::{Command, ExitCode, Output};

use pipeline_admission::{
    GitChangePaths, OccupiedSet, admit_authored, declared_mergeable,
    git_change_paths_from_name_status_z,
};

const REMOTE: &str = "origin";
const TRUNK_REF: &str = "refs/remotes/origin/dev";
const FETCH_BATCH_SIZE: usize = 128;

#[derive(Debug)]
struct Config {
    repository: String,
    pull_request: u64,
    token: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let config = config_from_env()?;
    let open = list_open_pull_requests(&config.repository)?;
    if !open.contains(&config.pull_request) {
        return Err(format!(
            "current pull request {} was absent from the complete open-PR listing",
            config.pull_request
        ));
    }

    fetch_trunk_and_pull_heads(&open, &config.token)?;
    let current_head = pull_head_ref(config.pull_request);
    let current_base = git_text(
        &config.token,
        &["merge-base", current_head.as_str(), TRUNK_REF],
    )?;
    let this = git_change_paths(&config.token, &current_base, &current_head)?.occupied;

    let mut in_flight = Vec::with_capacity(open.len().saturating_sub(1));
    for number in open {
        if number == config.pull_request {
            continue;
        }
        let head = pull_head_ref(number);
        let merge_base = git_text(&config.token, &["merge-base", head.as_str(), TRUNK_REF])?;
        let paths = git_change_paths(&config.token, &merge_base, &head)?.occupied;
        if !paths.is_empty() {
            in_flight.push(OccupiedSet {
                id: format!("pr-{number}"),
                paths,
            });
        }
    }

    // Read the declaration from TRUNK, never from the candidate tree. The
    // workflow compiles this binary from a separate trusted checkout precisely
    // so a pull request cannot supply its own ruleset; reading `.gitattributes`
    // out of `candidate` would have handed that control straight back. One
    // line — `some/shared/file.rs merge=union` — would have dropped a shared
    // source path out of the comparison and self-widened the lane, which D-40
    // forbids by name. Reading from `dev` also keeps the verdict symmetric:
    // every open PR's run resolves the same declaration, so a pair cannot
    // disagree about whether they collide.
    // `?`, never a default. "Could not read the policy" is not "there is no
    // policy": defaulting to an empty declaration silently exempts nothing and
    // degrades this gate to its pre-amendment behaviour while every check stays
    // green — which is exactly how the first version of this read shipped dead.
    let attributes = git_blob_text(
        &config.token,
        &["show", &format!("{TRUNK_REF}:.gitattributes")],
    )?;
    let mergeable = declared_mergeable(&attributes).map_err(|error| error.message())?;
    admit_authored(&this, &in_flight, &mergeable).map_err(|error| error.message())
}

fn config_from_env() -> Result<Config, String> {
    let repository = required_env("OYATIE_REPOSITORY")?;
    let Some((owner, repo)) = repository.split_once('/') else {
        return Err("OYATIE_REPOSITORY must be `owner/repo`".to_owned());
    };
    if owner.is_empty()
        || repo.is_empty()
        || repo.contains('/')
        || !repository
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.'))
    {
        return Err("OYATIE_REPOSITORY contains an invalid byte".to_owned());
    }
    let pull_request = pull_request_number(&required_env("GITHUB_REF")?)?;
    Ok(Config {
        repository,
        pull_request,
        token: required_env("GH_TOKEN")?,
    })
}

fn pull_request_number(reference: &str) -> Result<u64, String> {
    let raw = reference
        .strip_prefix("refs/pull/")
        .and_then(|value| value.strip_suffix("/merge"))
        .ok_or_else(|| "GITHUB_REF must be `refs/pull/<positive integer>/merge`".to_owned())?;
    let number = raw
        .parse::<u64>()
        .map_err(|_| "GITHUB_REF must contain a positive pull request number".to_owned())?;
    if number == 0 || raw.starts_with('0') {
        return Err("GITHUB_REF must contain a canonical positive pull request number".to_owned());
    }
    Ok(number)
}

fn required_env(name: &str) -> Result<String, String> {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing non-empty {name}"))
}

fn list_open_pull_requests(repository: &str) -> Result<BTreeSet<u64>, String> {
    let endpoint = format!("repos/{repository}/pulls");
    let mut command = Command::new("gh");
    command.args([
        "api",
        "--method",
        "GET",
        "--paginate",
        endpoint.as_str(),
        "-f",
        "state=open",
        "-f",
        "base=dev",
        "-f",
        "per_page=100",
        "--jq",
        ".[].number",
    ]);
    let output = command_output(command, "list all open pull requests")?;
    let text = String::from_utf8(output.stdout)
        .map_err(|_| "open pull request listing was not UTF-8".to_owned())?;
    let mut pulls = BTreeSet::new();
    for line in text.lines() {
        let number = line
            .parse::<u64>()
            .map_err(|_| format!("invalid pull request number {line:?}"))?;
        if number == 0 || !pulls.insert(number) {
            return Err(format!("invalid or duplicate pull request number {number}"));
        }
    }
    Ok(pulls)
}

fn fetch_trunk_and_pull_heads(pulls: &BTreeSet<u64>, token: &str) -> Result<(), String> {
    let refs: Vec<String> = pulls.iter().copied().map(pull_refspec).collect();
    for (index, batch) in refs.chunks(FETCH_BATCH_SIZE).enumerate() {
        let mut args = vec!["fetch", "--no-tags", REMOTE];
        if index == 0 {
            args.push("+refs/heads/dev:refs/remotes/origin/dev");
        }
        args.extend(batch.iter().map(String::as_str));
        git_output(token, &args)?;
    }
    if refs.is_empty() {
        git_output(
            token,
            &[
                "fetch",
                "--no-tags",
                REMOTE,
                "+refs/heads/dev:refs/remotes/origin/dev",
            ],
        )?;
    }
    Ok(())
}

fn pull_refspec(number: u64) -> String {
    format!("+refs/pull/{number}/head:refs/oyatie-occupancy/pr-{number}")
}

fn pull_head_ref(number: u64) -> String {
    format!("refs/oyatie-occupancy/pr-{number}")
}

fn git_change_paths(token: &str, merge_base: &str, head: &str) -> Result<GitChangePaths, String> {
    let output = git_output(
        token,
        &["diff", "--name-status", "-z", "-M", merge_base, head, "--"],
    )?;
    git_change_paths_from_name_status_z(&output.stdout).map_err(|error| error.message())
}

/// Reads a git BLOB. Deliberately does no object-id validation: the content is
/// arbitrary text. `git_text` cannot serve this purpose — it rejects any byte
/// that is not hex, so `.gitattributes` fails at its first character.
fn git_blob_text(token: &str, args: &[&str]) -> Result<String, String> {
    let output = git_output(token, args)?;
    String::from_utf8(output.stdout)
        .map_err(|_| format!("git {} returned non-UTF-8 output", args[0]))
}

/// Reads a git OBJECT ID, and validates that it is one.
fn git_text(token: &str, args: &[&str]) -> Result<String, String> {
    let output = git_output(token, args)?;
    let value = String::from_utf8(output.stdout)
        .map_err(|_| format!("git {} returned non-UTF-8 output", args[0]))?;
    let value = value.trim_end_matches(['\r', '\n']);
    if !is_object_id(value) {
        return Err(format!("git {} returned an invalid object id", args[0]));
    }
    Ok(value.to_owned())
}

fn is_object_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn git_output(token: &str, args: &[&str]) -> Result<Output, String> {
    let auth = format!(
        "AUTHORIZATION: basic {}",
        base64(format!("x-access-token:{token}").as_bytes())
    );
    let mut command = Command::new("git");
    command
        .args(args)
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "http.https://github.com/.extraheader")
        .env("GIT_CONFIG_VALUE_0", auth);
    command_output(command, &format!("git {}", args[0]))
}

fn command_output(mut command: Command, label: &str) -> Result<Output, String> {
    let output = command
        .output()
        .map_err(|error| format!("start {label}: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("{label} failed: {}", stderr.trim()))
    }
}

fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(a >> 2) as usize] as char);
        encoded.push(TABLE[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(c & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_auth_encoding_matches_git_https() {
        assert_eq!(
            base64(b"x-access-token:test"),
            "eC1hY2Nlc3MtdG9rZW46dGVzdA=="
        );
    }

    #[test]
    fn pull_refspec_has_an_explicit_destination() {
        assert_eq!(
            pull_refspec(2223),
            "+refs/pull/2223/head:refs/oyatie-occupancy/pr-2223"
        );
    }

    #[test]
    fn file_content_is_not_an_object_id() {
        // `git_text` validates its output IS a hex object id, because it was
        // written for `merge-base`. Reading `.gitattributes` through it always
        // failed — at the first byte, `#` — and paired with a defaulting
        // unwrap that made the whole exemption silently empty while every gate
        // stayed green. `git_blob_text` exists for content; this pins why.
        assert!(is_object_id("a355428b265db665a18c29e4fc0a35872fbd0053"));
        assert!(!is_object_id(""));
        assert!(
            !is_object_id("# Cargo.lock is generated"),
            "prose must never pass the object-id validator"
        );
        assert!(!is_object_id("Cargo.lock merge=cargo-lock"));
    }

    #[test]
    fn pull_request_number_comes_from_the_automatic_merge_ref() {
        assert_eq!(pull_request_number("refs/pull/2223/merge"), Ok(2223));
        for reference in [
            "refs/pull/0/merge",
            "refs/pull/02223/merge",
            "refs/pull/2223/head",
            "refs/heads/dev",
            "refs/pull/not-a-number/merge",
        ] {
            assert!(pull_request_number(reference).is_err(), "{reference}");
        }
    }
}
