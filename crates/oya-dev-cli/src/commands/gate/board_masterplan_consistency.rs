use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;

use crate::usage;

const DEFAULT_MASTER_PLAN: &str = "specs/masterplan.json";
const DEFAULT_BOARD_SNAPSHOT: &str = "evidence/board-sync/board-snapshot.json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoardMasterplanConsistencyArgs {
    pub master_plan_path: PathBuf,
    pub board_snapshot_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoardMasterplanConsistencyReport {
    pub masterplan_deliverables_checked: usize,
    pub board_deliverables_checked: usize,
}

pub(crate) fn parse_board_masterplan_consistency_args(
    args: Vec<String>,
) -> Result<BoardMasterplanConsistencyArgs, String> {
    let mut parsed = BoardMasterplanConsistencyArgs {
        master_plan_path: PathBuf::from(DEFAULT_MASTER_PLAN),
        board_snapshot_path: PathBuf::from(DEFAULT_BOARD_SNAPSHOT),
    };

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--check" => {}
            "--master-plan" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.master_plan_path = PathBuf::from(value);
            }
            "--board-snapshot" | "--snapshot" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.board_snapshot_path = PathBuf::from(value);
            }
            _ => return Err(usage()),
        }
    }

    Ok(parsed)
}

pub(crate) fn validate_board_masterplan_consistency(
    args: BoardMasterplanConsistencyArgs,
) -> Result<BoardMasterplanConsistencyReport, String> {
    let master_plan_text = std::fs::read_to_string(&args.master_plan_path).map_err(|error| {
        format!(
            "masterplan unreadable {}: {error}",
            args.master_plan_path.display()
        )
    })?;
    let board_snapshot_text =
        std::fs::read_to_string(&args.board_snapshot_path).map_err(|error| {
            format!(
                "board snapshot unreadable {}: {error}",
                args.board_snapshot_path.display()
            )
        })?;
    validate_board_masterplan_consistency_strings(&master_plan_text, &board_snapshot_text)
}

pub(crate) fn validate_board_masterplan_consistency_strings(
    master_plan_text: &str,
    board_snapshot_text: &str,
) -> Result<BoardMasterplanConsistencyReport, String> {
    let master_plan: Value = serde_json::from_str(master_plan_text)
        .map_err(|error| format!("masterplan JSON invalid: {error}"))?;
    let board_snapshot: Value = serde_json::from_str(board_snapshot_text)
        .map_err(|error| format!("board snapshot JSON invalid: {error}"))?;

    let masterplan_ids = collect_masterplan_deliverable_ids(&master_plan)?;
    let board_ids = collect_board_deliverable_ids(&board_snapshot)?;

    let missing_on_board = masterplan_ids
        .difference(&board_ids)
        .cloned()
        .collect::<Vec<_>>();
    let orphaned_on_board = board_ids
        .difference(&masterplan_ids)
        .cloned()
        .collect::<Vec<_>>();

    let mut errors = Vec::new();
    if !missing_on_board.is_empty() {
        errors.push(format!(
            "masterplan deliverables missing from board snapshot: {}",
            missing_on_board.join(", ")
        ));
    }
    if !orphaned_on_board.is_empty() {
        errors.push(format!(
            "board snapshot deliverables missing from masterplan: {}",
            orphaned_on_board.join(", ")
        ));
    }

    if !errors.is_empty() {
        return Err(errors
            .iter()
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n"));
    }

    Ok(BoardMasterplanConsistencyReport {
        masterplan_deliverables_checked: masterplan_ids.len(),
        board_deliverables_checked: board_ids.len(),
    })
}

fn collect_masterplan_deliverable_ids(master_plan: &Value) -> Result<BTreeSet<String>, String> {
    let index = master_plan
        .get("live_implementation_index")
        .ok_or_else(|| "masterplan missing live_implementation_index".to_string())?;
    let milestones = index
        .get("milestones")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "masterplan live_implementation_index.milestones must be an array".to_string()
        })?;

    let mut ids = BTreeSet::new();
    for milestone in milestones {
        let Some(phases) = milestone.get("phases").and_then(Value::as_array) else {
            continue;
        };
        for phase in phases {
            let Some(implementation_plans) =
                phase.get("implementation_plans").and_then(Value::as_array)
            else {
                continue;
            };
            for plan in implementation_plans {
                let id = plan
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| {
                        "masterplan implementation plan missing non-empty id".to_string()
                    })?;
                ids.insert(id.to_string());
            }
        }
    }
    Ok(ids)
}

fn collect_board_deliverable_ids(snapshot: &Value) -> Result<BTreeSet<String>, String> {
    let items = snapshot
        .get("issues")
        .or_else(|| snapshot.get("items"))
        .or_else(|| snapshot.get("cards"))
        .and_then(Value::as_array)
        .ok_or_else(|| "board snapshot must contain issues, items, or cards array".to_string())?;

    let mut ids = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let extracted = extract_board_item_deliverable_id(item).ok_or_else(|| {
            format!(
                "board snapshot item #{index} missing deliverable id (expected direct field or label prefix masterplan:/deliverable:/ip:)"
            )
        })?;
        ids.insert(extracted);
    }
    Ok(ids)
}

fn extract_board_item_deliverable_id(item: &Value) -> Option<String> {
    for key in [
        "deliverable_id",
        "masterplan_id",
        "implementation_plan_id",
        "ip_id",
        "masterplan_deliverable_id",
    ] {
        if let Some(id) = item.get(key).and_then(Value::as_str).and_then(clean_id) {
            return Some(id);
        }
    }

    for key in ["deliverable", "masterplan", "implementation_plan"] {
        if let Some(id) = item
            .get(key)
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str)
            .and_then(clean_id)
        {
            return Some(id);
        }
    }

    if let Some(labels) = item.get("labels").and_then(Value::as_array) {
        for label in labels {
            let label_name = label
                .as_str()
                .or_else(|| label.get("name").and_then(Value::as_str));
            if let Some(id) = label_name.and_then(parse_labeled_id) {
                return Some(id);
            }
        }
    }

    for key in ["body", "description", "title"] {
        if let Some(id) = item
            .get(key)
            .and_then(Value::as_str)
            .and_then(parse_embedded_id)
        {
            return Some(id);
        }
    }

    None
}

fn parse_labeled_id(label: &str) -> Option<String> {
    let normalized = label.trim();
    for prefix in [
        "masterplan:",
        "masterplan/",
        "deliverable:",
        "deliverable/",
        "ip:",
        "ip/",
        "implementation-plan:",
        "implementation-plan/",
    ] {
        if let Some(value) = normalized.strip_prefix(prefix) {
            return clean_id(value);
        }
    }
    None
}

fn parse_embedded_id(text: &str) -> Option<String> {
    for marker in ["masterplan:", "deliverable:", "ip:", "masterplan_id="] {
        if let Some((_, suffix)) = text.split_once(marker) {
            let id = suffix
                .split(|ch: char| {
                    ch.is_whitespace() || ch == ']' || ch == ')' || ch == ',' || ch == ';'
                })
                .next()
                .unwrap_or_default();
            if let Some(id) = clean_id(id) {
                return Some(id);
            }
        }
    }
    None
}

fn clean_id(value: &str) -> Option<String> {
    let cleaned = value
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAN: &str = r#"{
      "live_implementation_index": {
        "milestones": [{
          "phases": [{
            "id": "P-01",
            "implementation_plans": [
              {"id":"IP-001"},
              {"id":"IP-002"}
            ]
          }]
        }]
      }
    }"#;

    #[test]
    fn passes_when_board_snapshot_matches_masterplan_deliverables() {
        let board = r#"{
          "issues": [
            {"number": 1, "deliverable_id": "IP-001"},
            {"number": 2, "labels": [{"name":"masterplan:IP-002"}]}
          ]
        }"#;
        let report = validate_board_masterplan_consistency_strings(PLAN, board).expect("match");
        assert_eq!(report.masterplan_deliverables_checked, 2);
        assert_eq!(report.board_deliverables_checked, 2);
    }

    #[test]
    fn rejects_masterplan_deliverable_missing_from_board_snapshot() {
        let board = r#"{"issues":[{"number":1,"deliverable_id":"IP-001"}]}"#;
        let error =
            validate_board_masterplan_consistency_strings(PLAN, board).expect_err("missing");
        assert!(error.contains("masterplan deliverables missing from board snapshot"));
        assert!(error.contains("IP-002"));
    }

    #[test]
    fn rejects_board_deliverable_missing_from_masterplan() {
        let board = r#"{
          "issues": [
            {"number": 1, "deliverable_id": "IP-001"},
            {"number": 2, "deliverable_id": "IP-002"},
            {"number": 3, "deliverable_id": "IP-999"}
          ]
        }"#;
        let error = validate_board_masterplan_consistency_strings(PLAN, board).expect_err("orphan");
        assert!(error.contains("board snapshot deliverables missing from masterplan"));
        assert!(error.contains("IP-999"));
    }
}
