//! Predicate laws of the conformance suite: exact typed equality and
//! kind-scoped inclusive ranges, refusals loud.

use data_ontology_kernel::PropertyValue;

use crate::conformance::{ProjectionFixture, applied, fail, object};
use crate::predicate::{PredicateError, PropertyPredicate};
use crate::store::{PageRequest, ProjectionStore, ProjectionStoreError};

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

pub fn check_range_predicate_is_kind_scoped<F: ProjectionFixture>(
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
        Err(PredicateError::MixedValueKinds) => {}
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

pub fn check_range_kind_mismatch_is_refused<F: ProjectionFixture>(
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
        Err(ProjectionStoreError::KindMismatch { property }) if property == "celsius" => {}
        other => {
            return Err(fail(
                "a kind-mismatched stored value is loud, never silent false",
                format!("{other:?}"),
            ));
        }
    }
    for (ordinal, (object_ref, celsius)) in [("ent_a2", 3), ("ent_a3", 5)].into_iter().enumerate() {
        store
            .apply(applied(
                "ten_a",
                ordinal as u64 + 2,
                vec![object(
                    "ten_a",
                    object_ref,
                    "ety_reading",
                    vec![("celsius", PropertyValue::Integer(celsius))],
                )],
            ))
            .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    }
    let after_bad = PageRequest::after(
        1,
        crate::store::ProjectionCursor {
            after_object_ref: "ent_a1".to_owned(),
        },
    );
    match store.filter("ten_a", "ety_reading", &predicate, &after_bad) {
        Err(ProjectionStoreError::KindMismatch { property }) if property == "celsius" => Ok(()),
        other => Err(fail(
            "kind drift refuses window-independently - a cursor never hides it",
            format!("{other:?}"),
        )),
    }
}

pub fn check_cross_kind_comparisons_fail_closed<F: ProjectionFixture>(
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
                vec![("celsius", PropertyValue::Boolean(true))],
            )],
        ))
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    match PropertyPredicate::range(
        "celsius",
        PropertyValue::Integer(1),
        PropertyValue::Boolean(true),
    ) {
        Err(PredicateError::MixedValueKinds) => {}
        other => {
            return Err(fail(
                "a cross-kind range never constructs, one storage class or not",
                format!("{other:?}"),
            ));
        }
    }
    match PropertyPredicate::range(
        "celsius",
        PropertyValue::Array(vec![]),
        PropertyValue::Array(vec![PropertyValue::Integer(1)]),
    ) {
        Err(PredicateError::UnrankedValueKind) => {}
        other => {
            return Err(fail(
                "an unrankable kind never constructs a range",
                format!("{other:?}"),
            ));
        }
    }
    let predicate = PropertyPredicate::range(
        "celsius",
        PropertyValue::Integer(1),
        PropertyValue::Integer(9),
    )
    .map_err(|error| fail("range constructs", format!("{error:?}")))?;
    match store.filter("ten_a", "ety_reading", &predicate, &PageRequest::first(10)) {
        Err(ProjectionStoreError::KindMismatch { property }) if property == "celsius" => {}
        other => {
            return Err(fail(
                "an Integer range over a Boolean-valued property refuses loudly",
                format!("{other:?}"),
            ));
        }
    }
    let equals = PropertyPredicate::equals("celsius", PropertyValue::Integer(1))
        .map_err(|error| fail("equals constructs", format!("{error:?}")))?;
    let page = store
        .filter("ten_a", "ety_reading", &equals, &PageRequest::first(10))
        .map_err(|error| fail("cross-kind equals reads", format!("{error:?}")))?;
    if !page.objects.is_empty() {
        return Err(fail(
            "Boolean(true) never equals Integer(1)",
            format!("{page:?}"),
        ));
    }
    Ok(())
}
