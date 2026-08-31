use super::*;

impl CloudDcopsCatalog {
    pub(super) fn validate_installation(
        &self,
        equipment_id: &EquipmentId,
        site_id: &DatacenterSiteId,
        installation: &EquipmentInstallation,
    ) -> Result<(), CloudDcopsError> {
        let rack = self
            .racks
            .get(&installation.rack_id)
            .ok_or(CloudDcopsError::UnknownRack)?;
        let power_zone = self
            .power_zones
            .get(&installation.power_zone_id)
            .ok_or(CloudDcopsError::UnknownPowerZone)?;
        let cooling_zone = self
            .cooling_zones
            .get(&installation.cooling_zone_id)
            .ok_or(CloudDcopsError::UnknownCoolingZone)?;
        validate_same_site(site_id, &rack.site_id.value)?;
        validate_same_site(site_id, &power_zone.site_id.value)?;
        validate_same_site(site_id, &cooling_zone.site_id.value)?;
        if rack.state.value != RackState::Active
            || power_zone.state.value != PowerZoneState::Energized
            || cooling_zone.state.value != CoolingZoneState::Active
        {
            return Err(CloudDcopsError::InactiveParent);
        }
        let end_u = installation_end_u(installation)?;
        if end_u > rack.u_height.value {
            return Err(CloudDcopsError::InvalidRackUnits);
        }
        let accounting = self.installation_accounting(
            rack,
            &installation.power_zone_id,
            &installation.cooling_zone_id,
            equipment_id,
            Some(installation),
        )?;
        if accounting.rack_unit_overlap {
            return Err(CloudDcopsError::RackUnitOverlap);
        }
        if accounting.rack_capacity.used_u > rack.u_height.value
            || accounting.rack_capacity.used_power_watts > rack.rated_power_watts.value
            || accounting.rack_capacity.used_heat_watts > rack.max_heat_watts.value
            || accounting.rack_capacity.used_weight_kg > rack.max_weight_kg.value
        {
            return Err(CloudDcopsError::RackCapacityExceeded);
        }
        if accounting.power_zone_used_watts > power_zone.capacity_watts.value {
            return Err(CloudDcopsError::PowerZoneCapacityExceeded);
        }
        if accounting.cooling_zone_used_watts > cooling_zone.heat_capacity_watts.value {
            return Err(CloudDcopsError::CoolingZoneCapacityExceeded);
        }
        Ok(())
    }

    pub(super) fn installation_accounting(
        &self,
        rack: &Rack,
        power_zone_id: &PowerZoneId,
        cooling_zone_id: &CoolingZoneId,
        equipment_id: &EquipmentId,
        proposed: Option<&EquipmentInstallation>,
    ) -> Result<InstallationAccounting, CloudDcopsError> {
        let mut accounting = InstallationAccounting {
            rack_capacity: self
                .rack_capacity_by_id
                .get(&rack.id.value)
                .copied()
                .unwrap_or_default(),
            power_zone_used_watts: self
                .power_zone_used_watts_by_id
                .get(power_zone_id)
                .copied()
                .unwrap_or_default(),
            cooling_zone_used_watts: self
                .cooling_zone_used_watts_by_id
                .get(cooling_zone_id)
                .copied()
                .unwrap_or_default(),
            rack_unit_overlap: false,
        };
        let proposed_end_u = proposed.map(installation_end_u).transpose()?;
        if let (Some(proposed), Some(proposed_end_u), Some(allocations)) = (
            proposed,
            proposed_end_u,
            self.rack_unit_allocations_by_id.get(&rack.id.value),
        ) {
            for (allocated_id, (allocated_start_u, allocated_end_u)) in allocations {
                if allocated_id != equipment_id
                    && u_ranges_overlap(
                        proposed.start_u,
                        proposed_end_u,
                        *allocated_start_u,
                        *allocated_end_u,
                    )
                {
                    accounting.rack_unit_overlap = true;
                    break;
                }
            }
        }
        if let Some(installation) = proposed {
            if installation.rack_id == rack.id.value {
                add_installation_capacity(&mut accounting.rack_capacity, installation);
            }
            if installation.power_zone_id == *power_zone_id {
                accounting.power_zone_used_watts = accounting
                    .power_zone_used_watts
                    .saturating_add(installation.power_watts);
            }
            if installation.cooling_zone_id == *cooling_zone_id {
                accounting.cooling_zone_used_watts = accounting
                    .cooling_zone_used_watts
                    .saturating_add(installation.heat_watts);
            }
        }
        finalize_rack_capacity(rack, &mut accounting.rack_capacity);
        Ok(accounting)
    }

    pub(super) fn apply_capacity_accounting(
        &mut self,
        equipment_id: &EquipmentId,
        equipment: &Equipment,
    ) -> Result<(), CloudDcopsError> {
        if !equipment_counts_against_capacity(equipment) {
            return Ok(());
        }
        let Some(installation) = equipment.installation.value.as_ref() else {
            return Ok(());
        };
        let end_u = installation_end_u(installation)?;
        add_installation_capacity(
            self.rack_capacity_by_id
                .entry(installation.rack_id.clone())
                .or_default(),
            installation,
        );
        let power_used = self
            .power_zone_used_watts_by_id
            .entry(installation.power_zone_id.clone())
            .or_default();
        *power_used = (*power_used).saturating_add(installation.power_watts);
        let cooling_used = self
            .cooling_zone_used_watts_by_id
            .entry(installation.cooling_zone_id.clone())
            .or_default();
        *cooling_used = (*cooling_used).saturating_add(installation.heat_watts);
        self.rack_unit_allocations_by_id
            .entry(installation.rack_id.clone())
            .or_default()
            .insert(equipment_id.clone(), (installation.start_u, end_u));
        Ok(())
    }

    pub(super) fn release_capacity_accounting(
        &mut self,
        equipment_id: &EquipmentId,
        equipment: &Equipment,
    ) -> Result<(), CloudDcopsError> {
        if !equipment_counts_against_capacity(equipment) {
            return Ok(());
        }
        let Some(installation) = equipment.installation.value.as_ref() else {
            return Ok(());
        };
        let remove_rack =
            if let Some(capacity) = self.rack_capacity_by_id.get_mut(&installation.rack_id) {
                subtract_installation_capacity(capacity, installation);
                capacity.used_u == 0
                    && capacity.used_power_watts == 0
                    && capacity.used_heat_watts == 0
                    && capacity.used_weight_kg == 0
            } else {
                false
            };
        if remove_rack {
            self.rack_capacity_by_id.remove(&installation.rack_id);
        }
        let remove_power = if let Some(used) = self
            .power_zone_used_watts_by_id
            .get_mut(&installation.power_zone_id)
        {
            *used = used.saturating_sub(installation.power_watts);
            *used == 0
        } else {
            false
        };
        if remove_power {
            self.power_zone_used_watts_by_id
                .remove(&installation.power_zone_id);
        }
        let remove_cooling = if let Some(used) = self
            .cooling_zone_used_watts_by_id
            .get_mut(&installation.cooling_zone_id)
        {
            *used = used.saturating_sub(installation.heat_watts);
            *used == 0
        } else {
            false
        };
        if remove_cooling {
            self.cooling_zone_used_watts_by_id
                .remove(&installation.cooling_zone_id);
        }
        let remove_allocations = if let Some(allocations) = self
            .rack_unit_allocations_by_id
            .get_mut(&installation.rack_id)
        {
            allocations.remove(equipment_id);
            allocations.is_empty()
        } else {
            false
        };
        if remove_allocations {
            self.rack_unit_allocations_by_id
                .remove(&installation.rack_id);
        }
        Ok(())
    }
}
