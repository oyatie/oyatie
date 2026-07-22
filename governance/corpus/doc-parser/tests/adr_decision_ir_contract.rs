use corpus_doc_parser::{AdrFrontmatterValue, AdrParseError, AdrParseInput, parse_adr_decision};

const ADR_PATH: &str = "docs/decisions/ADR-0517-owned-parser.md";
const LEGACY_MISSING_REQUIRED: &str =
    include_str!("fixtures/adr-corpus-missing-required-date-owner.md");
const LEGACY_FILENAME_RELATION: &str = include_str!("fixtures/adr-corpus-filename-relation.md");
const LEGACY_GENERIC_NESTING: &str = include_str!("fixtures/adr-corpus-generic-nested-metadata.md");

fn minimal_adr(frontmatter: &str) -> String {
    format!(
        "---\n{frontmatter}---\n\n# ADR-0517: One owned parser\n\nBody prose mentions ADR-9999 but does not create an edge.\n"
    )
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
