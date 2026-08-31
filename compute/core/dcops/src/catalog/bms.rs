use super::*;

impl CloudDcopsCatalog {
    pub fn add_bms_point(&mut self, input: BmsPointCreate) -> Result<BmsPoint, CloudDcopsError> {
        let point = BmsPoint::new(input)?;
        self.require_active_site(&point.site_id.value)?;
        if let Some(equipment_id) = point.equipment_id.value.as_ref() {
            let equipment = self
                .equipment
                .get(equipment_id)
                .ok_or(CloudDcopsError::UnknownEquipment)?;
            validate_same_site(&point.site_id.value, &equipment.site_id.value)?;
        }
        if self.bms_points.contains_key(&point.id.value) {
            return Err(CloudDcopsError::DuplicateBmsPoint);
        }
        self.bms_points
            .insert(point.id.value.clone(), point.clone());
        Ok(point)
    }

    pub fn transition_bms_point(
        &mut self,
        point_id: &BmsPointId,
        next_state: BmsPointState,
        updated_at_epoch_seconds: u64,
    ) -> Result<BmsPoint, CloudDcopsError> {
        let point = self
            .bms_points
            .get(point_id)
            .ok_or(CloudDcopsError::UnknownBmsPoint)?;
        let next = point.transition(next_state, updated_at_epoch_seconds)?;
        self.bms_points.insert(point_id.clone(), next.clone());
        Ok(next)
    }

    pub fn record_bms_reading(
        &mut self,
        input: BmsReadingCreate,
    ) -> Result<BmsReading, CloudDcopsError> {
        let reading = BmsReading::new(input)?;
        let point = self
            .bms_points
            .get(&reading.point_id.value)
            .ok_or(CloudDcopsError::UnknownBmsPoint)?;
        validate_same_site(&reading.site_id.value, &point.site_id.value)?;
        if point.state.value != BmsPointState::Enabled {
            return Err(CloudDcopsError::InactiveParent);
        }
        let key = (
            reading.point_id.value.clone(),
            reading.observed_at_epoch_seconds.value,
        );
        if !self.remember_bms_reading(key) {
            return Err(CloudDcopsError::DuplicateBmsReading);
        }
        Ok(reading)
    }
}
