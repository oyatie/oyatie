//! Isolation laws: a discard destroys exactly one tenant, and refuses
//! an id it cannot trust.
//!
//! Separate from the discard laws because these are the ones a WRONG
//! implementation passes most easily. A reset written with `starts_with`
//! rather than equality satisfies every test whose tenants share no
//! prefix — so the neighbour here has the reset tenant's id as a prefix,
//! and every map the store keeps is read back, not just the two a read
//! path happens to expose.

use crate::conformance::{ProjectionFixture, applied, applied_with_links, edge, fail, object};
use crate::keys::KeyDesignations;
use crate::store::{ProjectionStore, ProjectionStoreError};

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
        store
            .apply(
                crate::store::AppliedEntry {
                    tenant_id: tenant.to_owned(),
                    ordinal: 2,
                    outcome: crate::store::EntryOutcome::Poisoned {
                        reason: "payload_decode".to_owned(),
                    },
                },
                &KeyDesignations::default(),
            )
            .map_err(|error| fail("seed poison", format!("{error:?}")))?;
    }

    store
        .reset_tenant("ten_a")
        .map_err(|error| fail("reset runs", format!("{error:?}")))?;

    let head = store
        .applied_head("ten_ab")
        .map_err(|error| fail("neighbour head reads", format!("{error:?}")))?;
    if head != 2 {
        return Err(fail("the neighbour keeps its head", format!("{head}")));
    }
    let kept = store
        .get("ten_ab", "ent_1")
        .map_err(|error| fail("neighbour get reads", format!("{error:?}")))?;
    if kept.is_none() {
        return Err(fail("the neighbour keeps its objects", "absent".to_owned()));
    }
    // Two of the store's maps are reachable through NO read path here:
    // the applied-entry map and the neighbour's poison ledger. A discard
    // that over-reached into either leaves every assertion above green.
    // The entry map is read by re-applying the neighbour's own ordinal:
    // a store that still holds it dedups, a store that lost it reports
    // divergence — which is what a projector redelivering after a
    // restart would be told about a tenant nothing touched.
    let receipt = store
        .apply(
            applied_with_links(
                "ten_ab",
                1,
                vec![
                    object("ten_ab", "ent_1", "ety_reading", vec![]),
                    object("ten_ab", "ent_2", "ety_reading", vec![]),
                ],
                vec![edge("ent_1", "ent_2")],
            ),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("the neighbour keeps its entries", format!("{error:?}")))?;
    if !receipt.deduplicated {
        return Err(fail(
            "the neighbour's entry is still there to dedup against",
            format!("{receipt:?}"),
        ));
    }
    let poisons = store
        .poisoned("ten_ab")
        .map_err(|error| fail("neighbour poisons read", format!("{error:?}")))?;
    if poisons.len() != 1 {
        return Err(fail(
            "the neighbour keeps its poison ledger",
            format!("{poisons:?}"),
        ));
    }
    let edges = store
        .links_from("ten_ab", "ent_1")
        .map_err(|error| fail("neighbour links read", format!("{error:?}")))?;
    if edges.len() != 1 {
        return Err(fail("the neighbour keeps its edges", format!("{edges:?}")));
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
