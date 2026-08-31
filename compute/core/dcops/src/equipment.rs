use data_boundary_kernel::Classified;

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{
    ASSET_TAG_PREFIX, AssetTag, CoolingZoneId, DatacenterSiteId, EQUIPMENT_ID_PREFIX, EquipmentId,
    PROCUREMENT_REF_PREFIX, PowerZoneId, RackId,
};
use crate::lifecycle::equipment_lifecycle_transition_allowed;
use crate::validation::{
    typed_network_drop_refs, validate_child_id, validate_install_shape, validate_non_empty,
    validate_positive_time, validate_ref_path, validate_time_order,
};

const DCOPS_EQUIPMENT_SCHEMA_VERSION: u32 = 1;

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
