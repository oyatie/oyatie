//! Reset laws: discard one tenant's projection so a rebuild can start
//! from empty — the remedy every other refusal here names and the port
//! could not perform. It must discard EVERYTHING for that tenant, or it
//! produces the mixture it was called to escape, and touch NOBODY else,
//! because too wide a blast radius cannot be undone.

use crate::conformance::{ProjectionFixture, applied, applied_with_links, fail, object};
use crate::keys::KeyDesignations;
use data_ontology_kernel::PropertyValue;

use crate::predicate::PropertyPredicate;
use crate::store::{
    EntryOutcome, PageRequest, ProjectedLink, ProjectionStore, ProjectionStoreError,
};

fn edge(from: &str, to: &str) -> ProjectedLink {
    ProjectedLink {
        link_type: "lty_measures".to_owned(),
        from_object_ref: from.to_owned(),
        to_object_ref: to.to_owned(),
        observed_at_epoch_ms: 1_700_000_000_000,
    }
}

/// Objects, edges, the poison ledger and the head all go. A reset that
/// left any of them would rebuild a projection that still disagrees with
/// its log, in a way the head no longer reveals.
pub fn check_reset_discards_everything_for_the_tenant<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(
            applied_with_links(
                "ten_a",
                1,
                vec![
                    object("ten_a", "ent_a1", "ety_reading", vec![]),
                    object("ten_a", "ent_a2", "ety_reading", vec![]),
                ],
                vec![edge("ent_a1", "ent_a2")],
            ),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    store
        .apply(
            crate::store::AppliedEntry {
                tenant_id: "ten_a".to_owned(),
                ordinal: 2,
                outcome: EntryOutcome::Poisoned {
                    reason: "payload_decode".to_owned(),
                },
            },
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("seed poison", format!("{error:?}")))?;

    let discarded = store
        .reset_tenant("ten_a")
        .map_err(|error| fail("reset runs", format!("{error:?}")))?;

    if discarded != 2 {
        return Err(fail(
            "reset returns the discarded head",
            format!("{discarded}"),
        ));
    }
    let head = store
        .applied_head("ten_a")
        .map_err(|error| fail("head reads", format!("{error:?}")))?;
    if head != 0 {
        return Err(fail("the head is back to zero", format!("{head}")));
    }
    let object_read = store
        .get("ten_a", "ent_a1")
        .map_err(|error| fail("get reads", format!("{error:?}")))?;
    if object_read.is_some() {
        return Err(fail("objects are gone", format!("{object_read:?}")));
    }
    let scan = store
        .objects_of_type("ten_a", "ety_reading", &PageRequest::first(50))
        .map_err(|error| fail("scan reads", format!("{error:?}")))?;
    if !scan.objects.is_empty() {
        return Err(fail("the type scan is empty", format!("{scan:?}")));
    }
    let outbound = store
        .links_from("ten_a", "ent_a1")
        .map_err(|error| fail("links read", format!("{error:?}")))?;
    let inbound = store
        .links_to("ten_a", "ent_a2")
        .map_err(|error| fail("inbound links read", format!("{error:?}")))?;
    if !outbound.is_empty() || !inbound.is_empty() {
        return Err(fail(
            "edges are gone in both directions",
            format!("{outbound:?} {inbound:?}"),
        ));
    }
    // The property index is queried ALONE by the kind-drift probe, so a
    // surviving row there is observable while every other assertion in
    // this check reads a different table. A range in a kind the
    // discarded value never had is empty on a clean store and a loud
    // `KindMismatch` if the old row is still indexed.
    let drifted = store.filter(
        "ten_a",
        "ety_reading",
        &PropertyPredicate::range("name", PropertyValue::Integer(0), PropertyValue::Integer(9))
            .expect("a well-formed range"),
        &PageRequest::first(50),
    );
    if !matches!(&drifted, Ok(page) if page.objects.is_empty()) {
        return Err(fail("the property index is gone", format!("{drifted:?}")));
    }
    let poisons = store
        .poisoned("ten_a")
        .map_err(|error| fail("poisons read", format!("{error:?}")))?;
    if !poisons.is_empty() {
        return Err(fail("the poison ledger is gone", format!("{poisons:?}")));
    }
    Ok(())
}

/// The blast radius is exactly one tenant. The neighbour's id has
/// `ten_a` as a PREFIX: a discard written with `starts_with` rather
/// than equality passes every test whose tenants share none, while
/// destroying `ten_ab` and `ten_alpha`.
pub fn check_reset_leaves_other_tenants_untouched<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    for tenant in ["ten_a", "ten_ab"] {
        store
            .apply(
                applied_with_links(
                    tenant,
                    1,
                    vec![
                        object(tenant, "ent_1", "ety_reading", vec![]),
                        object(tenant, "ent_2", "ety_reading", vec![]),
                    ],
                    vec![edge("ent_1", "ent_2")],
                ),
                &KeyDesignations::default(),
            )
            .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    }

    store
        .reset_tenant("ten_a")
        .map_err(|error| fail("reset runs", format!("{error:?}")))?;

    let head = store
        .applied_head("ten_ab")
        .map_err(|error| fail("neighbour head reads", format!("{error:?}")))?;
    if head != 1 {
        return Err(fail("the neighbour keeps its head", format!("{head}")));
    }
    let kept = store
        .get("ten_ab", "ent_1")
        .map_err(|error| fail("neighbour get reads", format!("{error:?}")))?;
    if kept.is_none() {
        return Err(fail("the neighbour keeps its objects", "absent".to_owned()));
    }
    let edges = store
        .links_from("ten_ab", "ent_1")
        .map_err(|error| fail("neighbour links read", format!("{error:?}")))?;
    if edges.len() != 1 {
        return Err(fail("the neighbour keeps its edges", format!("{edges:?}")));
    }
    Ok(())
}

/// Resetting a tenant the store never held is a no-op that reports zero,
/// not an error. A rebuild must not have to know whether the store was
/// already empty.
pub fn check_resetting_an_unknown_tenant_discards_nothing<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    let discarded = store
        .reset_tenant("ten_never")
        .map_err(|error| fail("reset runs", format!("{error:?}")))?;
    if discarded != 0 {
        return Err(fail("nothing was discarded", format!("{discarded}")));
    }
    Ok(())
}

/// After a reset the dense-ordinal law starts again at 1. A store that
/// kept its old head would refuse the very rebuild it was reset for.
pub fn check_applies_restart_at_ordinal_one_after_reset<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    for ordinal in 1..=3 {
        store
            .apply(
                applied(
                    "ten_a",
                    ordinal,
                    vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
                ),
                &KeyDesignations::default(),
            )
            .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    }
    store
        .reset_tenant("ten_a")
        .map_err(|error| fail("reset runs", format!("{error:?}")))?;

    // Resuming where the old head was must be refused, or the reset left
    // the store expecting a continuation of a log it no longer holds.
    if store
        .apply(
            applied(
                "ten_a",
                4,
                vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
            ),
            &KeyDesignations::default(),
        )
        .is_ok()
    {
        return Err(fail(
            "the old head is not still expected",
            "ordinal 4 applied".to_owned(),
        ));
    }
    store
        .apply(
            applied(
                "ten_a",
                1,
                vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
            ),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("the rebuild starts at one", format!("{error:?}")))?;
    Ok(())
}

/// The discard is durable, or an operator's destruction returns on
/// restart. Vacuous pass for a volatile fixture, as elsewhere.
pub fn check_reset_survives_reopen<F: ProjectionFixture>(fixture: &mut F) -> Result<(), String> {
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
    store
        .reset_tenant("ten_a")
        .map_err(|error| fail("reset runs", format!("{error:?}")))?;
    if !fixture.reopen() {
        return Ok(());
    }
    let store = fixture.store();
    let head = store
        .applied_head("ten_a")
        .map_err(|error| fail("head reads after reopen", format!("{error:?}")))?;
    if head != 0 {
        return Err(fail("the discard is durable", format!("{head}")));
    }
    let read = store
        .get("ten_a", "ent_a1")
        .map_err(|error| fail("get reads after reopen", format!("{error:?}")))?;
    if read.is_some() {
        return Err(fail("the rows are durably gone", format!("{read:?}")));
    }
    Ok(())
}

/// Both planes refuse a blank or untrimmed tenant id, the SAME way. A
/// destructive operation one plane performs and the other declines is
/// worse than either alone: code developed against the reference reads
/// "nothing to discard" where production refuses.
pub fn check_reset_refuses_a_blank_tenant<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    for blank in ["", "   ", " ten_a", "ten_a "] {
        let refused = store.reset_tenant(blank);
        if !matches!(refused, Err(ProjectionStoreError::Entry { .. })) {
            return Err(fail("an untrimmed tenant is refused", format!("{blank:?}")));
        }
    }
    Ok(())
}
