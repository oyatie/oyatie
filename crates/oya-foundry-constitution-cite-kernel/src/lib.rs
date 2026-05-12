//! Foundry Constitution citation-coverage kernel.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConstitutionCitationDocument {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstitutionCitationReport {
    pub document_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstitutionCitationError {
    EmptyDocumentPath,
    MissingConstitutionCitation,
}

pub fn validate_constitution_cite_coverage(
    documents: &[ConstitutionCitationDocument],
) -> Result<ConstitutionCitationReport, ConstitutionCitationError> {
    for document in documents {
        if document.path.trim().is_empty() {
            return Err(ConstitutionCitationError::EmptyDocumentPath);
        }
        if !has_h1_or_h2_constitution_citation(document) {
            return Err(ConstitutionCitationError::MissingConstitutionCitation);
        }
    }
    Ok(ConstitutionCitationReport {
        document_count: documents.len(),
    })
}

fn has_h1_or_h2_constitution_citation(document: &ConstitutionCitationDocument) -> bool {
    document.contents.lines().any(|line| {
        let trimmed = line.trim_start();
        let is_h1_or_h2 = (trimmed.starts_with("# ") || trimmed.starts_with("## "))
            && !trimmed.starts_with("### ");
        is_h1_or_h2
            && (trimmed.contains("CONSTITUTION.md")
                || (document.path.ends_with("CONSTITUTION.md") && trimmed.contains("Constitution")))
    })
}
