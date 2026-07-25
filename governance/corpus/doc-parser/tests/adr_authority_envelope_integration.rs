use corpus_doc_parser::{AdrParseInput, parse_adr_authority_envelope, parse_adr_decision};
use sha2::{Digest, Sha256};

const ADR_0515_PATH: &str =
    "docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md";
const ADR_0515_SOURCE: &str = include_str!(
    "../adr-0515-source/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md"
);

#[test]
fn tracked_adr_0515_is_an_exact_authority_envelope_without_decision_ir_promotion() {
    let input = AdrParseInput::new(ADR_0515_PATH, ADR_0515_SOURCE);
    assert!(
        parse_adr_decision(&input).is_err(),
        "the strict decision parser must continue rejecting ADR-0515's nested frontmatter"
    );

    let envelope = parse_adr_authority_envelope(&input)
        .expect("the tracked ADR-0515 source has a valid authority-envelope surface");
    assert_eq!(envelope.source_path(), ADR_0515_PATH);
    assert_eq!(envelope.canonical_bytes(), ADR_0515_SOURCE.as_bytes());
    assert_eq!(
        envelope.content_hash().to_hex(),
        format!("{:x}", Sha256::digest(ADR_0515_SOURCE.as_bytes()))
    );
    assert_eq!(envelope.id().as_str(), "ADR-0515");
    assert_eq!(envelope.status(), "Accepted");
    assert_eq!(
        envelope
            .supersedes()
            .iter()
            .map(|reference| reference.id().as_str())
            .collect::<Vec<_>>(),
        [
            "ADR-0124", "ADR-0349", "ADR-0359", "ADR-0361", "ADR-0511", "ADR-0513", "ADR-0514",
        ]
    );

    let frontmatter_end = ADR_0515_SOURCE
        .find("\n---\n")
        .expect("the tracked ADR has a closing frontmatter fence")
        + "\n---\n".len();
    assert_eq!(
        &ADR_0515_SOURCE[envelope.frontmatter_span().start() as usize
            ..envelope.frontmatter_span().end() as usize],
        &ADR_0515_SOURCE[..frontmatter_end],
        "frontmatter provenance includes both fences and excludes the body"
    );

    let affected = envelope
        .opaque_field("affected_surfaces")
        .expect("nested metadata is retained only as opaque provenance");
    assert_eq!(
        affected.raw_bytes(),
        ADR_0515_SOURCE[affected.span().start() as usize..affected.span().end() as usize]
            .as_bytes()
    );
    assert!(affected.raw_bytes().starts_with(b"affected_surfaces:\n"));
    assert!(
        affected
            .raw_bytes()
            .windows(b"oya-cloud-ci-firewall-app".len())
            .any(|window| window == b"oya-cloud-ci-firewall-app")
    );
    assert!(envelope.opaque_field("session_context").is_some());
    assert!(envelope.opaque_field("deciders").is_some());
}
