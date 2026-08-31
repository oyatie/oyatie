use data_boundary_kernel::Classified;

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{
    DatacenterSiteId, FacilityZoneId, RACK_ID_PREFIX, RackId, SecurityZoneId,
};
use crate::lifecycle::rack_transition_allowed;
use crate::validation::{
    validate_child_id, validate_path_segment, validate_positive_time, validate_rack_shape,
    validate_time_order,
};

const DCOPS_RACK_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RackState {
    Planned,
    Active,
    Quarantined,
    Retired,
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
