//! Deterministic controller-owned masterplan projection.
//!
//! This is the owned Rust producer behind the controller materializer and the retirement-marked
//! development CLI adapter. Keeping the projection core here lets Cargo tests derive an ignored
//! face in temporary storage without creating a second generator or treating a CLI as authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Map, Value};

const DELIVERABLE_STATUS_V1: &str = "declared";

/// One deliverable parsed from an ADR planning front-matter block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningDeliverable {
    /// Stable deliverable identity.
    pub id: String,
    /// Human-readable deliverable description.
    pub description: String,
    /// Evidence-backed completion criterion.
    pub exit_criteria: String,
    /// Validator or evidence reference that proves completion.
    pub verified_by: String,
}

/// Planning fields parsed from one ADR.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningAdr {
    /// ADR identity derived from its canonical filename.
    pub id: String,
    /// ADR lifecycle status.
    pub status: String,
    /// Projection milestone.
    pub milestone: String,
    /// ADR identities that precede this ADR.
    pub depends_on: Vec<String>,
    /// Whether the ADR declared a `deliverables` field, including an empty field.
    pub has_deliverables_field: bool,
    /// Parsed planning deliverables.
    pub deliverables: Vec<PlanningDeliverable>,
    /// Canonical repository-relative ADR path.
    pub path: String,
}

/// A deterministic generated masterplan projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MasterplanProjection {
    /// Milestones in stable dependency order.
    pub milestones: Vec<MilestoneGroup>,
    /// Number of accepted planning ADRs in the projection.
    pub adr_count: usize,
    /// Number of deliverables in the projection.
    pub deliverable_count: usize,
}

/// Projected ADRs grouped under one milestone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneGroup {
    /// Milestone identity.
    pub milestone: String,
    /// ADRs assigned to the milestone.
    pub adrs: Vec<ProjectedAdr>,
}

/// One ADR in the generated projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedAdr {
    /// ADR identity.
    pub id: String,
    /// ADR lifecycle status.
    pub status: String,
    /// Projected dependency identities.
    pub depends_on: Vec<String>,
    /// Projected deliverables.
    pub deliverables: Vec<ProjectedDeliverable>,
}

/// One deliverable in the generated projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedDeliverable {
    /// Stable deliverable identity.
    pub id: String,
    /// Human-readable deliverable description.
    pub description: String,
    /// Evidence-backed completion criterion.
    pub exit_criteria: String,
    /// Validator or evidence reference that proves completion.
    pub verified_by: String,
}

/// Parse one canonical ADR document when it contributes to the masterplan projection.
///
/// Returns `None` for documents without front-matter or without `planning_impact: true`.
pub fn parse_planning_impact_adr(source_path: &Path, contents: &str) -> Option<PlanningAdr> {
    let frontmatter = read_frontmatter(contents)?;
    if frontmatter_scalar(frontmatter, "planning_impact").as_deref() != Some("true") {
        return None;
    }
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let id = file_name.get(0..8).unwrap_or_default().to_owned();
    Some(parse_planning_adr(
        frontmatter,
        id,
        format!("docs/decisions/{file_name}"),
    ))
}

/// Extract the YAML-subset front-matter between leading `---` fences.
pub fn read_frontmatter(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Read an unindented scalar from an ADR front-matter block.
pub fn frontmatter_scalar(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((found_key, value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() == key {
            return Some(clean_scalar(value));
        }
    }
    None
}

/// Read an inline or indented block list from an ADR front-matter block.
pub fn frontmatter_list(frontmatter: &str, key: &str) -> Vec<String> {
    let mut lines = frontmatter.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((found_key, value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() != key {
            continue;
        }
        let value = value.trim();
        if value.starts_with('[') {
            return parse_inline_list(value);
        }
        if !value.is_empty() {
            return vec![clean_scalar(value)];
        }
        let mut items = Vec::new();
        while let Some(next) = lines.peek() {
            let trimmed = next.trim_start();
            let indented = next.starts_with(char::is_whitespace);
            if let Some(item) = trimmed.strip_prefix("- ") {
                if !indented {
                    break;
                }
                items.push(clean_scalar(item));
                lines.next();
            } else if indented && trimmed.is_empty() {
                lines.next();
            } else {
                break;
            }
        }
        return items;
    }
    Vec::new()
}

/// Render the masterplan projection from parsed canonical ADR planning inputs.
///
/// The returned JSON includes one trailing newline and is byte-deterministic for identical source
/// inputs. This function performs no writes; the controller or test adapter owns output placement.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn render_masterplan_projection(
    adrs: &[PlanningAdr],
) -> Result<(MasterplanProjection, String), String> {
    let projection = generate_masterplan_projection(adrs);
    let json = serde_json::to_string_pretty(&projection_to_json(&projection))
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|error| format!("projection serialization failed: {error}"))?;
    Ok((projection, json))
}

fn parse_planning_adr(frontmatter: &str, id: String, path: String) -> PlanningAdr {
    let status = frontmatter_scalar(frontmatter, "status").unwrap_or_default();
    let milestone = frontmatter_scalar(frontmatter, "milestone").unwrap_or_default();
    let depends_on = frontmatter_list(frontmatter, "depends_on");
    let (has_deliverables_field, deliverables) = parse_deliverables(frontmatter);
    PlanningAdr {
        id,
        status,
        milestone,
        depends_on,
        has_deliverables_field,
        deliverables,
        path,
    }
}

fn parse_inline_list(value: &str) -> Vec<String> {
    let inner = value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(clean_scalar)
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_deliverables(frontmatter: &str) -> (bool, Vec<PlanningDeliverable>) {
    let mut lines = frontmatter.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with(char::is_whitespace) || line.starts_with('#') {
            continue;
        }
        let Some((found_key, value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() != "deliverables" {
            continue;
        }
        if value.trim().starts_with('[') {
            return (true, Vec::new());
        }
        let mut records: Vec<Vec<(String, String)>> = Vec::new();
        let mut current: Option<Vec<(String, String)>> = None;
        while let Some(next) = lines.peek() {
            if !next.starts_with(char::is_whitespace) || next.trim().is_empty() {
                if next.trim().is_empty() {
                    lines.next();
                    continue;
                }
                break;
            }
            let trimmed = next.trim_start();
            if let Some(rest) = trimmed.strip_prefix("- ") {
                if let Some(record) = current.take() {
                    records.push(record);
                }
                let mut record = Vec::new();
                if let Some((key, value)) = rest.split_once(':') {
                    record.push((key.trim().to_owned(), clean_scalar(value)));
                }
                current = Some(record);
            } else if let Some((key, value)) = trimmed.split_once(':')
                && let Some(record) = current.as_mut()
            {
                record.push((key.trim().to_owned(), clean_scalar(value)));
            }
            lines.next();
        }
        if let Some(record) = current {
            records.push(record);
        }
        let deliverables = records
            .into_iter()
            .map(|record| {
                let get = |key: &str| {
                    record
                        .iter()
                        .find(|(found, _)| found == key)
                        .map(|(_, value)| value.clone())
                        .unwrap_or_default()
                };
                PlanningDeliverable {
                    id: get("id"),
                    description: get("description"),
                    exit_criteria: get("exit_criteria"),
                    verified_by: get("verified_by"),
                }
            })
            .collect();
        return (true, deliverables);
    }
    (false, Vec::new())
}

fn clean_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned()
}

fn generate_masterplan_projection(adrs: &[PlanningAdr]) -> MasterplanProjection {
    let accepted: Vec<PlanningAdr> = adrs
        .iter()
        .filter(|adr| is_accepted_planning_status(&adr.status))
        .cloned()
        .collect();
    let ordered = topo_sort(&accepted);
    let mut deliverable_count = 0usize;
    let mut milestone_order = Vec::new();
    let mut grouped: BTreeMap<String, Vec<ProjectedAdr>> = BTreeMap::new();

    for adr in &ordered {
        let milestone = if adr.milestone.trim().is_empty() {
            "UNASSIGNED".to_owned()
        } else {
            adr.milestone.clone()
        };
        if !milestone_order.contains(&milestone) {
            milestone_order.push(milestone.clone());
        }
        let deliverables: Vec<ProjectedDeliverable> = adr
            .deliverables
            .iter()
            .map(|deliverable| ProjectedDeliverable {
                id: deliverable.id.clone(),
                description: deliverable.description.clone(),
                exit_criteria: deliverable.exit_criteria.clone(),
                verified_by: deliverable.verified_by.clone(),
            })
            .collect();
        deliverable_count += deliverables.len();
        grouped.entry(milestone).or_default().push(ProjectedAdr {
            id: adr.id.clone(),
            status: adr.status.clone(),
            depends_on: adr.depends_on.clone(),
            deliverables,
        });
    }

    let milestones = milestone_order
        .into_iter()
        .map(|milestone| MilestoneGroup {
            adrs: grouped.remove(&milestone).unwrap_or_default(),
            milestone,
        })
        .collect();
    MasterplanProjection {
        milestones,
        adr_count: ordered.len(),
        deliverable_count,
    }
}

fn is_accepted_planning_status(status: &str) -> bool {
    let normalized = status.trim().to_ascii_lowercase();
    normalized == "accepted"
        || normalized
            .strip_prefix("accepted")
            .is_some_and(|suffix| suffix.trim_start().starts_with('('))
}

fn topo_sort(adrs: &[PlanningAdr]) -> Vec<PlanningAdr> {
    let present: BTreeSet<String> = adrs.iter().map(|adr| adr.id.clone()).collect();
    let mut remaining: BTreeMap<String, PlanningAdr> = adrs
        .iter()
        .map(|adr| (adr.id.clone(), adr.clone()))
        .collect();
    let mut emitted = BTreeSet::new();
    let mut ordered = Vec::with_capacity(adrs.len());
    while !remaining.is_empty() {
        let next_id = remaining
            .iter()
            .find(|(_, adr)| {
                adr.depends_on
                    .iter()
                    .all(|dependency| !present.contains(dependency) || emitted.contains(dependency))
            })
            .map(|(id, _)| id.clone())
            .or_else(|| remaining.keys().next().cloned());
        let Some(next_id) = next_id else {
            break;
        };
        if let Some(adr) = remaining.remove(&next_id) {
            emitted.insert(next_id);
            ordered.push(adr);
        }
    }
    ordered
}

fn projection_to_json(projection: &MasterplanProjection) -> Value {
    let milestones: Vec<Value> = projection
        .milestones
        .iter()
        .map(|group| {
            let adrs: Vec<Value> = group
                .adrs
                .iter()
                .map(|adr| {
                    let deliverables: Vec<Value> = adr
                        .deliverables
                        .iter()
                        .map(|deliverable| {
                            let mut map = Map::new();
                            map.insert("id".into(), Value::String(deliverable.id.clone()));
                            map.insert(
                                "description".into(),
                                Value::String(deliverable.description.clone()),
                            );
                            map.insert(
                                "exit_criteria".into(),
                                Value::String(deliverable.exit_criteria.clone()),
                            );
                            map.insert(
                                "verified_by".into(),
                                Value::String(deliverable.verified_by.clone()),
                            );
                            map.insert(
                                "status".into(),
                                Value::String(DELIVERABLE_STATUS_V1.to_owned()),
                            );
                            Value::Object(map)
                        })
                        .collect();
                    let mut map = Map::new();
                    map.insert("id".into(), Value::String(adr.id.clone()));
                    map.insert("status".into(), Value::String(adr.status.clone()));
                    map.insert(
                        "depends_on".into(),
                        Value::Array(
                            adr.depends_on
                                .iter()
                                .map(|dependency| Value::String(dependency.clone()))
                                .collect(),
                        ),
                    );
                    map.insert("deliverables".into(), Value::Array(deliverables));
                    Value::Object(map)
                })
                .collect();
            let mut map = Map::new();
            map.insert("milestone".into(), Value::String(group.milestone.clone()));
            map.insert("adrs".into(), Value::Array(adrs));
            Value::Object(map)
        })
        .collect();

    let mut root = Map::new();
    root.insert(
        "_generated".into(),
        Value::String(
            "GENERATED by `oya gen masterplan` from docs/decisions/*.md planning_impact:true ADRs (ADR-0364). Do not hand-edit; run `oya gen masterplan --write` to regenerate."
                .to_owned(),
        ),
    );
    root.insert(
        "generator".into(),
        Value::String("oya gen masterplan".into()),
    );
    root.insert("source".into(), Value::String("docs/decisions/*.md".into()));
    root.insert(
        "deliverable_status_model".into(),
        Value::String(format!(
            "{DELIVERABLE_STATUS_V1} (v1; CI-derived status deferred per ADR-0364 §3)"
        )),
    );
    root.insert("adr_count".into(), Value::from(projection.adr_count));
    root.insert(
        "deliverable_count".into(),
        Value::from(projection.deliverable_count),
    );
    root.insert("milestones".into(), Value::Array(milestones));
    Value::Object(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"---
status: Accepted
planning_impact: true
milestone: M-TEST
depends_on: [ADR-0001]
deliverables:
  - id: ADR-0002-D1
    description: output
    exit_criteria: green
    verified_by: cargo test
---
# ADR
"#;

    fn adr(id: &str, milestone: &str, depends_on: &[&str]) -> PlanningAdr {
        PlanningAdr {
            id: id.to_owned(),
            status: "Accepted".to_owned(),
            milestone: milestone.to_owned(),
            depends_on: depends_on.iter().map(|value| (*value).to_owned()).collect(),
            has_deliverables_field: true,
            deliverables: vec![PlanningDeliverable {
                id: format!("{id}-D1"),
                description: "output".to_owned(),
                exit_criteria: "green".to_owned(),
                verified_by: "cargo test".to_owned(),
            }],
            path: format!("docs/decisions/{id}-test.md"),
        }
    }

    #[test]
    fn frontmatter_parser_preserves_planning_fields() {
        let frontmatter = read_frontmatter(SAMPLE).expect("frontmatter");
        let parsed = parse_planning_adr(
            frontmatter,
            "ADR-0002".to_owned(),
            "docs/decisions/ADR-0002-test.md".to_owned(),
        );
        assert_eq!(parsed.milestone, "M-TEST");
        assert_eq!(parsed.depends_on, ["ADR-0001"]);
        assert_eq!(parsed.deliverables[0].id, "ADR-0002-D1");
    }

    #[test]
    fn deliverable_parser_preserves_multiple_records() {
        let frontmatter = r#"id: ADR-0001
deliverables:
  - id: ADR-0001-D1
    description: first
    exit_criteria: first green
    verified_by: first test
  - id: ADR-0001-D2
    description: second
    exit_criteria: second green
    verified_by: second test"#;
        let (present, deliverables) = parse_deliverables(frontmatter);
        assert!(present);
        assert_eq!(deliverables.len(), 2);
        assert_eq!(deliverables[0].id, "ADR-0001-D1");
        assert_eq!(deliverables[0].description, "first");
        assert_eq!(deliverables[1].verified_by, "second test");
    }

    #[test]
    fn missing_and_empty_deliverables_remain_distinct() {
        let missing = "id: ADR-0001\nplanning_impact: true\nmilestone: M-TEST";
        let (missing_present, missing_deliverables) = parse_deliverables(missing);
        assert!(!missing_present);
        assert!(missing_deliverables.is_empty());

        let empty = "id: ADR-0001\ndeliverables: []";
        let (empty_present, empty_deliverables) = parse_deliverables(empty);
        assert!(empty_present);
        assert!(empty_deliverables.is_empty());
    }

    #[test]
    fn non_frontmatter_document_is_not_a_planning_adr() {
        assert!(read_frontmatter("# no frontmatter").is_none());
        assert!(
            parse_planning_impact_adr(Path::new("ADR-0001-test.md"), "# no frontmatter").is_none()
        );
    }

    #[test]
    fn projection_orders_dependencies_and_groups_milestones() {
        let projection = generate_masterplan_projection(&[
            adr("ADR-0002", "M-SECOND", &["ADR-0001"]),
            adr("ADR-0001", "M-FIRST", &[]),
        ]);
        assert_eq!(projection.adr_count, 2);
        assert_eq!(projection.deliverable_count, 2);
        assert_eq!(projection.milestones[0].milestone, "M-FIRST");
        assert_eq!(projection.milestones[1].milestone, "M-SECOND");
    }

    #[test]
    fn projection_is_cycle_safe_and_filters_nonaccepted_adrs() {
        let mut superseded = adr("ADR-0003", "M-THIRD", &[]);
        superseded.status = "Superseded".to_owned();
        let projection = generate_masterplan_projection(&[
            adr("ADR-0002", "M-TEST", &["ADR-0001"]),
            adr("ADR-0001", "M-TEST", &["ADR-0002"]),
            superseded,
        ]);
        assert_eq!(projection.adr_count, 2);
        assert_eq!(projection.milestones[0].adrs.len(), 2);
    }

    #[test]
    fn external_dependency_does_not_block_projection() {
        let projection =
            generate_masterplan_projection(&[adr("ADR-0005", "M-TEST", &["ADR-9999"])]);
        assert_eq!(projection.adr_count, 1);
        assert_eq!(projection.milestones[0].adrs[0].id, "ADR-0005");
    }

    #[test]
    fn projection_json_keeps_declared_status_and_stable_keys() {
        let projection = generate_masterplan_projection(&[adr("ADR-0001", "M-TEST", &[])]);
        let first = projection_to_json(&projection);
        let second = projection_to_json(&projection);
        assert_eq!(first, second);
        assert_eq!(
            first["milestones"][0]["adrs"][0]["deliverables"][0]["status"],
            "declared"
        );
    }
}
