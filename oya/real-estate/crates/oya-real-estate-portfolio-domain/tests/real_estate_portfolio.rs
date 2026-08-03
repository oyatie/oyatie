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
    assert_eq!(
        property.property_id.value.value.as_str(),
        "prop_seoul_tower"
    );
    assert_eq!(property.tenant_id.value.value.as_str(), "ten_enterprise");
    assert_eq!(property.legal_entity_id.value.value.as_str(), "le_kr001");
    assert_eq!(
        property.business_entity_id.value.value.as_str(),
        "be_seoul_portfolio"
    );
    assert_eq!(
        property.building_id.value.value.as_str(),
        "bldg_seoul_tower_a"
    );
    assert_eq!(
        property.rental_object_id.value.value.as_str(),
        "rent_floor_12_east"
    );
    assert_eq!(
        property.object_type.value,
        RealEstateObjectType::RentalSpace
    );
    assert_eq!(property.usage_kind.value, RealEstateUsageKind::LeaseOut);
    assert_eq!(property.gross_area_square_meters.value, 1_200);
    assert_eq!(property.state.value, PropertyRegistrationState::Registered);
    assert_eq!(property.rentable_area_square_meters.value, 900);
    assert_eq!(property.capacity_seats.value, 120);
    assert_eq!(property.valid_from_yyyymmdd.value, 20260701);
    assert_eq!(property.valid_to_yyyymmdd.value, 20360630);
    assert_eq!(
        property.object_source_ref.value.value.as_str(),
        "src/real-estate/object/seoul-tower-floor-12-east"
    );
    assert_eq!(
        property.registration_evidence_ref.value.value.as_str(),
        "audit/real-estate/object/prop_seoul_tower/register"
    );
    assert_eq!(
        property.idempotency_key.value.as_str(),
        "real-estate:object:ten_enterprise:le_kr001:prop_seoul_tower"
    );
    assert!(!property.architectural_view_attached.value);
    assert!(!property.sap_re_fx_backend_attached.value);
    assert!(!property.fixed_asset_master_attached.value);
    assert!(
        !property
            .plant_maintenance_functional_location_attached
            .value
    );
    assert!(!property.cloud_deployment_attached.value);
    assert_eq!(property.schema_version.value, 1);

    let lease = register_lease_contract(lease_input(true)).unwrap();
    assert_eq!(
        lease.lease_contract_id.value.value.as_str(),
        "lease_seoul_tower_floor_12"
    );
    assert_eq!(lease.property_id.value.value.as_str(), "prop_seoul_tower");
    assert_eq!(lease.tenant_id.value.value.as_str(), "ten_enterprise");
    assert_eq!(lease.legal_entity_id.value.value.as_str(), "le_kr001");
    assert_eq!(
        lease.business_partner_id.value.value.as_str(),
        "bp_anchor_tenant"
    );
    assert_eq!(lease.lease_direction.value, LeaseDirection::LeaseOut);
    assert_eq!(
        lease.accounting_classification.value,
        LeaseAccountingClassification::RevenueOperating
    );
    assert_eq!(lease.commencement_yyyymmdd.value, 20260701);
    assert_eq!(lease.expiration_yyyymmdd.value, 20290630);
    assert_eq!(lease.term_months.value, 36);
    assert_eq!(lease.monthly_base_rent_cents.value, 2_500_000);
    assert_eq!(lease.security_deposit_cents.value, 5_000_000);
    assert_eq!(lease.state.value, LeaseContractState::Registered);
    assert_eq!(lease.total_nominal_rent_cents.value, 90_000_000);
    assert_eq!(
        lease.contract_source_ref.value.value.as_str(),
        "src/real-estate/lease/lease_seoul_tower_floor_12"
    );
    assert_eq!(
        lease.contract_evidence_ref.value.value.as_str(),
        "audit/real-estate/lease/lease_seoul_tower_floor_12/register"
    );
    assert_eq!(
        lease.idempotency_key.value.as_str(),
        "real-estate:lease:ten_enterprise:le_kr001:lease_seoul_tower_floor_12"
    );
    assert!(!lease.lease_accounting_engine_attached.value);
    assert!(!lease.general_ledger_posting_attached.value);
    assert!(!lease.accounts_payable_or_receivable_attached.value);
    assert!(!lease.document_archive_attached.value);
    assert!(!lease.cloud_deployment_attached.value);
    assert_eq!(lease.schema_version.value, 1);

    let cash_flow = project_lease_cash_flow(cash_flow_input(true)).unwrap();
    assert_eq!(
        cash_flow.cash_flow_id.value.value.as_str(),
        "cashflow_seoul_tower_floor_12"
    );
    assert_eq!(
        cash_flow.lease_contract_id.value.value.as_str(),
        "lease_seoul_tower_floor_12"
    );
    assert_eq!(cash_flow.tenant_id.value.value.as_str(), "ten_enterprise");
    assert_eq!(cash_flow.legal_entity_id.value.value.as_str(), "le_kr001");
    assert_eq!(cash_flow.payment_frequency.value, PaymentFrequency::Monthly);
    assert_eq!(cash_flow.number_of_periods.value, 36);
    assert_eq!(cash_flow.recurring_payment_cents.value, 2_500_000);
    assert_eq!(cash_flow.state.value, CashFlowProjectionState::Projected);
    assert_eq!(cash_flow.projected_total_cash_flow_cents.value, 90_000_000);
    assert_eq!(cash_flow.first_due_yyyymmdd.value, 20260731);
    assert_eq!(cash_flow.final_due_yyyymmdd.value, 20290630);
    assert_eq!(
        cash_flow.cash_flow_source_ref.value.value.as_str(),
        "src/real-estate/cash-flow/lease_seoul_tower_floor_12"
    );
    assert_eq!(
        cash_flow.cash_flow_evidence_ref.value.value.as_str(),
        "audit/real-estate/cash-flow/lease_seoul_tower_floor_12/project"
    );
    assert_eq!(
        cash_flow.idempotency_key.value.as_str(),
        "real-estate:cash-flow:ten_enterprise:le_kr001:cashflow_seoul_tower_floor_12"
    );
    assert!(!cash_flow.periodic_posting_attached.value);
    assert!(!cash_flow.payment_run_attached.value);
    assert!(!cash_flow.subledger_accounting_attached.value);
    assert!(!cash_flow.cloud_deployment_attached.value);
    assert_eq!(cash_flow.schema_version.value, 1);

    let occupancy = plan_space_occupancy(occupancy_input(true)).unwrap();
    assert_eq!(
        occupancy.occupancy_plan_id.value.value.as_str(),
        "occupancy_seoul_tower_team_alpha"
    );
    assert_eq!(
        occupancy.property_id.value.value.as_str(),
        "prop_seoul_tower"
    );
    assert_eq!(
        occupancy.rental_object_id.value.value.as_str(),
        "rent_floor_12_east"
    );
    assert_eq!(occupancy.tenant_id.value.value.as_str(), "ten_enterprise");
    assert_eq!(occupancy.legal_entity_id.value.value.as_str(), "le_kr001");
    assert_eq!(occupancy.total_rentable_area_square_meters.value, 900);
    assert_eq!(occupancy.already_committed_area_square_meters.value, 350);
    assert_eq!(occupancy.requested_area_square_meters.value, 250);
    assert_eq!(occupancy.state.value, OccupancyPlanState::Planned);
    assert_eq!(occupancy.remaining_area_square_meters.value, 300);
    assert_eq!(occupancy.requested_seats.value, 40);
    assert_eq!(
        occupancy.team_ref.value.value.as_str(),
        "team/workplace-alpha"
    );
    assert_eq!(occupancy.occupancy_start_yyyymmdd.value, 20260701);
    assert_eq!(occupancy.occupancy_end_yyyymmdd.value, 20270630);
    assert_eq!(
        occupancy.occupancy_source_ref.value.value.as_str(),
        "src/real-estate/occupancy/team-alpha"
    );
    assert_eq!(
        occupancy.occupancy_evidence_ref.value.value.as_str(),
        "audit/real-estate/occupancy/team-alpha/plan"
    );
    assert_eq!(
        occupancy.idempotency_key.value.as_str(),
        "real-estate:occupancy:ten_enterprise:le_kr001:occupancy_seoul_tower_team_alpha"
    );
    assert!(occupancy.area_capacity_sufficient.value);
    assert!(!occupancy.workspace_runtime_attached.value);
    assert!(!occupancy.team_directory_sync_attached.value);
    assert!(!occupancy.reservation_mutation_attached.value);
    assert!(!occupancy.cloud_deployment_attached.value);
    assert_eq!(occupancy.schema_version.value, 1);

    let facility = prepare_facility_maintenance_link(facility_input(true)).unwrap();
    assert_eq!(
        facility.facility_link_id.value.value.as_str(),
        "facility_seoul_tower_floor_12_hvac"
    );
    assert_eq!(
        facility.property_id.value.value.as_str(),
        "prop_seoul_tower"
    );
    assert_eq!(
        facility.rental_object_id.value.value.as_str(),
        "rent_floor_12_east"
    );
    assert_eq!(facility.tenant_id.value.value.as_str(), "ten_enterprise");
    assert_eq!(facility.legal_entity_id.value.value.as_str(), "le_kr001");
    assert_eq!(
        facility.maintenance_asset_ref.value.value.as_str(),
        "asset/pm/hvac-floor-12"
    );
    assert_eq!(
        facility.service_priority.value,
        FacilityServicePriority::High
    );
    assert_eq!(facility.planned_window_days.value, 14);
    assert_eq!(
        facility.facility_source_ref.value.value.as_str(),
        "src/real-estate/facility/hvac-floor-12"
    );
    assert_eq!(
        facility.facility_evidence_ref.value.value.as_str(),
        "audit/real-estate/facility/hvac-floor-12/prepare"
    );
    assert_eq!(facility.state.value, FacilityLinkState::Prepared);
    assert_eq!(
        facility.idempotency_key.value.as_str(),
        "real-estate:facility:ten_enterprise:le_kr001:facility_seoul_tower_floor_12_hvac"
    );
    assert!(!facility.plant_maintenance_order_attached.value);
    assert!(!facility.iot_or_scada_ingestion_attached.value);
    assert!(!facility.service_ticket_runtime_attached.value);
    assert!(!facility.workflow_execution_attached.value);
    assert!(!facility.cloud_deployment_attached.value);
    assert_eq!(facility.schema_version.value, 1);
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

    let mut impossible_date = property_input();
    impossible_date.valid_to_yyyymmdd = 20260230;
    assert_eq!(
        register_real_estate_object(impossible_date),
        Err(RealEstatePortfolioError::InvalidDate)
    );

    let mut bad_term = lease_input(true);
    bad_term.term_months = 0;
    assert_eq!(
        register_lease_contract(bad_term),
        Err(RealEstatePortfolioError::InvalidTerm)
    );

    let mut zero_deposit = lease_input(true);
    zero_deposit.security_deposit_cents = 0;
    assert_eq!(
        register_lease_contract(zero_deposit),
        Err(RealEstatePortfolioError::InvalidAmount)
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
fn real_estate_refuses_prefix_only_identifiers() {
    let mut bad_property = property_input();
    bad_property.property_id = "prop_".to_owned();
    assert_eq!(
        register_real_estate_object(bad_property),
        Err(RealEstatePortfolioError::InvalidPropertyId)
    );

    let mut bad_tenant = property_input();
    bad_tenant.tenant_id = "ten_".to_owned();
    assert_eq!(
        register_real_estate_object(bad_tenant),
        Err(RealEstatePortfolioError::InvalidTenantId)
    );

    let mut bad_legal_entity = property_input();
    bad_legal_entity.legal_entity_id = "le_".to_owned();
    assert_eq!(
        register_real_estate_object(bad_legal_entity),
        Err(RealEstatePortfolioError::InvalidLegalEntityId)
    );

    let mut bad_business_entity = property_input();
    bad_business_entity.business_entity_id = "be_".to_owned();
    assert_eq!(
        register_real_estate_object(bad_business_entity),
        Err(RealEstatePortfolioError::InvalidBusinessEntityId)
    );

    let mut bad_building = property_input();
    bad_building.building_id = "bldg_".to_owned();
    assert_eq!(
        register_real_estate_object(bad_building),
        Err(RealEstatePortfolioError::InvalidBuildingId)
    );

    let mut bad_rental_object = property_input();
    bad_rental_object.rental_object_id = "rent_".to_owned();
    assert_eq!(
        register_real_estate_object(bad_rental_object),
        Err(RealEstatePortfolioError::InvalidRentalObjectId)
    );

    let mut bad_lease = lease_input(true);
    bad_lease.lease_contract_id = "lease_".to_owned();
    assert_eq!(
        register_lease_contract(bad_lease),
        Err(RealEstatePortfolioError::InvalidLeaseContractId)
    );

    let mut bad_business_partner = lease_input(true);
    bad_business_partner.business_partner_id = "bp_".to_owned();
    assert_eq!(
        register_lease_contract(bad_business_partner),
        Err(RealEstatePortfolioError::InvalidBusinessPartnerId)
    );

    let mut bad_cash_flow = cash_flow_input(true);
    bad_cash_flow.cash_flow_id = "cashflow_".to_owned();
    assert_eq!(
        project_lease_cash_flow(bad_cash_flow),
        Err(RealEstatePortfolioError::InvalidCashFlowId)
    );

    let mut bad_occupancy = occupancy_input(true);
    bad_occupancy.occupancy_plan_id = "occupancy_".to_owned();
    assert_eq!(
        plan_space_occupancy(bad_occupancy),
        Err(RealEstatePortfolioError::InvalidOccupancyPlanId)
    );

    let mut bad_facility = facility_input(true);
    bad_facility.facility_link_id = "facility_".to_owned();
    assert_eq!(
        prepare_facility_maintenance_link(bad_facility),
        Err(RealEstatePortfolioError::InvalidFacilityLinkId)
    );
}

#[test]
fn real_estate_refuses_whitespace_and_control_characters() {
    let mut bad_property = property_input();
    bad_property.tenant_id = "ten_enter prise".to_owned();
    assert_eq!(
        register_real_estate_object(bad_property),
        Err(RealEstatePortfolioError::InvalidTenantId)
    );

    let mut bad_lease = lease_input(true);
    bad_lease.business_partner_id = "bp_anchor\ntenant".to_owned();
    assert_eq!(
        register_lease_contract(bad_lease),
        Err(RealEstatePortfolioError::InvalidBusinessPartnerId)
    );

    let mut bad_occupancy = occupancy_input(true);
    bad_occupancy.team_ref = "team/workplace alpha".to_owned();
    assert_eq!(
        plan_space_occupancy(bad_occupancy),
        Err(RealEstatePortfolioError::InvalidTeamRef)
    );
}

#[test]
fn real_estate_refuses_zero_capacity_and_invalid_period_count() {
    let mut zero_property_capacity = property_input();
    zero_property_capacity.capacity_seats = 0;
    assert_eq!(
        register_real_estate_object(zero_property_capacity),
        Err(RealEstatePortfolioError::InvalidCapacity)
    );

    let mut zero_occupancy_seats = occupancy_input(true);
    zero_occupancy_seats.requested_seats = 0;
    assert_eq!(
        plan_space_occupancy(zero_occupancy_seats),
        Err(RealEstatePortfolioError::InvalidCapacity)
    );

    let mut zero_periods = cash_flow_input(true);
    zero_periods.number_of_periods = 0;
    assert_eq!(
        project_lease_cash_flow(zero_periods),
        Err(RealEstatePortfolioError::InvalidPeriodCount)
    );
}

#[test]
fn real_estate_refuses_type_specific_unsafe_refs() {
    let mut bad_team = occupancy_input(true);
    bad_team.team_ref = "team/../workplace-alpha".to_owned();
    assert_eq!(
        plan_space_occupancy(bad_team),
        Err(RealEstatePortfolioError::InvalidTeamRef)
    );

    let mut bad_asset = facility_input(true);
    bad_asset.maintenance_asset_ref = "asset/secret-token".to_owned();
    assert_eq!(
        prepare_facility_maintenance_link(bad_asset),
        Err(RealEstatePortfolioError::InvalidMaintenanceAssetRef)
    );

    let mut bad_source = lease_input(true);
    bad_source.contract_source_ref = "src/".to_owned();
    assert_eq!(
        register_lease_contract(bad_source),
        Err(RealEstatePortfolioError::InvalidSourceDocumentRef)
    );

    let mut bad_evidence = cash_flow_input(true);
    bad_evidence.cash_flow_evidence_ref = "audit/".to_owned();
    assert_eq!(
        project_lease_cash_flow(bad_evidence),
        Err(RealEstatePortfolioError::InvalidEvidenceRef)
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
