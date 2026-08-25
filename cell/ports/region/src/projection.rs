use cell_region::{
    AzState, CloudAz, CloudCell, CloudCellState, CloudRegion, CloudRegionCatalog, RegionState,
    TenantDensityClass,
};
use network_residency::ResidencyClass;

use crate::model::{
    CloudAzRecord, CloudCellIsolationEvidenceRecord, CloudRegionAzRef, CloudRegionCellRef,
    CloudRegionPowerZoneRef, CloudRegionRecord, CloudRegionRegulatoryPackRef,
};

pub(crate) fn region_record(region: &CloudRegion) -> CloudRegionRecord {
    CloudRegionRecord {
        code: region.code.value.value.clone(),
        display_name: region.display_name.value.clone(),
        regulatory_packs: region
            .regulatory_packs
            .value
            .iter()
            .map(|value| CloudRegionRegulatoryPackRef {
                value: value.clone(),
            })
            .collect(),
        azs: region
            .azs
            .value
            .iter()
            .map(|az| CloudRegionAzRef {
                value: az.value.clone(),
            })
            .collect(),
        state: region_state_label(region.state.value).to_string(),
        provider_facing: region.provider_facing.value,
        residency_strictness: residency_class_label(&region.residency_strictness.value).to_string(),
        created_at_epoch_seconds: region.created_at_epoch_seconds.value,
        schema_version: region.schema_version.value,
    }
}

pub(crate) fn az_record(az: &CloudAz, catalog: &CloudRegionCatalog) -> CloudAzRecord {
    let cell_isolation_evidence: Vec<_> = catalog
        .cells_for_region(&az.region_code.value)
        .filter(|cell| cell.az_code.value == az.code.value)
        .map(cell_isolation_evidence_record)
        .collect();
    CloudAzRecord {
        code: az.code.value.value.clone(),
        region_code: az.region_code.value.value.clone(),
        power_zones: az
            .power_zones
            .value
            .iter()
            .map(|value| CloudRegionPowerZoneRef {
                value: value.clone(),
            })
            .collect(),
        cells: az
            .cells
            .value
            .iter()
            .map(|cell| CloudRegionCellRef {
                value: cell.value.clone(),
            })
            .collect(),
        cell_isolation_evidence,
        state: az_state_label(az.state.value).to_string(),
        created_at_epoch_seconds: az.created_at_epoch_seconds.value,
        schema_version: az.schema_version.value,
    }
}

fn cell_isolation_evidence_record(cell: &CloudCell) -> CloudCellIsolationEvidenceRecord {
    CloudCellIsolationEvidenceRecord {
        cell_id: cell.id.value.value.clone(),
        region_code: cell.region_code.value.value.clone(),
        az_code: cell.az_code.value.value.clone(),
        state: cloud_cell_state_label(cell.state.value).to_string(),
        tenant_density: tenant_density_label(cell.tenant_density.value).to_string(),
        allowed_residency: cell
            .allowed_residency
            .value
            .iter()
            .map(|residency_class| residency_class_label(residency_class).to_string())
            .collect(),
        evidence_ref: format!(
            "cell-isolation://{}/{}/{}",
            cell.region_code.value.value, cell.az_code.value.value, cell.id.value.value
        ),
        schema_version: cell.schema_version.value,
    }
}

fn region_state_label(state: RegionState) -> &'static str {
    match state {
        RegionState::Planned => "planned",
        RegionState::Preview => "preview",
        RegionState::Ga => "ga",
        RegionState::Retiring => "retiring",
    }
}

fn az_state_label(state: AzState) -> &'static str {
    match state {
        AzState::Planned => "planned",
        AzState::Active => "active",
        AzState::DrOnly => "dr_only",
        AzState::Retiring => "retiring",
    }
}

fn cloud_cell_state_label(state: CloudCellState) -> &'static str {
    match state {
        CloudCellState::Planned => "planned",
        CloudCellState::Active => "active",
        CloudCellState::DrOnly => "dr_only",
        CloudCellState::Draining => "draining",
        CloudCellState::Retired => "retired",
    }
}

fn tenant_density_label(density: TenantDensityClass) -> &'static str {
    match density {
        TenantDensityClass::Shared => "shared",
        TenantDensityClass::Dedicated => "dedicated",
        TenantDensityClass::Sovereign => "sovereign",
        TenantDensityClass::AirGapped => "air_gapped",
        TenantDensityClass::FoundryRuntime => "foundry_runtime",
    }
}

fn residency_class_label(residency_class: &ResidencyClass) -> &'static str {
    match residency_class {
        ResidencyClass::StrictHomeRegion => "strict_home_region",
        ResidencyClass::HomeWithRecoveryFailover => "home_with_recovery_failover",
        ResidencyClass::Global => "global",
        ResidencyClass::PerPack(_) => "per_pack",
    }
}
