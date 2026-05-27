//! `oya gen board-sync` (ADR-0377 D3) — deterministic Forgejo board snapshot.
//!
//! The command projects generated masterplan deliverables into the shape that a
//! Forgejo issue/label reconciler would apply. It intentionally produces an
//! idempotent diff/snapshot only: no GitHub Projects, no long-running service,
//! and no network side effects.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{Value, json};

const DEFAULT_MASTER_PLAN: &str = "docs/machine-readable/masterplan.generated.json";
const DEFAULT_SNAPSHOT: &str = "docs/machine-readable/board-sync.generated.json";

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoardSyncArgs {
    master_plan: PathBuf,
    snapshot: PathBuf,
    write: bool,
    check: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoardIssue {
    deliverable_id: String,
    title: String,
    body: String,
    labels: Vec<String>,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match execute(&parsed) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("gen board-sync: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args(args: Vec<String>, usage: &str) -> Result<BoardSyncArgs, String> {
    let mut parsed = BoardSyncArgs {
        master_plan: PathBuf::from(DEFAULT_MASTER_PLAN),
        snapshot: PathBuf::from(DEFAULT_SNAPSHOT),
        write: false,
        check: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--master-plan" => {
                parsed.master_plan = PathBuf::from(iter.next().ok_or_else(|| usage.to_owned())?);
            }
            "--snapshot" | "--output" => {
                parsed.snapshot = PathBuf::from(iter.next().ok_or_else(|| usage.to_owned())?);
            }
            "--write" => parsed.write = true,
            "--check" => parsed.check = true,
            _ => return Err(usage.to_owned()),
        }
    }
    if parsed.write && parsed.check {
        return Err("gen board-sync: --write and --check are mutually exclusive".into());
    }
    Ok(parsed)
}

fn execute(args: &BoardSyncArgs) -> Result<(), String> {
    let snapshot = render_snapshot(&read_issues(&args.master_plan)?)?;
    if args.check {
        let committed = std::fs::read_to_string(&args.snapshot).map_err(|error| {
            format!(
                "snapshot unreadable {}: {error}; run `oya gen board-sync --write`",
                args.snapshot.display()
            )
        })?;
        if committed == snapshot {
            println!(
                "gen board-sync --check passed: {} is current",
                args.snapshot.display()
            );
            return Ok(());
        }
        return Err(format!(
            "snapshot drifted: {}; run `oya gen board-sync --write`",
            args.snapshot.display()
        ));
    }
    if args.write {
        if let Some(parent) = args.snapshot.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "snapshot directory unwritable {}: {error}",
                    parent.display()
                )
            })?;
        }
        std::fs::write(&args.snapshot, &snapshot)
            .map_err(|error| format!("snapshot unwritable {}: {error}", args.snapshot.display()))?;
        println!("gen board-sync wrote {}", args.snapshot.display());
        return Ok(());
    }
    print!("{snapshot}");
    Ok(())
}

fn read_issues(master_plan: &Path) -> Result<Vec<BoardIssue>, String> {
    let raw = std::fs::read_to_string(master_plan)
        .map_err(|error| format!("master plan unreadable {}: {error}", master_plan.display()))?;
    let value: Value = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "master plan JSON invalid {}: {error}",
            master_plan.display()
        )
    })?;
    let mut issues = Vec::new();
    collect_issues(&value, None, &mut issues);
    if issues.is_empty() {
        return Err(format!(
            "master plan contains no deliverables: {}",
            master_plan.display()
        ));
    }
    Ok(issues)
}

fn collect_issues(value: &Value, milestone: Option<&str>, issues: &mut Vec<BoardIssue>) {
    match value {
        Value::Object(object) => {
            let milestone = object
                .get("milestone")
                .and_then(Value::as_str)
                .or(milestone);
            if let Some(Value::Array(deliverables)) = object.get("deliverables") {
                for deliverable in deliverables {
                    if let Some(issue) = issue_from_deliverable(deliverable, milestone) {
                        issues.push(issue);
                    }
                }
            }
            for child in object.values() {
                collect_issues(child, milestone, issues);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_issues(item, milestone, issues);
            }
        }
        _ => {}
    }
}

fn issue_from_deliverable(value: &Value, milestone: Option<&str>) -> Option<BoardIssue> {
    let object = value.as_object()?;
    let id = object.get("id")?.as_str()?.to_string();
    let description = object
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let status = object
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("declared");
    let mut labels = vec![
        format!("state/{}", label_segment(status)),
        "owner/unassigned".into(),
        format!("deliverable/{}", label_segment(&id)),
    ];
    if let Some(milestone) = milestone {
        labels.push(format!("milestone/{}", label_segment(milestone)));
    }
    Some(BoardIssue {
        deliverable_id: id.clone(),
        title: format!("{id}: {description}"),
        body: format!(
            "Generated from masterplan deliverable `{id}`.\n\n{description}\n\n<!-- oya-board-sync:{id} -->\n"
        ),
        labels,
    })
}

fn render_snapshot(issues: &[BoardIssue]) -> Result<String, String> {
    let mut sorted = issues.to_vec();
    sorted.sort_by(|left, right| left.deliverable_id.cmp(&right.deliverable_id));
    let value = json!({
        "_generated": "GENERATED by `oya gen board-sync` from masterplan deliverables. Do not hand-edit.",
        "forgejo_projection": {
            "issue_identity": "deliverable_id",
            "exclusive_label_scopes": ["state", "owner", "deliverable", "milestone"]
        },
        "issues": sorted.iter().map(|issue| json!({
            "deliverable_id": issue.deliverable_id,
            "title": issue.title,
            "body": issue.body,
            "labels": issue.labels
        })).collect::<Vec<_>>()
    });
    serde_json::to_string_pretty(&value)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|error| format!("board snapshot serialization failed: {error}"))
}

fn label_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_sync_snapshot_is_stable_and_uses_exclusive_scopes() {
        let value = json!({
            "milestones": [{
                "milestone": "M-AGENTIC-PIPELINE",
                "adrs": [{
                    "deliverables": [{
                        "id": "ADR-0377-D3",
                        "description": "board sync",
                        "status": "declared"
                    }]
                }]
            }]
        });
        let mut issues = Vec::new();
        collect_issues(&value, None, &mut issues);
        let snapshot = render_snapshot(&issues).expect("snapshot renders");
        assert!(snapshot.contains("\"deliverable_id\": \"ADR-0377-D3\""));
        assert!(snapshot.contains("\"state/declared\""));
        assert!(snapshot.contains("\"owner/unassigned\""));
        assert!(snapshot.contains("\"deliverable/adr-0377-d3\""));
        assert!(snapshot.contains("\"milestone/m-agentic-pipeline\""));
        assert_eq!(snapshot, render_snapshot(&issues).expect("stable rerender"));
    }
}
