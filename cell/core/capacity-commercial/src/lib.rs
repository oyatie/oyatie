//! Cloud capacity commercial-control kernel.
//!
//! This crate owns the stable purchase and capacity contracts for reserved
//! capacity, committed-use discounts, spot/preemptible capacity, and controlled
//! cell rebalancing. Compute keeps runtime metadata; billing keeps invoices;
//! this kernel keeps the capacity promise enforceable across public-cloud,
//! colo, and own-datacenter phases.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use billing_domain::Money;
use cell_region::{CellId, RegionCode};
use compute_resource::InstanceFlavor;
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use billing_metering::{
    AxisId, Meter, MeterEvent, MeterEventCreate, MeterUnit, MeteringError, PlaneTag,
};

const CAPACITY_SCHEMA_VERSION: u32 = 1;
pub const CLOUD_OPS_FOUNDATION_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const SKU_ID_PREFIX: &str = "csku_";
const RESERVATION_ID_PREFIX: &str = "cres_";
const COMMITMENT_ID_PREFIX: &str = "cuc_";
const SPOT_POOL_ID_PREFIX: &str = "spot_";
const SPOT_ASSIGNMENT_ID_PREFIX: &str = "spota_";
const REBALANCE_PLAN_ID_PREFIX: &str = "crb_";
const APPROVAL_REF_PREFIX: &str = "approval/";
pub const OPS_CAPACITY_EVIDENCE_PREFIX: &str = "evidence/cloud-ops/capacity/";
pub const OPS_CELL_EVIDENCE_PREFIX: &str = "evidence/cloud-ops/cell/";
pub const OPS_DCOPS_EVIDENCE_PREFIX: &str = "evidence/cloud-ops/dcops/";
pub const OPS_FINOPS_EVIDENCE_PREFIX: &str = "evidence/cloud-ops/finops/";
pub const OPS_MARKETPLACE_EVIDENCE_PREFIX: &str = "evidence/cloud-ops/marketplace/";
pub const OPS_FSH_EVIDENCE_PREFIX: &str = "evidence/cloud-ops/fsh/";
pub const OPS_AUDIT_CHAIN_PREFIX: &str = "audit-chain/cloud-ops/";
const CAPACITY_METER_CAPABILITY_ID: &str = "cap.cloud.capacity.commercial";
pub const REQUIRED_STABLE_HEADROOM_BPS: u16 = 3_000;
pub const MIN_SPOT_INTERRUPTION_NOTICE_SECONDS: u32 = 120;
pub const MAX_REBALANCE_MOVE_BPS: u16 = 1_000;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CapacitySkuId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CapacityReservationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CommitmentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SpotPoolId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SpotAssignmentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RebalancePlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApprovalRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CapacityUnits {
    pub vcpu: u32,         // data_class: PUBLIC
    pub memory_gb: u32,    // data_class: PUBLIC
    pub gpu_count: u32,    // data_class: PUBLIC
    pub local_ssd_gb: u32, // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellCapacityEnvelope {
    pub total: CapacityUnits,         // data_class: INTERNAL_ONLY
    pub allocated: CapacityUnits,     // data_class: INTERNAL_ONLY
    pub reserved: CapacityUnits,      // data_class: INTERNAL_ONLY
    pub spot_assigned: CapacityUnits, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CapacityTermMonths {
    One,
    Twelve,
    ThirtySix,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CapacityReservationState {
    Active,
    Expired,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CommitmentState {
    Active,
    Expired,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SpotPoolState {
    Open,
    Draining,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SpotAssignmentState {
    Active,
    Preempting,
    Terminated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RebalancePlanState {
    Proposed,
    Approved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapacitySkuCreate {
    pub id: String,                    // data_class: PUBLIC
    pub region: String,                // data_class: PUBLIC
    pub cell_id: String,               // data_class: PUBLIC
    pub flavor: InstanceFlavor,        // data_class: PUBLIC
    pub unit: CapacityUnits,           // data_class: PUBLIC
    pub hourly_price: Money,           // data_class: FINANCIAL
    pub data_class: DataClass,         // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapacitySku {
    pub id: Classified<CapacitySkuId>,             // data_class: PUBLIC
    pub region: Classified<RegionCode>,            // data_class: PUBLIC
    pub cell_id: Classified<CellId>,               // data_class: PUBLIC
    pub flavor: Classified<InstanceFlavor>,        // data_class: PUBLIC
    pub unit: Classified<CapacityUnits>,           // data_class: PUBLIC
    pub hourly_price: Classified<Money>,           // data_class: FINANCIAL
    pub data_class: Classified<PrivacyDataClass>,  // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapacityReservationCreate {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub sku_id: String,                  // data_class: PUBLIC
    pub units: CapacityUnits,            // data_class: PUBLIC
    pub term_months: CapacityTermMonths, // data_class: PUBLIC
    pub start_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapacityReservation {
    pub id: Classified<CapacityReservationId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub sku_id: Classified<CapacitySkuId>,     // data_class: PUBLIC
    pub region: Classified<RegionCode>,        // data_class: PUBLIC
    pub cell_id: Classified<CellId>,           // data_class: PUBLIC
    pub units: Classified<CapacityUnits>,      // data_class: PUBLIC
    pub term_months: Classified<CapacityTermMonths>, // data_class: PUBLIC
    pub state: Classified<CapacityReservationState>, // data_class: INTERNAL_ONLY
    pub start_epoch_seconds: Classified<u64>,  // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: Classified<u64>,    // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: FINANCIAL
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedUseCreate {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub region: String,                  // data_class: PUBLIC
    pub term_months: CapacityTermMonths, // data_class: PUBLIC
    pub spend_commitment: Money,         // data_class: FINANCIAL
    pub discount_bps: u16,               // data_class: FINANCIAL
    pub start_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedUseContract {
    pub id: Classified<CommitmentId>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub term_months: Classified<CapacityTermMonths>, // data_class: PUBLIC
    pub spend_commitment: Classified<Money>, // data_class: FINANCIAL
    pub discount_bps: Classified<u16>,  // data_class: FINANCIAL
    pub state: Classified<CommitmentState>, // data_class: INTERNAL_ONLY
    pub start_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: FINANCIAL
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpotPoolCreate {
    pub id: String,                       // data_class: INTERNAL_ONLY
    pub sku_id: String,                   // data_class: PUBLIC
    pub available_units: CapacityUnits,   // data_class: PUBLIC
    pub current_price: Money,             // data_class: FINANCIAL
    pub interruption_notice_seconds: u32, // data_class: PUBLIC
    pub data_class: DataClass,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpotPool {
    pub id: Classified<SpotPoolId>,        // data_class: INTERNAL_ONLY
    pub sku_id: Classified<CapacitySkuId>, // data_class: PUBLIC
    pub region: Classified<RegionCode>,    // data_class: PUBLIC
    pub cell_id: Classified<CellId>,       // data_class: PUBLIC
    pub available_units: Classified<CapacityUnits>, // data_class: PUBLIC
    pub current_price: Classified<Money>,  // data_class: FINANCIAL
    pub interruption_notice_seconds: Classified<u32>, // data_class: PUBLIC
    pub state: Classified<SpotPoolState>,  // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpotAssignmentCreate {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub spot_pool_id: String,            // data_class: INTERNAL_ONLY
    pub units: CapacityUnits,            // data_class: PUBLIC
    pub max_price: Money,                // data_class: FINANCIAL
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpotAssignment {
    pub id: Classified<SpotAssignmentId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub spot_pool_id: Classified<SpotPoolId>, // data_class: INTERNAL_ONLY
    pub sku_id: Classified<CapacitySkuId>, // data_class: PUBLIC
    pub region: Classified<RegionCode>,   // data_class: PUBLIC
    pub cell_id: Classified<CellId>,      // data_class: PUBLIC
    pub units: Classified<CapacityUnits>, // data_class: PUBLIC
    pub max_price: Classified<Money>,     // data_class: FINANCIAL
    pub state: Classified<SpotAssignmentState>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: FINANCIAL
    pub schema_version: Classified<u32>,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalancePlanCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub source_cell_id: String,        // data_class: PUBLIC
    pub target_cell_id: String,        // data_class: PUBLIC
    pub moved_units: CapacityUnits,    // data_class: PUBLIC
    pub source_total: CapacityUnits,   // data_class: INTERNAL_ONLY
    pub approval_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalancePlan {
    pub id: Classified<RebalancePlanId>, // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,  // data_class: PUBLIC
    pub source_cell_id: Classified<CellId>, // data_class: PUBLIC
    pub target_cell_id: Classified<CellId>, // data_class: PUBLIC
    pub moved_units: Classified<CapacityUnits>, // data_class: PUBLIC
    pub move_bps: Classified<u16>,       // data_class: INTERNAL_ONLY
    pub approval_ref: Classified<Option<ApprovalRef>>, // data_class: INTERNAL_ONLY
    pub state: Classified<RebalancePlanState>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapacityMeterCreate {
    pub meter_event_id: String,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub units: Vec<MeterUnit>,          // data_class: INTERNAL_ONLY
    pub recorded_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudOpsFoundationGuardrailCreate {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub region: String,                                 // data_class: PUBLIC
    pub cell_id: String,                                // data_class: PUBLIC
    pub stable_capacity_envelope: CellCapacityEnvelope, // data_class: INTERNAL_ONLY
    pub stable_reservation_units: CapacityUnits,        // data_class: PUBLIC
    pub rebalance_source_total: CapacityUnits,          // data_class: INTERNAL_ONLY
    pub rebalance_move_units: CapacityUnits,            // data_class: PUBLIC
    pub capacity_evidence_ref: String,                  // data_class: AUDIT
    pub cell_lifecycle_evidence_ref: String,            // data_class: AUDIT
    pub dcops_evidence_ref: String,                     // data_class: AUDIT
    pub finops_evidence_ref: String,                    // data_class: AUDIT
    pub marketplace_evidence_ref: String,               // data_class: AUDIT
    pub filesystem_handoff_evidence_ref: String,        // data_class: AUDIT
    pub audit_chain_ref: String,                        // data_class: AUDIT
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudOpsFoundationGuardrail {
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub cell_id: Classified<CellId>,    // data_class: PUBLIC
    pub stable_capacity_envelope: Classified<CellCapacityEnvelope>, // data_class: INTERNAL_ONLY
    pub stable_reservation_units: Classified<CapacityUnits>, // data_class: PUBLIC
    pub stable_headroom_bps: Classified<u16>, // data_class: INTERNAL_ONLY
    pub rebalance_source_total: Classified<CapacityUnits>, // data_class: INTERNAL_ONLY
    pub rebalance_move_units: Classified<CapacityUnits>, // data_class: PUBLIC
    pub rebalance_move_bps: Classified<u16>, // data_class: INTERNAL_ONLY
    pub capacity_evidence_ref: Classified<String>, // data_class: AUDIT
    pub cell_lifecycle_evidence_ref: Classified<String>, // data_class: AUDIT
    pub dcops_evidence_ref: Classified<String>, // data_class: AUDIT
    pub finops_evidence_ref: Classified<String>, // data_class: AUDIT
    pub marketplace_evidence_ref: Classified<String>, // data_class: AUDIT
    pub filesystem_handoff_evidence_ref: Classified<String>, // data_class: AUDIT
    pub audit_chain_ref: Classified<String>, // data_class: AUDIT
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudCapacityError {
    InvalidSkuId,
    InvalidReservationId,
    InvalidCommitmentId,
    InvalidSpotPoolId,
    InvalidSpotAssignmentId,
    InvalidRebalancePlanId,
    InvalidApprovalRef,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidCapacityUnits,
    InvalidSkuUnitShape,
    InvalidHeadroom,
    InvalidTerm,
    InvalidTimeOrder,
    InvalidMoney,
    InvalidDiscount,
    InvalidInterruptionNotice,
    InvalidSpotPrice,
    InvalidSpotCapacity,
    InvalidDataClass,
    InvalidRebalanceMove,
    InvalidOpsEvidenceRef,
    InvalidAuditChainRef,
    UnknownSku,
    UnknownSpotPool,
    DuplicateSku,
    DuplicateReservation,
    DuplicateCommitment,
    DuplicateSpotPool,
    DuplicateSpotAssignment,
    DuplicateRebalancePlan,
    MeteringRejected,
    InvalidTransition,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudCapacityCatalog {
    skus: BTreeMap<CapacitySkuId, CapacitySku>,
    reservations: BTreeMap<CapacityReservationId, CapacityReservation>,
    commitments: BTreeMap<CommitmentId, CommittedUseContract>,
    spot_pools: BTreeMap<SpotPoolId, SpotPool>,
    spot_assignments: BTreeMap<SpotAssignmentId, SpotAssignment>,
    rebalance_plans: BTreeMap<RebalancePlanId, RebalancePlan>,
    meter: Meter,
}

pub trait CapacityRepo {
    fn register_sku(&mut self, input: CapacitySkuCreate)
    -> Result<CapacitySku, CloudCapacityError>;
    fn purchase_reservation(
        &mut self,
        cell_capacity: CellCapacityEnvelope,
        input: CapacityReservationCreate,
    ) -> Result<CapacityReservation, CloudCapacityError>;
    fn purchase_commitment(
        &mut self,
        input: CommittedUseCreate,
    ) -> Result<CommittedUseContract, CloudCapacityError>;
    fn open_spot_pool(&mut self, input: SpotPoolCreate) -> Result<SpotPool, CloudCapacityError>;
    fn assign_spot_capacity(
        &mut self,
        input: SpotAssignmentCreate,
    ) -> Result<SpotAssignment, CloudCapacityError>;
    fn propose_rebalance(
        &mut self,
        input: RebalancePlanCreate,
    ) -> Result<RebalancePlan, CloudCapacityError>;
    fn record_capacity_meter(
        &mut self,
        input: CapacityMeterCreate,
    ) -> Result<MeterEvent, CloudCapacityError>;
}

impl CapacityUnits {
    pub const fn zero() -> Self {
        Self {
            vcpu: 0,
            memory_gb: 0,
            gpu_count: 0,
            local_ssd_gb: 0,
        }
    }
}

impl CapacitySkuId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudCapacityError> {
        prefixed(
            value.into(),
            SKU_ID_PREFIX,
            CloudCapacityError::InvalidSkuId,
        )
        .map(|value| Self { value })
    }
}
impl CapacityReservationId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudCapacityError> {
        prefixed(
            value.into(),
            RESERVATION_ID_PREFIX,
            CloudCapacityError::InvalidReservationId,
        )
        .map(|value| Self { value })
    }
}
impl CommitmentId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudCapacityError> {
        prefixed(
            value.into(),
            COMMITMENT_ID_PREFIX,
            CloudCapacityError::InvalidCommitmentId,
        )
        .map(|value| Self { value })
    }
}
impl SpotPoolId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudCapacityError> {
        prefixed(
            value.into(),
            SPOT_POOL_ID_PREFIX,
            CloudCapacityError::InvalidSpotPoolId,
        )
        .map(|value| Self { value })
    }
}
impl SpotAssignmentId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudCapacityError> {
        prefixed(
            value.into(),
            SPOT_ASSIGNMENT_ID_PREFIX,
            CloudCapacityError::InvalidSpotAssignmentId,
        )
        .map(|value| Self { value })
    }
}
impl RebalancePlanId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudCapacityError> {
        prefixed(
            value.into(),
            REBALANCE_PLAN_ID_PREFIX,
            CloudCapacityError::InvalidRebalancePlanId,
        )
        .map(|value| Self { value })
    }
}
impl ApprovalRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudCapacityError> {
        prefixed(
            value.into(),
            APPROVAL_REF_PREFIX,
            CloudCapacityError::InvalidApprovalRef,
        )
        .map(|value| Self { value })
    }
}

impl CapacityTermMonths {
    pub const fn months(self) -> u64 {
        match self {
            Self::One => 1,
            Self::Twelve => 12,
            Self::ThirtySix => 36,
        }
    }
}

impl CapacitySku {
    pub fn new(input: CapacitySkuCreate) -> Result<Self, CloudCapacityError> {
        validate_units(input.unit)?;
        validate_nonzero_time(input.created_at_epoch_seconds)?;
        validate_money(&input.hourly_price)?;
        let region =
            RegionCode::new(input.region).map_err(|_| CloudCapacityError::InvalidRegion)?;
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudCapacityError::InvalidCellId)?;
        validate_cell_region(&cell_id, &region)?;
        Ok(Self {
            id: public(CapacitySkuId::new(input.id)?),
            region: public(region),
            cell_id: public(cell_id),
            flavor: public(input.flavor),
            unit: public(input.unit),
            hourly_price: financial(input.hourly_price),
            data_class: public_class(input.data_class)?,
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(CAPACITY_SCHEMA_VERSION),
        })
    }
}

impl CapacityReservation {
    pub fn active(
        sku: &CapacitySku,
        cell_capacity: CellCapacityEnvelope,
        input: CapacityReservationCreate,
    ) -> Result<Self, CloudCapacityError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_units(input.units)?;
        validate_units_match_sku_shape(input.units, sku.unit.value)?;
        validate_financial_window(
            input.term_months,
            input.start_epoch_seconds,
            input.end_epoch_seconds,
        )?;
        let sku_id = CapacitySkuId::new(input.sku_id)?;
        if sku_id != sku.id.value {
            return Err(CloudCapacityError::UnknownSku);
        }
        validate_reservation_headroom(cell_capacity, input.units)?;
        Ok(Self {
            id: internal(CapacityReservationId::new(input.id)?),
            tenant_id: internal(input.tenant_id),
            sku_id: public(sku_id),
            region: public(sku.region.value.clone()),
            cell_id: public(sku.cell_id.value.clone()),
            units: public(input.units),
            term_months: public(input.term_months),
            state: internal(CapacityReservationState::Active),
            start_epoch_seconds: internal(input.start_epoch_seconds),
            end_epoch_seconds: internal(input.end_epoch_seconds),
            data_class: financial_class(input.data_class)?,
            schema_version: public(CAPACITY_SCHEMA_VERSION),
        })
    }
}

impl CapacityReservation {
    /// Transition `Active -> Expired`. Requires `now_epoch_seconds >= end_epoch_seconds`.
    /// Returns `InvalidTimeOrder` for premature expiry and `InvalidTransition` from any
    /// non-Active state.
    pub fn expire_at(self, now_epoch_seconds: u64) -> Result<Self, CloudCapacityError> {
        if self.state.value != CapacityReservationState::Active {
            return Err(CloudCapacityError::InvalidTransition);
        }
        if now_epoch_seconds < self.end_epoch_seconds.value {
            return Err(CloudCapacityError::InvalidTimeOrder);
        }
        Ok(Self {
            state: internal(CapacityReservationState::Expired),
            ..self
        })
    }

    /// Transition `Active -> Cancelled`. Returns `InvalidTransition` from `Expired` or
    /// `Cancelled`.
    pub fn cancel(self) -> Result<Self, CloudCapacityError> {
        if self.state.value != CapacityReservationState::Active {
            return Err(CloudCapacityError::InvalidTransition);
        }
        Ok(Self {
            state: internal(CapacityReservationState::Cancelled),
            ..self
        })
    }
}

impl CommittedUseContract {
    pub fn active(input: CommittedUseCreate) -> Result<Self, CloudCapacityError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_financial_window(
            input.term_months,
            input.start_epoch_seconds,
            input.end_epoch_seconds,
        )?;
        if input.term_months == CapacityTermMonths::One {
            return Err(CloudCapacityError::InvalidTerm);
        }
        validate_money(&input.spend_commitment)?;
        if input.discount_bps == 0 || input.discount_bps > 6_000 {
            return Err(CloudCapacityError::InvalidDiscount);
        }
        Ok(Self {
            id: internal(CommitmentId::new(input.id)?),
            tenant_id: internal(input.tenant_id),
            region: public(
                RegionCode::new(input.region).map_err(|_| CloudCapacityError::InvalidRegion)?,
            ),
            term_months: public(input.term_months),
            spend_commitment: financial(input.spend_commitment),
            discount_bps: financial(input.discount_bps),
            state: internal(CommitmentState::Active),
            start_epoch_seconds: internal(input.start_epoch_seconds),
            end_epoch_seconds: internal(input.end_epoch_seconds),
            data_class: financial_class(input.data_class)?,
            schema_version: public(CAPACITY_SCHEMA_VERSION),
        })
    }
}

impl CommittedUseContract {
    /// Transition `Active -> Expired`. Requires `now_epoch_seconds >= end_epoch_seconds`.
    /// Returns `InvalidTimeOrder` for premature expiry and `InvalidTransition` from any
    /// non-Active state.
    pub fn expire_at(self, now_epoch_seconds: u64) -> Result<Self, CloudCapacityError> {
        if self.state.value != CommitmentState::Active {
            return Err(CloudCapacityError::InvalidTransition);
        }
        if now_epoch_seconds < self.end_epoch_seconds.value {
            return Err(CloudCapacityError::InvalidTimeOrder);
        }
        Ok(Self {
            state: internal(CommitmentState::Expired),
            ..self
        })
    }

    /// Transition `Active -> Cancelled`. Returns `InvalidTransition` from `Expired` or
    /// `Cancelled`.
    pub fn cancel(self) -> Result<Self, CloudCapacityError> {
        if self.state.value != CommitmentState::Active {
            return Err(CloudCapacityError::InvalidTransition);
        }
        Ok(Self {
            state: internal(CommitmentState::Cancelled),
            ..self
        })
    }
}

impl SpotPool {
    pub fn open(sku: &CapacitySku, input: SpotPoolCreate) -> Result<Self, CloudCapacityError> {
        validate_units(input.available_units)?;
        validate_units_match_sku_shape(input.available_units, sku.unit.value)?;
        validate_money(&input.current_price)?;
        if input.interruption_notice_seconds < MIN_SPOT_INTERRUPTION_NOTICE_SECONDS {
            return Err(CloudCapacityError::InvalidInterruptionNotice);
        }
        let sku_id = CapacitySkuId::new(input.sku_id)?;
        if sku_id != sku.id.value {
            return Err(CloudCapacityError::UnknownSku);
        }
        Ok(Self {
            id: internal(SpotPoolId::new(input.id)?),
            sku_id: public(sku_id),
            region: public(sku.region.value.clone()),
            cell_id: public(sku.cell_id.value.clone()),
            available_units: public(input.available_units),
            current_price: financial(input.current_price),
            interruption_notice_seconds: public(input.interruption_notice_seconds),
            state: public(SpotPoolState::Open),
            data_class: public_class(input.data_class)?,
            schema_version: public(CAPACITY_SCHEMA_VERSION),
        })
    }
}

impl SpotAssignment {
    pub fn active(
        pool: &SpotPool,
        sku: &CapacitySku,
        input: SpotAssignmentCreate,
    ) -> Result<Self, CloudCapacityError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_units(input.units)?;
        validate_units_match_sku_shape(input.units, sku.unit.value)?;
        validate_nonzero_time(input.requested_at_epoch_seconds)?;
        validate_money(&input.max_price)?;
        if pool.state.value != SpotPoolState::Open {
            return Err(CloudCapacityError::UnknownSpotPool);
        }
        if pool.sku_id.value != sku.id.value {
            return Err(CloudCapacityError::UnknownSku);
        }
        if !units_fit(input.units, pool.available_units.value) {
            return Err(CloudCapacityError::InvalidSpotCapacity);
        }
        if input.max_price.minor_units < pool.current_price.value.minor_units
            || input.max_price.currency != pool.current_price.value.currency
        {
            return Err(CloudCapacityError::InvalidSpotPrice);
        }
        Ok(Self {
            id: internal(SpotAssignmentId::new(input.id)?),
            tenant_id: internal(input.tenant_id),
            spot_pool_id: internal(pool.id.value.clone()),
            sku_id: public(pool.sku_id.value.clone()),
            region: public(pool.region.value.clone()),
            cell_id: public(pool.cell_id.value.clone()),
            units: public(input.units),
            max_price: financial(input.max_price),
            state: internal(SpotAssignmentState::Active),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            data_class: financial_class(input.data_class)?,
            schema_version: public(CAPACITY_SCHEMA_VERSION),
        })
    }
}

impl RebalancePlan {
    pub fn new(input: RebalancePlanCreate) -> Result<Self, CloudCapacityError> {
        validate_units(input.moved_units)?;
        validate_units(input.source_total)?;
        validate_nonzero_time(input.created_at_epoch_seconds)?;
        let region =
            RegionCode::new(input.region).map_err(|_| CloudCapacityError::InvalidRegion)?;
        let source_cell_id =
            CellId::new(input.source_cell_id).map_err(|_| CloudCapacityError::InvalidCellId)?;
        let target_cell_id =
            CellId::new(input.target_cell_id).map_err(|_| CloudCapacityError::InvalidCellId)?;
        validate_cell_region(&source_cell_id, &region)?;
        validate_cell_region(&target_cell_id, &region)?;
        if source_cell_id == target_cell_id {
            return Err(CloudCapacityError::InvalidCellId);
        }
        let move_bps = max_capacity_ratio_bps(input.moved_units, input.source_total)?;
        if move_bps > MAX_REBALANCE_MOVE_BPS {
            return Err(CloudCapacityError::InvalidRebalanceMove);
        }
        let approval_ref = input.approval_ref.map(ApprovalRef::new).transpose()?;
        Ok(Self {
            id: internal(RebalancePlanId::new(input.id)?),
            region: public(region),
            source_cell_id: public(source_cell_id),
            target_cell_id: public(target_cell_id),
            moved_units: public(input.moved_units),
            move_bps: internal(move_bps),
            approval_ref: internal(approval_ref.clone()),
            state: internal(if approval_ref.is_some() {
                RebalancePlanState::Approved
            } else {
                RebalancePlanState::Proposed
            }),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            data_class: internal_class(input.data_class)?,
            schema_version: public(CAPACITY_SCHEMA_VERSION),
        })
    }
}

impl CloudOpsFoundationGuardrail {
    pub fn new(input: CloudOpsFoundationGuardrailCreate) -> Result<Self, CloudCapacityError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_units(input.stable_reservation_units)?;
        validate_units(input.rebalance_source_total)?;
        validate_units(input.rebalance_move_units)?;
        let region =
            RegionCode::new(input.region.clone()).map_err(|_| CloudCapacityError::InvalidRegion)?;
        let cell_id =
            CellId::new(input.cell_id.clone()).map_err(|_| CloudCapacityError::InvalidCellId)?;
        validate_cell_region(&cell_id, &region)?;
        validate_reservation_headroom(
            input.stable_capacity_envelope,
            input.stable_reservation_units,
        )?;
        let stable_headroom_bps = stable_headroom_after_reservation_bps(
            input.stable_capacity_envelope,
            input.stable_reservation_units,
        )?;
        let rebalance_move_bps =
            max_capacity_ratio_bps(input.rebalance_move_units, input.rebalance_source_total)?;
        if rebalance_move_bps > MAX_REBALANCE_MOVE_BPS {
            return Err(CloudCapacityError::InvalidRebalanceMove);
        }

        let capacity_evidence_ref = validate_ops_evidence_ref(
            &input.capacity_evidence_ref,
            OPS_CAPACITY_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region,
            &cell_id,
        )?;
        let cell_lifecycle_evidence_ref = validate_ops_evidence_ref(
            &input.cell_lifecycle_evidence_ref,
            OPS_CELL_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region,
            &cell_id,
        )?;
        let dcops_evidence_ref = validate_ops_evidence_ref(
            &input.dcops_evidence_ref,
            OPS_DCOPS_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region,
            &cell_id,
        )?;
        let finops_evidence_ref = validate_ops_evidence_ref(
            &input.finops_evidence_ref,
            OPS_FINOPS_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region,
            &cell_id,
        )?;
        let marketplace_evidence_ref = validate_ops_evidence_ref(
            &input.marketplace_evidence_ref,
            OPS_MARKETPLACE_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region,
            &cell_id,
        )?;
        let filesystem_handoff_evidence_ref = validate_ops_evidence_ref(
            &input.filesystem_handoff_evidence_ref,
            OPS_FSH_EVIDENCE_PREFIX,
            &input.tenant_id,
            &region,
            &cell_id,
        )?;
        let audit_chain_ref =
            validate_audit_chain_ref(&input.audit_chain_ref, &input.tenant_id, &region, &cell_id)?;

        Ok(Self {
            tenant_id: internal(input.tenant_id),
            region: public(region),
            cell_id: public(cell_id),
            stable_capacity_envelope: internal(input.stable_capacity_envelope),
            stable_reservation_units: public(input.stable_reservation_units),
            stable_headroom_bps: internal(stable_headroom_bps),
            rebalance_source_total: internal(input.rebalance_source_total),
            rebalance_move_units: public(input.rebalance_move_units),
            rebalance_move_bps: internal(rebalance_move_bps),
            capacity_evidence_ref: audit(capacity_evidence_ref),
            cell_lifecycle_evidence_ref: audit(cell_lifecycle_evidence_ref),
            dcops_evidence_ref: audit(dcops_evidence_ref),
            finops_evidence_ref: audit(finops_evidence_ref),
            marketplace_evidence_ref: audit(marketplace_evidence_ref),
            filesystem_handoff_evidence_ref: audit(filesystem_handoff_evidence_ref),
            audit_chain_ref: audit(audit_chain_ref),
            schema_version: public(CLOUD_OPS_FOUNDATION_SCHEMA_VERSION),
        })
    }
}

impl CapacityRepo for CloudCapacityCatalog {
    fn register_sku(
        &mut self,
        input: CapacitySkuCreate,
    ) -> Result<CapacitySku, CloudCapacityError> {
        let sku = CapacitySku::new(input)?;
        if self.skus.contains_key(&sku.id.value) {
            return Err(CloudCapacityError::DuplicateSku);
        }
        self.skus.insert(sku.id.value.clone(), sku.clone());
        Ok(sku)
    }

    fn purchase_reservation(
        &mut self,
        cell_capacity: CellCapacityEnvelope,
        input: CapacityReservationCreate,
    ) -> Result<CapacityReservation, CloudCapacityError> {
        let sku_id = CapacitySkuId::new(input.sku_id.clone())?;
        let sku = self
            .skus
            .get(&sku_id)
            .ok_or(CloudCapacityError::UnknownSku)?;
        let reservation = CapacityReservation::active(sku, cell_capacity, input)?;
        if self.reservations.contains_key(&reservation.id.value) {
            return Err(CloudCapacityError::DuplicateReservation);
        }
        self.reservations
            .insert(reservation.id.value.clone(), reservation.clone());
        Ok(reservation)
    }

    fn purchase_commitment(
        &mut self,
        input: CommittedUseCreate,
    ) -> Result<CommittedUseContract, CloudCapacityError> {
        let commitment = CommittedUseContract::active(input)?;
        if self.commitments.contains_key(&commitment.id.value) {
            return Err(CloudCapacityError::DuplicateCommitment);
        }
        self.commitments
            .insert(commitment.id.value.clone(), commitment.clone());
        Ok(commitment)
    }

    fn open_spot_pool(&mut self, input: SpotPoolCreate) -> Result<SpotPool, CloudCapacityError> {
        let sku_id = CapacitySkuId::new(input.sku_id.clone())?;
        let sku = self
            .skus
            .get(&sku_id)
            .ok_or(CloudCapacityError::UnknownSku)?;
        let pool = SpotPool::open(sku, input)?;
        if self.spot_pools.contains_key(&pool.id.value) {
            return Err(CloudCapacityError::DuplicateSpotPool);
        }
        self.spot_pools.insert(pool.id.value.clone(), pool.clone());
        Ok(pool)
    }

    fn assign_spot_capacity(
        &mut self,
        input: SpotAssignmentCreate,
    ) -> Result<SpotAssignment, CloudCapacityError> {
        let pool_id = SpotPoolId::new(input.spot_pool_id.clone())?;
        let pool = self
            .spot_pools
            .get(&pool_id)
            .ok_or(CloudCapacityError::UnknownSpotPool)?;
        let sku = self
            .skus
            .get(&pool.sku_id.value)
            .ok_or(CloudCapacityError::UnknownSku)?;
        let assignment = SpotAssignment::active(pool, sku, input)?;
        if self.spot_assignments.contains_key(&assignment.id.value) {
            return Err(CloudCapacityError::DuplicateSpotAssignment);
        }
        let pool = self
            .spot_pools
            .get_mut(&pool_id)
            .ok_or(CloudCapacityError::UnknownSpotPool)?;
        pool.available_units = public(subtract_units(
            pool.available_units.value,
            assignment.units.value,
        )?);
        self.spot_assignments
            .insert(assignment.id.value.clone(), assignment.clone());
        Ok(assignment)
    }

    fn propose_rebalance(
        &mut self,
        input: RebalancePlanCreate,
    ) -> Result<RebalancePlan, CloudCapacityError> {
        let plan = RebalancePlan::new(input)?;
        if self.rebalance_plans.contains_key(&plan.id.value) {
            return Err(CloudCapacityError::DuplicateRebalancePlan);
        }
        self.rebalance_plans
            .insert(plan.id.value.clone(), plan.clone());
        Ok(plan)
    }

    fn record_capacity_meter(
        &mut self,
        input: CapacityMeterCreate,
    ) -> Result<MeterEvent, CloudCapacityError> {
        validate_tenant_id(&input.tenant_id)?;
        self.meter
            .record(MeterEventCreate {
                id: input.meter_event_id,
                tenant_id: input.tenant_id,
                capability_id: CAPACITY_METER_CAPABILITY_ID.to_string(),
                plane: PlaneTag::Control,
                units: input.units,
                source_axis: AxisId::Cloud,
                recorded_at_epoch_seconds: input.recorded_at_epoch_seconds,
                idempotency_key: input.idempotency_key,
                data_class: input.data_class,
            })
            .map_err(map_metering_error)
    }
}

impl CloudCapacityCatalog {
    pub fn skus(&self) -> impl Iterator<Item = &CapacitySku> {
        self.skus.values()
    }
    pub fn reservations(&self) -> impl Iterator<Item = &CapacityReservation> {
        self.reservations.values()
    }
    pub fn commitments(&self) -> impl Iterator<Item = &CommittedUseContract> {
        self.commitments.values()
    }
    pub fn spot_pools(&self) -> impl Iterator<Item = &SpotPool> {
        self.spot_pools.values()
    }
    pub fn spot_assignments(&self) -> impl Iterator<Item = &SpotAssignment> {
        self.spot_assignments.values()
    }
    pub fn rebalance_plans(&self) -> impl Iterator<Item = &RebalancePlan> {
        self.rebalance_plans.values()
    }
    pub fn meter_events(&self) -> impl Iterator<Item = &MeterEvent> {
        self.meter.events()
    }
}

fn validate_units(units: CapacityUnits) -> Result<(), CloudCapacityError> {
    if units.vcpu == 0 || units.memory_gb == 0 {
        return Err(CloudCapacityError::InvalidCapacityUnits);
    }
    Ok(())
}

fn validate_units_match_sku_shape(
    requested: CapacityUnits,
    sku_unit: CapacityUnits,
) -> Result<(), CloudCapacityError> {
    validate_units(requested)?;
    validate_units(sku_unit)?;
    let count = required_dimension_unit_count(requested.vcpu, sku_unit.vcpu)?;
    if required_dimension_unit_count(requested.memory_gb, sku_unit.memory_gb)? != count {
        return Err(CloudCapacityError::InvalidSkuUnitShape);
    }
    validate_optional_dimension_unit_count(requested.gpu_count, sku_unit.gpu_count, count)?;
    validate_optional_dimension_unit_count(requested.local_ssd_gb, sku_unit.local_ssd_gb, count)?;
    Ok(())
}

fn required_dimension_unit_count(requested: u32, sku_unit: u32) -> Result<u32, CloudCapacityError> {
    if requested == 0 || sku_unit == 0 || !requested.is_multiple_of(sku_unit) {
        return Err(CloudCapacityError::InvalidSkuUnitShape);
    }
    Ok(requested / sku_unit)
}

fn validate_optional_dimension_unit_count(
    requested: u32,
    sku_unit: u32,
    expected_count: u32,
) -> Result<(), CloudCapacityError> {
    match (requested, sku_unit) {
        (0, 0) => Ok(()),
        (value, 0) if value > 0 => Err(CloudCapacityError::InvalidSkuUnitShape),
        (0, _) => Err(CloudCapacityError::InvalidSkuUnitShape),
        (value, unit) if value % unit == 0 && value / unit == expected_count => Ok(()),
        _ => Err(CloudCapacityError::InvalidSkuUnitShape),
    }
}

fn validate_reservation_headroom(
    envelope: CellCapacityEnvelope,
    reservation: CapacityUnits,
) -> Result<(), CloudCapacityError> {
    let headroom_bps = stable_headroom_after_reservation_bps(envelope, reservation)?;
    if headroom_bps < REQUIRED_STABLE_HEADROOM_BPS {
        Err(CloudCapacityError::InvalidHeadroom)
    } else {
        Ok(())
    }
}

fn stable_headroom_after_reservation_bps(
    envelope: CellCapacityEnvelope,
    reservation: CapacityUnits,
) -> Result<u16, CloudCapacityError> {
    let stable_used = add_units(envelope.allocated, envelope.reserved)?;
    let reclaimable_used = add_units(stable_used, envelope.spot_assigned)?;
    let used = add_units(reclaimable_used, reservation)?;
    capacity_headroom_bps(envelope.total, used)
}

fn capacity_headroom_bps(
    total: CapacityUnits,
    used: CapacityUnits,
) -> Result<u16, CloudCapacityError> {
    validate_units(total)?;
    if !units_fit(used, total) {
        return Err(CloudCapacityError::InvalidHeadroom);
    }
    let vcpu_headroom = ratio_bps(total.vcpu - used.vcpu, total.vcpu)?;
    let mem_headroom = ratio_bps(total.memory_gb - used.memory_gb, total.memory_gb)?;
    let gpu_headroom = if total.gpu_count == 0 {
        u16::MAX
    } else {
        ratio_bps(total.gpu_count - used.gpu_count, total.gpu_count)?
    };
    let ssd_headroom = if total.local_ssd_gb == 0 {
        u16::MAX
    } else {
        ratio_bps(total.local_ssd_gb - used.local_ssd_gb, total.local_ssd_gb)?
    };
    Ok(vcpu_headroom
        .min(mem_headroom)
        .min(gpu_headroom)
        .min(ssd_headroom))
}

fn add_units(
    left: CapacityUnits,
    right: CapacityUnits,
) -> Result<CapacityUnits, CloudCapacityError> {
    Ok(CapacityUnits {
        vcpu: left
            .vcpu
            .checked_add(right.vcpu)
            .ok_or(CloudCapacityError::InvalidCapacityUnits)?,
        memory_gb: left
            .memory_gb
            .checked_add(right.memory_gb)
            .ok_or(CloudCapacityError::InvalidCapacityUnits)?,
        gpu_count: left
            .gpu_count
            .checked_add(right.gpu_count)
            .ok_or(CloudCapacityError::InvalidCapacityUnits)?,
        local_ssd_gb: left
            .local_ssd_gb
            .checked_add(right.local_ssd_gb)
            .ok_or(CloudCapacityError::InvalidCapacityUnits)?,
    })
}

fn subtract_units(
    left: CapacityUnits,
    right: CapacityUnits,
) -> Result<CapacityUnits, CloudCapacityError> {
    if !units_fit(right, left) {
        return Err(CloudCapacityError::InvalidCapacityUnits);
    }
    Ok(CapacityUnits {
        vcpu: left.vcpu - right.vcpu,
        memory_gb: left.memory_gb - right.memory_gb,
        gpu_count: left.gpu_count - right.gpu_count,
        local_ssd_gb: left.local_ssd_gb - right.local_ssd_gb,
    })
}

fn units_fit(requested: CapacityUnits, available: CapacityUnits) -> bool {
    requested.vcpu <= available.vcpu
        && requested.memory_gb <= available.memory_gb
        && requested.gpu_count <= available.gpu_count
        && requested.local_ssd_gb <= available.local_ssd_gb
}

fn ratio_bps(numerator: u32, denominator: u32) -> Result<u16, CloudCapacityError> {
    if denominator == 0 {
        return Err(CloudCapacityError::InvalidCapacityUnits);
    }
    Ok(((numerator as u128 * 10_000) / denominator as u128) as u16)
}

fn max_capacity_ratio_bps(
    moved: CapacityUnits,
    source_total: CapacityUnits,
) -> Result<u16, CloudCapacityError> {
    if !units_fit(moved, source_total) {
        return Err(CloudCapacityError::InvalidRebalanceMove);
    }
    Ok([
        ratio_bps(moved.vcpu, source_total.vcpu)?,
        ratio_bps(moved.memory_gb, source_total.memory_gb)?,
        optional_ratio_bps(moved.gpu_count, source_total.gpu_count)?,
        optional_ratio_bps(moved.local_ssd_gb, source_total.local_ssd_gb)?,
    ]
    .into_iter()
    .max()
    .unwrap_or(0))
}

fn optional_ratio_bps(numerator: u32, denominator: u32) -> Result<u16, CloudCapacityError> {
    match (numerator, denominator) {
        (0, 0) => Ok(0),
        (value, 0) if value > 0 => Err(CloudCapacityError::InvalidRebalanceMove),
        _ => ratio_bps(numerator, denominator),
    }
}

fn validate_financial_window(
    term: CapacityTermMonths,
    start: u64,
    end: u64,
) -> Result<(), CloudCapacityError> {
    validate_nonzero_time(start)?;
    let min_seconds = term.months() * 28 * 24 * 60 * 60;
    if end <= start || end - start < min_seconds {
        Err(CloudCapacityError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_money(value: &Money) -> Result<(), CloudCapacityError> {
    if value.minor_units == 0 || !matches!(value.currency.value.as_str(), "OYC" | "USD") {
        Err(CloudCapacityError::InvalidMoney)
    } else {
        Ok(())
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudCapacityError> {
    let Some(segment) = value.strip_prefix(TENANT_ID_PREFIX) else {
        return Err(CloudCapacityError::InvalidTenantId);
    };
    if segment.is_empty()
        || segment.starts_with('-')
        || segment.ends_with('-')
        || segment.contains("--")
    {
        return Err(CloudCapacityError::InvalidTenantId);
    }
    if segment.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
    }) {
        Ok(())
    } else {
        Err(CloudCapacityError::InvalidTenantId)
    }
}

fn validate_nonzero_time(value: u64) -> Result<(), CloudCapacityError> {
    if value == 0 {
        Err(CloudCapacityError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_cell_region(cell_id: &CellId, region: &RegionCode) -> Result<(), CloudCapacityError> {
    let prefix = format!("cell-{}-", region.value);
    if cell_id.value.starts_with(&prefix) {
        Ok(())
    } else {
        Err(CloudCapacityError::InvalidCellId)
    }
}

fn validate_ops_evidence_ref(
    value: &str,
    prefix: &str,
    tenant_id: &str,
    region: &RegionCode,
    cell_id: &CellId,
) -> Result<String, CloudCapacityError> {
    validate_scoped_ref(
        value,
        prefix,
        tenant_id,
        region,
        cell_id,
        CloudCapacityError::InvalidOpsEvidenceRef,
    )
}

fn validate_audit_chain_ref(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    cell_id: &CellId,
) -> Result<String, CloudCapacityError> {
    validate_scoped_ref(
        value,
        OPS_AUDIT_CHAIN_PREFIX,
        tenant_id,
        region,
        cell_id,
        CloudCapacityError::InvalidAuditChainRef,
    )
}

fn validate_scoped_ref(
    value: &str,
    prefix: &str,
    tenant_id: &str,
    region: &RegionCode,
    cell_id: &CellId,
    error: CloudCapacityError,
) -> Result<String, CloudCapacityError> {
    if !value.starts_with(prefix) || has_unsafe_ref_bytes(value) || has_secret_marker(value) {
        return Err(error);
    }
    let scoped = value.strip_prefix(prefix).ok_or_else(|| error.clone())?;
    let segments = safe_ref_segments(scoped).map_err(|_| error.clone())?;
    if segments.len() < 4
        || segments[0] != tenant_id
        || segments[1] != region.value.as_str()
        || segments[2] != cell_id.value.as_str()
    {
        return Err(error);
    }
    Ok(value.to_string())
}

fn has_unsafe_ref_bytes(value: &str) -> bool {
    value
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace() || matches!(ch, '\\' | '?' | '#'))
}

fn has_secret_marker(value: &str) -> bool {
    const SECRET_MARKERS: [&str; 11] = [
        "secret",
        "password",
        "private_key",
        "api_key",
        "apikey",
        "access_token",
        "token=",
        "bearer",
        "credential",
        "kubeconfig",
        "openbao",
    ];
    let lowercase = value.to_ascii_lowercase();
    SECRET_MARKERS
        .iter()
        .any(|marker| lowercase.contains(marker))
}

fn safe_ref_segments(value: &str) -> Result<Vec<&str>, ()> {
    let mut segments = Vec::new();
    for segment in value.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(());
        }
        segments.push(segment);
    }
    Ok(segments)
}

fn prefixed(
    value: String,
    prefix: &str,
    error: CloudCapacityError,
) -> Result<String, CloudCapacityError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

fn public_class(data_class: DataClass) -> Result<Classified<PrivacyDataClass>, CloudCapacityError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudCapacityError::InvalidDataClass)?;
    if class.data_class() == DataClass::Public {
        Ok(public(class))
    } else {
        Err(CloudCapacityError::InvalidDataClass)
    }
}

fn internal_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, CloudCapacityError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudCapacityError::InvalidDataClass)?;
    if class.data_class() == DataClass::InternalOnly {
        Ok(internal(class))
    } else {
        Err(CloudCapacityError::InvalidDataClass)
    }
}

fn financial_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, CloudCapacityError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudCapacityError::InvalidDataClass)?;
    if matches!(class.data_class(), DataClass::Financial) {
        Ok(financial(class))
    } else {
        Err(CloudCapacityError::InvalidDataClass)
    }
}

fn map_metering_error(_error: MeteringError) -> CloudCapacityError {
    CloudCapacityError::MeteringRejected
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}
fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}
fn financial<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Financial)
}
fn audit<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Audit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use billing_metering::{MeterUnit, MeterUnitKind};

    fn units(vcpu: u32, memory_gb: u32) -> CapacityUnits {
        CapacityUnits {
            vcpu,
            memory_gb,
            gpu_count: 0,
            local_ssd_gb: 0,
        }
    }

    fn sku_create() -> CapacitySkuCreate {
        CapacitySkuCreate {
            id: "csku_gp_region_alpha_a".to_string(),
            region: "region-alpha".to_string(),
            cell_id: "cell-region-alpha-a-001".to_string(),
            flavor: InstanceFlavor::GeneralPurpose,
            unit: units(4, 16),
            hourly_price: Money::new("OYC", 120_000).expect("money"),
            data_class: DataClass::Public,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn envelope() -> CellCapacityEnvelope {
        CellCapacityEnvelope {
            total: units(1_000, 4_000),
            allocated: units(400, 1_600),
            reserved: units(100, 400),
            spot_assigned: CapacityUnits::zero(),
        }
    }

    fn reservation_create() -> CapacityReservationCreate {
        CapacityReservationCreate {
            id: "cres_ten_alpha_gp".to_string(),
            tenant_id: "ten_alpha".to_string(),
            sku_id: "csku_gp_region_alpha_a".to_string(),
            units: units(100, 400),
            term_months: CapacityTermMonths::Twelve,
            start_epoch_seconds: 1_700_000_100,
            end_epoch_seconds: 1_732_000_100,
            data_class: DataClass::Financial,
        }
    }

    fn commitment_create() -> CommittedUseCreate {
        CommittedUseCreate {
            id: "cuc_ten_alpha_12m".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha".to_string(),
            term_months: CapacityTermMonths::Twelve,
            spend_commitment: Money::new("OYC", 100_000_000).expect("money"),
            discount_bps: 2_000,
            start_epoch_seconds: 1_700_000_100,
            end_epoch_seconds: 1_732_000_100,
            data_class: DataClass::Financial,
        }
    }

    fn spot_pool_create() -> SpotPoolCreate {
        SpotPoolCreate {
            id: "spot_gp_region_alpha_a".to_string(),
            sku_id: "csku_gp_region_alpha_a".to_string(),
            available_units: units(200, 800),
            current_price: Money::new("OYC", 30_000).expect("money"),
            interruption_notice_seconds: 120,
            data_class: DataClass::Public,
        }
    }

    fn seeded_catalog() -> CloudCapacityCatalog {
        let mut catalog = CloudCapacityCatalog::default();
        catalog.register_sku(sku_create()).expect("sku");
        catalog
    }

    #[test]
    fn registers_capacity_sku_with_region_cell_and_public_shape() {
        let mut catalog = CloudCapacityCatalog::default();
        let sku = catalog.register_sku(sku_create()).expect("sku is valid");
        assert_eq!(sku.id.value.value, "csku_gp_region_alpha_a");
        assert_eq!(sku.cell_id.value.value, "cell-region-alpha-a-001");
        assert_eq!(sku.data_class.value.data_class(), DataClass::Public);
        assert_eq!(catalog.skus().count(), 1);
    }

    #[test]
    fn reservation_enforces_stable_headroom_and_term_window() {
        let mut catalog = seeded_catalog();
        let reservation = catalog
            .purchase_reservation(envelope(), reservation_create())
            .expect("reservation preserves headroom");
        assert_eq!(reservation.state.value, CapacityReservationState::Active);
        assert_eq!(reservation.term_months.value, CapacityTermMonths::Twelve);

        let headroom_error = catalog
            .purchase_reservation(
                CellCapacityEnvelope {
                    reserved: units(250, 1_000),
                    ..envelope()
                },
                CapacityReservationCreate {
                    id: "cres_too_large".to_string(),
                    units: units(200, 800),
                    ..reservation_create()
                },
            )
            .expect_err("reservation cannot consume stable headroom");
        assert_eq!(headroom_error, CloudCapacityError::InvalidHeadroom);

        let shape_error = catalog
            .purchase_reservation(
                envelope(),
                CapacityReservationCreate {
                    id: "cres_bad_shape".to_string(),
                    units: CapacityUnits {
                        vcpu: 100,
                        memory_gb: 416,
                        gpu_count: 0,
                        local_ssd_gb: 0,
                    },
                    ..reservation_create()
                },
            )
            .expect_err("reservation must be an integral SKU shape");
        assert_eq!(shape_error, CloudCapacityError::InvalidSkuUnitShape);
    }

    #[test]
    fn committed_use_requires_long_term_supported_currency_and_bounded_discount() {
        let mut catalog = CloudCapacityCatalog::default();
        let commitment = catalog
            .purchase_commitment(commitment_create())
            .expect("commitment");
        assert_eq!(commitment.state.value, CommitmentState::Active);
        assert_eq!(commitment.spend_commitment.value.currency.value, "OYC");

        let term_error = catalog
            .purchase_commitment(CommittedUseCreate {
                id: "cuc_one_month".to_string(),
                term_months: CapacityTermMonths::One,
                ..commitment_create()
            })
            .expect_err("committed use requires annual term or longer");
        assert_eq!(term_error, CloudCapacityError::InvalidTerm);

        let discount_error = catalog
            .purchase_commitment(CommittedUseCreate {
                id: "cuc_bad_discount".to_string(),
                discount_bps: 6_001,
                ..commitment_create()
            })
            .expect_err("discount is bounded");
        assert_eq!(discount_error, CloudCapacityError::InvalidDiscount);
    }

    #[test]
    fn spot_pool_and_assignment_require_notice_capacity_and_price_ceiling() {
        let mut catalog = seeded_catalog();
        let pool = catalog
            .open_spot_pool(spot_pool_create())
            .expect("spot pool");
        assert_eq!(pool.state.value, SpotPoolState::Open);
        let assignment = catalog
            .assign_spot_capacity(SpotAssignmentCreate {
                id: "spota_ten_alpha_gp".to_string(),
                tenant_id: "ten_alpha".to_string(),
                spot_pool_id: "spot_gp_region_alpha_a".to_string(),
                units: units(20, 80),
                max_price: Money::new("OYC", 35_000).expect("money"),
                requested_at_epoch_seconds: 1_700_000_300,
                data_class: DataClass::Financial,
            })
            .expect("spot assignment");
        assert_eq!(assignment.state.value, SpotAssignmentState::Active);
        assert_eq!(
            catalog
                .spot_pools()
                .next()
                .expect("pool")
                .available_units
                .value,
            units(180, 720)
        );

        let notice_error = catalog
            .open_spot_pool(SpotPoolCreate {
                id: "spot_short_notice".to_string(),
                interruption_notice_seconds: 60,
                ..spot_pool_create()
            })
            .expect_err("spot notice must be at least two minutes");
        assert_eq!(notice_error, CloudCapacityError::InvalidInterruptionNotice);

        let price_error = catalog
            .assign_spot_capacity(SpotAssignmentCreate {
                id: "spota_low_price".to_string(),
                tenant_id: "ten_alpha".to_string(),
                spot_pool_id: "spot_gp_region_alpha_a".to_string(),
                units: units(20, 80),
                max_price: Money::new("OYC", 20_000).expect("money"),
                requested_at_epoch_seconds: 1_700_000_301,
                data_class: DataClass::Financial,
            })
            .expect_err("spot max price must cover current price");
        assert_eq!(price_error, CloudCapacityError::InvalidSpotPrice);

        let capacity_error = catalog
            .assign_spot_capacity(SpotAssignmentCreate {
                id: "spota_too_large".to_string(),
                tenant_id: "ten_alpha".to_string(),
                spot_pool_id: "spot_gp_region_alpha_a".to_string(),
                units: units(300, 1_200),
                max_price: Money::new("OYC", 35_000).expect("money"),
                requested_at_epoch_seconds: 1_700_000_302,
                data_class: DataClass::Financial,
            })
            .expect_err("spot assignment cannot exceed debited pool capacity");
        assert_eq!(capacity_error, CloudCapacityError::InvalidSpotCapacity);
    }

    #[test]
    fn rebalance_plan_limits_move_size_and_records_approval_state() {
        let mut catalog = CloudCapacityCatalog::default();
        let plan = catalog
            .propose_rebalance(RebalancePlanCreate {
                id: "crb_alpha_small_move".to_string(),
                region: "region-alpha".to_string(),
                source_cell_id: "cell-region-alpha-a-001".to_string(),
                target_cell_id: "cell-region-alpha-b-001".to_string(),
                moved_units: units(50, 200),
                source_total: units(1_000, 4_000),
                approval_ref: Some("approval/cloud-capacity/alpha-small-move".to_string()),
                created_at_epoch_seconds: 1_700_000_500,
                data_class: DataClass::InternalOnly,
            })
            .expect("approved rebalance plan");
        assert_eq!(plan.state.value, RebalancePlanState::Approved);
        assert_eq!(plan.move_bps.value, 500);

        let move_error = catalog
            .propose_rebalance(RebalancePlanCreate {
                id: "crb_alpha_big_move".to_string(),
                moved_units: units(200, 800),
                approval_ref: None,
                ..RebalancePlanCreate {
                    id: "crb_base".to_string(),
                    region: "region-alpha".to_string(),
                    source_cell_id: "cell-region-alpha-a-001".to_string(),
                    target_cell_id: "cell-region-alpha-b-001".to_string(),
                    moved_units: units(50, 200),
                    source_total: units(1_000, 4_000),
                    approval_ref: None,
                    created_at_epoch_seconds: 1_700_000_501,
                    data_class: DataClass::InternalOnly,
                }
            })
            .expect_err("rebalance proposal cannot move over ten percent");
        assert_eq!(move_error, CloudCapacityError::InvalidRebalanceMove);

        let memory_move_error = catalog
            .propose_rebalance(RebalancePlanCreate {
                id: "crb_alpha_memory_big_move".to_string(),
                region: "region-alpha".to_string(),
                source_cell_id: "cell-region-alpha-a-001".to_string(),
                target_cell_id: "cell-region-alpha-b-001".to_string(),
                moved_units: units(10, 800),
                source_total: units(1_000, 4_000),
                approval_ref: None,
                created_at_epoch_seconds: 1_700_000_502,
                data_class: DataClass::InternalOnly,
            })
            .expect_err("rebalance cap applies to the largest moved dimension");
        assert_eq!(memory_move_error, CloudCapacityError::InvalidRebalanceMove);
    }

    // --- CapacityReservation state-transition tests ---

    fn active_reservation() -> CapacityReservation {
        let sku = CapacitySku::new(sku_create()).expect("sku");
        CapacityReservation::active(&sku, envelope(), reservation_create()).expect("reservation")
    }

    #[test]
    fn reservation_expire_at_on_or_after_end_transitions_to_expired() {
        let r = active_reservation();
        let end = r.end_epoch_seconds.value;

        // at exactly end_epoch_seconds
        let expired_exact = r.clone().expire_at(end).expect("expire at end");
        assert_eq!(expired_exact.state.value, CapacityReservationState::Expired);

        // after end_epoch_seconds
        let expired_after = r.expire_at(end + 1).expect("expire after end");
        assert_eq!(expired_after.state.value, CapacityReservationState::Expired);
    }

    #[test]
    fn reservation_expire_at_before_end_is_rejected() {
        let r = active_reservation();
        let end = r.end_epoch_seconds.value;
        let err = r.expire_at(end - 1).expect_err("premature expiry");
        assert_eq!(err, CloudCapacityError::InvalidTimeOrder);
    }

    #[test]
    fn reservation_expire_at_on_already_expired_is_rejected() {
        let r = active_reservation();
        let end = r.end_epoch_seconds.value;
        let expired = r.expire_at(end).expect("first expire");
        let err = expired.expire_at(end + 1).expect_err("double-expire");
        assert_eq!(err, CloudCapacityError::InvalidTransition);
    }

    #[test]
    fn reservation_cancel_on_active_transitions_to_cancelled() {
        let r = active_reservation();
        let cancelled = r.cancel().expect("cancel active");
        assert_eq!(cancelled.state.value, CapacityReservationState::Cancelled);
    }

    #[test]
    fn reservation_double_cancel_is_rejected() {
        let r = active_reservation();
        let cancelled = r.cancel().expect("first cancel");
        let err = cancelled.cancel().expect_err("double-cancel");
        assert_eq!(err, CloudCapacityError::InvalidTransition);
    }

    #[test]
    fn reservation_cancel_after_expire_is_rejected() {
        let r = active_reservation();
        let end = r.end_epoch_seconds.value;
        let expired = r.expire_at(end).expect("expire");
        let err = expired.cancel().expect_err("cancel after expire");
        assert_eq!(err, CloudCapacityError::InvalidTransition);
    }

    // --- CommittedUseContract state-transition tests ---

    fn active_commitment() -> CommittedUseContract {
        CommittedUseContract::active(commitment_create()).expect("commitment")
    }

    #[test]
    fn commitment_expire_at_on_or_after_end_transitions_to_expired() {
        let c = active_commitment();
        let end = c.end_epoch_seconds.value;

        let expired_exact = c.clone().expire_at(end).expect("expire at end");
        assert_eq!(expired_exact.state.value, CommitmentState::Expired);

        let expired_after = c.expire_at(end + 1).expect("expire after end");
        assert_eq!(expired_after.state.value, CommitmentState::Expired);
    }

    #[test]
    fn commitment_expire_at_before_end_is_rejected() {
        let c = active_commitment();
        let end = c.end_epoch_seconds.value;
        let err = c.expire_at(end - 1).expect_err("premature expiry");
        assert_eq!(err, CloudCapacityError::InvalidTimeOrder);
    }

    #[test]
    fn commitment_expire_at_on_already_expired_is_rejected() {
        let c = active_commitment();
        let end = c.end_epoch_seconds.value;
        let expired = c.expire_at(end).expect("first expire");
        let err = expired.expire_at(end + 1).expect_err("double-expire");
        assert_eq!(err, CloudCapacityError::InvalidTransition);
    }

    #[test]
    fn commitment_cancel_on_active_transitions_to_cancelled() {
        let c = active_commitment();
        let cancelled = c.cancel().expect("cancel active");
        assert_eq!(cancelled.state.value, CommitmentState::Cancelled);
    }

    #[test]
    fn commitment_double_cancel_is_rejected() {
        let c = active_commitment();
        let cancelled = c.cancel().expect("first cancel");
        let err = cancelled.cancel().expect_err("double-cancel");
        assert_eq!(err, CloudCapacityError::InvalidTransition);
    }

    #[test]
    fn commitment_cancel_after_expire_is_rejected() {
        let c = active_commitment();
        let end = c.end_epoch_seconds.value;
        let expired = c.expire_at(end).expect("expire");
        let err = expired.cancel().expect_err("cancel after expire");
        assert_eq!(err, CloudCapacityError::InvalidTransition);
    }

    #[test]
    fn capacity_meter_events_are_cloud_axis_and_idempotent() {
        let mut catalog = CloudCapacityCatalog::default();
        let units =
            vec![MeterUnit::new(MeterUnitKind::ResourceSecond, 3_600_000_000).expect("unit")];
        let first = catalog
            .record_capacity_meter(CapacityMeterCreate {
                meter_event_id: "mtr_capacity_001".to_string(),
                tenant_id: "ten_alpha".to_string(),
                units: units.clone(),
                recorded_at_epoch_seconds: 1_700_000_600,
                idempotency_key: "idem_capacity_001".to_string(),
                data_class: DataClass::Public,
            })
            .expect("meter event");
        let replay = catalog
            .record_capacity_meter(CapacityMeterCreate {
                meter_event_id: "mtr_capacity_ignored".to_string(),
                tenant_id: "ten_alpha".to_string(),
                units,
                recorded_at_epoch_seconds: 1_700_000_601,
                idempotency_key: "idem_capacity_001".to_string(),
                data_class: DataClass::Public,
            })
            .expect("idempotent meter replay");
        assert_eq!(first.id.value, replay.id.value);
        assert_eq!(first.source_axis.value, AxisId::Cloud);
        assert_eq!(catalog.meter_events().count(), 1);
    }
}
