//! Foundry authority-cohesion kernel.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityDocument {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityCohesionReport {
    pub document_count: usize,
    pub declaration: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootHubPointerTarget {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootHubPointerReport {
    pub pointer_count: usize,
    pub target_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootHubPointerError {
    EmptyTargetPath,
    InvalidRootHubJson,
    MissingEntryPointsObject,
    MissingPointerPath,
    MissingPointerFragment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityCohesionError {
    EmptyDocumentPath,
    MissingDeclaration,
    DeclarationDrift,
    RetiredPrescribedAuthority,
}

pub fn validate_authority_cohesion(
    documents: &[AuthorityDocument],
) -> Result<AuthorityCohesionReport, AuthorityCohesionError> {
    let mut baseline = None;
    for document in documents {
        if document.path.trim().is_empty() {
            return Err(AuthorityCohesionError::EmptyDocumentPath);
        }
        if contains_retired_prescribed_authority(&document.contents) {
            return Err(AuthorityCohesionError::RetiredPrescribedAuthority);
        }
        let declaration = extract_authority_chain_declaration(&document.contents)?;
        match &baseline {
            Some(expected) if expected != &declaration => {
                return Err(AuthorityCohesionError::DeclarationDrift);
            }
            Some(_) => {}
            None => baseline = Some(declaration),
        }
    }
    let declaration = baseline.ok_or(AuthorityCohesionError::MissingDeclaration)?;
    Ok(AuthorityCohesionReport {
        document_count: documents.len(),
        declaration,
    })
}
pub fn validate_root_hub_pointer_reachability(
    root_hub_contents: &str,
    targets: &[RootHubPointerTarget],
) -> Result<RootHubPointerReport, RootHubPointerError> {
    let root_hub: serde_json::Value = serde_json::from_str(root_hub_contents)
        .map_err(|_| RootHubPointerError::InvalidRootHubJson)?;
    let entry_points = root_hub
        .get("entry_points")
        .and_then(serde_json::Value::as_object)
        .ok_or(RootHubPointerError::MissingEntryPointsObject)?;

    let mut target_by_path = BTreeMap::new();
    for target in targets {
        let normalized_path = normalize_repo_path(&target.path);
        if normalized_path.is_empty() {
            return Err(RootHubPointerError::EmptyTargetPath);
        }
        target_by_path.insert(normalized_path, target.contents.as_str());
    }

    let mut pointer_count = 0;
    validate_pointer_values(entry_points, &target_by_path, &mut pointer_count)?;

    Ok(RootHubPointerReport {
        pointer_count,
        target_count: target_by_path.len(),
    })
}

fn validate_pointer_values(
    value: &serde_json::Map<String, serde_json::Value>,
    target_by_path: &BTreeMap<String, &str>,
    pointer_count: &mut usize,
) -> Result<(), RootHubPointerError> {
    for (key, child) in value {
        if let Some(pointer) = child.as_str().filter(|_| is_reachability_pointer_key(key))
            && validate_pointer_value(pointer, target_by_path)?
        {
            *pointer_count += 1;
        }

        match child {
            serde_json::Value::Object(object) => {
                validate_pointer_values(object, target_by_path, pointer_count)?;
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    if let serde_json::Value::Object(object) = item {
                        validate_pointer_values(object, target_by_path, pointer_count)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_reachability_pointer_key(key: &str) -> bool {
    matches!(key, "current_path" | "canonical_authority_path") || key.ends_with("_pointer")
}

fn validate_pointer_value(
    pointer: &str,
    target_by_path: &BTreeMap<String, &str>,
) -> Result<bool, RootHubPointerError> {
    let Some((raw_path, fragment)) = split_repo_pointer(pointer) else {
        return Ok(false);
    };

    let path = normalize_repo_path(raw_path);
    let Some(contents) = target_by_path.get(&path) else {
        return Err(RootHubPointerError::MissingPointerPath);
    };

    if let Some(fragment) = fragment {
        validate_fragment(contents, fragment)?;
    }

    Ok(true)
}

fn split_repo_pointer(pointer: &str) -> Option<(&str, Option<&str>)> {
    let trimmed = pointer.trim();
    if trimmed.is_empty()
        || trimmed == "same"
        || trimmed.starts_with("same ")
        || trimmed.starts_with("same (")
        || !trimmed.contains('/')
    {
        return None;
    }

    let (path, fragment) = trimmed
        .split_once('#')
        .map_or((trimmed, None), |(path, fragment)| (path, Some(fragment)));

    if path.trim().is_empty() {
        None
    } else {
        Some((path, fragment))
    }
}

fn validate_fragment(contents: &str, fragment: &str) -> Result<(), RootHubPointerError> {
    if fragment.trim().is_empty() {
        return Err(RootHubPointerError::MissingPointerFragment);
    }

    let Ok(document) = serde_json::from_str::<serde_json::Value>(contents) else {
        return Err(RootHubPointerError::MissingPointerFragment);
    };

    if let Some(json_pointer) = fragment.strip_prefix('/') {
        let pointer = format!("/{json_pointer}");
        if document.pointer(&pointer).is_some() {
            return Ok(());
        }
        return Err(RootHubPointerError::MissingPointerFragment);
    }

    let mut current = &document;
    for segment in fragment.split('.') {
        match current {
            serde_json::Value::Object(object) => {
                current = object
                    .get(segment)
                    .ok_or(RootHubPointerError::MissingPointerFragment)?;
            }
            serde_json::Value::Array(items) => {
                let index = segment
                    .parse::<usize>()
                    .map_err(|_| RootHubPointerError::MissingPointerFragment)?;
                current = items
                    .get(index)
                    .ok_or(RootHubPointerError::MissingPointerFragment)?;
            }
            _ => return Err(RootHubPointerError::MissingPointerFragment),
        }
    }

    Ok(())
}

fn normalize_repo_path(path: &str) -> String {
    path.trim().trim_start_matches('/').to_string()
}

fn extract_authority_chain_declaration(contents: &str) -> Result<String, AuthorityCohesionError> {
    let mut in_declaration = false;
    let mut declaration = Vec::new();
    for line in contents.lines() {
        if !in_declaration {
            if line.trim_end() == "authority_chain_declaration: |" {
                in_declaration = true;
            }
            continue;
        }
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("  ") {
            declaration.push(value);
        } else if line.trim().is_empty() {
            declaration.push("");
        } else {
            break;
        }
    }
    if declaration.is_empty() {
        Err(AuthorityCohesionError::MissingDeclaration)
    } else {
        Ok(declaration.join("\n"))
    }
}

fn contains_retired_prescribed_authority(contents: &str) -> bool {
    RETIRED_PRESCRIBED_AUTHORITY_FRAGMENTS
        .iter()
        .any(|fragment| contents.contains(fragment))
}

const RETIRED_PRESCRIBED_AUTHORITY_FRAGMENTS: &[&str] = &[
    "canonical_authority: docs/CONSTITUTION.md",
    "Foundation ADRs: ADR-0052, ADR-0053, ADR-0054",
    "grit-compatible `claim/work/done/promote`",
    "grit claim/work/done (HG-GRIT)",
    "HG-GRIT operational requirement",
];
