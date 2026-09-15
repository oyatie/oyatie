//! Fixtures for object sets, shaped as a per-tenant surface: three tenants,
//! the first and the last populated, the middle one reading.
//!
//! Both neighbours hold objects of the queried type under refs the reader has
//! none of, so a set that leaked would name one of them. The reader also
//! holds an object of a second entity type, and `grade` is stored on every
//! object while only revision 2 declares it, so the type law and the filter
//! guard each have something real to exclude.

use std::collections::BTreeMap;

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, ObjectEntity, ObjectProperty,
    OntologyEngine, PropertyTier, PropertyValue,
};
use foundry_projection_draft::{
    AppliedEntry, EntryOutcome, KeyDesignations, MemoryProjectionStore, PageRequest,
    ProjectedObject, ProjectionStore,
};
use foundry_spine::{MAX_SET_MEMBERS, ObjectSet, SetDefinition, SetError, materialize_object_set};

// The reader sorts BETWEEN its neighbours, so a bound dropped at either end
// of the store's tenant scan shows up as a leak.
pub(crate) const FIRST: &str = "ten_alpha";
pub(crate) const READER: &str = "ten_middle";
pub(crate) const LAST: &str = "ten_omega";
pub(crate) const READING: &str = "ety_reading";
pub(crate) const OTHER: &str = "ety_other";

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

/// One projected object carrying `name` and `grade`, written at revision 1.
pub(crate) fn stored(
    tenant_id: &str,
    object_ref: &str,
    entity_type: &str,
    name: &str,
) -> ProjectedObject {
    let properties = [("name", name), ("grade", "A")]
        .into_iter()
        .map(|(property, value)| {
            ObjectProperty::typed(
                property.to_owned(),
                PropertyValue::String(value.to_owned()),
                internal(),
            )
        })
        .collect();
    ProjectedObject {
        entity: ObjectEntity::new(
            tenant_id.to_owned(),
            object_ref.to_owned(),
            entity_type.to_owned(),
            properties,
        )
        .unwrap(),
        schema_revision: 1,
        last_ordinal: 1,
        last_actor: "prn_projector".to_owned(),
    }
}

fn declared(
    tenant_id: &str,
    entity_type: &str,
    revision: u32,
    names: &[&str],
) -> EntityTypeDefinition {
    let properties = names
        .iter()
        .map(|name| {
            EntityTypePropertyDefinition::new(*name, PropertyTier::Scalar, internal(), false)
                .unwrap()
        })
        .collect();
    EntityTypeDefinition::new(
        tenant_id,
        EntityTypeId::new(entity_type).unwrap(),
        "Reading",
        properties,
        revision,
    )
    .unwrap()
}

/// Every tenant declares `ety_reading` with `name` at revision 1 and adds
/// `grade` at 2, and declares `ety_other` at 1.
pub(crate) fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    for tenant_id in [FIRST, READER, LAST] {
        engine
            .register_entity_type(declared(tenant_id, READING, 1, &["name"]))
            .unwrap();
        engine
            .evolve_entity_type(declared(tenant_id, READING, 2, &["name", "grade"]))
            .unwrap();
        engine
            .register_entity_type(declared(tenant_id, OTHER, 1, &["name"]))
            .unwrap();
    }
    engine
}

/// A store holding `objects`, one applied entry each, ordinals dense per
/// tenant in the order given.
pub(crate) fn holding(objects: &[ProjectedObject]) -> MemoryProjectionStore {
    let mut store = MemoryProjectionStore::default();
    let mut ordinals: BTreeMap<String, u64> = BTreeMap::new();
    for object in objects {
        let tenant_id = object.entity.tenant_id.clone();
        let ordinal = ordinals.entry(tenant_id.clone()).or_default();
        *ordinal += 1;
        let outcome = EntryOutcome::Applied {
            objects: vec![object.clone()],
            links: Vec::new(),
        };
        let entry = AppliedEntry {
            tenant_id,
            ordinal: *ordinal,
            outcome,
        };
        store.apply(entry, &KeyDesignations::default()).unwrap();
    }
    store
}

/// The reader holds `ent_mid_a` named Ada and `ent_mid_b` named Grace of the
/// queried type, plus `ent_mid_other` of the second type; each neighbour
/// holds two objects of the queried type under its own refs.
pub(crate) fn three_tenants() -> MemoryProjectionStore {
    holding(&[
        stored(FIRST, "ent_first_a", READING, "Ada"),
        stored(FIRST, "ent_first_b", READING, "Grace"),
        stored(READER, "ent_mid_a", READING, "Ada"),
        stored(READER, "ent_mid_b", READING, "Grace"),
        stored(READER, "ent_mid_other", OTHER, "Ada"),
        stored(LAST, "ent_last_a", READING, "Ada"),
        stored(LAST, "ent_last_b", READING, "Grace"),
    ])
}

pub(crate) fn named(object_refs: &[&str]) -> SetDefinition {
    SetDefinition::Named(
        object_refs
            .iter()
            .map(|object_ref| (*object_ref).to_owned())
            .collect(),
    )
}

/// One page of the set `definition` decides, as the middle tenant: each
/// member's `object_ref` in order, then `next:<ref>` when the page carries a
/// cursor. Membership only — [`rows_of`] renders what each row holds.
pub(crate) fn page_of(
    store: &dyn ProjectionStore,
    definition: SetDefinition,
    pinned: u32,
    page: &PageRequest,
) -> Result<Vec<String>, SetError> {
    let set = ObjectSet {
        entity_type: EntityTypeId::new(READING).unwrap(),
        definition,
    };
    let view = materialize_object_set(store, &registry(), READER, &set, pinned, page)?;
    let mut rows: Vec<String> = view
        .objects
        .iter()
        .map(|(object_ref, _)| object_ref.clone())
        .collect();
    if let Some(cursor) = view.next {
        rows.push(format!("next:{}", cursor.after_object_ref));
    }
    Ok(rows)
}

/// One page as "ref properties rWRITTEN state" per row, so a named leaf's rows
/// are read and not only counted: the same rendering `paged_reads.rs` uses for
/// the listing, over the same helper both build rows with.
pub(crate) fn rows_of(
    store: &dyn ProjectionStore,
    definition: SetDefinition,
    pinned: u32,
) -> Result<Vec<String>, SetError> {
    let set = ObjectSet {
        entity_type: EntityTypeId::new(READING).unwrap(),
        definition,
    };
    let view = materialize_object_set(
        store,
        &registry(),
        READER,
        &set,
        pinned,
        &PageRequest::first(MAX_SET_MEMBERS),
    )?;
    Ok(view
        .objects
        .iter()
        .map(|(object_ref, row)| {
            let names: Vec<&str> = row.properties.keys().map(String::as_str).collect();
            format!(
                "{object_ref} {} r{} {:?}",
                names.join(","),
                row.written_revision,
                row.upcast_state
            )
        })
        .collect())
}

/// A store holding `count` objects of the queried type for the reader, half
/// named Ada and half Grace, so a predicate can take a strict part of it.
pub(crate) fn crowded(count: usize) -> MemoryProjectionStore {
    let objects: Vec<ProjectedObject> = (0..count)
        .map(|index| {
            let name = if index % 2 == 0 { "Ada" } else { "Grace" };
            stored(READER, &format!("ent_{index:05}"), READING, name)
        })
        .collect();
    holding(&objects)
}

/// A store holding the neighbours' objects and none of the reader's, for the
/// null condition: a reader with nothing of its own sees nothing of theirs.
pub(crate) fn neighbours_only() -> MemoryProjectionStore {
    holding(&[
        stored(FIRST, "ent_first_a", READING, "Ada"),
        stored(FIRST, "ent_mid_a", READING, "Ada"),
        stored(LAST, "ent_last_a", READING, "Ada"),
        stored(LAST, "ent_mid_b", READING, "Grace"),
    ])
}

/// The whole membership of `definition` over [`three_tenants`], pinned at
/// revision 1.
pub(crate) fn members(definition: SetDefinition) -> Vec<String> {
    page_of(
        &three_tenants(),
        definition,
        1,
        &PageRequest::first(MAX_SET_MEMBERS),
    )
    .expect("the set is materializable")
}
