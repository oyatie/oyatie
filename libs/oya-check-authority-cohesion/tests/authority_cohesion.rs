// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_check_authority_cohesion::{
    AuthorityCohesionError, AuthorityDocument, validate_authority_cohesion,
};

#[test]
fn authority_cohesion_accepts_character_identical_declarations() {
    let docs = [
        doc("docs/AGENTS.md", CHAIN),
        doc("docs/README.md", CHAIN),
        doc("docs/MASTERPLAN.md", CHAIN),
    ];

    let report = validate_authority_cohesion(&docs).expect("declarations match");

    assert_eq!(report.document_count, 3);
    assert_eq!(report.declaration, "first\nsecond");
}

#[test]
fn authority_cohesion_rejects_drift_between_declarations() {
    let docs = [
        doc("docs/AGENTS.md", DRIFTED_CHAIN),
        doc("docs/README.md", CHAIN),
        doc("docs/MASTERPLAN.md", CHAIN),
    ];

    assert_eq!(
        validate_authority_cohesion(&docs),
        Err(AuthorityCohesionError::DeclarationDrift)
    );
}

#[test]
fn authority_cohesion_rejects_missing_declaration_or_path() {
    let missing_declaration = [AuthorityDocument {
        path: "docs/MASTERPLAN.md".into(),
        contents: "---\ndoc_class: MasterPlan\n---\n".into(),
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

#[test]
fn authority_cohesion_rejects_retired_prescribed_authority() {
    let docs = [AuthorityDocument {
        path: "docs/MASTERPLAN.md".into(),
        contents: "---\ncanonical_authority: docs/CONSTITUTION.md\nauthority_chain_declaration: |\n  first\n  second\n---\n".into(),
    }];

    assert_eq!(
        validate_authority_cohesion(&docs),
        Err(AuthorityCohesionError::RetiredPrescribedAuthority)
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
