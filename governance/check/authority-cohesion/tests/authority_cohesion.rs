// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use check_authority_cohesion::{
    AuthorityCohesionError, AuthorityDocument, RootHubPointerError, RootHubPointerTarget,
    validate_authority_cohesion, validate_root_hub_pointer_reachability,
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

#[test]
fn preservation_quarantine_requires_off_machine_verified_restore_evidence() {
    let contract =
        std::fs::read_to_string(repo_root().join("docs/AGENTS.md")).expect("read agent contract");
    assert!(contract.contains(
        "encrypted quarantine stored\noff-machine or otherwise durably beyond the machine being wiped, with a verified ciphertext hash\nand a successful clean-room decrypt-and-restore traversal using externally recoverable identities;\nor documented and reviewed explicit intentional discard"
    ));
}

#[test]
fn root_hub_pointer_reachability_accepts_green_fixture() {
    let targets = [
        target("specs/root-hub-pointers.json", "{}"),
        target(
            "specs/master-plan-sequencing.json",
            include_str!("fixtures/master-plan-green.json"),
        ),
    ];

    let report = validate_root_hub_pointer_reachability(
        include_str!("fixtures/root-hub-green.json"),
        &targets,
    )
    .expect("green root-hub fixture resolves");

    assert_eq!(report.pointer_count, 3);
    assert_eq!(report.target_count, 2);
}

#[test]
fn root_hub_pointer_reachability_rejects_red_missing_path_fixture() {
    let targets = [target("specs/root-hub-pointers.json", "{}")];

    assert_eq!(
        validate_root_hub_pointer_reachability(
            include_str!("fixtures/root-hub-red-missing-path.json"),
            &targets,
        ),
        Err(RootHubPointerError::MissingPointerPath)
    );
}

#[test]
fn root_hub_pointer_reachability_rejects_red_missing_fragment_fixture() {
    let targets = [
        target("specs/root-hub-pointers.json", "{}"),
        target(
            "specs/master-plan-sequencing.json",
            include_str!("fixtures/master-plan-missing-fragment.json"),
        ),
    ];

    assert_eq!(
        validate_root_hub_pointer_reachability(
            include_str!("fixtures/root-hub-red-missing-fragment.json"),
            &targets,
        ),
        Err(RootHubPointerError::MissingPointerFragment)
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
fn target(path: &str, contents: &str) -> RootHubPointerTarget {
    RootHubPointerTarget {
        path: path.into(),
        contents: contents.into(),
    }
}

/// The repo root, resolved from the crate's own manifest location rather than the test process's
/// working directory (cargo runs workspace tests with the package dir as cwd, not the repo root).
fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| candidate.join("specs/root-hub-pointers.json").is_file())
        .expect("repo root")
        .to_path_buf()
}
