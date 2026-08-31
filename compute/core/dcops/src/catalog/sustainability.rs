use super::*;

impl CloudDcopsCatalog {
    pub fn record_sustainability_snapshot(
        &mut self,
        input: SustainabilitySnapshotCreate,
    ) -> Result<SustainabilitySnapshot, CloudDcopsError> {
        let snapshot = SustainabilitySnapshot::new(input)?;
        let site = self
            .sites
            .get(&snapshot.site_id.value)
            .ok_or(CloudDcopsError::UnknownSite)?;
        if site.state.value != DatacenterState::Active {
            return Err(CloudDcopsError::InactiveParent);
        }
        if snapshot.pue_milli.value > u64::from(site.pue_target_milli.value)
            || snapshot.wue_milli.value > u64::from(site.wue_target_milli.value)
            || snapshot.cue_milli.value > u64::from(site.cue_target_milli.value)
        {
            return Err(CloudDcopsError::InvalidTargetRatio);
        }
        if self
            .sustainability_snapshots
            .contains_key(&snapshot.id.value)
        {
            return Err(CloudDcopsError::DuplicateSustainabilitySnapshot);
        }
        self.sustainability_snapshots
            .insert(snapshot.id.value.clone(), snapshot.clone());
        Ok(snapshot)
    }
}
