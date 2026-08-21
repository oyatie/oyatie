//! Cloud datacenter-operations kernel.
//!
//! This crate owns the preview DC-ops control contract named by `cloud.dcops.*`:
//! DCIM hierarchy, BMS points, power and cooling capacity, cable maps, physical
//! security zones, asset lifecycle, work orders, and sustainability evidence.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use cell_region::{AzCode, RegionCode};
use data_boundary_kernel::{Classified, DataClass, OperationalDataClass};

const DCOPS_SITE_SCHEMA_VERSION: u32 = 1;
const DCOPS_FACILITY_ZONE_SCHEMA_VERSION: u32 = 1;
const DCOPS_POWER_ZONE_SCHEMA_VERSION: u32 = 1;
const DCOPS_COOLING_ZONE_SCHEMA_VERSION: u32 = 1;
const DCOPS_SECURITY_ZONE_SCHEMA_VERSION: u32 = 1;
const DCOPS_RACK_SCHEMA_VERSION: u32 = 1;
const DCOPS_EQUIPMENT_SCHEMA_VERSION: u32 = 1;
const DCOPS_CABLE_SCHEMA_VERSION: u32 = 1;
const DCOPS_BMS_SCHEMA_VERSION: u32 = 1;
const DCOPS_WORK_ORDER_SCHEMA_VERSION: u32 = 1;
const DCOPS_SUSTAINABILITY_SCHEMA_VERSION: u32 = 1;
const DEFAULT_BMS_READING_RETENTION_LIMIT: usize = 1024;

const SITE_ID_PREFIX: &str = "dc";
const FACILITY_ZONE_ID_PREFIX: &str = "zone";
const POWER_ZONE_ID_PREFIX: &str = "power";
const COOLING_ZONE_ID_PREFIX: &str = "cooling";
const SECURITY_ZONE_ID_PREFIX: &str = "security";
const RACK_ID_PREFIX: &str = "rack";
const EQUIPMENT_ID_PREFIX: &str = "equip";
const CABLE_ID_PREFIX: &str = "cable";
const BMS_POINT_ID_PREFIX: &str = "bms";
const WORK_ORDER_ID_PREFIX: &str = "wo";
const SUSTAINABILITY_ID_PREFIX: &str = "sustainability";
const ASSET_TAG_PREFIX: &str = "asset";
const PROCUREMENT_REF_PREFIX: &str = "proc";
const SAFETY_PLAN_REF_PREFIX: &str = "safety";
const RESOLUTION_REF_PREFIX: &str = "resolution";
const PHYSICAL_REF_PREFIX: &str = "physical";
const USER_PRINCIPAL_PREFIX: &str = "usr_";
const SERVICE_PRINCIPAL_PREFIX: &str = "sp_";
const MIN_RACK_U_HEIGHT: u16 = 24;
const MAX_RACK_U_HEIGHT: u16 = 60;
const MAX_FIBER_LOSS_MILLI_DB: u32 = 30_000;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DatacenterSiteId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FacilityZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PowerZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CoolingZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SecurityZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RackId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EquipmentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CableRunId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BmsPointId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SustainabilitySnapshotId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AssetTag {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PrincipalId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DcSubstratePhase {
    ColoCage,
    OwnedInterior,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DatacenterTier {
    Tier2,
    Tier3,
    Tier4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DatacenterState {
    Planned,
    Commissioning,
    Active,
    Draining,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FacilityZoneKind {
    DataHall,
    MeetMeRoom,
    PowerRoom,
    CoolingPlant,
    SecurityLobby,
    Staging,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FacilityZoneState {
    Planned,
    Active,
    Isolated,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PowerRedundancy {
    N,
    NPlusOne,
    TwoN,
    TwoNPlusOne,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PowerZoneState {
    Planned,
    Energized,
    Maintenance,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CoolingTechnology {
    CraH,
    ChilledWater,
    FreeAir,
    HotAisleContainment,
    LiquidCooling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CoolingZoneState {
    Planned,
    Active,
    Maintenance,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecurityZoneKind {
    Badge,
    Cctv,
    Mantrap,
    EnvironmentalSensor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecurityZoneState {
    Planned,
    Armed,
    Isolated,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RackState {
    Planned,
    Active,
    Quarantined,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EquipmentKind {
    Server,
    GpuServer,
    Pdu,
    Ats,
    Ups,
    Generator,
    CraH,
    Chiller,
    PatchPanel,
    Router,
    Switch,
    Camera,
    BadgeReader,
    EnvironmentalSensor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EquipmentLifecycle {
    Ordered,
    Received,
    Installed,
    InService,
    Maintenance,
    Decommissioning,
    Sanitized,
    EwasteTransferred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CableMedia {
    SingleModeFiber,
    MultiModeFiber,
    Copper,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CableState {
    Planned,
    Installed,
    Certified,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BmsPointKind {
    Hvac,
    Fire,
    Water,
    Temperature,
    Humidity,
    Fuel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BmsPointState {
    Commissioning,
    Enabled,
    Disabled,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkOrderKind {
    Install,
    Repair,
    PreventiveMaintenance,
    Decommission,
    Audit,
    Sustainability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkOrderPriority {
    P0,
    P1,
    P2,
    P3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkOrderState {
    Open,
    Assigned,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatacenterSiteCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub availability_zone: String,     // data_class: PUBLIC
    pub physical_ref: String,          // data_class: INTERNAL_ONLY
    pub phase: DcSubstratePhase,       // data_class: PUBLIC
    pub tier: DatacenterTier,          // data_class: PUBLIC
    pub state: DatacenterState,        // data_class: PUBLIC
    pub provider_facing: bool,         // data_class: PUBLIC
    pub pue_target_milli: u32,         // data_class: INTERNAL_ONLY
    pub wue_target_milli: u32,         // data_class: INTERNAL_ONLY
    pub cue_target_milli: u32,         // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatacenterSite {
    pub id: Classified<DatacenterSiteId>,
    pub region: Classified<RegionCode>,
    pub availability_zone: Classified<AzCode>,
    pub physical_ref: Classified<String>,
    pub phase: Classified<DcSubstratePhase>,
    pub tier: Classified<DatacenterTier>,
    pub state: Classified<DatacenterState>,
    pub provider_facing: Classified<bool>,
    pub pue_target_milli: Classified<u32>,
    pub wue_target_milli: Classified<u32>,
    pub cue_target_milli: Classified<u32>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacilityZoneCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub site_id: String,               // data_class: INTERNAL_ONLY
    pub kind: FacilityZoneKind,        // data_class: PUBLIC
    pub state: FacilityZoneState,      // data_class: PUBLIC
    pub display_name: String,          // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FacilityZone {
    pub id: Classified<FacilityZoneId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub kind: Classified<FacilityZoneKind>,
    pub state: Classified<FacilityZoneState>,
    pub display_name: Classified<String>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PowerZoneCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub site_id: String,               // data_class: INTERNAL_ONLY
    pub redundancy: PowerRedundancy,   // data_class: PUBLIC
    pub state: PowerZoneState,         // data_class: PUBLIC
    pub capacity_watts: u64,           // data_class: INTERNAL_ONLY
    pub utility_feed_count: u8,        // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PowerZone {
    pub id: Classified<PowerZoneId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub redundancy: Classified<PowerRedundancy>,
    pub state: Classified<PowerZoneState>,
    pub capacity_watts: Classified<u64>,
    pub utility_feed_count: Classified<u8>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoolingZoneCreate {
    pub id: String,                        // data_class: INTERNAL_ONLY
    pub site_id: String,                   // data_class: INTERNAL_ONLY
    pub technology: CoolingTechnology,     // data_class: PUBLIC
    pub state: CoolingZoneState,           // data_class: PUBLIC
    pub heat_capacity_watts: u64,          // data_class: INTERNAL_ONLY
    pub water_budget_liters_per_hour: u64, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoolingZone {
    pub id: Classified<CoolingZoneId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub technology: Classified<CoolingTechnology>,
    pub state: Classified<CoolingZoneState>,
    pub heat_capacity_watts: Classified<u64>,
    pub water_budget_liters_per_hour: Classified<u64>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityZoneCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub site_id: String,               // data_class: INTERNAL_ONLY
    pub kind: SecurityZoneKind,        // data_class: PUBLIC
    pub state: SecurityZoneState,      // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityZone {
    pub id: Classified<SecurityZoneId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub kind: Classified<SecurityZoneKind>,
    pub state: Classified<SecurityZoneState>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RackCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub site_id: String,               // data_class: INTERNAL_ONLY
    pub facility_zone_id: String,      // data_class: INTERNAL_ONLY
    pub security_zone_id: String,      // data_class: INTERNAL_ONLY
    pub row_label: String,             // data_class: INTERNAL_ONLY
    pub state: RackState,              // data_class: PUBLIC
    pub u_height: u16,                 // data_class: INTERNAL_ONLY
    pub rated_power_watts: u64,        // data_class: INTERNAL_ONLY
    pub max_heat_watts: u64,           // data_class: INTERNAL_ONLY
    pub max_weight_kg: u64,            // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rack {
    pub id: Classified<RackId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub facility_zone_id: Classified<FacilityZoneId>,
    pub security_zone_id: Classified<SecurityZoneId>,
    pub row_label: Classified<String>,
    pub state: Classified<RackState>,
    pub u_height: Classified<u16>,
    pub rated_power_watts: Classified<u64>,
    pub max_heat_watts: Classified<u64>,
    pub max_weight_kg: Classified<u64>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub site_id: String,               // data_class: INTERNAL_ONLY
    pub kind: EquipmentKind,           // data_class: PUBLIC
    pub lifecycle: EquipmentLifecycle, // data_class: PUBLIC
    pub procurement_ref: String,       // data_class: INTERNAL_ONLY
    pub vendor: String,                // data_class: INTERNAL_ONLY
    pub model: String,                 // data_class: INTERNAL_ONLY
    pub ordered_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentInstallPlan {
    pub rack_id: String,                 // data_class: INTERNAL_ONLY
    pub power_zone_id: String,           // data_class: INTERNAL_ONLY
    pub cooling_zone_id: String,         // data_class: INTERNAL_ONLY
    pub start_u: u16,                    // data_class: INTERNAL_ONLY
    pub height_u: u16,                   // data_class: INTERNAL_ONLY
    pub power_watts: u64,                // data_class: INTERNAL_ONLY
    pub heat_watts: u64,                 // data_class: INTERNAL_ONLY
    pub weight_kg: u64,                  // data_class: INTERNAL_ONLY
    pub network_drop_refs: Vec<String>,  // data_class: INTERNAL_ONLY
    pub installed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquipmentInstallation {
    pub rack_id: RackId,                 // data_class: INTERNAL_ONLY
    pub power_zone_id: PowerZoneId,      // data_class: INTERNAL_ONLY
    pub cooling_zone_id: CoolingZoneId,  // data_class: INTERNAL_ONLY
    pub start_u: u16,                    // data_class: INTERNAL_ONLY
    pub height_u: u16,                   // data_class: INTERNAL_ONLY
    pub power_watts: u64,                // data_class: INTERNAL_ONLY
    pub heat_watts: u64,                 // data_class: INTERNAL_ONLY
    pub weight_kg: u64,                  // data_class: INTERNAL_ONLY
    pub network_drop_refs: Vec<String>,  // data_class: INTERNAL_ONLY
    pub installed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Equipment {
    pub id: Classified<EquipmentId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub kind: Classified<EquipmentKind>,
    pub lifecycle: Classified<EquipmentLifecycle>,
    pub procurement_ref: Classified<String>,
    pub vendor: Classified<String>,
    pub model: Classified<String>,
    pub asset_tag: Classified<Option<AssetTag>>,
    pub serial_number: Classified<Option<String>>,
    pub installation: Classified<Option<EquipmentInstallation>>,
    pub ordered_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CableEndpoint {
    pub equipment_id: String, // data_class: INTERNAL_ONLY
    pub port_name: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CableRunCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub site_id: String,               // data_class: INTERNAL_ONLY
    pub from: CableEndpoint,           // data_class: INTERNAL_ONLY
    pub to: CableEndpoint,             // data_class: INTERNAL_ONLY
    pub media: CableMedia,             // data_class: PUBLIC
    pub state: CableState,             // data_class: PUBLIC
    pub measured_loss_milli_db: u32,   // data_class: INTERNAL_ONLY
    pub loss_budget_milli_db: u32,     // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CableRun {
    pub id: Classified<CableRunId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub from: Classified<CableEndpointTyped>,
    pub to: Classified<CableEndpointTyped>,
    pub media: Classified<CableMedia>,
    pub state: Classified<CableState>,
    pub measured_loss_milli_db: Classified<u32>,
    pub loss_budget_milli_db: Classified<u32>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CableEndpointTyped {
    pub equipment_id: EquipmentId, // data_class: INTERNAL_ONLY
    pub port_name: String,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BmsPointCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub site_id: String,               // data_class: INTERNAL_ONLY
    pub equipment_id: Option<String>,  // data_class: INTERNAL_ONLY
    pub kind: BmsPointKind,            // data_class: PUBLIC
    pub state: BmsPointState,          // data_class: PUBLIC
    pub unit: String,                  // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BmsPoint {
    pub id: Classified<BmsPointId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub equipment_id: Classified<Option<EquipmentId>>,
    pub kind: Classified<BmsPointKind>,
    pub state: Classified<BmsPointState>,
    pub unit: Classified<String>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BmsReadingCreate {
    pub point_id: String,               // data_class: INTERNAL_ONLY
    pub site_id: String,                // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub milli_value: i64,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BmsReading {
    pub point_id: Classified<BmsPointId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub observed_at_epoch_seconds: Classified<u64>,
    pub milli_value: Classified<i64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkOrderCreate {
    pub id: String,                   // data_class: INTERNAL_ONLY
    pub site_id: String,              // data_class: INTERNAL_ONLY
    pub equipment_id: Option<String>, // data_class: INTERNAL_ONLY
    pub kind: WorkOrderKind,          // data_class: PUBLIC
    pub priority: WorkOrderPriority,  // data_class: INTERNAL_ONLY
    pub state: WorkOrderState,        // data_class: PUBLIC
    pub opened_by: String,            // data_class: INTERNAL_ONLY
    pub assigned_to: Option<String>,  // data_class: INTERNAL_ONLY
    pub safety_plan_ref: String,      // data_class: INTERNAL_ONLY
    pub data_class: DataClass,        // data_class: PUBLIC
    pub opened_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkOrderResolution {
    pub completed_by: String,            // data_class: INTERNAL_ONLY
    pub resolution_ref: String,          // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkOrder {
    pub id: Classified<WorkOrderId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub equipment_id: Classified<Option<EquipmentId>>,
    pub kind: Classified<WorkOrderKind>,
    pub priority: Classified<WorkOrderPriority>,
    pub state: Classified<WorkOrderState>,
    pub opened_by: Classified<PrincipalId>,
    pub assigned_to: Classified<Option<PrincipalId>>,
    pub safety_plan_ref: Classified<String>,
    pub data_class: Classified<DataClass>,
    pub resolution: Classified<Option<WorkOrderResolutionTyped>>,
    pub opened_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkOrderResolutionTyped {
    pub completed_by: PrincipalId,       // data_class: INTERNAL_ONLY
    pub resolution_ref: String,          // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SustainabilitySnapshotCreate {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub site_id: String,                 // data_class: INTERNAL_ONLY
    pub period_start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub period_end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub it_energy_kwh_milli: u64,        // data_class: INTERNAL_ONLY
    pub facility_energy_kwh_milli: u64,  // data_class: INTERNAL_ONLY
    pub water_liters_milli: u64,         // data_class: INTERNAL_ONLY
    pub carbon_grams: u64,               // data_class: INTERNAL_ONLY
    pub pue_milli: u64,                  // data_class: INTERNAL_ONLY
    pub wue_milli: u64,                  // data_class: INTERNAL_ONLY
    pub cue_milli: u64,                  // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SustainabilitySnapshot {
    pub id: Classified<SustainabilitySnapshotId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub period_start_epoch_seconds: Classified<u64>,
    pub period_end_epoch_seconds: Classified<u64>,
    pub it_energy_kwh_milli: Classified<u64>,
    pub facility_energy_kwh_milli: Classified<u64>,
    pub water_liters_milli: Classified<u64>,
    pub carbon_grams: Classified<u64>,
    pub pue_milli: Classified<u64>,
    pub wue_milli: Classified<u64>,
    pub cue_milli: Classified<u64>,
    pub data_class: Classified<DataClass>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct RackCapacitySnapshot {
    pub used_u: u16,                // data_class: INTERNAL_ONLY
    pub free_u: u16,                // data_class: INTERNAL_ONLY
    pub used_power_watts: u64,      // data_class: INTERNAL_ONLY
    pub remaining_power_watts: u64, // data_class: INTERNAL_ONLY
    pub used_heat_watts: u64,       // data_class: INTERNAL_ONLY
    pub remaining_heat_watts: u64,  // data_class: INTERNAL_ONLY
    pub used_weight_kg: u64,        // data_class: INTERNAL_ONLY
    pub remaining_weight_kg: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct InstallationAccounting {
    rack_capacity: RackCapacitySnapshot,
    power_zone_used_watts: u64,
    cooling_zone_used_watts: u64,
    rack_unit_overlap: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudDcopsError {
    InvalidDatacenterSiteId,
    InvalidFacilityZoneId,
    InvalidPowerZoneId,
    InvalidCoolingZoneId,
    InvalidSecurityZoneId,
    InvalidRackId,
    InvalidEquipmentId,
    InvalidCableRunId,
    InvalidBmsPointId,
    InvalidWorkOrderId,
    InvalidSustainabilitySnapshotId,
    InvalidAssetTag,
    InvalidPrincipalId,
    InvalidRegion,
    InvalidAz,
    RegionMismatch,
    AzRegionMismatch,
    InvalidPhysicalRef,
    InvalidText,
    InvalidInitialState,
    InvalidStateTransition,
    InvalidTimeOrder,
    InvalidTargetRatio,
    InvalidCapacity,
    InvalidRedundancy,
    InvalidRackUnits,
    InvalidInstallPlan,
    InvalidPort,
    InvalidCableLoss,
    InvalidBmsReading,
    InvalidDataClass,
    ParentMismatch,
    InactiveParent,
    DuplicateSite,
    DuplicateFacilityZone,
    DuplicatePowerZone,
    DuplicateCoolingZone,
    DuplicateSecurityZone,
    DuplicateRack,
    DuplicateEquipment,
    DuplicateCableRun,
    DuplicateBmsPoint,
    DuplicateBmsReading,
    DuplicateWorkOrder,
    DuplicateSustainabilitySnapshot,
    UnknownSite,
    UnknownFacilityZone,
    UnknownPowerZone,
    UnknownCoolingZone,
    UnknownSecurityZone,
    UnknownRack,
    UnknownEquipment,
    UnknownCableRun,
    UnknownBmsPoint,
    UnknownWorkOrder,
    RackUnitOverlap,
    RackCapacityExceeded,
    PowerZoneCapacityExceeded,
    CoolingZoneCapacityExceeded,
    CrossSiteReference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudDcopsCatalog {
    sites: BTreeMap<DatacenterSiteId, DatacenterSite>,
    facility_zones: BTreeMap<FacilityZoneId, FacilityZone>,
    power_zones: BTreeMap<PowerZoneId, PowerZone>,
    cooling_zones: BTreeMap<CoolingZoneId, CoolingZone>,
    security_zones: BTreeMap<SecurityZoneId, SecurityZone>,
    racks: BTreeMap<RackId, Rack>,
    equipment: BTreeMap<EquipmentId, Equipment>,
    cable_runs: BTreeMap<CableRunId, CableRun>,
    bms_points: BTreeMap<BmsPointId, BmsPoint>,
    bms_readings: BTreeSet<(BmsPointId, u64)>,
    bms_reading_retention_limit: usize,
    work_orders: BTreeMap<WorkOrderId, WorkOrder>,
    sustainability_snapshots: BTreeMap<SustainabilitySnapshotId, SustainabilitySnapshot>,
    rack_capacity_by_id: BTreeMap<RackId, RackCapacitySnapshot>,
    power_zone_used_watts_by_id: BTreeMap<PowerZoneId, u64>,
    cooling_zone_used_watts_by_id: BTreeMap<CoolingZoneId, u64>,
    rack_unit_allocations_by_id: BTreeMap<RackId, BTreeMap<EquipmentId, (u16, u16)>>,
}

impl Default for CloudDcopsCatalog {
    fn default() -> Self {
        Self::with_bms_reading_retention_limit(DEFAULT_BMS_READING_RETENTION_LIMIT)
    }
}

impl DatacenterSiteId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            SITE_ID_PREFIX,
            3,
            CloudDcopsError::InvalidDatacenterSiteId,
        )?;
        let region = site_region_from_path(&value)?;
        RegionCode::new(region).map_err(|_| CloudDcopsError::InvalidDatacenterSiteId)?;
        Ok(Self { value })
    }

    pub fn region(&self) -> Result<RegionCode, CloudDcopsError> {
        RegionCode::new(site_region_from_path(&self.value)?)
            .map_err(|_| CloudDcopsError::InvalidDatacenterSiteId)
    }
}

impl FacilityZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            FACILITY_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidFacilityZoneId,
        )?;
        Ok(Self { value })
    }
}

impl PowerZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            POWER_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidPowerZoneId,
        )?;
        Ok(Self { value })
    }
}

impl CoolingZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            COOLING_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidCoolingZoneId,
        )?;
        Ok(Self { value })
    }
}

impl SecurityZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            SECURITY_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidSecurityZoneId,
        )?;
        Ok(Self { value })
    }
}

impl RackId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(&value, RACK_ID_PREFIX, 5, CloudDcopsError::InvalidRackId)?;
        Ok(Self { value })
    }
}

impl EquipmentId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            EQUIPMENT_ID_PREFIX,
            5,
            CloudDcopsError::InvalidEquipmentId,
        )?;
        Ok(Self { value })
    }
}

impl CableRunId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            CABLE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidCableRunId,
        )?;
        Ok(Self { value })
    }
}

impl BmsPointId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            BMS_POINT_ID_PREFIX,
            5,
            CloudDcopsError::InvalidBmsPointId,
        )?;
        Ok(Self { value })
    }
}

impl WorkOrderId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            WORK_ORDER_ID_PREFIX,
            5,
            CloudDcopsError::InvalidWorkOrderId,
        )?;
        Ok(Self { value })
    }
}

impl SustainabilitySnapshotId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            SUSTAINABILITY_ID_PREFIX,
            5,
            CloudDcopsError::InvalidSustainabilitySnapshotId,
        )?;
        Ok(Self { value })
    }
}

impl AssetTag {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            ASSET_TAG_PREFIX,
            5,
            CloudDcopsError::InvalidAssetTag,
        )?;
        Ok(Self { value })
    }
}

impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        if (value.starts_with(USER_PRINCIPAL_PREFIX) && value.len() > USER_PRINCIPAL_PREFIX.len())
            || (value.starts_with(SERVICE_PRINCIPAL_PREFIX)
                && value.len() > SERVICE_PRINCIPAL_PREFIX.len())
        {
            Ok(Self { value })
        } else {
            Err(CloudDcopsError::InvalidPrincipalId)
        }
    }
}

impl DatacenterSite {
    pub fn new(input: DatacenterSiteCreate) -> Result<Self, CloudDcopsError> {
        if input.state != DatacenterState::Planned {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        validate_physical_ref(&input.physical_ref)?;
        validate_sustainability_targets(
            input.pue_target_milli,
            input.wue_target_milli,
            input.cue_target_milli,
        )?;
        let id = DatacenterSiteId::new(input.id)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudDcopsError::InvalidRegion)?;
        let az = AzCode::new(input.availability_zone).map_err(|_| CloudDcopsError::InvalidAz)?;
        if id.region()? != region {
            return Err(CloudDcopsError::RegionMismatch);
        }
        validate_az_region(&az, &region)?;
        Ok(Self {
            id: internal(id),
            region: public(region),
            availability_zone: public(az),
            physical_ref: internal(input.physical_ref),
            phase: public(input.phase),
            tier: public(input.tier),
            state: public(input.state),
            provider_facing: public(input.provider_facing),
            pue_target_milli: internal(input.pue_target_milli),
            wue_target_milli: internal(input.wue_target_milli),
            cue_target_milli: internal(input.cue_target_milli),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_SITE_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: DatacenterState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !datacenter_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl FacilityZone {
    pub fn new(input: FacilityZoneCreate) -> Result<Self, CloudDcopsError> {
        if input.state != FacilityZoneState::Planned {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        validate_non_empty(&input.display_name)?;
        let id = FacilityZoneId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            FACILITY_ZONE_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            kind: public(input.kind),
            state: public(input.state),
            display_name: internal(input.display_name),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_FACILITY_ZONE_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: FacilityZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !facility_zone_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl PowerZone {
    pub fn new(input: PowerZoneCreate) -> Result<Self, CloudDcopsError> {
        if input.state != PowerZoneState::Planned {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        validate_power_redundancy(input.redundancy, input.utility_feed_count)?;
        if input.capacity_watts == 0 {
            return Err(CloudDcopsError::InvalidCapacity);
        }
        let id = PowerZoneId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            POWER_ZONE_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            redundancy: public(input.redundancy),
            state: public(input.state),
            capacity_watts: internal(input.capacity_watts),
            utility_feed_count: internal(input.utility_feed_count),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_POWER_ZONE_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: PowerZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !power_zone_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl CoolingZone {
    pub fn new(input: CoolingZoneCreate) -> Result<Self, CloudDcopsError> {
        if input.state != CoolingZoneState::Planned {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        if input.heat_capacity_watts == 0 {
            return Err(CloudDcopsError::InvalidCapacity);
        }
        let id = CoolingZoneId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            COOLING_ZONE_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            technology: public(input.technology),
            state: public(input.state),
            heat_capacity_watts: internal(input.heat_capacity_watts),
            water_budget_liters_per_hour: internal(input.water_budget_liters_per_hour),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_COOLING_ZONE_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: CoolingZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !cooling_zone_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl SecurityZone {
    pub fn new(input: SecurityZoneCreate) -> Result<Self, CloudDcopsError> {
        if input.state != SecurityZoneState::Planned {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        let id = SecurityZoneId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            SECURITY_ZONE_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            kind: public(input.kind),
            state: public(input.state),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_SECURITY_ZONE_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: SecurityZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !security_zone_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl Rack {
    pub fn new(input: RackCreate) -> Result<Self, CloudDcopsError> {
        if input.state != RackState::Planned {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        validate_rack_shape(
            input.u_height,
            input.rated_power_watts,
            input.max_heat_watts,
            input.max_weight_kg,
        )?;
        validate_path_segment(&input.row_label).map_err(|_| CloudDcopsError::InvalidText)?;
        let id = RackId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        let facility_zone_id = FacilityZoneId::new(input.facility_zone_id)?;
        let security_zone_id = SecurityZoneId::new(input.security_zone_id)?;
        validate_child_id(
            &id.value,
            RACK_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            facility_zone_id: internal(facility_zone_id),
            security_zone_id: internal(security_zone_id),
            row_label: internal(input.row_label),
            state: public(input.state),
            u_height: internal(input.u_height),
            rated_power_watts: internal(input.rated_power_watts),
            max_heat_watts: internal(input.max_heat_watts),
            max_weight_kg: internal(input.max_weight_kg),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_RACK_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: RackState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !rack_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl EquipmentKind {
    pub const fn requires_power(self) -> bool {
        !matches!(self, Self::PatchPanel)
    }
}

impl EquipmentInstallPlan {
    pub fn typed(&self, kind: EquipmentKind) -> Result<EquipmentInstallation, CloudDcopsError> {
        let rack_id = RackId::new(self.rack_id.clone())?;
        let power_zone_id = PowerZoneId::new(self.power_zone_id.clone())?;
        let cooling_zone_id = CoolingZoneId::new(self.cooling_zone_id.clone())?;
        validate_install_shape(self, kind)?;
        Ok(EquipmentInstallation {
            rack_id,
            power_zone_id,
            cooling_zone_id,
            start_u: self.start_u,
            height_u: self.height_u,
            power_watts: self.power_watts,
            heat_watts: self.heat_watts,
            weight_kg: self.weight_kg,
            network_drop_refs: typed_network_drop_refs(&self.network_drop_refs)?,
            installed_at_epoch_seconds: self.installed_at_epoch_seconds,
        })
    }
}

impl Equipment {
    pub fn new(input: EquipmentCreate) -> Result<Self, CloudDcopsError> {
        if input.lifecycle != EquipmentLifecycle::Ordered {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.ordered_at_epoch_seconds)?;
        validate_ref_path(
            &input.procurement_ref,
            PROCUREMENT_REF_PREFIX,
            CloudDcopsError::InvalidText,
        )?;
        validate_non_empty(&input.vendor)?;
        validate_non_empty(&input.model)?;
        let id = EquipmentId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            EQUIPMENT_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            kind: public(input.kind),
            lifecycle: public(input.lifecycle),
            procurement_ref: internal(input.procurement_ref),
            vendor: internal(input.vendor),
            model: internal(input.model),
            asset_tag: internal(None),
            serial_number: internal(None),
            installation: internal(None),
            ordered_at_epoch_seconds: internal(input.ordered_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.ordered_at_epoch_seconds),
            schema_version: public(DCOPS_EQUIPMENT_SCHEMA_VERSION),
        })
    }

    pub fn receive(
        &self,
        asset_tag: String,
        serial_number: String,
        received_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            received_at_epoch_seconds,
        )?;
        if self.lifecycle.value != EquipmentLifecycle::Ordered {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let asset_tag = AssetTag::new(asset_tag)?;
        validate_child_id(
            &asset_tag.value,
            ASSET_TAG_PREFIX,
            &self.site_id.value.value,
            CloudDcopsError::ParentMismatch,
        )?;
        validate_non_empty(&serial_number)?;
        let mut next = self.clone();
        next.lifecycle = public(EquipmentLifecycle::Received);
        next.asset_tag = internal(Some(asset_tag));
        next.serial_number = internal(Some(serial_number));
        next.updated_at_epoch_seconds = internal(received_at_epoch_seconds);
        Ok(next)
    }

    pub fn install(&self, installation: EquipmentInstallation) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            installation.installed_at_epoch_seconds,
        )?;
        if self.lifecycle.value != EquipmentLifecycle::Received {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let installed_at_epoch_seconds = installation.installed_at_epoch_seconds;
        let mut next = self.clone();
        next.lifecycle = public(EquipmentLifecycle::Installed);
        next.installation = internal(Some(installation));
        next.updated_at_epoch_seconds = internal(installed_at_epoch_seconds);
        Ok(next)
    }

    pub fn transition_lifecycle(
        &self,
        next_lifecycle: EquipmentLifecycle,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !equipment_lifecycle_transition_allowed(self.lifecycle.value, next_lifecycle) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.lifecycle = public(next_lifecycle);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl CableEndpointTyped {
    pub fn new(input: CableEndpoint) -> Result<Self, CloudDcopsError> {
        let equipment_id = EquipmentId::new(input.equipment_id)?;
        validate_port(&input.port_name)?;
        Ok(Self {
            equipment_id,
            port_name: input.port_name,
        })
    }
}

impl CableRun {
    pub fn new(input: CableRunCreate) -> Result<Self, CloudDcopsError> {
        if input.state != CableState::Planned {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        validate_cable_loss(input.measured_loss_milli_db, input.loss_budget_milli_db)?;
        let id = CableRunId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            CABLE_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        let from = CableEndpointTyped::new(input.from)?;
        let to = CableEndpointTyped::new(input.to)?;
        if from == to {
            return Err(CloudDcopsError::InvalidPort);
        }
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            from: internal(from),
            to: internal(to),
            media: public(input.media),
            state: public(input.state),
            measured_loss_milli_db: internal(input.measured_loss_milli_db),
            loss_budget_milli_db: internal(input.loss_budget_milli_db),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_CABLE_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: CableState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !cable_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl BmsPoint {
    pub fn new(input: BmsPointCreate) -> Result<Self, CloudDcopsError> {
        if input.state != BmsPointState::Commissioning {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        validate_unit(&input.unit)?;
        let id = BmsPointId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            BMS_POINT_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        let equipment_id = input.equipment_id.map(EquipmentId::new).transpose()?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            equipment_id: internal(equipment_id),
            kind: public(input.kind),
            state: public(input.state),
            unit: public(input.unit),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(DCOPS_BMS_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: BmsPointState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !bms_point_transition_allowed(self.state.value, next_state) {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl BmsReading {
    pub fn new(input: BmsReadingCreate) -> Result<Self, CloudDcopsError> {
        validate_positive_time(input.observed_at_epoch_seconds)?;
        let point_id = BmsPointId::new(input.point_id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        Ok(Self {
            point_id: internal(point_id),
            site_id: internal(site_id),
            observed_at_epoch_seconds: internal(input.observed_at_epoch_seconds),
            milli_value: internal(input.milli_value),
            schema_version: public(DCOPS_BMS_SCHEMA_VERSION),
        })
    }
}

impl WorkOrder {
    pub fn new(input: WorkOrderCreate) -> Result<Self, CloudDcopsError> {
        if input.state != WorkOrderState::Open {
            return Err(CloudDcopsError::InvalidInitialState);
        }
        validate_positive_time(input.opened_at_epoch_seconds)?;
        validate_ref_path(
            &input.safety_plan_ref,
            SAFETY_PLAN_REF_PREFIX,
            CloudDcopsError::InvalidText,
        )?;
        validate_work_order_data_class(input.data_class)?;
        let id = WorkOrderId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            WORK_ORDER_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        let equipment_id = input.equipment_id.map(EquipmentId::new).transpose()?;
        let opened_by = PrincipalId::new(input.opened_by)?;
        let assigned_to = input.assigned_to.map(PrincipalId::new).transpose()?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            equipment_id: internal(equipment_id),
            kind: public(input.kind),
            priority: internal(input.priority),
            state: public(input.state),
            opened_by: internal(opened_by),
            assigned_to: internal(assigned_to),
            safety_plan_ref: internal(input.safety_plan_ref),
            data_class: public(input.data_class),
            resolution: audit(None),
            opened_at_epoch_seconds: internal(input.opened_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.opened_at_epoch_seconds),
            schema_version: public(DCOPS_WORK_ORDER_SCHEMA_VERSION),
        })
    }

    pub fn assign(
        &self,
        assigned_to: String,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if self.state.value != WorkOrderState::Open {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(WorkOrderState::Assigned);
        next.assigned_to = internal(Some(PrincipalId::new(assigned_to)?));
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }

    pub fn start(&self, updated_at_epoch_seconds: u64) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if self.state.value != WorkOrderState::Assigned || self.assigned_to.value.is_none() {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(WorkOrderState::InProgress);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }

    pub fn complete(&self, resolution: WorkOrderResolution) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            resolution.completed_at_epoch_seconds,
        )?;
        if self.state.value != WorkOrderState::InProgress {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        validate_ref_path(
            &resolution.resolution_ref,
            RESOLUTION_REF_PREFIX,
            CloudDcopsError::InvalidText,
        )?;
        let resolution = WorkOrderResolutionTyped {
            completed_by: PrincipalId::new(resolution.completed_by)?,
            resolution_ref: resolution.resolution_ref,
            completed_at_epoch_seconds: resolution.completed_at_epoch_seconds,
        };
        let mut next = self.clone();
        next.state = public(WorkOrderState::Completed);
        next.updated_at_epoch_seconds = internal(resolution.completed_at_epoch_seconds);
        next.resolution = audit(Some(resolution));
        Ok(next)
    }
}

impl SustainabilitySnapshot {
    pub fn new(input: SustainabilitySnapshotCreate) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            input.period_start_epoch_seconds,
            input.period_end_epoch_seconds,
        )?;
        validate_sustainability_data_class(input.data_class)?;
        if input.it_energy_kwh_milli == 0
            || input.facility_energy_kwh_milli < input.it_energy_kwh_milli
            || input.water_liters_milli == 0
            || input.carbon_grams == 0
        {
            return Err(CloudDcopsError::InvalidCapacity);
        }
        let expected_pue =
            exact_ratio_milli(input.facility_energy_kwh_milli, input.it_energy_kwh_milli)?;
        let expected_wue = exact_ratio_milli(input.water_liters_milli, input.it_energy_kwh_milli)?;
        let expected_cue = exact_ratio_milli(input.carbon_grams, input.it_energy_kwh_milli)?;
        if input.pue_milli != expected_pue
            || input.wue_milli != expected_wue
            || input.cue_milli != expected_cue
        {
            return Err(CloudDcopsError::InvalidTargetRatio);
        }
        let id = SustainabilitySnapshotId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            SUSTAINABILITY_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            period_start_epoch_seconds: internal(input.period_start_epoch_seconds),
            period_end_epoch_seconds: internal(input.period_end_epoch_seconds),
            it_energy_kwh_milli: internal(input.it_energy_kwh_milli),
            facility_energy_kwh_milli: internal(input.facility_energy_kwh_milli),
            water_liters_milli: internal(input.water_liters_milli),
            carbon_grams: internal(input.carbon_grams),
            pue_milli: internal(input.pue_milli),
            wue_milli: internal(input.wue_milli),
            cue_milli: internal(input.cue_milli),
            data_class: public(input.data_class),
            schema_version: public(DCOPS_SUSTAINABILITY_SCHEMA_VERSION),
        })
    }
}

impl CloudDcopsCatalog {
    pub fn with_bms_reading_retention_limit(bms_reading_retention_limit: usize) -> Self {
        Self {
            sites: BTreeMap::new(),
            facility_zones: BTreeMap::new(),
            power_zones: BTreeMap::new(),
            cooling_zones: BTreeMap::new(),
            security_zones: BTreeMap::new(),
            racks: BTreeMap::new(),
            equipment: BTreeMap::new(),
            cable_runs: BTreeMap::new(),
            bms_points: BTreeMap::new(),
            bms_readings: BTreeSet::new(),
            bms_reading_retention_limit: bms_reading_retention_limit.max(1),
            work_orders: BTreeMap::new(),
            sustainability_snapshots: BTreeMap::new(),
            rack_capacity_by_id: BTreeMap::new(),
            power_zone_used_watts_by_id: BTreeMap::new(),
            cooling_zone_used_watts_by_id: BTreeMap::new(),
            rack_unit_allocations_by_id: BTreeMap::new(),
        }
    }

    pub fn bms_reading_count(&self) -> usize {
        self.bms_readings.len()
    }

    fn remember_bms_reading(&mut self, key: (BmsPointId, u64)) -> bool {
        if self.bms_readings.contains(&key) {
            return false;
        }
        if self.bms_readings.len() >= self.bms_reading_retention_limit
            && let Some(evicted) = self.bms_readings.iter().next().cloned()
        {
            self.bms_readings.remove(&evicted);
        }
        self.bms_readings.insert(key)
    }

    pub fn add_site(
        &mut self,
        input: DatacenterSiteCreate,
    ) -> Result<DatacenterSite, CloudDcopsError> {
        let site = DatacenterSite::new(input)?;
        if self.sites.contains_key(&site.id.value) {
            return Err(CloudDcopsError::DuplicateSite);
        }
        self.sites.insert(site.id.value.clone(), site.clone());
        Ok(site)
    }

    pub fn transition_site(
        &mut self,
        site_id: &DatacenterSiteId,
        next_state: DatacenterState,
        updated_at_epoch_seconds: u64,
    ) -> Result<DatacenterSite, CloudDcopsError> {
        let site = self
            .sites
            .get(site_id)
            .ok_or(CloudDcopsError::UnknownSite)?;
        let next = site.transition(next_state, updated_at_epoch_seconds)?;
        self.sites.insert(site_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_facility_zone(
        &mut self,
        input: FacilityZoneCreate,
    ) -> Result<FacilityZone, CloudDcopsError> {
        let zone = FacilityZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.facility_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicateFacilityZone);
        }
        self.facility_zones
            .insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_facility_zone(
        &mut self,
        zone_id: &FacilityZoneId,
        next_state: FacilityZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<FacilityZone, CloudDcopsError> {
        let zone = self
            .facility_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownFacilityZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.facility_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_power_zone(&mut self, input: PowerZoneCreate) -> Result<PowerZone, CloudDcopsError> {
        let zone = PowerZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.power_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicatePowerZone);
        }
        self.power_zones.insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_power_zone(
        &mut self,
        zone_id: &PowerZoneId,
        next_state: PowerZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<PowerZone, CloudDcopsError> {
        let zone = self
            .power_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownPowerZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.power_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_cooling_zone(
        &mut self,
        input: CoolingZoneCreate,
    ) -> Result<CoolingZone, CloudDcopsError> {
        let zone = CoolingZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.cooling_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicateCoolingZone);
        }
        self.cooling_zones
            .insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_cooling_zone(
        &mut self,
        zone_id: &CoolingZoneId,
        next_state: CoolingZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<CoolingZone, CloudDcopsError> {
        let zone = self
            .cooling_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownCoolingZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.cooling_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_security_zone(
        &mut self,
        input: SecurityZoneCreate,
    ) -> Result<SecurityZone, CloudDcopsError> {
        let zone = SecurityZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.security_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicateSecurityZone);
        }
        self.security_zones
            .insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_security_zone(
        &mut self,
        zone_id: &SecurityZoneId,
        next_state: SecurityZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<SecurityZone, CloudDcopsError> {
        let zone = self
            .security_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownSecurityZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.security_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_rack(&mut self, input: RackCreate) -> Result<Rack, CloudDcopsError> {
        let rack = Rack::new(input)?;
        self.require_active_site(&rack.site_id.value)?;
        let facility_zone = self
            .facility_zones
            .get(&rack.facility_zone_id.value)
            .ok_or(CloudDcopsError::UnknownFacilityZone)?;
        let security_zone = self
            .security_zones
            .get(&rack.security_zone_id.value)
            .ok_or(CloudDcopsError::UnknownSecurityZone)?;
        validate_same_site(&rack.site_id.value, &facility_zone.site_id.value)?;
        validate_same_site(&rack.site_id.value, &security_zone.site_id.value)?;
        if facility_zone.state.value != FacilityZoneState::Active
            || security_zone.state.value != SecurityZoneState::Armed
        {
            return Err(CloudDcopsError::InactiveParent);
        }
        if self.racks.contains_key(&rack.id.value) {
            return Err(CloudDcopsError::DuplicateRack);
        }
        self.racks.insert(rack.id.value.clone(), rack.clone());
        Ok(rack)
    }

    pub fn transition_rack(
        &mut self,
        rack_id: &RackId,
        next_state: RackState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Rack, CloudDcopsError> {
        let rack = self
            .racks
            .get(rack_id)
            .ok_or(CloudDcopsError::UnknownRack)?;
        let next = rack.transition(next_state, updated_at_epoch_seconds)?;
        self.racks.insert(rack_id.clone(), next.clone());
        Ok(next)
    }

    pub fn order_equipment(
        &mut self,
        input: EquipmentCreate,
    ) -> Result<Equipment, CloudDcopsError> {
        let equipment = Equipment::new(input)?;
        self.require_active_site(&equipment.site_id.value)?;
        if self.equipment.contains_key(&equipment.id.value) {
            return Err(CloudDcopsError::DuplicateEquipment);
        }
        self.equipment
            .insert(equipment.id.value.clone(), equipment.clone());
        Ok(equipment)
    }

    pub fn receive_equipment(
        &mut self,
        equipment_id: &EquipmentId,
        asset_tag: String,
        serial_number: String,
        received_at_epoch_seconds: u64,
    ) -> Result<Equipment, CloudDcopsError> {
        let current = self
            .equipment
            .get(equipment_id)
            .ok_or(CloudDcopsError::UnknownEquipment)?;
        let next = current.receive(asset_tag, serial_number, received_at_epoch_seconds)?;
        self.equipment.insert(equipment_id.clone(), next.clone());
        Ok(next)
    }

    pub fn install_equipment(
        &mut self,
        equipment_id: &EquipmentId,
        input: EquipmentInstallPlan,
    ) -> Result<Equipment, CloudDcopsError> {
        let current = self
            .equipment
            .get(equipment_id)
            .cloned()
            .ok_or(CloudDcopsError::UnknownEquipment)?;
        if current.lifecycle.value != EquipmentLifecycle::Received {
            return Err(CloudDcopsError::InvalidStateTransition);
        }
        let installation = input.typed(current.kind.value)?;
        self.validate_installation(equipment_id, &current.site_id.value, &installation)?;
        let next = current.install(installation)?;
        self.apply_capacity_accounting(equipment_id, &next)?;
        self.equipment.insert(equipment_id.clone(), next.clone());
        Ok(next)
    }

    pub fn transition_equipment(
        &mut self,
        equipment_id: &EquipmentId,
        next_lifecycle: EquipmentLifecycle,
        updated_at_epoch_seconds: u64,
    ) -> Result<Equipment, CloudDcopsError> {
        let current = self
            .equipment
            .get(equipment_id)
            .cloned()
            .ok_or(CloudDcopsError::UnknownEquipment)?;
        let next = current.transition_lifecycle(next_lifecycle, updated_at_epoch_seconds)?;
        if equipment_counts_against_capacity(&current) && !equipment_counts_against_capacity(&next)
        {
            self.release_capacity_accounting(equipment_id, &current)?;
        } else if !equipment_counts_against_capacity(&current)
            && equipment_counts_against_capacity(&next)
        {
            self.apply_capacity_accounting(equipment_id, &next)?;
        }
        self.equipment.insert(equipment_id.clone(), next.clone());
        Ok(next)
    }

    pub fn rack_capacity(&self, rack_id: &RackId) -> Result<RackCapacitySnapshot, CloudDcopsError> {
        let rack = self
            .racks
            .get(rack_id)
            .ok_or(CloudDcopsError::UnknownRack)?;
        let mut capacity = self
            .rack_capacity_by_id
            .get(rack_id)
            .copied()
            .unwrap_or_default();
        finalize_rack_capacity(rack, &mut capacity);
        Ok(capacity)
    }

    pub fn add_cable_run(&mut self, input: CableRunCreate) -> Result<CableRun, CloudDcopsError> {
        let cable = CableRun::new(input)?;
        self.require_active_site(&cable.site_id.value)?;
        let from = self
            .equipment
            .get(&cable.from.value.equipment_id)
            .ok_or(CloudDcopsError::UnknownEquipment)?;
        let to = self
            .equipment
            .get(&cable.to.value.equipment_id)
            .ok_or(CloudDcopsError::UnknownEquipment)?;
        validate_same_site(&cable.site_id.value, &from.site_id.value)?;
        validate_same_site(&cable.site_id.value, &to.site_id.value)?;
        if self.cable_runs.contains_key(&cable.id.value) {
            return Err(CloudDcopsError::DuplicateCableRun);
        }
        self.cable_runs
            .insert(cable.id.value.clone(), cable.clone());
        Ok(cable)
    }

    pub fn transition_cable_run(
        &mut self,
        cable_id: &CableRunId,
        next_state: CableState,
        updated_at_epoch_seconds: u64,
    ) -> Result<CableRun, CloudDcopsError> {
        let cable = self
            .cable_runs
            .get(cable_id)
            .ok_or(CloudDcopsError::UnknownCableRun)?;
        let next = cable.transition(next_state, updated_at_epoch_seconds)?;
        self.cable_runs.insert(cable_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_bms_point(&mut self, input: BmsPointCreate) -> Result<BmsPoint, CloudDcopsError> {
        let point = BmsPoint::new(input)?;
        self.require_active_site(&point.site_id.value)?;
        if let Some(equipment_id) = point.equipment_id.value.as_ref() {
            let equipment = self
                .equipment
                .get(equipment_id)
                .ok_or(CloudDcopsError::UnknownEquipment)?;
            validate_same_site(&point.site_id.value, &equipment.site_id.value)?;
        }
        if self.bms_points.contains_key(&point.id.value) {
            return Err(CloudDcopsError::DuplicateBmsPoint);
        }
        self.bms_points
            .insert(point.id.value.clone(), point.clone());
        Ok(point)
    }

    pub fn transition_bms_point(
        &mut self,
        point_id: &BmsPointId,
        next_state: BmsPointState,
        updated_at_epoch_seconds: u64,
    ) -> Result<BmsPoint, CloudDcopsError> {
        let point = self
            .bms_points
            .get(point_id)
            .ok_or(CloudDcopsError::UnknownBmsPoint)?;
        let next = point.transition(next_state, updated_at_epoch_seconds)?;
        self.bms_points.insert(point_id.clone(), next.clone());
        Ok(next)
    }

    pub fn record_bms_reading(
        &mut self,
        input: BmsReadingCreate,
    ) -> Result<BmsReading, CloudDcopsError> {
        let reading = BmsReading::new(input)?;
        let point = self
            .bms_points
            .get(&reading.point_id.value)
            .ok_or(CloudDcopsError::UnknownBmsPoint)?;
        validate_same_site(&reading.site_id.value, &point.site_id.value)?;
        if point.state.value != BmsPointState::Enabled {
            return Err(CloudDcopsError::InactiveParent);
        }
        let key = (
            reading.point_id.value.clone(),
            reading.observed_at_epoch_seconds.value,
        );
        if !self.remember_bms_reading(key) {
            return Err(CloudDcopsError::DuplicateBmsReading);
        }
        Ok(reading)
    }

    pub fn open_work_order(
        &mut self,
        input: WorkOrderCreate,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let work_order = WorkOrder::new(input)?;
        self.require_active_site(&work_order.site_id.value)?;
        if let Some(equipment_id) = work_order.equipment_id.value.as_ref() {
            let equipment = self
                .equipment
                .get(equipment_id)
                .ok_or(CloudDcopsError::UnknownEquipment)?;
            validate_same_site(&work_order.site_id.value, &equipment.site_id.value)?;
        }
        if self.work_orders.contains_key(&work_order.id.value) {
            return Err(CloudDcopsError::DuplicateWorkOrder);
        }
        self.work_orders
            .insert(work_order.id.value.clone(), work_order.clone());
        Ok(work_order)
    }

    pub fn assign_work_order(
        &mut self,
        work_order_id: &WorkOrderId,
        assigned_to: String,
        updated_at_epoch_seconds: u64,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let current = self
            .work_orders
            .get(work_order_id)
            .ok_or(CloudDcopsError::UnknownWorkOrder)?;
        let next = current.assign(assigned_to, updated_at_epoch_seconds)?;
        self.work_orders.insert(work_order_id.clone(), next.clone());
        Ok(next)
    }

    pub fn start_work_order(
        &mut self,
        work_order_id: &WorkOrderId,
        updated_at_epoch_seconds: u64,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let current = self
            .work_orders
            .get(work_order_id)
            .ok_or(CloudDcopsError::UnknownWorkOrder)?;
        let next = current.start(updated_at_epoch_seconds)?;
        self.work_orders.insert(work_order_id.clone(), next.clone());
        Ok(next)
    }

    pub fn complete_work_order(
        &mut self,
        work_order_id: &WorkOrderId,
        resolution: WorkOrderResolution,
    ) -> Result<WorkOrder, CloudDcopsError> {
        let current = self
            .work_orders
            .get(work_order_id)
            .ok_or(CloudDcopsError::UnknownWorkOrder)?;
        let next = current.complete(resolution)?;
        self.work_orders.insert(work_order_id.clone(), next.clone());
        Ok(next)
    }

    pub fn record_sustainability_snapshot(
        &mut self,
        input: SustainabilitySnapshotCreate,
    ) -> Result<SustainabilitySnapshot, CloudDcopsError> {
        let snapshot = SustainabilitySnapshot::new(input)?;
        let site = self
            .sites
            .get(&snapshot.site_id.value)
            .ok_or(CloudDcopsError::UnknownSite)?;
        if site.state.value != DatacenterState::Active {
            return Err(CloudDcopsError::InactiveParent);
        }
        if snapshot.pue_milli.value > u64::from(site.pue_target_milli.value)
            || snapshot.wue_milli.value > u64::from(site.wue_target_milli.value)
            || snapshot.cue_milli.value > u64::from(site.cue_target_milli.value)
        {
            return Err(CloudDcopsError::InvalidTargetRatio);
        }
        if self
            .sustainability_snapshots
            .contains_key(&snapshot.id.value)
        {
            return Err(CloudDcopsError::DuplicateSustainabilitySnapshot);
        }
        self.sustainability_snapshots
            .insert(snapshot.id.value.clone(), snapshot.clone());
        Ok(snapshot)
    }

    pub fn sites(&self) -> impl Iterator<Item = &DatacenterSite> {
        self.sites.values()
    }

    pub fn equipment(&self) -> impl Iterator<Item = &Equipment> {
        self.equipment.values()
    }

    fn require_active_site(&self, site_id: &DatacenterSiteId) -> Result<(), CloudDcopsError> {
        let site = self
            .sites
            .get(site_id)
            .ok_or(CloudDcopsError::UnknownSite)?;
        if site.state.value == DatacenterState::Active {
            Ok(())
        } else {
            Err(CloudDcopsError::InactiveParent)
        }
    }

    fn validate_installation(
        &self,
        equipment_id: &EquipmentId,
        site_id: &DatacenterSiteId,
        installation: &EquipmentInstallation,
    ) -> Result<(), CloudDcopsError> {
        let rack = self
            .racks
            .get(&installation.rack_id)
            .ok_or(CloudDcopsError::UnknownRack)?;
        let power_zone = self
            .power_zones
            .get(&installation.power_zone_id)
            .ok_or(CloudDcopsError::UnknownPowerZone)?;
        let cooling_zone = self
            .cooling_zones
            .get(&installation.cooling_zone_id)
            .ok_or(CloudDcopsError::UnknownCoolingZone)?;
        validate_same_site(site_id, &rack.site_id.value)?;
        validate_same_site(site_id, &power_zone.site_id.value)?;
        validate_same_site(site_id, &cooling_zone.site_id.value)?;
        if rack.state.value != RackState::Active
            || power_zone.state.value != PowerZoneState::Energized
            || cooling_zone.state.value != CoolingZoneState::Active
        {
            return Err(CloudDcopsError::InactiveParent);
        }
        let end_u = installation_end_u(installation)?;
        if end_u > rack.u_height.value {
            return Err(CloudDcopsError::InvalidRackUnits);
        }
        let accounting = self.installation_accounting(
            rack,
            &installation.power_zone_id,
            &installation.cooling_zone_id,
            equipment_id,
            Some(installation),
        )?;
        if accounting.rack_unit_overlap {
            return Err(CloudDcopsError::RackUnitOverlap);
        }
        if accounting.rack_capacity.used_u > rack.u_height.value
            || accounting.rack_capacity.used_power_watts > rack.rated_power_watts.value
            || accounting.rack_capacity.used_heat_watts > rack.max_heat_watts.value
            || accounting.rack_capacity.used_weight_kg > rack.max_weight_kg.value
        {
            return Err(CloudDcopsError::RackCapacityExceeded);
        }
        if accounting.power_zone_used_watts > power_zone.capacity_watts.value {
            return Err(CloudDcopsError::PowerZoneCapacityExceeded);
        }
        if accounting.cooling_zone_used_watts > cooling_zone.heat_capacity_watts.value {
            return Err(CloudDcopsError::CoolingZoneCapacityExceeded);
        }
        Ok(())
    }

    fn installation_accounting(
        &self,
        rack: &Rack,
        power_zone_id: &PowerZoneId,
        cooling_zone_id: &CoolingZoneId,
        equipment_id: &EquipmentId,
        proposed: Option<&EquipmentInstallation>,
    ) -> Result<InstallationAccounting, CloudDcopsError> {
        let mut accounting = InstallationAccounting {
            rack_capacity: self
                .rack_capacity_by_id
                .get(&rack.id.value)
                .copied()
                .unwrap_or_default(),
            power_zone_used_watts: self
                .power_zone_used_watts_by_id
                .get(power_zone_id)
                .copied()
                .unwrap_or_default(),
            cooling_zone_used_watts: self
                .cooling_zone_used_watts_by_id
                .get(cooling_zone_id)
                .copied()
                .unwrap_or_default(),
            rack_unit_overlap: false,
        };
        let proposed_end_u = proposed.map(installation_end_u).transpose()?;
        if let (Some(proposed), Some(proposed_end_u), Some(allocations)) = (
            proposed,
            proposed_end_u,
            self.rack_unit_allocations_by_id.get(&rack.id.value),
        ) {
            for (allocated_id, (allocated_start_u, allocated_end_u)) in allocations {
                if allocated_id != equipment_id
                    && u_ranges_overlap(
                        proposed.start_u,
                        proposed_end_u,
                        *allocated_start_u,
                        *allocated_end_u,
                    )
                {
                    accounting.rack_unit_overlap = true;
                    break;
                }
            }
        }
        if let Some(installation) = proposed {
            if installation.rack_id == rack.id.value {
                add_installation_capacity(&mut accounting.rack_capacity, installation);
            }
            if installation.power_zone_id == *power_zone_id {
                accounting.power_zone_used_watts = accounting
                    .power_zone_used_watts
                    .saturating_add(installation.power_watts);
            }
            if installation.cooling_zone_id == *cooling_zone_id {
                accounting.cooling_zone_used_watts = accounting
                    .cooling_zone_used_watts
                    .saturating_add(installation.heat_watts);
            }
        }
        finalize_rack_capacity(rack, &mut accounting.rack_capacity);
        Ok(accounting)
    }

    fn apply_capacity_accounting(
        &mut self,
        equipment_id: &EquipmentId,
        equipment: &Equipment,
    ) -> Result<(), CloudDcopsError> {
        if !equipment_counts_against_capacity(equipment) {
            return Ok(());
        }
        let Some(installation) = equipment.installation.value.as_ref() else {
            return Ok(());
        };
        let end_u = installation_end_u(installation)?;
        add_installation_capacity(
            self.rack_capacity_by_id
                .entry(installation.rack_id.clone())
                .or_default(),
            installation,
        );
        let power_used = self
            .power_zone_used_watts_by_id
            .entry(installation.power_zone_id.clone())
            .or_default();
        *power_used = (*power_used).saturating_add(installation.power_watts);
        let cooling_used = self
            .cooling_zone_used_watts_by_id
            .entry(installation.cooling_zone_id.clone())
            .or_default();
        *cooling_used = (*cooling_used).saturating_add(installation.heat_watts);
        self.rack_unit_allocations_by_id
            .entry(installation.rack_id.clone())
            .or_default()
            .insert(equipment_id.clone(), (installation.start_u, end_u));
        Ok(())
    }

    fn release_capacity_accounting(
        &mut self,
        equipment_id: &EquipmentId,
        equipment: &Equipment,
    ) -> Result<(), CloudDcopsError> {
        if !equipment_counts_against_capacity(equipment) {
            return Ok(());
        }
        let Some(installation) = equipment.installation.value.as_ref() else {
            return Ok(());
        };
        let remove_rack =
            if let Some(capacity) = self.rack_capacity_by_id.get_mut(&installation.rack_id) {
                subtract_installation_capacity(capacity, installation);
                capacity.used_u == 0
                    && capacity.used_power_watts == 0
                    && capacity.used_heat_watts == 0
                    && capacity.used_weight_kg == 0
            } else {
                false
            };
        if remove_rack {
            self.rack_capacity_by_id.remove(&installation.rack_id);
        }
        let remove_power = if let Some(used) = self
            .power_zone_used_watts_by_id
            .get_mut(&installation.power_zone_id)
        {
            *used = used.saturating_sub(installation.power_watts);
            *used == 0
        } else {
            false
        };
        if remove_power {
            self.power_zone_used_watts_by_id
                .remove(&installation.power_zone_id);
        }
        let remove_cooling = if let Some(used) = self
            .cooling_zone_used_watts_by_id
            .get_mut(&installation.cooling_zone_id)
        {
            *used = used.saturating_sub(installation.heat_watts);
            *used == 0
        } else {
            false
        };
        if remove_cooling {
            self.cooling_zone_used_watts_by_id
                .remove(&installation.cooling_zone_id);
        }
        let remove_allocations = if let Some(allocations) = self
            .rack_unit_allocations_by_id
            .get_mut(&installation.rack_id)
        {
            allocations.remove(equipment_id);
            allocations.is_empty()
        } else {
            false
        };
        if remove_allocations {
            self.rack_unit_allocations_by_id
                .remove(&installation.rack_id);
        }
        Ok(())
    }
}

fn equipment_counts_against_capacity(equipment: &Equipment) -> bool {
    equipment.lifecycle.value != EquipmentLifecycle::EwasteTransferred
        && equipment.installation.value.is_some()
}

fn add_installation_capacity(
    capacity: &mut RackCapacitySnapshot,
    installation: &EquipmentInstallation,
) {
    capacity.used_u = capacity.used_u.saturating_add(installation.height_u);
    capacity.used_power_watts = capacity
        .used_power_watts
        .saturating_add(installation.power_watts);
    capacity.used_heat_watts = capacity
        .used_heat_watts
        .saturating_add(installation.heat_watts);
    capacity.used_weight_kg = capacity
        .used_weight_kg
        .saturating_add(installation.weight_kg);
}

fn subtract_installation_capacity(
    capacity: &mut RackCapacitySnapshot,
    installation: &EquipmentInstallation,
) {
    capacity.used_u = capacity.used_u.saturating_sub(installation.height_u);
    capacity.used_power_watts = capacity
        .used_power_watts
        .saturating_sub(installation.power_watts);
    capacity.used_heat_watts = capacity
        .used_heat_watts
        .saturating_sub(installation.heat_watts);
    capacity.used_weight_kg = capacity
        .used_weight_kg
        .saturating_sub(installation.weight_kg);
}

fn finalize_rack_capacity(rack: &Rack, capacity: &mut RackCapacitySnapshot) {
    capacity.free_u = rack.u_height.value.saturating_sub(capacity.used_u);
    capacity.remaining_power_watts = rack
        .rated_power_watts
        .value
        .saturating_sub(capacity.used_power_watts);
    capacity.remaining_heat_watts = rack
        .max_heat_watts
        .value
        .saturating_sub(capacity.used_heat_watts);
    capacity.remaining_weight_kg = rack
        .max_weight_kg
        .value
        .saturating_sub(capacity.used_weight_kg);
}

fn datacenter_transition_allowed(current: DatacenterState, next: DatacenterState) -> bool {
    matches!(
        (current, next),
        (DatacenterState::Planned, DatacenterState::Commissioning)
            | (DatacenterState::Commissioning, DatacenterState::Active)
            | (DatacenterState::Active, DatacenterState::Draining)
            | (DatacenterState::Draining, DatacenterState::Retired)
    )
}

fn facility_zone_transition_allowed(current: FacilityZoneState, next: FacilityZoneState) -> bool {
    matches!(
        (current, next),
        (FacilityZoneState::Planned, FacilityZoneState::Active)
            | (FacilityZoneState::Active, FacilityZoneState::Isolated)
            | (FacilityZoneState::Isolated, FacilityZoneState::Active)
            | (FacilityZoneState::Isolated, FacilityZoneState::Retired)
    )
}

fn power_zone_transition_allowed(current: PowerZoneState, next: PowerZoneState) -> bool {
    matches!(
        (current, next),
        (PowerZoneState::Planned, PowerZoneState::Energized)
            | (PowerZoneState::Energized, PowerZoneState::Maintenance)
            | (PowerZoneState::Maintenance, PowerZoneState::Energized)
            | (PowerZoneState::Maintenance, PowerZoneState::Retired)
    )
}

fn cooling_zone_transition_allowed(current: CoolingZoneState, next: CoolingZoneState) -> bool {
    matches!(
        (current, next),
        (CoolingZoneState::Planned, CoolingZoneState::Active)
            | (CoolingZoneState::Active, CoolingZoneState::Maintenance)
            | (CoolingZoneState::Maintenance, CoolingZoneState::Active)
            | (CoolingZoneState::Maintenance, CoolingZoneState::Retired)
    )
}

fn security_zone_transition_allowed(current: SecurityZoneState, next: SecurityZoneState) -> bool {
    matches!(
        (current, next),
        (SecurityZoneState::Planned, SecurityZoneState::Armed)
            | (SecurityZoneState::Armed, SecurityZoneState::Isolated)
            | (SecurityZoneState::Isolated, SecurityZoneState::Armed)
            | (SecurityZoneState::Isolated, SecurityZoneState::Retired)
    )
}

fn rack_transition_allowed(current: RackState, next: RackState) -> bool {
    matches!(
        (current, next),
        (RackState::Planned, RackState::Active)
            | (RackState::Active, RackState::Quarantined)
            | (RackState::Quarantined, RackState::Active)
            | (RackState::Quarantined, RackState::Retired)
    )
}

fn equipment_lifecycle_transition_allowed(
    current: EquipmentLifecycle,
    next: EquipmentLifecycle,
) -> bool {
    matches!(
        (current, next),
        (EquipmentLifecycle::Installed, EquipmentLifecycle::InService)
            | (
                EquipmentLifecycle::InService,
                EquipmentLifecycle::Maintenance
            )
            | (
                EquipmentLifecycle::Maintenance,
                EquipmentLifecycle::InService
            )
            | (
                EquipmentLifecycle::InService,
                EquipmentLifecycle::Decommissioning
            )
            | (
                EquipmentLifecycle::Maintenance,
                EquipmentLifecycle::Decommissioning
            )
            | (
                EquipmentLifecycle::Decommissioning,
                EquipmentLifecycle::Sanitized
            )
            | (
                EquipmentLifecycle::Sanitized,
                EquipmentLifecycle::EwasteTransferred
            )
    )
}

fn cable_transition_allowed(current: CableState, next: CableState) -> bool {
    matches!(
        (current, next),
        (CableState::Planned, CableState::Installed)
            | (CableState::Installed, CableState::Certified)
            | (CableState::Certified, CableState::Retired)
    )
}

fn bms_point_transition_allowed(current: BmsPointState, next: BmsPointState) -> bool {
    matches!(
        (current, next),
        (BmsPointState::Commissioning, BmsPointState::Enabled)
            | (BmsPointState::Enabled, BmsPointState::Disabled)
            | (BmsPointState::Disabled, BmsPointState::Enabled)
            | (BmsPointState::Disabled, BmsPointState::Retired)
    )
}

fn validate_path(
    value: &str,
    prefix: &str,
    min_segments: usize,
    error: CloudDcopsError,
) -> Result<(), CloudDcopsError> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() < min_segments || parts.first().copied() != Some(prefix) {
        return Err(error);
    }
    for part in parts.iter().skip(1) {
        validate_path_segment(part).map_err(|_| error.clone())?;
    }
    Ok(())
}

fn validate_child_id(
    value: &str,
    prefix: &str,
    parent_id: &str,
    error: CloudDcopsError,
) -> Result<(), CloudDcopsError> {
    let required = format!("{prefix}/{parent_id}/");
    if value.starts_with(&required) {
        Ok(())
    } else {
        Err(error)
    }
}

fn validate_path_segment(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Err(())
    } else {
        Ok(())
    }
}

fn site_region_from_path(value: &str) -> Result<&str, CloudDcopsError> {
    value
        .split('/')
        .nth(1)
        .ok_or(CloudDcopsError::InvalidDatacenterSiteId)
}

fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudDcopsError> {
    let prefix = format!("{}-", region.value);
    if az.value.starts_with(&prefix) && az.value.len() > prefix.len() {
        Ok(())
    } else {
        Err(CloudDcopsError::AzRegionMismatch)
    }
}

fn validate_physical_ref(value: &str) -> Result<(), CloudDcopsError> {
    validate_ref_path(
        value,
        PHYSICAL_REF_PREFIX,
        CloudDcopsError::InvalidPhysicalRef,
    )
}

fn validate_ref_path(
    value: &str,
    prefix: &str,
    error: CloudDcopsError,
) -> Result<(), CloudDcopsError> {
    validate_path(value, prefix, 3, error)
}

fn validate_positive_time(value: u64) -> Result<(), CloudDcopsError> {
    if value == 0 {
        Err(CloudDcopsError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), CloudDcopsError> {
    if start == 0 || end <= start {
        Err(CloudDcopsError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_sustainability_targets(
    pue_target_milli: u32,
    wue_target_milli: u32,
    cue_target_milli: u32,
) -> Result<(), CloudDcopsError> {
    if !(1_000..=2_500).contains(&pue_target_milli)
        || wue_target_milli == 0
        || wue_target_milli > 10_000
        || cue_target_milli == 0
        || cue_target_milli > 10_000
    {
        Err(CloudDcopsError::InvalidTargetRatio)
    } else {
        Ok(())
    }
}

fn validate_power_redundancy(
    redundancy: PowerRedundancy,
    utility_feed_count: u8,
) -> Result<(), CloudDcopsError> {
    let required = match redundancy {
        PowerRedundancy::N => 1,
        PowerRedundancy::NPlusOne | PowerRedundancy::TwoN => 2,
        PowerRedundancy::TwoNPlusOne => 3,
    };
    if utility_feed_count >= required {
        Ok(())
    } else {
        Err(CloudDcopsError::InvalidRedundancy)
    }
}

fn validate_rack_shape(
    u_height: u16,
    rated_power_watts: u64,
    max_heat_watts: u64,
    max_weight_kg: u64,
) -> Result<(), CloudDcopsError> {
    if !(MIN_RACK_U_HEIGHT..=MAX_RACK_U_HEIGHT).contains(&u_height)
        || rated_power_watts == 0
        || max_heat_watts == 0
        || max_weight_kg == 0
    {
        Err(CloudDcopsError::InvalidCapacity)
    } else {
        Ok(())
    }
}

fn validate_install_shape(
    input: &EquipmentInstallPlan,
    kind: EquipmentKind,
) -> Result<(), CloudDcopsError> {
    validate_positive_time(input.installed_at_epoch_seconds)?;
    if input.start_u == 0 || input.height_u == 0 || input.weight_kg == 0 {
        return Err(CloudDcopsError::InvalidInstallPlan);
    }
    if kind.requires_power() && (input.power_watts == 0 || input.heat_watts == 0) {
        return Err(CloudDcopsError::InvalidInstallPlan);
    }
    if !kind.requires_power() && input.heat_watts > 0 && input.power_watts == 0 {
        return Err(CloudDcopsError::InvalidInstallPlan);
    }
    typed_network_drop_refs(&input.network_drop_refs)?;
    Ok(())
}

fn typed_network_drop_refs(values: &[String]) -> Result<Vec<String>, CloudDcopsError> {
    let mut seen = BTreeSet::new();
    let mut refs = Vec::with_capacity(values.len());
    for value in values {
        validate_ref_path(value, "netdrop", CloudDcopsError::InvalidInstallPlan)?;
        if !seen.insert(value.clone()) {
            return Err(CloudDcopsError::InvalidInstallPlan);
        }
        refs.push(value.clone());
    }
    Ok(refs)
}

fn installation_end_u(installation: &EquipmentInstallation) -> Result<u16, CloudDcopsError> {
    installation
        .start_u
        .checked_add(installation.height_u)
        .and_then(|value| value.checked_sub(1))
        .ok_or(CloudDcopsError::InvalidRackUnits)
}

fn u_ranges_overlap(start_a: u16, end_a: u16, start_b: u16, end_b: u16) -> bool {
    start_a <= end_b && start_b <= end_a
}

fn validate_port(value: &str) -> Result<(), CloudDcopsError> {
    if value.trim().is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'/' | b'.'))
    {
        Err(CloudDcopsError::InvalidPort)
    } else {
        Ok(())
    }
}

fn validate_cable_loss(
    measured_milli_db: u32,
    budget_milli_db: u32,
) -> Result<(), CloudDcopsError> {
    if budget_milli_db == 0
        || budget_milli_db > MAX_FIBER_LOSS_MILLI_DB
        || measured_milli_db > budget_milli_db
    {
        Err(CloudDcopsError::InvalidCableLoss)
    } else {
        Ok(())
    }
}

fn validate_unit(value: &str) -> Result<(), CloudDcopsError> {
    if matches!(
        value,
        "milli-celsius" | "milli-percent" | "milli-liter" | "milli-watt" | "boolean"
    ) {
        Ok(())
    } else {
        Err(CloudDcopsError::InvalidBmsReading)
    }
}

fn validate_work_order_data_class(data_class: DataClass) -> Result<(), CloudDcopsError> {
    match data_class {
        DataClass::InternalOnly | DataClass::PiiIdentifying | DataClass::PiiQuasiIdentifier => {
            Ok(())
        }
        _ => Err(CloudDcopsError::InvalidDataClass),
    }
}

fn validate_sustainability_data_class(data_class: DataClass) -> Result<(), CloudDcopsError> {
    match data_class {
        DataClass::InternalOnly | DataClass::Financial | DataClass::BehavioralTenantProduct => {
            Ok(())
        }
        _ => Err(CloudDcopsError::InvalidDataClass),
    }
}

fn exact_ratio_milli(numerator: u64, denominator: u64) -> Result<u64, CloudDcopsError> {
    let scaled = numerator
        .checked_mul(1_000)
        .ok_or(CloudDcopsError::InvalidTargetRatio)?;
    if denominator == 0 || scaled % denominator != 0 {
        return Err(CloudDcopsError::InvalidTargetRatio);
    }
    Ok(scaled / denominator)
}

fn validate_non_empty(value: &str) -> Result<(), CloudDcopsError> {
    if value.trim().is_empty() {
        Err(CloudDcopsError::InvalidText)
    } else {
        Ok(())
    }
}

fn validate_same_site(
    expected: &DatacenterSiteId,
    actual: &DatacenterSiteId,
) -> Result<(), CloudDcopsError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CloudDcopsError::CrossSiteReference)
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

fn audit<T>(value: T) -> Classified<T> {
    Classified::new(value, OperationalDataClass::Audit)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITE_ID: &str = "dc/region-alpha1/site-a";
    const HALL_ID: &str = "zone/dc/region-alpha1/site-a/hall-a";
    const POWER_ID: &str = "power/dc/region-alpha1/site-a/power-a";
    const COOLING_ID: &str = "cooling/dc/region-alpha1/site-a/cooling-a";
    const SECURITY_ID: &str = "security/dc/region-alpha1/site-a/sec-a";
    const RACK_ID_VALUE: &str = "rack/dc/region-alpha1/site-a/rack-a";
    const EQUIP_ID: &str = "equip/dc/region-alpha1/site-a/server-a";
    const EQUIP_ID_B: &str = "equip/dc/region-alpha1/site-a/server-b";

    fn site_create() -> DatacenterSiteCreate {
        DatacenterSiteCreate {
            id: SITE_ID.to_string(),
            region: "region-alpha1".to_string(),
            availability_zone: "region-alpha1-a".to_string(),
            physical_ref: "physical/colo/site-a".to_string(),
            phase: DcSubstratePhase::ColoCage,
            tier: DatacenterTier::Tier3,
            state: DatacenterState::Planned,
            provider_facing: true,
            pue_target_milli: 1_500,
            wue_target_milli: 2_000,
            cue_target_milli: 1_000,
            created_at_epoch_seconds: 1,
        }
    }

    fn active_catalog() -> CloudDcopsCatalog {
        let mut catalog = CloudDcopsCatalog::default();
        let site = catalog.add_site(site_create()).expect("site");
        catalog
            .transition_site(&site.id.value, DatacenterState::Commissioning, 2)
            .expect("commissioning");
        catalog
            .transition_site(&site.id.value, DatacenterState::Active, 3)
            .expect("active");
        let hall = catalog
            .add_facility_zone(FacilityZoneCreate {
                id: HALL_ID.to_string(),
                site_id: SITE_ID.to_string(),
                kind: FacilityZoneKind::DataHall,
                state: FacilityZoneState::Planned,
                display_name: "hall a".to_string(),
                created_at_epoch_seconds: 4,
            })
            .expect("hall");
        catalog
            .transition_facility_zone(&hall.id.value, FacilityZoneState::Active, 5)
            .expect("hall active");
        let power = catalog
            .add_power_zone(PowerZoneCreate {
                id: POWER_ID.to_string(),
                site_id: SITE_ID.to_string(),
                redundancy: PowerRedundancy::TwoN,
                state: PowerZoneState::Planned,
                capacity_watts: 20_000,
                utility_feed_count: 2,
                created_at_epoch_seconds: 6,
            })
            .expect("power");
        catalog
            .transition_power_zone(&power.id.value, PowerZoneState::Energized, 7)
            .expect("power energized");
        let cooling = catalog
            .add_cooling_zone(CoolingZoneCreate {
                id: COOLING_ID.to_string(),
                site_id: SITE_ID.to_string(),
                technology: CoolingTechnology::ChilledWater,
                state: CoolingZoneState::Planned,
                heat_capacity_watts: 20_000,
                water_budget_liters_per_hour: 10_000,
                created_at_epoch_seconds: 8,
            })
            .expect("cooling");
        catalog
            .transition_cooling_zone(&cooling.id.value, CoolingZoneState::Active, 9)
            .expect("cooling active");
        let security = catalog
            .add_security_zone(SecurityZoneCreate {
                id: SECURITY_ID.to_string(),
                site_id: SITE_ID.to_string(),
                kind: SecurityZoneKind::Badge,
                state: SecurityZoneState::Planned,
                created_at_epoch_seconds: 10,
            })
            .expect("security");
        catalog
            .transition_security_zone(&security.id.value, SecurityZoneState::Armed, 11)
            .expect("security armed");
        let rack = catalog
            .add_rack(RackCreate {
                id: RACK_ID_VALUE.to_string(),
                site_id: SITE_ID.to_string(),
                facility_zone_id: HALL_ID.to_string(),
                security_zone_id: SECURITY_ID.to_string(),
                row_label: "row-a".to_string(),
                state: RackState::Planned,
                u_height: 42,
                rated_power_watts: 12_000,
                max_heat_watts: 12_000,
                max_weight_kg: 1_200,
                created_at_epoch_seconds: 12,
            })
            .expect("rack");
        catalog
            .transition_rack(&rack.id.value, RackState::Active, 13)
            .expect("rack active");
        catalog
    }

    fn equipment_create(id: &str) -> EquipmentCreate {
        EquipmentCreate {
            id: id.to_string(),
            site_id: SITE_ID.to_string(),
            kind: EquipmentKind::Server,
            lifecycle: EquipmentLifecycle::Ordered,
            procurement_ref: "proc/order-1/server".to_string(),
            vendor: "oya-approved-vendor".to_string(),
            model: "srv-1".to_string(),
            ordered_at_epoch_seconds: 20,
        }
    }

    fn install_plan(start_u: u16, power_watts: u64) -> EquipmentInstallPlan {
        EquipmentInstallPlan {
            rack_id: RACK_ID_VALUE.to_string(),
            power_zone_id: POWER_ID.to_string(),
            cooling_zone_id: COOLING_ID.to_string(),
            start_u,
            height_u: 2,
            power_watts,
            heat_watts: power_watts,
            weight_kg: 35,
            network_drop_refs: vec!["netdrop/rack-a/a1".to_string()],
            installed_at_epoch_seconds: 30 + u64::from(start_u),
        }
    }

    fn received_equipment(catalog: &mut CloudDcopsCatalog, id: &str) -> EquipmentId {
        let equipment = catalog
            .order_equipment(equipment_create(id))
            .expect("ordered");
        let asset = format!("asset/{SITE_ID}/{}", id.rsplit('/').next().expect("slug"));
        catalog
            .receive_equipment(&equipment.id.value, asset, "serial-1".to_string(), 25)
            .expect("received");
        equipment.id.value
    }

    #[test]
    fn builds_active_dcops_hierarchy_with_strict_parent_states() {
        let catalog = active_catalog();
        assert_eq!(catalog.sites().count(), 1);
        let inactive_site_error = {
            let mut catalog = CloudDcopsCatalog::default();
            catalog.add_site(site_create()).expect("site");
            catalog
                .add_facility_zone(FacilityZoneCreate {
                    id: HALL_ID.to_string(),
                    site_id: SITE_ID.to_string(),
                    kind: FacilityZoneKind::DataHall,
                    state: FacilityZoneState::Planned,
                    display_name: "hall a".to_string(),
                    created_at_epoch_seconds: 4,
                })
                .expect_err("inactive site must reject child")
        };
        assert_eq!(inactive_site_error, CloudDcopsError::InactiveParent);
    }

    #[test]
    fn rejects_forged_initial_states_and_bad_region_or_redundancy() {
        assert_eq!(
            DatacenterSite::new(DatacenterSiteCreate {
                state: DatacenterState::Active,
                ..site_create()
            })
            .expect_err("site active state is forged"),
            CloudDcopsError::InvalidInitialState
        );
        assert_eq!(
            DatacenterSite::new(DatacenterSiteCreate {
                id: "dc/region-beta1/site-a".to_string(),
                ..site_create()
            })
            .expect_err("id region must match payload"),
            CloudDcopsError::RegionMismatch
        );
        assert_eq!(
            PowerZone::new(PowerZoneCreate {
                id: POWER_ID.to_string(),
                site_id: SITE_ID.to_string(),
                redundancy: PowerRedundancy::TwoNPlusOne,
                state: PowerZoneState::Planned,
                capacity_watts: 1,
                utility_feed_count: 2,
                created_at_epoch_seconds: 1,
            })
            .expect_err("2N+1 requires three feeds"),
            CloudDcopsError::InvalidRedundancy
        );
    }

    #[test]
    fn enforces_equipment_lifecycle_and_capacity_without_overlap() {
        let mut catalog = active_catalog();
        let equipment_id = received_equipment(&mut catalog, EQUIP_ID);
        catalog
            .install_equipment(&equipment_id, install_plan(1, 4_000))
            .expect("install first server");
        catalog
            .transition_equipment(&equipment_id, EquipmentLifecycle::InService, 40)
            .expect("in service");
        let capacity = catalog
            .rack_capacity(&RackId::new(RACK_ID_VALUE).expect("rack id"))
            .expect("capacity");
        assert_eq!(capacity.used_u, 2);
        assert_eq!(capacity.remaining_power_watts, 8_000);

        let other_id = received_equipment(&mut catalog, EQUIP_ID_B);
        assert_eq!(
            catalog
                .install_equipment(&other_id, install_plan(2, 1_000))
                .expect_err("U ranges overlap"),
            CloudDcopsError::RackUnitOverlap
        );
        assert_eq!(
            catalog
                .install_equipment(&other_id, install_plan(3, 9_000))
                .expect_err("rack power budget exceeded"),
            CloudDcopsError::RackCapacityExceeded
        );
    }

    #[test]
    fn ewaste_transferred_equipment_releases_installation_capacity() {
        let mut catalog = active_catalog();
        let retired_id = received_equipment(&mut catalog, EQUIP_ID);
        catalog
            .install_equipment(&retired_id, install_plan(1, 10_000))
            .expect("install retired server");
        catalog
            .transition_equipment(&retired_id, EquipmentLifecycle::InService, 40)
            .expect("retired in service");
        catalog
            .transition_equipment(&retired_id, EquipmentLifecycle::Decommissioning, 41)
            .expect("retired decommissioning");
        catalog
            .transition_equipment(&retired_id, EquipmentLifecycle::Sanitized, 42)
            .expect("retired sanitized");
        catalog
            .transition_equipment(&retired_id, EquipmentLifecycle::EwasteTransferred, 43)
            .expect("retired transferred");

        let replacement_id = received_equipment(&mut catalog, EQUIP_ID_B);
        let mut replacement_plan = install_plan(1, 12_000);
        replacement_plan.installed_at_epoch_seconds = 44;
        catalog
            .install_equipment(&replacement_id, replacement_plan)
            .expect("ewaste-transferred equipment releases rack, power, and cooling budgets");

        let capacity = catalog
            .rack_capacity(&RackId::new(RACK_ID_VALUE).expect("rack id"))
            .expect("capacity");
        assert_eq!(capacity.used_u, 2);
        assert_eq!(capacity.remaining_power_watts, 0);
        assert_eq!(capacity.remaining_heat_watts, 0);
    }

    #[test]
    fn rejects_installing_equipment_before_receipt() {
        let mut catalog = active_catalog();
        let equipment = catalog
            .order_equipment(equipment_create(EQUIP_ID))
            .expect("ordered");
        assert_eq!(
            catalog
                .install_equipment(&equipment.id.value, install_plan(1, 1_000))
                .expect_err("ordered equipment is not installable"),
            CloudDcopsError::InvalidStateTransition
        );
    }

    #[test]
    fn maps_and_certifies_network_cables_with_loss_budget() {
        let mut catalog = active_catalog();
        let first = received_equipment(&mut catalog, EQUIP_ID);
        let second = received_equipment(&mut catalog, EQUIP_ID_B);
        catalog
            .install_equipment(&first, install_plan(1, 2_000))
            .expect("first install");
        catalog
            .install_equipment(&second, install_plan(3, 2_000))
            .expect("second install");
        let cable = catalog
            .add_cable_run(CableRunCreate {
                id: "cable/dc/region-alpha1/site-a/cable-a".to_string(),
                site_id: SITE_ID.to_string(),
                from: CableEndpoint {
                    equipment_id: EQUIP_ID.to_string(),
                    port_name: "eth0".to_string(),
                },
                to: CableEndpoint {
                    equipment_id: EQUIP_ID_B.to_string(),
                    port_name: "eth0".to_string(),
                },
                media: CableMedia::SingleModeFiber,
                state: CableState::Planned,
                measured_loss_milli_db: 1_000,
                loss_budget_milli_db: 3_000,
                created_at_epoch_seconds: 50,
            })
            .expect("cable");
        let cable = catalog
            .transition_cable_run(&cable.id.value, CableState::Installed, 51)
            .expect("installed");
        assert_eq!(cable.state.value, CableState::Installed);
        assert_eq!(
            CableRun::new(CableRunCreate {
                id: "cable/dc/region-alpha1/site-a/cable-b".to_string(),
                site_id: SITE_ID.to_string(),
                from: CableEndpoint {
                    equipment_id: EQUIP_ID.to_string(),
                    port_name: "eth1".to_string(),
                },
                to: CableEndpoint {
                    equipment_id: EQUIP_ID_B.to_string(),
                    port_name: "eth1".to_string(),
                },
                media: CableMedia::SingleModeFiber,
                state: CableState::Planned,
                measured_loss_milli_db: 4_000,
                loss_budget_milli_db: 3_000,
                created_at_epoch_seconds: 50,
            })
            .expect_err("measured loss must fit budget"),
            CloudDcopsError::InvalidCableLoss
        );
    }

    #[test]
    fn records_bms_readings_only_for_enabled_points_once() {
        let mut catalog = active_catalog();
        let point = catalog
            .add_bms_point(BmsPointCreate {
                id: "bms/dc/region-alpha1/site-a/temp-a".to_string(),
                site_id: SITE_ID.to_string(),
                equipment_id: None,
                kind: BmsPointKind::Temperature,
                state: BmsPointState::Commissioning,
                unit: "milli-celsius".to_string(),
                created_at_epoch_seconds: 60,
            })
            .expect("point");
        assert_eq!(
            catalog
                .record_bms_reading(BmsReadingCreate {
                    point_id: point.id.value.value.clone(),
                    site_id: SITE_ID.to_string(),
                    observed_at_epoch_seconds: 61,
                    milli_value: 22_000,
                })
                .expect_err("disabled point rejects reading"),
            CloudDcopsError::InactiveParent
        );
        catalog
            .transition_bms_point(&point.id.value, BmsPointState::Enabled, 61)
            .expect("enabled");
        catalog
            .record_bms_reading(BmsReadingCreate {
                point_id: point.id.value.value.clone(),
                site_id: SITE_ID.to_string(),
                observed_at_epoch_seconds: 62,
                milli_value: 22_000,
            })
            .expect("reading");
        assert_eq!(
            catalog
                .record_bms_reading(BmsReadingCreate {
                    point_id: point.id.value.value,
                    site_id: SITE_ID.to_string(),
                    observed_at_epoch_seconds: 62,
                    milli_value: 22_100,
                })
                .expect_err("duplicate point timestamp rejected"),
            CloudDcopsError::DuplicateBmsReading
        );
    }

    #[test]
    fn bms_reading_store_enforces_bounded_retention() {
        let mut catalog = active_catalog();
        catalog.bms_reading_retention_limit = 1;
        let point = catalog
            .add_bms_point(BmsPointCreate {
                id: "bms/dc/region-alpha1/site-a/temp-retention".to_string(),
                site_id: SITE_ID.to_string(),
                equipment_id: None,
                kind: BmsPointKind::Temperature,
                state: BmsPointState::Commissioning,
                unit: "milli-celsius".to_string(),
                created_at_epoch_seconds: 71,
            })
            .expect("point");
        catalog
            .transition_bms_point(&point.id.value, BmsPointState::Enabled, 72)
            .expect("point enabled");

        catalog
            .record_bms_reading(BmsReadingCreate {
                point_id: point.id.value.value.clone(),
                site_id: SITE_ID.to_string(),
                observed_at_epoch_seconds: 73,
                milli_value: 22_000,
            })
            .expect("first reading");
        catalog
            .record_bms_reading(BmsReadingCreate {
                point_id: point.id.value.value.clone(),
                site_id: SITE_ID.to_string(),
                observed_at_epoch_seconds: 74,
                milli_value: 22_100,
            })
            .expect("second reading");

        assert_eq!(catalog.bms_reading_count(), 1);
    }

    #[test]
    fn work_orders_require_safe_state_machine_and_privacy_class() {
        let mut catalog = active_catalog();
        let equipment_id = received_equipment(&mut catalog, EQUIP_ID);
        let work_order = catalog
            .open_work_order(WorkOrderCreate {
                id: "wo/dc/region-alpha1/site-a/wo-a".to_string(),
                site_id: SITE_ID.to_string(),
                equipment_id: Some(equipment_id.value.clone()),
                kind: WorkOrderKind::Install,
                priority: WorkOrderPriority::P1,
                state: WorkOrderState::Open,
                opened_by: "usr_operator".to_string(),
                assigned_to: None,
                safety_plan_ref: "safety/site-a/install".to_string(),
                data_class: DataClass::PiiQuasiIdentifier,
                opened_at_epoch_seconds: 70,
            })
            .expect("work order");
        assert_eq!(
            WorkOrder::new(WorkOrderCreate {
                id: "wo/dc/region-alpha1/site-a/wo-b".to_string(),
                site_id: SITE_ID.to_string(),
                equipment_id: None,
                kind: WorkOrderKind::Audit,
                priority: WorkOrderPriority::P3,
                state: WorkOrderState::Completed,
                opened_by: "usr_operator".to_string(),
                assigned_to: None,
                safety_plan_ref: "safety/site-a/audit".to_string(),
                data_class: DataClass::Audit,
                opened_at_epoch_seconds: 70,
            })
            .expect_err("state and data class are not accepted"),
            CloudDcopsError::InvalidInitialState
        );
        let assigned = catalog
            .assign_work_order(&work_order.id.value, "usr_tech".to_string(), 71)
            .expect("assigned");
        assert_eq!(assigned.state.value, WorkOrderState::Assigned);
        catalog
            .start_work_order(&work_order.id.value, 72)
            .expect("started");
        let completed = catalog
            .complete_work_order(
                &work_order.id.value,
                WorkOrderResolution {
                    completed_by: "usr_tech".to_string(),
                    resolution_ref: "resolution/site-a/wo-a".to_string(),
                    completed_at_epoch_seconds: 73,
                },
            )
            .expect("completed");
        assert_eq!(completed.state.value, WorkOrderState::Completed);
    }

    #[test]
    fn sustainability_snapshot_verifies_exact_ratios_and_targets() {
        let mut catalog = active_catalog();
        let snapshot = catalog
            .record_sustainability_snapshot(SustainabilitySnapshotCreate {
                id: "sustainability/dc/region-alpha1/site-a/day-1".to_string(),
                site_id: SITE_ID.to_string(),
                period_start_epoch_seconds: 100,
                period_end_epoch_seconds: 200,
                it_energy_kwh_milli: 1_000,
                facility_energy_kwh_milli: 1_500,
                water_liters_milli: 2_000,
                carbon_grams: 1_000,
                pue_milli: 1_500,
                wue_milli: 2_000,
                cue_milli: 1_000,
                data_class: DataClass::InternalOnly,
            })
            .expect("snapshot");
        assert_eq!(snapshot.pue_milli.value, 1_500);
        assert_eq!(
            SustainabilitySnapshot::new(SustainabilitySnapshotCreate {
                id: "sustainability/dc/region-alpha1/site-a/day-2".to_string(),
                site_id: SITE_ID.to_string(),
                period_start_epoch_seconds: 100,
                period_end_epoch_seconds: 200,
                it_energy_kwh_milli: 1_000,
                facility_energy_kwh_milli: 1_400,
                water_liters_milli: 2_000,
                carbon_grams: 1_000,
                pue_milli: 1_500,
                wue_milli: 2_000,
                cue_milli: 1_000,
                data_class: DataClass::InternalOnly,
            })
            .expect_err("provided ratios must equal source measurements"),
            CloudDcopsError::InvalidTargetRatio
        );
    }
}
