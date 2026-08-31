use data_boundary_kernel::{Classified, DataClass};

use crate::CloudDcopsError;
use crate::classification::{audit, internal, public};
use crate::identifiers::{
    DatacenterSiteId, EquipmentId, PrincipalId, RESOLUTION_REF_PREFIX, SAFETY_PLAN_REF_PREFIX,
    WORK_ORDER_ID_PREFIX, WorkOrderId,
};
use crate::validation::{
    validate_child_id, validate_positive_time, validate_ref_path, validate_time_order,
    validate_work_order_data_class,
};

const DCOPS_WORK_ORDER_SCHEMA_VERSION: u32 = 1;

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
