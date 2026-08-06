use corpus_doc_parser::census::{
    CensusInput, CensusSource, CensusSourceKind, CensusViolation, SELECTOR_ID, build_receipt,
};
use corpus_doc_parser::{
    AdrFrontmatterValue, AdrParseError, AdrParseInput, DocNodeKind, DocParseError, DocParseInput,
    TaintReason, chronology::ChronologyDisposition, chronology::ChronologyFinding,
    chronology::ChronologyInput, chronology::ChronologyViolation,
    chronology::evaluate_controlling_adr_chronology as evaluate_input, parse_adr_decision,
    parse_markdown_doc,
};

const ADR_FIXTURE: &str = include_str!("fixtures/adr-heading-reference.md");
const ADVERSARIAL_FIXTURE: &str = include_str!("fixtures/adversarial-exfil.md");

const OID_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OID_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn census_source(path: &str, bytes: &[u8]) -> CensusSource {
    CensusSource {
        kind: CensusSourceKind::Decision,
        path: path.to_owned(),
        blob_oid: OID_A.to_owned(),
        bytes: bytes.to_vec(),
    }
}

fn census_input(decision_sources: Vec<CensusSource>) -> CensusInput {
    CensusInput {
        repository_commit: OID_A.to_owned(),
        repository_tree: OID_B.to_owned(),
        docs_tree: "cccccccccccccccccccccccccccccccccccccccc".to_owned(),
        selector_id: SELECTOR_ID.to_owned(),
        parser_commit: "dddddddddddddddddddddddddddddddddddddddd".to_owned(),
        parser_sources: vec![CensusSource {
            kind: CensusSourceKind::Parser,
            path: "governance/corpus/doc-parser/src/lib.rs".to_owned(),
            blob_oid: "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_owned(),
            bytes: include_bytes!("../src/lib.rs").to_vec(),
        }],
        decision_sources,
    }
}

#[test]
fn adr_census_builder_is_pure_deterministic_and_hold_bounded() {
    let source = census_source(
        "docs/decisions/ADR-0001-example.md",
        b"---\nid: ADR-0001\nstatus: Proposed\ndate: 2026-01-01\nowner: corpus\n---\n\n# ADR-0001: Example\n",
    );
    let first = build_receipt(&census_input(vec![source.clone()]))
        .expect("pure selected sources produce a receipt");
    let second = build_receipt(&census_input(vec![source]))
        .expect("the same immutable input is deterministic");

    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.canonical_digest(), second.canonical_digest());
    assert_eq!(first.parsed_count(), 1);
    assert_eq!(first.rejected_count(), 0);
    assert_eq!(first.entries()[0].blob_oid(), OID_A);
    assert_eq!(first.claim_ceiling(), "BLOCKED/HOLD");
}

#[test]
fn adr_census_receipt_uses_a_domain_and_length_framed_entry_fold() {
    let source = census_source(
        "docs/decisions/ADR-0001-example.md",
        b"---\nid: ADR-0001\nstatus: Proposed\ndate: 2026-01-01\nowner: corpus\n---\n\n# ADR-0001: Example\n",
    );
    let receipt = build_receipt(&census_input(vec![source]))
        .expect("a selected ADR produces a deterministic receipt");

    assert_eq!(
        receipt.aggregate_fold(),
        "3a1b72ce1df5c06e90a958666d7d4835092eb5ac1e4d869f0a784f4e4fb1575a",
        "the aggregate fold is a stable, domain-separated length-framed digest"
    );
}

#[test]
fn adr_census_builder_fails_closed_for_selector_duplicates_and_parser_mismatch() {
    let source = census_source("docs/decisions/ADR-0001-example.md", b"not an ADR");
    assert_eq!(
        build_receipt(&census_input(vec![source.clone(), source])).unwrap_err(),
        CensusViolation::DuplicatePath
    );

    assert_eq!(
        build_receipt(&census_input(vec![census_source(
            "docs/decisions/archive/ADR-0001-hidden.md",
            b"not an ADR",
        )]))
        .unwrap_err(),
        CensusViolation::SelectorPath
    );

    let mut input = census_input(vec![census_source(
        "docs/decisions/ADR-0001-example.md",
        b"not an ADR",
    )]);
    input.parser_sources[0].bytes.push(b'!');
    assert_eq!(
        build_receipt(&input).unwrap_err(),
        CensusViolation::ParserSource
    );
}

#[test]
fn adr_census_builder_accepts_crlf_only_divergence_on_parser_source_bytes() {
    // Hosted windows-latest checks out *.rs as CRLF while git blobs stay LF. Soft platform
    // smoke failed with ParserSource when include_bytes captured CRLF and git cat-file LF.
    let mut input = census_input(vec![census_source(
        "docs/decisions/ADR-0001-example.md",
        b"---\nid: ADR-0001\nstatus: Proposed\ndate: 2026-01-01\nowner: corpus\n---\n\n# ADR-0001: Example\n",
    )]);
    let lf = input.parser_sources[0].bytes.clone();
    let mut crlf = Vec::with_capacity(lf.len().saturating_mul(2));
    for &b in &lf {
        if b == b'\n' {
            crlf.push(b'\r');
        }
        crlf.push(b);
    }
    input.parser_sources[0].bytes = crlf;
    build_receipt(&input).expect("CRLF-only parser source divergence must not fail closed");
}

#[test]
fn adr_census_builder_fails_closed_for_invalid_object_ids_and_wrong_source_roles() {
    let source = census_source("docs/decisions/ADR-0001-example.md", b"not an ADR");
    let mut invalid_object_id = census_input(vec![source.clone()]);
    invalid_object_id.repository_commit = OID_A.to_uppercase();
    assert_eq!(
        build_receipt(&invalid_object_id).unwrap_err(),
        CensusViolation::InvalidObjectId
    );

    let mut wrong_role = source;
    wrong_role.kind = CensusSourceKind::Parser;
    assert_eq!(
        build_receipt(&census_input(vec![wrong_role])).unwrap_err(),
        CensusViolation::SourceKind
    );
}

#[test]
fn adr_census_builder_retains_only_the_first_parser_error_with_its_source_span() {
    let receipt = build_receipt(&census_input(vec![census_source(
        "docs/decisions/ADR-0001-example.md",
        b"---\nid: ADR-0002\nid: ADR-0001\n---\n# ADR-0001: Example\n",
    )]))
    .expect("parser errors remain deterministic diagnostic data");

    let error = receipt.entries()[0]
        .first_error()
        .expect("rejected entry retains its first parser error");
    assert_eq!(receipt.entries()[0].outcome(), "rejected");
    assert_eq!(error.kind(), "DuplicateFrontmatterKey");
    assert!(error.span().is_some());
    assert!(error.raw().contains("duplicate ADR frontmatter key"));
    assert_eq!(
        receipt.first_error_kind_totals(),
        &std::collections::BTreeMap::from([("DuplicateFrontmatterKey".to_owned(), 1)])
    );
}

#[test]
fn adr_fixture_produces_stable_heading_and_reference_ids() {
    let input = DocParseInput::new(
        "tenant-foundation",
        "docs/decisions/ADR-0700-ci-admission-live-apex.md",
        ADR_FIXTURE,
    );

    let first = parse_markdown_doc(&input).expect("ADR fixture parses");
    let second = parse_markdown_doc(&input).expect("same ADR fixture parses again");
    let first_ids: Vec<&str> = first.nodes().iter().map(|node| node.stable_id()).collect();
    let second_ids: Vec<&str> = second.nodes().iter().map(|node| node.stable_id()).collect();

    assert_eq!(
        first_ids, second_ids,
        "node IDs must be stable across identical parses"
    );
    assert!(
        first_ids
            .iter()
            .all(|id| id.starts_with("docnode:v1:sha256:")),
        "node IDs must expose the content-addressed docnode wire prefix: {first_ids:?}"
    );

    let headings: Vec<(u8, &str)> = first
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            DocNodeKind::Heading { level } => Some((*level, node.text())),
            _ => None,
        })
        .collect();
    assert_eq!(
        headings,
        vec![
            (
                1,
                "ADR-0517: One owned AST substrate read by every consumer"
            ),
            (2, "Decision"),
            (3, "Threat model")
        ]
    );

    let references: Vec<(&str, &str)> = first
        .nodes()
        .iter()
        .filter(|node| node.kind() == &DocNodeKind::Reference)
        .filter_map(|node| Some((node.text(), node.target()?)))
        .collect();
    assert_eq!(
        references,
        vec![
            ("ADR-0541", "ADR-0541-corpus-liveness-graph.md"),
            ("root hub", "../../specs/root-hub-pointers.json")
        ]
    );
}

#[test]
fn reference_definition_span_includes_whitespace_before_target() {
    let source = "[tracked spec]:   ../../specs/root-hub-pointers.json\n";
    let parsed = parse_markdown_doc(&DocParseInput::new(
        "tenant-foundation",
        "docs/decisions/ADR-reference-definition.md",
        source,
    ))
    .expect("reference definition parses");

    let reference = parsed
        .nodes()
        .iter()
        .find(|node| node.kind() == &DocNodeKind::Reference)
        .expect("reference node exists");

    assert_eq!(reference.text(), "tracked spec");
    assert_eq!(
        reference.target(),
        Some("../../specs/root-hub-pointers.json")
    );
    assert_eq!(
        reference.span(),
        (
            0,
            "[tracked spec]:   ../../specs/root-hub-pointers.json".len() as u64
        ),
        "reference-definition provenance span must include the skipped spaces before the target"
    );
}

#[test]
fn adversarial_markdown_is_data_not_instruction_or_exfil() {
    let input = DocParseInput::new(
        "tenant-foundation",
        "docs/decisions/ADR-9999-adversarial-fixture.md",
        ADVERSARIAL_FIXTURE,
    );

    let parsed = parse_markdown_doc(&input).expect("adversarial fixture parses as data");
    let headings: Vec<&str> = parsed
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind(), DocNodeKind::Heading { .. }))
        .map(|node| node.text())
        .collect();
    assert_eq!(headings, vec!["Adversarial doc"]);
    assert!(
        !headings.contains(&"Fake heading inside code fence"),
        "code-fenced Markdown must stay data, not parsed instructions"
    );

    let tainted_targets: Vec<&str> = parsed
        .nodes()
        .iter()
        .filter(|node| node.taint() == Some(&TaintReason::ForbiddenLinkTarget))
        .filter_map(|node| node.target())
        .collect();
    assert_eq!(
        tainted_targets,
        vec![
            "file:///Users/jasonlee/.ssh/id_rsa",
            "http://169.254.169.254/latest/meta-data/",
            "~/Library/Keychains/login.keychain-db",
            "javascript:alert%281%29"
        ]
    );

    assert!(
        parsed.nodes().iter().any(|node| {
            node.kind() == &DocNodeKind::Rejected
                && node.taint() == Some(&TaintReason::ExecutableHtml)
        }),
        "executable HTML must be surfaced as a rejected data node"
    );
}

#[test]
fn tenant_namespace_is_external_while_source_path_remains_part_of_identity() {
    let tenant_a = parse_markdown_doc(&DocParseInput::new(
        "tenant-a",
        "docs/decisions/ADR-0700-ci-admission-live-apex.md",
        ADR_FIXTURE,
    ))
    .expect("tenant A parses");
    let tenant_b = parse_markdown_doc(&DocParseInput::new(
        "tenant-b",
        "docs/decisions/ADR-0700-ci-admission-live-apex.md",
        ADR_FIXTURE,
    ))
    .expect("tenant B parses");
    let moved_source = parse_markdown_doc(&DocParseInput::new(
        "tenant-a",
        "docs/archive/ADR-0517-one-owned-ast-substrate-content-addressed.md",
        ADR_FIXTURE,
    ))
    .expect("moved source parses");

    assert_eq!(tenant_a.nodes().len(), tenant_b.nodes().len());
    for (tenant_a_node, tenant_b_node) in tenant_a.nodes().iter().zip(tenant_b.nodes()) {
        assert_eq!(
            tenant_a_node.work_area_node_id().work_area_hash(),
            tenant_b_node.work_area_node_id().work_area_hash(),
            "tenant namespaces must not alter WorkAreaHash bytes"
        );
        assert_eq!(
            tenant_a_node.work_area_node_id().node_hash(),
            tenant_b_node.work_area_node_id().node_hash(),
            "tenant namespaces must not alter NodeContentHash bytes"
        );
        assert_eq!(
            tenant_a_node.stable_id(),
            tenant_b_node.stable_id(),
            "tenant namespaces must wrap stable identities externally"
        );
    }

    assert_eq!(tenant_a.nodes().len(), moved_source.nodes().len());
    for (original_node, moved_node) in tenant_a.nodes().iter().zip(moved_source.nodes()) {
        assert_ne!(
            original_node.work_area_node_id().locator().artifact_path(),
            moved_node.work_area_node_id().locator().artifact_path(),
            "moving identical bytes must change the source locator"
        );
        assert_ne!(
            original_node.stable_id(),
            moved_node.stable_id(),
            "source path remains part of stable occurrence identity"
        );
    }
}

#[test]
fn malformed_frontmatter_fails_closed() {
    let input = DocParseInput::new(
        "tenant-foundation",
        "docs/decisions/ADR-broken.md",
        "---\nid: ADR-broken\n# Missing closing frontmatter fence\n",
    );

    assert_eq!(
        parse_markdown_doc(&input).expect_err("malformed frontmatter must fail closed"),
        DocParseError::MalformedFrontmatter
    );
}

const ADR_PATH: &str = "docs/decisions/ADR-0517-owned-parser.md";
const LEGACY_MISSING_REQUIRED: &str = r"---
id: ADR-0517
status: Accepted
---

# ADR-0517: Legacy record without required population metadata
";
const LEGACY_FILENAME_RELATION: &str = r"---
id: ADR-0517
status: Accepted
date: 2026-06-08
owner: founder
related:
  - ADR-0516-owned-parser-foundation.md
---

# ADR-0517: Legacy filename-shaped relation
";
const LEGACY_GENERIC_NESTING: &str = r"---
id: ADR-0517
status: Accepted
date: 2026-06-08
owner: founder
session_context:
  authored: 2026-06-08
  lane: parser-foundation
---

# ADR-0517: Legacy generic nested metadata
";

fn minimal_adr(frontmatter: &str) -> String {
    format!(
        "---\n{frontmatter}---\n\n# ADR-0517: One owned parser\n\nBody prose mentions ADR-9999 but does not create an edge.\n"
    )
}

fn chronology_adr(id: &str, status: &str, date: &str, relationships: &str) -> (String, String) {
    (
        format!("docs/decisions/{id}-chronology-fixture.md"),
        format!(
            "---\nid: {id}\nstatus: {status}\ndate: {date}\nowner: governance\n{relationships}---\n\n# {id}: Chronology fixture\n"
        ),
    )
}

fn parse_chronology_adr(
    id: &str,
    status: &str,
    date: &str,
    relationships: &str,
) -> corpus_doc_parser::AdrDecision {
    let (path, source) = chronology_adr(id, status, date, relationships);
    parse_adr_decision(&AdrParseInput::new(path, source)).expect("chronology fixture parses")
}

fn parse_chronology_adr_at_path(
    path: &str,
    id: &str,
    status: &str,
    date: &str,
    relationships: &str,
) -> corpus_doc_parser::AdrDecision {
    let (_, source) = chronology_adr(id, status, date, relationships);
    parse_adr_decision(&AdrParseInput::new(path, source)).expect("chronology fixture parses")
}

fn roster(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

fn evaluate_controlling_adr_chronology(
    decisions: &[corpus_doc_parser::AdrDecision],
    controlling_ids: &[String],
) -> Result<corpus_doc_parser::chronology::ChronologyReport, ChronologyViolation> {
    evaluate_input(ChronologyInput {
        decisions,
        controlling_ids,
    })
}

#[test]
fn controlling_adr_chronology_table_drives_all_relationship_directions() {
    for (relation, reciprocal) in [
        ("amends", "amended_by"),
        ("amended_by", "amends"),
        ("supersedes", "superseded_by"),
        ("superseded_by", "supersedes"),
    ] {
        let logical_relation = if matches!(relation, "amends" | "supersedes") {
            relation
        } else {
            reciprocal
        };
        let source = parse_chronology_adr(
            "ADR-0001",
            "Accepted",
            "2026-01-02",
            &format!("{relation}: [ADR-0002]\n"),
        );
        let target = parse_chronology_adr(
            "ADR-0002",
            "Amended",
            "2026-01-02",
            &format!("{reciprocal}: [ADR-0001]\n"),
        );
        let report = evaluate_controlling_adr_chronology(
            &[source.clone(), target.clone()],
            &roster(&["ADR-0001"]),
        )
        .expect("same-day reciprocal lifecycle relationship is valid");
        assert!(
            report.findings().is_empty(),
            "{relation} must allow same-day lifecycle records"
        );

        let missing_reciprocal = evaluate_controlling_adr_chronology(
            &[
                source.clone(),
                parse_chronology_adr("ADR-0002", "Amended", "2026-01-02", ""),
            ],
            &roster(&["ADR-0001"]),
        )
        .expect("parsed population remains representable");
        assert!(missing_reciprocal.findings().iter().any(|finding| matches!(finding, ChronologyFinding::ReciprocalMismatch { relation: actual, .. } if actual == logical_relation)));

        let (source_date, target_date) = if matches!(relation, "amends" | "supersedes") {
            ("2026-01-01", "2026-01-02")
        } else {
            ("2026-01-02", "2026-01-01")
        };
        let invalid_source = parse_chronology_adr(
            "ADR-0001",
            "Accepted",
            source_date,
            &format!("{relation}: [ADR-0002]\n"),
        );
        let invalid_target = parse_chronology_adr(
            "ADR-0002",
            "Amended",
            target_date,
            &format!("{reciprocal}: [ADR-0001]\n"),
        );
        let invalid_date = evaluate_controlling_adr_chronology(
            &[invalid_source, invalid_target],
            &roster(&["ADR-0001"]),
        )
        .expect("parsed population remains representable");
        assert!(invalid_date.findings().iter().any(|finding| matches!(finding, ChronologyFinding::DateContradiction { relation: actual, .. } if actual == logical_relation)));
    }
}

#[test]
fn controlling_adr_chronology_roster_population_and_status_contract_is_fail_closed() {
    let accepted = parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", "");
    for status in ["Accepted", "Amended", "Accepted (amendment)"] {
        let decision = parse_chronology_adr("ADR-0001", status, "2026-01-01", "");
        assert!(
            evaluate_controlling_adr_chronology(&[decision], &roster(&["ADR-0001"]))
                .expect("allowlisted status is binding")
                .findings()
                .is_empty()
        );
    }
    for status in ["accepted", "\"Accepted \""] {
        let decision = parse_chronology_adr("ADR-0001", status, "2026-01-01", "");
        let report = evaluate_controlling_adr_chronology(&[decision], &roster(&["ADR-0001"]))
            .expect("near-spelling remains representable");
        assert_eq!(report.disposition(), ChronologyDisposition::Blocked);
        assert!(
            report
                .findings()
                .iter()
                .any(|finding| matches!(finding, ChronologyFinding::NonBindingController { .. }))
        );
    }
    assert_eq!(
        evaluate_controlling_adr_chronology(&[accepted.clone()], &[]).unwrap_err(),
        ChronologyViolation::EmptyRoster
    );
    assert!(matches!(
        evaluate_controlling_adr_chronology(&[accepted.clone()], &roster(&["ADR-1"])),
        Err(ChronologyViolation::InvalidRosterId { .. })
    ));
    assert!(matches!(
        evaluate_controlling_adr_chronology(
            &[accepted.clone()],
            &roster(&["ADR-0001", "ADR-0001"])
        ),
        Err(ChronologyViolation::DuplicateRosterId { .. })
    ));
    assert_eq!(
        evaluate_controlling_adr_chronology(&[accepted.clone()], &roster(&["ADR-9999"]))
            .expect("missing roster member is a deterministic finding")
            .findings(),
        &[ChronologyFinding::MissingRosterId {
            id: "ADR-9999".into()
        }]
    );
    assert!(matches!(
        evaluate_controlling_adr_chronology(&[accepted.clone(), accepted], &roster(&["ADR-0001"])),
        Err(ChronologyViolation::DuplicateSourcePath { .. })
    ));
}

#[test]
fn controlling_adr_chronology_normalizes_nonbinding_edges_and_unrostered_sources() {
    for (raw_relation, relation_on_proposed, logical_relation) in [
        ("amends", true, "amends"),
        ("supersedes", true, "supersedes"),
        ("amended_by", false, "amends"),
        ("superseded_by", false, "supersedes"),
    ] {
        let root_edges = if relation_on_proposed {
            "".to_owned()
        } else {
            format!("{raw_relation}: [ADR-0002]\n")
        };
        let proposed_edges = if relation_on_proposed {
            format!("{raw_relation}: [ADR-0001]\n")
        } else {
            "".to_owned()
        };
        let root = parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", &root_edges);
        let proposed = parse_chronology_adr("ADR-0002", "Proposed", "2026-01-01", &proposed_edges);
        let report = evaluate_controlling_adr_chronology(&[root, proposed], &roster(&["ADR-0001"]))
            .expect("nonbinding logical source is representable");
        assert!(report.findings().iter().any(|finding| matches!(finding, ChronologyFinding::NonBindingLifecycleEdge { source_id, relation, .. } if source_id == "ADR-0002" && relation == logical_relation)));
        assert!(!report.findings().iter().any(|finding| matches!(finding, ChronologyFinding::ReciprocalMismatch { source_id, .. } if source_id == "ADR-0002")));
    }

    let root = parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", "");
    let historical = parse_chronology_adr(
        "ADR-0002",
        "Superseded",
        "2026-01-02",
        "amends: [ADR-0003]\n",
    );
    let proposed_target = parse_chronology_adr(
        "ADR-0003",
        "Proposed",
        "2026-01-01",
        "amended_by: [ADR-0002]\n",
    );
    let report = evaluate_controlling_adr_chronology(
        &[root, historical, proposed_target],
        &roster(&["ADR-0001"]),
    )
    .expect("unrostered Superseded source and Proposed target are representable");
    assert!(!report.findings().iter().any(|finding| matches!(finding, ChronologyFinding::ReciprocalMismatch { source_id, .. } if source_id == "ADR-0002")));

    let missing =
        parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", "amends: [ADR-9999]\n");
    assert!(evaluate_controlling_adr_chronology(&[missing], &roster(&["ADR-0001"]))
        .expect("missing target is a finding")
        .findings()
        .iter()
        .any(|finding| matches!(finding, ChronologyFinding::MissingTargetId { target_id, .. } if target_id == "ADR-9999")));
}

#[test]
fn nonbinding_forward_edges_precede_self_and_missing_target_failures() {
    for status in ["Proposed", "Conditional"] {
        for relation in ["amends", "supersedes"] {
            for target_id in ["ADR-0002", "ADR-9999"] {
                let root = parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", "");
                let nonbinding = parse_chronology_adr(
                    "ADR-0002",
                    status,
                    "2026-01-01",
                    &format!("{relation}: [{target_id}]\n"),
                );
                let report = evaluate_controlling_adr_chronology(
                    &[root, nonbinding],
                    &roster(&["ADR-0001"]),
                )
                .expect("known nonbinding forward source is representable");
                assert_eq!(report.disposition(), ChronologyDisposition::PreparedUnbound);
                assert_eq!(
                    report
                        .findings()
                        .iter()
                        .filter(|finding| matches!(finding, ChronologyFinding::NonBindingLifecycleEdge { relation: actual, target_id: actual_target, .. } if actual == relation && actual_target == target_id))
                        .count(),
                    1
                );
                assert!(!report.findings().iter().any(|finding| matches!(
                    finding,
                    ChronologyFinding::SelfReference { .. }
                        | ChronologyFinding::MissingTargetId { .. }
                        | ChronologyFinding::ReciprocalMismatch { .. }
                        | ChronologyFinding::DateContradiction { .. }
                        | ChronologyFinding::LifecycleCycle { .. }
                )));
            }
        }
    }

    for relation in ["amended_by", "superseded_by"] {
        let target = parse_chronology_adr(
            "ADR-0001",
            "Accepted",
            "2026-01-01",
            &format!("{relation}: [ADR-9999]\n"),
        );
        let report = evaluate_controlling_adr_chronology(&[target], &roster(&["ADR-0001"]))
            .expect("unresolved inverse logical source is a finding");
        assert_eq!(report.disposition(), ChronologyDisposition::Blocked);
        assert!(report.findings().iter().any(|finding| matches!(finding, ChronologyFinding::MissingTargetId { relation: actual, target_id, .. } if actual == relation && target_id == "ADR-9999")));
    }
}

#[test]
fn controlling_adr_chronology_keeps_full_population_provenance_and_detects_cycles() {
    let accepted = parse_chronology_adr_at_path(
        "docs/decisions/ADR-0001-accepted.md",
        "ADR-0001",
        "Accepted",
        "2026-01-01",
        "amends: [ADR-0002]\n",
    );
    let proposed_duplicate = parse_chronology_adr_at_path(
        "docs/decisions/ADR-0001-proposed.md",
        "ADR-0001",
        "Proposed",
        "2026-01-01",
        "",
    );
    let proposed_one = parse_chronology_adr_at_path(
        "docs/decisions/ADR-0003-proposed-a.md",
        "ADR-0003",
        "Proposed",
        "2026-01-01",
        "",
    );
    let proposed_two = parse_chronology_adr_at_path(
        "docs/decisions/ADR-0003-proposed-b.md",
        "ADR-0003",
        "Proposed",
        "2026-01-01",
        "",
    );
    let target = parse_chronology_adr(
        "ADR-0002",
        "Accepted",
        "2026-01-01",
        "amended_by: [ADR-0001]\n",
    );
    let report = evaluate_controlling_adr_chronology(
        &[
            accepted,
            proposed_duplicate,
            proposed_one,
            proposed_two,
            target,
        ],
        &roster(&["ADR-0001", "ADR-0002"]),
    )
    .expect("distinct source paths are representable");
    assert!(report.findings().iter().any(|finding| matches!(finding, ChronologyFinding::DuplicateTargetId { target_id, source_paths } if target_id == "ADR-0001" && source_paths.len() == 2)));
    assert_eq!(report.findings().iter().filter(|finding| matches!(finding, ChronologyFinding::NonBindingDecision { id, source_path, .. } if id == "ADR-0003" && source_path.contains("proposed"))).count(), 2);

    let first = parse_chronology_adr(
        "ADR-0004",
        "Accepted",
        "2026-01-01",
        "amends: [ADR-0005]\nsuperseded_by: [ADR-0005]\n",
    );
    let second = parse_chronology_adr(
        "ADR-0005",
        "Accepted",
        "2026-01-01",
        "amended_by: [ADR-0004]\nsupersedes: [ADR-0004]\n",
    );
    let cycle =
        evaluate_controlling_adr_chronology(&[first, second], &roster(&["ADR-0004", "ADR-0005"]))
            .expect("cycle fixture is representable");
    assert_eq!(
        cycle.findings(),
        &[ChronologyFinding::LifecycleCycle {
            ids: vec!["ADR-0004".into(), "ADR-0005".into()]
        }]
    );
}

#[test]
fn controlling_adr_chronology_preserves_hold_and_only_cycles_forward_amends_and_supersedes() {
    let accepted = parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", "");
    let nonbinding = parse_chronology_adr_at_path(
        "docs/decisions/ADR-0002-proposed.md",
        "ADR-0002",
        "Proposed",
        "2026-01-01",
        "amends: [ADR-0001]\n",
    );
    let prepared = evaluate_controlling_adr_chronology(
        &[accepted.clone(), nonbinding.clone()],
        &roster(&["ADR-0001"]),
    )
    .expect("nonbinding supporting ADR is representable");
    assert_eq!(prepared.claim_ceiling(), "BLOCKED/HOLD");
    assert_eq!(
        prepared.disposition(),
        ChronologyDisposition::PreparedUnbound
    );
    assert!(prepared.findings().iter().any(|finding| matches!(finding, ChronologyFinding::NonBindingLifecycleEdge { source_id, relation, .. } if source_id == "ADR-0002" && relation == "amends")));

    let blocked =
        evaluate_controlling_adr_chronology(&[accepted, nonbinding], &roster(&["ADR-0002"]))
            .expect("rostered nonbinding ADR is representable but blocking");
    assert_eq!(blocked.disposition(), ChronologyDisposition::Blocked);
    assert!(blocked.findings().iter().any(|finding| matches!(finding, ChronologyFinding::NonBindingController { id, .. } if id == "ADR-0002")));

    let cycle_cases = [
        ("amends: [ADR-0001]\n", "", vec!["ADR-0001"]),
        (
            "amends: [ADR-0002]\n",
            "amends: [ADR-0001]\n",
            vec!["ADR-0001", "ADR-0002"],
        ),
        (
            "amends: [ADR-0002]\n",
            "supersedes: [ADR-0001]\n",
            vec!["ADR-0001", "ADR-0002"],
        ),
    ];
    for (first_edges, second_edges, ids) in cycle_cases {
        let first = parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", first_edges);
        let second = parse_chronology_adr("ADR-0002", "Accepted", "2026-01-01", second_edges);
        let report = evaluate_controlling_adr_chronology(
            &[first, second],
            &roster(&["ADR-0001", "ADR-0002"]),
        )
        .expect("cycle fixture is representable");
        if ids.len() == 1 {
            assert!(
                report
                    .findings()
                    .iter()
                    .any(|finding| matches!(finding, ChronologyFinding::SelfReference { .. }))
            );
        } else {
            assert!(report.findings().iter().any(|finding| matches!(finding, ChronologyFinding::LifecycleCycle { ids: actual } if actual == &ids.iter().map(|id| (*id).to_owned()).collect::<Vec<_>>())));
        }
    }

    let first = parse_chronology_adr("ADR-0001", "Accepted", "2026-01-01", "amends: [ADR-0002]\n");
    let second = parse_chronology_adr(
        "ADR-0002",
        "Accepted",
        "2026-01-01",
        "supersedes: [ADR-0003]\n",
    );
    let third = parse_chronology_adr("ADR-0003", "Accepted", "2026-01-01", "amends: [ADR-0001]\n");
    let ids = roster(&["ADR-0001", "ADR-0002", "ADR-0003"]);
    let forward =
        evaluate_controlling_adr_chronology(&[first.clone(), second.clone(), third.clone()], &ids)
            .expect("three-node fixture is representable");
    let reversed = evaluate_controlling_adr_chronology(&[third, second, first], &ids)
        .expect("permutation is representable");
    assert_eq!(forward, reversed);
    assert!(forward.findings().iter().any(|finding| matches!(finding, ChronologyFinding::LifecycleCycle { ids } if ids == &vec![String::from("ADR-0001"), String::from("ADR-0002"), String::from("ADR-0003")])));
}

#[test]
fn adr_ir_requires_frontmatter_at_byte_zero() {
    let missing = "# ADR-0517: One owned parser\n";
    assert_eq!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, missing))
            .expect_err("ADR metadata must be a leading frontmatter block"),
        AdrParseError::MissingLeadingFrontmatter
    );

    let mid_document = "# ADR-0517: One owned parser\n---\nid: ADR-0517\n---\n";
    assert_eq!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, mid_document))
            .expect_err("a body fence is not ADR frontmatter"),
        AdrParseError::MissingLeadingFrontmatter
    );
}

#[test]
fn adr_ir_rejects_unterminated_frontmatter_and_duplicate_keys() {
    let unterminated = "---\nid: ADR-0517\nstatus: Accepted\n";
    assert_eq!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, unterminated))
            .expect_err("unterminated frontmatter must fail closed"),
        AdrParseError::UnterminatedFrontmatter
    );

    let duplicate = minimal_adr(
        "id: ADR-0517\nstatus: Proposed\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\n",
    );
    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, duplicate)),
        Err(AdrParseError::DuplicateFrontmatterKey { ref key, .. }) if key == "status"
    ));
}

#[test]
fn adr_ir_parses_crlf_quoted_commented_null_and_empty_forms_with_byte_spans() {
    let source = concat!(
        "---\r\n",
        "id: ADR-0517\r\n",
        "title: \"One # owned parser\" # source comment\r\n",
        "status: Accepted # lifecycle is not normalized\r\n",
        "date: '2026-06-08'\r\n",
        "owner: founder\r\n",
        "supersedes: null\r\n",
        "superseded_by:\r\n",
        "amends: []\r\n",
        "amended_by: ~\r\n",
        "depends_on: ['ADR-0516', \"ADR-0520\"] # exact edges\r\n",
        "related: []\r\n",
        "---\r\n",
        "\r\n",
        "# ADR-0517: One owned parser\r\n",
    );

    let decision = parse_adr_decision(&AdrParseInput::new(ADR_PATH, source))
        .expect("supported scalar and list forms parse");

    assert_eq!(decision.id().as_str(), "ADR-0517");
    assert_eq!(decision.status(), "Accepted");
    assert!(decision.supersedes().is_empty());
    assert!(decision.superseded_by().is_empty());
    assert!(decision.amends().is_empty());
    assert!(decision.amended_by().is_empty());
    assert_eq!(
        decision
            .depends_on()
            .iter()
            .map(|reference| reference.id().as_str())
            .collect::<Vec<_>>(),
        vec!["ADR-0516", "ADR-0520"]
    );

    let title = decision.field("title").expect("title field retained");
    assert_eq!(
        title.raw_value(),
        " \"One # owned parser\" # source comment"
    );
    assert_eq!(
        &source[title.value_span().start() as usize..title.value_span().end() as usize],
        title.raw_value(),
        "frontmatter spans are byte offsets into the original CRLF source"
    );
    assert_eq!(
        title.value(),
        &AdrFrontmatterValue::Scalar("One # owned parser".into())
    );
}

#[test]
fn adr_ir_preserves_mid_token_hashes_in_plain_scalars() {
    let source =
        minimal_adr("id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: team#alpha\n");

    let decision = parse_adr_decision(&AdrParseInput::new(ADR_PATH, source))
        .expect("a mid-token hash is data, not a comment delimiter");

    assert_eq!(decision.owner(), "team#alpha");
}

#[test]
fn adr_ir_rejects_bad_indentation_and_unsupported_nesting() {
    let bad_indent = minimal_adr(
        "id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\ndepends_on:\n - ADR-0516\n",
    );
    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, bad_indent)),
        Err(AdrParseError::InvalidFrontmatter { .. })
    ));

    let nested = minimal_adr(
        "id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\naffected_surfaces:\n  crates:\n    - nested:\n        value: unsupported\n",
    );
    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, nested)),
        Err(AdrParseError::UnsupportedFrontmatterNesting { .. })
    ));
}

#[test]
fn adr_ir_accepts_only_exact_canonical_adr_ids_for_typed_edges() {
    for invalid in [
        "ADR-517",
        "ADR-05170",
        "adr-0517",
        "see ADR-0517",
        "ADR-0517 draft",
        "ADR-0517:",
    ] {
        let source = minimal_adr(&format!(
            "id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\ndepends_on: [{invalid}]\n"
        ));
        assert!(matches!(
            parse_adr_decision(&AdrParseInput::new(ADR_PATH, source)),
            Err(AdrParseError::InvalidAdrReference { .. })
        ));
    }

    let source = minimal_adr(
        "id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\ndepends_on: [ADR-0516]\n",
    );
    let decision = parse_adr_decision(&AdrParseInput::new(ADR_PATH, source))
        .expect("exact canonical edge parses");
    assert_eq!(decision.depends_on()[0].id().as_str(), "ADR-0516");
    assert_eq!(decision.depends_on()[0].raw_value(), "ADR-0516");
    assert!(
        decision
            .depends_on()
            .iter()
            .all(|reference| reference.id().as_str() != "ADR-9999"),
        "body prose references stay document nodes, not decision edges"
    );
}

#[test]
fn adr_ir_rejects_noncanonical_identity_path_mismatch_and_invalid_calendar_date() {
    let noncanonical =
        minimal_adr("id: ADR-517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\n");
    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, noncanonical)),
        Err(AdrParseError::InvalidAdrId { .. })
    ));

    let mismatch =
        minimal_adr("id: ADR-0518\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\n");
    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, mismatch)),
        Err(AdrParseError::AdrIdPathMismatch { .. })
    ));

    let invalid_date =
        minimal_adr("id: ADR-0517\nstatus: Accepted\ndate: 2026-02-30\nowner: founder\n");
    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, invalid_date)),
        Err(AdrParseError::InvalidDate { .. })
    ));
}

#[test]
fn adr_ir_carries_complete_lifecycle_surface_and_deliverable_fields() {
    let source = minimal_adr(concat!(
        "id: ADR-0517\n",
        "status: Accepted\n",
        "date: 2026-06-08\n",
        "owner: founder\n",
        "depends_on: [ADR-0516]\n",
        "supersedes: [ADR-0100]\n",
        "superseded_by: [ADR-0600]\n",
        "amends: ADR-0200\n",
        "amended_by:\n",
        "  - ADR-0700\n",
        "related: [ADR-0300, ADR-0400]\n",
        "affected_surfaces:\n",
        "  crates: [corpus-doc-parser, oya-check-adr-index]\n",
        "  specs:\n",
        "    - /specs/root-hub-pointers.json\n",
        "deliverables:\n",
        "  - id: ADR-0517-D1\n",
        "    description: \"Typed parser IR\"\n",
        "    exit_criteria: \"All parser contracts pass\"\n",
        "    verified_by: \"Buck and Cargo tests\"\n",
    ));

    let decision = parse_adr_decision(&AdrParseInput::new(ADR_PATH, &source))
        .expect("complete lifecycle fixture parses");

    assert_eq!(decision.date(), "2026-06-08");
    assert_eq!(decision.owner(), "founder");
    assert_eq!(decision.supersedes()[0].id().as_str(), "ADR-0100");
    assert_eq!(decision.superseded_by()[0].id().as_str(), "ADR-0600");
    assert_eq!(decision.amends()[0].id().as_str(), "ADR-0200");
    assert_eq!(decision.amended_by()[0].id().as_str(), "ADR-0700");
    assert_eq!(decision.related().len(), 2);
    assert_eq!(decision.affected_surfaces().len(), 2);
    assert_eq!(decision.affected_surfaces()[0].category(), "crates");
    assert_eq!(
        decision.affected_surfaces()[0].values(),
        ["corpus-doc-parser", "oya-check-adr-index"]
    );
    assert_eq!(decision.deliverables().len(), 1);
    assert_eq!(decision.deliverables()[0].id(), "ADR-0517-D1");
    assert_eq!(
        decision.deliverables()[0].description(),
        Some("Typed parser IR")
    );
    assert_eq!(
        decision.deliverables()[0].exit_criteria(),
        Some("All parser contracts pass")
    );
    assert_eq!(
        decision.deliverables()[0].verified_by(),
        Some("Buck and Cargo tests")
    );
    for key in ["affected_surfaces", "deliverables"] {
        let field = decision.field(key).expect("structured field retained");
        assert_eq!(
            &source[field.value_span().start() as usize..field.value_span().end() as usize],
            field.raw_value(),
            "structured field spans cover the complete raw nested value"
        );
        assert!(field.raw_value().contains('\n'));
    }
}

#[test]
fn adr_ir_bytes_hashes_and_order_are_stable_and_tenant_namespace_is_external() {
    let source = minimal_adr(
        "id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\ndepends_on: [ADR-0516, ADR-0520]\n",
    );
    let first =
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, &source)).expect("first parse succeeds");
    let second =
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, &source)).expect("second parse succeeds");

    assert_eq!(first.fields(), second.fields());
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.content_hash(), second.content_hash());
    assert_eq!(first.content_hash().to_hex().len(), 64);

    let tenant_a = first
        .clone()
        .within_tenant("tenant-a")
        .expect("tenant A wrapper");
    let tenant_b = second.within_tenant("tenant-b").expect("tenant B wrapper");
    assert_eq!(
        tenant_a.decision().content_hash(),
        tenant_b.decision().content_hash(),
        "tenant namespace never changes content identity"
    );
    assert_ne!(tenant_a.identity(), tenant_b.identity());
    assert_eq!(tenant_a.identity().tenant().as_str(), "tenant-a");
    assert_eq!(tenant_b.identity().tenant().as_str(), "tenant-b");

    let changed = parse_adr_decision(&AdrParseInput::new(
        ADR_PATH,
        source.replace("Accepted", "Proposed"),
    ))
    .expect("changed source still parses");
    assert_ne!(tenant_a.decision().content_hash(), changed.content_hash());
}

#[test]
fn adr_ir_preserves_lifecycle_status_spelling_without_normalization() {
    let source = minimal_adr(
        "id: ADR-0517\nstatus: Accepted (amendment)\ndate: 2026-06-08\nowner: founder\n",
    );
    let decision = parse_adr_decision(&AdrParseInput::new(ADR_PATH, source))
        .expect("live lifecycle spelling remains valid data");

    assert_eq!(decision.status(), "Accepted (amendment)");
}

#[test]
fn adr_ir_retains_supported_multiline_scalars_from_live_frontmatter_shapes() {
    let source = minimal_adr(concat!(
        "id: ADR-0517\n",
        "status: Accepted\n",
        "date: 2026-06-08\n",
        "owner: founder\n",
        "amendment_2026_05_26: true\n",
        "purpose: >\n",
        "  Define the canonical parser IR while preserving\n",
        "  exact source provenance for every retained field.\n",
    ));

    let decision = parse_adr_decision(&AdrParseInput::new(ADR_PATH, &source))
        .expect("the live folded-scalar frontmatter shape parses");
    let purpose = decision.field("purpose").expect("purpose is retained");
    assert_eq!(
        decision
            .field("amendment_2026_05_26")
            .expect("numbered metadata key retained")
            .value(),
        &AdrFrontmatterValue::Scalar("true".into())
    );

    assert_eq!(
        purpose.value(),
        &AdrFrontmatterValue::Scalar(
            "Define the canonical parser IR while preserving exact source provenance for every retained field."
                .into()
        )
    );
    assert_eq!(
        &source[purpose.value_span().start() as usize..purpose.value_span().end() as usize],
        purpose.raw_value(),
        "multiline raw value and byte span remain exact source slices"
    );
    assert!(purpose.raw_value().starts_with(" >\n  Define"));
}

#[test]
fn adr_ir_rejects_structured_metadata_forms_it_cannot_represent() {
    for unsupported in [
        "affected_surfaces: [crates]\n",
        "deliverables: [ADR-0517-D1]\n",
        "affected_surfaces: []\n  crates: [corpus-doc-parser]\n",
        "deliverables: null\n  - id: ADR-0517-D1\n",
    ] {
        let source = minimal_adr(&format!(
            "id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\n{unsupported}"
        ));
        assert!(matches!(
            parse_adr_decision(&AdrParseInput::new(ADR_PATH, source)),
            Err(AdrParseError::InvalidFrontmatter { .. })
        ));
    }
}

#[test]
fn adr_ir_rejects_block_scalars_in_affected_surface_categories() {
    for marker in ["|", ">", "|-", ">+", "|2", ">-2"] {
        let source = minimal_adr(&format!(
            "id: ADR-0517\nstatus: Accepted\ndate: 2026-06-08\nowner: founder\naffected_surfaces:\n  crates: {marker}\n    - corpus-doc-parser\n"
        ));
        assert!(
            matches!(
                parse_adr_decision(&AdrParseInput::new(ADR_PATH, source)),
                Err(AdrParseError::InvalidFrontmatter { ref message, .. })
                    if message == "block scalar values are not supported for affected surface categories"
            ),
            "block scalar marker {marker:?} must fail closed instead of becoming a surface list"
        );
    }
}

#[test]
fn adr_ir_uses_the_exact_first_h1_title_and_requires_repo_relative_paths() {
    let source = concat!(
        "---\n",
        "id: ADR-0517\n",
        "title: A different frontmatter summary\n",
        "status: Accepted\n",
        "date: 2026-06-08\n",
        "owner: founder\n",
        "---\n\n",
        "<!-- retained source note -->\n",
        "# ADR-0517 — Canonical projected title\n",
    );
    let decision =
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, source)).expect("em-dash H1 shape parses");
    assert_eq!(decision.title(), "Canonical projected title");

    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(
            "/repo/docs/decisions/ADR-0517-owned-parser.md",
            source,
        )),
        Err(AdrParseError::InvalidSourcePath { .. })
    ));
}

#[test]
fn current_corpus_migration_defects_remain_named_and_fail_closed() {
    assert_eq!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, LEGACY_MISSING_REQUIRED))
            .expect_err("missing canonical metadata cannot enter the decision population"),
        AdrParseError::MissingRequiredField { key: "date".into() }
    );

    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, LEGACY_FILENAME_RELATION)),
        Err(AdrParseError::InvalidAdrReference { ref key, ref value, .. })
            if key == "related" && value == "ADR-0516-owned-parser-foundation.md"
    ));

    assert!(matches!(
        parse_adr_decision(&AdrParseInput::new(ADR_PATH, LEGACY_GENERIC_NESTING)),
        Err(AdrParseError::UnsupportedFrontmatterNesting { .. })
    ));
}
