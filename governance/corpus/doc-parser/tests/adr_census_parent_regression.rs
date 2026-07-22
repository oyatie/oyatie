//! Cargo-only immutable-parent regression fixture.
//!
//! This test owns the Git boundary needed to construct the selected blob set.
//! The Buck targets intentionally do not include this file, so their tests stay
//! pure and hermetic.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

use corpus_doc_parser::census::{
    CensusInput, CensusSource, CensusSourceKind, CensusViolation, SELECTOR_ID, build_receipt,
};

const PROTECTED_PARENT: &str = "1fa09da22be819b062881eb59252f4dd4c6b550a";
const PR1_HEAD: &str = "23091bec8a0d6741ad20a37e9f58ab8054b59464";
const EXPECTED_DIRECT_ADRS: usize = 429;
const EXPECTED_PARSED: usize = 184;
const EXPECTED_REJECTED: usize = 245;
const CURRENT_PARSER_PATH: &str = "governance/corpus/doc-parser/src/lib.rs";

#[test]
fn protected_parent_corpus_regression_is_complete_deterministic_and_fail_closed() {
    let fixture = immutable_parent_fixture();
    assert_eq!(fixture.decisions.len(), EXPECTED_DIRECT_ADRS);

    let first = build_receipt(&fixture.input(fixture.decisions.clone()))
        .expect("the explicitly selected protected-parent corpus is structurally valid");
    let mut permuted = fixture.decisions.clone();
    permuted.reverse();
    let second = build_receipt(&fixture.input(permuted))
        .expect("input ordering does not change the deterministic receipt");

    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.parsed_count(), EXPECTED_PARSED);
    assert_eq!(first.rejected_count(), EXPECTED_REJECTED);
    assert_eq!(
        first.first_error_kind_totals(),
        &BTreeMap::from([
            ("InvalidAdrReference".to_owned(), 28),
            ("InvalidFrontmatter".to_owned(), 4),
            ("MissingLeadingFrontmatter".to_owned(), 26),
            ("MissingRequiredField".to_owned(), 142),
            ("UnsupportedFrontmatterNesting".to_owned(), 45),
        ])
    );
    assert_eq!(first.claim_ceiling(), "BLOCKED/HOLD");

    assert_population_rejected(&fixture, &fixture.decisions[..193]);
    assert_population_rejected(&fixture, &fixture.decisions[..236]);

    let mut byte_mismatch = fixture.input(fixture.decisions.clone());
    byte_mismatch.parser_sources[0].bytes.push(b'!');
    assert_eq!(
        build_receipt(&byte_mismatch),
        Err(CensusViolation::ParserSource),
        "the census parser source must equal the current compiled source bytes"
    );

    let parent_parser = git_bytes(
        &fixture.repo_root,
        &format!("{PROTECTED_PARENT}:{CURRENT_PARSER_PATH}"),
    );
    assert_ne!(parent_parser, include_bytes!("../src/lib.rs"));
    let mut parent_mismatch = fixture.input(fixture.decisions.clone());
    parent_mismatch.parser_sources[0].bytes = parent_parser;
    assert_eq!(
        build_receipt(&parent_mismatch),
        Err(CensusViolation::ParserSource),
        "a parent/parser source mismatch fails closed"
    );
}

fn assert_population_rejected(fixture: &ImmutableParentFixture, sources: &[CensusSource]) {
    let count = sources.len();
    assert!(
        matches!(count, 193 | 236),
        "this regression must cover the specified stale/base population mutations"
    );
    let receipt = build_receipt(&fixture.input(sources.to_vec()))
        .expect("a partial immutable input remains structurally parseable");
    assert!(
        validate_complete_parent_corpus(&receipt).is_err(),
        "a stale/base population must reject at the complete-corpus candidate boundary"
    );
}

fn validate_complete_parent_corpus(
    receipt: &corpus_doc_parser::census::CensusReceipt,
) -> Result<(), String> {
    if receipt.entries().len() != EXPECTED_DIRECT_ADRS {
        return Err(format!(
            "expected {EXPECTED_DIRECT_ADRS} direct ADR blobs, got {}",
            receipt.entries().len()
        ));
    }
    if receipt.parsed_count() != EXPECTED_PARSED || receipt.rejected_count() != EXPECTED_REJECTED {
        return Err("receipt totals do not match the pinned parent corpus".to_owned());
    }
    Ok(())
}

struct ImmutableParentFixture {
    repo_root: PathBuf,
    repository_tree: String,
    docs_tree: String,
    parser_blob_oid: String,
    decisions: Vec<CensusSource>,
}

impl ImmutableParentFixture {
    fn input(&self, decision_sources: Vec<CensusSource>) -> CensusInput {
        CensusInput {
            repository_commit: PROTECTED_PARENT.to_owned(),
            repository_tree: self.repository_tree.clone(),
            docs_tree: self.docs_tree.clone(),
            selector_id: SELECTOR_ID.to_owned(),
            parser_commit: PR1_HEAD.to_owned(),
            parser_sources: vec![CensusSource {
                kind: CensusSourceKind::Parser,
                path: CURRENT_PARSER_PATH.to_owned(),
                blob_oid: self.parser_blob_oid.clone(),
                bytes: include_bytes!("../src/lib.rs").to_vec(),
            }],
            decision_sources,
        }
    }
}

fn immutable_parent_fixture() -> ImmutableParentFixture {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the crate lives below the repository root")
        .to_path_buf();
    let mut decisions = Vec::new();
    let listing = git_output(
        &repo_root,
        &[
            "ls-tree",
            "-z",
            &format!("{PROTECTED_PARENT}:docs/decisions"),
        ],
    );
    for entry in listing
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let tab = entry
            .iter()
            .position(|byte| *byte == b'\t')
            .expect("git ls-tree entry has a tab before its path");
        let (metadata, name_with_tab) = entry.split_at(tab);
        let name = &name_with_tab[1..];
        let mut fields = metadata.split(|byte| *byte == b' ');
        let mode = fields.next().expect("ls-tree mode");
        let kind = fields.next().expect("ls-tree object kind");
        let blob_oid = std::str::from_utf8(fields.next().expect("ls-tree object id"))
            .expect("git object ids are UTF-8");
        let name = std::str::from_utf8(name).expect("decision paths are UTF-8");
        if mode == b"100644" && kind == b"blob" && name.starts_with("ADR-") && name.ends_with(".md")
        {
            decisions.push(CensusSource {
                kind: CensusSourceKind::Decision,
                path: format!("docs/decisions/{name}"),
                blob_oid: blob_oid.to_owned(),
                bytes: git_bytes(
                    &repo_root,
                    &format!("{PROTECTED_PARENT}:docs/decisions/{name}"),
                ),
            });
        }
    }

    let current_parser = git_bytes(&repo_root, &format!("{PR1_HEAD}:{CURRENT_PARSER_PATH}"));
    assert_eq!(current_parser, include_bytes!("../src/lib.rs"));

    ImmutableParentFixture {
        repository_tree: git_text(&repo_root, &format!("{PROTECTED_PARENT}^{{tree}}")),
        docs_tree: git_text(&repo_root, &format!("{PROTECTED_PARENT}:docs")),
        parser_blob_oid: git_text(&repo_root, &format!("{PR1_HEAD}:{CURRENT_PARSER_PATH}")),
        repo_root,
        decisions,
    }
}

fn git_text(repo_root: &Path, revision: &str) -> String {
    String::from_utf8(git_output(repo_root, &["rev-parse", revision]))
        .expect("git revisions are UTF-8")
        .trim()
        .to_owned()
}

fn git_bytes(repo_root: &Path, revision: &str) -> Vec<u8> {
    git_output(repo_root, &["show", revision])
}

fn git_output(repo_root: &Path, arguments: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(arguments)
        .output()
        .expect("Git is required only to construct this Cargo fixture");
    assert!(
        output.status.success(),
        "Git fixture command failed: git {}\nstderr: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
