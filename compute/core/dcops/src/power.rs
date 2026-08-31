use data_boundary_kernel::Classified;

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{
    COOLING_ZONE_ID_PREFIX, CoolingZoneId, DatacenterSiteId, POWER_ZONE_ID_PREFIX, PowerZoneId,
};
use crate::lifecycle::{cooling_zone_transition_allowed, power_zone_transition_allowed};
use crate::validation::{
    validate_child_id, validate_positive_time, validate_power_redundancy, validate_time_order,
};

const DCOPS_POWER_ZONE_SCHEMA_VERSION: u32 = 1;
const DCOPS_COOLING_ZONE_SCHEMA_VERSION: u32 = 1;

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
