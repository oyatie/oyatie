#![forbid(unsafe_code)]

//! Pure planning projections consumed by the cloud-ci generated-artifact controller.
//!
//! This module deliberately accepts an already-derived masterplan projection. The controller
//! materializes that projection from the candidate tree before invoking this function, keeping
//! filesystem/process concerns at the edge while preserving the legacy board wire format exactly.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoardIssue {
    deliverable_id: String,
    title: String,
    body: String,
    labels: Vec<String>,
}

/// Render the controller-owned board projection from a masterplan projection.
///
/// The output is byte-stable and preserves the existing
/// `docs/machine-readable/board-sync.generated.json` wire shape.
///
/// # Errors
///
/// Returns an error when the input contains no deliverables, malformed deliverables, or duplicate
/// identities. This prevents a malformed or mismatched masterplan input from materializing a
/// partial or ambiguous board projection.
pub fn render_board_sync_projection(masterplan: &Value) -> Result<String, String> {
    let mut issues = Vec::new();
    collect_issues(masterplan, None, &mut issues)?;
    if issues.is_empty() {
        return Err("masterplan contains no deliverables".to_owned());
    }
    let mut deliverable_ids = BTreeSet::new();
    for issue in &issues {
        if !deliverable_ids.insert(issue.deliverable_id.as_str()) {
            return Err(format!(
                "masterplan contains duplicate deliverable_id {}",
                issue.deliverable_id
            ));
        }
    }
    render_snapshot(&issues)
}

fn collect_issues(
    value: &Value,
    milestone: Option<&str>,
    issues: &mut Vec<BoardIssue>,
) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            let milestone = object
                .get("milestone")
                .and_then(Value::as_str)
                .or(milestone);
            if let Some(deliverables) = object.get("deliverables") {
                let deliverables = deliverables
                    .as_array()
                    .ok_or_else(|| "masterplan deliverables must be an array".to_owned())?;
                for deliverable in deliverables {
                    issues.push(issue_from_deliverable(deliverable, milestone)?);
                }
            }
            for child in object.values() {
                collect_issues(child, milestone, issues)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_issues(item, milestone, issues)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn issue_from_deliverable(value: &Value, milestone: Option<&str>) -> Result<BoardIssue, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "masterplan deliverable must be an object".to_owned())?;
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "masterplan deliverable missing string id".to_owned())?
        .to_owned();
    let description = object
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let status = object
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("declared");
    let mut labels = vec![
        format!("state/{}", label_segment(status)),
        "owner/unassigned".to_owned(),
        format!("deliverable/{}", label_segment(&id)),
    ];
    if let Some(milestone) = milestone {
        labels.push(format!("milestone/{}", label_segment(milestone)));
    }
    Ok(BoardIssue {
        title: format!("{id}: {description}"),
        body: format!(
            "Generated from masterplan deliverable `{id}`.\n\n{description}\n\n<!-- oya-board-sync:{id} -->\n"
        ),
        deliverable_id: id,
        labels,
    })
}

fn render_snapshot(issues: &[BoardIssue]) -> Result<String, String> {
    let mut sorted = issues.to_vec();
    sorted.sort_by(|left, right| left.deliverable_id.cmp(&right.deliverable_id));
    // The committed board projection is the canonical wire shape. Use BTreeMap rather than the
    // serde_json macro so this remains byte-stable even when serde_json is built with
    // `preserve_order`; the wire keys are lexicographically ordered in the historical artifact.
    let issues = sorted
        .iter()
        .map(|issue| {
            Value::Object(
                BTreeMap::from([
                    ("body".to_owned(), Value::String(issue.body.clone())),
                    (
                        "deliverable_id".to_owned(),
                        Value::String(issue.deliverable_id.clone()),
                    ),
                    (
                        "labels".to_owned(),
                        Value::Array(issue.labels.iter().cloned().map(Value::String).collect()),
                    ),
                    ("title".to_owned(), Value::String(issue.title.clone())),
                ])
                .into_iter()
                .collect(),
            )
        })
        .collect();
    let value = Value::Object(
        BTreeMap::from([
            (
                "_generated".to_owned(),
                Value::String(
                    "GENERATED by `oya gen board-sync` from masterplan deliverables. Do not hand-edit."
                        .to_owned(),
                ),
            ),
            (
                "github_projection".to_owned(),
                Value::Object(
                    BTreeMap::from([
                        (
                            "exclusive_label_scopes".to_owned(),
                            Value::Array(
                                ["state", "owner", "deliverable", "milestone"]
                                    .into_iter()
                                    .map(|scope| Value::String(scope.to_owned()))
                                    .collect(),
                            ),
                        ),
                        (
                            "issue_identity".to_owned(),
                            Value::String("deliverable_id".to_owned()),
                        ),
                    ])
                    .into_iter()
                    .collect(),
                ),
            ),
            ("issues".to_owned(), Value::Array(issues)),
        ])
        .into_iter()
        .collect(),
    );
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
    use serde_json::json;

    use super::render_board_sync_projection;

    #[test]
    fn board_sync_projection_is_byte_stable_and_uses_the_legacy_wire_shape() {
        let masterplan = json!({
            "milestones": [
                {
                    "milestone": "M-AGENTIC-PIPELINE",
                    "adrs": [{
                        "deliverables": [{
                            "id": "ADR-0377-D3",
                            "description": "board sync",
                            "status": "declared"
                        }]
                    }]
                },
                {
                    "milestone": "M-ALPHA",
                    "adrs": [{
                        "deliverables": [{
                            "id": "A-1",
                            "description": "first item",
                            "status": "in progress"
                        }]
                    }]
                }
            ]
        });

        let rendered = render_board_sync_projection(&masterplan).expect("projection renders");

        assert_eq!(
            rendered,
            include_str!("fixtures/board-sync-legacy-canonical.json")
        );
    }

    #[test]
    fn board_sync_projection_rejects_a_masterplan_without_deliverables() {
        let error = render_board_sync_projection(&json!({"milestones": []}))
            .expect_err("empty input must fail closed");

        assert!(error.contains("contains no deliverables"));
    }

    #[test]
    fn malformed_deliverable_entries_do_not_materialize_an_empty_projection() {
        let error = render_board_sync_projection(&json!({
            "milestones": [{"deliverables": [{"description": "missing id"}]}]
        }))
        .expect_err("malformed deliverables must fail closed");

        assert_eq!(error, "masterplan deliverable missing string id");
    }

    #[test]
    fn malformed_deliverable_cannot_be_hidden_among_valid_deliverables() {
        let error = render_board_sync_projection(&json!({
            "milestones": [{
                "milestone": "M-test",
                "adrs": [{
                    "deliverables": [
                        {"id": "D-1", "description": "valid", "status": "declared"},
                        {"description": "missing id"}
                    ]
                }]
            }]
        }))
        .expect_err("a mixed valid and malformed deliverable set must fail closed");

        assert!(error.contains("missing string id"));
    }

    #[test]
    fn duplicate_deliverable_ids_fail_closed() {
        let error = render_board_sync_projection(&json!({
            "milestones": [{
                "milestone": "M-test",
                "adrs": [{
                    "deliverables": [
                        {"id": "D-1", "description": "first", "status": "declared"},
                        {"id": "D-1", "description": "second", "status": "declared"}
                    ]
                }]
            }]
        }))
        .expect_err("duplicate identity must fail closed");

        assert_eq!(error, "masterplan contains duplicate deliverable_id D-1");
    }
}
