use oya_real_estate_portfolio_domain::{
    CashFlowProjectionState, FacilityLinkState, FacilityMaintenanceLinkInput,
    FacilityServicePriority, LeaseAccountingClassification, LeaseCashFlowInput, LeaseContractInput,
    LeaseContractState, LeaseDirection, OccupancyPlanState, PaymentFrequency,
    PropertyRegistrationState, RealEstateObjectInput, RealEstateObjectType,
    RealEstatePortfolioError, RealEstateUsageKind, SpaceOccupancyInput, plan_space_occupancy,
    prepare_facility_maintenance_link, project_lease_cash_flow, register_lease_contract,
    register_real_estate_object,
};

fn property_input() -> RealEstateObjectInput {
    RealEstateObjectInput {
        property_id: "prop_seoul_tower".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_kr001".to_owned(),
        business_entity_id: "be_seoul_portfolio".to_owned(),
        building_id: "bldg_seoul_tower_a".to_owned(),
        rental_object_id: "rent_floor_12_east".to_owned(),
        object_type: RealEstateObjectType::RentalSpace,
        usage_kind: RealEstateUsageKind::LeaseOut,
        gross_area_square_meters: 1_200,
        rentable_area_square_meters: 900,
        capacity_seats: 120,
        valid_from_yyyymmdd: 20260701,
        valid_to_yyyymmdd: 20360630,
        object_source_ref: "src/real-estate/object/seoul-tower-floor-12-east".to_owned(),
        registration_evidence_ref: "audit/real-estate/object/prop_seoul_tower/register".to_owned(),
    }
}

fn lease_input(property_registered: bool) -> LeaseContractInput {
    LeaseContractInput {
        lease_contract_id: "lease_seoul_tower_floor_12".to_owned(),
        property_id: "prop_seoul_tower".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_kr001".to_owned(),
        business_partner_id: "bp_anchor_tenant".to_owned(),
        property_registered,
        lease_direction: LeaseDirection::LeaseOut,
        accounting_classification: LeaseAccountingClassification::RevenueOperating,
        commencement_yyyymmdd: 20260701,
        expiration_yyyymmdd: 20290630,
        term_months: 36,
        monthly_base_rent_cents: 2_500_000,
        security_deposit_cents: 5_000_000,
        contract_source_ref: "src/real-estate/lease/lease_seoul_tower_floor_12".to_owned(),
        contract_evidence_ref: "audit/real-estate/lease/lease_seoul_tower_floor_12/register"
            .to_owned(),
    }
}

fn cash_flow_input(lease_contract_registered: bool) -> LeaseCashFlowInput {
    LeaseCashFlowInput {
        cash_flow_id: "cashflow_seoul_tower_floor_12".to_owned(),
        lease_contract_id: "lease_seoul_tower_floor_12".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_kr001".to_owned(),
        lease_contract_registered,
        payment_frequency: PaymentFrequency::Monthly,
        number_of_periods: 36,
        recurring_payment_cents: 2_500_000,
        first_due_yyyymmdd: 20260731,
        final_due_yyyymmdd: 20290630,
        cash_flow_source_ref: "src/real-estate/cash-flow/lease_seoul_tower_floor_12".to_owned(),
        cash_flow_evidence_ref: "audit/real-estate/cash-flow/lease_seoul_tower_floor_12/project"
            .to_owned(),
    }
}

fn occupancy_input(property_registered: bool) -> SpaceOccupancyInput {
    SpaceOccupancyInput {
        occupancy_plan_id: "occupancy_seoul_tower_team_alpha".to_owned(),
        property_id: "prop_seoul_tower".to_owned(),
        rental_object_id: "rent_floor_12_east".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_kr001".to_owned(),
        property_registered,
        total_rentable_area_square_meters: 900,
        already_committed_area_square_meters: 350,
        requested_area_square_meters: 250,
        requested_seats: 40,
        team_ref: "team/workplace-alpha".to_owned(),
        occupancy_start_yyyymmdd: 20260701,
        occupancy_end_yyyymmdd: 20270630,
        occupancy_source_ref: "src/real-estate/occupancy/team-alpha".to_owned(),
        occupancy_evidence_ref: "audit/real-estate/occupancy/team-alpha/plan".to_owned(),
    }
}

fn facility_input(property_registered: bool) -> FacilityMaintenanceLinkInput {
    FacilityMaintenanceLinkInput {
        facility_link_id: "facility_seoul_tower_floor_12_hvac".to_owned(),
        property_id: "prop_seoul_tower".to_owned(),
        rental_object_id: "rent_floor_12_east".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_kr001".to_owned(),
        property_registered,
        maintenance_asset_ref: "asset/pm/hvac-floor-12".to_owned(),
        service_priority: FacilityServicePriority::High,
        planned_window_days: 14,
        facility_source_ref: "src/real-estate/facility/hvac-floor-12".to_owned(),
        facility_evidence_ref: "audit/real-estate/facility/hvac-floor-12/prepare".to_owned(),
    }
}

#[test]
fn property_drives_lease_cash_flow_space_and_facility_linkage() {
    let property = register_real_estate_object(property_input()).unwrap();
    assert_eq!(property.state.value, PropertyRegistrationState::Registered);
    assert_eq!(property.rentable_area_square_meters.value, 900);
    assert!(!property.architectural_view_attached.value);
    assert!(!property.sap_re_fx_backend_attached.value);
    assert!(!property.fixed_asset_master_attached.value);
    assert!(
        !property
            .plant_maintenance_functional_location_attached
            .value
    );
    assert!(!property.cloud_deployment_attached.value);

    let lease = register_lease_contract(lease_input(true)).unwrap();
    assert_eq!(lease.state.value, LeaseContractState::Registered);
    assert_eq!(lease.total_nominal_rent_cents.value, 90_000_000);
    assert!(!lease.lease_accounting_engine_attached.value);
    assert!(!lease.general_ledger_posting_attached.value);
    assert!(!lease.accounts_payable_or_receivable_attached.value);
    assert!(!lease.document_archive_attached.value);

    let cash_flow = project_lease_cash_flow(cash_flow_input(true)).unwrap();
    assert_eq!(cash_flow.state.value, CashFlowProjectionState::Projected);
    assert_eq!(cash_flow.projected_total_cash_flow_cents.value, 90_000_000);
    assert!(!cash_flow.periodic_posting_attached.value);
    assert!(!cash_flow.payment_run_attached.value);
    assert!(!cash_flow.subledger_accounting_attached.value);

    let occupancy = plan_space_occupancy(occupancy_input(true)).unwrap();
    assert_eq!(occupancy.state.value, OccupancyPlanState::Planned);
    assert_eq!(occupancy.remaining_area_square_meters.value, 300);
    assert!(occupancy.area_capacity_sufficient.value);
    assert!(!occupancy.workspace_runtime_attached.value);
    assert!(!occupancy.team_directory_sync_attached.value);
    assert!(!occupancy.reservation_mutation_attached.value);

    let facility = prepare_facility_maintenance_link(facility_input(true)).unwrap();
    assert_eq!(facility.state.value, FacilityLinkState::Prepared);
    assert!(!facility.plant_maintenance_order_attached.value);
    assert!(!facility.iot_or_scada_ingestion_attached.value);
    assert!(!facility.service_ticket_runtime_attached.value);
    assert!(!facility.workflow_execution_attached.value);
}

#[test]
fn real_estate_refuses_missing_prerequisites() {
    assert_eq!(
        register_lease_contract(lease_input(false)),
        Err(RealEstatePortfolioError::PropertyRegistrationRequired)
    );
    assert_eq!(
        project_lease_cash_flow(cash_flow_input(false)),
        Err(RealEstatePortfolioError::LeaseContractRequired)
    );
    assert_eq!(
        plan_space_occupancy(occupancy_input(false)),
        Err(RealEstatePortfolioError::PropertyRegistrationRequired)
    );
    assert_eq!(
        prepare_facility_maintenance_link(facility_input(false)),
        Err(RealEstatePortfolioError::PropertyRegistrationRequired)
    );
}

#[test]
fn real_estate_validates_refs_dates_areas_terms_and_amounts() {
    let mut unsafe_property = property_input();
    unsafe_property.registration_evidence_ref = "audit/real-estate/secret-token".to_owned();
    assert_eq!(
        register_real_estate_object(unsafe_property),
        Err(RealEstatePortfolioError::InvalidEvidenceRef)
    );

    let mut bad_area = property_input();
    bad_area.rentable_area_square_meters = 1_300;
    assert_eq!(
        register_real_estate_object(bad_area),
        Err(RealEstatePortfolioError::InvalidArea)
    );

    let mut bad_date = lease_input(true);
    bad_date.expiration_yyyymmdd = 20260630;
    assert_eq!(
        register_lease_contract(bad_date),
        Err(RealEstatePortfolioError::InvalidDate)
    );

    let mut bad_term = lease_input(true);
    bad_term.term_months = 0;
    assert_eq!(
        register_lease_contract(bad_term),
        Err(RealEstatePortfolioError::InvalidTerm)
    );

    let mut bad_source = occupancy_input(true);
    bad_source.occupancy_source_ref = "src/../occupancy".to_owned();
    assert_eq!(
        plan_space_occupancy(bad_source),
        Err(RealEstatePortfolioError::InvalidSourceDocumentRef)
    );

    let mut bad_window = facility_input(true);
    bad_window.planned_window_days = 0;
    assert_eq!(
        prepare_facility_maintenance_link(bad_window),
        Err(RealEstatePortfolioError::InvalidWindow)
    );
}

#[test]
fn real_estate_records_capacity_shortage_without_reservation_claim() {
    let mut oversized = occupancy_input(true);
    oversized.requested_area_square_meters = 800;
    assert_eq!(
        plan_space_occupancy(oversized),
        Err(RealEstatePortfolioError::AreaCapacityExceeded)
    );

    let mut zero_payment = cash_flow_input(true);
    zero_payment.recurring_payment_cents = 0;
    assert_eq!(
        project_lease_cash_flow(zero_payment),
        Err(RealEstatePortfolioError::InvalidAmount)
    );
}
