// sovereign-tenant-pin readiness gate fixtures. ADR-0083 Tier-3: tests assert via unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_cloud_ci_sovereign_tenant_pin_app::{Verdict, evaluate, evaluate_keyed};
use serde_json::{Value, json};

fn scenario(
    scenario_id: &str,
    current_cell_region: &str,
    decision: &str,
    status: Option<u16>,
    location: Option<&str>,
) -> Value {
    json!({
        "scenario_id": scenario_id,
        "tenant_id": "ten_ksa_alpha",
        "home_region": "KSA-Riyadh",
        "allowed_regions": ["KSA-Riyadh"],
        "residency_class": "strict_ksa",
        "pack_id": "pack-ksa",
        "current_cell_region": current_cell_region,
        "decision": decision,
        "status": status,
        "location": location,
    })
}

#[test]
fn accepted_home_cell_and_mismatched_cell_fixtures_are_green() {
    let input = json!({
        "gate_id": "sovereign-tenant-pin",
        "scenarios": [
            scenario("ksa-home-cell-admitted", "KSA-Riyadh", "admit", Some(202), None),
            scenario(
                "ksa-us-cell-misdirected",
                "US-East1",
                "misdirect",
                Some(421),
                Some("https://api.ksa-riyadh.oyatie.example")
            ),
        ]
    });

    assert_eq!(evaluate(&input).verdict, Verdict::Green);
    assert!(evaluate_keyed(&input).is_empty());
}

#[test]
fn mismatched_cell_requires_421_misdirected_request_and_location() {
    let input = json!({
        "gate_id": "sovereign-tenant-pin",
        "scenarios": [
            scenario("missing-location", "US-East1", "misdirect", Some(421), None),
            scenario("wrong-status", "US-East1", "misdirect", Some(302), Some("https://api.ksa-riyadh.oyatie.example")),
        ]
    });

    let findings = evaluate_keyed(&input);
    let pairs = findings
        .iter()
        .map(|finding| (finding.code.as_str(), finding.key.as_str()))
        .collect::<Vec<_>>();

    assert!(pairs.contains(&("tenant_pin_location_header_missing", "missing-location")));
    assert!(pairs.contains(&("tenant_pin_misdirected_status_not_421", "wrong-status")));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn admitted_cell_requires_202_status() {
    let input = json!({
        "gate_id": "sovereign-tenant-pin",
        "scenarios": [
            scenario("home-cell-wrong-status", "KSA-Riyadh", "admit", Some(200), None),
            scenario(
                "ksa-us-cell-misdirected",
                "US-East1",
                "misdirect",
                Some(421),
                Some("https://api.ksa-riyadh.oyatie.example")
            ),
        ]
    });

    let findings = evaluate_keyed(&input);
    assert!(findings.iter().any(|finding| {
        finding.code == "tenant_pin_admitted_status_not_202"
            && finding.key == "home-cell-wrong-status"
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn corpus_must_include_both_admitted_and_misdirected_scenario_shapes() {
    let only_admitted = json!({
        "gate_id": "sovereign-tenant-pin",
        "scenarios": [scenario("ksa-home-cell-admitted", "KSA-Riyadh", "admit", Some(202), None)]
    });
    let only_misdirected = json!({
        "gate_id": "sovereign-tenant-pin",
        "scenarios": [scenario(
            "ksa-us-cell-misdirected",
            "US-East1",
            "misdirect",
            Some(421),
            Some("https://api.ksa-riyadh.oyatie.example")
        )]
    });

    let admitted_findings = evaluate_keyed(&only_admitted);
    assert!(admitted_findings.iter().any(|finding| {
        finding.code == "tenant_pin_no_misdirected_scenario"
            && finding.key == "<missing-misdirected-scenario>"
    }));

    let misdirected_findings = evaluate_keyed(&only_misdirected);
    assert!(misdirected_findings.iter().any(|finding| {
        finding.code == "tenant_pin_no_admitted_scenario"
            && finding.key == "<missing-admitted-scenario>"
    }));
}

#[test]
fn tenant_registry_fields_fail_closed_when_missing() {
    let input = json!({
        "gate_id": "sovereign-tenant-pin",
        "scenarios": [{
            "scenario_id": "incomplete-registry-row",
            "tenant_id": "ten_ksa_alpha",
            "home_region": "KSA-Riyadh",
            "residency_class": "strict_ksa",
            "current_cell_region": "KSA-Riyadh",
            "decision": "admit"
        }]
    });

    let findings = evaluate_keyed(&input);
    let keys = findings
        .iter()
        .map(|finding| (finding.code.as_str(), finding.key.as_str()))
        .collect::<Vec<_>>();

    assert!(keys.contains(&(
        "tenant_pin_row_missing_field",
        "incomplete-registry-row:allowed_regions"
    )));
    assert!(keys.contains(&(
        "tenant_pin_row_missing_field",
        "incomplete-registry-row:pack_id"
    )));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}

#[test]
fn strict_sovereign_residency_pins_to_one_allowed_home_region() {
    let input = json!({
        "gate_id": "sovereign-tenant-pin",
        "scenarios": [{
            "scenario_id": "strict-class-too-wide",
            "tenant_id": "ten_ksa_alpha",
            "home_region": "KSA-Riyadh",
            "allowed_regions": ["KSA-Riyadh", "US-East1"],
            "residency_class": "strict_ksa",
            "pack_id": "pack-ksa",
            "current_cell_region": "KSA-Riyadh",
            "decision": "admit",
            "status": 202
        }]
    });

    let findings = evaluate_keyed(&input);
    assert!(findings.iter().any(|finding| {
        finding.code == "tenant_pin_strict_residency_not_single_home_region"
            && finding.key == "strict-class-too-wide"
    }));
    assert_eq!(evaluate(&input).verdict, Verdict::Red);
}
