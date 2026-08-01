use std::fs;
use std::path::{Path, PathBuf};

use check_honest_claims::{
    ChangeSetPlanViolation, HonestClaimsDocument, HonestClaimsViolation,
    ImplementationPlanDocument, validate_changeset_plan_graph, validate_honest_claims,
};

use crate::{path_has_component, slash_path, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HonestClaimsValidateArgs {
    corpus_roots: Vec<PathBuf>,
    plans_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HonestClaimsGateReport {
    pub documents_checked: usize,
    pub lines_checked: usize,
    pub plans_checked: usize,
    pub dependency_edges: usize,
    pub serialization_edges: usize,
    pub global_artifact_writes: usize,
    pub legacy_missing_split_rule: usize,
}

pub(crate) fn parse_honest_claims_validate_args(
    args: Vec<String>,
) -> Result<HonestClaimsValidateArgs, String> {
    let mut parsed = HonestClaimsValidateArgs {
        corpus_roots: vec![
            PathBuf::from("docs/PRD.md"),
            PathBuf::from("docs/decisions"),
            PathBuf::from("docs/prds"),
            PathBuf::from("docs/products"),
            PathBuf::from("docs/raw/agentic-delivery-fabric-executable-prd.md"),
            PathBuf::from("docs/standards"),
            PathBuf::from("specs"),
        ],
        plans_dir: PathBuf::from(".omc/plans/milestones"),
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
            "--plans-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.plans_dir = PathBuf::from(value);
            }
            _ => return Err(usage()),
        }
    }
    if parsed.corpus_roots.is_empty() {
        return Err("honest-claims requires at least one --corpus-root".to_string());
    }
    Ok(parsed)
}

pub(crate) fn validate_honest_claims_gate(
    args: HonestClaimsValidateArgs,
) -> Result<HonestClaimsGateReport, String> {
    let documents = read_honest_claim_documents(&args.corpus_roots)?;
    let claims_report = validate_honest_claims(documents).map_err(render_claim_violations)?;

    let plan_documents = read_implementation_plan_documents(&args.plans_dir)?;
    let plan_report =
        validate_changeset_plan_graph(plan_documents).map_err(render_plan_violations)?;

    Ok(HonestClaimsGateReport {
        documents_checked: claims_report.documents_checked,
        lines_checked: claims_report.lines_checked,
        plans_checked: plan_report.plans_checked,
        dependency_edges: plan_report.dependency_edges,
        serialization_edges: plan_report.serialization_edges,
        global_artifact_writes: plan_report.global_artifact_writes,
        legacy_missing_split_rule: plan_report.legacy_missing_split_rule,
    })
}

fn read_honest_claim_documents(roots: &[PathBuf]) -> Result<Vec<HonestClaimsDocument>, String> {
    let mut documents = Vec::new();
    for root in roots {
        let root = root.as_path();
        if root.is_file() {
            if is_corpus_file(root) {
                documents.push(read_claim_document(root, root)?);
            }
            continue;
        }
        collect_claim_documents(root, root, &mut documents)?;
    }
    if documents.is_empty() {
        Err("honest-claims corpus roots contained no supported files".to_string())
    } else {
        Ok(documents)
    }
}

fn collect_claim_documents(
    root: &Path,
    current: &Path,
    documents: &mut Vec<HonestClaimsDocument>,
) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|error| format!("honest-claims corpus root unreadable: {error}"))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("honest-claims corpus entry unreadable: {error}"))?;
        let path = entry.path();
        if path_has_component(&path, "target") || path_has_component(&path, ".git") {
            continue;
        }
        if path.is_dir() {
            collect_claim_documents(root, &path, documents)?;
        } else if path.is_file() && is_corpus_file(&path) {
            documents.push(read_claim_document(root, &path)?);
        }
    }
    Ok(())
}

fn read_claim_document(root: &Path, path: &Path) -> Result<HonestClaimsDocument, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "honest-claims corpus file unreadable {}: {error}",
            path.display()
        )
    })?;
    Ok(HonestClaimsDocument {
        path: display_relative(root, path),
        contents,
    })
}

fn read_implementation_plan_documents(
    plans_dir: &Path,
) -> Result<Vec<ImplementationPlanDocument>, String> {
    let mut documents = Vec::new();
    collect_implementation_plan_documents(plans_dir, plans_dir, &mut documents)?;
    if documents.is_empty() {
        Err(format!(
            "honest-claims plans dir contains no IP markdown files: {}",
            plans_dir.display()
        ))
    } else {
        Ok(documents)
    }
}

fn collect_implementation_plan_documents(
    root: &Path,
    current: &Path,
    documents: &mut Vec<ImplementationPlanDocument>,
) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|error| format!("honest-claims plans dir unreadable: {error}"))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("honest-claims plans entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_implementation_plan_documents(root, &path, documents)?;
            continue;
        }
        if !path.is_file()
            || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("IP-") {
            continue;
        }
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "honest-claims implementation plan unreadable {}: {error}",
                path.display()
            )
        })?;
        documents.push(ImplementationPlanDocument {
            path: display_relative(root, &path),
            contents,
        });
    }
    Ok(())
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

fn render_claim_violations(violations: Vec<HonestClaimsViolation>) -> String {
    let mut rendered = format!("{} honest-claims violation(s)", violations.len());
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

fn render_plan_violations(violations: Vec<ChangeSetPlanViolation>) -> String {
    let mut rendered = format!("{} ChangeSet plan violation(s)", violations.len());
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
