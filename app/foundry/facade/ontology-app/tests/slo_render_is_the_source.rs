//! The checked-in SLO payloads are RENDERED, not written.
//!
//! Each `.generated.openslo.yaml` under `app/foundry/observability/slos/` must
//! be byte-identical to what the in-crate renderer produces from its typed
//! IR. A hand-edited payload therefore fails here rather than drifting
//! silently from the objective the code believes it is serving — which is the
//! failure mode that makes a declared SLO worse than none.
//!
//! Regenerate with `OYATIE_FOUNDRY_RENDER_SLOS=1 cargo nextest run -p
//! foundry-ontology-app`. That is the only writer: `src/bin/` and an explicit
//! `[[bin]]` are both refused by the layout gate, so a generator binary
//! cannot exist in this crate.
//!
//! Operator procedure: a failure here means the payload and the IR disagree.
//! Fix the IR and regenerate; never hand-edit the payload to match.

#[path = "facade_support/mod.rs"]
mod support;

use foundry_ontology_app::slo::{SLOS, SloSpec, render_openslo};
use std::collections::BTreeSet;
use support::Fixture;

/// Every declared SLI must name a metric this process actually exports.
/// An objective over a metric with no samples is a declared objective with
/// no signal, which reads as coverage while providing none.
#[tokio::test]
async fn every_indicator_names_an_exported_metric() {
    // Both sides are derived: the names come from the same table the
    // exposition renders, and the metrics come from scanning the queries
    // themselves. Neither is a list someone maintains by hand, so deleting
    // an exported metric or editing a query breaks this.
    let fixture = Fixture::new("slo-exported");
    let state = fixture.state();
    let exported = foundry_ontology_app::metrics::objective_eligible_metrics(&state);
    let ineligible = foundry_ontology_app::metrics::objective_ineligible_metrics(&state);
    // A table with no objectives satisfies every check below vacuously.
    // Deleting an objective is now an exercised move, so the floor has to be
    // executable rather than assumed.
    assert!(
        !SLOS.is_empty(),
        "no objectives are declared, so every check in this file passes over nothing"
    );
    for spec in SLOS {
        let referenced = spec.referenced_metrics();
        // A scanner that finds nothing satisfies the loop below vacuously,
        // so an objective querying series this process never exports would
        // pass by naming none of them recognisably.
        assert!(
            !referenced.is_empty(),
            "{}: no metric was recovered from this objective's queries, so the \
             check below proves nothing about it",
            spec.name
        );
        for metric in referenced {
            if let Some((_, reason)) = ineligible.iter().find(|(name, _)| *name == metric) {
                panic!(
                    "{}: an indicator names `{metric}`, which this process exports but \
                     which may not back an objective: {reason}",
                    spec.name
                );
            }
            assert!(
                exported.contains(metric.as_str()),
                "{}: an indicator names `{metric}`, which this process does not export; \
                 eligible are {exported:?}",
                spec.name
            );
        }
    }
}

#[test]
fn each_payload_is_byte_identical_to_its_render() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../observability/slos/");
    let regenerate = std::env::var("OYATIE_FOUNDRY_RENDER_SLOS").is_ok();
    for spec in SLOS {
        let path = format!("{root}{}.generated.openslo.yaml", spec.name);
        let rendered = render_openslo(spec);
        if regenerate {
            std::fs::write(&path, &rendered).expect("the payload is writable");
            continue;
        }
        let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("{path}: no rendered payload on disk ({error}); regenerate with OYATIE_FOUNDRY_RENDER_SLOS=1")
        });
        assert_eq!(
            on_disk, rendered,
            "{path} is not what the IR renders — fix the IR and regenerate, never hand-edit"
        );
    }
}

/// The filename carries the objective's identity, so a rename that misses
/// the payload would leave an orphan the gate still admits.
#[test]
fn every_payload_on_disk_belongs_to_a_declared_objective() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../observability/slos/");
    let declared: Vec<&str> = SLOS.iter().map(|spec| spec.name).collect();
    let entries = std::fs::read_dir(root).expect("the slos directory exists");
    for entry in entries {
        let file = entry.expect("a readable entry").file_name();
        let file = file.to_string_lossy();
        let stem = file
            .strip_suffix(".generated.openslo.yaml")
            .unwrap_or_else(|| panic!("{file}: only generated payloads belong here"));
        assert!(
            declared.contains(&stem),
            "{file} has no declared objective; delete it or declare it"
        );
    }
}

/// The scanner must recover the metric an objective queries, and ONLY that.
///
/// Both directions are load-bearing and neither is exercised by the shipped
/// objectives. A substring hit inside the recording-rule idiom
/// `job:foundry_read_served_total:rate5m` recovers a name whose series the
/// process does emit, so the exported-name check passes while the objective
/// reads a rule that may not exist. A hit that is never found makes that
/// check vacuous.
#[test]
fn the_scanner_reads_whole_metric_names_only() {
    let spec = |good: &'static str, total: &'static str| SloSpec {
        name: "probe",
        display_name: "probe",
        sli_class: "probe",
        description: "probe",
        good_query: good,
        total_query: total,
        target: "0.99",
        objective_display: "99%",
        counter: false,
    };

    // A bare name is recovered.
    assert_eq!(
        spec("sum(foundry_read_served_total)", "vector(1)").referenced_metrics(),
        BTreeSet::from(["foundry_read_served_total".to_owned()]),
    );
    // A recording rule that merely CONTAINS one is not that metric.
    assert!(
        spec("job:foundry_read_served_total:rate5m", "vector(1)")
            .referenced_metrics()
            .is_empty(),
        "a recording-rule name must not be reported as the metric it derives from"
    );
    // Nor is a longer name that starts with one.
    assert!(
        spec("foundry_read_served_total_bucket", "vector(1)")
            .referenced_metrics()
            .contains("foundry_read_served_total_bucket"),
    );
    // A query naming no exported series recovers nothing, which is what
    // makes the emptiness assertion in the check above necessary.
    assert!(
        spec("vector(0)", "vector(1)")
            .referenced_metrics()
            .is_empty(),
    );
}

/// The ineligible set is FROZEN, and it is currently EMPTY.
///
/// `objective_eligible` partitions the metric table and nothing else
/// reconciles it, so the verdict has to be asserted rather than merely
/// consumed. `foundry_projection_lag` was the sole member until its head
/// became durable: a gauge that could not move became one that can, so an
/// objective over it can breach and it is no longer barred.
///
/// Emptiness is the claim, not the absence of one. Marking any series
/// ineligible fails here, which is the point — barring a series from backing
/// an objective is a judgement a human should confirm, and the reason it
/// carries is the thing an operator reads when their objective is refused.
#[tokio::test]
async fn no_series_is_currently_barred_from_backing_an_objective() {
    let fixture = Fixture::new("slo-eligibility");
    let state = fixture.state();
    let ineligible = foundry_ontology_app::metrics::objective_ineligible_metrics(&state);
    let names: BTreeSet<&str> = ineligible.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        names,
        BTreeSet::new(),
        "a series was barred from backing an objective; if that is deliberate, \
         say why here and give the reason an operator will read"
    );
    // The converse, which IS reachable today over every sample: an eligible
    // series must carry no reason. A verdict flipped to eligible while its
    // reason text stayed behind is the half-edit the freeze above cannot see.
    for sample in foundry_ontology_app::metrics::samples(&state) {
        assert_eq!(
            sample.objective_eligible,
            sample.ineligible_because.is_empty(),
            "{}: an eligible series must carry no reason, and a barred one must \
             explain itself; got eligible={} reason={:?}",
            sample.name,
            sample.objective_eligible,
            sample.ineligible_because
        );
    }
}
