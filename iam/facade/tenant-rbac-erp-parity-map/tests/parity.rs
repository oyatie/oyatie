use iam_tenant_rbac_erp_parity_map::{
    ErpParityMapError, HyperscalerParityFacet, HyperscalerParityStatus, ParityTier, SapModuleCode,
    erp_hyperscaler_parity_matrix, erp_parity_map_capabilities, find_erp_module,
    is_forbidden_erp_platform_destination, tenant_rbac_erp_parity_map,
    validate_erp_hyperscaler_parity_matrix, validate_erp_parity_map,
};
use std::path::Path;

fn path_without_fragment(reference: &str) -> &str {
    reference
        .split_once('#')
        .map_or(reference, |(path, _)| path)
}

#[test]
fn erp_parity_map_covers_sap_modules_without_erp_platform_service() {
    let rows = tenant_rbac_erp_parity_map();
    validate_erp_parity_map(rows).expect("ERP parity map validates");
    assert_eq!(rows.len(), 23);
    for row in rows {
        assert!(!row.oyatie_destinations.is_empty());
        assert!(!row.production_runtime_claimed);
        assert!(!row.cloud_integration_ready);
        for destination in row.oyatie_destinations {
            assert!(!is_forbidden_erp_platform_destination(destination));
        }
    }
}

#[test]
fn hcm_and_financial_rows_reference_landed_foundations_without_cloud_claim() {
    let hcm = find_erp_module(SapModuleCode::Hcm).expect("HCM row exists");
    assert!(
        hcm.oyatie_destinations
            .contains(&"specs/microservices/hr.json")
    );
    assert!(
        hcm.oyatie_destinations
            .contains(&"specs/microservices/payroll.json")
    );
    assert!(hcm.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-platform-local-inmemory-harness-1779541800.json"
    ));
    assert_eq!(hcm.tier, ParityTier::ComposedCoverage);
    assert!(!hcm.production_runtime_claimed);

    let financial_accounting = find_erp_module(SapModuleCode::Fi).expect("FI row exists");
    assert!(
        financial_accounting
            .oyatie_destinations
            .contains(&"specs/microservices/accounting.json")
    );
    assert!(financial_accounting.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-accounting-storage-adapter-inmemory-1779540600.json"
    ));
    assert!(!financial_accounting.cloud_integration_ready);
}

#[test]
fn map_records_remaining_new_required_business_gaps_as_flat_services() {
    let rows = tenant_rbac_erp_parity_map();
    assert!(
        rows.iter()
            .all(|row| row.tier != ParityTier::NewFlatServiceRequired)
    );
}

#[test]
fn real_estate_row_references_portfolio_domain_foundation() {
    let real_estate = find_erp_module(SapModuleCode::ReFx).expect("RE-FX row exists");
    assert_eq!(
        real_estate.first_write_owner,
        "real-estate-portfolio-domain"
    );
    assert_eq!(real_estate.tier, ParityTier::ComposedCoverage);
    assert!(
        real_estate
            .oyatie_destinations
            .contains(&"specs/microservices/real-estate.json")
    );
    assert!(
        real_estate
            .oyatie_destinations
            .contains(&"microservices/real-estate/crates/real-estate-portfolio-domain")
    );
    assert!(
        real_estate.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-real-estate-portfolio-domain-1779550800.json"
        )
    );
    assert!(!real_estate.production_runtime_claimed);
}

#[test]
fn global_trade_row_references_compliance_domain_foundation() {
    let global_trade = find_erp_module(SapModuleCode::Gts).expect("GTS row exists");
    assert_eq!(
        global_trade.first_write_owner,
        "global-trade-compliance-domain"
    );
    assert_eq!(global_trade.tier, ParityTier::ComposedCoverage);
    assert!(
        global_trade
            .oyatie_destinations
            .contains(&"specs/microservices/global-trade.json")
    );
    assert!(
        global_trade
            .oyatie_destinations
            .contains(&"microservices/global-trade/crates/global-trade-compliance-domain")
    );
    assert!(
        global_trade.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-global-trade-compliance-domain-1779550200.json"
        )
    );
    assert!(!global_trade.production_runtime_claimed);
}

#[test]
fn crm_row_references_customer_engagement_domain_foundation() {
    let crm = find_erp_module(SapModuleCode::Crm).expect("CRM row exists");
    assert_eq!(crm.first_write_owner, "crm-customer-engagement-domain");
    assert_eq!(crm.tier, ParityTier::ComposedCoverage);
    assert!(
        crm.oyatie_destinations
            .contains(&"specs/microservices/crm.json")
    );
    assert!(
        crm.oyatie_destinations
            .contains(&"microservices/crm/crates/crm-customer-engagement-domain")
    );
    assert!(
        crm.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-crm-customer-engagement-domain-1779549600.json"
        )
    );
    assert!(!crm.production_runtime_claimed);
}

#[test]
fn supply_chain_planning_row_references_domain_foundation() {
    let advanced_planning = find_erp_module(SapModuleCode::ScmApo).expect("SCM/APO row exists");
    assert_eq!(
        advanced_planning.first_write_owner,
        "supply-chain-planning-domain"
    );
    assert_eq!(advanced_planning.tier, ParityTier::ComposedCoverage);
    assert!(
        advanced_planning
            .oyatie_destinations
            .contains(&"specs/microservices/supply-chain-planning.json")
    );
    assert!(
        advanced_planning.oyatie_destinations.contains(
            &"microservices/supply-chain-planning/crates/supply-chain-planning-domain"
        )
    );
    assert!(
        advanced_planning.evidence_refs.contains(
            &"evidence/multispectrum/cs-ent-supply-chain-planning-domain-1779549000.json"
        )
    );
    assert!(!advanced_planning.production_runtime_claimed);
}

#[test]
fn plant_maintenance_row_references_domain_foundation() {
    let plant_maintenance = find_erp_module(SapModuleCode::Pm).expect("PM row exists");
    assert_eq!(
        plant_maintenance.first_write_owner,
        "plant-maintenance-domain"
    );
    assert_eq!(plant_maintenance.tier, ParityTier::ComposedCoverage);
    assert!(
        plant_maintenance
            .oyatie_destinations
            .contains(&"specs/microservices/plant-maintenance.json")
    );
    assert!(
        plant_maintenance
            .oyatie_destinations
            .contains(&"microservices/plant-maintenance/crates/plant-maintenance-domain")
    );
    assert!(
        plant_maintenance
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-plant-maintenance-domain-1779548400.json")
    );
    assert!(!plant_maintenance.production_runtime_claimed);
}

#[test]
fn quality_management_row_references_domain_foundation() {
    let quality = find_erp_module(SapModuleCode::Qm).expect("QM row exists");
    assert_eq!(quality.first_write_owner, "quality-management-domain");
    assert_eq!(quality.tier, ParityTier::ComposedCoverage);
    assert!(
        quality
            .oyatie_destinations
            .contains(&"specs/microservices/quality-management.json")
    );
    assert!(
        quality
            .oyatie_destinations
            .contains(&"microservices/quality-management/crates/quality-management-domain")
    );
    assert!(
        quality
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-quality-management-domain-1779547800.json")
    );
    assert!(!quality.production_runtime_claimed);
}

#[test]
fn production_planning_row_references_domain_foundation() {
    let production_planning = find_erp_module(SapModuleCode::Pp).expect("PP row exists");
    assert_eq!(
        production_planning.first_write_owner,
        "production-planning-domain"
    );
    assert_eq!(production_planning.tier, ParityTier::ComposedCoverage);
    assert!(
        production_planning
            .oyatie_destinations
            .contains(&"specs/microservices/production-planning.json")
    );
    assert!(
        production_planning
            .oyatie_destinations
            .contains(&"microservices/production-planning/crates/production-planning-domain")
    );
    assert!(
        production_planning
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-production-planning-domain-1779547200.json")
    );
    assert!(!production_planning.production_runtime_claimed);
}

#[test]
fn materials_and_supplier_rows_reference_procurement_foundation() {
    let materials = find_erp_module(SapModuleCode::Mm).expect("MM row exists");
    assert_eq!(materials.first_write_owner, "procurement-source-to-pay");
    assert!(
        materials
            .oyatie_destinations
            .contains(&"specs/microservices/procurement.json")
    );
    assert!(
        materials
            .oyatie_destinations
            .contains(&"microservices/procurement/crates/procurement-source-to-pay-domain")
    );
    assert!(materials.evidence_refs.contains(
        &"evidence/multispectrum/cs-ent-procurement-source-to-pay-domain-1779545400.json"
    ));
    assert!(!materials.cloud_integration_ready);

    let supplier_relationship = find_erp_module(SapModuleCode::Srm).expect("SRM row exists");
    assert_eq!(
        supplier_relationship.first_write_owner,
        "procurement-source-to-pay"
    );
    assert!(
        supplier_relationship
            .oyatie_destinations
            .contains(&"microservices/procurement/crates/procurement-source-to-pay-domain")
    );
    assert!(!supplier_relationship.production_runtime_claimed);
}

#[test]
fn materials_and_warehouse_rows_reference_warehouse_inventory_foundation() {
    let materials = find_erp_module(SapModuleCode::Mm).expect("MM row exists");
    assert!(
        materials
            .oyatie_destinations
            .contains(&"specs/microservices/warehouse.json")
    );
    assert!(
        materials
            .oyatie_destinations
            .contains(&"microservices/warehouse/crates/warehouse-inventory-domain")
    );
    assert!(
        materials
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-warehouse-inventory-domain-1779546600.json")
    );

    let warehouse = find_erp_module(SapModuleCode::Ewm).expect("EWM row exists");
    assert_eq!(warehouse.first_write_owner, "warehouse-inventory");
    assert_eq!(warehouse.tier, ParityTier::ComposedCoverage);
    assert!(
        warehouse
            .oyatie_destinations
            .contains(&"specs/microservices/warehouse.json")
    );
    assert!(
        warehouse
            .oyatie_destinations
            .contains(&"microservices/warehouse/crates/warehouse-inventory-domain")
    );
    assert!(!warehouse.production_runtime_claimed);
}

#[test]
fn financial_and_treasury_rows_reference_treasury_cash_foundation() {
    let financial_accounting = find_erp_module(SapModuleCode::Fi).expect("FI row exists");
    assert!(
        financial_accounting
            .oyatie_destinations
            .contains(&"specs/microservices/treasury.json")
    );
    assert!(
        financial_accounting
            .oyatie_destinations
            .contains(&"microservices/treasury/crates/treasury-cash-domain")
    );
    assert!(
        financial_accounting
            .evidence_refs
            .contains(&"evidence/multispectrum/cs-ent-treasury-cash-domain-1779546000.json")
    );

    let treasury = find_erp_module(SapModuleCode::Trm).expect("TRM row exists");
    assert_eq!(treasury.first_write_owner, "treasury-cash");
    assert_eq!(treasury.tier, ParityTier::ComposedCoverage);
    assert!(
        treasury
            .oyatie_destinations
            .contains(&"specs/microservices/treasury.json")
    );
    assert!(
        treasury
            .oyatie_destinations
            .contains(&"microservices/treasury/crates/treasury-cash-domain")
    );
    assert!(!treasury.production_runtime_claimed);
}

#[test]
fn capabilities_keep_runtime_and_cloud_non_claims_false() {
    let capabilities = erp_parity_map_capabilities();
    assert_eq!(capabilities.sap_module_count, 23);
    assert!(capabilities.compositional_parity_map_attached);
    assert!(!capabilities.erp_platform_microservice_created);
    assert!(!capabilities.deployed_listener_attached);
    assert!(!capabilities.durable_business_document_store_attached);
    assert!(!capabilities.workflow_engine_execution_attached);
    assert!(!capabilities.cloud_deployment_attached);
    assert!(!capabilities.runtime_audit_chain_emission_attached);
    assert_eq!(capabilities.schema_version, 1);
}

#[test]
fn validation_rejects_erp_platform_destination() {
    let mut rows = tenant_rbac_erp_parity_map().to_vec();
    rows[0].oyatie_destinations = &["microservices/erp"];
    let error = validate_erp_parity_map(&rows).expect_err("ERP platform destination is rejected");
    assert_eq!(
        error,
        ErpParityMapError::ForbiddenErpPlatformDestination {
            sap_code: SapModuleCode::Fi,
            destination: "microservices/erp",
        }
    );
}

#[test]
fn hyperscaler_parity_matrix_tracks_required_control_plane_facets() {
    let matrix = erp_hyperscaler_parity_matrix();
    validate_erp_hyperscaler_parity_matrix(matrix).expect("hyperscaler parity matrix validates");
    assert_eq!(matrix.len(), 15);

    for criterion in matrix {
        assert!(!criterion.benchmark_surface.trim().is_empty());
        assert!(!criterion.oyatie_evidence_refs.is_empty());
        assert!(!criterion.gap_closure_gate.trim().is_empty());
    }

    assert!(matrix.iter().any(|criterion| {
        criterion.facet == HyperscalerParityFacet::ControlPlaneApi
            && criterion.status == HyperscalerParityStatus::GapTracked
    }));
    assert!(matrix.iter().any(|criterion| {
        criterion.facet == HyperscalerParityFacet::BillingMetering
            && criterion.status == HyperscalerParityStatus::GapTracked
    }));
    assert!(matrix.iter().any(|criterion| {
        criterion.facet == HyperscalerParityFacet::OperationalRunbooks
            && criterion.status == HyperscalerParityStatus::GapTracked
    }));
    assert!(
        matrix
            .iter()
            .all(|criterion| criterion.status == HyperscalerParityStatus::GapTracked),
        "status must stay GapTracked until this crate owns executable semantic verification"
    );
}

#[test]
fn hyperscaler_parity_matrix_validation_rejects_missing_required_facet() {
    let mut matrix = erp_hyperscaler_parity_matrix().to_vec();
    matrix.retain(|criterion| criterion.facet != HyperscalerParityFacet::ControlPlaneApi);

    let error = validate_erp_hyperscaler_parity_matrix(&matrix)
        .expect_err("missing control-plane API facet is rejected");
    assert_eq!(
        error,
        ErpParityMapError::MissingHyperscalerParityFacet("control-plane API")
    );
}

#[test]
fn hyperscaler_parity_matrix_validation_rejects_missing_benchmark_surface() {
    let mut matrix = erp_hyperscaler_parity_matrix().to_vec();
    matrix[0].benchmark_surface = " ";

    let error = validate_erp_hyperscaler_parity_matrix(&matrix)
        .expect_err("missing hyperscaler benchmark surface is rejected");
    assert_eq!(
        error,
        ErpParityMapError::MissingHyperscalerParityBenchmark(
            HyperscalerParityFacet::ControlPlaneApi,
        )
    );
}

#[test]
fn hyperscaler_parity_matrix_validation_rejects_unverified_verified_status() {
    let mut matrix = erp_hyperscaler_parity_matrix().to_vec();
    matrix[0].status = HyperscalerParityStatus::Verified;

    let error = validate_erp_hyperscaler_parity_matrix(&matrix)
        .expect_err("Verified status is rejected until semantic verification exists");
    assert_eq!(
        error,
        ErpParityMapError::UnverifiedHyperscalerParityClaim(
            HyperscalerParityFacet::ControlPlaneApi,
        )
    );
}

#[test]
fn hyperscaler_parity_matrix_evidence_refs_stay_repo_relative() {
    for criterion in erp_hyperscaler_parity_matrix() {
        for reference in criterion.oyatie_evidence_refs {
            let path = path_without_fragment(reference);
            assert!(
                Path::new(path).is_relative(),
                "matrix evidence references must stay repo-relative: {reference}"
            );
            assert!(
                !Path::new(path)
                    .components()
                    .any(|component| matches!(component, std::path::Component::ParentDir)),
                "matrix evidence references must not climb out of the repo: {reference}"
            );
        }
    }
}
