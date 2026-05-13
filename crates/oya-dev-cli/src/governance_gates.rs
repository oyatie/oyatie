use std::fs;
use std::path::{Path, PathBuf};

use oya_foundry_authority_cohesion_kernel::{validate_authority_cohesion, AuthorityDocument};
use oya_foundry_catalog_kernel::CatalogIndex;
use oya_foundry_claim_ceiling_kernel::FoundationClaimCeiling;
use oya_foundry_constitution_cite_kernel::{
    validate_constitution_cite_coverage, ConstitutionCitationDocument,
};

use crate::{read_catalog_records, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaneClassValidateArgs {
    registry_dir: PathBuf,
    baseline_registry_dir: Option<PathBuf>,
    reviewed_changes: Vec<String>,
}

pub(crate) fn parse_plane_class_validate_args(
    args: Vec<String>,
) -> Result<PlaneClassValidateArgs, String> {
    let mut parsed = PlaneClassValidateArgs {
        registry_dir: PathBuf::from("registry/catalog"),
        baseline_registry_dir: None,
        reviewed_changes: Vec::new(),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--registry" => parsed.registry_dir = PathBuf::from(value),
            "--baseline" => parsed.baseline_registry_dir = Some(PathBuf::from(value)),
            "--reviewed-change" => parsed.reviewed_changes.push(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_plane_class_gate(
    args: PlaneClassValidateArgs,
) -> Result<(usize, usize), String> {
    let current_records = read_catalog_records(&args.registry_dir)?;
    let current_index = CatalogIndex::from_records(current_records)
        .map_err(|error| format!("catalog index invalid: {error:?}"))?;
    if let Some(baseline_registry_dir) = args.baseline_registry_dir {
        let baseline_records = read_catalog_records(&baseline_registry_dir)?;
        let baseline_index = CatalogIndex::from_records(baseline_records)
            .map_err(|error| format!("baseline catalog index invalid: {error:?}"))?;
        current_index
            .validate_plane_stability(
                &baseline_index,
                args.reviewed_changes.iter().map(String::as_str),
            )
            .map_err(|error| format!("plane transition review invalid: {error:?}"))?;
    }
    Ok((current_index.len(), args.reviewed_changes.len()))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimCeilingValidateArgs {
    registry_dir: PathBuf,
}

pub(crate) fn parse_claim_ceiling_validate_args(
    args: Vec<String>,
) -> Result<ClaimCeilingValidateArgs, String> {
    let mut parsed = ClaimCeilingValidateArgs {
        registry_dir: PathBuf::from("registry/catalog"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--registry" => parsed.registry_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_claim_ceiling_gate(args: ClaimCeilingValidateArgs) -> Result<usize, String> {
    let records = read_catalog_records(&args.registry_dir)?;
    let index = CatalogIndex::from_records(records)
        .map_err(|error| format!("catalog index invalid: {error:?}"))?;
    FoundationClaimCeiling::preview_foundation()
        .validate_catalog(&index)
        .map_err(|error| format!("catalog claim exceeds shipped foundation: {error:?}"))?;
    Ok(index.len())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityCohesionValidateArgs {
    docs_dir: PathBuf,
}

pub(crate) fn parse_authority_cohesion_validate_args(
    args: Vec<String>,
) -> Result<AuthorityCohesionValidateArgs, String> {
    let mut parsed = AuthorityCohesionValidateArgs {
        docs_dir: PathBuf::from("docs"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_authority_cohesion_gate(
    args: AuthorityCohesionValidateArgs,
) -> Result<usize, String> {
    let documents = read_authority_documents(&args.docs_dir)?;
    let report = validate_authority_cohesion(&documents)
        .map_err(|error| format!("authority chain declaration invalid: {error:?}"))?;
    Ok(report.document_count)
}

fn read_authority_documents(docs_dir: &Path) -> Result<Vec<AuthorityDocument>, String> {
    ["CONSTITUTION.md", "AGENTS.md", "README.md"]
        .into_iter()
        .map(|file_name| {
            let path = docs_dir.join(file_name);
            let contents = fs::read_to_string(&path)
                .map_err(|error| format!("authority document unreadable: {error}"))?;
            Ok(AuthorityDocument {
                path: path.display().to_string(),
                contents,
            })
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConstitutionCiteValidateArgs {
    docs_dir: PathBuf,
}

pub(crate) fn parse_constitution_cite_validate_args(
    args: Vec<String>,
) -> Result<ConstitutionCiteValidateArgs, String> {
    let mut parsed = ConstitutionCiteValidateArgs {
        docs_dir: PathBuf::from("docs"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_constitution_cite_gate(
    args: ConstitutionCiteValidateArgs,
) -> Result<usize, String> {
    let documents = read_tier_one_documents(&args.docs_dir)?;
    let report = validate_constitution_cite_coverage(&documents)
        .map_err(|error| format!("tier-one Constitution citation invalid: {error:?}"))?;
    Ok(report.document_count)
}

fn read_tier_one_documents(docs_dir: &Path) -> Result<Vec<ConstitutionCitationDocument>, String> {
    let readme_path = docs_dir.join("README.md");
    let readme = fs::read_to_string(&readme_path)
        .map_err(|error| format!("docs README unreadable: {error}"))?;
    parse_tier_one_doc_paths(&readme)?
        .into_iter()
        .map(|doc_path| {
            let path = docs_dir.join(&doc_path);
            let contents = fs::read_to_string(&path).map_err(|error| {
                format!("tier-one document unreadable {}: {error}", path.display())
            })?;
            Ok(ConstitutionCitationDocument {
                path: path.display().to_string(),
                contents,
            })
        })
        .collect()
}

fn parse_tier_one_doc_paths(readme: &str) -> Result<Vec<String>, String> {
    let section = readme
        .split("## Tier-1 documents")
        .nth(1)
        .ok_or_else(|| "docs README missing Tier-1 documents section".to_string())?
        .split("\n## ")
        .next()
        .ok_or_else(|| "docs README missing Tier-1 section body".to_string())?;
    let paths = section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("- [") {
                return None;
            }
            let (_, after_open) = trimmed.split_once("](")?;
            let (path, _) = after_open.split_once(')')?;
            Some(path.to_string())
        })
        .collect::<Vec<_>>();
    if paths.is_empty() {
        Err("docs README Tier-1 document list is empty".to_string())
    } else {
        Ok(paths)
    }
}
