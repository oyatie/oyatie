//! `oya gen masterplan` (ADR-0364 D3) — generate the masterplan projection
//! from accepted `planning_impact: true` ADRs.
//!
//! Reads every `docs/decisions/ADR-*.md`, parses the YAML front-matter between
//! the leading `---` fences, selects ADRs with `planning_impact: true`,
//! extracts `{id, status, milestone, depends_on, deliverables[]}`, topo-sorts
//! by `depends_on` (stable, cycle-safe), groups by `milestone`, and emits a
//! generated projection to `docs/machine-readable/masterplan.generated.json`.
//!
//! - default: prints a human summary.
//! - `--write`: writes the JSON projection.
//! - `--check`: regenerate in-memory and diff vs the committed projection;
//!   non-zero exit on drift (ADR-0364 D4's mechanism).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::adr_planning_frontmatter::{PlanningAdr, read_planning_impact_adrs};

const DEFAULT_DECISIONS_DIR: &str = "docs/decisions";
const DEFAULT_OUTPUT: &str = "docs/machine-readable/masterplan.generated.json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenMasterplanArgs {
    pub(crate) decisions_dir: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) write: bool,
    pub(crate) check: bool,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    execute(&parsed)
}

fn parse_args(args: Vec<String>, usage: &str) -> Result<GenMasterplanArgs, String> {
    let mut parsed = GenMasterplanArgs {
        decisions_dir: PathBuf::from(DEFAULT_DECISIONS_DIR),
        output: PathBuf::from(DEFAULT_OUTPUT),
        write: false,
        check: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--write" => parsed.write = true,
            "--check" => parsed.check = true,
            "--decisions-dir" => {
                parsed.decisions_dir = PathBuf::from(iter.next().ok_or_else(|| usage.to_owned())?);
            }
            "--output" => {
                parsed.output = PathBuf::from(iter.next().ok_or_else(|| usage.to_owned())?);
            }
            _ => return Err(usage.to_owned()),
        }
    }
    if parsed.write && parsed.check {
        return Err("gen masterplan: --write and --check are mutually exclusive".to_string());
    }
    Ok(parsed)
}

fn execute(args: &GenMasterplanArgs) -> ExitCode {
    let (projection, json) = match render_projection(&args.decisions_dir) {
        Ok(rendered) => rendered,
        Err(message) => {
            eprintln!("gen masterplan: {message}");
            return ExitCode::FAILURE;
        }
    };

    if args.check {
        let committed = match std::fs::read_to_string(&args.output) {
            Ok(text) => text,
            Err(error) => {
                eprintln!(
                    "gen masterplan --check: committed projection unreadable {}: {error}",
                    args.output.display()
                );
                eprintln!("  run `oya gen masterplan --write` to generate it");
                return ExitCode::FAILURE;
            }
        };
        if committed == json {
            println!(
                "gen masterplan --check passed: {} matches the regenerated projection ({} ADRs, {} deliverables, {} milestones)",
                args.output.display(),
                projection.adr_count,
                projection.deliverable_count,
                projection.milestones.len()
            );
            return ExitCode::SUCCESS;
        }
        eprintln!(
            "gen masterplan --check failed: {} drifted from the regenerated projection",
            args.output.display()
        );
        eprintln!("  run `oya gen masterplan --write` to regenerate it");
        for line in first_diff_lines(&committed, &json) {
            eprintln!("  {line}");
        }
        return ExitCode::FAILURE;
    }

    if args.write {
        if let Some(parent) = args.output.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            eprintln!(
                "gen masterplan --write: output directory unwritable {}: {error}",
                parent.display()
            );
            return ExitCode::FAILURE;
        }
        if let Err(error) = std::fs::write(&args.output, &json) {
            eprintln!(
                "gen masterplan --write: output unwritable {}: {error}",
                args.output.display()
            );
            return ExitCode::FAILURE;
        }
        println!(
            "gen masterplan wrote {}: {} ADRs, {} deliverables, {} milestones",
            args.output.display(),
            projection.adr_count,
            projection.deliverable_count,
            projection.milestones.len()
        );
        return ExitCode::SUCCESS;
    }

    println!(
        "gen masterplan summary: {} accepted planning_impact ADRs, {} deliverables, {} milestones",
        projection.adr_count,
        projection.deliverable_count,
        projection.milestones.len()
    );
    for milestone in &projection.milestones {
        println!("  {} ({} ADRs)", milestone.milestone, milestone.adrs.len());
        for adr in &milestone.adrs {
            println!(
                "    {} [{}] {} deliverable(s)",
                adr.id,
                adr.status,
                adr.deliverables.len()
            );
        }
    }
    ExitCode::SUCCESS
}

pub(crate) fn render_projection(
    decisions_dir: &Path,
) -> Result<(MasterplanProjection, String), String> {
    let adrs = read_planning_impact_adrs(decisions_dir)?;
    let projection = generate_masterplan_projection(&adrs);
    let json = serde_json::to_string_pretty(&projection_to_json(&projection))
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|error| format!("projection serialization failed: {error}"))?;
    Ok((projection, json))
}

fn first_diff_lines(committed: &str, regenerated: &str) -> Vec<String> {
    let committed_lines: Vec<&str> = committed.lines().collect();
    let regenerated_lines: Vec<&str> = regenerated.lines().collect();
    let max = committed_lines.len().max(regenerated_lines.len());
    let mut out = Vec::new();
    for index in 0..max {
        let committed_line = committed_lines.get(index).copied().unwrap_or("<absent>");
        let regenerated_line = regenerated_lines.get(index).copied().unwrap_or("<absent>");
        if committed_line != regenerated_line {
            out.push(format!("first drift at line {}:", index + 1));
            out.push(format!("    committed:    {committed_line}"));
            out.push(format!("    regenerated:  {regenerated_line}"));
            break;
        }
    }
    out
}

// --- projection model (serialized to JSON via serde_json::Value) ---

#[derive(Clone, Debug)]
pub(crate) struct MasterplanProjection {
    pub(crate) milestones: Vec<MilestoneGroup>,
    pub(crate) adr_count: usize,
    pub(crate) deliverable_count: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct MilestoneGroup {
    pub(crate) milestone: String,
    pub(crate) adrs: Vec<ProjectedAdr>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectedAdr {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) depends_on: Vec<String>,
    pub(crate) deliverables: Vec<ProjectedDeliverable>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectedDeliverable {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) exit_criteria: String,
    pub(crate) verified_by: String,
}

/// Build the masterplan projection: keep only Accepted ADRs, topo-sort them by
/// `depends_on`, then group by `milestone` (milestone order = first appearance
/// in the topo-sorted list). Superseded/Proposed/Rejected ADRs are governance
/// history or draft intent, not live masterplan work.
pub(crate) fn generate_masterplan_projection(adrs: &[PlanningAdr]) -> MasterplanProjection {
    let accepted: Vec<PlanningAdr> = adrs
        .iter()
        .filter(|adr| is_accepted_planning_status(&adr.status))
        .cloned()
        .collect();
    let ordered = topo_sort(&accepted);
    let mut deliverable_count = 0usize;
    let mut milestone_order: Vec<String> = Vec::new();
    let mut grouped: BTreeMap<String, Vec<ProjectedAdr>> = BTreeMap::new();

    for adr in &ordered {
        let milestone = if adr.milestone.trim().is_empty() {
            "UNASSIGNED".to_string()
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
    normalized == "amended"
        || normalized == "accepted"
        || normalized
            .strip_prefix("accepted")
            .is_some_and(|suffix| suffix.trim_start().starts_with('('))
}

/// Stable, cycle-safe topological sort by `depends_on`. ADRs whose declared
/// dependencies are all already emitted (or not part of this planning set) are
/// emitted first; ties broken by ascending ADR id for determinism. If a cycle
/// remains, the lowest-id remaining ADR is forced out (cycle-safe) so the
/// generator never hangs or drops a node.
fn topo_sort(adrs: &[PlanningAdr]) -> Vec<PlanningAdr> {
    let present: BTreeSet<String> = adrs.iter().map(|adr| adr.id.clone()).collect();
    let mut remaining: BTreeMap<String, PlanningAdr> = adrs
        .iter()
        .map(|adr| (adr.id.clone(), adr.clone()))
        .collect();
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    let mut ordered: Vec<PlanningAdr> = Vec::with_capacity(adrs.len());

    while !remaining.is_empty() {
        // BTreeMap iterates in ascending key order, so this is deterministic.
        let next_id = remaining
            .iter()
            .find(|(_, adr)| {
                adr.depends_on
                    .iter()
                    .all(|dep| !present.contains(dep) || emitted.contains(dep))
            })
            .map(|(id, _)| id.clone())
            // Cycle-safe fallback: force the lowest-id remaining node.
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

// Deliverable status v1 = "declared" for every deliverable. ADR-0364 §3
// specifies status is DERIVED from `verified_by` (gate green => done), but the
// real CI-derivation is deferred: the generator does not yet execute the
// `verified_by` gates. Until that lands, every deliverable is emitted with
// status "declared" so the projection stays honest (no unearned "done").
const DELIVERABLE_STATUS_V1: &str = "declared";

fn projection_to_json(projection: &MasterplanProjection) -> serde_json::Value {
    use serde_json::{Map, Value};
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
                                Value::String(DELIVERABLE_STATUS_V1.to_string()),
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
                                .map(|dep| Value::String(dep.clone()))
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
            "GENERATED by `oya gen masterplan` from docs/decisions/*.md planning_impact:true ADRs (ADR-0364). Do not hand-edit; run `oya gen masterplan --write` to regenerate.".to_string(),
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
    root.insert(
        "adr_count".into(),
        Value::Number(projection.adr_count.into()),
    );
    root.insert(
        "deliverable_count".into(),
        Value::Number(projection.deliverable_count.into()),
    );
    root.insert("milestones".into(), Value::Array(milestones));
    Value::Object(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adr_planning_frontmatter::{PlanningAdr, PlanningDeliverable};

    fn adr(id: &str, milestone: &str, depends_on: &[&str]) -> PlanningAdr {
        PlanningAdr {
            id: id.into(),
            status: "Accepted".into(),
            milestone: milestone.into(),
            depends_on: depends_on.iter().map(|d| (*d).into()).collect(),
            has_deliverables_field: true,
            deliverables: vec![PlanningDeliverable {
                id: format!("{id}-D1"),
                description: "d".into(),
                exit_criteria: "e".into(),
                verified_by: "v".into(),
            }],
            path: format!("docs/decisions/{id}-x.md"),
        }
    }

    #[test]
    fn topo_sort_orders_dependencies_first() {
        let adrs = vec![
            adr("ADR-0003", "M2", &["ADR-0002"]),
            adr("ADR-0002", "M1", &["ADR-0001"]),
            adr("ADR-0001", "M1", &[]),
        ];
        let ordered = topo_sort(&adrs);
        let ids: Vec<&str> = ordered.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, vec!["ADR-0001", "ADR-0002", "ADR-0003"]);
    }

    #[test]
    fn topo_sort_is_cycle_safe() {
        let adrs = vec![
            adr("ADR-0001", "M1", &["ADR-0002"]),
            adr("ADR-0002", "M1", &["ADR-0001"]),
        ];
        let ordered = topo_sort(&adrs);
        assert_eq!(ordered.len(), 2, "cycle must not drop nodes");
    }

    #[test]
    fn external_dependency_does_not_block() {
        // depends_on an ADR not in the planning set => not a blocker.
        let adrs = vec![adr("ADR-0005", "M1", &["ADR-9999"])];
        let ordered = topo_sort(&adrs);
        assert_eq!(ordered.len(), 1);
    }

    #[test]
    fn projection_groups_by_milestone_in_topo_order() {
        let adrs = vec![
            adr("ADR-0002", "M-PLANNING", &["ADR-0001"]),
            adr("ADR-0001", "M-FOUNDATION", &[]),
        ];
        let projection = generate_masterplan_projection(&adrs);
        assert_eq!(projection.adr_count, 2);
        assert_eq!(projection.deliverable_count, 2);
        assert_eq!(projection.milestones[0].milestone, "M-FOUNDATION");
        assert_eq!(projection.milestones[1].milestone, "M-PLANNING");
    }

    #[test]
    fn json_deliverables_carry_declared_status() {
        let adrs = vec![adr("ADR-0001", "M1", &[])];
        let projection = generate_masterplan_projection(&adrs);
        let json = projection_to_json(&projection);
        let status = json["milestones"][0]["adrs"][0]["deliverables"][0]["status"]
            .as_str()
            .expect("status");
        assert_eq!(status, "declared");
    }
}
