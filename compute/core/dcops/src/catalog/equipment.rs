use super::*;

impl CloudDcopsCatalog {
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
}
