use std::collections::{BTreeMap, BTreeSet};

use crate::accounting::{
    InstallationAccounting, add_installation_capacity, equipment_counts_against_capacity,
    finalize_rack_capacity, subtract_installation_capacity,
};
use crate::validation::{installation_end_u, u_ranges_overlap, validate_same_site};
use crate::{
    BmsPoint, BmsPointCreate, BmsPointId, BmsPointState, BmsReading, BmsReadingCreate, CableRun,
    CableRunCreate, CableRunId, CableState, CloudDcopsError, CoolingZone, CoolingZoneCreate,
    CoolingZoneId, CoolingZoneState, DatacenterSite, DatacenterSiteCreate, DatacenterSiteId,
    DatacenterState, Equipment, EquipmentCreate, EquipmentId, EquipmentInstallPlan,
    EquipmentInstallation, EquipmentLifecycle, FacilityZone, FacilityZoneCreate, FacilityZoneId,
    FacilityZoneState, PowerZone, PowerZoneCreate, PowerZoneId, PowerZoneState, Rack,
    RackCapacitySnapshot, RackCreate, RackId, RackState, SecurityZone, SecurityZoneCreate,
    SecurityZoneId, SecurityZoneState, SustainabilitySnapshot, SustainabilitySnapshotCreate,
    SustainabilitySnapshotId, WorkOrder, WorkOrderCreate, WorkOrderId, WorkOrderResolution,
};

mod bms;
mod cabling;
mod capacity;
mod equipment;
mod facilities;
mod sustainability;
mod work_orders;

const DEFAULT_BMS_READING_RETENTION_LIMIT: usize = 1024;

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
    pub(crate) bms_reading_retention_limit: usize,
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
}
