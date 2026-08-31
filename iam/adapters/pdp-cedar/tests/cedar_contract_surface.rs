//! Unknown actions, obligation pass-through, and crate-local seed drift.
//!
//! Part of the G004 Cedar conformance suite; shared fixtures in `conformance/`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod conformance;

use conformance::*;

#[test]
fn unknown_action_fails_closed() {
    let pdp = pdp(vec![]);
    let err = pdp
        .authorize(
            &request(
                "req-err-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.purge",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap_err();
    assert!(matches!(err, PdpError::UnknownAction { .. }));
}

#[test]
fn obligations_ride_out_with_annotated_permits() {
    let mut bundle = locked_seed_bundle("psv-000001", vec![]);
    bundle.policies_src.push_str(
        "\n@id(\"workload-read-grant\")\n@obligation(\"emit-step-up-audit\")\npermit (\n  principal is OyaPlatform::WorkloadIdentity,\n  action == OyaPlatform::Action::\"ReadResource\",\n  resource\n)\nwhen { principal.tenant_id == resource.tenant_id };\n",
    );
    let pdp = CedarPdp::load(&bundle, Arc::new(SeededIdGenerator::default()), 64).unwrap();
    // acme-doc-2 is non-restricted: the obligation rides out on an ordinary
    // grant, undisturbed by the step-up forbid.
    let outcome = pdp
        .authorize(
            &request(
                "req-obl-1",
                "acme",
                entity_ref("OyaPlatform::WorkloadIdentity", "payments"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome
            .response
            .obligations
            .iter()
            .map(|o| o.obligation_id.as_str())
            .collect::<Vec<_>>(),
        vec!["emit-step-up-audit"]
    );
}

// ------------------------------------------------ seed parity guard ----

#[test]
fn crate_local_cedar_seeds_match_canonical() {
    const PAIRS: &[(&str, &str)] = &[
        (
            "cedar/platform.cedarschema",
            "iam/core/platform-contracts-kernel/cedar/platform.cedarschema",
        ),
        (
            "cedar/platform-policies.cedar",
            "iam/core/platform-contracts-kernel/cedar/platform-policies.cedar",
        ),
        (
            "cedar/platform-templates.cedar",
            "iam/core/platform-contracts-kernel/cedar/platform-templates.cedar",
        ),
    ];
    let (Some(crate_dir), Some(root)) = (manifest_dir(), repo_root()) else {
        eprintln!(
            "cedar_seed_parity: repo root marker not reachable (hermetic sandbox); \
             skipped {} pairs — cargo CI lane enforces parity",
            PAIRS.len()
        );
        return;
    };
    let mut mismatches = Vec::new();
    for (local, canonical) in PAIRS {
        let local_bytes = std::fs::read(crate_dir.join(local))
            .unwrap_or_else(|e| panic!("crate-local cedar seed missing: {local}: {e}"));
        let canonical_path = root.join(canonical);
        let canonical_bytes = std::fs::read(&canonical_path).unwrap_or_else(|e| {
            panic!(
                "canonical cedar seed missing: {}: {e}",
                canonical_path.display()
            )
        });
        if local_bytes != canonical_bytes {
            mismatches.push(format!("{local} != {canonical}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "crate-local cedar seed copies drifted from the canonical contract-lock \
         sources (canonical wins; sync the crate copy in the same change):\n  {}",
        mismatches.join("\n  ")
    );
}
