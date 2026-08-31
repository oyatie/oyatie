use data_boundary_kernel::Classified;

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{BMS_POINT_ID_PREFIX, BmsPointId, DatacenterSiteId, EquipmentId};
use crate::lifecycle::bms_point_transition_allowed;
use crate::validation::{
    validate_child_id, validate_positive_time, validate_time_order, validate_unit,
};

const DCOPS_BMS_SCHEMA_VERSION: u32 = 1;

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
