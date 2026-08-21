//! The residency decision matrix: every Deny variant reachable and justified,
//! every unknown-input path denied, the cell diagonal allowed, the conjunctive
//! cross-border requirements pinned, and the audit evidence pinned.
//!
//! Each case names the artifact it holds the engine to. If one of these fails
//! after a policy edit, the hand-written mirror in `src/domain.rs` has drifted
//! from `tenancy/policy/data-residency.{cedar,md}` — which is the failure mode
//! the crate's "Gaps" paragraph warns about.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use tenancy_data_residency_enforcer::domain::ResidencyRule;
use tenancy_data_residency_enforcer::inmemory::{
    InMemoryDenialAuditSink, InMemoryRegionCatalog, InMemoryTransferRegister, default_engine,
    default_engine_with_register, explain,
};
use tenancy_data_residency_enforcer::kernel::{
    CrossJurisdictionPermitEntry, NoTransferRegister, RegionRecord, RegionRole, ResidencyClass,
    ResidencyDataClass, ResidencyOperation, ResidencyOverlay, ResidencyRegionCatalog,
    ResidencyTransferRegister, SccRegisterEntry, TransferBasis,
};
use tenancy_data_residency_enforcer::usecase::ResidencyPolicyEngine;
use tenancy_data_residency_enforcer::{
    ResidencyAdapterError, ResidencyContext, ResidencyDecision, ResidencyDenialAuditSink,
    ResidencyPolicyEvaluator, enforce, enforce_detailed,
};

/// A route with the loosest residency class, so a denial in these cases is
/// never the residency class doing the work by accident.
fn global_route(source: &str, destination: &str, data_class: &str, op: &str) -> ResidencyContext {
    ResidencyContext::new("tenant:t1", source, destination, data_class, "global", op)
}

fn engine() -> ResidencyPolicyEngine<InMemoryRegionCatalog> {
    default_engine().expect("the documented roster must validate")
}

fn decide(ctx: &ResidencyContext) -> ResidencyDecision {
    engine().evaluate(ctx).expect("catalog answers")
}

fn rule(ctx: &ResidencyContext) -> ResidencyRule {
    explain(&engine(), ctx).expect("catalog answers")
}

/// The SCC assertion a caller fills in for the documented EU→US transfer.
fn eu_us_scc_assertion() -> TransferBasis {
    TransferBasis::StandardContractualClauses {
        register_ref: "transfer-register#7".to_owned(),
        transfer_purpose: "DR failover to pack-us".to_owned(),
        adequacy_or_safeguard: true,
        supplementary_measures: true,
    }
}

/// The matching register row: the same route, the same tenant, the same
/// purpose, with the two facts about the world that the register — not the
/// request — is entitled to assert.
fn eu_us_scc_row() -> SccRegisterEntry {
    SccRegisterEntry {
        register_ref: "transfer-register#7".to_owned(),
        tenant_id: "tenant:t1".to_owned(),
        source_jurisdiction: "EU".to_owned(),
        destination_jurisdiction: "US".to_owned(),
        transfer_purpose: "DR failover to pack-us".to_owned(),
        adequacy_or_safeguard: true,
        supplementary_measures: true,
    }
}

/// A four-conjunct migration permit assertion, as
/// `tenancy/cedar/policies.cedar` lines 40-49 require.
fn permit_assertion(permit_id: &str) -> TransferBasis {
    TransferBasis::CrossJurisdictionCedarPermit {
        permit_id: permit_id.to_owned(),
        cross_jurisdiction_permit_id: format!("xj-{permit_id}"),
        audit_chain_emit: true,
        microservice: "tenancy".to_owned(),
    }
}

fn permit_row(permit_id: &str, source: &str, destination: &str) -> CrossJurisdictionPermitEntry {
    CrossJurisdictionPermitEntry {
        permit_id: permit_id.to_owned(),
        tenant_id: "tenant:t1".to_owned(),
        source_jurisdiction: source.to_owned(),
        destination_jurisdiction: destination.to_owned(),
        audit_chain_emit: true,
    }
}

fn registered_engine(
    sccs: Vec<SccRegisterEntry>,
    permits: Vec<CrossJurisdictionPermitEntry>,
) -> ResidencyPolicyEngine<InMemoryRegionCatalog, InMemoryTransferRegister> {
    default_engine_with_register(InMemoryTransferRegister::new(sccs, permits))
        .expect("the documented roster must validate")
}

// ---------------------------------------------------------------- the diagonal

#[test]
fn same_cell_is_allowed_for_every_cell_in_the_roster() {
    let catalog = InMemoryRegionCatalog::oyatie_pack_roster();
    let engine = engine();
    let rows = catalog.regions().unwrap();
    assert_eq!(
        rows.len(),
        19,
        "roster size changed; re-read multi-region.md"
    );
    for record in &rows {
        let route = global_route(
            &record.region_id,
            &record.region_id,
            "INTERNAL_ONLY",
            "emit_event",
        )
        .with_pack(&record.pack_id);
        assert_eq!(
            engine.evaluate(&route).unwrap(),
            ResidencyDecision::Allow,
            "the diagonal must be allowed: {}/{}",
            record.region_id,
            record.pack_id
        );
        assert_eq!(explain(&engine, &route).unwrap(), ResidencyRule::SameRegion);
    }
}

#[test]
fn the_diagonal_does_not_rescue_an_unknown_region() {
    // Same source and destination, but the region is not in the catalog. A
    // control that shortcuts to Allow on equality would pass this route.
    let route = global_route(
        "mars-olympus-1",
        "mars-olympus-1",
        "INTERNAL_ONLY",
        "emit_event",
    );
    assert_eq!(decide(&route), ResidencyDecision::DenyResidency);
    assert_eq!(rule(&route), ResidencyRule::UnknownRegion);
}

#[test]
fn a_region_hosting_two_packs_must_say_which_cell_it_means() {
    // `data-residency.md` puts pack-us and pack-us-healthcare on the same OCI
    // regions with different jurisdiction codes. Resolving `us-ashburn-1` to
    // whichever row came last is how PHI silently acquires the pack-us rules.
    let unqualified = global_route("us-ashburn-1", "us-ashburn-1", "AUDIT", "emit_event");
    assert_eq!(decide(&unqualified), ResidencyDecision::DenyResidency);
    assert_eq!(
        rule(&unqualified),
        ResidencyRule::AmbiguousRegionRequiresPack
    );

    let qualified = unqualified.clone().with_pack("pack-us-healthcare");
    assert_eq!(decide(&qualified), ResidencyDecision::Allow);
}

// -------------------------------------------------------- unknown-input paths

#[test]
fn an_unknown_data_class_denies() {
    let route = global_route(
        "ap-seoul-1",
        "ap-seoul-1",
        "TOTALLY_FINE_HONEST",
        "emit_event",
    );
    assert_eq!(decide(&route), ResidencyDecision::DenyDataClass);
    assert_eq!(rule(&route), ResidencyRule::UnknownDataClass);
}

#[test]
fn data_class_parsing_is_case_sensitive_and_a_miscased_label_denies() {
    let route = global_route("ap-seoul-1", "ap-seoul-1", "internal_only", "emit_event");
    assert_eq!(decide(&route), ResidencyDecision::DenyDataClass);
}

#[test]
fn an_unknown_residency_class_denies() {
    let route = ResidencyContext::new(
        "tenant:t1",
        "ap-seoul-1",
        "ap-seoul-1",
        "INTERNAL_ONLY",
        "whatever_is_convenient",
        "emit_event",
    );
    assert_eq!(decide(&route), ResidencyDecision::DenyResidency);
    assert_eq!(rule(&route), ResidencyRule::UnknownResidencyClass);
}

#[test]
fn an_unknown_operation_denies() {
    let route = global_route("ap-seoul-1", "ap-seoul-1", "INTERNAL_ONLY", "exfiltrate");
    assert_eq!(decide(&route), ResidencyDecision::DenyResidency);
    assert_eq!(rule(&route), ResidencyRule::UnknownOperation);
}

#[test]
fn an_unknown_residency_overlay_denies() {
    let route = global_route("ap-seoul-1", "ap-seoul-1", "INTERNAL_ONLY", "emit_event")
        .with_overlays(["xx-newlaw"]);
    assert_eq!(decide(&route), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(rule(&route), ResidencyRule::UnknownResidencyOverlay);
}

#[test]
fn an_unknown_destination_region_denies_even_from_a_known_source() {
    let route = global_route("ap-seoul-1", "ap-atlantis-1", "INTERNAL_ONLY", "emit_event");
    assert_eq!(decide(&route), ResidencyDecision::DenyResidency);
    assert_eq!(rule(&route), ResidencyRule::UnknownRegion);
}

#[test]
fn a_pack_the_region_does_not_host_denies_as_an_unknown_cell() {
    let route = global_route("ap-seoul-1", "ap-seoul-1", "INTERNAL_ONLY", "emit_event")
        .with_pack("pack-jp");
    assert_eq!(decide(&route), ResidencyDecision::DenyResidency);
    assert_eq!(rule(&route), ResidencyRule::UnknownRegion);
}

// ---------------------------------------------------- compliance-pack overlays

#[test]
fn kr_csap_tenant_cannot_be_processed_outside_the_kr_processing_region() {
    let offshore = global_route("ap-seoul-1", "ap-tokyo-1", "INTERNAL_ONLY", "emit_event")
        .with_overlays(["kr-csap"]);
    assert_eq!(decide(&offshore), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(rule(&offshore), ResidencyRule::OverlayKrCsapOffshore);
}

#[test]
fn kr_csap_is_read_literally_like_its_sibling_overlays_and_denies_in_ap_seoul() {
    // `data-residency.cedar` rule 2 compares `processing_region != "kr"` —
    // a region literal, exactly like rule 4's `!= "cn-onshore"`. No roster
    // region is named `kr`, so the literal rule refuses even the KR tenant's
    // home region, in the same shape as the eu-sovereign contradiction. Reading
    // this one as a jurisdiction code (`destination.jurisdiction == "KR"`)
    // would be inventing the permissive reading for the one arm that then
    // produces an Allow.
    let home = global_route("ap-seoul-1", "ap-seoul-1", "INTERNAL_ONLY", "emit_event")
        .with_overlays(["kr-csap"]);
    assert_eq!(decide(&home), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(rule(&home), ResidencyRule::OverlayKrCsapOffshore);

    // With a region literally named `kr` registered, the same overlay permits —
    // the rule is satisfiable, the documented roster just does not satisfy it.
    let mut rows = InMemoryRegionCatalog::oyatie_pack_roster()
        .regions()
        .unwrap();
    rows.push(RegionRecord::new(
        "kr",
        "pack-kr",
        "KR",
        RegionRole::Primary,
    ));
    let engine = ResidencyPolicyEngine::try_new(InMemoryRegionCatalog::new(rows)).unwrap();
    let onshore =
        global_route("kr", "kr", "INTERNAL_ONLY", "emit_event").with_overlays(["kr-csap"]);
    assert_eq!(engine.evaluate(&onshore).unwrap(), ResidencyDecision::Allow);
}

#[test]
fn cn_pipl_tenant_denies_everywhere_but_cn_onshore() {
    let route = global_route("ap-seoul-1", "ap-seoul-1", "INTERNAL_ONLY", "emit_event")
        .with_overlays(["cn-pipl"]);
    assert_eq!(decide(&route), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(rule(&route), ResidencyRule::OverlayCnPiplOffshore);

    let mut rows = InMemoryRegionCatalog::oyatie_pack_roster()
        .regions()
        .unwrap();
    rows.push(RegionRecord::new(
        "cn-onshore",
        "pack-cn",
        "CN",
        RegionRole::Primary,
    ));
    let engine = ResidencyPolicyEngine::try_new(InMemoryRegionCatalog::new(rows)).unwrap();
    let onshore = global_route("cn-onshore", "cn-onshore", "INTERNAL_ONLY", "emit_event")
        .with_overlays(["cn-pipl"]);
    assert_eq!(engine.evaluate(&onshore).unwrap(), ResidencyDecision::Allow);
}

#[test]
fn eu_sovereign_overlay_denies_across_the_whole_documented_roster() {
    // This is the recorded contradiction, not a bug: the Cedar fragment demands
    // an `eu-sovereign-*` processing region and no roster region has that
    // prefix, so the literal rule refuses even the EU tenant's home region.
    let home = global_route(
        "eu-frankfurt-1",
        "eu-frankfurt-1",
        "INTERNAL_ONLY",
        "emit_event",
    )
    .with_overlays(["eu-sovereign"]);
    assert_eq!(decide(&home), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(
        rule(&home),
        ResidencyRule::OverlayEuSovereignNonSovereignRegion
    );
}

#[test]
fn the_overlay_forbid_beats_the_same_region_allow() {
    // Cedar `forbid` is unconditional. If the same-region allow ran first this
    // route would be permitted, which is exactly the ordering bug this pins.
    let route = global_route("ap-tokyo-1", "ap-tokyo-1", "INTERNAL_ONLY", "emit_event")
        .with_overlays(["cn-pipl"]);
    assert_eq!(decide(&route), ResidencyDecision::DenyJurisdictionPack);
}

// ------------------------------------------------------------- residency class

#[test]
fn a_strict_tenant_cannot_use_even_its_own_packs_dr_region() {
    for class in ["strict_home_region", "strict_federated_region"] {
        let route = ResidencyContext::new(
            "tenant:t1",
            "us-ashburn-1",
            "us-phoenix-1",
            "INTERNAL_ONLY",
            class,
            "replicate_storage",
        )
        .with_pack("pack-us");
        assert_eq!(
            decide(&route),
            ResidencyDecision::DenyResidency,
            "class {class} must forbid every cross-region route"
        );
        assert_eq!(
            rule(&route),
            ResidencyRule::StrictResidencyForbidsCrossRegion
        );
    }
}

#[test]
fn recovery_failover_reaches_its_dr_pair_for_recovery_work_only() {
    let recovery = ResidencyContext::new(
        "tenant:t1",
        "eu-frankfurt-1",
        "eu-amsterdam-1",
        "INTERNAL_ONLY",
        "home_with_recovery_failover",
        "promote_dr",
    );
    assert_eq!(decide(&recovery), ResidencyDecision::Allow);
    assert_eq!(rule(&recovery), ResidencyRule::IntraPackDrPairTransfer);

    let ordinary = ResidencyContext::new(
        "tenant:t1",
        "eu-frankfurt-1",
        "eu-amsterdam-1",
        "INTERNAL_ONLY",
        "home_with_recovery_failover",
        "rpc_call",
    );
    assert_eq!(decide(&ordinary), ResidencyDecision::DenyResidency);
    assert_eq!(
        rule(&ordinary),
        ResidencyRule::RecoveryFailoverRequiresIntraPackDrPair
    );
}

#[test]
fn recovery_failover_cannot_reach_another_packs_primary() {
    let route = ResidencyContext::new(
        "tenant:t1",
        "ap-sydney-1",
        "ap-hyderabad-1",
        "INTERNAL_ONLY",
        "home_with_recovery_failover",
        "promote_dr",
    );
    assert_eq!(decide(&route), ResidencyDecision::DenyResidency);
    assert_eq!(
        rule(&route),
        ResidencyRule::RecoveryFailoverRequiresIntraPackDrPair
    );
}

#[test]
fn recovery_failover_may_run_the_documented_failback_direction() {
    // `multi-region.md` §Failback: failback is a real, scheduled procedure that
    // "mirrors DR Failover steps in reverse", and the Replication table
    // constrains replication to "intra-pack only" with no direction constraint.
    // Refusing DR-pair -> primary made the residency class that exists to
    // support DR the only class unable to finish a DR cycle, while `global` was
    // permitted the identical route.
    let failback = ResidencyContext::new(
        "tenant:t1",
        "eu-amsterdam-1",
        "eu-frankfurt-1",
        "INTERNAL_ONLY",
        "home_with_recovery_failover",
        "promote_dr",
    );
    assert_eq!(decide(&failback), ResidencyDecision::Allow);
    assert_eq!(rule(&failback), ResidencyRule::IntraPackDrPairTransfer);

    // The direction is free; the OPERATION is not. Failback does not open the
    // DR pair to ordinary traffic.
    let ordinary_failback = ResidencyContext::new(
        "tenant:t1",
        "eu-amsterdam-1",
        "eu-frankfurt-1",
        "INTERNAL_ONLY",
        "home_with_recovery_failover",
        "rpc_call",
    );
    assert_eq!(decide(&ordinary_failback), ResidencyDecision::DenyResidency);

    // And it is still intra-pack only.
    let cross_pack = ResidencyContext::new(
        "tenant:t1",
        "ap-melbourne-1",
        "ap-hyderabad-1",
        "INTERNAL_ONLY",
        "home_with_recovery_failover",
        "promote_dr",
    );
    assert_eq!(decide(&cross_pack), ResidencyDecision::DenyResidency);
}

// ------------------------------------------------------------------ data class

#[test]
fn pipa_sensitive_data_never_leaves_its_jurisdiction() {
    let route = global_route(
        "ap-seoul-1",
        "ap-tokyo-1",
        "SENSITIVE_PIPA_ART23",
        "emit_event",
    );
    assert_eq!(decide(&route), ResidencyDecision::DenyDataClass);
    assert_eq!(rule(&route), ResidencyRule::SensitiveDataCrossJurisdiction);
}

#[test]
fn pipa_sensitive_data_denies_even_with_a_registered_scc_basis() {
    // Higher-restriction-wins: an SCC is a GDPR transfer mechanism and does not
    // buy a PIPA Art. 23-2 exemption, even when the register confirms it.
    let engine = registered_engine(
        vec![SccRegisterEntry {
            register_ref: "transfer-register#7".to_owned(),
            tenant_id: "tenant:t1".to_owned(),
            source_jurisdiction: "KR".to_owned(),
            destination_jurisdiction: "JP".to_owned(),
            transfer_purpose: "DR failover".to_owned(),
            adequacy_or_safeguard: true,
            supplementary_measures: true,
        }],
        Vec::new(),
    );
    let route = global_route(
        "ap-seoul-1",
        "ap-tokyo-1",
        "SENSITIVE_PIPA_ART23",
        "emit_event",
    )
    .with_transfer_basis(TransferBasis::StandardContractualClauses {
        register_ref: "transfer-register#7".to_owned(),
        transfer_purpose: "DR failover".to_owned(),
        adequacy_or_safeguard: true,
        supplementary_measures: true,
    });
    assert_eq!(
        engine.evaluate(&route).unwrap(),
        ResidencyDecision::DenyDataClass
    );
}

#[test]
fn key_material_does_not_leave_its_pack_but_may_move_inside_it() {
    let cross_pack = global_route(
        "ap-sydney-1",
        "ap-hyderabad-1",
        "SECRET",
        "replicate_storage",
    );
    assert_eq!(decide(&cross_pack), ResidencyDecision::DenyDataClass);
    assert_eq!(rule(&cross_pack), ResidencyRule::SecretCrossPack);

    let intra_pack = ResidencyContext::new(
        "tenant:t1",
        "ap-sydney-1",
        "ap-melbourne-1",
        "SECRET",
        "home_with_recovery_failover",
        "replicate_storage",
    );
    assert_eq!(decide(&intra_pack), ResidencyDecision::Allow);
}

// ------------------------------------------------------------------------ DSR

#[test]
fn dsr_receipt_aggregation_is_a_permitted_intra_pack_route() {
    // IP-020 §D.4 names DSR receipt aggregation as a permitted route, and
    // `data-residency.md` §DSR Cascade steps 4-5 make the tenancy worker the
    // aggregator. Inside the pack it is allowed under its own rule, so an
    // auditor can count DSR fan-in separately from ordinary traffic.
    let global_dsr = ResidencyContext::new(
        "tenant:t1",
        "ap-sydney-1",
        "ap-melbourne-1",
        "AUDIT",
        "global",
        "aggregate_dsr_receipt",
    );
    assert_eq!(decide(&global_dsr), ResidencyDecision::Allow);
    assert_eq!(
        rule(&global_dsr),
        ResidencyRule::IntraPackDsrReceiptAggregation
    );

    // A `home_with_recovery_failover` tenant may aggregate onto its own DR pair
    // too: refusing it would leave the DSR cascade unable to run for exactly
    // the tenants that have a DR pair.
    let recovery_dsr = ResidencyContext::new(
        "tenant:t1",
        "ap-sydney-1",
        "ap-melbourne-1",
        "AUDIT",
        "home_with_recovery_failover",
        "aggregate_dsr_receipt",
    );
    assert_eq!(decide(&recovery_dsr), ResidencyDecision::Allow);
    assert_eq!(
        rule(&recovery_dsr),
        ResidencyRule::IntraPackDsrReceiptAggregation
    );
}

#[test]
fn dsr_receipt_aggregation_across_packs_denies_under_its_own_rule() {
    // No sentence in the corpus authorises a cross-pack receipt fan-in:
    // `data-residency.md` gives every pack its own audit-chain instance and
    // forbids cross-pack replication by default. It denies — but as a DSR
    // denial, not as anonymous cross-pack traffic, so the audit trail shows
    // which control blocked the cascade.
    let route = global_route(
        "ap-sydney-1",
        "ap-singapore-1",
        "AUDIT",
        "aggregate_dsr_receipt",
    );
    assert_eq!(decide(&route), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(rule(&route), ResidencyRule::DsrAggregationRequiresIntraPack);
    assert_ne!(
        rule(&route),
        rule(&global_route(
            "ap-sydney-1",
            "ap-singapore-1",
            "AUDIT",
            "emit_event"
        )),
        "a blocked DSR fan-in must not be byte-identical to a blocked event"
    );
}

// ------------------------------------------------------------ transfer bases

#[test]
fn eu_cross_jurisdiction_transfer_needs_an_scc_the_register_confirms() {
    let bare = global_route(
        "eu-frankfurt-1",
        "us-ashburn-1",
        "PII_IDENTIFYING",
        "rpc_call",
    )
    .with_destination_pack("pack-us");
    assert_eq!(decide(&bare), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(rule(&bare), ResidencyRule::EuTransferRequiresScc);

    // A completely filled-in assertion is still only an assertion: with no
    // register behind the engine there is nothing to check it against, and two
    // booleans the caller set itself are not an adequacy decision.
    let asserted = bare.clone().with_transfer_basis(eu_us_scc_assertion());
    assert_eq!(decide(&asserted), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(rule(&asserted), ResidencyRule::EuTransferRequiresScc);

    // With the row on file, the transfer is authorised — and says so under a
    // rule code distinct from the refusal.
    let engine = registered_engine(vec![eu_us_scc_row()], Vec::new());
    assert_eq!(
        engine.evaluate(&asserted).unwrap(),
        ResidencyDecision::Allow
    );
    assert_eq!(
        explain(&engine, &asserted).unwrap(),
        ResidencyRule::EuTransferAuthorisedByScc
    );
}

#[test]
fn a_register_row_is_not_a_bearer_token() {
    let route = global_route(
        "eu-frankfurt-1",
        "us-ashburn-1",
        "PII_IDENTIFYING",
        "rpc_call",
    )
    .with_destination_pack("pack-us")
    .with_transfer_basis(eu_us_scc_assertion());

    // Right row, wrong tenant.
    let other_tenant = registered_engine(
        vec![SccRegisterEntry {
            tenant_id: "tenant:someone-else".to_owned(),
            ..eu_us_scc_row()
        }],
        Vec::new(),
    );
    assert_eq!(
        other_tenant.evaluate(&route).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );

    // Right tenant, wrong destination jurisdiction.
    let other_route = registered_engine(
        vec![SccRegisterEntry {
            destination_jurisdiction: "JP".to_owned(),
            ..eu_us_scc_row()
        }],
        Vec::new(),
    );
    assert_eq!(
        other_route.evaluate(&route).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );

    // Right route, but the register does not record the safeguards. These are
    // facts about the receiving jurisdiction and the deployment; the request
    // claiming them does not make them true.
    for row in [
        SccRegisterEntry {
            adequacy_or_safeguard: false,
            ..eu_us_scc_row()
        },
        SccRegisterEntry {
            supplementary_measures: false,
            ..eu_us_scc_row()
        },
        SccRegisterEntry {
            transfer_purpose: "something else entirely".to_owned(),
            ..eu_us_scc_row()
        },
    ] {
        let engine = registered_engine(vec![row], Vec::new());
        assert_eq!(
            engine.evaluate(&route).unwrap(),
            ResidencyDecision::DenyJurisdictionPack,
            "a register row that does not cover this transfer must not authorise it"
        );
    }
}

#[test]
fn a_half_filled_scc_is_not_a_transfer_basis() {
    let base = global_route(
        "eu-frankfurt-1",
        "us-ashburn-1",
        "PII_IDENTIFYING",
        "rpc_call",
    )
    .with_destination_pack("pack-us");
    let engine = registered_engine(vec![eu_us_scc_row()], Vec::new());
    let incomplete = [
        TransferBasis::StandardContractualClauses {
            register_ref: String::new(),
            transfer_purpose: "DR failover to pack-us".to_owned(),
            adequacy_or_safeguard: true,
            supplementary_measures: true,
        },
        TransferBasis::StandardContractualClauses {
            register_ref: "transfer-register#7".to_owned(),
            transfer_purpose: "   ".to_owned(),
            adequacy_or_safeguard: true,
            supplementary_measures: true,
        },
        TransferBasis::StandardContractualClauses {
            register_ref: "transfer-register#7".to_owned(),
            transfer_purpose: "DR failover to pack-us".to_owned(),
            adequacy_or_safeguard: false,
            supplementary_measures: true,
        },
        TransferBasis::StandardContractualClauses {
            register_ref: "transfer-register#7".to_owned(),
            transfer_purpose: "DR failover to pack-us".to_owned(),
            adequacy_or_safeguard: true,
            supplementary_measures: false,
        },
    ];
    for basis in incomplete {
        let route = base.clone().with_transfer_basis(basis);
        assert_eq!(
            engine.evaluate(&route).unwrap(),
            ResidencyDecision::DenyJurisdictionPack,
            "an incomplete SCC must not authorise a transfer"
        );
    }
}

#[test]
fn an_scc_does_not_authorise_a_non_eu_source() {
    // The documented exception is for EU-resident data. A tenant in pack-au
    // waving an SCC — even a registered one — is not covered by it.
    let engine = registered_engine(
        vec![SccRegisterEntry {
            register_ref: "transfer-register#9".to_owned(),
            tenant_id: "tenant:t1".to_owned(),
            source_jurisdiction: "AU".to_owned(),
            destination_jurisdiction: "SG".to_owned(),
            transfer_purpose: "analytics".to_owned(),
            adequacy_or_safeguard: true,
            supplementary_measures: true,
        }],
        Vec::new(),
    );
    let route = global_route("ap-sydney-1", "ap-singapore-1", "PII_QUASI", "rpc_call")
        .with_transfer_basis(TransferBasis::StandardContractualClauses {
            register_ref: "transfer-register#9".to_owned(),
            transfer_purpose: "analytics".to_owned(),
            adequacy_or_safeguard: true,
            supplementary_measures: true,
        });
    assert_eq!(
        engine.evaluate(&route).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );
    assert_eq!(
        explain(&engine, &route).unwrap(),
        ResidencyRule::CrossJurisdictionForbiddenByDefault
    );
}

#[test]
fn cross_jurisdiction_migration_needs_every_conjunct_the_cedar_rule_names() {
    let bare = global_route(
        "ap-sydney-1",
        "ap-singapore-1",
        "BEHAVIORAL_TENANT_PRODUCT",
        "migrate_tenant_cross_jurisdiction",
    );
    assert_eq!(decide(&bare), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(
        rule(&bare),
        ResidencyRule::CrossJurisdictionMigrationRequiresPermit
    );

    let engine = registered_engine(
        Vec::new(),
        vec![permit_row("permit-2026-08-20-001", "AU", "SG")],
    );

    // `tenancy/cedar/policies.cedar` forbids the migration unless the resource
    // is the tenancy µservice AND both permit ids are non-empty AND audit-chain
    // emission is on. Dropping any single conjunct must not authorise it.
    let broken = [
        TransferBasis::CrossJurisdictionCedarPermit {
            permit_id: "  ".to_owned(),
            cross_jurisdiction_permit_id: "xj-1".to_owned(),
            audit_chain_emit: true,
            microservice: "tenancy".to_owned(),
        },
        TransferBasis::CrossJurisdictionCedarPermit {
            permit_id: "permit-2026-08-20-001".to_owned(),
            cross_jurisdiction_permit_id: String::new(),
            audit_chain_emit: true,
            microservice: "tenancy".to_owned(),
        },
        TransferBasis::CrossJurisdictionCedarPermit {
            permit_id: "permit-2026-08-20-001".to_owned(),
            cross_jurisdiction_permit_id: "xj-1".to_owned(),
            audit_chain_emit: false,
            microservice: "tenancy".to_owned(),
        },
        TransferBasis::CrossJurisdictionCedarPermit {
            permit_id: "permit-2026-08-20-001".to_owned(),
            cross_jurisdiction_permit_id: "xj-1".to_owned(),
            audit_chain_emit: true,
            microservice: "some-other-service".to_owned(),
        },
    ];
    for basis in broken {
        let route = bare.clone().with_transfer_basis(basis);
        assert_eq!(
            engine.evaluate(&route).unwrap(),
            ResidencyDecision::DenyJurisdictionPack,
            "a permit missing a Cedar conjunct must not authorise a migration"
        );
    }

    // A complete assertion still needs to be an ISSUED permit. Without a
    // register there is nothing but the caller's own string.
    let complete = bare
        .clone()
        .with_transfer_basis(permit_assertion("permit-2026-08-20-001"));
    assert_eq!(decide(&complete), ResidencyDecision::DenyJurisdictionPack);

    // A registered permit for a different route does not travel.
    let wrong_route = registered_engine(
        Vec::new(),
        vec![permit_row("permit-2026-08-20-001", "AU", "JP")],
    );
    assert_eq!(
        wrong_route.evaluate(&complete).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );

    assert_eq!(
        engine.evaluate(&complete).unwrap(),
        ResidencyDecision::Allow
    );
    assert_eq!(
        explain(&engine, &complete).unwrap(),
        ResidencyRule::CrossJurisdictionMigrationAuthorised,
        "an authorised migration must be a different audit row from a refused one"
    );
}

#[test]
fn an_eu_sourced_migration_owes_the_scc_as_well_as_the_permit() {
    // `multi-region.md`: "EU-resident tenant metadata never reaches a non-EU
    // region without a Schrems-II-compatible SCC + supplementary measures on
    // file." Unconditional — so the migration branch does not get to skip it.
    // Reading the two rules as alternatives let EU data leave the EU on a bare
    // permit id, while the identical route with `rpc_call` denied.
    let route = global_route(
        "eu-frankfurt-1",
        "us-ashburn-1",
        "PII_IDENTIFYING",
        "migrate_tenant_cross_jurisdiction",
    )
    .with_destination_pack("pack-us");
    let engine = registered_engine(vec![eu_us_scc_row()], vec![permit_row("p-1", "EU", "US")]);

    let permit_only = route.clone().with_transfer_basis(permit_assertion("p-1"));
    assert_eq!(
        engine.evaluate(&permit_only).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );
    assert_eq!(
        explain(&engine, &permit_only).unwrap(),
        ResidencyRule::EuTransferRequiresScc
    );

    let scc_only = route.clone().with_transfer_basis(eu_us_scc_assertion());
    assert_eq!(
        engine.evaluate(&scc_only).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );
    assert_eq!(
        explain(&engine, &scc_only).unwrap(),
        ResidencyRule::CrossJurisdictionMigrationRequiresPermit
    );

    let both = route
        .with_transfer_basis(eu_us_scc_assertion())
        .with_additional_transfer_bases([permit_assertion("p-1")]);
    assert_eq!(engine.evaluate(&both).unwrap(), ResidencyDecision::Allow);
    assert_eq!(
        explain(&engine, &both).unwrap(),
        ResidencyRule::CrossJurisdictionMigrationAuthorised
    );
}

#[test]
fn healthcare_data_does_not_leave_its_jurisdiction_under_any_basis() {
    let engine = registered_engine(Vec::new(), vec![permit_row("permit-1", "US-HC", "EU")]);
    let route = global_route(
        "us-ashburn-1",
        "eu-frankfurt-1",
        "AUDIT",
        "replicate_storage",
    )
    .with_tenant_pack("pack-us-healthcare")
    .with_destination_pack("pack-eu")
    .with_transfer_basis(permit_assertion("permit-1"));
    assert_eq!(
        engine.evaluate(&route).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );
    assert_eq!(
        explain(&engine, &route).unwrap(),
        ResidencyRule::HealthcarePackCrossJurisdictionUnauthorised
    );
}

#[test]
fn the_healthcare_dr_pair_is_the_one_route_that_pack_may_take() {
    let route = ResidencyContext::new(
        "tenant:t1",
        "us-ashburn-1",
        "us-phoenix-1",
        "AUDIT",
        "home_with_recovery_failover",
        "promote_dr",
    )
    .with_pack("pack-us-healthcare");
    assert_eq!(decide(&route), ResidencyDecision::Allow);
    assert_eq!(rule(&route), ResidencyRule::IntraPackDrPairTransfer);
}

#[test]
fn phi_does_not_replicate_onto_the_non_healthcare_cluster() {
    // The topology an operator can actually configure: `pack-us` and
    // `pack-us-healthcare` on the SAME OCI regions, distinguished by
    // jurisdiction code. A healthcare cell replicating onto the non-HC cell is
    // the leak the `US-HC` rule exists to stop, and it must be caught on the
    // real region ids rather than on invented `-hc` ones.
    let route = global_route("us-ashburn-1", "us-phoenix-1", "AUDIT", "replicate_storage")
        .with_tenant_pack("pack-us-healthcare")
        .with_destination_pack("pack-us");
    assert_eq!(decide(&route), ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(
        rule(&route),
        ResidencyRule::HealthcarePackCrossJurisdictionUnauthorised
    );

    // Same region, different cluster, is a crossing too.
    let same_region = global_route("us-ashburn-1", "us-ashburn-1", "AUDIT", "replicate_storage")
        .with_tenant_pack("pack-us-healthcare")
        .with_destination_pack("pack-us");
    assert_eq!(
        decide(&same_region),
        ResidencyDecision::DenyJurisdictionPack
    );
    assert_eq!(
        rule(&same_region),
        ResidencyRule::HealthcarePackCrossJurisdictionUnauthorised
    );
}

#[test]
fn two_packs_in_one_jurisdiction_stay_isolated() {
    // Reachability proof for the intra-jurisdiction pack rule, which the
    // documented roster alone cannot exercise because every roster pack has its
    // own jurisdiction code.
    let engine = ResidencyPolicyEngine::try_new(InMemoryRegionCatalog::new(vec![
        RegionRecord::new("us-a-1", "pack-us", "US", RegionRole::Primary),
        RegionRecord::new("us-b-1", "pack-us-analytics", "US", RegionRole::Primary),
    ]))
    .unwrap();
    let route = global_route("us-a-1", "us-b-1", "AUDIT", "replicate_storage");
    assert_eq!(
        engine.evaluate(&route).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );
    assert_eq!(
        engine.evaluate_detailed(&route).unwrap().rule,
        ResidencyRule::CrossPackWithinJurisdictionForbidden
    );
}

// ------------------------------------------------------------- allow labelling

#[test]
fn ordinary_intra_pack_traffic_is_not_labelled_as_dr_work() {
    // The DR-failover citation must not be attached to every same-pack route,
    // or an operator auditing "allows granted under DR authority" counts all
    // ordinary in-pack traffic.
    let mut rows = InMemoryRegionCatalog::oyatie_pack_roster()
        .regions()
        .unwrap();
    rows.push(RegionRecord::new(
        "us-reno-1",
        "pack-us",
        "US",
        RegionRole::Primary,
    ));
    let engine = ResidencyPolicyEngine::try_new(InMemoryRegionCatalog::new(rows)).unwrap();
    let primary_to_primary =
        global_route("us-ashburn-1", "us-reno-1", "AUDIT", "emit_event").with_pack("pack-us");
    assert_eq!(
        engine.evaluate(&primary_to_primary).unwrap(),
        ResidencyDecision::Allow
    );
    assert_eq!(
        explain(&engine, &primary_to_primary).unwrap(),
        ResidencyRule::IntraPackTransfer
    );

    // A DR-pair endpoint doing recovery work still earns the DR label.
    let dr = ResidencyContext::new(
        "tenant:t1",
        "us-ashburn-1",
        "us-phoenix-1",
        "AUDIT",
        "global",
        "replicate_storage",
    )
    .with_pack("pack-us");
    assert_eq!(
        explain(&engine, &dr).unwrap(),
        ResidencyRule::IntraPackDrPairTransfer
    );
}

// ------------------------------------------------------- evidence + error paths

#[test]
fn every_denial_in_the_matrix_produces_exactly_one_audit_record_naming_its_rule() {
    let engine = engine();
    let cases = [
        (
            global_route(
                "ap-seoul-1",
                "ap-tokyo-1",
                "SENSITIVE_PIPA_ART23",
                "emit_event",
            ),
            ResidencyRule::SensitiveDataCrossJurisdiction,
        ),
        (
            global_route(
                "ap-sydney-1",
                "ap-hyderabad-1",
                "SECRET",
                "replicate_storage",
            ),
            ResidencyRule::SecretCrossPack,
        ),
        (
            global_route(
                "eu-frankfurt-1",
                "us-ashburn-1",
                "PII_IDENTIFYING",
                "rpc_call",
            )
            .with_destination_pack("pack-us"),
            ResidencyRule::EuTransferRequiresScc,
        ),
        (
            global_route("ap-seoul-1", "mars-olympus-1", "AUDIT", "emit_event"),
            ResidencyRule::UnknownRegion,
        ),
        (
            global_route("ap-seoul-1", "ap-seoul-1", "NOT_A_CLASS", "emit_event"),
            ResidencyRule::UnknownDataClass,
        ),
        (
            global_route("ap-seoul-1", "ap-tokyo-1", "AUDIT", "emit_event")
                .with_overlays(["kr-csap"]),
            ResidencyRule::OverlayKrCsapOffshore,
        ),
        (
            global_route("ap-seoul-1", "ap-tokyo-1", "AUDIT", "emit_event")
                .with_overlays(["kr-csapp"]),
            ResidencyRule::UnknownResidencyOverlay,
        ),
        (
            ResidencyContext::new(
                "tenant:t1",
                "us-ashburn-1",
                "us-phoenix-1",
                "AUDIT",
                "strict_home_region",
                "replicate_storage",
            )
            .with_pack("pack-us"),
            ResidencyRule::StrictResidencyForbidsCrossRegion,
        ),
    ];
    for (index, (route, expected_rule)) in cases.iter().enumerate() {
        let sink = InMemoryDenialAuditSink::new();
        let decision = enforce(&engine, &sink, route).unwrap();
        assert_ne!(
            decision,
            ResidencyDecision::Allow,
            "route {index} must deny"
        );
        assert_eq!(
            sink.denial_count(),
            1,
            "route {index} must leave exactly one audit record"
        );
        let record = &sink.denials()[0];
        assert_eq!(record.decision, decision);
        assert_eq!(&record.context, route);
        assert_eq!(
            record.rule, *expected_rule,
            "route {index} must record WHICH rule refused it"
        );
    }
}

#[test]
fn a_caller_defect_and_a_compliance_block_are_distinguishable_audit_rows() {
    // Both are `DenyJurisdictionPack`. One should page the calling team, the
    // other is a GDPR event; the decision alone cannot tell them apart.
    let engine = engine();
    let typo = global_route("ap-seoul-1", "ap-tokyo-1", "AUDIT", "emit_event")
        .with_overlays(["kr-csapp"])
        .with_correlation_id("corr-typo");
    let gdpr = global_route(
        "eu-frankfurt-1",
        "us-ashburn-1",
        "PII_IDENTIFYING",
        "rpc_call",
    )
    .with_destination_pack("pack-us")
    .with_correlation_id("corr-gdpr");

    let sink = InMemoryDenialAuditSink::new();
    enforce(&engine, &sink, &typo).unwrap();
    enforce(&engine, &sink, &gdpr).unwrap();
    let records = sink.denials();
    assert_eq!(records[0].decision, records[1].decision);
    assert_ne!(records[0].rule, records[1].rule);
    assert_eq!(records[0].rule.code(), "unknown-residency-overlay");
    assert_eq!(records[1].rule.code(), "eu-transfer-requires-scc");
    assert_eq!(
        records[0].context.audit_correlation_id.as_deref(),
        Some("corr-typo")
    );
    assert!(records[0].sequence < records[1].sequence);
}

#[test]
fn the_audit_sink_is_bounded_and_refuses_rather_than_growing_without_limit() {
    // A caller retrying a blocked cross-border replication in a backoff loop
    // must not be able to grow the audit buffer until the process dies.
    let engine = engine();
    let sink = InMemoryDenialAuditSink::with_capacity(2);
    let route = global_route("ap-seoul-1", "ap-tokyo-1", "AUDIT", "emit_event");
    for _ in 0..2 {
        enforce(&engine, &sink, &route).unwrap();
    }
    assert!(sink.is_full());
    let overflow = enforce(&engine, &sink, &route);
    assert_eq!(overflow, Err(ResidencyAdapterError::AuditSinkUnavailable));
    assert!(!tenancy_data_residency_enforcer::dispatch_permitted(
        &overflow
    ));
    assert_eq!(
        sink.denial_count(),
        2,
        "the flood must not evict the evidence already held"
    );

    let drained = sink.drain_denials();
    assert_eq!(drained.len(), 2);
    assert_eq!(sink.denial_count(), 0);
    enforce(&engine, &sink, &route).unwrap();
    assert_eq!(sink.denial_count(), 1);
}

#[test]
fn the_seal_log_is_bounded_too_and_an_unsealable_transfer_is_refused() {
    // The allow path must not be an unbounded buffer either: a caller looping
    // an authorised intra-pack replication would grow it just as fast.
    let engine = engine();
    let sink = InMemoryDenialAuditSink::with_capacity(1);
    assert_eq!(sink.capacity(), 1);
    let route = ResidencyContext::new(
        "tenant:t1",
        "us-ashburn-1",
        "us-phoenix-1",
        "AUDIT",
        "global",
        "replicate_storage",
    )
    .with_pack("pack-us");
    assert_eq!(
        enforce(&engine, &sink, &route).unwrap(),
        ResidencyDecision::Allow
    );
    assert_eq!(sink.seal_count(), 1);

    // At capacity the transfer is REFUSED rather than dispatched unsealed.
    let overflow = enforce(&engine, &sink, &route);
    assert_eq!(overflow, Err(ResidencyAdapterError::AuditSinkUnavailable));
    assert!(!tenancy_data_residency_enforcer::dispatch_permitted(
        &overflow
    ));

    let drained = sink.drain_seals();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].rule, ResidencyRule::IntraPackDrPairTransfer);
    assert_eq!(sink.seal_count(), 0);
    assert_eq!(
        enforce(&engine, &sink, &route).unwrap(),
        ResidencyDecision::Allow
    );
}

#[test]
fn enforce_detailed_returns_the_rule_that_was_recorded() {
    // The caller gets the same rule the audit record got, without re-running
    // the engine against a catalog that may have moved in between.
    let engine = engine();
    let sink = InMemoryDenialAuditSink::new();
    let route = global_route(
        "eu-frankfurt-1",
        "us-ashburn-1",
        "PII_IDENTIFYING",
        "rpc_call",
    )
    .with_destination_pack("pack-us");
    let outcome = enforce_detailed(&engine, &sink, &route).unwrap();
    assert_eq!(outcome.decision, ResidencyDecision::DenyJurisdictionPack);
    assert_eq!(outcome.rule, ResidencyRule::EuTransferRequiresScc);
    assert!(!outcome.is_allow());
    assert_eq!(sink.denials()[0].rule, outcome.rule);
}

#[test]
fn the_audit_sink_can_be_shared_across_threads() {
    // The guard sits on outbound dispatch, which runs multi-threaded; a sink
    // that is not `Sync` cannot be wired there at all.
    fn assert_shareable<T: Send + Sync>() {}
    assert_shareable::<InMemoryDenialAuditSink>();

    let engine = Arc::new(engine());
    let sink = Arc::new(InMemoryDenialAuditSink::new());
    let route = global_route("ap-seoul-1", "ap-tokyo-1", "AUDIT", "emit_event");
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let engine = Arc::clone(&engine);
            let sink = Arc::clone(&sink);
            let route = route.clone();
            scope.spawn(move || {
                enforce(engine.as_ref(), sink.as_ref(), &route).unwrap();
            });
        }
    });
    assert_eq!(sink.denial_count(), 4);
}

#[test]
fn a_catalog_outage_at_construction_is_an_error_not_an_engine() {
    let mut catalog = InMemoryRegionCatalog::oyatie_pack_roster();
    catalog.set_unavailable(true);
    assert_eq!(
        ResidencyPolicyEngine::try_new(catalog).err(),
        Some(ResidencyAdapterError::EvaluationFailed)
    );
}

#[test]
fn a_register_that_cannot_answer_is_an_error_not_a_denial_and_not_an_allow() {
    let mut register = InMemoryTransferRegister::new(vec![eu_us_scc_row()], Vec::new());
    register.set_unavailable(true);
    let engine = default_engine_with_register(register).unwrap();
    let route = global_route(
        "eu-frankfurt-1",
        "us-ashburn-1",
        "PII_IDENTIFYING",
        "rpc_call",
    )
    .with_destination_pack("pack-us")
    .with_transfer_basis(eu_us_scc_assertion());
    let sink = InMemoryDenialAuditSink::new();
    let result = enforce(&engine, &sink, &route);
    assert_eq!(result, Err(ResidencyAdapterError::EvaluationFailed));
    assert!(!tenancy_data_residency_enforcer::dispatch_permitted(
        &result
    ));
}

#[test]
fn a_catalog_that_goes_down_after_construction_denies_dispatch_and_logs_nothing() {
    // The dangerous shape: the catalog validated at startup and failed later.
    // The route must not be dispatched, and no denial record is written either
    // — the control did not decide, so there is nothing honest to record.
    struct FlakyCatalog;
    impl ResidencyRegionCatalog for FlakyCatalog {
        fn lookup(&self, _region_id: &str) -> Result<Option<RegionRecord>, ResidencyAdapterError> {
            Err(ResidencyAdapterError::EvaluationFailed)
        }
        fn regions(&self) -> Result<Vec<RegionRecord>, ResidencyAdapterError> {
            Ok(vec![RegionRecord::new(
                "ap-seoul-1",
                "pack-kr",
                "KR",
                RegionRole::Primary,
            )])
        }
    }
    let engine = ResidencyPolicyEngine::try_new(FlakyCatalog).unwrap();
    let sink = InMemoryDenialAuditSink::new();
    let route = global_route("ap-seoul-1", "ap-seoul-1", "AUDIT", "emit_event");
    let result = enforce(&engine, &sink, &route);
    assert_eq!(result, Err(ResidencyAdapterError::EvaluationFailed));
    assert!(!tenancy_data_residency_enforcer::dispatch_permitted(
        &result
    ));
    assert_eq!(sink.denial_count(), 0);
}

#[test]
fn a_catalog_that_answers_with_the_wrong_row_is_malformed() {
    struct LiarCatalog;
    impl ResidencyRegionCatalog for LiarCatalog {
        fn lookup(&self, _region_id: &str) -> Result<Option<RegionRecord>, ResidencyAdapterError> {
            Ok(Some(RegionRecord::new(
                "somewhere-else-1",
                "pack-kr",
                "KR",
                RegionRole::Primary,
            )))
        }
        fn regions(&self) -> Result<Vec<RegionRecord>, ResidencyAdapterError> {
            Ok(vec![RegionRecord::new(
                "ap-seoul-1",
                "pack-kr",
                "KR",
                RegionRole::Primary,
            )])
        }
    }
    let engine = ResidencyPolicyEngine::try_new(LiarCatalog).unwrap();
    let route = global_route("ap-seoul-1", "ap-seoul-1", "AUDIT", "emit_event");
    assert_eq!(
        engine.evaluate(&route),
        Err(ResidencyAdapterError::PolicyMalformed)
    );
}

#[test]
fn a_contradictory_duplicate_cell_is_malformed_through_the_real_catalog() {
    // The shape an operator config file with a duplicated key produces. A
    // catalog keyed by region id keeps whichever row came last and reports one
    // row, so the engine's duplicate check never sees the contradiction.
    let catalog = InMemoryRegionCatalog::new(vec![
        RegionRecord::new("ap-seoul-1", "pack-kr", "KR", RegionRole::Primary),
        RegionRecord::new("ap-seoul-1", "pack-kr", "JP", RegionRole::Primary),
    ]);
    assert_eq!(catalog.len(), 2, "no row may be silently dropped");
    assert_eq!(
        ResidencyPolicyEngine::try_new(catalog).err(),
        Some(ResidencyAdapterError::PolicyMalformed)
    );
}

#[test]
fn one_region_hosting_two_packs_is_topology_not_contradiction() {
    // Same region id, DIFFERENT pack: the documented co-tenancy. It builds, and
    // the ambiguity is resolved by the caller naming the cell — never by the
    // catalog picking one.
    let catalog = InMemoryRegionCatalog::new(vec![
        RegionRecord::new("ap-seoul-1", "pack-kr", "KR", RegionRole::Primary),
        RegionRecord::new("ap-seoul-1", "pack-jp", "JP", RegionRole::Primary),
        RegionRecord::new("ap-tokyo-1", "pack-jp", "JP", RegionRole::Primary),
    ]);
    let engine = ResidencyPolicyEngine::try_new(catalog).unwrap();

    let unqualified = global_route("ap-seoul-1", "ap-tokyo-1", "AUDIT", "emit_event");
    assert_eq!(
        engine.evaluate(&unqualified).unwrap(),
        ResidencyDecision::DenyResidency
    );
    assert_eq!(
        explain(&engine, &unqualified).unwrap(),
        ResidencyRule::AmbiguousRegionRequiresPack
    );

    // Declared as the KR cell, the route is the cross-jurisdiction move it
    // really is — not the same-pack move a last-row-wins map would have made it.
    let kr = unqualified
        .clone()
        .with_tenant_pack("pack-kr")
        .with_destination_pack("pack-jp");
    assert_eq!(
        engine.evaluate(&kr).unwrap(),
        ResidencyDecision::DenyJurisdictionPack
    );
    assert_eq!(
        explain(&engine, &kr).unwrap(),
        ResidencyRule::CrossJurisdictionForbiddenByDefault
    );
}

// ------------------------------------------------------------------ vocabulary

#[test]
fn every_vocabulary_label_round_trips_and_unknown_labels_do_not() {
    for class in [
        ResidencyClass::StrictHomeRegion,
        ResidencyClass::StrictFederatedRegion,
        ResidencyClass::HomeWithRecoveryFailover,
        ResidencyClass::Global,
    ] {
        assert_eq!(ResidencyClass::parse_label(class.label()), Some(class));
    }
    for data_class in [
        ResidencyDataClass::Public,
        ResidencyDataClass::InternalOnly,
        ResidencyDataClass::Audit,
        ResidencyDataClass::BehavioralTenantProduct,
        ResidencyDataClass::PiiQuasi,
        ResidencyDataClass::PiiIdentifying,
        ResidencyDataClass::SensitivePipaArt23,
        ResidencyDataClass::Secret,
    ] {
        assert_eq!(
            ResidencyDataClass::parse_label(data_class.label()),
            Some(data_class)
        );
    }
    for op in [
        ResidencyOperation::ReadTenantMetadata,
        ResidencyOperation::EmitEvent,
        ResidencyOperation::RpcCall,
        ResidencyOperation::ReplicateStorage,
        ResidencyOperation::AssignDrCell,
        ResidencyOperation::PromoteDr,
        ResidencyOperation::AggregateDsrReceipt,
        ResidencyOperation::MigrateTenantCrossJurisdiction,
    ] {
        assert_eq!(ResidencyOperation::parse_label(op.label()), Some(op));
    }
    for overlay in [
        ResidencyOverlay::KrCsap,
        ResidencyOverlay::EuSovereign,
        ResidencyOverlay::CnPipl,
    ] {
        assert_eq!(
            ResidencyOverlay::parse_label(overlay.label()),
            Some(overlay)
        );
    }
    assert_eq!(ResidencyClass::parse_label(""), None);
    assert_eq!(ResidencyDataClass::parse_label("pii"), None);
    assert_eq!(ResidencyOperation::parse_label("EmitEvent"), None);
    assert_eq!(ResidencyOverlay::parse_label("kr"), None);
    // The catalog spells the quasi-identifier class differently from the
    // architecture doc; both must resolve to the same class.
    assert_eq!(
        ResidencyDataClass::parse_label("PII_QUASI_IDENTIFIER"),
        Some(ResidencyDataClass::PiiQuasi)
    );
}

#[test]
fn the_empty_register_authorises_nothing() {
    let register = NoTransferRegister;
    assert_eq!(register.scc_entry("transfer-register#7").unwrap(), None);
    assert_eq!(
        register.cross_jurisdiction_permit("permit-1").unwrap(),
        None
    );
}

#[test]
fn every_rule_carries_a_distinct_code_and_a_citation() {
    let rules = [
        ResidencyRule::UnknownDataClass,
        ResidencyRule::UnknownResidencyClass,
        ResidencyRule::UnknownOperation,
        ResidencyRule::UnknownResidencyOverlay,
        ResidencyRule::UnknownRegion,
        ResidencyRule::AmbiguousRegionRequiresPack,
        ResidencyRule::OverlayKrCsapOffshore,
        ResidencyRule::OverlayEuSovereignNonSovereignRegion,
        ResidencyRule::OverlayCnPiplOffshore,
        ResidencyRule::SameRegion,
        ResidencyRule::SensitiveDataCrossJurisdiction,
        ResidencyRule::SecretCrossPack,
        ResidencyRule::DsrAggregationRequiresIntraPack,
        ResidencyRule::StrictResidencyForbidsCrossRegion,
        ResidencyRule::RecoveryFailoverRequiresIntraPackDrPair,
        ResidencyRule::CrossJurisdictionMigrationRequiresPermit,
        ResidencyRule::CrossJurisdictionMigrationAuthorised,
        ResidencyRule::EuTransferRequiresScc,
        ResidencyRule::EuTransferAuthorisedByScc,
        ResidencyRule::HealthcarePackCrossJurisdictionUnauthorised,
        ResidencyRule::CrossJurisdictionForbiddenByDefault,
        ResidencyRule::IntraPackDrPairTransfer,
        ResidencyRule::IntraPackTransfer,
        ResidencyRule::IntraPackDsrReceiptAggregation,
        ResidencyRule::CrossPackWithinJurisdictionForbidden,
        ResidencyRule::RuleNotReported,
    ];
    let mut codes: Vec<&str> = rules.iter().map(|rule| rule.code()).collect();
    codes.sort_unstable();
    let unique = codes.len();
    codes.dedup();
    assert_eq!(codes.len(), unique, "rule codes must be distinct");
    for rule in rules {
        assert!(
            !rule.citation().is_empty(),
            "{} lacks a citation",
            rule.code()
        );
    }
}
