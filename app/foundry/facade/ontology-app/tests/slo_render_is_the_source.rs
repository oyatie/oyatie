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

use foundry_ontology_app::slo::{SLOS, render_openslo};
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
    let exported = foundry_ontology_app::metrics::exported_metric_names(&state);
    for spec in SLOS {
        for metric in spec.referenced_metrics() {
            assert!(
                exported.contains(metric.as_str()),
                "{}: an indicator names `{metric}`, which this process does not export; \
                 exported are {exported:?}",
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
