use super::{
    ASSET_TAG_PREFIX, AssetTag, BMS_POINT_ID_PREFIX, BmsPointId, CABLE_ID_PREFIX, CableRunId,
    EQUIPMENT_ID_PREFIX, EquipmentId, PrincipalId, SERVICE_PRINCIPAL_PREFIX,
    SUSTAINABILITY_ID_PREFIX, SustainabilitySnapshotId, USER_PRINCIPAL_PREFIX,
    WORK_ORDER_ID_PREFIX, WorkOrderId,
};
use crate::CloudDcopsError;
use crate::validation::validate_path;

impl EquipmentId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            EQUIPMENT_ID_PREFIX,
            5,
            CloudDcopsError::InvalidEquipmentId,
        )?;
        Ok(Self { value })
    }
}

impl CableRunId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            CABLE_ID_PREFIX,
            5,
            CloudDcopsError::InvalidCableRunId,
        )?;
        Ok(Self { value })
    }
}

impl BmsPointId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            BMS_POINT_ID_PREFIX,
            5,
            CloudDcopsError::InvalidBmsPointId,
        )?;
        Ok(Self { value })
    }
}

impl WorkOrderId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            WORK_ORDER_ID_PREFIX,
            5,
            CloudDcopsError::InvalidWorkOrderId,
        )?;
        Ok(Self { value })
    }
}

impl SustainabilitySnapshotId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            SUSTAINABILITY_ID_PREFIX,
            5,
            CloudDcopsError::InvalidSustainabilitySnapshotId,
        )?;
        Ok(Self { value })
    }
}

impl AssetTag {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        validate_path(
            &value,
            ASSET_TAG_PREFIX,
            5,
            CloudDcopsError::InvalidAssetTag,
        )?;
        Ok(Self { value })
    }
}

impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDcopsError> {
        let value = value.into();
        if (value.starts_with(USER_PRINCIPAL_PREFIX) && value.len() > USER_PRINCIPAL_PREFIX.len())
            || (value.starts_with(SERVICE_PRINCIPAL_PREFIX)
                && value.len() > SERVICE_PRINCIPAL_PREFIX.len())
        {
            Ok(Self { value })
        } else {
            Err(CloudDcopsError::InvalidPrincipalId)
        }
    }
}
