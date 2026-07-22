use corpus_doc_parser::census::{
    CensusInput, CensusSource, CensusSourceKind, CensusViolation, SELECTOR_ID, build_receipt,
    census_from_git,
};

const PARENT: &str = "e548d6f4035104e15ef6e290a4799d0ff3ee66e6";

fn source(path: &str, bytes: &[u8]) -> CensusSource {
    CensusSource {
        kind: CensusSourceKind::Decision,
        path: path.to_owned(),
        blob_oid: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        bytes: bytes.to_vec(),
    }
}

fn input(sources: Vec<CensusSource>) -> CensusInput {
    CensusInput {
        repository_commit: "1111111111111111111111111111111111111111".to_owned(),
        repository_tree: "2222222222222222222222222222222222222222".to_owned(),
        docs_tree: "3333333333333333333333333333333333333333".to_owned(),
        selector_id: SELECTOR_ID.to_owned(),
        parser_sources: vec![CensusSource {
            kind: CensusSourceKind::Parser,
            path: "governance/corpus/doc-parser/src/adr.rs".to_owned(),
            blob_oid: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
            bytes: b"parser source".to_vec(),
        }],
        decision_sources: sources,
    }
}

#[test]
fn parent_corpus_is_exactly_429_direct_child_adrs_and_tree_bound() {
    let receipt = census_from_git(PARENT).expect("parent corpus census succeeds");

    assert_eq!(receipt.repository_commit(), PARENT);
    assert_eq!(receipt.selector_id(), SELECTOR_ID);
    assert_eq!(receipt.entries().len(), 429);
    assert!(
        receipt
            .entries()
            .windows(2)
            .all(|pair| pair[0].path() < pair[1].path())
    );
    assert!(receipt.docs_tree().chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(receipt.claim_ceiling(), "BLOCKED/HOLD");
}

#[test]
fn repeated_census_has_identical_canonical_bytes_and_digest() {
    let input = input(vec![source(
        "docs/decisions/ADR-0001-example.md",
        b"---\nid: ADR-0001\nstatus: proposed\ndate: 2026-01-01\nowner: corpus\n---\n# ADR-0001: Example\n",
    )]);
    let first = build_receipt(&input).expect("first census succeeds");
    let second = build_receipt(&input).expect("second census succeeds");

    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.canonical_digest(), second.canonical_digest());
}

#[test]
fn duplicate_and_omitted_paths_fail_closed() {
    let one = source("docs/decisions/ADR-0001-example.md", b"not an adr");
    let duplicate = build_receipt(&input(vec![one.clone(), one])).expect_err("duplicate rejected");
    assert_eq!(duplicate, CensusViolation::DuplicatePath);

    let omitted = build_receipt(&input(vec![source(
        "docs/decisions/archive/ADR-0002-hidden.md",
        b"not an adr",
    )]))
    .expect_err("recursive ADR is omitted from the direct-child selector");
    assert_eq!(omitted, CensusViolation::SelectorPath);
}

#[test]
fn records_only_the_first_parser_error_with_span_and_raw_message() {
    let receipt = build_receipt(&input(vec![source(
        "docs/decisions/ADR-0001-example.md",
        b"---\nid: ADR-0002\nid: ADR-0001\n---\n# ADR-0001: Example\n",
    )]))
    .expect("census records parse errors instead of dropping files");
    let error = receipt.entries()[0]
        .first_error()
        .expect("first parser error recorded");

    assert_eq!(error.kind(), "DuplicateFrontmatterKey");
    assert!(error.raw().contains("duplicate ADR frontmatter key"));
    assert!(error.span().is_some());
}

#[test]
fn a_one_byte_mutation_changes_entry_and_aggregate_digests() {
    let original = input(vec![source(
        "docs/decisions/ADR-0001-example.md",
        b"---\nid: ADR-0001\nstatus: proposed\ndate: 2026-01-01\nowner: corpus\n---\n# ADR-0001: Example\n",
    )]);
    let mut changed = original.clone();
    changed.decision_sources[0].bytes[20] = b'x';

    let before = build_receipt(&original).expect("original census");
    let after = build_receipt(&changed).expect("mutated census");
    assert_ne!(before.entries()[0].sha256(), after.entries()[0].sha256());
    assert_ne!(before.aggregate_fold(), after.aggregate_fold());
    assert_ne!(before.canonical_digest(), after.canonical_digest());
}

#[test]
fn parser_and_tree_bindings_change_the_canonical_digest() {
    let original = input(vec![source(
        "docs/decisions/ADR-0001-example.md",
        b"not an adr",
    )]);
    let mut changed_tree = original.clone();
    changed_tree.docs_tree.replace_range(..1, "4");
    let mut changed_parser = original.clone();
    changed_parser.parser_sources[0].bytes.push(b'!');

    let before = build_receipt(&original).expect("original census");
    assert_ne!(
        before.canonical_digest(),
        build_receipt(&changed_tree).unwrap().canonical_digest()
    );
    assert_ne!(
        before.canonical_digest(),
        build_receipt(&changed_parser).unwrap().canonical_digest()
    );
}

#[test]
fn stale_193_and_236_counts_are_not_comparable_to_the_pinned_parent_corpus() {
    let receipt = census_from_git(PARENT).expect("parent corpus census succeeds");
    assert_ne!(receipt.entries().len(), 193);
    assert_ne!(receipt.entries().len(), 236);
}
