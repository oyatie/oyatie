use oya_check_constitution_cite::{
    ConstitutionCitationDocument, ConstitutionCitationError, validate_constitution_cite_coverage,
};

#[test]
fn constitution_cite_coverage_accepts_h1_or_h2_constitution_citations() {
    let docs = [
        doc("docs/CONSTITUTION.md", "# Oyatie Constitution\n"),
        doc(
            "docs/AGENTS.md",
            "# Agent Contract\n\n## Constitutional authority — [CONSTITUTION.md](CONSTITUTION.md)\n",
        ),
    ];

    let report = validate_constitution_cite_coverage(&docs).expect("citations are covered");

    assert_eq!(report.document_count, 2);
}

#[test]
fn constitution_cite_coverage_rejects_missing_h1_h2_citation() {
    let docs = [doc(
        "docs/AGENTS.md",
        "# Agent Contract\n\nBody cites [CONSTITUTION.md](CONSTITUTION.md), but heading does not.\n",
    )];

    assert_eq!(
        validate_constitution_cite_coverage(&docs),
        Err(ConstitutionCitationError::MissingConstitutionCitation)
    );
}

#[test]
fn constitution_cite_coverage_rejects_empty_path() {
    let docs = [doc("", "# Oyatie Constitution\n")];

    assert_eq!(
        validate_constitution_cite_coverage(&docs),
        Err(ConstitutionCitationError::EmptyDocumentPath)
    );
}

fn doc(path: &str, contents: &str) -> ConstitutionCitationDocument {
    ConstitutionCitationDocument {
        path: path.into(),
        contents: contents.into(),
    }
}
