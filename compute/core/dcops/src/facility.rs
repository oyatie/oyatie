use data_boundary_kernel::Classified;

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{
    DatacenterSiteId, FACILITY_ZONE_ID_PREFIX, FacilityZoneId, SECURITY_ZONE_ID_PREFIX,
    SecurityZoneId,
};
use crate::lifecycle::{facility_zone_transition_allowed, security_zone_transition_allowed};
use crate::validation::{
    validate_child_id, validate_non_empty, validate_positive_time, validate_time_order,
};

const DCOPS_FACILITY_ZONE_SCHEMA_VERSION: u32 = 1;
const DCOPS_SECURITY_ZONE_SCHEMA_VERSION: u32 = 1;

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
