use super::{AdrDocument, audit_adr_shape_fitness};

fn document(path: &str, text: &str) -> AdrDocument {
    AdrDocument {
        path: path.to_owned(),
        text: text.to_owned(),
    }
}

#[test]
fn diagnostic_is_deterministic_for_reversed_input_order() {
    let valid = document(
        "docs/decisions/ADR-9001-valid.md",
        "# ADR-9001: Record the diagnostic boundary\n\n> **Status:** Proposed\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- Determinism.\n\n## Consequences\nC\n",
    );
    let malformed = document(
        "docs/decisions/ADR-9002-malformed.md",
        "    # ADR-9002: Pseudo ADR\n\n    ## Context\n",
    );

    let forward = audit_adr_shape_fitness(&[valid.clone(), malformed.clone()]);
    let reverse = audit_adr_shape_fitness(&[malformed, valid]);

    assert_eq!(forward, reverse);
    assert!(
        forward
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_SECTION_MISSING")
    );
}

#[test]
fn diagnostic_rejects_malformed_and_misnested_structure_without_false_sections() {
    let report = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9003-structure.md",
        "# ADR-9003: Diagnose fenced headings\n\n> **Status:** Proposed\n\n```md\n## Context\n~~~\n## Decision\n```still-open\n## Consequences\n\n### Decision Drivers\n- misplaced\n",
    )]);

    assert!(report.findings.iter().any(
        |finding| finding.code == "ADR_SECTION_MISSING" && finding.message.contains("Context")
    ));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.code == "ADR_DECISION_DRIVERS_MISNESTED_OR_MISSING" })
    );
}

#[test]
fn diagnostic_reports_legacy_status_as_migration_inventory_not_live_acceptance() {
    let report = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9004-legacy-status.md",
        "# ADR-9004: Preserve status evidence\n\n> **Status:** accepted (historical)\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
    )]);

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_STATUS_MIGRATION_INVENTORY")
    );
}

#[test]
fn diagnostic_rejects_duplicate_and_misordered_headings() {
    let report = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9005-order.md",
        "# ADR-9005: Preserve structure\n\n> **Status:** Proposed\n\n## Decision\nB\n\n## Context\nA\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n\n## Context\nDuplicate\n",
    )]);

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_SECTION_DUPLICATE")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_SECTION_OUT_OF_ORDER")
    );
}

#[test]
fn diagnostic_does_not_treat_escaped_table_pipes_as_headings() {
    let report = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9006-escaped-pipe.md",
        "# ADR-9006: Preserve table boundaries\n\n> **Status:** Proposed\n\n| Field | Value |\n| --- | --- |\n| note | \\| ## Context |\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
    )]);

    assert!(report.findings.iter().any(|finding| {
        finding.code == "ADR_SECTION_MISSING" && finding.message.contains("Context")
    }));
}

#[test]
fn diagnostic_inventories_missing_status_and_ignores_fenced_pseudo_status() {
    let missing = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9007-missing-status.md",
        "# ADR-9007: Missing status\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
    )]);
    assert!(
        missing
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
    );

    let fenced = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9008-fenced-status.md",
        "# ADR-9008: Fenced status\n\n```md\n> **Status:** Accepted\n```\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
    )]);
    assert!(
        fenced
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
    );

    let block_literal = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9009-block-status.md",
        "---\nnotes: |\n  > **Status:** Accepted\n---\n\n# ADR-9009: Block status\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
    )]);
    assert!(
        block_literal
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
    );
}

#[test]
fn diagnostic_uses_visible_body_status_when_frontmatter_omits_status() {
    let report = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9010-frontmatter-body-status.md",
        "---\nid: ADR-9010\n---\n\n# ADR-9010: Body status fallback\n\n> **Status:** Accepted\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
    )]);

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| { finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY" })
    );
}

#[test]
fn diagnostic_ignores_status_inside_block_quoted_fences() {
    let report = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9011-quoted-fence.md",
        "# ADR-9011: Quoted fence\n\n> ```md\n> **Status:** Accepted\n> ```\n\n## Context\nA\n\n## Decision\nB\n\n## Decision Drivers\n- C\n\n## Consequences\nD\n",
    )]);

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
    );
}

#[test]
fn diagnostic_does_not_promote_fully_quoted_pseudo_adr_sections() {
    let report = audit_adr_shape_fitness(&[document(
        "docs/decisions/ADR-9012-quoted-pseudo-adr.md",
        "> # ADR-9012: Quoted pseudo ADR\n>\n> **Status:** Proposed\n>\n> ## Context\n> A\n>\n> ## Decision\n> B\n>\n> ## Decision Drivers\n> - C\n>\n> ## Consequences\n> D\n",
    )]);

    for section in ["Context", "Decision", "Decision Drivers", "Consequences"] {
        assert!(report.findings.iter().any(|finding| {
            finding.code == "ADR_SECTION_MISSING"
                && finding.message == format!("missing ## {section}")
        }));
    }
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.code == "ADR_STATUS_MISSING_MIGRATION_INVENTORY")
    );
}
