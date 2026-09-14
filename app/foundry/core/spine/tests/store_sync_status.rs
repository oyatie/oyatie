//! Reads answered from the DURABLE store: its sync status, and its page.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
};
use data_ontology_kernel::{ObjectEntity, ObjectProperty, PropertyTier};
use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, EntryOutcome, KeyDesignations, MemoryProjectionStore, Page,
    PageRequest, ProjectedLink, ProjectedObject, ProjectionStore, ProjectionStoreError,
    PropertyPredicate,
};
use foundry_spine::{UpcastState, objects_of_type_at_revision, store_sync_status};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn object(object_ref: &str) -> ProjectedObject {
    typed(object_ref, "ety_reading", 1)
}

fn typed(object_ref: &str, entity_type: &str, schema_revision: u32) -> ProjectedObject {
    let properties = ["name", "grade"]
        .into_iter()
        .map(|name| {
            ObjectProperty::new(
                name.to_owned(),
                "Ada".to_owned(),
                PropertyTier::Scalar,
                internal(),
            )
        })
        .collect();
    ProjectedObject {
        entity: ObjectEntity::new(
            "ten_test".to_owned(),
            object_ref.to_owned(),
            entity_type.to_owned(),
            properties,
        )
        .unwrap(),
        schema_revision,
        last_ordinal: 1,
        last_actor: "prn_projector".to_owned(),
    }
}

fn applied(ordinal: u64, object_ref: &str) -> AppliedEntry {
    applied_of(ordinal, object(object_ref))
}

fn applied_of(ordinal: u64, object: ProjectedObject) -> AppliedEntry {
    AppliedEntry {
        tenant_id: "ten_test".to_owned(),
        ordinal,
        outcome: EntryOutcome::Applied {
            objects: vec![object],
            links: Vec::new(),
        },
    }
}

fn poisoned(ordinal: u64, reason: &str) -> AppliedEntry {
    AppliedEntry {
        tenant_id: "ten_test".to_owned(),
        ordinal,
        outcome: EntryOutcome::Poisoned {
            reason: reason.to_owned(),
        },
    }
}

/// A store whose reads fail — an outage must surface, never read as
/// "caught up".
struct BrokenStore;

impl ProjectionStore for BrokenStore {
    fn apply(
        &mut self,
        _entry: AppliedEntry,
        _keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        unreachable!("the lag surface never writes")
    }

    fn applied_head(&self, _tenant_id: &str) -> Result<u64, ProjectionStoreError> {
        Err(ProjectionStoreError::Storage {
            detail: "disk gone".to_owned(),
        })
    }

    fn get(
        &self,
        _tenant_id: &str,
        _object_ref: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        unreachable!()
    }

    fn objects_of_type(
        &self,
        _tenant_id: &str,
        _entity_type: &str,
        _page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        unreachable!()
    }

    fn filter(
        &self,
        _tenant_id: &str,
        _entity_type: &str,
        _predicate: &PropertyPredicate,
        _page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        unreachable!()
    }

    fn links_from(
        &self,
        _tenant_id: &str,
        _object_ref: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        unreachable!()
    }

    fn links_to(
        &self,
        _tenant_id: &str,
        _object_ref: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        unreachable!()
    }

    fn poisoned(&self, _tenant_id: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        unreachable!()
    }
}

#[test]
fn lag_is_measured_against_what_the_store_durably_holds() {
    let mut store = MemoryProjectionStore::default();
    store
        .apply(applied(1, "ent_r1"), &KeyDesignations::default())
        .unwrap();
    store
        .apply(applied(2, "ent_r2"), &KeyDesignations::default())
        .unwrap();

    // The log is at 5; the store durably holds 2.
    let status = store_sync_status(&store, "ten_test", 5).unwrap();
    assert_eq!(status.applied_ordinal, 2);
    assert_eq!(status.head, 5);
    assert_eq!(status.lag, 3, "three entries are not yet durable");
}

#[test]
fn a_caught_up_store_reports_no_lag() {
    let mut store = MemoryProjectionStore::default();
    store
        .apply(applied(1, "ent_r1"), &KeyDesignations::default())
        .unwrap();
    let status = store_sync_status(&store, "ten_test", 1).unwrap();
    assert_eq!(status.lag, 0);
    assert_eq!(status.poisoned_count, 0);
    assert_eq!(status.first_poisoned_ordinal, None);
}

#[test]
fn poisons_come_from_the_store_and_name_where_an_operator_starts() {
    let mut store = MemoryProjectionStore::default();
    store
        .apply(applied(1, "ent_r1"), &KeyDesignations::default())
        .unwrap();
    store
        .apply(poisoned(2, "payload_decode"), &KeyDesignations::default())
        .unwrap();
    store
        .apply(poisoned(3, "receipt_mismatch"), &KeyDesignations::default())
        .unwrap();

    let status = store_sync_status(&store, "ten_test", 3).unwrap();
    assert_eq!(status.poisoned_count, 2);
    assert_eq!(
        status.first_poisoned_ordinal,
        Some(2),
        "the EARLIEST poison is where an operator starts reading",
    );
}

#[test]
fn a_store_outage_surfaces_instead_of_reading_as_caught_up() {
    let failure = store_sync_status(&BrokenStore, "ten_test", 5)
        .expect_err("an unreadable store must not answer");
    assert!(
        matches!(failure, ProjectionStoreError::Storage { .. }),
        "the outage is reported, never rendered as lag 0: {failure:?}",
    );
}

fn declared(entity_type: &str, revision: u32, names: &[&str]) -> EntityTypeDefinition {
    let properties = names
        .iter()
        .map(|name| {
            EntityTypePropertyDefinition::new(*name, PropertyTier::Scalar, internal(), false)
                .unwrap()
        })
        .collect();
    EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new(entity_type).unwrap(),
        "Reading",
        properties,
        revision,
    )
    .unwrap()
}

/// `ety_reading` declares `name` at revision 1 and adds `grade` at 2, both
/// retained; `ety_other` gives a page of one type another to wrongly hold.
fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(declared("ety_reading", 1, &["name"]))
        .unwrap();
    engine
        .evolve_entity_type(declared("ety_reading", 2, &["name", "grade"]))
        .unwrap();
    engine
        .register_entity_type(declared("ety_other", 1, &["name"]))
        .unwrap();
    engine
}

/// One page rendered a row at a time as "ref properties rWRITTEN state",
/// over a store holding `ent_a` written at revision 1, `ent_b` at 2, and
/// `ent_c` of the other type. Every object carries `name` and `grade`, so
/// a property a page drops is the pin's doing, not the object's.
fn rows(entity_type: &str, pinned: u32) -> Vec<String> {
    let mut store = MemoryProjectionStore::default();
    for (ordinal, (object_ref, of_type, revision)) in [
        ("ent_a", "ety_reading", 1),
        ("ent_b", "ety_reading", 2),
        ("ent_c", "ety_other", 1),
    ]
    .into_iter()
    .enumerate()
    {
        let entry = applied_of(ordinal as u64 + 1, typed(object_ref, of_type, revision));
        store.apply(entry, &KeyDesignations::default()).unwrap();
    }
    objects_of_type_at_revision(
        &store,
        &registry(),
        "ten_test",
        &EntityTypeId::new(entity_type).unwrap(),
        pinned,
        &PageRequest::first(10),
    )
    .unwrap()
    .objects
    .iter()
    .map(|(object_ref, view)| {
        let names: Vec<&str> = view.properties.keys().map(String::as_str).collect();
        format!(
            "{object_ref} {} r{} {:?}",
            names.join(","),
            view.written_revision,
            view.upcast_state
        )
    })
    .collect()
}

/// Both sides of the pin, row by row, and only the queried type: behind
/// the pin no row carries the property it does not declare, at it every
/// row does, and each row reports its own written revision.
#[test]
fn every_row_of_a_page_is_pinned_on_its_own_and_only_its_type_is_held() {
    assert_eq!(
        rows("ety_reading", 1),
        ["ent_a name r1 Current", "ent_b name r2 Current"],
        "revision 1 declares no grade, and both writes are at or beyond it",
    );
    assert_eq!(
        rows("ety_reading", 2),
        [
            "ent_a grade,name r1 UpcastPending",
            "ent_b grade,name r2 Current"
        ],
        "revision 2 declares grade, and only the earlier write is behind it",
    );
    assert_eq!(
        rows("ety_other", 1),
        ["ent_c name r1 Current"],
        "ent_c was there for the pages above to wrongly include",
    );
}
