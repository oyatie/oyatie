//! The incident that motivated this kernel (oyatie-gr1n), replayed against REAL crates.io index
//! records rather than invented ones.
//!
//! `serde_yaml` was nearly added to a new crate while unmaintained. Every existing gate said it was
//! fine: RustSec has never filed an unmaintained advisory for it, so `deny.toml`'s
//! `advisories.unmaintained = "workspace"` had nothing to fire on, and `0.9.34+deprecated` IS the
//! latest published version, so any "is a newer version available?" check reports it current.
//!
//! The fixtures are trimmed verbatim from the crates.io sparse index — first release plus the six
//! most recent, reduced to the fields this kernel reads.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use ci_dep_freshness::kernel::{DeclaredDependency, Waivers, distill, evaluate};

/// The day the incident was reported, so this test never depends on a clock.
const AS_OF: &str = "2026-08-17";
/// Matches the `oya-deps.toml` default. Passed as data, never read from a constant in the kernel.
const STALE_AFTER_DAYS: i64 = 90;

fn fixture(name: &str) -> (String, String) {
    let path = format!(
        "{}/tests/fixtures/{name}.ndjson",
        env!("CARGO_MANIFEST_DIR")
    );
    (
        name.to_string(),
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}")),
    )
}

#[test]
fn serde_yaml_is_flagged_stale_and_saphyr_is_not() {
    let mirror = distill(&[fixture("serde_yaml"), fixture("saphyr")]);

    let serde_yaml = mirror.iter().find(|r| r.name == "serde_yaml").unwrap();
    assert_eq!(serde_yaml.latest_stable, "0.9.34+deprecated");
    assert_eq!(serde_yaml.last_release_date, "2024-03-25");
    let saphyr = mirror.iter().find(|r| r.name == "saphyr").unwrap();
    assert_eq!(saphyr.last_release_date, "2026-07-11");

    // Both are declared at exactly their latest version, so NEITHER is BEHIND. That is the whole
    // point: a version-only bot reports both as up to date, forever.
    let declared = vec![
        DeclaredDependency {
            name: "serde_yaml".into(),
            version: "0.9.34+deprecated".into(),
        },
        DeclaredDependency {
            name: "saphyr".into(),
            version: "0.0.11".into(),
        },
    ];
    let findings = evaluate(
        &mirror,
        &declared,
        STALE_AFTER_DAYS,
        AS_OF,
        &BTreeMap::new(),
        &Waivers::new(),
    );

    let codes: Vec<_> = findings
        .iter()
        .map(|f| (f.name.as_str(), f.signal.code()))
        .collect();
    assert_eq!(
        codes,
        [("serde_yaml", "DEP-FRESHNESS-STALE")],
        "serde_yaml must be the only finding: saphyr is fresh, and neither is behind"
    );
}

#[test]
fn the_stale_window_is_a_parameter_not_a_hidden_constant() {
    let mirror = distill(&[fixture("saphyr")]);
    let declared = vec![DeclaredDependency {
        name: "saphyr".into(),
        version: "0.0.11".into(),
    }];
    let findings_at = |days| {
        evaluate(
            &mirror,
            &declared,
            days,
            AS_OF,
            &BTreeMap::new(),
            &Waivers::new(),
        )
        .len()
    };
    // saphyr released 2026-07-11; as of 2026-08-17 that is 37 days.
    assert_eq!(findings_at(90), 0, "fresh under the default window");
    assert_eq!(findings_at(30), 1, "stale under a stricter window");
}
