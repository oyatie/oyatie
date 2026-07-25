//! Generic fail-closed gate for the active append-only ADR census epoch.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use ci_scm_facts_snapshot::{
    ADR_CENSUS_EPOCH_RECEIPT_PATH, select_census_event_from_event,
    validate_adr_census_epoch_receipt_for_event, validate_census_event_transition,
    validate_dormant_p3_epoch_policy_for_event,
};

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("adr-census-epoch-receipt-gate: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut repo_root = None;
    let mut github_event = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo-root" => {
                index += 1;
                repo_root = Some(PathBuf::from(
                    args.get(index).ok_or("--repo-root requires a path")?,
                ));
            }
            "--github-event" => github_event = true,
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }
    if !github_event {
        return Err(
            "--github-event is required; protected gate binaries never fall back to ambient HEAD"
                .to_owned(),
        );
    }
    let repo_root = repo_root.unwrap_or_else(repo_root_from_current_dir);
    validate_gate_from_event(&repo_root)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GateEventContext {
    protected: String,
    evaluated: String,
    event: String,
    event_ref: String,
    base_ref: String,
    subject: String,
}

fn required_event(
    read: &mut impl FnMut(&str) -> Option<String>,
    key: &str,
) -> Result<String, String> {
    read(key)
        .ok_or_else(|| format!("{key} must be set for --github-event"))
        .and_then(|value| {
            if value.is_empty() {
                Err(format!("{key} must be non-empty for --github-event"))
            } else {
                Ok(value)
            }
        })
}

fn event_context_from_facts(
    mut read: impl FnMut(&str) -> Option<String>,
) -> Result<GateEventContext, String> {
    let event = required_event(&mut read, "EVENT_NAME")?;
    let event_ref = required_event(&mut read, "EVENT_REF")?;
    let evaluated = required_event(&mut read, "EVENT_EVALUATED_SHA")?;
    let (protected, base_ref, subject) = match event.as_str() {
        "pull_request" => (
            required_event(&mut read, "EVENT_PULL_REQUEST_BASE_SHA")?,
            required_event(&mut read, "EVENT_PULL_REQUEST_BASE_REF")?,
            required_event(&mut read, "EVENT_PULL_REQUEST_HEAD_SHA")?,
        ),
        "push" => {
            let protected = required_event(&mut read, "EVENT_PUSH_BEFORE_SHA")?;
            let provider_head = required_event(&mut read, "EVENT_PUSH_AFTER_SHA")?;
            if provider_head != evaluated {
                return Err("provider event head must equal EVENT_EVALUATED_SHA".to_owned());
            }
            (protected, "refs/heads/dev".to_owned(), provider_head)
        }
        "merge_group" => {
            let protected = required_event(&mut read, "EVENT_MERGE_GROUP_BASE_SHA")?;
            let provider_head = required_event(&mut read, "EVENT_MERGE_GROUP_HEAD_SHA")?;
            let base_ref = required_event(&mut read, "EVENT_MERGE_GROUP_BASE_REF")?;
            if provider_head != evaluated {
                return Err("provider event head must equal EVENT_EVALUATED_SHA".to_owned());
            }
            (protected, base_ref, provider_head)
        }
        _ => return Err("EVENT_NAME must be pull_request, push, or merge_group".to_owned()),
    };
    Ok(GateEventContext {
        protected,
        evaluated,
        event,
        event_ref,
        base_ref,
        subject,
    })
}

fn validate_gate_from_event(repo_root: &Path) -> Result<(), String> {
    let context = event_context_from_facts(|key| std::env::var(key).ok())?;
    let selection = select_census_event_from_event(
        repo_root,
        &context.protected,
        &context.evaluated,
        &context.event,
        &context.event_ref,
        &context.base_ref,
        &context.subject,
    )?;
    let validated = validate_census_event_transition(&selection)?;
    validate_adr_census_epoch_receipt_for_event(
        &validated,
        &repo_root.join(ADR_CENSUS_EPOCH_RECEIPT_PATH),
    )?;
    validate_dormant_p3_epoch_policy_for_event(&validated)
}

fn repo_root_from_current_dir() -> PathBuf {
    let mut directory = std::env::current_dir().unwrap_or_else(|error| {
        panic!("adr-census-epoch-receipt-gate: resolve current directory: {error}")
    });
    for _ in 0..16 {
        if directory.join("specs/root-hub-pointers.json").is_file() {
            return directory;
        }
        if !directory.pop() {
            break;
        }
    }
    panic!("adr-census-epoch-receipt-gate: repository root not found")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn oid(byte: char) -> String {
        byte.to_string().repeat(40)
    }

    fn common_facts(event: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("EVENT_NAME".to_owned(), event.to_owned()),
            (
                "EVENT_REF".to_owned(),
                match event {
                    "pull_request" => "refs/pull/1376/merge",
                    "push" => "refs/heads/dev",
                    "merge_group" => "refs/heads/gh-readonly-queue/dev/pr-1376",
                    _ => "refs/unknown",
                }
                .to_owned(),
            ),
            ("EVENT_EVALUATED_SHA".to_owned(), oid('b')),
        ])
    }

    fn parse(facts: &BTreeMap<String, String>) -> Result<GateEventContext, String> {
        event_context_from_facts(|key| facts.get(key).cloned())
    }

    #[test]
    fn protected_binary_requires_explicit_github_event_mode() {
        assert!(run(Vec::new()).is_err());
    }

    #[test]
    fn event_context_rejects_missing_and_empty_required_facts() {
        let facts = BTreeMap::from([("EVENT_REF".to_owned(), "refs/heads/dev".to_owned())]);
        let missing = parse(&facts).expect_err("missing event name must fail closed");
        assert!(missing.contains("EVENT_NAME must be set"), "{missing}");

        let mut facts = common_facts("pull_request");
        facts.insert("EVENT_PULL_REQUEST_BASE_SHA".to_owned(), String::new());
        facts.insert("EVENT_PULL_REQUEST_HEAD_SHA".to_owned(), oid('c'));
        facts.insert("EVENT_PULL_REQUEST_BASE_REF".to_owned(), "dev".to_owned());
        let empty = parse(&facts).expect_err("empty protected base must fail closed");
        assert!(
            empty.contains("EVENT_PULL_REQUEST_BASE_SHA must be non-empty"),
            "{empty}"
        );
    }

    #[test]
    fn event_context_rejects_unsupported_events_and_provider_head_mismatch() {
        let unsupported =
            parse(&common_facts("workflow_dispatch")).expect_err("unsupported event must fail");
        assert!(
            unsupported.contains("EVENT_NAME must be pull_request, push, or merge_group"),
            "{unsupported}"
        );

        let mut facts = common_facts("push");
        facts.insert("EVENT_PUSH_BEFORE_SHA".to_owned(), oid('a'));
        facts.insert("EVENT_PUSH_AFTER_SHA".to_owned(), oid('c'));
        let mismatch = parse(&facts).expect_err("provider head mismatch must fail");
        assert!(
            mismatch.contains("provider event head must equal EVENT_EVALUATED_SHA"),
            "{mismatch}"
        );
    }

    #[test]
    fn event_context_maps_pull_request_provider_tuple() {
        let mut facts = common_facts("pull_request");
        facts.insert("EVENT_PULL_REQUEST_BASE_SHA".to_owned(), oid('a'));
        facts.insert("EVENT_PULL_REQUEST_HEAD_SHA".to_owned(), oid('c'));
        facts.insert("EVENT_PULL_REQUEST_BASE_REF".to_owned(), "dev".to_owned());
        assert_eq!(
            parse(&facts).expect("parse pull request tuple"),
            GateEventContext {
                protected: oid('a'),
                evaluated: oid('b'),
                event: "pull_request".to_owned(),
                event_ref: "refs/pull/1376/merge".to_owned(),
                base_ref: "dev".to_owned(),
                subject: oid('c'),
            }
        );
    }

    #[test]
    fn event_context_maps_push_provider_tuple() {
        let mut facts = common_facts("push");
        facts.insert("EVENT_PUSH_BEFORE_SHA".to_owned(), oid('a'));
        facts.insert("EVENT_PUSH_AFTER_SHA".to_owned(), oid('b'));
        assert_eq!(
            parse(&facts).expect("parse push tuple"),
            GateEventContext {
                protected: oid('a'),
                evaluated: oid('b'),
                event: "push".to_owned(),
                event_ref: "refs/heads/dev".to_owned(),
                base_ref: "refs/heads/dev".to_owned(),
                subject: oid('b'),
            }
        );
    }

    #[test]
    fn event_context_maps_merge_group_provider_tuple() {
        let mut facts = common_facts("merge_group");
        facts.insert("EVENT_MERGE_GROUP_BASE_SHA".to_owned(), oid('a'));
        facts.insert("EVENT_MERGE_GROUP_HEAD_SHA".to_owned(), oid('b'));
        facts.insert(
            "EVENT_MERGE_GROUP_BASE_REF".to_owned(),
            "refs/heads/dev".to_owned(),
        );
        assert_eq!(
            parse(&facts).expect("parse merge-group tuple"),
            GateEventContext {
                protected: oid('a'),
                evaluated: oid('b'),
                event: "merge_group".to_owned(),
                event_ref: "refs/heads/gh-readonly-queue/dev/pr-1376".to_owned(),
                base_ref: "refs/heads/dev".to_owned(),
                subject: oid('b'),
            }
        );
    }
}
