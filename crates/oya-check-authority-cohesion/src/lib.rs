//! Foundry authority-cohesion kernel.

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
