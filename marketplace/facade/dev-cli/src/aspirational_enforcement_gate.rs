use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use oya_check_aspirational_enforcement::{
    AspirationalDocument, AspirationalViolation, KnownEnforcementSurfaces,
    validate_aspirational_enforcement,
};

use crate::{path_has_component, slash_path, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AspirationalEnforcementValidateArgs {
    corpus_roots: Vec<PathBuf>,
    catalog_dir: PathBuf,
    workflows_dir: PathBuf,
    quality_lanes: PathBuf,
    branch_protection: PathBuf,
    branch: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AspirationalEnforcementGateReport {
    pub documents_checked: usize,
    pub lines_checked: usize,
    pub binding_mentions: usize,
    pub check_sites: usize,
    pub check_capabilities: usize,
    pub workflow_contexts: usize,
    pub quality_lane_contexts: usize,
    pub branch_required_contexts: usize,
}

pub(crate) fn parse_aspirational_enforcement_validate_args(
    args: Vec<String>,
) -> Result<AspirationalEnforcementValidateArgs, String> {
    let mut parsed = AspirationalEnforcementValidateArgs {
        corpus_roots: vec![
            PathBuf::from("docs"),
            PathBuf::from("specs"),
            PathBuf::from("registry"),
        ],
        catalog_dir: PathBuf::from("registry/catalog"),
        workflows_dir: PathBuf::from(".github/workflows"),
        quality_lanes: PathBuf::from("registry/quality/lanes.yaml"),
        branch_protection: PathBuf::from(".github/branch-protection.yaml"),
        branch: "dev".to_string(),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--corpus-root" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.corpus_roots.push(PathBuf::from(value));
            }
            "--clear-default-corpus" => parsed.corpus_roots.clear(),
            "--catalog-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.catalog_dir = PathBuf::from(value);
            }
            "--workflows-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.workflows_dir = PathBuf::from(value);
            }
            "--quality-lanes" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.quality_lanes = PathBuf::from(value);
            }
            "--branch-protection" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.branch_protection = PathBuf::from(value);
            }
            "--branch" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.branch = value;
            }
            _ => return Err(usage()),
        }
    }
    if parsed.corpus_roots.is_empty() {
        return Err("aspirational-enforcement requires at least one --corpus-root".to_string());
    }
    Ok(parsed)
}

pub(crate) fn validate_aspirational_enforcement_gate(
    args: AspirationalEnforcementValidateArgs,
) -> Result<AspirationalEnforcementGateReport, String> {
    let documents = read_documents(&args.corpus_roots)?;
    let workflow_contexts = read_workflow_contexts(&args.workflows_dir)?;
    let quality_lane_contexts = read_active_quality_lane_contexts(&args.quality_lanes)?;
    let mut branch_required_contexts =
        read_branch_required_contexts(&args.branch_protection, &args.branch)?;
    if branch_required_contexts.contains("oya-pr-review") {
        branch_required_contexts.extend(quality_lane_contexts.iter().cloned());
    }
    let surfaces = KnownEnforcementSurfaces {
        check_capabilities: read_check_capabilities(&args.catalog_dir)?,
        workflow_contexts,
        quality_lane_contexts,
        branch_required_contexts,
        declared_lane_ids: read_declared_lane_ids(&args.quality_lanes)?,
    };
    let check_capabilities = surfaces.check_capabilities.len();
    let workflow_contexts = surfaces.workflow_contexts.len();
    let quality_lane_contexts = surfaces.quality_lane_contexts.len();
    let branch_required_contexts = surfaces.branch_required_contexts.len();
    let report =
        validate_aspirational_enforcement(documents, &surfaces).map_err(render_violations)?;

    // STANDING RULE, corpus side: the surface scan can be healthy while the
    // CORPUS scan is the empty one (a rename that changes how documents spell
    // the gates). Zero observed sites means the tokenizer stopped matching, so
    // fail rather than report clean over a corpus we measured nothing in.
    if report.check_sites == 0 {
        return Err(format!(
            "aspirational-enforcement observed ZERO check-capability sites across {} document(s) / {} line(s) while {check_capabilities} check capabilities are registered — the corpus scan is empty, which is a broken scan, not a clean corpus",
            report.documents_checked, report.lines_checked
        ));
    }

    Ok(AspirationalEnforcementGateReport {
        documents_checked: report.documents_checked,
        lines_checked: report.lines_checked,
        binding_mentions: report.binding_mentions,
        check_sites: report.check_sites,
        check_capabilities,
        workflow_contexts,
        quality_lane_contexts,
        branch_required_contexts,
    })
}

fn read_documents(roots: &[PathBuf]) -> Result<Vec<AspirationalDocument>, String> {
    let mut documents = Vec::new();
    for root in roots {
        if root.is_file() {
            if is_corpus_file(root) {
                documents.push(read_document(root, root)?);
            }
            continue;
        }
        collect_documents(root, root, &mut documents)?;
    }
    if documents.is_empty() {
        Err("aspirational-enforcement corpus roots contained no supported files".to_string())
    } else {
        Ok(documents)
    }
}

fn collect_documents(
    root: &Path,
    current: &Path,
    documents: &mut Vec<AspirationalDocument>,
) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|error| format!("aspirational-enforcement corpus root unreadable: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!("aspirational-enforcement corpus entry unreadable: {error}")
        })?;
        let path = entry.path();
        if path_has_component(&path, "target") || path_has_component(&path, ".git") {
            continue;
        }
        if path.is_dir() {
            collect_documents(root, &path, documents)?;
        } else if path.is_file() && is_corpus_file(&path) {
            documents.push(read_document(root, &path)?);
        }
    }
    Ok(())
}

fn read_document(root: &Path, path: &Path) -> Result<AspirationalDocument, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "aspirational-enforcement corpus file unreadable {}: {error}",
            path.display()
        )
    })?;
    Ok(AspirationalDocument {
        path: display_relative(root, path),
        contents,
    })
}

/// Check-gate identities come from the `capability:` facet of the catalog
/// records, NOT from scanning a directory for an `oya-check-` name prefix.
/// The facet is invariant under relocation: moving `libs/oya-check-adr-citation`
/// to `governance/check/adr-citation` renames the record STEM but leaves
/// `capability: check-adr-citation` untouched, so the scan cannot be emptied by
/// a rename. Mirrors the tier gate's `capability: fitness-*` keying.
fn read_check_capabilities(catalog_dir: &Path) -> Result<BTreeSet<String>, String> {
    let entries = fs::read_dir(catalog_dir).map_err(|error| {
        format!(
            "aspirational-enforcement catalog dir unreadable {}: {error}",
            catalog_dir.display()
        )
    })?;
    let mut capabilities = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!("aspirational-enforcement catalog entry unreadable: {error}")
        })?;
        let path = entry.path();
        if !path.is_file()
            || !matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("yaml" | "yml")
            )
        {
            continue;
        }
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "aspirational-enforcement catalog record unreadable {}: {error}",
                path.display()
            )
        })?;
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("capability:") {
                let capability = normalize_yaml_scalar(value);
                if capability.starts_with("check-") {
                    capabilities.insert(capability);
                }
                break;
            }
        }
    }
    // STANDING RULE: a rule whose measured site count is 0 is a broken scan,
    // not a clean repo. Without this the gate runs, observes nothing, and
    // reports clean — indistinguishable from passing. The identical guard on
    // read_branch_required_contexts below is the precedent.
    if capabilities.is_empty() {
        return Err(format!(
            "aspirational-enforcement observed ZERO check capabilities under {} — the check-gate surface scan is empty, which is a broken scan, not a clean repo (expected `capability: check-*` facets in the catalog records)",
            catalog_dir.display()
        ));
    }
    Ok(capabilities)
}

fn read_workflow_contexts(workflows_dir: &Path) -> Result<BTreeSet<String>, String> {
    let mut contexts = BTreeSet::new();
    // ADR-0361: Jenkins is the CI. The Jenkins-reported status contexts are the
    // authoritative job/context source; an absent .github/workflows dir is tolerated.
    seed_jenkins_reported_contexts(&mut contexts);
    let entries = match fs::read_dir(workflows_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(contexts),
        Err(error) => {
            return Err(format!(
                "aspirational-enforcement workflows dir unreadable {}: {error}",
                workflows_dir.display()
            ));
        }
    };
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!("aspirational-enforcement workflow entry unreadable: {error}")
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("yml" | "yaml")
        ) {
            continue;
        }
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "aspirational-enforcement workflow unreadable {}: {error}",
                path.display()
            )
        })?;
        collect_workflow_contexts(&contents, &mut contexts);
    }
    Ok(contexts)
}

/// ADR-0361: seed the valid job/context set from the Jenkins-reported status
/// contexts manifest, so binding claims that reference CI contexts resolve once
/// the GitHub Actions workflows are retired. Best-effort; absent manifest = no-op.
fn seed_jenkins_reported_contexts(contexts: &mut BTreeSet<String>) {
    let Ok(text) = fs::read_to_string("infra/ci/jenkins/reported-status-contexts.json") else {
        return;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    if let Some(arr) = value
        .get("reported_status_contexts")
        .and_then(|v| v.as_array())
    {
        for c in arr {
            if let Some(s) = c.as_str() {
                contexts.insert(s.to_string());
            }
        }
    }
}

fn collect_workflow_contexts(contents: &str, contexts: &mut BTreeSet<String>) {
    let mut jobs_indent = None::<usize>;
    let mut current_job_indent = None::<usize>;
    for line in contents.lines() {
        let uncommented = strip_inline_comment(line).trim_end();
        let stripped = uncommented.trim_start();
        if stripped.is_empty() {
            continue;
        }
        let indent = uncommented.len() - stripped.len();

        if indent == 0 {
            current_job_indent = None;
            jobs_indent = None;
            if let Some(value) = stripped.strip_prefix("name:") {
                insert_context(value, contexts);
            } else if stripped == "jobs:" {
                jobs_indent = Some(indent);
            }
            continue;
        }

        let Some(parent_indent) = jobs_indent else {
            continue;
        };
        if indent <= parent_indent {
            current_job_indent = None;
            jobs_indent = None;
            continue;
        }
        if indent == parent_indent + 2
            && let Some(key) = stripped.strip_suffix(':')
        {
            current_job_indent = Some(indent);
            insert_context(key, contexts);
            continue;
        }
        if let Some(job_indent) = current_job_indent
            && indent == job_indent + 2
            && let Some(value) = stripped.strip_prefix("name:")
        {
            insert_context(value, contexts);
        }
    }
}

// All lane ids declared in the registry, regardless of status. Used to
// distinguish real (declared) governance lanes from planned/aspirational lane
// references (undeclared) per ADR-0362 (a).
fn read_declared_lane_ids(quality_lanes: &Path) -> Result<BTreeSet<String>, String> {
    let contents = fs::read_to_string(quality_lanes).map_err(|error| {
        format!(
            "aspirational-enforcement quality-lanes registry unreadable {}: {error}",
            quality_lanes.display()
        )
    })?;
    let mut ids = BTreeSet::new();
    for line in contents.lines() {
        if let Some(id) = line.trim().strip_prefix("- id:") {
            ids.insert(normalize_yaml_scalar(id));
        }
    }
    Ok(ids)
}

fn read_active_quality_lane_contexts(quality_lanes: &Path) -> Result<BTreeSet<String>, String> {
    let contents = fs::read_to_string(quality_lanes).map_err(|error| {
        format!(
            "aspirational-enforcement quality-lanes registry unreadable {}: {error}",
            quality_lanes.display()
        )
    })?;
    let mut contexts = BTreeSet::new();
    let mut current_id = None::<String>;
    let mut current_status = None::<String>;

    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(id) = trimmed.strip_prefix("- id:") {
            insert_active_quality_lane(&current_id, &current_status, &mut contexts);
            current_id = Some(normalize_yaml_scalar(id));
            current_status = None;
        } else if let Some(status) = trimmed.strip_prefix("status:") {
            current_status = Some(normalize_yaml_scalar(status));
        }
    }
    insert_active_quality_lane(&current_id, &current_status, &mut contexts);
    Ok(contexts)
}

fn insert_active_quality_lane(
    id: &Option<String>,
    status: &Option<String>,
    contexts: &mut BTreeSet<String>,
) {
    let Some(id) = id.as_ref() else {
        return;
    };
    let Some(status) = status.as_ref() else {
        return;
    };
    if status == "active" && id.starts_with("oya-governance-") {
        contexts.insert(id.to_string());
    }
}

fn read_branch_required_contexts(
    branch_protection: &Path,
    branch: &str,
) -> Result<BTreeSet<String>, String> {
    let contents = fs::read_to_string(branch_protection).map_err(|error| {
        format!(
            "aspirational-enforcement branch-protection unreadable {}: {error}",
            branch_protection.display()
        )
    })?;
    let mut contexts = BTreeSet::new();
    let mut in_target_branch = false;
    let mut in_required_status_checks = false;
    let branch_header = format!("{branch}:");
    for line in contents.lines() {
        let line = line.trim_end();
        let stripped = line.trim_start();
        let line_without_comment = strip_inline_comment(line).trim_end();
        if stripped.is_empty() {
            continue;
        }
        if line_without_comment.starts_with("  ")
            && line_without_comment == format!("  {branch_header}")
        {
            in_target_branch = true;
            in_required_status_checks = false;
            continue;
        }
        if in_target_branch && line.starts_with("  ") && !line.starts_with("    ") {
            in_target_branch = false;
            in_required_status_checks = false;
        }
        if !in_target_branch {
            continue;
        }
        let stripped_without_comment = strip_inline_comment(stripped).trim();
        if stripped_without_comment.starts_with("required_status_checks:") {
            in_required_status_checks = true;
            continue;
        }
        if in_required_status_checks {
            if let Some(value) = stripped.strip_prefix("- ") {
                let value = value.trim();
                if !value.is_empty() && !value.starts_with('#') {
                    insert_context(value, &mut contexts);
                }
                continue;
            }
            if !stripped.starts_with('#') {
                in_required_status_checks = false;
            }
        }
    }
    if contexts.is_empty() {
        return Err(format!(
            "aspirational-enforcement branch-protection has zero required_status_checks for branch `{branch}`"
        ));
    }
    Ok(contexts)
}

fn insert_context(value: &str, contexts: &mut BTreeSet<String>) {
    let value = normalize_yaml_scalar(value);
    if value.starts_with("oya-") || value.starts_with("cargo-") {
        contexts.insert(value);
    }
}

fn normalize_yaml_scalar(value: &str) -> String {
    strip_inline_comment(value)
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn strip_inline_comment(value: &str) -> &str {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    for (index, character) in value.char_indices() {
        match character {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '#' if !in_single_quote && !in_double_quote => return &value[..index],
            _ => {}
        }
    }
    value
}

fn is_corpus_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("md" | "json" | "yaml" | "yml")
    )
}

fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
        .map(slash_path)
        .unwrap_or_else(|| slash_path(path))
}

fn render_violations(violations: Vec<AspirationalViolation>) -> String {
    let mut rendered = format!("{} aspirational-enforcement violation(s)", violations.len());
    for violation in violations.iter().take(20) {
        rendered.push_str(&format!("\n- {violation}"));
    }
    if violations.len() > 20 {
        rendered.push_str(&format!(
            "\n- ... {} additional violation(s) omitted",
            violations.len() - 20
        ));
    }
    rendered
}
