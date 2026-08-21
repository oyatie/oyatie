// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_capability_domain::{
    AutonomyTier, Capability, CapabilityCostProfile, CapabilityError, CapabilityMcpContract,
    CapabilityRegistry,
};
use data_boundary_kernel::{DataClass, PrivacyDataClass};

#[test]
fn tenant_discovery_filters_by_license_mcp_visibility_and_autonomy_ceiling() {
    let mut registry = CapabilityRegistry::default();
    let t1_visible = capability("cap.demo.visible", AutonomyTier::T1ViewOnly);
    let t3_hidden = capability("cap.demo.hidden", AutonomyTier::T3ExecuteWithApproval);
    let t3_visible = capability("cap.demo.too-high", AutonomyTier::T3ExecuteWithApproval);
    registry.publish(t1_visible.clone()).unwrap();
    registry.publish(t3_hidden.clone()).unwrap();
    registry.publish(t3_visible.clone()).unwrap();

    registry
        .grant_to_tenant("ten_alpha".into(), t1_visible.id.clone(), true)
        .unwrap();
    registry
        .grant_to_tenant("ten_alpha".into(), t3_hidden.id.clone(), false)
        .unwrap();
    registry
        .grant_to_tenant("ten_alpha".into(), t3_visible.id.clone(), true)
        .unwrap();

    assert!(registry.is_licensed_for_tenant("ten_alpha", &t3_hidden.id));

    let discovered = registry
        .discover_for_tenant("ten_alpha", AutonomyTier::T2Advisory)
        .unwrap();
    assert_eq!(
        discovered
            .iter()
            .map(|capability| capability.id.as_str())
            .collect::<Vec<_>>(),
        vec!["cap.demo.visible"]
    );
}

#[test]
fn capability_registry_rejects_duplicate_invalid_and_unknown_bindings() {
    let mut registry = CapabilityRegistry::default();
    let capability = capability("cap.demo.registry", AutonomyTier::T1ViewOnly);
    registry.publish(capability.clone()).unwrap();
    assert_eq!(
        registry.publish(capability.clone()),
        Err(CapabilityError::DuplicateCapability)
    );
    assert_eq!(
        registry.grant_to_tenant("tenant-alpha".into(), capability.id.clone(), true),
        Err(CapabilityError::InvalidTenantId)
    );
    assert_eq!(
        registry.grant_to_tenant("ten_alpha".into(), "cap.demo.missing".into(), true),
        Err(CapabilityError::CapabilityNotFound)
    );
}

#[test]
fn capability_rejects_operational_or_subject_markers_as_touched_privacy_classes() {
    for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert_eq!(
            Capability::try_from_legacy_data_classes(
                "cap.demo.non-privacy".into(),
                "demo".into(),
                AutonomyTier::T1ViewOnly,
                vec![data_class],
                "oya.foundry.capability.invoked".into(),
            ),
            Err(CapabilityError::NonPrivacyDataClass)
        );
    }
}

#[test]
fn capability_legacy_projection_is_derived_from_typed_privacy_classes() {
    let capability = capability("cap.demo.projection", AutonomyTier::T1ViewOnly);

    assert_eq!(
        capability.touched_privacy_data_classes(),
        [privacy_data_class(DataClass::InternalOnly)].as_slice()
    );
    assert_eq!(
        capability.legacy_touched_data_classes(),
        vec![DataClass::InternalOnly]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            capability.touched_data_classes(),
            capability.legacy_touched_data_classes()
        );
    }
}

#[test]
fn capability_cost_profile_declares_cost_ceiling_and_ordered_provider_preference() {
    let profile = CapabilityCostProfile::new(
        125,
        10_000,
        vec!["anthropic-api".into(), "openai-api".into()],
    )
    .expect("valid cost profile is accepted");
    let capability = Capability::new_with_cost_profile(
        "cap.demo.profiled".into(),
        "demo".into(),
        AutonomyTier::T2Advisory,
        vec![privacy_data_class(DataClass::InternalOnly)],
        "oya.foundry.capability.invoked".into(),
        profile.clone(),
    )
    .unwrap();

    assert_eq!(capability.cost_profile(), &profile);
    assert_eq!(
        capability.provider_preference(),
        ["anthropic-api", "openai-api"]
    );
    assert!(capability.allows_projected_invocation_cost(125));
    assert!(!capability.allows_projected_invocation_cost(126));
}

#[test]
fn capability_cost_profile_rejects_missing_or_malformed_provider_preference() {
    assert_eq!(
        CapabilityCostProfile::new(1, 10, vec![]),
        Err(CapabilityError::MissingProviderPreference)
    );
    assert_eq!(
        CapabilityCostProfile::new(0, 10, vec!["foundation-local".into()]),
        Err(CapabilityError::InvalidCostProfile)
    );
    assert_eq!(
        CapabilityCostProfile::new(1, 0, vec!["foundation-local".into()]),
        Err(CapabilityError::InvalidCostProfile)
    );
    assert_eq!(
        CapabilityCostProfile::new(1, 10, vec!["OpenAI".into()]),
        Err(CapabilityError::InvalidProviderPreference)
    );
}

#[test]
fn capability_mcp_contract_carries_authored_descriptions_and_schemas() {
    let contract = CapabilityMcpContract::new(
        "Agent: run readiness check with tenant-scoped evidence.".into(),
        "Human: readiness check for release operators.".into(),
        r#"{"type":"object","required":["release_id"]}"#.into(),
        r#"{"type":"object","required":["status"]}"#.into(),
    )
    .unwrap();
    let capability = Capability::new_with_mcp_contract(
        "cap.demo.mcp-contract".into(),
        "demo".into(),
        AutonomyTier::T1ViewOnly,
        vec![privacy_data_class(DataClass::InternalOnly)],
        "oya.foundry.capability.invoked".into(),
        contract.clone(),
    )
    .unwrap();

    assert_eq!(capability.mcp_contract(), &contract);
    assert_eq!(
        capability.mcp_contract().agent_readable_description.value,
        "Agent: run readiness check with tenant-scoped evidence."
    );
    assert!(
        capability
            .mcp_contract()
            .input_schema
            .value
            .contains("release_id")
    );
    assert!(
        capability
            .mcp_contract()
            .output_schema
            .value
            .contains("status")
    );
}

#[test]
fn capability_mcp_contract_rejects_empty_descriptions_and_non_object_schemas() {
    assert_eq!(
        CapabilityMcpContract::new(
            "".into(),
            "Human docs".into(),
            r#"{"type":"object"}"#.into(),
            r#"{"type":"object"}"#.into(),
        ),
        Err(CapabilityError::InvalidMcpContract)
    );
    assert_eq!(
        CapabilityMcpContract::new(
            "Agent docs".into(),
            "Human docs".into(),
            r#"[]"#.into(),
            r#"{"type":"object"}"#.into(),
        ),
        Err(CapabilityError::InvalidMcpContract)
    );
}

fn capability(id: &str, tier: AutonomyTier) -> Capability {
    Capability::new(
        id.into(),
        "demo".into(),
        tier,
        vec![privacy_data_class(DataClass::InternalOnly)],
        "oya.foundry.capability.invoked".into(),
    )
    .unwrap()
}

fn privacy_data_class(data_class: DataClass) -> PrivacyDataClass {
    PrivacyDataClass::try_from(data_class).expect("test fixture uses privacy data class")
}
