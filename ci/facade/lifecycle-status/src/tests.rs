// ADR-0083 Tier 3: tests assert invariants with unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

type SeenFixture<'a> = (&'a str, Option<usize>, &'a [(&'a str, usize)]);

fn policy(baseline_json: &str) -> Policy {
    policy_with(baseline_json, "{}")
}

fn policy_with(baseline_json: &str, broken_json: &str) -> Policy {
    parse_policy(&format!(
        r#"{{"configs_dir":"specs/lifecycle-configs","frozen_violation_baseline":{baseline_json},"known_broken_lanes":{broken_json}}}"#
    ))
    .expect("fixture policy parses")
}

fn lanes(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

/// Build observations. Each entry is `(lane, artifacts, violations)`; an artifact count of `None`
/// means the lane failed discovery.
fn seen(pairs: &[SeenFixture<'_>]) -> BTreeMap<String, LaneObservation> {
    pairs
        .iter()
        .map(|(lane, artifacts, kinds)| {
            let observation = match artifacts {
                None => LaneObservation::DiscoveryFailed("missing source root: gone/".to_owned()),
                Some(artifacts) => LaneObservation::Observed {
                    artifacts: *artifacts,
                    violations: kinds
                        .iter()
                        .map(|(kind, count)| ((*kind).to_owned(), *count))
                        .collect(),
                },
            };
            ((*lane).to_owned(), observation)
        })
        .collect()
}

#[test]
fn parses_the_committed_policy_shape() {
    let parsed = policy_with(
        r#"{"_comment":"prose","adr-status":{"_note":"prose","missing_supersession":3}}"#,
        r#"{"_comment":"prose","plan-status":{"defect":"root is gitignored","resolution":"delete the config"}}"#,
    );
    assert_eq!(parsed.configs_dir, "specs/lifecycle-configs");
    assert_eq!(parsed.baseline["adr-status"]["missing_supersession"], 3);
    assert_eq!(
        parsed.baseline.len(),
        1,
        "`_`-prefixed keys are prose, not lanes"
    );
    assert_eq!(parsed.known_broken_lanes.len(), 1);
    assert!(parsed.known_broken_lanes["plan-status"].contains("delete the config"));
}

#[test]
fn a_prose_only_ledger_lists_no_lane_and_leaves_every_floor_armed() {
    // The shape this policy SHIPS after the seven dark lanes took their re-root-or-delete
    // rulings: `known_broken_lanes` retains its key (parse_policy requires the object) and
    // carries nothing but review prose. The whole claim "the floors are armed" rests on that
    // prose NOT being read as a lane, because a listed lane is the only thing that suppresses
    // LaneDiscoveryFailed and LaneDiscoveredNothing. Asserted at the parse+compare seam rather
    // than trusting the two halves separately.
    let shipped = policy_with(
        "{}",
        r#"{"_comment":"EMPTY, and the emptiness is the enforcement."}"#,
    );
    assert!(
        shipped.known_broken_lanes.is_empty(),
        "prose must not be admitted as a standing excuse: {:?}",
        shipped.known_broken_lanes
    );

    let findings = compare(
        &lanes(&["api-stability-tier-lifecycle", "doc-status-lifecycle"]),
        &seen(&[
            ("api-stability-tier-lifecycle", None, &[]),
            ("doc-status-lifecycle", Some(0), &[]),
        ]),
        &shipped,
    );
    assert_eq!(
        findings,
        vec![
            Finding::LaneDiscoveryFailed {
                lane: "api-stability-tier-lifecycle".to_owned(),
                error: "missing source root: gone/".to_owned(),
            },
            Finding::LaneDiscoveredNothing {
                lane: "doc-status-lifecycle".to_owned()
            },
        ],
        "with an empty ledger a moved scan root must RED, never observe zero and report clean"
    );
}

#[test]
fn a_zero_baseline_row_is_rejected_rather_than_silently_accepted() {
    let error = parse_policy(
        r#"{"configs_dir":"d","known_broken_lanes":{},"frozen_violation_baseline":{"adr-status":{"unknown_stage":0}}}"#,
    )
    .expect_err("a zero row must not parse");
    assert!(error.contains("remove the row"), "{error}");
}

#[test]
fn a_broken_lane_entry_without_a_stated_resolution_is_rejected() {
    for entry in [
        r#"{"defect":"root gone"}"#,
        r#"{"defect":"root gone","resolution":"  "}"#,
    ] {
        let error = parse_policy(&format!(
            r#"{{"configs_dir":"d","frozen_violation_baseline":{{}},"known_broken_lanes":{{"l":{entry}}}}}"#
        ))
        .expect_err("a defect with no resolution is a permanent excuse, not debt");
        assert!(error.contains("resolution"), "{error}");
    }
}

#[test]
fn a_missing_or_malformed_policy_fails_closed() {
    assert!(parse_policy("not json").is_err());
    assert!(parse_policy(r#"{"frozen_violation_baseline":{},"known_broken_lanes":{}}"#).is_err());
    assert!(parse_policy(r#"{"configs_dir":"d","known_broken_lanes":{}}"#).is_err());
    assert!(
        parse_policy(r#"{"configs_dir":"d","frozen_violation_baseline":{}}"#).is_err(),
        "an absent known-broken ledger must not default to empty"
    );
    assert!(
        parse_policy(
            r#"{"configs_dir":"d","known_broken_lanes":{},"frozen_violation_baseline":{"l":{"k":"3"}}}"#
        )
        .is_err(),
        "a stringly-typed count must not be coerced"
    );
}

#[test]
fn a_clean_corpus_at_its_exact_baseline_is_green() {
    let findings = compare(
        &lanes(&["adr-status", "plan-status"]),
        &seen(&[
            ("adr-status", Some(440), &[("missing_supersession", 3)]),
            ("plan-status", Some(12), &[]),
        ]),
        &policy(r#"{"adr-status":{"missing_supersession":3}}"#),
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn a_config_that_no_lane_evaluates_fails_closed() {
    // The exact dark-gate failure this crate retires: the config is on disk, nothing runs it.
    let findings = compare(
        &lanes(&["adr-status", "doc-status"]),
        &seen(&[("adr-status", Some(1), &[])]),
        &policy("{}"),
    );
    assert_eq!(
        findings,
        vec![Finding::ConfigNotEvaluated {
            lane: "doc-status".to_owned()
        }]
    );
}

#[test]
fn an_unlisted_lane_that_cannot_walk_its_corpus_is_born_blocking() {
    let findings = compare(
        &lanes(&["crate-status"]),
        &seen(&[("crate-status", None, &[])]),
        &policy("{}"),
    );
    assert_eq!(
        findings,
        vec![Finding::LaneDiscoveryFailed {
            lane: "crate-status".to_owned(),
            error: "missing source root: gone/".to_owned(),
        }]
    );
}

#[test]
fn an_unlisted_lane_that_matches_zero_artifacts_is_born_blocking() {
    // The kernel's glob only errors on a MISSING DIRECTORY. A live directory whose pattern selects
    // nothing returns Ok(vec![]) and evaluates perfectly clean — a vacuous green.
    let findings = compare(
        &lanes(&["capability-status"]),
        &seen(&[("capability-status", Some(0), &[])]),
        &policy("{}"),
    );
    assert_eq!(
        findings,
        vec![Finding::LaneDiscoveredNothing {
            lane: "capability-status".to_owned()
        }]
    );
}

#[test]
fn a_listed_broken_lane_is_tolerated_until_it_starts_working() {
    let ledger = r#"{"crate-status":{"defect":"root crates/*-domain no longer exists","resolution":"re-root or delete the config"}}"#;
    let base = policy_with("{}", ledger);

    let still_broken = compare(
        &lanes(&["crate-status"]),
        &seen(&[("crate-status", None, &[])]),
        &base,
    );
    assert!(still_broken.is_empty(), "{still_broken:?}");

    let still_empty = compare(
        &lanes(&["crate-status"]),
        &seen(&[("crate-status", Some(0), &[])]),
        &base,
    );
    assert!(still_empty.is_empty(), "{still_empty:?}");

    let repaired = compare(
        &lanes(&["crate-status"]),
        &seen(&[("crate-status", Some(7), &[])]),
        &base,
    );
    assert_eq!(
        repaired,
        vec![Finding::KnownBrokenLaneNowLive {
            lane: "crate-status".to_owned(),
            artifacts: 7,
        }],
        "a lane that starts working must lose its standing excuse in the same PR"
    );
}

#[test]
fn a_ledger_entry_outliving_its_config_fails_closed() {
    let findings = compare(
        &lanes(&["adr-status"]),
        &seen(&[("adr-status", Some(1), &[])]),
        &policy_with(
            r#"{"gone":{"unknown_stage":1}}"#,
            r#"{"also-gone":{"defect":"d","resolution":"r"}}"#,
        ),
    );
    assert_eq!(
        findings,
        vec![
            Finding::BaselineLaneWithoutConfig {
                lane: "gone".to_owned()
            },
            Finding::KnownBrokenLaneWithoutConfig {
                lane: "also-gone".to_owned()
            },
        ]
    );
}

#[test]
fn a_new_violation_pair_is_born_blocking_without_a_baseline_row() {
    let findings = compare(
        &lanes(&["adr-status"]),
        &seen(&[("adr-status", Some(9), &[("unknown_stage", 1)])]),
        &policy("{}"),
    );
    assert_eq!(
        findings,
        vec![Finding::UnbaselinedViolation {
            lane: "adr-status".to_owned(),
            kind: "unknown_stage".to_owned(),
            observed: 1,
        }]
    );
}

#[test]
fn a_growing_count_regresses_and_a_shrinking_one_is_not_clerkwork() {
    let found = lanes(&["adr-status"]);
    let base = policy(r#"{"adr-status":{"missing_supersession":3}}"#);

    let grew = compare(
        &found,
        &seen(&[("adr-status", Some(440), &[("missing_supersession", 4)])]),
        &base,
    );
    assert_eq!(
        grew,
        vec![Finding::BaselineRegression {
            lane: "adr-status".to_owned(),
            kind: "missing_supersession".to_owned(),
            observed: 4,
            baseline: 3,
        }]
    );

    // PROCESS_TAX DELETE: shrink below frozen baseline is NOT BaselineStale merge-blocking.
    let shrank = compare(
        &found,
        &seen(&[("adr-status", Some(440), &[("missing_supersession", 1)])]),
        &base,
    );
    assert!(
        shrank.is_empty(),
        "honest shrink must not force hand re-freeze: {shrank:?}"
    );

    let fixed = compare(&found, &seen(&[("adr-status", Some(440), &[])]), &base);
    assert!(
        fixed.is_empty(),
        "fully fixed pair must not force hand re-freeze: {fixed:?}"
    );
}

#[test]
fn every_finding_code_and_message_is_distinct_and_names_its_subject() {
    let findings = compare(
        &lanes(&["a", "b", "c", "d"]),
        &seen(&[
            ("a", Some(3), &[("unknown_stage", 2)]),
            ("c", None, &[]),
            ("d", Some(0), &[]),
        ]),
        &policy_with(
            r#"{"a":{"overdue_transition":1},"gone":{"unknown_stage":1}}"#,
            r#"{"also-gone":{"defect":"d","resolution":"r"}}"#,
        ),
    );
    let codes: std::collections::BTreeSet<&str> = findings.iter().map(Finding::code).collect();
    // BaselineStale is retained on the enum for message/code stability but is not emitted
    // (PROCESS_TAX); this fixture covers the six live finding codes.
    assert_eq!(codes.len(), 6, "{findings:?}");
    for finding in &findings {
        assert!(finding.message().starts_with(finding.code()));
    }
}
