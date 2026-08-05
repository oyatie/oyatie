// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::expect_used, clippy::panic)]

const CONTRACT: &str = include_str!("../../../contracts/cloud-secrets-resource-contract.json");

const REQUIRED_RESOURCE_TYPES: &[&str] = &["Secret", "SecretMount", "SecretPolicy"];
const REQUIRED_FACETS: &[&str] = &[
    "lifecycle",
    "identity",
    "policy",
    "quota",
    "billing",
    "audit",
    "observability",
    "rollback",
    "reconciliation",
];
const REQUIRED_GAP_FIELDS: &[&str] = &[
    "orn_resource_registry",
    "operation_ledger_lro",
    "cedar_runtime",
    "quota_billing_meters",
    "audit_chain_persistence",
    "opentelemetry_openslo_evidence",
    "rollback",
    "reconciliation",
    "live_openbao_hsm_actuation",
];
const READINESS_OVERCLAIMS: &[&str] = &[
    "production_readiness",
    "runtime_ready",
    "measured_slo_ready",
    "audit_chain_persistence",
    "live_openbao_hsm_actuation",
];

fn matching_object_end(packet: &str, object_start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, byte) in packet.as_bytes()[object_start..].iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }

            match byte {
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(object_start + offset + 1);
                }
            }
            _ => {}
        }
    }

    None
}

fn resource_contract_object<'a>(packet: &'a str, resource_type: &str) -> Result<&'a str, String> {
    let needle = format!("\"resource_type\": \"{resource_type}\"");
    let resource_field_start = packet
        .find(&needle)
        .ok_or_else(|| format!("missing resource type {resource_type}"))?;
    let object_start = packet[..resource_field_start]
        .rfind('{')
        .ok_or_else(|| format!("missing object for resource type {resource_type}"))?;
    let object_end = matching_object_end(packet, object_start)
        .ok_or_else(|| format!("unterminated object for resource type {resource_type}"))?;

    Ok(&packet[object_start..object_end])
}

fn assert_local_resource_contract(packet: &str) -> Result<(), String> {
    for resource_type in REQUIRED_RESOURCE_TYPES {
        let resource_object = resource_contract_object(packet, resource_type)?;

        for facet in REQUIRED_FACETS {
            let needle = format!("\"{facet}\":");
            if !resource_object.contains(&needle) {
                return Err(format!(
                    "missing required facet {facet} for resource type {resource_type}"
                ));
            }
        }
    }

    for gap_field in REQUIRED_GAP_FIELDS {
        let needle = format!("\"{gap_field}\"");
        if !packet.contains(&needle) {
            return Err(format!("missing gap/non-claim field {gap_field}"));
        }
    }

    for overclaim in READINESS_OVERCLAIMS {
        let true_overclaim = format!("\"{overclaim}\": true");
        if packet.contains(&true_overclaim) {
            return Err(format!("forbidden readiness overclaim {overclaim}=true"));
        }
    }

    if !packet.contains("\"raw_secret_material_allowed\": false") {
        return Err("contract must explicitly reject raw secret material".to_owned());
    }

    if !packet.contains("t_bc655724")
        || !packet.contains("t_49514ca4")
        || !packet.contains("t_688c8b9b")
    {
        return Err("contract must record active de-dupe task boundaries".to_owned());
    }

    Ok(())
}

#[test]
fn cloud_secrets_resource_contract_has_required_facets_and_non_claims() {
    assert_local_resource_contract(CONTRACT)
        .expect("local Cloud Secrets resource contract is valid");
}

#[test]
fn red_missing_required_facets_are_rejected() {
    let missing_facets = r#"
    {
      "contract_id": "bad-missing-facets",
      "resource_types": [
        { "resource_type": "Secret" },
        { "resource_type": "SecretMount" },
        { "resource_type": "SecretPolicy" }
      ],
      "gap_non_claims": {
        "orn_resource_registry": "named only",
        "operation_ledger_lro": "named only",
        "cedar_runtime": "named only",
        "quota_billing_meters": "named only",
        "audit_chain_persistence": "named only",
        "opentelemetry_openslo_evidence": "named only",
        "rollback": "named only",
        "reconciliation": "named only",
        "live_openbao_hsm_actuation": "named only"
      },
      "raw_secret_material_allowed": false
    }
    "#;

    let err = assert_local_resource_contract(missing_facets).unwrap_err();
    assert!(
        err.contains("missing required facet lifecycle for resource type Secret"),
        "{err}"
    );
}

#[test]
fn red_runtime_and_evidence_overclaims_are_rejected() {
    for overclaim in READINESS_OVERCLAIMS {
        let overclaim_field = format!("\"{overclaim}\": false");
        let overclaim_true = format!("\"{overclaim}\": true");
        assert!(
            CONTRACT.contains(&overclaim_field),
            "CONTRACT must include {overclaim_field}"
        );

        let overclaiming_packet = CONTRACT.replacen(&overclaim_field, &overclaim_true, 1);
        let err = assert_local_resource_contract(&overclaiming_packet).unwrap_err();
        assert!(
            err.contains(&format!("forbidden readiness overclaim {overclaim}=true")),
            "{err}"
        );
    }
}
