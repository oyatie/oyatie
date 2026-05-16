// Purpose: render or check the compact status ledger from
// `specs/masterplan.json`. Validates IPs declare
// `execution_unit=ChangeSet` and `changeset_contract=claimable-verifiable-
// bundleable-promotable`; `--check` mode verifies parity against the on-disk
// ledger. Ported from `scripts/render-master-plan-ledger.py` per
// `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-9.
// Naming-justification: `master_plan_ledger` lives under `commands/doc/`
// mirroring the existing `doc adr-index` / `doc mdbook` renderer family. CLI
// surface `doc render master-plan-ledger` is canonical kebab-case verb-noun
// (ADR-0105 v4 BNF).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_MASTER_PLAN: &str = "specs/masterplan.json";
const DEFAULT_OUTPUT: &str = "evidence/master-plan-ledger.md";
const EXPECTED_EXECUTION_UNIT: &str = "ChangeSet";
const EXPECTED_CHANGESET_CONTRACT: &str = "claimable-verifiable-bundleable-promotable";

pub(super) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    match parse_args(args, usage) {
        Ok(args) => execute(args),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MasterPlanLedgerArgs {
    master_plan_path: PathBuf,
    output: PathBuf,
    check: bool,
    write: bool,
}

fn parse_args(args: Vec<String>, usage: &str) -> Result<MasterPlanLedgerArgs, String> {
    let mut parsed = MasterPlanLedgerArgs {
        master_plan_path: PathBuf::from(DEFAULT_MASTER_PLAN),
        output: PathBuf::from(DEFAULT_OUTPUT),
        check: false,
        write: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--check" => parsed.check = true,
            "--write" => parsed.write = true,
            "--master-plan" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.master_plan_path = PathBuf::from(value);
            }
            "--output" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.output = PathBuf::from(value);
            }
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn execute(args: MasterPlanLedgerArgs) -> ExitCode {
    let master_plan_text = match std::fs::read_to_string(&args.master_plan_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!(
                "render-master-plan-ledger: masterplan unreadable {}: {error}",
                args.master_plan_path.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let repo_root = match resolve_repo_root(&args.master_plan_path) {
        Some(path) => path,
        None => PathBuf::from("."),
    };
    let rendered =
        match render_master_plan_ledger(&master_plan_text, &repo_root, &args.master_plan_path) {
            Ok(text) => text,
            Err(message) => {
                eprintln!("render-master-plan-ledger: {message}");
                return ExitCode::FAILURE;
            }
        };
    if args.write {
        if let Some(parent) = args.output.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            eprintln!(
                "render-master-plan-ledger: output dir unwritable {}: {error}",
                parent.display()
            );
            return ExitCode::FAILURE;
        }
        if let Err(error) = std::fs::write(&args.output, &rendered) {
            eprintln!(
                "render-master-plan-ledger: output unwritable {}: {error}",
                args.output.display()
            );
            return ExitCode::FAILURE;
        }
        println!("wrote {}", args.output.display());
        return ExitCode::SUCCESS;
    }
    if args.check {
        if args.output.exists() {
            match std::fs::read_to_string(&args.output) {
                Ok(current) => {
                    if current != rendered {
                        eprintln!(
                            "render-master-plan-ledger: {} is stale; run with --write",
                            args.output.display()
                        );
                        return ExitCode::FAILURE;
                    }
                    println!("render-master-plan-ledger: output parity ok");
                }
                Err(error) => {
                    eprintln!(
                        "render-master-plan-ledger: output unreadable {}: {error}",
                        args.output.display()
                    );
                    return ExitCode::FAILURE;
                }
            }
        } else {
            println!("render-master-plan-ledger: output absent; source ledger ok");
        }
        return ExitCode::SUCCESS;
    }
    print!("{rendered}");
    ExitCode::SUCCESS
}

fn resolve_repo_root(master_plan_path: &Path) -> Option<PathBuf> {
    master_plan_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

pub(crate) fn render_master_plan_ledger(
    master_plan_text: &str,
    repo_root: &Path,
    master_plan_path: &Path,
) -> Result<String, String> {
    let data: serde_json::Value = serde_json::from_str(master_plan_text)
        .map_err(|error| format!("masterplan JSON invalid: {error}"))?;
    let index = data
        .get("live_implementation_index")
        .ok_or_else(|| "masterplan missing live_implementation_index".to_string())?;
    let errors = validate_index(index, repo_root);
    if !errors.is_empty() {
        return Err(errors
            .iter()
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n"));
    }
    Ok(render_index(index, master_plan_path, repo_root))
}

fn validate_index(index: &serde_json::Value, repo_root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    for (kind, item) in iter_items(index) {
        let item_id = item
            .get("id")
            .and_then(|value| value.as_str())
            .unwrap_or("<missing-id>");
        if item
            .get("status")
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .is_none()
        {
            errors.push(format!("{kind} {item_id} missing status"));
        }
        for key in &["path", "index", "file"] {
            let Some(rel) = item.get(key).and_then(|value| value.as_str()) else {
                continue;
            };
            if rel.is_empty() {
                continue;
            }
            if !repo_root.join(rel).exists() {
                errors.push(format!("{kind} {item_id} references missing {key}: {rel}"));
            }
        }
        if kind == "ip" {
            let execution_unit = item
                .get("execution_unit")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if execution_unit != EXPECTED_EXECUTION_UNIT {
                errors.push(format!("ip {item_id} execution_unit is not ChangeSet"));
            }
            let changeset_contract = item
                .get("changeset_contract")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if changeset_contract != EXPECTED_CHANGESET_CONTRACT {
                errors.push(format!("ip {item_id} changeset_contract is invalid"));
            }
        }
    }
    if let Some(coverage) = index.get("coverage").and_then(|value| value.as_object()) {
        for (key, value) in coverage {
            if key.starts_with("all_") && value != &serde_json::Value::Bool(true) {
                errors.push(format!("coverage {key} is not true"));
            }
        }
    }
    errors
}

fn iter_items(index: &serde_json::Value) -> Vec<(&'static str, &serde_json::Value)> {
    let mut items: Vec<(&'static str, &serde_json::Value)> = Vec::new();
    let milestones = index.get("milestones").and_then(|value| value.as_array());
    let Some(milestones) = milestones else {
        return items;
    };
    for milestone in milestones {
        items.push(("milestone", milestone));
        let Some(phases) = milestone.get("phases").and_then(|value| value.as_array()) else {
            continue;
        };
        for phase in phases {
            items.push(("phase", phase));
            let Some(ips) = phase
                .get("implementation_plans")
                .and_then(|value| value.as_array())
            else {
                continue;
            };
            for ip in ips {
                items.push(("ip", ip));
            }
        }
    }
    items
}

fn render_index(index: &serde_json::Value, master_plan_path: &Path, repo_root: &Path) -> String {
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for (kind, item) in iter_items(index) {
        let status = item
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("<missing>")
            .to_string();
        *counts.entry((kind.to_string(), status)).or_insert(0) += 1;
    }
    let milestone_count = index
        .get("milestone_count")
        .map(format_scalar)
        .unwrap_or_else(|| "null".into());
    let phase_count = index
        .get("phase_count")
        .map(format_scalar)
        .unwrap_or_else(|| "null".into());
    let implementation_plan_count = index
        .get("implementation_plan_count")
        .map(format_scalar)
        .unwrap_or_else(|| "null".into());
    let relative_master_plan = master_plan_path
        .strip_prefix(repo_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| master_plan_path.to_path_buf());

    let mut lines: Vec<String> = vec![
        "# Master Plan Ledger".into(),
        "".into(),
        "<!-- generated by oya doc render master-plan-ledger -->".into(),
        "".into(),
        format!("- Source: `{}`", relative_master_plan.display()),
        format!("- Milestones: {milestone_count}"),
        format!("- Phases: {phase_count}"),
        format!("- Implementation plans: {implementation_plan_count}"),
        "".into(),
        "## Status counts".into(),
        "".into(),
        "| Kind | Status | Count |".into(),
        "|---|---|---:|".into(),
    ];
    for ((kind, status), count) in counts {
        lines.push(format!("| {kind} | {status} | {count} |"));
    }
    lines.push("".into());
    lines.join("\n")
}

fn format_scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".into(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_PLAN: &str = r#"{
        "live_implementation_index": {
            "milestone_count": 1,
            "phase_count": 1,
            "implementation_plan_count": 1,
            "coverage": {"all_phases": true},
            "milestones": [
                {
                    "id": "M-X",
                    "status": "complete",
                    "phases": [
                        {
                            "id": "P-X",
                            "status": "complete",
                            "implementation_plans": [
                                {
                                    "id": "IP-001",
                                    "status": "complete",
                                    "execution_unit": "ChangeSet",
                                    "changeset_contract": "claimable-verifiable-bundleable-promotable"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    }"#;

    #[test]
    fn render_passes_on_valid_plan() {
        let rendered =
            render_master_plan_ledger(VALID_PLAN, Path::new("."), Path::new("masterplan.json"))
                .expect("valid plan must render");
        assert!(rendered.contains("# Master Plan Ledger"));
        assert!(rendered.contains("| ip | complete | 1 |"));
    }

    #[test]
    fn render_rejects_invalid_changeset_contract() {
        let plan = VALID_PLAN.replace(
            "\"changeset_contract\": \"claimable-verifiable-bundleable-promotable\"",
            "\"changeset_contract\": \"bogus\"",
        );
        let error = render_master_plan_ledger(&plan, Path::new("."), Path::new("masterplan.json"))
            .expect_err("invalid contract must fail");
        assert!(error.contains("changeset_contract"));
    }

    #[test]
    fn render_rejects_non_changeset_execution_unit() {
        let plan = VALID_PLAN.replace("\"ChangeSet\"", "\"Other\"");
        let error = render_master_plan_ledger(&plan, Path::new("."), Path::new("masterplan.json"))
            .expect_err("non-ChangeSet must fail");
        assert!(error.contains("execution_unit"));
    }
}
