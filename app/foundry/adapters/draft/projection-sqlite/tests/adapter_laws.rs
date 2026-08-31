//! Adapter-specific laws beyond the shared suite: byte-faithful
//! round-trips across reopen, the Date index key's order agreement with
//! the kernel's `Ord`, the property index actually serving queries, and
//! the typed columns never aliasing across kinds.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    CalendarDate, FiniteDouble, ObjectEntity, ObjectProperty, PropertyTier, PropertyValue,
};
use foundry_projection_draft::{
    AppliedEntry, EntryOutcome, PageRequest, ProjectedObject, ProjectionStore, PropertyPredicate,
};
use foundry_projection_sqlite_draft::{PROPERTY_INDEX_NAME, SqliteProjectionStore};

fn scratch(case: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "foundry-projection-laws-{case}-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::internal_only()
}

fn typed(name: &str, value: PropertyValue) -> ObjectProperty {
    ObjectProperty::typed(name.to_owned(), value, internal())
}

fn entry(ordinal: u64, objects: Vec<ProjectedObject>) -> AppliedEntry {
    AppliedEntry {
        tenant_id: "ten_a".to_owned(),
        ordinal,
        outcome: EntryOutcome::Applied { objects },
    }
}

fn projected(object_ref: &str, properties: Vec<ObjectProperty>) -> ProjectedObject {
    ProjectedObject {
        entity: ObjectEntity::new(
            "ten_a".to_owned(),
            object_ref.to_owned(),
            "ety_reading".to_owned(),
            properties,
        )
        .unwrap(),
        schema_revision: 3,
        last_ordinal: 1,
        last_actor: "prn_projector".to_owned(),
    }
}

#[test]
fn a_rich_object_round_trips_byte_faithfully_across_reopen() {
    let path = scratch("roundtrip");
    let _ = std::fs::remove_file(&path);
    let mut nested = BTreeMap::new();
    nested.insert("lat".to_owned(), PropertyValue::Integer(37));
    nested.insert(
        "site".to_owned(),
        PropertyValue::Array(vec![PropertyValue::String("hq".to_owned())]),
    );
    let exotic_tier = ObjectProperty {
        name: "trace".to_owned(),
        value: Classified::new(
            PropertyValue::String("t1,t2".to_owned()),
            PrivacyDataClass::new(DataClass::PiiIdentifying).unwrap(),
        ),
        tier: PropertyTier::Timeseries,
    };
    let stored = projected(
        "ent_rich",
        vec![
            typed("name", PropertyValue::String("Ada".to_owned())),
            typed("celsius", PropertyValue::Integer(-40)),
            typed(
                "ratio",
                PropertyValue::Double(FiniteDouble::new(-2.5).unwrap()),
            ),
            typed("armed", PropertyValue::Boolean(true)),
            typed(
                "born",
                PropertyValue::Date(CalendarDate::new(1815, 12, 10).unwrap()),
            ),
            typed("seen", PropertyValue::Timestamp { epoch_millis: -5 }),
            typed("place", PropertyValue::Struct(nested)),
            exotic_tier,
        ],
    );
    {
        let mut store = SqliteProjectionStore::open(&path).unwrap();
        store.apply(entry(1, vec![stored.clone()])).unwrap();
    }
    let store = SqliteProjectionStore::open(&path).unwrap();
    let read = store.get("ten_a", "ent_rich").unwrap();
    assert_eq!(read.as_ref(), Some(&stored), "Eq-identical after reopen");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_date_index_key_agrees_with_kernel_order() {
    let path = scratch("dates");
    let _ = std::fs::remove_file(&path);
    let mut store = SqliteProjectionStore::open(&path).unwrap();
    for (ordinal, (object_ref, y, m, d)) in [
        ("ent_d1", 2023, 12, 31),
        ("ent_d2", 2024, 1, 1),
        ("ent_d3", 2024, 2, 29),
    ]
    .into_iter()
    .enumerate()
    {
        let date = PropertyValue::Date(CalendarDate::new(y, m, d).unwrap());
        store
            .apply(entry(
                ordinal as u64 + 1,
                vec![projected(object_ref, vec![typed("closed_on", date)])],
            ))
            .unwrap();
    }
    let predicate = PropertyPredicate::range(
        "closed_on",
        PropertyValue::Date(CalendarDate::new(2024, 1, 1).unwrap()),
        PropertyValue::Date(CalendarDate::new(2024, 12, 31).unwrap()),
    )
    .unwrap();
    let page = store
        .filter("ten_a", "ety_reading", &predicate, &PageRequest::first(10))
        .unwrap();
    let refs: Vec<&str> = page
        .objects
        .iter()
        .map(|object| object.entity.id.as_str())
        .collect();
    assert_eq!(
        refs,
        vec!["ent_d2", "ent_d3"],
        "the year-boundary date stays out; chronology, not string order",
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn typed_index_columns_never_alias_across_kinds() {
    let path = scratch("alias");
    let _ = std::fs::remove_file(&path);
    let mut store = SqliteProjectionStore::open(&path).unwrap();
    store
        .apply(entry(
            1,
            vec![
                projected(
                    "ent_bool",
                    vec![typed("flag", PropertyValue::Boolean(true))],
                ),
                projected("ent_int", vec![typed("flag", PropertyValue::Integer(1))]),
            ],
        ))
        .unwrap();
    let predicate = PropertyPredicate::equals("flag", PropertyValue::Integer(1)).unwrap();
    let page = store
        .filter("ten_a", "ety_reading", &predicate, &PageRequest::first(10))
        .unwrap();
    let refs: Vec<&str> = page
        .objects
        .iter()
        .map(|object| object.entity.id.as_str())
        .collect();
    assert_eq!(
        refs,
        vec!["ent_int"],
        "int_value=1 rows of Boolean kind never alias Integer(1)",
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn predicate_queries_run_through_the_property_index() {
    let path = scratch("plan");
    let _ = std::fs::remove_file(&path);
    let mut store = SqliteProjectionStore::open(&path).unwrap();
    store
        .apply(entry(
            1,
            vec![projected(
                "ent_a1",
                vec![typed("celsius", PropertyValue::Integer(21))],
            )],
        ))
        .unwrap();
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    let plan: String = connection
        .query_row(
            "EXPLAIN QUERY PLAN
             SELECT o.object_bytes FROM projection_property_index i
             JOIN projection_objects o
               ON o.tenant_id = i.tenant_id AND o.object_ref = i.object_ref
             WHERE i.tenant_id = 'ten_a' AND i.entity_type = 'ety_reading'
               AND i.property = 'celsius' AND i.value_kind = 'integer'
               AND i.object_ref > ''
             ORDER BY i.object_ref",
            [],
            |row| row.get(3),
        )
        .unwrap();
    assert!(
        plan.contains(PROPERTY_INDEX_NAME),
        "the shaped query uses the property index, not a table scan: {plan}",
    );
    let _ = std::fs::remove_file(&path);
}
