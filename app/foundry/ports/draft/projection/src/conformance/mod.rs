//! The executable contract: check functions any [`ProjectionStore`]
//! implementation must pass, mirroring the records-port suite idiom.

mod keys;
mod predicates;
mod reads;

pub use keys::{
    check_a_composite_key_value_is_refused, check_a_duplicate_primary_key_is_refused,
    check_a_missing_key_property_is_refused, check_an_object_may_keep_its_own_key,
    check_an_undeclared_key_constrains_nothing,
    check_keys_are_scoped_to_their_entity_type_and_tenant,
    check_two_objects_in_one_entry_cannot_share_a_key,
};
pub use predicates::{
    check_cross_kind_comparisons_fail_closed, check_equals_predicate_matches_exactly,
    check_range_kind_mismatch_is_refused, check_range_predicate_is_kind_scoped,
};
pub use reads::{
    check_get_returns_the_projected_object, check_reads_are_tenant_isolated,
    check_type_scan_pages_partition_deterministically,
};

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{ObjectEntity, ObjectProperty, PropertyValue};

use crate::keys::KeyDesignations;
use crate::store::{
    AppliedEntry, EntryOutcome, ProjectedObject, ProjectionStore, ProjectionStoreError,
};

/// A store under test plus its lifecycle.
pub trait ProjectionFixture {
    type Store: ProjectionStore;

    fn store(&mut self) -> &mut Self::Store;

    /// Close and reopen the underlying store; `false` means the fixture
    /// is volatile and durability cannot be checked against it.
    fn reopen(&mut self) -> bool;
}

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly)
        .expect("conformance fixtures use a privacy data class")
}

pub(crate) fn object(
    tenant: &str,
    object_ref: &str,
    entity_type: &str,
    properties: Vec<(&str, PropertyValue)>,
) -> ProjectedObject {
    // The kernel refuses property-less entities; fixtures that do not
    // care about properties still carry one.
    let properties = if properties.is_empty() {
        vec![("name", PropertyValue::String("Ada".to_owned()))]
    } else {
        properties
    };
    let properties = properties
        .into_iter()
        .map(|(name, value)| ObjectProperty::typed(name.to_owned(), value, internal()))
        .collect();
    ProjectedObject {
        entity: ObjectEntity::new(
            tenant.to_owned(),
            object_ref.to_owned(),
            entity_type.to_owned(),
            properties,
        )
        .expect("conformance fixtures construct valid entities"),
        schema_revision: 1,
        last_ordinal: 1,
        last_actor: "prn_projector".to_owned(),
    }
}

pub(crate) fn applied(tenant: &str, ordinal: u64, objects: Vec<ProjectedObject>) -> AppliedEntry {
    AppliedEntry {
        tenant_id: tenant.to_owned(),
        ordinal,
        outcome: EntryOutcome::Applied { objects },
    }
}

fn fail(clause: &str, detail: String) -> String {
    format!("{clause}: {detail}")
}

pub fn check_apply_requires_the_next_dense_ordinal<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    let first = object("ten_a", "ent_a1", "ety_reading", vec![]);
    store
        .apply(
            applied("ten_a", 1, vec![first]),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("first dense apply is accepted", format!("{error:?}")))?;
    match store.apply(applied("ten_a", 3, vec![]), &KeyDesignations::default()) {
        Err(ProjectionStoreError::NonDenseOrdinal {
            expected: 2,
            found: 3,
        }) => {}
        other => return Err(fail("a skipped ordinal is refused", format!("{other:?}"))),
    }
    let head = store
        .applied_head("ten_a")
        .map_err(|error| fail("head reads", format!("{error:?}")))?;
    if head != 1 {
        return Err(fail("the head never skips", format!("head={head}")));
    }
    Ok(())
}

pub fn check_identical_reapply_is_a_deduplicated_noop<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    let entry = applied(
        "ten_a",
        1,
        vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
    );
    store
        .apply(entry.clone(), &KeyDesignations::default())
        .map_err(|error| fail("first apply", format!("{error:?}")))?;
    let receipt = store
        .apply(entry, &KeyDesignations::default())
        .map_err(|error| fail("byte-identical re-apply is a no-op", format!("{error:?}")))?;
    if !receipt.deduplicated {
        return Err(fail(
            "the re-apply receipt says deduplicated",
            format!("{receipt:?}"),
        ));
    }
    Ok(())
}

pub fn check_divergent_reapply_is_refused<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(
            applied(
                "ten_a",
                1,
                vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
            ),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("first apply", format!("{error:?}")))?;
    let mut divergent = object("ten_a", "ent_a1", "ety_reading", vec![]);
    divergent.last_actor = "prn_forger".to_owned();
    match store.apply(
        applied("ten_a", 1, vec![divergent]),
        &KeyDesignations::default(),
    ) {
        Err(ProjectionStoreError::DivergentReplay { ordinal: 1 }) => Ok(()),
        other => Err(fail(
            "divergent content at an applied ordinal is loud",
            format!("{other:?}"),
        )),
    }
}

pub fn check_a_refused_apply_leaves_state_untouched<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(
            applied(
                "ten_a",
                1,
                vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
            ),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    let refused = store.apply(
        applied(
            "ten_a",
            3,
            vec![object("ten_a", "ent_a3", "ety_reading", vec![])],
        ),
        &KeyDesignations::default(),
    );
    if refused.is_ok() {
        return Err(fail("the skip was refused", format!("{refused:?}")));
    }
    let leaked = store
        .get("ten_a", "ent_a3")
        .map_err(|error| fail("get reads", format!("{error:?}")))?;
    if leaked.is_some() {
        return Err(fail(
            "a refused apply writes nothing",
            format!("{leaked:?}"),
        ));
    }
    Ok(())
}

pub fn check_poisoned_entries_advance_the_head_without_objects<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(
            AppliedEntry {
                tenant_id: "ten_a".to_owned(),
                ordinal: 1,
                outcome: EntryOutcome::Poisoned {
                    reason: "receipt_mismatch".to_owned(),
                },
            },
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("a poison mirror applies", format!("{error:?}")))?;
    let head = store
        .applied_head("ten_a")
        .map_err(|error| fail("head reads", format!("{error:?}")))?;
    if head != 1 {
        return Err(fail("the poisoned ordinal was spent", format!("{head}")));
    }
    let poisons = store
        .poisoned("ten_a")
        .map_err(|error| fail("poisons read", format!("{error:?}")))?;
    if poisons != vec![(1, "receipt_mismatch".to_owned())] {
        return Err(fail("nothing hidden", format!("{poisons:?}")));
    }
    Ok(())
}

pub fn check_durability_across_reopen<F: ProjectionFixture>(fixture: &mut F) -> Result<(), String> {
    let entry = applied(
        "ten_a",
        1,
        vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
    );
    fixture
        .store()
        .apply(entry, &KeyDesignations::default())
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    if !fixture.reopen() {
        return Ok(());
    }
    let head = fixture
        .store()
        .applied_head("ten_a")
        .map_err(|error| fail("head survives reopen", format!("{error:?}")))?;
    if head != 1 {
        return Err(fail("the head survives reopen", format!("{head}")));
    }
    let stored = fixture
        .store()
        .get("ten_a", "ent_a1")
        .map_err(|error| fail("get survives reopen", format!("{error:?}")))?;
    if stored.is_none() {
        return Err(fail("the object survives reopen", "None".to_owned()));
    }
    Ok(())
}
