//! Read and predicate laws of the conformance suite.

use data_ontology_kernel::PropertyValue;

use crate::conformance::{ProjectionFixture, applied, fail, object};
use crate::predicate::{PredicateError, PropertyPredicate};
use crate::store::{PageRequest, ProjectionStore, ProjectionStoreError};

pub fn check_get_returns_the_projected_object<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    let stored = object(
        "ten_a",
        "ent_a1",
        "ety_reading",
        vec![("name", PropertyValue::String("Ada".to_owned()))],
    );
    store
        .apply(applied("ten_a", 1, vec![stored.clone()]))
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    let read = store
        .get("ten_a", "ent_a1")
        .map_err(|error| fail("get reads", format!("{error:?}")))?;
    if read.as_ref() != Some(&stored) {
        return Err(fail("get returns the stored object", format!("{read:?}")));
    }
    Ok(())
}

pub fn check_reads_are_tenant_isolated<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(applied(
            "ten_a",
            1,
            vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
        ))
        .map_err(|error| fail("tenant a seed", format!("{error:?}")))?;
    store
        .apply(applied(
            "ten_b",
            1,
            vec![object("ten_b", "ent_b1", "ety_reading", vec![])],
        ))
        .map_err(|error| fail("tenant b seed", format!("{error:?}")))?;
    let crossed = store
        .get("ten_a", "ent_b1")
        .map_err(|error| fail("get reads", format!("{error:?}")))?;
    if crossed.is_some() {
        return Err(fail("no cross-tenant get", format!("{crossed:?}")));
    }
    let page = store
        .objects_of_type("ten_a", "ety_reading", &PageRequest::first(10))
        .map_err(|error| fail("scan reads", format!("{error:?}")))?;
    if page.objects.len() != 1 || page.objects[0].entity.id != "ent_a1" {
        return Err(fail("scans stay inside the tenant", format!("{page:?}")));
    }
    Ok(())
}

pub fn check_type_scan_pages_partition_deterministically<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    for (ordinal, object_ref) in ["ent_a1", "ent_a2", "ent_a3", "ent_a4", "ent_a5"]
        .into_iter()
        .enumerate()
    {
        let entity_type = if object_ref == "ent_a4" {
            "ety_other"
        } else {
            "ety_reading"
        };
        store
            .apply(applied(
                "ten_a",
                ordinal as u64 + 1,
                vec![object("ten_a", object_ref, entity_type, vec![])],
            ))
            .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    }
    let mut seen = Vec::new();
    let mut request = PageRequest::first(2);
    loop {
        let page = store
            .objects_of_type("ten_a", "ety_reading", &request)
            .map_err(|error| fail("scan reads", format!("{error:?}")))?;
        seen.extend(page.objects.iter().map(|stored| stored.entity.id.clone()));
        match page.next {
            Some(cursor) => request = PageRequest::after(2, cursor),
            None => break,
        }
    }
    if seen != vec!["ent_a1", "ent_a2", "ent_a3", "ent_a5"] {
        return Err(fail(
            "pages partition the type's full result in order",
            format!("{seen:?}"),
        ));
    }
    let past_the_end = store
        .objects_of_type(
            "ten_a",
            "ety_reading",
            &PageRequest::after(
                2,
                crate::store::ProjectionCursor {
                    after_object_ref: "ent_a5".to_owned(),
                },
            ),
        )
        .map_err(|error| fail("past-the-end reads", format!("{error:?}")))?;
    if !past_the_end.objects.is_empty() || past_the_end.next.is_some() {
        return Err(fail(
            "past the end is an empty page",
            format!("{past_the_end:?}"),
        ));
    }
    Ok(())
}

pub fn check_equals_predicate_matches_exactly<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(applied(
            "ten_a",
            1,
            vec![
                object(
                    "ten_a",
                    "ent_a1",
                    "ety_reading",
                    vec![("celsius", PropertyValue::Integer(21))],
                ),
                object(
                    "ten_a",
                    "ent_a2",
                    "ety_reading",
                    vec![("celsius", PropertyValue::Integer(35))],
                ),
                object("ten_a", "ent_a3", "ety_reading", vec![]),
            ],
        ))
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    let predicate = PropertyPredicate::equals("celsius", PropertyValue::Integer(21))
        .map_err(|error| fail("equals constructs", format!("{error:?}")))?;
    let page = store
        .filter("ten_a", "ety_reading", &predicate, &PageRequest::first(10))
        .map_err(|error| fail("filter reads", format!("{error:?}")))?;
    let refs: Vec<&str> = page
        .objects
        .iter()
        .map(|stored| stored.entity.id.as_str())
        .collect();
    if refs != vec!["ent_a1"] {
        return Err(fail(
            "equals matches exactly; absent property is no match",
            format!("{refs:?}"),
        ));
    }
    Ok(())
}

pub fn check_range_predicate_is_class_scoped<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    for (ordinal, (object_ref, celsius)) in [("ent_a1", 3), ("ent_a2", 5), ("ent_a3", 7)]
        .into_iter()
        .enumerate()
    {
        store
            .apply(applied(
                "ten_a",
                ordinal as u64 + 1,
                vec![object(
                    "ten_a",
                    object_ref,
                    "ety_reading",
                    vec![("celsius", PropertyValue::Integer(celsius))],
                )],
            ))
            .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    }
    let predicate = PropertyPredicate::range(
        "celsius",
        PropertyValue::Integer(3),
        PropertyValue::Integer(5),
    )
    .map_err(|error| fail("range constructs", format!("{error:?}")))?;
    let page = store
        .filter("ten_a", "ety_reading", &predicate, &PageRequest::first(10))
        .map_err(|error| fail("filter reads", format!("{error:?}")))?;
    let refs: Vec<&str> = page
        .objects
        .iter()
        .map(|stored| stored.entity.id.as_str())
        .collect();
    if refs != vec!["ent_a1", "ent_a2"] {
        return Err(fail(
            "the range is inclusive of both bounds",
            format!("{refs:?}"),
        ));
    }
    match PropertyPredicate::range(
        "celsius",
        PropertyValue::Integer(1),
        PropertyValue::String("x".to_owned()),
    ) {
        Err(PredicateError::MixedStorageClasses) => {}
        other => {
            return Err(fail(
                "mixed classes refuse construction",
                format!("{other:?}"),
            ));
        }
    }
    match PropertyPredicate::range(
        "celsius",
        PropertyValue::Integer(9),
        PropertyValue::Integer(1),
    ) {
        Err(PredicateError::InvertedRange) => Ok(()),
        other => Err(fail("an inverted range is refused", format!("{other:?}"))),
    }
}

pub fn check_range_class_mismatch_is_refused<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(applied(
            "ten_a",
            1,
            vec![object(
                "ten_a",
                "ent_a1",
                "ety_reading",
                vec![("celsius", PropertyValue::String("warm".to_owned()))],
            )],
        ))
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    let predicate = PropertyPredicate::range(
        "celsius",
        PropertyValue::Integer(1),
        PropertyValue::Integer(9),
    )
    .map_err(|error| fail("range constructs", format!("{error:?}")))?;
    match store.filter("ten_a", "ety_reading", &predicate, &PageRequest::first(10)) {
        Err(ProjectionStoreError::ClassMismatch { property }) if property == "celsius" => Ok(()),
        other => Err(fail(
            "a class-mismatched stored value is loud, never silent false",
            format!("{other:?}"),
        )),
    }
}
