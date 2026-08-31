use super::*;

impl CloudDcopsCatalog {
    pub fn add_cable_run(&mut self, input: CableRunCreate) -> Result<CableRun, CloudDcopsError> {
        let cable = CableRun::new(input)?;
        self.require_active_site(&cable.site_id.value)?;
        let from = self
            .equipment
            .get(&cable.from.value.equipment_id)
            .ok_or(CloudDcopsError::UnknownEquipment)?;
        let to = self
            .equipment
            .get(&cable.to.value.equipment_id)
            .ok_or(CloudDcopsError::UnknownEquipment)?;
        validate_same_site(&cable.site_id.value, &from.site_id.value)?;
        validate_same_site(&cable.site_id.value, &to.site_id.value)?;
        if self.cable_runs.contains_key(&cable.id.value) {
            return Err(CloudDcopsError::DuplicateCableRun);
        }
        self.cable_runs
            .insert(cable.id.value.clone(), cable.clone());
        Ok(cable)
    }

    pub fn transition_cable_run(
        &mut self,
        cable_id: &CableRunId,
        next_state: CableState,
        updated_at_epoch_seconds: u64,
    ) -> Result<CableRun, CloudDcopsError> {
        let cable = self
            .cable_runs
            .get(cable_id)
            .ok_or(CloudDcopsError::UnknownCableRun)?;
        let next = cable.transition(next_state, updated_at_epoch_seconds)?;
        self.cable_runs.insert(cable_id.clone(), next.clone());
        Ok(next)
    }
}
