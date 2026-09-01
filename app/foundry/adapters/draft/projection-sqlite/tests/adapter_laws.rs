//! Adapter-specific laws beyond the shared suite: byte-faithful
//! round-trips across reopen, the Date index key's order agreement with
//! the kernel's `Ord`, the property index actually serving queries, and
//! the typed columns never aliasing across kinds.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// A monotonic per-process counter. The wall-clock recipe alone
/// duplicated across parallel tests under load — two tests then shared
/// one database and produced divergent-replay and malformed-image
/// failures that read as real defects.
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    CalendarDate, FiniteDouble, ObjectEntity, ObjectProperty, PropertyTier, PropertyValue,
};
use foundry_projection_draft::{
    AppliedEntry, EntryOutcome, KeyDesignations, PageRequest, ProjectedObject, ProjectionStore,
    ProjectionStoreError, PropertyPredicate,
};
use foundry_projection_sqlite_draft::{PROPERTY_INDEX_NAME, SqliteProjectionStore};

fn scratch(case: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "foundry-projection-laws-{case}-{}-{}-{}.sqlite",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
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
        outcome: EntryOutcome::Applied {
            objects,
            links: Vec::new(),
        },
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
    sweep(&path);
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
        store
            .apply(entry(1, vec![stored.clone()]), &KeyDesignations::default())
            .unwrap();
    }
    let store = SqliteProjectionStore::open(&path).unwrap();
    let read = store.get("ten_a", "ent_rich").unwrap();
    assert_eq!(read.as_ref(), Some(&stored), "Eq-identical after reopen");
    sweep(&path);
}

#[test]
fn the_date_index_key_agrees_with_kernel_order() {
    let path = scratch("dates");
    sweep(&path);
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
            .apply(
                entry(
                    ordinal as u64 + 1,
                    vec![projected(object_ref, vec![typed("closed_on", date)])],
                ),
                &KeyDesignations::default(),
            )
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
    sweep(&path);
}

#[test]
fn typed_index_columns_never_alias_across_kinds() {
    let path = scratch("alias");
    sweep(&path);
    let mut store = SqliteProjectionStore::open(&path).unwrap();
    store
        .apply(
            entry(
                1,
                vec![
                    projected(
                        "ent_bool",
                        vec![typed("flag", PropertyValue::Boolean(true))],
                    ),
                    projected("ent_int", vec![typed("flag", PropertyValue::Integer(1))]),
                ],
            ),
            &KeyDesignations::default(),
        )
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
    sweep(&path);
}

#[test]
fn predicate_queries_run_through_the_property_index() {
    let path = scratch("plan");
    sweep(&path);
    let mut store = SqliteProjectionStore::open(&path).unwrap();
    store
        .apply(
            entry(
                1,
                vec![projected(
                    "ent_a1",
                    vec![typed("celsius", PropertyValue::Integer(21))],
                )],
            ),
            &KeyDesignations::default(),
        )
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
    sweep(&path);
}

/// A head that will not convert is a CORRUPT store, not an empty one,
/// so the discard refuses. `try_into().unwrap_or(0)` returned `Ok(0)` —
/// nothing discarded — while deleting every row: the exact "loss" a
/// returned head exists to distinguish itself from.
#[test]
fn a_corrupt_head_refuses_the_discard_and_keeps_the_rows() {
    let path = scratch("corrupt-head");
    sweep(&path);
    let stored = projected(
        "ent_a1",
        vec![typed("name", PropertyValue::String("Ada".to_owned()))],
    );
    {
        let mut store = SqliteProjectionStore::open(&path).unwrap();
        store
            .apply(entry(1, vec![stored.clone()]), &KeyDesignations::default())
            .unwrap();
    }
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute(
            "UPDATE projection_heads SET applied_ordinal = -5 WHERE tenant_id = 'ten_a'",
            [],
        )
        .unwrap();

    let mut store = SqliteProjectionStore::open(&path).unwrap();
    let refused = store.reset_tenant("ten_a");

    assert!(
        matches!(refused, Err(ProjectionStoreError::Storage { .. })),
        "a corrupt head refuses: {refused:?}"
    );
    // The refusal rolls the transaction back: nothing was lost.
    let survivors = rusqlite::Connection::open(&path)
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM projection_objects WHERE tenant_id = 'ten_a'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap();
    assert_eq!(survivors, 1, "nothing was destroyed by the refusal");
    sweep(&path);
}

/// WAL writes two sidecars beside the database; removing only the file
/// leaked 72 strays per suite run, unbounded once names became unique.
fn sweep(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    for suffix in ["-wal", "-shm"] {
        let mut sidecar = path.as_os_str().to_owned();
        sidecar.push(suffix);
        let _ = std::fs::remove_file(std::path::PathBuf::from(sidecar));
    }
}
