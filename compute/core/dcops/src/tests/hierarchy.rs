use super::support::*;
use crate::*;

#[test]
fn builds_active_dcops_hierarchy_with_strict_parent_states() {
    let catalog = active_catalog();
    assert_eq!(catalog.sites().count(), 1);
    let inactive_site_error = {
        let mut catalog = CloudDcopsCatalog::default();
        catalog.add_site(site_create()).expect("site");
        catalog
            .add_facility_zone(FacilityZoneCreate {
                id: HALL_ID.to_string(),
                site_id: SITE_ID.to_string(),
                kind: FacilityZoneKind::DataHall,
                state: FacilityZoneState::Planned,
                display_name: "hall a".to_string(),
                created_at_epoch_seconds: 4,
            })
            .expect_err("inactive site must reject child")
    };
    assert_eq!(inactive_site_error, CloudDcopsError::InactiveParent);
}

#[test]
fn rejects_forged_initial_states_and_bad_region_or_redundancy() {
    assert_eq!(
        DatacenterSite::new(DatacenterSiteCreate {
            state: DatacenterState::Active,
            ..site_create()
        })
        .expect_err("site active state is forged"),
        CloudDcopsError::InvalidInitialState
    );
    assert_eq!(
        DatacenterSite::new(DatacenterSiteCreate {
            id: "dc/region-beta1/site-a".to_string(),
            ..site_create()
        })
        .expect_err("id region must match payload"),
        CloudDcopsError::RegionMismatch
    );
    assert_eq!(
        PowerZone::new(PowerZoneCreate {
            id: POWER_ID.to_string(),
            site_id: SITE_ID.to_string(),
            redundancy: PowerRedundancy::TwoNPlusOne,
            state: PowerZoneState::Planned,
            capacity_watts: 1,
            utility_feed_count: 2,
            created_at_epoch_seconds: 1,
        })
        .expect_err("2N+1 requires three feeds"),
        CloudDcopsError::InvalidRedundancy
    );
}
