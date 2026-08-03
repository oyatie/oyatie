//! Real-estate portfolio domain foundation.
//!
//! This crate owns pure, metadata-only real-estate invariants for property and
//! rental-object registration, lease-contract metadata, lease cash-flow
//! projection, space-occupancy planning, and facility-maintenance linkage
//! metadata. It does not perform durable persistence, SAP RE-FX/Cloud for Real
//! Estate integration, lease-accounting journal creation, AP/AR/GL posting,
//! payment execution, plant-maintenance work-order creation, workspace/team
//! synchronization, document archive writes, Workflow execution, runtime
//! audit-chain emission, or cloud runtime I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// panic assertions to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const PROPERTY_ID_PREFIX: &str = "prop_";
const BUSINESS_ENTITY_ID_PREFIX: &str = "be_";
const BUILDING_ID_PREFIX: &str = "bldg_";
const RENTAL_OBJECT_ID_PREFIX: &str = "rent_";
const LEASE_CONTRACT_ID_PREFIX: &str = "lease_";
const CASH_FLOW_ID_PREFIX: &str = "cashflow_";
const OCCUPANCY_PLAN_ID_PREFIX: &str = "occupancy_";
const FACILITY_LINK_ID_PREFIX: &str = "facility_";
const BUSINESS_PARTNER_ID_PREFIX: &str = "bp_";
const TEAM_REF_PREFIX: &str = "team/";
const MAINTENANCE_ASSET_REF_PREFIX: &str = "asset/";
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const SOURCE_REF_PREFIX: &str = "src/";
const AUDIT_REF_PREFIX: &str = "audit/";
const REAL_ESTATE_PORTFOLIO_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RealEstatePortfolioDomain;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PropertyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BusinessEntityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BuildingId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RentalObjectId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LeaseContractId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CashFlowId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OccupancyPlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FacilityLinkId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BusinessPartnerId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TenantId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LegalEntityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TeamRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaintenanceAssetRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceDocumentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RealEstateObjectType {
    BusinessEntity,
    Building,
    RentalUnit,
    RentalSpace,
    PooledSpace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RealEstateUsageKind {
    OwnPortfolio,
    LeaseIn,
    LeaseOut,
    MixedUse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PropertyRegistrationState {
    Registered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaseDirection {
    LeaseIn,
    LeaseOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaseAccountingClassification {
    Operating,
    Finance,
    Exempt,
    RevenueOperating,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaseContractState {
    Registered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PaymentFrequency {
    Monthly,
    Quarterly,
    SemiAnnual,
    Annual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CashFlowProjectionState {
    Projected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum OccupancyPlanState {
    Planned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FacilityLinkState {
    Prepared,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FacilityServicePriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealEstateObjectInput {
    pub property_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,           // data_class: INTERNAL_ONLY
    pub business_entity_id: String,        // data_class: INTERNAL_ONLY
    pub building_id: String,               // data_class: INTERNAL_ONLY
    pub rental_object_id: String,          // data_class: INTERNAL_ONLY
    pub object_type: RealEstateObjectType, // data_class: INTERNAL_ONLY
    pub usage_kind: RealEstateUsageKind,   // data_class: INTERNAL_ONLY
    pub gross_area_square_meters: u32,     // data_class: FINANCIAL
    pub rentable_area_square_meters: u32,  // data_class: FINANCIAL
    pub capacity_seats: u32,               // data_class: FINANCIAL
    pub valid_from_yyyymmdd: u32,          // data_class: INTERNAL_ONLY
    pub valid_to_yyyymmdd: u32,            // data_class: INTERNAL_ONLY
    pub object_source_ref: String,         // data_class: INTERNAL_ONLY
    pub registration_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealEstateObjectRegistration {
    pub property_id: Classified<PropertyId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,     // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub business_entity_id: Classified<BusinessEntityId>, // data_class: INTERNAL_ONLY
    pub building_id: Classified<BuildingId>, // data_class: INTERNAL_ONLY
    pub rental_object_id: Classified<RentalObjectId>, // data_class: INTERNAL_ONLY
    pub object_type: Classified<RealEstateObjectType>, // data_class: INTERNAL_ONLY
    pub usage_kind: Classified<RealEstateUsageKind>, // data_class: INTERNAL_ONLY
    pub gross_area_square_meters: Classified<u32>, // data_class: FINANCIAL
    pub rentable_area_square_meters: Classified<u32>, // data_class: FINANCIAL
    pub capacity_seats: Classified<u32>,     // data_class: FINANCIAL
    pub valid_from_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub valid_to_yyyymmdd: Classified<u32>,  // data_class: INTERNAL_ONLY
    pub object_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub registration_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<PropertyRegistrationState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub architectural_view_attached: Classified<bool>, // data_class: PUBLIC
    pub sap_re_fx_backend_attached: Classified<bool>, // data_class: PUBLIC
    pub fixed_asset_master_attached: Classified<bool>, // data_class: PUBLIC
    pub plant_maintenance_functional_location_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseContractInput {
    pub lease_contract_id: String,       // data_class: INTERNAL_ONLY
    pub property_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,         // data_class: INTERNAL_ONLY
    pub business_partner_id: String,     // data_class: INTERNAL_ONLY
    pub property_registered: bool,       // data_class: INTERNAL_ONLY
    pub lease_direction: LeaseDirection, // data_class: INTERNAL_ONLY
    pub accounting_classification: LeaseAccountingClassification, // data_class: INTERNAL_ONLY
    pub commencement_yyyymmdd: u32,      // data_class: INTERNAL_ONLY
    pub expiration_yyyymmdd: u32,        // data_class: INTERNAL_ONLY
    pub term_months: u16,                // data_class: FINANCIAL
    pub monthly_base_rent_cents: u64,    // data_class: FINANCIAL
    pub security_deposit_cents: u64,     // data_class: FINANCIAL
    pub contract_source_ref: String,     // data_class: INTERNAL_ONLY
    pub contract_evidence_ref: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseContractRegistration {
    pub lease_contract_id: Classified<LeaseContractId>, // data_class: INTERNAL_ONLY
    pub property_id: Classified<PropertyId>,            // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,     // data_class: INTERNAL_ONLY
    pub business_partner_id: Classified<BusinessPartnerId>, // data_class: INTERNAL_ONLY
    pub lease_direction: Classified<LeaseDirection>,    // data_class: INTERNAL_ONLY
    pub accounting_classification: Classified<LeaseAccountingClassification>, // data_class: INTERNAL_ONLY
    pub commencement_yyyymmdd: Classified<u32>, // data_class: INTERNAL_ONLY
    pub expiration_yyyymmdd: Classified<u32>,   // data_class: INTERNAL_ONLY
    pub term_months: Classified<u16>,           // data_class: FINANCIAL
    pub monthly_base_rent_cents: Classified<u64>, // data_class: FINANCIAL
    pub security_deposit_cents: Classified<u64>, // data_class: FINANCIAL
    pub total_nominal_rent_cents: Classified<u64>, // data_class: FINANCIAL
    pub contract_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub contract_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<LeaseContractState>,  // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,    // data_class: INTERNAL_ONLY
    pub lease_accounting_engine_attached: Classified<bool>, // data_class: PUBLIC
    pub general_ledger_posting_attached: Classified<bool>, // data_class: PUBLIC
    pub accounts_payable_or_receivable_attached: Classified<bool>, // data_class: PUBLIC
    pub document_archive_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseCashFlowInput {
    pub cash_flow_id: String,                // data_class: INTERNAL_ONLY
    pub lease_contract_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,             // data_class: INTERNAL_ONLY
    pub lease_contract_registered: bool,     // data_class: INTERNAL_ONLY
    pub payment_frequency: PaymentFrequency, // data_class: INTERNAL_ONLY
    pub number_of_periods: u16,              // data_class: FINANCIAL
    pub recurring_payment_cents: u64,        // data_class: FINANCIAL
    pub first_due_yyyymmdd: u32,             // data_class: INTERNAL_ONLY
    pub final_due_yyyymmdd: u32,             // data_class: INTERNAL_ONLY
    pub cash_flow_source_ref: String,        // data_class: INTERNAL_ONLY
    pub cash_flow_evidence_ref: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseCashFlowProjection {
    pub cash_flow_id: Classified<CashFlowId>, // data_class: INTERNAL_ONLY
    pub lease_contract_id: Classified<LeaseContractId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,      // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub payment_frequency: Classified<PaymentFrequency>, // data_class: INTERNAL_ONLY
    pub number_of_periods: Classified<u16>,   // data_class: FINANCIAL
    pub recurring_payment_cents: Classified<u64>, // data_class: FINANCIAL
    pub projected_total_cash_flow_cents: Classified<u64>, // data_class: FINANCIAL
    pub first_due_yyyymmdd: Classified<u32>,  // data_class: INTERNAL_ONLY
    pub final_due_yyyymmdd: Classified<u32>,  // data_class: INTERNAL_ONLY
    pub cash_flow_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub cash_flow_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<CashFlowProjectionState>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,  // data_class: INTERNAL_ONLY
    pub periodic_posting_attached: Classified<bool>, // data_class: PUBLIC
    pub payment_run_attached: Classified<bool>, // data_class: PUBLIC
    pub subledger_accounting_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpaceOccupancyInput {
    pub occupancy_plan_id: String,              // data_class: INTERNAL_ONLY
    pub property_id: String,                    // data_class: INTERNAL_ONLY
    pub rental_object_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                // data_class: INTERNAL_ONLY
    pub property_registered: bool,              // data_class: INTERNAL_ONLY
    pub total_rentable_area_square_meters: u32, // data_class: FINANCIAL
    pub already_committed_area_square_meters: u32, // data_class: FINANCIAL
    pub requested_area_square_meters: u32,      // data_class: FINANCIAL
    pub requested_seats: u32,                   // data_class: FINANCIAL
    pub team_ref: String,                       // data_class: INTERNAL_ONLY
    pub occupancy_start_yyyymmdd: u32,          // data_class: INTERNAL_ONLY
    pub occupancy_end_yyyymmdd: u32,            // data_class: INTERNAL_ONLY
    pub occupancy_source_ref: String,           // data_class: INTERNAL_ONLY
    pub occupancy_evidence_ref: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpaceOccupancyPlan {
    pub occupancy_plan_id: Classified<OccupancyPlanId>, // data_class: INTERNAL_ONLY
    pub property_id: Classified<PropertyId>,            // data_class: INTERNAL_ONLY
    pub rental_object_id: Classified<RentalObjectId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,                // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,     // data_class: INTERNAL_ONLY
    pub total_rentable_area_square_meters: Classified<u32>, // data_class: FINANCIAL
    pub already_committed_area_square_meters: Classified<u32>, // data_class: FINANCIAL
    pub requested_area_square_meters: Classified<u32>,  // data_class: FINANCIAL
    pub remaining_area_square_meters: Classified<u32>,  // data_class: FINANCIAL
    pub requested_seats: Classified<u32>,               // data_class: FINANCIAL
    pub team_ref: Classified<TeamRef>,                  // data_class: INTERNAL_ONLY
    pub occupancy_start_yyyymmdd: Classified<u32>,      // data_class: INTERNAL_ONLY
    pub occupancy_end_yyyymmdd: Classified<u32>,        // data_class: INTERNAL_ONLY
    pub occupancy_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub occupancy_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<OccupancyPlanState>,          // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,            // data_class: INTERNAL_ONLY
    pub area_capacity_sufficient: Classified<bool>,     // data_class: PUBLIC
    pub workspace_runtime_attached: Classified<bool>,   // data_class: PUBLIC
    pub team_directory_sync_attached: Classified<bool>, // data_class: PUBLIC
    pub reservation_mutation_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,    // data_class: PUBLIC
    pub schema_version: Classified<u32>,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacilityMaintenanceLinkInput {
    pub facility_link_id: String,      // data_class: INTERNAL_ONLY
    pub property_id: String,           // data_class: INTERNAL_ONLY
    pub rental_object_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,       // data_class: INTERNAL_ONLY
    pub property_registered: bool,     // data_class: INTERNAL_ONLY
    pub maintenance_asset_ref: String, // data_class: INTERNAL_ONLY
    pub service_priority: FacilityServicePriority, // data_class: INTERNAL_ONLY
    pub planned_window_days: u16,      // data_class: INTERNAL_ONLY
    pub facility_source_ref: String,   // data_class: INTERNAL_ONLY
    pub facility_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacilityMaintenanceLinkPreparation {
    pub facility_link_id: Classified<FacilityLinkId>, // data_class: INTERNAL_ONLY
    pub property_id: Classified<PropertyId>,          // data_class: INTERNAL_ONLY
    pub rental_object_id: Classified<RentalObjectId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,   // data_class: INTERNAL_ONLY
    pub maintenance_asset_ref: Classified<MaintenanceAssetRef>, // data_class: INTERNAL_ONLY
    pub service_priority: Classified<FacilityServicePriority>, // data_class: INTERNAL_ONLY
    pub planned_window_days: Classified<u16>,         // data_class: INTERNAL_ONLY
    pub facility_source_ref: Classified<SourceDocumentRef>, // data_class: INTERNAL_ONLY
    pub facility_evidence_ref: Classified<EvidenceRef>, // data_class: INTERNAL_ONLY
    pub state: Classified<FacilityLinkState>,         // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,          // data_class: INTERNAL_ONLY
    pub plant_maintenance_order_attached: Classified<bool>, // data_class: PUBLIC
    pub iot_or_scada_ingestion_attached: Classified<bool>, // data_class: PUBLIC
    pub service_ticket_runtime_attached: Classified<bool>, // data_class: PUBLIC
    pub workflow_execution_attached: Classified<bool>, // data_class: PUBLIC
    pub cloud_deployment_attached: Classified<bool>,  // data_class: PUBLIC
    pub schema_version: Classified<u32>,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RealEstatePortfolioError {
    InvalidPropertyId,
    InvalidBusinessEntityId,
    InvalidBuildingId,
    InvalidRentalObjectId,
    InvalidLeaseContractId,
    InvalidCashFlowId,
    InvalidOccupancyPlanId,
    InvalidFacilityLinkId,
    InvalidBusinessPartnerId,
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidTeamRef,
    InvalidMaintenanceAssetRef,
    InvalidSourceDocumentRef,
    InvalidEvidenceRef,
    InvalidDate,
    InvalidArea,
    InvalidCapacity,
    InvalidTerm,
    InvalidAmount,
    InvalidPeriodCount,
    InvalidWindow,
    PropertyRegistrationRequired,
    LeaseContractRequired,
    AreaCapacityExceeded,
}

pub fn register_real_estate_object(
    input: RealEstateObjectInput,
) -> Result<RealEstateObjectRegistration, RealEstatePortfolioError> {
    validate_property_id(&input.property_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_business_entity_id(&input.business_entity_id)?;
    validate_building_id(&input.building_id)?;
    validate_rental_object_id(&input.rental_object_id)?;
    validate_area(input.gross_area_square_meters)?;
    validate_area(input.rentable_area_square_meters)?;
    if input.rentable_area_square_meters > input.gross_area_square_meters {
        return Err(RealEstatePortfolioError::InvalidArea);
    }
    validate_capacity(input.capacity_seats)?;
    validate_date_range(input.valid_from_yyyymmdd, input.valid_to_yyyymmdd)?;
    validate_source_ref(&input.object_source_ref)?;
    validate_evidence_ref(&input.registration_evidence_ref)?;
    let idempotency_key = format!(
        "real-estate:object:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.property_id
    );

    Ok(RealEstateObjectRegistration {
        property_id: internal(PropertyId {
            value: input.property_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        business_entity_id: internal(BusinessEntityId {
            value: input.business_entity_id,
        }),
        building_id: internal(BuildingId {
            value: input.building_id,
        }),
        rental_object_id: internal(RentalObjectId {
            value: input.rental_object_id,
        }),
        object_type: internal(input.object_type),
        usage_kind: internal(input.usage_kind),
        gross_area_square_meters: financial(input.gross_area_square_meters),
        rentable_area_square_meters: financial(input.rentable_area_square_meters),
        capacity_seats: financial(input.capacity_seats),
        valid_from_yyyymmdd: internal(input.valid_from_yyyymmdd),
        valid_to_yyyymmdd: internal(input.valid_to_yyyymmdd),
        object_source_ref: internal(SourceDocumentRef {
            value: input.object_source_ref,
        }),
        registration_evidence_ref: internal(EvidenceRef {
            value: input.registration_evidence_ref,
        }),
        state: internal(PropertyRegistrationState::Registered),
        idempotency_key: internal(idempotency_key),
        architectural_view_attached: public(false),
        sap_re_fx_backend_attached: public(false),
        fixed_asset_master_attached: public(false),
        plant_maintenance_functional_location_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(REAL_ESTATE_PORTFOLIO_SCHEMA_VERSION),
    })
}

pub fn register_lease_contract(
    input: LeaseContractInput,
) -> Result<LeaseContractRegistration, RealEstatePortfolioError> {
    validate_lease_contract_id(&input.lease_contract_id)?;
    validate_property_id(&input.property_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    validate_business_partner_id(&input.business_partner_id)?;
    if !input.property_registered {
        return Err(RealEstatePortfolioError::PropertyRegistrationRequired);
    }
    validate_date_range(input.commencement_yyyymmdd, input.expiration_yyyymmdd)?;
    validate_term(input.term_months)?;
    validate_positive_amount(input.monthly_base_rent_cents)?;
    validate_source_ref(&input.contract_source_ref)?;
    validate_evidence_ref(&input.contract_evidence_ref)?;
    let total_nominal_rent_cents = input
        .monthly_base_rent_cents
        .checked_mul(u64::from(input.term_months))
        .ok_or(RealEstatePortfolioError::InvalidAmount)?;
    let idempotency_key = format!(
        "real-estate:lease:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.lease_contract_id
    );

    Ok(LeaseContractRegistration {
        lease_contract_id: internal(LeaseContractId {
            value: input.lease_contract_id,
        }),
        property_id: internal(PropertyId {
            value: input.property_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        business_partner_id: internal(BusinessPartnerId {
            value: input.business_partner_id,
        }),
        lease_direction: internal(input.lease_direction),
        accounting_classification: internal(input.accounting_classification),
        commencement_yyyymmdd: internal(input.commencement_yyyymmdd),
        expiration_yyyymmdd: internal(input.expiration_yyyymmdd),
        term_months: financial(input.term_months),
        monthly_base_rent_cents: financial(input.monthly_base_rent_cents),
        security_deposit_cents: financial(input.security_deposit_cents),
        total_nominal_rent_cents: financial(total_nominal_rent_cents),
        contract_source_ref: internal(SourceDocumentRef {
            value: input.contract_source_ref,
        }),
        contract_evidence_ref: internal(EvidenceRef {
            value: input.contract_evidence_ref,
        }),
        state: internal(LeaseContractState::Registered),
        idempotency_key: internal(idempotency_key),
        lease_accounting_engine_attached: public(false),
        general_ledger_posting_attached: public(false),
        accounts_payable_or_receivable_attached: public(false),
        document_archive_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(REAL_ESTATE_PORTFOLIO_SCHEMA_VERSION),
    })
}

pub fn project_lease_cash_flow(
    input: LeaseCashFlowInput,
) -> Result<LeaseCashFlowProjection, RealEstatePortfolioError> {
    validate_cash_flow_id(&input.cash_flow_id)?;
    validate_lease_contract_id(&input.lease_contract_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    if !input.lease_contract_registered {
        return Err(RealEstatePortfolioError::LeaseContractRequired);
    }
    validate_period_count(input.number_of_periods)?;
    validate_positive_amount(input.recurring_payment_cents)?;
    validate_date_range(input.first_due_yyyymmdd, input.final_due_yyyymmdd)?;
    validate_source_ref(&input.cash_flow_source_ref)?;
    validate_evidence_ref(&input.cash_flow_evidence_ref)?;
    let projected_total_cash_flow_cents = input
        .recurring_payment_cents
        .checked_mul(u64::from(input.number_of_periods))
        .ok_or(RealEstatePortfolioError::InvalidAmount)?;
    let idempotency_key = format!(
        "real-estate:cash-flow:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.cash_flow_id
    );

    Ok(LeaseCashFlowProjection {
        cash_flow_id: internal(CashFlowId {
            value: input.cash_flow_id,
        }),
        lease_contract_id: internal(LeaseContractId {
            value: input.lease_contract_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        payment_frequency: internal(input.payment_frequency),
        number_of_periods: financial(input.number_of_periods),
        recurring_payment_cents: financial(input.recurring_payment_cents),
        projected_total_cash_flow_cents: financial(projected_total_cash_flow_cents),
        first_due_yyyymmdd: internal(input.first_due_yyyymmdd),
        final_due_yyyymmdd: internal(input.final_due_yyyymmdd),
        cash_flow_source_ref: internal(SourceDocumentRef {
            value: input.cash_flow_source_ref,
        }),
        cash_flow_evidence_ref: internal(EvidenceRef {
            value: input.cash_flow_evidence_ref,
        }),
        state: internal(CashFlowProjectionState::Projected),
        idempotency_key: internal(idempotency_key),
        periodic_posting_attached: public(false),
        payment_run_attached: public(false),
        subledger_accounting_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(REAL_ESTATE_PORTFOLIO_SCHEMA_VERSION),
    })
}

pub fn plan_space_occupancy(
    input: SpaceOccupancyInput,
) -> Result<SpaceOccupancyPlan, RealEstatePortfolioError> {
    validate_occupancy_plan_id(&input.occupancy_plan_id)?;
    validate_property_id(&input.property_id)?;
    validate_rental_object_id(&input.rental_object_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    if !input.property_registered {
        return Err(RealEstatePortfolioError::PropertyRegistrationRequired);
    }
    validate_area(input.total_rentable_area_square_meters)?;
    validate_area(input.requested_area_square_meters)?;
    validate_capacity(input.requested_seats)?;
    let available_area = input
        .total_rentable_area_square_meters
        .checked_sub(input.already_committed_area_square_meters)
        .ok_or(RealEstatePortfolioError::InvalidArea)?;
    if input.requested_area_square_meters > available_area {
        return Err(RealEstatePortfolioError::AreaCapacityExceeded);
    }
    validate_team_ref(&input.team_ref)?;
    validate_date_range(input.occupancy_start_yyyymmdd, input.occupancy_end_yyyymmdd)?;
    validate_source_ref(&input.occupancy_source_ref)?;
    validate_evidence_ref(&input.occupancy_evidence_ref)?;
    let remaining_area_square_meters = available_area - input.requested_area_square_meters;
    let idempotency_key = format!(
        "real-estate:occupancy:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.occupancy_plan_id
    );

    Ok(SpaceOccupancyPlan {
        occupancy_plan_id: internal(OccupancyPlanId {
            value: input.occupancy_plan_id,
        }),
        property_id: internal(PropertyId {
            value: input.property_id,
        }),
        rental_object_id: internal(RentalObjectId {
            value: input.rental_object_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        total_rentable_area_square_meters: financial(input.total_rentable_area_square_meters),
        already_committed_area_square_meters: financial(input.already_committed_area_square_meters),
        requested_area_square_meters: financial(input.requested_area_square_meters),
        remaining_area_square_meters: financial(remaining_area_square_meters),
        requested_seats: financial(input.requested_seats),
        team_ref: internal(TeamRef {
            value: input.team_ref,
        }),
        occupancy_start_yyyymmdd: internal(input.occupancy_start_yyyymmdd),
        occupancy_end_yyyymmdd: internal(input.occupancy_end_yyyymmdd),
        occupancy_source_ref: internal(SourceDocumentRef {
            value: input.occupancy_source_ref,
        }),
        occupancy_evidence_ref: internal(EvidenceRef {
            value: input.occupancy_evidence_ref,
        }),
        state: internal(OccupancyPlanState::Planned),
        idempotency_key: internal(idempotency_key),
        area_capacity_sufficient: public(true),
        workspace_runtime_attached: public(false),
        team_directory_sync_attached: public(false),
        reservation_mutation_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(REAL_ESTATE_PORTFOLIO_SCHEMA_VERSION),
    })
}

pub fn prepare_facility_maintenance_link(
    input: FacilityMaintenanceLinkInput,
) -> Result<FacilityMaintenanceLinkPreparation, RealEstatePortfolioError> {
    validate_facility_link_id(&input.facility_link_id)?;
    validate_property_id(&input.property_id)?;
    validate_rental_object_id(&input.rental_object_id)?;
    validate_tenant_id(&input.tenant_id)?;
    validate_legal_entity_id(&input.legal_entity_id)?;
    if !input.property_registered {
        return Err(RealEstatePortfolioError::PropertyRegistrationRequired);
    }
    validate_maintenance_asset_ref(&input.maintenance_asset_ref)?;
    validate_window(input.planned_window_days)?;
    validate_source_ref(&input.facility_source_ref)?;
    validate_evidence_ref(&input.facility_evidence_ref)?;
    let idempotency_key = format!(
        "real-estate:facility:{}:{}:{}",
        input.tenant_id, input.legal_entity_id, input.facility_link_id
    );

    Ok(FacilityMaintenanceLinkPreparation {
        facility_link_id: internal(FacilityLinkId {
            value: input.facility_link_id,
        }),
        property_id: internal(PropertyId {
            value: input.property_id,
        }),
        rental_object_id: internal(RentalObjectId {
            value: input.rental_object_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        maintenance_asset_ref: internal(MaintenanceAssetRef {
            value: input.maintenance_asset_ref,
        }),
        service_priority: internal(input.service_priority),
        planned_window_days: internal(input.planned_window_days),
        facility_source_ref: internal(SourceDocumentRef {
            value: input.facility_source_ref,
        }),
        facility_evidence_ref: internal(EvidenceRef {
            value: input.facility_evidence_ref,
        }),
        state: internal(FacilityLinkState::Prepared),
        idempotency_key: internal(idempotency_key),
        plant_maintenance_order_attached: public(false),
        iot_or_scada_ingestion_attached: public(false),
        service_ticket_runtime_attached: public(false),
        workflow_execution_attached: public(false),
        cloud_deployment_attached: public(false),
        schema_version: public(REAL_ESTATE_PORTFOLIO_SCHEMA_VERSION),
    })
}

fn validate_property_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        PROPERTY_ID_PREFIX,
        RealEstatePortfolioError::InvalidPropertyId,
    )
}

fn validate_business_entity_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        BUSINESS_ENTITY_ID_PREFIX,
        RealEstatePortfolioError::InvalidBusinessEntityId,
    )
}

fn validate_building_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        BUILDING_ID_PREFIX,
        RealEstatePortfolioError::InvalidBuildingId,
    )
}

fn validate_rental_object_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        RENTAL_OBJECT_ID_PREFIX,
        RealEstatePortfolioError::InvalidRentalObjectId,
    )
}

fn validate_lease_contract_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        LEASE_CONTRACT_ID_PREFIX,
        RealEstatePortfolioError::InvalidLeaseContractId,
    )
}

fn validate_cash_flow_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        CASH_FLOW_ID_PREFIX,
        RealEstatePortfolioError::InvalidCashFlowId,
    )
}

fn validate_occupancy_plan_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        OCCUPANCY_PLAN_ID_PREFIX,
        RealEstatePortfolioError::InvalidOccupancyPlanId,
    )
}

fn validate_facility_link_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        FACILITY_LINK_ID_PREFIX,
        RealEstatePortfolioError::InvalidFacilityLinkId,
    )
}

fn validate_business_partner_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        BUSINESS_PARTNER_ID_PREFIX,
        RealEstatePortfolioError::InvalidBusinessPartnerId,
    )
}

fn validate_tenant_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        TENANT_ID_PREFIX,
        RealEstatePortfolioError::InvalidTenantId,
    )
}

fn validate_legal_entity_id(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_id(
        value,
        LEGAL_ENTITY_ID_PREFIX,
        RealEstatePortfolioError::InvalidLegalEntityId,
    )
}

fn validate_id(
    value: &str,
    prefix: &str,
    error: RealEstatePortfolioError,
) -> Result<(), RealEstatePortfolioError> {
    if !value.starts_with(prefix) || value.len() <= prefix.len() || has_unsafe_text(value) {
        return Err(error);
    }
    if has_path_traversal(value) || has_credential_shape(value) {
        return Err(error);
    }
    Ok(())
}

fn validate_team_ref(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_ref(
        value,
        TEAM_REF_PREFIX,
        RealEstatePortfolioError::InvalidTeamRef,
    )
}

fn validate_maintenance_asset_ref(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_ref(
        value,
        MAINTENANCE_ASSET_REF_PREFIX,
        RealEstatePortfolioError::InvalidMaintenanceAssetRef,
    )
}

fn validate_source_ref(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_ref(
        value,
        SOURCE_REF_PREFIX,
        RealEstatePortfolioError::InvalidSourceDocumentRef,
    )
}

fn validate_evidence_ref(value: &str) -> Result<(), RealEstatePortfolioError> {
    validate_ref(
        value,
        AUDIT_REF_PREFIX,
        RealEstatePortfolioError::InvalidEvidenceRef,
    )
}

fn validate_ref(
    value: &str,
    prefix: &str,
    error: RealEstatePortfolioError,
) -> Result<(), RealEstatePortfolioError> {
    if !value.starts_with(prefix)
        || value.len() <= prefix.len()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(error);
    }
    Ok(())
}

fn validate_area(value: u32) -> Result<(), RealEstatePortfolioError> {
    if value == 0 {
        return Err(RealEstatePortfolioError::InvalidArea);
    }
    Ok(())
}

fn validate_capacity(value: u32) -> Result<(), RealEstatePortfolioError> {
    if value == 0 {
        return Err(RealEstatePortfolioError::InvalidCapacity);
    }
    Ok(())
}

fn validate_term(value: u16) -> Result<(), RealEstatePortfolioError> {
    if value == 0 || value > 1_200 {
        return Err(RealEstatePortfolioError::InvalidTerm);
    }
    Ok(())
}

fn validate_positive_amount(value: u64) -> Result<(), RealEstatePortfolioError> {
    if value == 0 {
        return Err(RealEstatePortfolioError::InvalidAmount);
    }
    Ok(())
}

fn validate_period_count(value: u16) -> Result<(), RealEstatePortfolioError> {
    if value == 0 || value > 1_200 {
        return Err(RealEstatePortfolioError::InvalidPeriodCount);
    }
    Ok(())
}

fn validate_window(value: u16) -> Result<(), RealEstatePortfolioError> {
    if value == 0 || value > 366 {
        return Err(RealEstatePortfolioError::InvalidWindow);
    }
    Ok(())
}

fn validate_date_range(start: u32, end: u32) -> Result<(), RealEstatePortfolioError> {
    validate_yyyymmdd(start)?;
    validate_yyyymmdd(end)?;
    if start > end {
        return Err(RealEstatePortfolioError::InvalidDate);
    }
    Ok(())
}

fn validate_yyyymmdd(value: u32) -> Result<(), RealEstatePortfolioError> {
    let year = value / 10_000;
    let month = (value / 100) % 100;
    let day = value % 100;
    if !(2020..=2100).contains(&year) || !(1..=12).contains(&month) {
        return Err(RealEstatePortfolioError::InvalidDate);
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return Err(RealEstatePortfolioError::InvalidDate),
    };
    if day == 0 || day > max_day {
        return Err(RealEstatePortfolioError::InvalidDate);
    }
    Ok(())
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('\\') || value.contains("//")
}

fn has_credential_shape(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("api_key")
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}
