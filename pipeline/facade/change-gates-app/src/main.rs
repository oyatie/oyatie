use std::process::ExitCode;

use pipeline_admission::{
    CadenceEvent, git_change_paths_from_name_status_z, presubmit_change_gates,
};
use pipeline_repository_draft::RepositoryRead;
use pipeline_repository_git_draft::GitRepository;

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let repository = GitRepository;
    evaluate(std::env::args().skip(1), |base, head| {
        repository.changed_name_status(base, head)
    })
}

fn evaluate(
    arguments: impl IntoIterator<Item = String>,
    changed_name_status: impl FnOnce(&str, &str) -> Result<Vec<u8>, String>,
) -> Result<String, String> {
    let request = Request::parse(arguments)?;
    let raw_changes = changed_name_status(&request.base, &request.head)?;
    let changes = git_change_paths_from_name_status_z(&raw_changes)
        .map_err(|error| format!("classify repository change: {}", error.message()))?;
    let gates = presubmit_change_gates(request.event, changes.occupied.iter().map(String::as_str));
    Ok(format!(
        "live={}\nreindeer={}\n",
        gates.live_postgres(),
        gates.reindeer_source_qualification()
    ))
}

struct Request {
    event: CadenceEvent,
    base: String,
    head: String,
}

impl Request {
    fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut arguments = arguments.into_iter();
        let event = arguments.next().ok_or_else(usage)?;
        let base = arguments.next().ok_or_else(usage)?;
        let head = arguments.next().ok_or_else(usage)?;
        if arguments.next().is_some() {
            return Err(usage());
        }
        Ok(Self {
            event: protected_event(&event)?,
            base: object_id("base", base)?,
            head: object_id("head", head)?,
        })
    }
}

fn protected_event(value: &str) -> Result<CadenceEvent, String> {
    match value {
        "pull_request" => Ok(CadenceEvent::PullRequest),
        "merge_group" => Ok(CadenceEvent::MergeGroup),
        _ => Err(format!(
            "unsupported protected admission event {value:?}; expected pull_request or merge_group"
        )),
    }
}

fn object_id(name: &str, value: String) -> Result<String, String> {
    if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(format!("{name} must be a 40-digit Git object id"))
    }
}

fn usage() -> String {
    "usage: pipeline-change-gates-app <pull_request|merge_group> <base-sha> <head-sha>".to_owned()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    const BASE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HEAD: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn output(event: &str, changes: &[u8]) -> String {
        evaluate(
            [event, BASE, HEAD].into_iter().map(str::to_owned),
            |_, _| Ok(changes.to_vec()),
        )
        .unwrap()
    }

    #[test]
    fn one_repository_scan_produces_both_outputs() {
        let calls = Cell::new(0);
        let result = evaluate(
            ["pull_request", BASE, HEAD].into_iter().map(str::to_owned),
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(
                    b"M\0iam/adapters/identity-scim-store-postgres/src/lib.rs\0M\0Cargo.lock\0"
                        .to_vec(),
                )
            },
        )
        .unwrap();

        assert_eq!(calls.get(), 1);
        assert_eq!(result, "live=true\nreindeer=true\n");
    }

    #[test]
    fn deleting_a_qualification_input_requires_proof() {
        assert_eq!(
            output("pull_request", b"D\0Cargo.lock\0"),
            "live=false\nreindeer=true\n"
        );
    }

    #[test]
    fn renaming_into_the_qualification_closure_requires_proof() {
        assert_eq!(
            output("pull_request", b"R100\0docs/old.lock\0Cargo.lock\0"),
            "live=false\nreindeer=true\n"
        );
    }

    #[test]
    fn renaming_out_of_the_qualification_closure_requires_proof() {
        assert_eq!(
            output("pull_request", b"R100\0Cargo.lock\0docs/old.lock\0"),
            "live=false\nreindeer=true\n"
        );
    }

    #[test]
    fn control_characters_remain_inside_one_path() {
        let changes = b"M\0build/dependency-declarations/adapters/generation-reindeer/src/line\nwith\ttab.rs\0";
        assert_eq!(
            output("pull_request", changes),
            "live=false\nreindeer=true\n"
        );
        assert_eq!(
            output("pull_request", b"M\0docs/line\nwith\ttab.rs\0"),
            "live=false\nreindeer=false\n"
        );
    }

    #[test]
    fn pull_request_and_merge_group_use_the_same_closed_classifier() {
        let changes = b"M\0rust-toolchain.toml\0";
        for event in ["pull_request", "merge_group"] {
            assert_eq!(
                output(event, changes),
                "live=false\nreindeer=true\n",
                "{event}"
            );
        }
    }

    #[test]
    fn malformed_change_stream_refuses_without_output() {
        let result = evaluate(
            ["pull_request", BASE, HEAD].into_iter().map(str::to_owned),
            |_, _| Ok(b"M\0Cargo.lock".to_vec()),
        );
        assert!(result.unwrap_err().contains("not NUL-terminated"));
    }

    #[test]
    fn unsupported_event_refuses_before_reading_the_repository() {
        let calls = Cell::new(0);
        let result = evaluate(
            ["push", BASE, HEAD].into_iter().map(str::to_owned),
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(Vec::new())
            },
        );

        assert!(
            result
                .unwrap_err()
                .contains("unsupported protected admission event")
        );
        assert_eq!(calls.get(), 0);
    }
}
