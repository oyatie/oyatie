use corpus_doc_parser::{
    DocNodeKind, DocParseError, DocParseInput, TaintReason, parse_markdown_doc,
};

const ADR_FIXTURE: &str = include_str!("fixtures/adr-heading-reference.md");
const ADVERSARIAL_FIXTURE: &str = include_str!("fixtures/adversarial-exfil.md");

#[test]
fn adr_fixture_produces_stable_heading_and_reference_ids() {
    let input = DocParseInput::new(
        "tenant-foundation",
        "docs/decisions/ADR-0517-one-owned-ast-substrate-content-addressed.md",
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
        "docs/decisions/ADR-0517-one-owned-ast-substrate-content-addressed.md",
        ADR_FIXTURE,
    ))
    .expect("tenant A parses");
    let tenant_b = parse_markdown_doc(&DocParseInput::new(
        "tenant-b",
        "docs/decisions/ADR-0517-one-owned-ast-substrate-content-addressed.md",
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
