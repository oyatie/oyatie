use cell_region::RegionCode;

use super::{
    COOLING_ZONE_ID_PREFIX, CoolingZoneId, DatacenterSiteId, FACILITY_ZONE_ID_PREFIX,
    FacilityZoneId, POWER_ZONE_ID_PREFIX, PowerZoneId, RACK_ID_PREFIX, RackId,
    SECURITY_ZONE_ID_PREFIX, SITE_ID_PREFIX, SecurityZoneId,
};
use crate::CloudDcopsError;
use crate::validation::{site_region_from_path, validate_path};

impl DatacenterSiteId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            SITE_ID_PREFIX,
            3,
            CloudDcopsError::InvalidDatacenterSiteId,
        )?;
        let region = site_region_from_path(&value)?;
        RegionCode::new(region).map_err(|_| CloudDcopsError::InvalidDatacenterSiteId)?;
        Ok(Self { value })
    }

    pub fn region(&self) -> Result<RegionCode, CloudDcopsError> {
        RegionCode::new(site_region_from_path(&self.value)?)
            .map_err(|_| CloudDcopsError::InvalidDatacenterSiteId)
    }
}

impl FacilityZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            FACILITY_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidFacilityZoneId,
        )?;
        Ok(Self { value })
    }
}

impl PowerZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            POWER_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidPowerZoneId,
        )?;
        Ok(Self { value })
    }
}

impl CoolingZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            COOLING_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidCoolingZoneId,
        )?;
        Ok(Self { value })
    }
}

impl SecurityZoneId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            SECURITY_ZONE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidSecurityZoneId,
        )?;
        Ok(Self { value })
    }
}

impl RackId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(&value, RACK_ID_PREFIX, 5, CloudDcopsError::InvalidRackId)?;
        Ok(Self { value })
    }
}
