use cell_region::{AzCode, RegionCode};
use data_boundary_kernel::Classified;

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{DatacenterSiteId, PHYSICAL_REF_PREFIX};
use crate::lifecycle::datacenter_transition_allowed;
use crate::validation::{
    validate_az_region, validate_physical_ref, validate_positive_time,
    validate_sustainability_targets, validate_time_order,
};

const DCOPS_SITE_SCHEMA_VERSION: u32 = 1;

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
