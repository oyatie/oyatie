use super::*;

impl CloudDcopsCatalog {
    pub fn add_site(
        &mut self,
        input: DatacenterSiteCreate,
    ) -> Result<DatacenterSite, CloudDcopsError> {
        let site = DatacenterSite::new(input)?;
        if self.sites.contains_key(&site.id.value) {
            return Err(CloudDcopsError::DuplicateSite);
        }
        self.sites.insert(site.id.value.clone(), site.clone());
        Ok(site)
    }

    pub fn transition_site(
        &mut self,
        site_id: &DatacenterSiteId,
        next_state: DatacenterState,
        updated_at_epoch_seconds: u64,
    ) -> Result<DatacenterSite, CloudDcopsError> {
        let site = self
            .sites
            .get(site_id)
            .ok_or(CloudDcopsError::UnknownSite)?;
        let next = site.transition(next_state, updated_at_epoch_seconds)?;
        self.sites.insert(site_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_facility_zone(
        &mut self,
        input: FacilityZoneCreate,
    ) -> Result<FacilityZone, CloudDcopsError> {
        let zone = FacilityZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.facility_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicateFacilityZone);
        }
        self.facility_zones
            .insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_facility_zone(
        &mut self,
        zone_id: &FacilityZoneId,
        next_state: FacilityZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<FacilityZone, CloudDcopsError> {
        let zone = self
            .facility_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownFacilityZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.facility_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_power_zone(&mut self, input: PowerZoneCreate) -> Result<PowerZone, CloudDcopsError> {
        let zone = PowerZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.power_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicatePowerZone);
        }
        self.power_zones.insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_power_zone(
        &mut self,
        zone_id: &PowerZoneId,
        next_state: PowerZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<PowerZone, CloudDcopsError> {
        let zone = self
            .power_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownPowerZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.power_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_cooling_zone(
        &mut self,
        input: CoolingZoneCreate,
    ) -> Result<CoolingZone, CloudDcopsError> {
        let zone = CoolingZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.cooling_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicateCoolingZone);
        }
        self.cooling_zones
            .insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_cooling_zone(
        &mut self,
        zone_id: &CoolingZoneId,
        next_state: CoolingZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<CoolingZone, CloudDcopsError> {
        let zone = self
            .cooling_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownCoolingZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.cooling_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_security_zone(
        &mut self,
        input: SecurityZoneCreate,
    ) -> Result<SecurityZone, CloudDcopsError> {
        let zone = SecurityZone::new(input)?;
        self.require_active_site(&zone.site_id.value)?;
        if self.security_zones.contains_key(&zone.id.value) {
            return Err(CloudDcopsError::DuplicateSecurityZone);
        }
        self.security_zones
            .insert(zone.id.value.clone(), zone.clone());
        Ok(zone)
    }

    pub fn transition_security_zone(
        &mut self,
        zone_id: &SecurityZoneId,
        next_state: SecurityZoneState,
        updated_at_epoch_seconds: u64,
    ) -> Result<SecurityZone, CloudDcopsError> {
        let zone = self
            .security_zones
            .get(zone_id)
            .ok_or(CloudDcopsError::UnknownSecurityZone)?;
        let next = zone.transition(next_state, updated_at_epoch_seconds)?;
        self.security_zones.insert(zone_id.clone(), next.clone());
        Ok(next)
    }

    pub fn add_rack(&mut self, input: RackCreate) -> Result<Rack, CloudDcopsError> {
        let rack = Rack::new(input)?;
        self.require_active_site(&rack.site_id.value)?;
        let facility_zone = self
            .facility_zones
            .get(&rack.facility_zone_id.value)
            .ok_or(CloudDcopsError::UnknownFacilityZone)?;
        let security_zone = self
            .security_zones
            .get(&rack.security_zone_id.value)
            .ok_or(CloudDcopsError::UnknownSecurityZone)?;
        validate_same_site(&rack.site_id.value, &facility_zone.site_id.value)?;
        validate_same_site(&rack.site_id.value, &security_zone.site_id.value)?;
        if facility_zone.state.value != FacilityZoneState::Active
            || security_zone.state.value != SecurityZoneState::Armed
        {
            return Err(CloudDcopsError::InactiveParent);
        }
        if self.racks.contains_key(&rack.id.value) {
            return Err(CloudDcopsError::DuplicateRack);
        }
        self.racks.insert(rack.id.value.clone(), rack.clone());
        Ok(rack)
    }

    pub fn transition_rack(
        &mut self,
        rack_id: &RackId,
        next_state: RackState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Rack, CloudDcopsError> {
        let rack = self
            .racks
            .get(rack_id)
            .ok_or(CloudDcopsError::UnknownRack)?;
        let next = rack.transition(next_state, updated_at_epoch_seconds)?;
        self.racks.insert(rack_id.clone(), next.clone());
        Ok(next)
    }
}
