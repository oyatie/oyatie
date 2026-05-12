use oya_foundry_authority_cohesion_kernel::{
    validate_authority_cohesion, AuthorityCohesionError, AuthorityDocument,
};

#[test]
fn authority_cohesion_accepts_character_identical_declarations() {
    let docs = [
        doc("docs/CONSTITUTION.md", CHAIN),
        doc("docs/AGENTS.md", CHAIN),
        doc("docs/README.md", CHAIN),
    ];

    let report = validate_authority_cohesion(&docs).expect("declarations match");

    assert_eq!(report.document_count, 3);
    assert_eq!(report.declaration, "first\nsecond");
}

#[test]
fn authority_cohesion_rejects_drift_between_declarations() {
    let docs = [
        doc("docs/CONSTITUTION.md", CHAIN),
        doc("docs/AGENTS.md", DRIFTED_CHAIN),
        doc("docs/README.md", CHAIN),
    ];

    assert_eq!(
        validate_authority_cohesion(&docs),
        Err(AuthorityCohesionError::DeclarationDrift)
    );
}

#[test]
fn authority_cohesion_rejects_missing_declaration_or_path() {
    let missing_declaration = [AuthorityDocument {
        path: "docs/CONSTITUTION.md".into(),
        contents: "---\ndoc_class: Constitution\n---\n".into(),
    }];
    assert_eq!(
        validate_authority_cohesion(&missing_declaration),
        Err(AuthorityCohesionError::MissingDeclaration)
    );

    let empty_path = [AuthorityDocument {
        path: "".into(),
        contents: CHAIN.into(),
    }];
    assert_eq!(
        validate_authority_cohesion(&empty_path),
        Err(AuthorityCohesionError::EmptyDocumentPath)
    );
}

const CHAIN: &str = "---\nauthority_chain_declaration: |\n  first\n  second\n---\n";
const DRIFTED_CHAIN: &str = "---\nauthority_chain_declaration: |\n  first\n  changed\n---\n";

fn doc(path: &str, contents: &str) -> AuthorityDocument {
    AuthorityDocument {
        path: path.into(),
        contents: contents.into(),
    }
}
