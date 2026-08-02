//! `oya gate validate protection-context-match` runner.
//!
//! Reads `.github/branch-protection.yaml` (the `required_status_checks`
//! list under the protected branch) and walks `.github/workflows/*.yml`
//! collecting every job `name:` field, then invokes the
//! [`check_protection_context_match`] kernel to assert every
//! required context is the `name:` of some workflow job. It also compares
//! the canonical YAML against the apply-source branch-protection JSON so
//! local preflight catches stale required-check config before GitHub mutation.
//! When given a live GitHub required-status-checks JSON snapshot, it also
//! asserts live branch protection requires exactly the canonical contexts.
//!
//! Lane id: `oya-governance-protection-context-match`. The lane
//! is the machine-checkable encoding of the
//! [[feedback_no_silent_regression]] directive applied to the
//! protection/workflow seam: prevents the silent-bypass class of bug
//! where `branch-protection.yaml` lists a context that no workflow
//! posts, leaving GitHub waiting forever for a non-existent check_run
//! while reporting the gate as "expected".
//!
//! Naming justification: module file is snake_case, no redundant
//! suffix; functions follow the existing
//! `parse_<lane>_validate_args` / `validate_<lane>_gate` naming
//! used by every other gate in this crate.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use check_protection_context_match::{
    ProtectionContextMatchReport, WorkflowJobNames, validate_protection_context_match,
};

const USAGE: &str = "oya gate validate protection-context-match \
                     [--branch-protection <.github/branch-protection.yaml>] \
                     [--workflows-dir <.github/workflows>] \
                     [--branch <dev>] \
                     [--applied-branch-protection <infra/branch-protection/dev.json>] \
                     [--skip-applied-branch-protection] \
                     [--live-required-contexts <required_status_checks.json>]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProtectionContextMatchValidateArgs {
    pub branch_protection_path: PathBuf,
    pub workflows_dir: PathBuf,
    pub branch_name: String,
    pub applied_branch_protection_path: Option<PathBuf>,
    pub live_required_contexts_path: Option<PathBuf>,
    /// ADR-0361: the Jenkins-reported status-context manifest. Producer source
    /// when GitHub Actions workflows are retired (the dir may be absent).
    pub reported_contexts_path: Option<PathBuf>,
}

impl Default for ProtectionContextMatchValidateArgs {
    fn default() -> Self {
        Self {
            branch_protection_path: PathBuf::from(".github/branch-protection.yaml"),
            workflows_dir: PathBuf::from(".github/workflows"),
            // Default to `dev` — the canonical default branch per the
            // FINAL-FINAL pipeline (M01-P17-2026-05-16 + the
            // branch-protection.yaml rewrite landing here in PR #4).
            // Override with `--branch <name>` when the lane needs to
            // validate a non-default branch's protection.
            branch_name: "dev".to_string(),
            applied_branch_protection_path: Some(PathBuf::from("infra/branch-protection/dev.json")),
            live_required_contexts_path: None,
            reported_contexts_path: Some(PathBuf::from(
                "infra/ci/jenkins/reported-status-contexts.json",
            )),
        }
    }
}

pub(crate) fn parse_protection_context_match_validate_args(
    args: Vec<String>,
) -> Result<ProtectionContextMatchValidateArgs, String> {
    let mut parsed = ProtectionContextMatchValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--branch-protection" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.branch_protection_path = PathBuf::from(value);
            }
            "--workflows-dir" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.workflows_dir = PathBuf::from(value);
            }
            "--branch" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.branch_name = value;
            }
            "--applied-branch-protection" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.applied_branch_protection_path = Some(PathBuf::from(value));
            }
            "--skip-applied-branch-protection" => {
                parsed.applied_branch_protection_path = None;
            }
            "--live-required-contexts" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.live_required_contexts_path = Some(PathBuf::from(value));
            }
            "--reported-contexts" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.reported_contexts_path = Some(PathBuf::from(value));
            }
            "--skip-reported-contexts" => {
                parsed.reported_contexts_path = None;
            }
            _ => return Err(USAGE.to_owned()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_protection_context_match_gate(
    args: ProtectionContextMatchValidateArgs,
) -> Result<ProtectionContextMatchReport, String> {
    let protection_text = fs::read_to_string(&args.branch_protection_path).map_err(|error| {
        format!(
            "could not read branch-protection at {}: {error}",
            args.branch_protection_path.display()
        )
    })?;
    let contexts = parse_required_status_checks(&protection_text, &args.branch_name)?;
    if let Some(applied_branch_protection_path) = &args.applied_branch_protection_path {
        let applied_text = fs::read_to_string(applied_branch_protection_path).map_err(|error| {
            format!(
                "could not read applied branch-protection config at {}: {error}",
                applied_branch_protection_path.display()
            )
        })?;
        let applied_contexts = parse_required_status_checks_json(&applied_text, "applied")?;
        validate_required_status_checks_match(
            &contexts,
            &applied_contexts,
            &args.branch_name,
            "applied branch-protection config",
        )?;
    }
    if let Some(live_required_contexts_path) = &args.live_required_contexts_path {
        let live_text = fs::read_to_string(live_required_contexts_path).map_err(|error| {
            format!(
                "could not read live required-status contexts at {}: {error}",
                live_required_contexts_path.display()
            )
        })?;
        let live_contexts = parse_live_required_status_checks(&live_text)?;
        validate_required_status_checks_match(
            &contexts,
            &live_contexts,
            &args.branch_name,
            "live branch protection",
        )?;
    }

    let mut workflows: Vec<WorkflowJobNames> = Vec::new();

    // ADR-0361: the Jenkins-reported status-context manifest is a first-class
    // producer source, so retiring `.github/workflows` does not strand the gate.
    if let Some(reported_contexts_path) = &args.reported_contexts_path {
        match fs::read_to_string(reported_contexts_path) {
            Ok(text) => {
                let job_names = parse_reported_status_contexts(&text)?;
                workflows.push(WorkflowJobNames {
                    workflow_path: reported_contexts_path.display().to_string(),
                    job_names,
                });
            }
            // Absent manifest (e.g. an isolated test cwd) is tolerated; other
            // producer sources must then cover the required contexts.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "could not read reported-status-contexts manifest at {}: {error}",
                    reported_contexts_path.display()
                ));
            }
        }
    }

    // GitHub Actions workflows are an ADDITIONAL producer source only while they
    // exist; a retired (absent) workflows dir is tolerated, not an error.
    match fs::read_dir(&args.workflows_dir) {
        Ok(entries) => {
            for entry in entries {
                let entry =
                    entry.map_err(|error| format!("could not read workflow entry: {error}"))?;
                let path = entry.path();
                let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
                    continue;
                };
                if extension != "yml" && extension != "yaml" {
                    continue;
                }
                let workflow_text = fs::read_to_string(&path).map_err(|error| {
                    format!("could not read workflow {}: {error}", path.display())
                })?;
                let job_names = parse_workflow_job_names(&workflow_text);
                workflows.push(WorkflowJobNames {
                    workflow_path: path.display().to_string(),
                    job_names,
                });
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // Retired per ADR-0361 — the Jenkins manifest is the producer source.
        }
        Err(error) => {
            return Err(format!(
                "could not list workflows dir at {}: {error}",
                args.workflows_dir.display()
            ));
        }
    }

    validate_protection_context_match(&contexts, &workflows).map_err(|error| error.to_string())
}

/// Parse the `reported_status_contexts` array from the Jenkins-reported
/// status-context manifest (ADR-0361).
fn parse_reported_status_contexts(text: &str) -> Result<Vec<String>, String> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| format!("reported-status-contexts manifest is invalid JSON: {error}"))?;
    let array = value
        .get("reported_status_contexts")
        .and_then(|v| v.as_array())
        .ok_or("reported-status-contexts manifest missing `reported_status_contexts` array")?;
    Ok(array
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect())
}

/// Extract the `required_status_checks` list for the specified branch
/// from a parsed branch-protection.yaml. The format is the small
/// custom YAML this repo ships (see file header); we don't depend on
/// a full YAML parser to keep this runner kernel-tier and dep-free.
fn parse_required_status_checks(yaml_text: &str, branch: &str) -> Result<Vec<String>, String> {
    let mut contexts: Vec<String> = Vec::new();
    let mut in_target_branch = false;
    let mut in_required_status_checks = false;
    let branch_header = format!("{branch}:");
    for raw_line in yaml_text.lines() {
        let line = raw_line.trim_end();
        let stripped = line.trim_start();
        if stripped.is_empty() {
            continue;
        }
        // Track section: `branches:` → `<branch>:`. We're conservative:
        // we exit the target-branch section the moment a sibling
        // section opens at the same indent.
        if line.starts_with("  ") && line.trim_end() == format!("  {branch_header}") {
            in_target_branch = true;
            in_required_status_checks = false;
            continue;
        }
        if in_target_branch && line.starts_with("  ") && !line.starts_with("    ") {
            // A new branch sibling at indent=2 closes our section.
            in_target_branch = false;
            in_required_status_checks = false;
        }
        if !in_target_branch {
            continue;
        }
        if stripped.starts_with("required_status_checks:") {
            in_required_status_checks = true;
            continue;
        }
        if in_required_status_checks {
            if let Some(value) = stripped.strip_prefix("- ") {
                // Skip commented bullets. The branch-protection.yaml
                // documents time-bounded relaxations as `# - foo`;
                // those are intentionally NOT enforced contexts.
                let value = value.trim();
                if !value.is_empty() && !value.starts_with('#') {
                    contexts.push(value.to_string());
                }
                continue;
            }
            // A non-bullet line ends the required_status_checks list.
            if !stripped.starts_with('#') {
                in_required_status_checks = false;
            }
        }
    }
    if contexts.is_empty() {
        return Err(format!(
            "branch-protection.yaml has zero required_status_checks for branch `{branch}` \
             — gate cannot validate an empty list"
        ));
    }
    Ok(contexts)
}

/// Extract live `required_status_checks.contexts` as emitted by
/// `gh api repos/<repo>/branches/<branch>/protection/required_status_checks`.
/// The workflow writes either the full endpoint object or the `.contexts`
/// array. The latter keeps the local test fixture small while preserving
/// exact live-API semantics.
fn parse_live_required_status_checks(json_text: &str) -> Result<Vec<String>, String> {
    parse_required_status_checks_json(json_text, "live")
}

fn parse_required_status_checks_json(
    json_text: &str,
    source_label: &str,
) -> Result<Vec<String>, String> {
    let value: serde_json::Value = serde_json::from_str(json_text).map_err(|error| {
        format!("could not parse {source_label} required-status contexts JSON: {error}")
    })?;
    let contexts_value = if value.is_array() {
        &value
    } else if let Some(contexts) = value.get("contexts") {
        contexts
    } else if let Some(contexts) = value.pointer("/required_status_checks/contexts") {
        contexts
    } else {
        return Err(format!(
            "{source_label} required-status contexts JSON must be an array, an object with \
             `contexts`, or an object with `required_status_checks.contexts`"
        ));
    };
    let contexts = contexts_value.as_array().ok_or_else(|| {
        format!("{source_label} required-status contexts JSON has a non-array contexts field")
    })?;
    let mut parsed = Vec::new();
    for context in contexts {
        let context = context.as_str().ok_or_else(|| {
            format!("{source_label} required-status contexts JSON contains a non-string context")
        })?;
        let context = context.trim();
        if context.is_empty() {
            return Err(format!(
                "{source_label} required-status contexts JSON contains an empty context"
            ));
        }
        parsed.push(context.to_string());
    }
    if parsed.is_empty() {
        return Err(format!(
            "{source_label} branch protection has zero required_status_checks contexts — gate \
             cannot validate an empty {source_label} list"
        ));
    }
    Ok(parsed)
}

fn validate_required_status_checks_match(
    canonical_contexts: &[String],
    compared_contexts: &[String],
    branch: &str,
    compared_label: &str,
) -> Result<(), String> {
    let canonical: BTreeSet<String> = canonical_contexts.iter().cloned().collect();
    let compared: BTreeSet<String> = compared_contexts.iter().cloned().collect();
    let missing_from_compared = canonical
        .difference(&compared)
        .cloned()
        .collect::<Vec<String>>();
    let extra_in_compared = compared
        .difference(&canonical)
        .cloned()
        .collect::<Vec<String>>();
    if missing_from_compared.is_empty() && extra_in_compared.is_empty() {
        return Ok(());
    }

    let mut message = vec![format!(
        "{compared_label} required_status_checks for branch `{branch}` diverge \
         from .github/branch-protection.yaml"
    )];
    if !missing_from_compared.is_empty() {
        message.push(format!(
            "missing from {compared_label}: {}",
            missing_from_compared.join(", ")
        ));
    }
    if !extra_in_compared.is_empty() {
        message.push(format!(
            "extra in {compared_label}: {}",
            extra_in_compared.join(", ")
        ));
    }
    Err(message.join("\n"))
}

/// Extract every job's `name:` field from a workflow file. Falls back
/// to the job-key when `name:` is omitted (matches GitHub's own
/// behavior: a missing display name defaults to the job-key).
fn parse_workflow_job_names(workflow_text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut in_jobs = false;
    let mut current_job_key: Option<String> = None;
    let mut current_job_name: Option<String> = None;
    for raw_line in workflow_text.lines() {
        let line = raw_line.trim_end();
        let stripped = line.trim_start();
        if line == "jobs:" {
            in_jobs = true;
            continue;
        }
        if !in_jobs {
            continue;
        }
        // Job-key line: `  <key>:` at exactly 2 spaces of indent and
        // ends with `:`. Save the prior job's resolved name first.
        if line.starts_with("  ")
            && !line.starts_with("    ")
            && line.ends_with(':')
            && !stripped.contains(' ')
        {
            if let Some(key) = current_job_key.take() {
                names.push(current_job_name.take().unwrap_or(key));
            }
            // strip trailing colon and leading spaces
            let key = stripped.trim_end_matches(':').to_string();
            current_job_key = Some(key);
            current_job_name = None;
            continue;
        }
        // `name:` lines inside a job. Only honor the FIRST `name:` we
        // see per job — subsequent name fields inside steps don't
        // count.
        if current_job_key.is_some()
            && current_job_name.is_none()
            && let Some(value) = stripped.strip_prefix("name:")
        {
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            current_job_name = Some(value);
        }
    }
    if let Some(key) = current_job_key {
        names.push(current_job_name.unwrap_or(key));
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uses_canonical_defaults() {
        let args =
            parse_protection_context_match_validate_args(Vec::new()).expect("no flags is valid");
        assert_eq!(
            args.branch_protection_path,
            PathBuf::from(".github/branch-protection.yaml")
        );
        assert_eq!(args.workflows_dir, PathBuf::from(".github/workflows"));
        assert_eq!(args.branch_name, "dev");
        assert_eq!(
            args.applied_branch_protection_path,
            Some(PathBuf::from("infra/branch-protection/dev.json"))
        );
        assert_eq!(args.live_required_contexts_path, None);
    }

    #[test]
    fn parse_can_skip_applied_branch_protection_check() {
        let args = parse_protection_context_match_validate_args(vec![
            "--skip-applied-branch-protection".into(),
        ])
        .expect("skip applied config flag is valid");
        assert_eq!(args.applied_branch_protection_path, None);
    }

    #[test]
    fn parse_accepts_applied_branch_protection_path() {
        let args = parse_protection_context_match_validate_args(vec![
            "--applied-branch-protection".into(),
            "infra/dev.json".into(),
        ])
        .expect("applied config flag is valid");
        assert_eq!(
            args.applied_branch_protection_path,
            Some(PathBuf::from("infra/dev.json"))
        );
    }

    #[test]
    fn parse_accepts_live_required_contexts_path() {
        let args = parse_protection_context_match_validate_args(vec![
            "--live-required-contexts".into(),
            "live.json".into(),
        ])
        .expect("live contexts flag is valid");
        assert_eq!(
            args.live_required_contexts_path,
            Some(PathBuf::from("live.json"))
        );
    }

    #[test]
    fn parse_required_status_checks_extracts_main_branch_contexts() {
        let yaml = "branches:\n  main:\n    require_pull_request: true\n    \
                    required_status_checks:\n      - cargo-fmt\n      - cargo-clippy\n      \
                    # - oya-pr-review  # commented bullet is skipped\n      \
                    - oya-governance-supply-chain\n    require_signed_commits: true\n";
        let contexts = parse_required_status_checks(yaml, "main").expect("parses");
        assert_eq!(
            contexts,
            vec![
                "cargo-fmt".to_string(),
                "cargo-clippy".to_string(),
                "oya-governance-supply-chain".to_string()
            ]
        );
    }

    #[test]
    fn parse_required_status_checks_rejects_empty_branch_section() {
        let yaml = "branches:\n  main:\n    require_pull_request: true\n";
        let error = parse_required_status_checks(yaml, "main").unwrap_err();
        assert!(error.contains("zero required_status_checks"));
    }

    #[test]
    fn parse_live_required_status_checks_accepts_api_object() {
        let json = r#"{"strict":false,"contexts":["cargo-fmt","oya-pr-review"]}"#;
        let contexts = parse_live_required_status_checks(json).expect("parses live object");
        assert_eq!(
            contexts,
            vec!["cargo-fmt".to_string(), "oya-pr-review".to_string()]
        );
    }

    #[test]
    fn parse_live_required_status_checks_accepts_full_protection_object() {
        let json = r#"{"required_status_checks":{"contexts":["cargo-fmt"]}}"#;
        let contexts = parse_live_required_status_checks(json).expect("parses full object");
        assert_eq!(contexts, vec!["cargo-fmt".to_string()]);
    }

    #[test]
    fn live_required_status_checks_match_reports_drift() {
        let canonical = vec!["cargo-fmt".to_string(), "oya-pr-review".to_string()];
        let live = vec!["cargo-fmt".to_string(), "stale-required-check".to_string()];
        let error = validate_required_status_checks_match(
            &canonical,
            &live,
            "dev",
            "live branch protection",
        )
        .unwrap_err();
        assert!(error.contains("branch `dev` diverge"));
        assert!(error.contains("missing from live branch protection: oya-pr-review"));
        assert!(error.contains("extra in live branch protection: stale-required-check"));
    }

    #[test]
    fn applied_required_status_checks_match_reports_drift() {
        let canonical = vec![
            "oya-governance-protection-context-match".to_string(),
            "oya-pr-review".to_string(),
        ];
        let applied = vec![
            "oya-governance-protection-context-match".to_string(),
            "stale-applied-check".to_string(),
        ];
        let error = validate_required_status_checks_match(
            &canonical,
            &applied,
            "dev",
            "applied branch-protection config",
        )
        .unwrap_err();
        assert!(error.contains("branch `dev` diverge"));
        assert!(
            error.contains("missing from applied branch-protection config: oya-pr-review"),
            "{error}"
        );
        assert!(
            error.contains("extra in applied branch-protection config: stale-applied-check"),
            "{error}"
        );
    }

    #[test]
    fn parse_workflow_job_names_extracts_each_job_name() {
        let workflow = "name: pr-tests\non:\n  pull_request:\njobs:\n  cargo-fmt:\n    \
                        name: cargo-fmt\n    runs-on: ubuntu-latest\n  cargo-clippy:\n    \
                        name: cargo-clippy\n    runs-on: ubuntu-latest\n";
        let names = parse_workflow_job_names(workflow);
        assert_eq!(
            names,
            vec!["cargo-fmt".to_string(), "cargo-clippy".to_string()]
        );
    }

    #[test]
    fn parse_workflow_job_names_falls_back_to_job_key_when_name_missing() {
        let workflow = "name: pr-tests\non:\n  pull_request:\njobs:\n  some-job:\n    \
                        runs-on: ubuntu-latest\n";
        let names = parse_workflow_job_names(workflow);
        assert_eq!(names, vec!["some-job".to_string()]);
    }

    #[test]
    fn parse_workflow_job_names_handles_quoted_names() {
        let workflow = "name: pr-tests\non:\n  pull_request:\njobs:\n  some-job:\n    \
                        name: \"some display name\"\n    runs-on: ubuntu-latest\n";
        let names = parse_workflow_job_names(workflow);
        assert_eq!(names, vec!["some display name".to_string()]);
    }

    #[test]
    fn parse_workflow_ignores_step_names() {
        // `step.name` fields nest deeper than `job.name`. The kernel
        // must only pick up the per-job display name.
        let workflow = "name: pr-tests\non:\n  pull_request:\njobs:\n  cargo-fmt:\n    \
                        name: cargo-fmt\n    runs-on: ubuntu-latest\n    steps:\n      \
                        - name: do thing\n        run: echo hi\n";
        let names = parse_workflow_job_names(workflow);
        assert_eq!(names, vec!["cargo-fmt".to_string()]);
    }
}
