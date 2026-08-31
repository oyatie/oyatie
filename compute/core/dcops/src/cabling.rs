use data_boundary_kernel::Classified;

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{CABLE_ID_PREFIX, CableRunId, DatacenterSiteId, EquipmentId};
use crate::lifecycle::cable_transition_allowed;
use crate::validation::{
    validate_cable_loss, validate_child_id, validate_port, validate_positive_time,
    validate_time_order,
};

const DCOPS_CABLE_SCHEMA_VERSION: u32 = 1;

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
