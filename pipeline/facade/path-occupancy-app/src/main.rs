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

mod git;

use git::{command_output, git_blob_text, git_change_paths, git_output, git_text};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_refspec_has_an_explicit_destination() {
        assert_eq!(
            pull_refspec(2223),
            "+refs/pull/2223/head:refs/oyatie-occupancy/pr-2223"
        );
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
