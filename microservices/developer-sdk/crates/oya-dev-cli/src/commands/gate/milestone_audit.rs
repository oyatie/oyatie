//! `oya gate validate milestone-audit` — validates the machine-readable
//! milestone audit registry instead of relying on shell/Markdown review loops.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
struct MilestoneAuditArgs {
    repo_root: PathBuf,
    audit_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MilestoneAuditReport {
    milestones: usize,
    phases: usize,
    implementation_plans: usize,
    blocked_transitions: usize,
    safe_parallel_lanes: usize,
}

pub(crate) fn run(args: Vec<String>) -> ExitCode {
    match parse_args(args) {
        Ok(parsed) => match validate(&parsed) {
            Ok(report) => {
                println!(
                    "milestone-audit validation passed: {} milestones, {} phases, {} IPs, {} blocked transitions, {} safe-parallel lanes",
                    report.milestones,
                    report.phases,
                    report.implementation_plans,
                    report.blocked_transitions,
                    report.safe_parallel_lanes
                );
                ExitCode::SUCCESS
            }
            Err(errors) => {
                eprintln!("milestone-audit validation failed:");
                for error in errors {
                    eprintln!("  {error}");
                }
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<MilestoneAuditArgs, String> {
    let mut parsed = MilestoneAuditArgs {
        repo_root: PathBuf::from("."),
        audit_path: PathBuf::from("registry/milestone-audit/index.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                parsed.repo_root = PathBuf::from(value_for(&mut iter, "repo-root")?);
            }
            "--audit" => {
                parsed.audit_path = PathBuf::from(value_for(&mut iter, "audit")?);
            }
            other => {
                return Err(format!(
                    "milestone-audit: unknown flag {other:?}; allowed: --repo-root, --audit"
                ));
            }
        }
    }
    Ok(parsed)
}

fn value_for(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("milestone-audit: --{flag} requires a value"))
}

fn validate(args: &MilestoneAuditArgs) -> Result<MilestoneAuditReport, Vec<String>> {
    let repo_root = fs::canonicalize(&args.repo_root)
        .map_err(|error| vec![format!("repo root unreadable: {error}")])?;
    let audit_path = repo_root.join(&args.audit_path);
    let text = fs::read_to_string(&audit_path).map_err(|error| {
        vec![format!(
            "audit unreadable {}: {error}",
            audit_path.display()
        )]
    })?;
    let audit: Value = serde_json::from_str(&text).map_err(|error| {
        vec![format!(
            "audit JSON invalid {}: {error}",
            audit_path.display()
        )]
    })?;

    let mut errors = Vec::new();
    if string_at(&audit, &["schema"]) != Some("oyatie.registry.milestone_audit.index.v1") {
        errors.push("schema must be oyatie.registry.milestone_audit.index.v1".to_string());
    }
    if string_at(&audit, &["_meta", "doc_class"]) != Some("Machine-Readable-Registry") {
        errors.push("_meta.doc_class must be Machine-Readable-Registry".to_string());
    }
    let production_bar = string_at(&audit, &["quality_bar", "production_bar"]);
    if !matches!(production_bar, Some(value) if value.contains("hyperscaler") && value.contains("production"))
    {
        errors.push("quality_bar.production_bar must name hyperscaler production bar".to_string());
    }
    let lower = text.to_lowercase();
    if lower.contains("good enough")
        || contains_token(&lower, "mvp")
        || lower.contains("prototype scope")
    {
        errors.push("audit must not use MVP/prototype/good-enough framing".to_string());
    }

    for source in string_array_at(&audit, &["source_inputs"]) {
        let source_path = source.split('#').next().unwrap_or(&source);
        if source_path.starts_with('/') || source_path.starts_with("agent-skills:") {
            continue;
        }
        if !repo_root.join(source_path).exists() {
            errors.push(format!("source_inputs path does not exist: {source_path}"));
        }
    }

    let milestones = array_at(&audit, &["milestones"]);
    if milestones.is_empty() {
        errors.push("milestones must be non-empty".to_string());
    }
    let declared_milestones = u64_at(&audit, &["summary", "total_milestones"]);
    if declared_milestones != Some(milestones.len() as u64) {
        errors.push(format!(
            "summary.total_milestones must equal milestones length ({})",
            milestones.len()
        ));
    }

    let mut phase_count = 0usize;
    let mut ip_count = 0usize;
    let mut saw_no_go = false;
    for milestone in &milestones {
        let id = string_at(milestone, &["id"]).unwrap_or_default();
        if id.is_empty() {
            errors.push("milestones[] missing id".to_string());
        }
        if string_at(milestone, &["title"])
            .unwrap_or_default()
            .is_empty()
        {
            errors.push(format!("milestone {id} missing title"));
        }
        let verdict = string_at(milestone, &["verdict"]).unwrap_or_default();
        if verdict.is_empty() {
            errors.push(format!("milestone {id} missing verdict"));
        }
        saw_no_go |= verdict.contains("no_go") || verdict.contains("blocked");

        let phases = array_at(milestone, &["phases"]);
        if phases.is_empty() {
            errors.push(format!("milestone {id} must list phases"));
        }
        phase_count += phases.len();
        for phase in phases {
            let phase_id = string_at(phase, &["id"]).unwrap_or_default();
            if phase_id.is_empty() {
                errors.push(format!("milestone {id} has phase missing id"));
            }
            if string_at(phase, &["status"]).unwrap_or_default().is_empty() {
                errors.push(format!("phase {id}/{phase_id} missing status"));
            }
            if string_at(phase, &["honest_done_verdict"])
                .unwrap_or_default()
                .is_empty()
            {
                errors.push(format!("phase {id}/{phase_id} missing honest_done_verdict"));
            }
            ip_count += array_at(phase, &["implementation_plans"]).len();
        }
    }

    let blocked = array_at(&audit, &["blocked_transitions"]);
    if blocked.is_empty() || !saw_no_go {
        errors.push("audit must record at least one blocked/no_go transition".to_string());
    }
    let lanes = array_at(&audit, &["safe_parallel_lanes"]);
    if lanes.is_empty() {
        errors.push("safe_parallel_lanes must be non-empty".to_string());
    }
    let findings = array_at(&audit, &["findings"]);
    if findings.is_empty() {
        errors.push("findings must be non-empty".to_string());
    }
    let has_high_finding = findings.iter().any(|finding| {
        matches!(
            string_at(finding, &["severity"]),
            Some("BLOCKER" | "HIGH" | "CRITICAL")
        )
    });
    if !has_high_finding {
        errors.push("findings must include at least one HIGH/BLOCKER/CRITICAL item".to_string());
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(MilestoneAuditReport {
        milestones: milestones.len(),
        phases: phase_count,
        implementation_plans: ip_count,
        blocked_transitions: blocked.len(),
        safe_parallel_lanes: lanes.len(),
    })
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str()
}

fn u64_at(value: &Value, path: &[&str]) -> Option<u64> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_u64()
}

fn array_at<'a>(value: &'a Value, path: &[&str]) -> Vec<&'a Value> {
    let mut cursor = value;
    for segment in path {
        let Some(next) = cursor.get(*segment) else {
            return Vec::new();
        };
        cursor = next;
    }
    cursor
        .as_array()
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn string_array_at(value: &Value, path: &[&str]) -> Vec<String> {
    array_at(value, path)
        .into_iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn contains_token(text: &str, token: &str) -> bool {
    text.split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|part| part == token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_defaults_to_registry_index() {
        let parsed = parse_args(Vec::new()).expect("parse args");
        assert_eq!(parsed.repo_root, Path::new("."));
        assert_eq!(
            parsed.audit_path,
            Path::new("registry/milestone-audit/index.json")
        );
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let error = parse_args(vec!["--bogus".to_string()]).expect_err("must fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn mvp_token_check_does_not_flag_kcmvp() {
        assert!(!contains_token("IP-002-isms-p-kcmvp-hsm.md", "mvp"));
        assert!(contains_token("this mvp framing is forbidden", "mvp"));
    }
}
