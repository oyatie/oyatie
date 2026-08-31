//! The G004 audit's evasion permit shapes against the sound detector.
//!
//! Part of the G004 Cedar conformance suite; shared fixtures in `conformance/`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod conformance;

use conformance::*;

#[test]
fn structural_forbid_present_in_every_per_tenant_merged_set() {
    // Load benign overlays for BOTH tenants, then prove the structural forbid
    // governs EACH per-tenant merged set: a cross-tenant read (acme principal
    // -> globex resource) is denied AND the deny is attributed to the
    // structural forbid, which can only be true if the forbid is present and
    // armed in the per-tenant set actually evaluated.
    let acme_overlay = r#"
@id("ovl-acme-benign")
permit (
  principal == OyaPlatform::Principal::"bob",
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { principal.tenant_id == resource.tenant_id };
"#;
    let globex_overlay = r#"
@id("ovl-globex-benign")
permit (
  principal == OyaPlatform::Principal::"mallory",
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { principal.tenant_id == resource.tenant_id };
"#;
    let pdp = pdp_with_overlays(BTreeMap::from([
        ("acme".to_owned(), acme_overlay.to_owned()),
        ("globex".to_owned(), globex_overlay.to_owned()),
    ]));
    // acme's merged set: bob (acme) reading a globex resource crosses the
    // boundary and must be denied by the structural forbid.
    let acme_outcome = pdp
        .authorize(
            &request(
                "req-merged-acme",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "globex-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(acme_outcome.response.decision, Decision::Deny);
    assert!(
        acme_outcome
            .response
            .determining_policy_ids
            .iter()
            .any(|id| id == "structural-tenant-isolation"),
        "the cross-tenant deny in acme's merged set must be attributed to the \
         structural forbid (present + armed), got {:?}",
        acme_outcome.response.determining_policy_ids
    );
    // globex's merged set: mallory (globex) reading an acme resource is the
    // symmetric cross-tenant read; same forbid must govern.
    let globex_outcome = pdp
        .authorize(
            &request(
                "req-merged-globex",
                "globex",
                entity_ref("OyaPlatform::Principal", "mallory"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-2"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(globex_outcome.response.decision, Decision::Deny);
    assert!(
        globex_outcome
            .response
            .determining_policy_ids
            .iter()
            .any(|id| id == "structural-tenant-isolation"),
        "the cross-tenant deny in globex's merged set must be attributed to the \
         structural forbid (present + armed), got {:?}",
        globex_outcome.response.determining_policy_ids
    );
}

#[test]
fn entity_slice_missing_tenant_id_is_rejected_by_schema_validation() {
    // The structural forbid is only sound because the schema makes `tenant_id`
    // a REQUIRED attribute on every principal/resource. Drop it from one
    // entity and the slice must be rejected by schema validation (fail closed),
    // never silently evaluated.
    let mut slice = entity_slice();
    // Strip tenant_id from the acme-doc-1 resource record.
    let doc = slice
        .entities
        .iter_mut()
        .find(|e| e.uid.entity_id == "acme-doc-1")
        .expect("fixture must contain acme-doc-1");
    doc.attributes.remove("tenant_id");
    let pdp = pdp(vec![]);
    let err = pdp
        .authorize(
            &request(
                "req-missing-tid",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &slice,
        )
        .unwrap_err();
    assert!(
        matches!(err, PdpError::Evaluation { .. }),
        "an entity slice missing the schema-required tenant_id must be rejected, got {err:?}"
    );
}

#[test]
fn structural_forbid_denies_cross_tenant_read_for_any_permit_shape() {
    // Prove the RUNTIME forbid is the boundary, independent of the load-time
    // overlay detector: inject each audit evasion permit DIRECTLY into the
    // GLOBAL policy set (bypassing the overlay path entirely), then show a
    // cross-tenant read (bob@acme -> globex resource) is still denied. The
    // forbid wins over every permit shape, so the boundary holds even if a
    // detector were defeated.
    for (id, body) in EVASION_PERMITS {
        let mut bundle = locked_seed_bundle("psv-000001", vec![]);
        bundle
            .policies_src
            .push_str(&format!("\n@id(\"{id}\")\n{body}\n"));
        let pdp = CedarPdp::load(&bundle, Arc::new(SeededIdGenerator::default()), 64)
            .unwrap_or_else(|e| panic!("global injection of {id} must still load: {e}"));
        let outcome = pdp
            .authorize(
                &request(
                    "req-evasion-global",
                    "acme",
                    entity_ref("OyaPlatform::Principal", "bob"),
                    "resource.read",
                    entity_ref("OyaPlatform::TenantResource", "globex-doc-1"),
                ),
                &entity_slice(),
            )
            .unwrap();
        assert_eq!(
            outcome.response.decision,
            Decision::Deny,
            "{id}: the structural forbid must deny the cross-tenant read whatever \
             shape the permit takes"
        );
    }
}

#[test]
fn sound_detector_rejects_every_audit_evasion_overlay() {
    // Each evasion overlay carries the same-tenant equality as a NON-binding
    // token (behind ||, in an unless, behind !, etc.). The sound load-time
    // detector must REJECT all 5 — a non-binding accept-on-presence detector
    // would (wrongly) admit them.
    for (id, body) in EVASION_PERMITS {
        let overlay = format!("@id(\"{id}\")\n{body}");
        let result = CedarPdp::load(
            &locked_seed_bundle_with_overlays(
                "psv-000001",
                vec![],
                BTreeMap::from([("acme".to_owned(), overlay)]),
            ),
            Arc::new(SeededIdGenerator::default()),
            64,
        );
        assert!(
            matches!(result, Err(PdpError::BundleRejected { .. })),
            "{id}: a non-binding same-tenant guard must be rejected at load, got {:?}",
            result.map(|_| "loaded")
        );
    }
}

#[test]
fn sound_detector_accepts_every_legitimate_overlay_shape() {
    // The sound detector must keep accepting genuinely tenant-confined permits:
    // canonical, operand-swapped, parenthesized, and &&-nested.
    for (id, body) in LEGITIMATE_PERMITS {
        let overlay = format!("@id(\"{id}\")\n{body}");
        let result = CedarPdp::load(
            &locked_seed_bundle_with_overlays(
                "psv-000001",
                vec![],
                BTreeMap::from([("acme".to_owned(), overlay)]),
            ),
            Arc::new(SeededIdGenerator::default()),
            64,
        );
        assert!(
            result.is_ok(),
            "{id}: a legitimately tenant-confined permit must load, got {:?}",
            result.err()
        );
    }
}
