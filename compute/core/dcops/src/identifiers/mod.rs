mod operations;
mod physical;

pub(crate) const SITE_ID_PREFIX: &str = "dc";
pub(crate) const FACILITY_ZONE_ID_PREFIX: &str = "zone";
pub(crate) const POWER_ZONE_ID_PREFIX: &str = "power";
pub(crate) const COOLING_ZONE_ID_PREFIX: &str = "cooling";
pub(crate) const SECURITY_ZONE_ID_PREFIX: &str = "security";
pub(crate) const RACK_ID_PREFIX: &str = "rack";
pub(crate) const EQUIPMENT_ID_PREFIX: &str = "equip";
pub(crate) const CABLE_ID_PREFIX: &str = "cable";
pub(crate) const BMS_POINT_ID_PREFIX: &str = "bms";
pub(crate) const WORK_ORDER_ID_PREFIX: &str = "wo";
pub(crate) const SUSTAINABILITY_ID_PREFIX: &str = "sustainability";
pub(crate) const ASSET_TAG_PREFIX: &str = "asset";
pub(crate) const PROCUREMENT_REF_PREFIX: &str = "proc";
pub(crate) const SAFETY_PLAN_REF_PREFIX: &str = "safety";
pub(crate) const RESOLUTION_REF_PREFIX: &str = "resolution";
pub(crate) const PHYSICAL_REF_PREFIX: &str = "physical";
pub(crate) const USER_PRINCIPAL_PREFIX: &str = "usr_";
pub(crate) const SERVICE_PRINCIPAL_PREFIX: &str = "sp_";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DatacenterSiteId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FacilityZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PowerZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CoolingZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SecurityZoneId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RackId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EquipmentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CableRunId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BmsPointId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkOrderId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SustainabilitySnapshotId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AssetTag {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PrincipalId {
    pub value: String, // data_class: INTERNAL_ONLY
}
