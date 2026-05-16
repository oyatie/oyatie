// Purpose: status-honesty audit for `specs/masterplan.json`.
// Verifies that no phase is marked complete with incomplete child IPs, and that
// every IP marked complete has a referencing evidence JSON record. Ported from
// `scripts/audit-master-plan-completion.py` per
// `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-7.
// Naming-justification: `master_plan_completion_audit_gate` is a check-family
// (`*_gate`) module under `oya-dev-cli`, satisfying predictable-naming-kernel
// `is_check_family(name)`. Surface command
// `gate audit master-plan-completion` is canonical kebab-case verb-noun pair
// (ADR-0105 v4 BNF).

use std::path::PathBuf;

use crate::usage;

const COMPLETE_STATUSES: &[&str] = &[
    "complete",
    "accepted",
    "foundation-cleared",
    "foundation cleared",
];
const INCOMPLETE_MARKERS: &[&str] = &[
    "stub",
    "planned",
    "pending",
    "blocked",
    "in-flight",
    "probe-green",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MasterPlanCompletionAuditArgs {
    pub master_plan_path: PathBuf,
    pub evidence_dirs: Vec<PathBuf>,
}

pub(crate) fn parse_master_plan_completion_audit_args(
    args: Vec<String>,
) -> Result<MasterPlanCompletionAuditArgs, String> {
    let mut parsed = MasterPlanCompletionAuditArgs {
        master_plan_path: PathBuf::from("specs/masterplan.json"),
        evidence_dirs: vec![
            PathBuf::from("evidence/foundation"),
            PathBuf::from("evidence/gitops-vcs"),
            PathBuf::from("evidence/agentic-pipeline"),
        ],
    };
    let mut iter = args.into_iter();
    let mut evidence_dirs_override: Option<Vec<PathBuf>> = None;
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--check" => {}
            "--master-plan" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.master_plan_path = PathBuf::from(value);
            }
            "--evidence-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                evidence_dirs_override
                    .get_or_insert_with(Vec::new)
                    .push(PathBuf::from(value));
            }
            _ => return Err(usage()),
        }
    }
    if let Some(dirs) = evidence_dirs_override {
        parsed.evidence_dirs = dirs;
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MasterPlanCompletionAuditReport {
    pub phases_checked: usize,
    pub implementation_plans_checked: usize,
}

pub(crate) fn audit_master_plan_completion(
    args: MasterPlanCompletionAuditArgs,
) -> Result<MasterPlanCompletionAuditReport, String> {
    let master_plan_text = std::fs::read_to_string(&args.master_plan_path).map_err(|error| {
        format!(
            "masterplan unreadable {}: {error}",
            args.master_plan_path.display()
        )
    })?;
    let evidence_text = collect_evidence_text(&args.evidence_dirs)?;
    audit_master_plan_completion_strings(&master_plan_text, &evidence_text)
}

fn collect_evidence_text(directories: &[PathBuf]) -> Result<String, String> {
    let mut chunks: Vec<String> = Vec::new();
    for directory in directories {
        if !directory.exists() {
            continue;
        }
        let entries = std::fs::read_dir(directory).map_err(|error| {
            format!(
                "evidence directory unreadable {}: {error}",
                directory.display()
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "evidence directory entry unreadable {}: {error}",
                    directory.display()
                )
            })?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            match std::fs::read_to_string(&path) {
                Ok(text) => chunks.push(text),
                Err(error) => {
                    return Err(format!(
                        "evidence file unreadable {}: {error}",
                        path.display()
                    ));
                }
            }
        }
    }
    Ok(chunks.join("\n"))
}

pub(crate) fn audit_master_plan_completion_strings(
    master_plan_text: &str,
    evidence_text: &str,
) -> Result<MasterPlanCompletionAuditReport, String> {
    let data: serde_json::Value = serde_json::from_str(master_plan_text)
        .map_err(|error| format!("masterplan JSON invalid: {error}"))?;
    let index = data
        .get("live_implementation_index")
        .ok_or_else(|| "masterplan missing live_implementation_index".to_string())?;
    let milestones = index.get("milestones").and_then(|value| value.as_array());

    let mut errors: Vec<String> = Vec::new();
    let mut phases_checked = 0usize;
    let mut ips_checked = 0usize;
    let milestones_iter = milestones.into_iter().flatten();
    for milestone in milestones_iter {
        let phases = milestone.get("phases").and_then(|value| value.as_array());
        let Some(phases) = phases else { continue };
        for phase in phases {
            phases_checked += 1;
            let phase_status = phase.get("status").and_then(|value| value.as_str());
            let phase_id = phase
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("<missing-id>");
            let ips_opt = phase
                .get("implementation_plans")
                .and_then(|value| value.as_array());
            let empty: Vec<serde_json::Value> = Vec::new();
            let ips: &Vec<serde_json::Value> = ips_opt.unwrap_or(&empty);
            let child_statuses: Vec<Option<&str>> = ips
                .iter()
                .map(|ip| ip.get("status").and_then(|value| value.as_str()))
                .collect();
            if is_complete(phase_status)
                && child_statuses.iter().any(|status| is_incomplete(*status))
            {
                errors.push(format!(
                    "phase {phase_id} is complete but has incomplete child IP"
                ));
            }
            for ip in ips {
                ips_checked += 1;
                let ip_status = ip.get("status").and_then(|value| value.as_str());
                let ip_id = ip
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or("<missing-id>");
                if is_complete(ip_status) && !evidence_text.contains(ip_id) {
                    errors.push(format!(
                        "complete IP {ip_id} has no evidence JSON reference"
                    ));
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors
            .iter()
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n"));
    }
    Ok(MasterPlanCompletionAuditReport {
        phases_checked,
        implementation_plans_checked: ips_checked,
    })
}

fn normalized(status: Option<&str>) -> String {
    status.unwrap_or("").trim().to_ascii_lowercase()
}

fn is_complete(status: Option<&str>) -> bool {
    let value = normalized(status);
    COMPLETE_STATUSES.contains(&value.as_str()) || value.ends_with(" complete")
}

fn is_incomplete(status: Option<&str>) -> bool {
    let value = normalized(status);
    !is_complete(status)
        || INCOMPLETE_MARKERS
            .iter()
            .any(|marker| value.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_PLAN: &str = r#"{
        "live_implementation_index": {
            "milestones": [
                {
                    "phases": [
                        {
                            "id": "P-X",
                            "status": "complete",
                            "implementation_plans": [
                                {"id": "IP-001", "status": "complete"}
                            ]
                        }
                    ]
                }
            ]
        }
    }"#;

    #[test]
    fn audit_passes_when_evidence_references_complete_ip() {
        let evidence = "{\"ip\": \"IP-001\"}";
        let report = audit_master_plan_completion_strings(MINIMAL_PLAN, evidence)
            .expect("complete IP with evidence must pass");
        assert_eq!(report.phases_checked, 1);
        assert_eq!(report.implementation_plans_checked, 1);
    }

    #[test]
    fn audit_rejects_complete_phase_with_incomplete_child() {
        let plan = MINIMAL_PLAN.replace(
            "{\"id\": \"IP-001\", \"status\": \"complete\"}",
            "{\"id\": \"IP-001\", \"status\": \"pending\"}",
        );
        let error = audit_master_plan_completion_strings(&plan, "")
            .expect_err("incomplete child must fail");
        assert!(error.contains("P-X"));
    }

    #[test]
    fn audit_rejects_complete_ip_without_evidence() {
        let error = audit_master_plan_completion_strings(MINIMAL_PLAN, "")
            .expect_err("missing evidence must fail");
        assert!(error.contains("IP-001"));
    }
}
