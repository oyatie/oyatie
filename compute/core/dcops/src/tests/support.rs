use crate::*;

pub(super) const SITE_ID: &str = "dc/region-alpha1/site-a";
pub(super) const HALL_ID: &str = "zone/dc/region-alpha1/site-a/hall-a";
pub(super) const POWER_ID: &str = "power/dc/region-alpha1/site-a/power-a";
pub(super) const COOLING_ID: &str = "cooling/dc/region-alpha1/site-a/cooling-a";
pub(super) const SECURITY_ID: &str = "security/dc/region-alpha1/site-a/sec-a";
pub(super) const RACK_ID_VALUE: &str = "rack/dc/region-alpha1/site-a/rack-a";
pub(super) const EQUIP_ID: &str = "equip/dc/region-alpha1/site-a/server-a";
pub(super) const EQUIP_ID_B: &str = "equip/dc/region-alpha1/site-a/server-b";

pub(super) fn site_create() -> DatacenterSiteCreate {
    DatacenterSiteCreate {
        id: SITE_ID.to_string(),
        region: "region-alpha1".to_string(),
        availability_zone: "region-alpha1-a".to_string(),
        physical_ref: "physical/colo/site-a".to_string(),
        phase: DcSubstratePhase::ColoCage,
        tier: DatacenterTier::Tier3,
        state: DatacenterState::Planned,
        provider_facing: true,
        pue_target_milli: 1_500,
        wue_target_milli: 2_000,
        cue_target_milli: 1_000,
        created_at_epoch_seconds: 1,
    }
}

pub(super) fn active_catalog() -> CloudDcopsCatalog {
    let mut catalog = CloudDcopsCatalog::default();
    let site = catalog.add_site(site_create()).expect("site");
    catalog
        .transition_site(&site.id.value, DatacenterState::Commissioning, 2)
        .expect("commissioning");
    catalog
        .transition_site(&site.id.value, DatacenterState::Active, 3)
        .expect("active");
    let hall = catalog
        .add_facility_zone(FacilityZoneCreate {
            id: HALL_ID.to_string(),
            site_id: SITE_ID.to_string(),
            kind: FacilityZoneKind::DataHall,
            state: FacilityZoneState::Planned,
            display_name: "hall a".to_string(),
            created_at_epoch_seconds: 4,
        })
        .expect("hall");
    catalog
        .transition_facility_zone(&hall.id.value, FacilityZoneState::Active, 5)
        .expect("hall active");
    let power = catalog
        .add_power_zone(PowerZoneCreate {
            id: POWER_ID.to_string(),
            site_id: SITE_ID.to_string(),
            redundancy: PowerRedundancy::TwoN,
            state: PowerZoneState::Planned,
            capacity_watts: 20_000,
            utility_feed_count: 2,
            created_at_epoch_seconds: 6,
        })
        .expect("power");
    catalog
        .transition_power_zone(&power.id.value, PowerZoneState::Energized, 7)
        .expect("power energized");
    let cooling = catalog
        .add_cooling_zone(CoolingZoneCreate {
            id: COOLING_ID.to_string(),
            site_id: SITE_ID.to_string(),
            technology: CoolingTechnology::ChilledWater,
            state: CoolingZoneState::Planned,
            heat_capacity_watts: 20_000,
            water_budget_liters_per_hour: 10_000,
            created_at_epoch_seconds: 8,
        })
        .expect("cooling");
    catalog
        .transition_cooling_zone(&cooling.id.value, CoolingZoneState::Active, 9)
        .expect("cooling active");
    let security = catalog
        .add_security_zone(SecurityZoneCreate {
            id: SECURITY_ID.to_string(),
            site_id: SITE_ID.to_string(),
            kind: SecurityZoneKind::Badge,
            state: SecurityZoneState::Planned,
            created_at_epoch_seconds: 10,
        })
        .expect("security");
    catalog
        .transition_security_zone(&security.id.value, SecurityZoneState::Armed, 11)
        .expect("security armed");
    let rack = catalog
        .add_rack(RackCreate {
            id: RACK_ID_VALUE.to_string(),
            site_id: SITE_ID.to_string(),
            facility_zone_id: HALL_ID.to_string(),
            security_zone_id: SECURITY_ID.to_string(),
            row_label: "row-a".to_string(),
            state: RackState::Planned,
            u_height: 42,
            rated_power_watts: 12_000,
            max_heat_watts: 12_000,
            max_weight_kg: 1_200,
            created_at_epoch_seconds: 12,
        })
        .expect("rack");
    catalog
        .transition_rack(&rack.id.value, RackState::Active, 13)
        .expect("rack active");
    catalog
}

pub(super) fn equipment_create(id: &str) -> EquipmentCreate {
    EquipmentCreate {
        id: id.to_string(),
        site_id: SITE_ID.to_string(),
        kind: EquipmentKind::Server,
        lifecycle: EquipmentLifecycle::Ordered,
        procurement_ref: "proc/order-1/server".to_string(),
        vendor: "approved-vendor".to_string(),
        model: "srv-1".to_string(),
        ordered_at_epoch_seconds: 20,
    }
}

pub(super) fn install_plan(start_u: u16, power_watts: u64) -> EquipmentInstallPlan {
    EquipmentInstallPlan {
        rack_id: RACK_ID_VALUE.to_string(),
        power_zone_id: POWER_ID.to_string(),
        cooling_zone_id: COOLING_ID.to_string(),
        start_u,
        height_u: 2,
        power_watts,
        heat_watts: power_watts,
        weight_kg: 35,
        network_drop_refs: vec!["netdrop/rack-a/a1".to_string()],
        installed_at_epoch_seconds: 30 + u64::from(start_u),
    }
}

pub(super) fn received_equipment(catalog: &mut CloudDcopsCatalog, id: &str) -> EquipmentId {
    let equipment = catalog
        .order_equipment(equipment_create(id))
        .expect("ordered");
    let asset = format!("asset/{SITE_ID}/{}", id.rsplit('/').next().expect("slug"));
    catalog
        .receive_equipment(&equipment.id.value, asset, "serial-1".to_string(), 25)
        .expect("received");
    equipment.id.value
}
