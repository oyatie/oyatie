use std::collections::BTreeMap;

use shared_platform_contracts_kernel::pdp::PolicyVersion;

use crate::PolicyBundle;

fn seed_bundle_json_without_overlays() -> String {
    // A pre-G004 flat bundle document: no `tenant_policies` field at all.
    serde_json::json!({
        "version": "psv-000001",
        "schema_src": "schema",
        "policies_src": "policies",
        "templates": [],
        "template_links": [],
        "action_map": {},
    })
    .to_string()
}

#[test]
fn flat_bundle_without_overlays_field_still_parses_backward_compatible() {
    let bundle: PolicyBundle = serde_json::from_str(&seed_bundle_json_without_overlays()).unwrap();
    assert!(
        bundle.tenant_policies.is_empty(),
        "an absent tenant_policies field defaults to empty (backward compatible)"
    );
}

#[test]
fn tenant_policies_round_trip_through_serde_deterministically() {
    let bundle = PolicyBundle {
        version: PolicyVersion::new("psv-000001").unwrap(),
        schema_src: "schema".to_owned(),
        policies_src: "policies".to_owned(),
        tenant_policies: BTreeMap::from([
            ("globex".to_owned(), "// globex overlay".to_owned()),
            ("acme".to_owned(), "// acme overlay".to_owned()),
        ]),
        templates: vec![],
        template_links: vec![],
        action_map: BTreeMap::new(),
    };
    let json = serde_json::to_string(&bundle).unwrap();
    let back: PolicyBundle = serde_json::from_str(&json).unwrap();
    assert_eq!(back, bundle);
    // BTreeMap keeps overlays in a deterministic (sorted) order.
    let keys: Vec<&String> = back.tenant_policies.keys().collect();
    assert_eq!(keys, vec!["acme", "globex"]);
}

#[test]
fn unknown_bundle_field_is_rejected_closed_schema() {
    let mut value: serde_json::Value =
        serde_json::from_str(&seed_bundle_json_without_overlays()).unwrap();
    value["smuggled"] = serde_json::json!("x");
    assert!(
        serde_json::from_value::<PolicyBundle>(value).is_err(),
        "deny_unknown_fields must still reject unknown fields after the overlay addition"
    );
}
