//! The paged, revision-pinned view over the durable store, and the filter
//! a caller may put on it.
//!
//! A filter is evaluated by the store against the object AS STORED, while
//! the rows the page returns are filtered to the pinned vocabulary. So a
//! filter may only name a property the pin declares: otherwise the page
//! would select rows by a value the response omits, and the surface would
//! answer questions about a property outside the pin. `grade` below is
//! exactly that property — stored on every object, declared only at
//! revision 2 — so the guard has something real to refuse.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, ObjectEntity, ObjectProperty,
    OntologyEngine, PropertyTier, PropertyValue,
};
use foundry_projection_draft::{
    AppliedEntry, EntryOutcome, KeyDesignations, MemoryProjectionStore, PageRequest,
    ProjectedObject, ProjectionStore, PropertyPredicate,
};
use foundry_spine::{PageError, objects_of_type_at_revision};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

/// Every object carries `name`, `grade` and an integer `rank`, so a filter
/// on a property the pin does not declare has a stored value to match on.
/// `grade` differs between objects, so a filter on it selects rather than
/// returning whatever the unfiltered page would.
fn stored(
    object_ref: &str,
    entity_type: &str,
    schema_revision: u32,
    rank: i64,
    grade: &str,
) -> ProjectedObject {
    let properties = [
        ObjectProperty::typed(
            "name".to_owned(),
            PropertyValue::String(object_ref.to_owned()),
            internal(),
        ),
        ObjectProperty::typed(
            "grade".to_owned(),
            PropertyValue::String(grade.to_owned()),
            internal(),
        ),
        ObjectProperty::typed("rank".to_owned(), PropertyValue::Integer(rank), internal()),
    ]
    .into_iter()
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

/// `ety_reading` declares `name` and `rank` at revision 1 and adds `grade`
/// at 2, both retained. `ety_other` gives a page of one type another type
/// to wrongly hold.
fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(declared("ety_reading", 1, &["name", "rank"]))
        .unwrap();
    engine
        .evolve_entity_type(declared("ety_reading", 2, &["name", "rank", "grade"]))
        .unwrap();
    engine
        .register_entity_type(declared("ety_other", 1, &["name", "rank"]))
        .unwrap();
    engine
}

/// `ent_a` written at revision 1 with rank 10, `ent_b` at revision 2 with
/// rank 20, and `ent_c` of the other type with rank 30.
fn populated() -> MemoryProjectionStore {
    let mut store = MemoryProjectionStore::default();
    for (ordinal, (object_ref, of_type, revision, rank, grade)) in [
        ("ent_a", "ety_reading", 1, 10, "A"),
        ("ent_b", "ety_reading", 2, 20, "B"),
        ("ent_c", "ety_other", 1, 30, "A"),
    ]
    .into_iter()
    .enumerate()
    {
        let entry = AppliedEntry {
            tenant_id: "ten_test".to_owned(),
            ordinal: ordinal as u64 + 1,
            outcome: EntryOutcome::Applied {
                objects: vec![stored(object_ref, of_type, revision, rank, grade)],
                links: Vec::new(),
            },
        };
        store.apply(entry, &KeyDesignations::default()).unwrap();
    }
    store
}

/// One page as "ref properties rWRITTEN state" per row.
fn page(
    entity_type: &str,
    pinned: u32,
    filter: Option<&PropertyPredicate>,
) -> Result<Vec<String>, PageError> {
    let view = objects_of_type_at_revision(
        &populated(),
        &registry(),
        "ten_test",
        &EntityTypeId::new(entity_type).unwrap(),
        pinned,
        &PageRequest::first(10),
        filter,
    )?;
    Ok(view
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
        .collect())
}

fn rows(entity_type: &str, pinned: u32) -> Vec<String> {
    page(entity_type, pinned, None).expect("an unfiltered page is served")
}

/// Both sides of the pin, row by row, and only the queried type: behind the
/// pin no row carries the property it does not declare, at it every row
/// does, and each row reports its own written revision.
#[test]
fn every_row_of_a_page_is_pinned_on_its_own_and_only_its_type_is_held() {
    assert_eq!(
        rows("ety_reading", 1),
        ["ent_a name,rank r1 Current", "ent_b name,rank r2 Current"],
        "revision 1 declares no grade, and both writes are at or beyond it",
    );
    assert_eq!(
        rows("ety_reading", 2),
        [
            "ent_a grade,name,rank r1 UpcastPending",
            "ent_b grade,name,rank r2 Current"
        ],
        "revision 2 declares grade, and only the earlier write is behind it",
    );
    assert_eq!(
        rows("ety_other", 1),
        ["ent_c name,rank r1 Current"],
        "ent_c was there for the pages above to wrongly include",
    );
}

/// Equality and an inclusive range, each selecting a strict subset of the
/// unfiltered page above.
#[test]
fn a_filter_selects_by_a_declared_property() {
    let equals = PropertyPredicate::equals("name", PropertyValue::String("ent_b".to_owned()))
        .expect("a well-formed equality");
    assert_eq!(
        page("ety_reading", 1, Some(&equals)).unwrap(),
        ["ent_b name,rank r2 Current"],
    );

    let narrow = PropertyPredicate::range(
        "rank",
        PropertyValue::Integer(5),
        PropertyValue::Integer(15),
    )
    .expect("a well-formed range");
    assert_eq!(
        page("ety_reading", 1, Some(&narrow)).unwrap(),
        ["ent_a name,rank r1 Current"],
    );

    let inclusive = PropertyPredicate::range(
        "rank",
        PropertyValue::Integer(10),
        PropertyValue::Integer(20),
    )
    .expect("a well-formed range");
    assert_eq!(
        page("ety_reading", 1, Some(&inclusive)).unwrap(),
        ["ent_a name,rank r1 Current", "ent_b name,rank r2 Current"],
        "an inclusive range takes both of its ends",
    );
}

/// A filter that matches nothing is an empty page, not a refusal — the
/// other side of the boundary the refusals below stand on.
#[test]
fn a_filter_matching_nothing_serves_an_empty_page() {
    let none = PropertyPredicate::equals("name", PropertyValue::String("ent_absent".to_owned()))
        .expect("a well-formed equality");
    assert_eq!(
        page("ety_reading", 1, Some(&none)).unwrap(),
        Vec::<String>::new()
    );
}

/// The guard, for each shape a predicate has: `grade` is stored on every
/// object and declared only at revision 2. Filtering on it at revision 1 is
/// refused, because the rows it would select do not carry the value that
/// selected them. At revision 2, where the pin declares it, the same filter
/// is served — and selects, since the two objects carry different grades.
#[test]
fn a_filter_on_a_property_the_pin_does_not_declare_is_refused() {
    let on_grade = PropertyPredicate::equals("grade", PropertyValue::String("A".to_owned()))
        .expect("a well-formed equality");
    assert_eq!(
        page("ety_reading", 1, Some(&on_grade)),
        Err(PageError::UndeclaredFilterProperty),
    );
    assert_eq!(
        page("ety_reading", 2, Some(&on_grade)).unwrap(),
        ["ent_a grade,name,rank r1 UpcastPending"],
        "revision 2 declares grade, so the same filter is answerable",
    );

    let range_on_grade = PropertyPredicate::range(
        "grade",
        PropertyValue::String("B".to_owned()),
        PropertyValue::String("Z".to_owned()),
    )
    .expect("a well-formed range");
    assert_eq!(
        page("ety_reading", 1, Some(&range_on_grade)),
        Err(PageError::UndeclaredFilterProperty),
        "a range is guarded by the property it names, as an equality is",
    );
    assert_eq!(
        page("ety_reading", 2, Some(&range_on_grade)).unwrap(),
        ["ent_b grade,name,rank r2 Current"],
    );

    let absent = PropertyPredicate::equals("nothing", PropertyValue::String("x".to_owned()))
        .expect("a well-formed equality");
    assert_eq!(
        page("ety_reading", 2, Some(&absent)),
        Err(PageError::UndeclaredFilterProperty),
        "a property no revision declares is refused the same way",
    );
}

/// A range whose bounds are of one kind over stored values of another is
/// the caller's error, reported as its own refusal rather than as an
/// unreadable store.
#[test]
fn a_range_over_values_of_another_kind_is_a_filter_refusal_not_a_store_fault() {
    let text_over_ints = PropertyPredicate::range(
        "rank",
        PropertyValue::String("a".to_owned()),
        PropertyValue::String("z".to_owned()),
    )
    .expect("the bounds agree with each other");
    assert_eq!(
        page("ety_reading", 1, Some(&text_over_ints)),
        Err(PageError::FilterKindMismatch {
            property: "rank".to_owned()
        }),
    );
}
