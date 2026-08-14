use std::fs;
use std::path::{Path, PathBuf};

use check_authority_cohesion::{AuthorityDocument, validate_authority_cohesion};
use intelligence_catalog_domain::CatalogIndex;
use oya_check_claim_ceiling::FoundationClaimCeiling;

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
    ["AGENTS.md", "README.md", "MASTERPLAN.md"]
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
