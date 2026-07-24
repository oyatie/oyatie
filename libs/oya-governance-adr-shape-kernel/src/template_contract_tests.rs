use super::audit_adr_shape_fitness;
use super::{AdrDocument, AdrShapeFitnessError, validate_adr_shape_fitness};

fn document(text: String) -> AdrDocument {
    AdrDocument {
        path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
        text,
    }
}

fn canonical_adr() -> String {
    r#"---
id: ADR-9000
status: Accepted
bominal_source: no Bominal equivalent
---

# ADR-9000: Enforce ADR shape

## Status
Accepted

## Context
The template is the repository authority.

## Decision
Enforce the published ADR contract.

## Consequences

### Concrete file and crate changes
| Path / Crate | Change type | BNF v4.1 name | Layer |
| --- | --- | --- | --- |
| `libs/oya-governance-adr-shape-kernel/` | update | `oya-governance-adr-shape-kernel` | kernel |

### Integration via Workflow + Ontology
Not applicable; the governing integration point is documented in the affected service PRD.

### Positive
- Prevents false green.

### Negative
- Requires truthful migrations.

### Operational
- Runs in the Buck2 governance lane.

## Clean Architecture Impact
| Lane | Impact | Action required |
| --- | --- | --- |
| `dependency-direction` | Not affected | none |
| `cross-product-refusal` | Not affected | none |
| `port-location` | Not affected | none |
| `layer-correctness` | Affected | kernel contract updated |
| `composition-root-only` | Not affected | none |
| `sdk-kernel-only` | Not affected | none |

## Alternatives Considered

### Alternative 1 — Preserve the false green
- Description: Retain the minimal parser.
- Pros: No migration work.
- Cons: Published requirements remain unenforced.
- Reason rejected: It permits governance false greens.

## References
- ADR-0056.
"#
    .to_owned()
}

fn expects_error(text: String, error: AdrShapeFitnessError) {
    assert_eq!(validate_adr_shape_fitness(&[document(text)]), Err(error));
}

#[test]
fn accepts_canonical_shape_without_decision_drivers() {
    assert!(validate_adr_shape_fitness(&[document(canonical_adr())]).is_ok());
}

#[test]
fn rejects_missing_concrete_file_and_crate_consequences() {
    let text = canonical_adr().replace("### Concrete file and crate changes", "### Scope");
    expects_error(
        text,
        AdrShapeFitnessError::MissingSection {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
            section: "Concrete file and crate changes",
        },
    );
}

#[test]
fn rejects_missing_workflow_and_ontology_applicability() {
    let text =
        canonical_adr().replace("### Integration via Workflow + Ontology", "### Integration");
    expects_error(
        text,
        AdrShapeFitnessError::MissingSection {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
            section: "Integration via Workflow + Ontology",
        },
    );
}

#[test]
fn rejects_each_required_consequence_surface() {
    for section in ["Positive", "Negative", "Operational"] {
        let text = canonical_adr().replace(&format!("### {section}"), "### Omitted");
        expects_error(
            text,
            AdrShapeFitnessError::MissingSection {
                path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
                section,
            },
        );
    }
}

#[test]
fn rejects_clean_architecture_impact_without_all_six_lanes() {
    let text = canonical_adr().replace("| `sdk-kernel-only` | Not affected | none |\n", "");
    expects_error(
        text,
        AdrShapeFitnessError::MissingCleanArchitectureLane {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
            lane: "sdk-kernel-only",
        },
    );
}

#[test]
fn rejects_alternatives_without_a_rejection_rationale() {
    let text = canonical_adr().replace(
        "- Reason rejected: It permits governance false greens.\n",
        "",
    );
    expects_error(
        text,
        AdrShapeFitnessError::MalformedAlternative {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
            missing: "Reason rejected",
        },
    );
}

#[test]
fn rejects_missing_references() {
    let text = canonical_adr().replace("## References\n- ADR-0056.\n", "");
    expects_error(
        text,
        AdrShapeFitnessError::MissingSection {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
            section: "References",
        },
    );
}

#[test]
fn rejects_missing_bominal_inheritance_declaration() {
    let text = canonical_adr().replace("bominal_source: no Bominal equivalent\n", "");
    expects_error(
        text,
        AdrShapeFitnessError::MissingBominalDeclaration {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
        },
    );
}

#[test]
fn rejects_duplicate_frontmatter_fields() {
    let text = canonical_adr().replace("status: Accepted", "status: Accepted\nstatus: Proposed");
    expects_error(
        text,
        AdrShapeFitnessError::DuplicateFrontmatterField {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
            field: "status".to_owned(),
        },
    );
}

#[test]
fn rejects_malformed_frontmatter_without_using_body_status() {
    let text = canonical_adr().replace("---\n\n# ADR", "# ADR");
    expects_error(
        text,
        AdrShapeFitnessError::MalformedFrontmatter {
            path: "docs/decisions/ADR-9000-enforce-adr-shape.md".to_owned(),
        },
    );
}

#[test]
fn structural_audit_ignores_fenced_headings_and_requires_all_table_fields() {
    let text = canonical_adr().replace(
        "id: ADR-9000\nstatus: Accepted\nbominal_source: no Bominal equivalent",
        "## Frontmatter\n| Field | Value |\n| --- | --- |\n| **id** | ADR-9000 |\n| **title** | Enforce ADR shape |\n| **status** | Accepted |\n| **date** | 2026-07-24 |\n| **supersedes** | - |\n| **superseded_by** | - |\n| **owner** | crew |\n| **related** | - |\n| **bominal_source** | no Bominal equivalent |\n\n```md\n## References\n```",
    );
    let report =
        audit_adr_shape_fitness(&[document(text.replace("## References\n- ADR-0056.\n", ""))]);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_SECTION_MISSING")
    );
}

#[test]
fn structural_audit_reports_empty_frontmatter_misnested_consequence_and_incomplete_alternative() {
    let text = canonical_adr()
        .replace("bominal_source: no Bominal equivalent", "bominal_source:")
        .replace("### Positive", "## Positive")
        .replace("- Cons: Published requirements remain unenforced.\n", "");
    let report = audit_adr_shape_fitness(&[document(text)]);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_FRONTMATTER_MISSING")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_CONSEQUENCE_MISNESTED_OR_MISSING")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_ALTERNATIVE_FIELD_MISSING")
    );
}

#[test]
fn structural_audit_reports_deterministic_out_of_order_and_misnested_sections() {
    let text = canonical_adr()
        .replace("## References\n- ADR-0056.\n", "")
        .replace("## Context", "## References\n- ADR-0056.\n\n## Context")
        .replace("### Positive", "## Positive");
    let report = audit_adr_shape_fitness(&[document(text)]);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_SECTION_OUT_OF_ORDER")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_CONSEQUENCE_MISNESTED_OR_MISSING")
    );
}

#[test]
fn structural_audit_keeps_indented_mixed_fences_closed() {
    let text = canonical_adr()
        .replace(
            "## Context\nThe template is the repository authority.\n",
            "",
        )
        .replace(
            "## Decision\n",
            "   ````md\n## Context\n~~~\n## Decision\n   ````\n\n## Decision\n",
        );
    let report = audit_adr_shape_fitness(&[document(text)]);
    assert!(report.findings.iter().any(|finding| {
        finding.code == "ADR_SECTION_MISSING" && finding.message.contains("Context")
    }));
}

#[test]
fn structural_audit_keeps_trailing_text_fence_closers_open() {
    let text = format!("```md\nplaceholder\n```still-open\n{}", canonical_adr());
    let report = audit_adr_shape_fitness(&[document(text)]);
    assert!(report.findings.iter().any(|finding| {
        finding.code == "ADR_FRONTMATTER_MISSING" && finding.message.contains("Frontmatter")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.code == "ADR_SECTION_MISSING" && finding.message.contains("Context")
    }));
}
