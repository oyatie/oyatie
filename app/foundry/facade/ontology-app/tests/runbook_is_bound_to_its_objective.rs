//! The response an operator acts on, and its binding to the objective it
//! answers.

use foundry_ontology_app::runbook::{RUNBOOKS, runbook_for};
use foundry_ontology_app::slo::{SLOS, render_openslo};
use std::collections::BTreeSet;

/// An objective an operator cannot act on is an alert that wakes someone with
/// nothing to do. Every objective therefore carries its response, and the
/// pairing is exhaustive in both directions: the failure mode is not a missing
/// runbook but a runbook for an objective that no longer exists, which reads
/// as coverage while naming nothing.
#[test]
fn every_objective_carries_a_response_and_no_response_outlives_its_objective() {
    let declared: BTreeSet<&str> = SLOS.iter().map(|spec| spec.name).collect();
    let responded: BTreeSet<&str> = RUNBOOKS.iter().map(|entry| entry.objective).collect();
    assert_eq!(
        declared, responded,
        "every declared objective needs a runbook and every runbook needs its objective"
    );
}

/// A runbook whose fields are present but empty satisfies the pairing above
/// while telling the operator nothing, so each field is required to say
/// something.
#[test]
fn a_response_names_a_symptom_a_first_check_a_mitigation_and_an_escalation() {
    for entry in RUNBOOKS {
        for (field, value) in [
            ("symptom", entry.runbook.symptom),
            ("first_check", entry.runbook.first_check),
            ("mitigation", entry.runbook.mitigation),
            ("escalation", entry.runbook.escalation),
        ] {
            assert!(
                !value.trim().is_empty(),
                "{}: runbook field `{field}` is empty",
                entry.objective
            );
        }
    }
}

/// The response travels with the objective rather than beside it: a consumer
/// that has the payload has the runbook, with no second artifact to fetch and
/// no link to rot.
#[test]
fn the_rendered_payload_carries_the_response_as_annotations() {
    for spec in SLOS {
        let rendered = render_openslo(spec);
        let runbook = runbook_for(spec.name).expect("a declared objective has a runbook");
        assert!(
            rendered.contains("    owner_team: foundry\n  annotations:\n"),
            "{}: rendered payload has no annotations block",
            spec.name
        );
        for value in [
            runbook.symptom,
            runbook.first_check,
            runbook.mitigation,
            runbook.escalation,
        ] {
            assert!(
                rendered.contains(value),
                "{}: rendered payload omits a runbook field",
                spec.name
            );
        }
    }
}

/// The renderer interpolates prose into double-quoted YAML scalars without
/// escaping, so a future runbook containing a quote or a backslash would emit
/// a payload that is byte-stable and unparseable. Byte-identity cannot see
/// that; parsing can.
#[test]
fn every_rendered_payload_is_parseable_yaml_carrying_its_runbook() {
    for spec in SLOS {
        let rendered = render_openslo(spec);
        let annotations = rendered
            .split_once("  annotations:\n")
            .expect("rendered payload carries an annotations block")
            .1;
        for key in [
            "runbook.symptom:",
            "runbook.first-check:",
            "runbook.mitigation:",
            "runbook.escalation:",
        ] {
            let line = annotations
                .lines()
                .find(|line| line.trim_start().starts_with(key))
                .unwrap_or_else(|| panic!("{}: no {key} annotation", spec.name));
            let value = line.trim_start().trim_start_matches(key).trim();
            assert!(
                value.starts_with('"') && value.ends_with('"'),
                "{}: {key} is not a closed quoted scalar",
                spec.name
            );
            let inner = &value[1..value.len() - 1];
            assert!(
                !inner.contains('"') && !inner.contains('\\'),
                "{}: {key} contains a character the renderer does not escape",
                spec.name
            );
        }
    }
}
