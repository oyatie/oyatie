use data_boundary_kernel::{Classified, DataClass};

use crate::CloudDcopsError;
use crate::classification::{internal, public};
use crate::identifiers::{DatacenterSiteId, SUSTAINABILITY_ID_PREFIX, SustainabilitySnapshotId};
use crate::validation::{
    exact_ratio_milli, validate_child_id, validate_sustainability_data_class, validate_time_order,
};

const DCOPS_SUSTAINABILITY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SustainabilitySnapshotCreate {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub site_id: String,                 // data_class: INTERNAL_ONLY
    pub period_start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub period_end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub it_energy_kwh_milli: u64,        // data_class: INTERNAL_ONLY
    pub facility_energy_kwh_milli: u64,  // data_class: INTERNAL_ONLY
    pub water_liters_milli: u64,         // data_class: INTERNAL_ONLY
    pub carbon_grams: u64,               // data_class: INTERNAL_ONLY
    pub pue_milli: u64,                  // data_class: INTERNAL_ONLY
    pub wue_milli: u64,                  // data_class: INTERNAL_ONLY
    pub cue_milli: u64,                  // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SustainabilitySnapshot {
    pub id: Classified<SustainabilitySnapshotId>,
    pub site_id: Classified<DatacenterSiteId>,
    pub period_start_epoch_seconds: Classified<u64>,
    pub period_end_epoch_seconds: Classified<u64>,
    pub it_energy_kwh_milli: Classified<u64>,
    pub facility_energy_kwh_milli: Classified<u64>,
    pub water_liters_milli: Classified<u64>,
    pub carbon_grams: Classified<u64>,
    pub pue_milli: Classified<u64>,
    pub wue_milli: Classified<u64>,
    pub cue_milli: Classified<u64>,
    pub data_class: Classified<DataClass>,
    pub schema_version: Classified<u32>,
}

impl SustainabilitySnapshot {
    pub fn new(input: SustainabilitySnapshotCreate) -> Result<Self, CloudDcopsError> {
        validate_time_order(
            input.period_start_epoch_seconds,
            input.period_end_epoch_seconds,
        )?;
        validate_sustainability_data_class(input.data_class)?;
        if input.it_energy_kwh_milli == 0
            || input.facility_energy_kwh_milli < input.it_energy_kwh_milli
            || input.water_liters_milli == 0
            || input.carbon_grams == 0
        {
            return Err(CloudDcopsError::InvalidCapacity);
        }
        let expected_pue =
            exact_ratio_milli(input.facility_energy_kwh_milli, input.it_energy_kwh_milli)?;
        let expected_wue = exact_ratio_milli(input.water_liters_milli, input.it_energy_kwh_milli)?;
        let expected_cue = exact_ratio_milli(input.carbon_grams, input.it_energy_kwh_milli)?;
        if input.pue_milli != expected_pue
            || input.wue_milli != expected_wue
            || input.cue_milli != expected_cue
        {
            return Err(CloudDcopsError::InvalidTargetRatio);
        }
        let id = SustainabilitySnapshotId::new(input.id)?;
        let site_id = DatacenterSiteId::new(input.site_id)?;
        validate_child_id(
            &id.value,
            SUSTAINABILITY_ID_PREFIX,
            &site_id.value,
            CloudDcopsError::ParentMismatch,
        )?;
        Ok(Self {
            id: internal(id),
            site_id: internal(site_id),
            period_start_epoch_seconds: internal(input.period_start_epoch_seconds),
            period_end_epoch_seconds: internal(input.period_end_epoch_seconds),
            it_energy_kwh_milli: internal(input.it_energy_kwh_milli),
            facility_energy_kwh_milli: internal(input.facility_energy_kwh_milli),
            water_liters_milli: internal(input.water_liters_milli),
            carbon_grams: internal(input.carbon_grams),
            pue_milli: internal(input.pue_milli),
            wue_milli: internal(input.wue_milli),
            cue_milli: internal(input.cue_milli),
            data_class: public(input.data_class),
            schema_version: public(DCOPS_SUSTAINABILITY_SCHEMA_VERSION),
        })
    }
}
