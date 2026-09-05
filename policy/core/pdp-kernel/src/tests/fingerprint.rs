use shared_platform_contracts_kernel::pdp::PolicyVersion;

use crate::request_fingerprint;

use super::{request, slice};

#[test]
fn fingerprint_ignores_correlation_and_freshness_fields() {
    let base = request_fingerprint(&request(), &slice());
    let mut r = request();
    r.request_id = "req-2".to_owned();
    r.min_policy_version = Some(PolicyVersion::new("psv-9").unwrap());
    assert_eq!(request_fingerprint(&r, &slice()), base);
}

#[test]
fn fingerprint_is_entity_order_independent() {
    let base = request_fingerprint(&request(), &slice());
    let mut reversed = slice();
    reversed.entities.reverse();
    assert_eq!(request_fingerprint(&request(), &reversed), base);
}

#[test]
fn fingerprint_tracks_decision_relevant_changes() {
    let base = request_fingerprint(&request(), &slice());
    let mut r = request();
    r.action = "resource.write".to_owned();
    assert_ne!(request_fingerprint(&r, &slice()), base);

    let mut attr_changed = slice();
    attr_changed.entities[0]
        .attributes
        .insert("step_up_class".to_owned(), serde_json::json!("a"));
    assert_ne!(request_fingerprint(&request(), &attr_changed), base);
}
