//! Link laws: the projection's edges are durable state, not a detail of
//! whatever in-memory engine happened to build them.
//!
//! Without these, a projection rebuilt from the store comes back with
//! its objects and NONE of its edges — the traversal surface would
//! silently see an empty graph. Both planes are held to the same checks
//! so neither can drift on that.

use data_ontology_kernel::PropertyValue;

use crate::conformance::{ProjectionFixture, applied_with_links, fail, object};
use crate::keys::KeyDesignations;
use crate::store::{ProjectedLink, ProjectionStore, ProjectionStoreError};

fn link(from: &str, to: &str) -> ProjectedLink {
    observed(from, to, 1_700_000_000_000)
}

fn observed(from: &str, to: &str, at: u64) -> ProjectedLink {
    ProjectedLink {
        link_type: "lty_measures".to_owned(),
        from_object_ref: from.to_owned(),
        to_object_ref: to.to_owned(),
        observed_at_epoch_ms: at,
    }
}

fn named(object_ref: &str, name: &str) -> Vec<(&'static str, PropertyValue)> {
    let _ = object_ref;
    vec![("name", PropertyValue::String(name.to_owned()))]
}

pub fn check_links_round_trip_in_both_directions<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    store
        .apply(
            applied_with_links(
                "ten_a",
                1,
                vec![
                    object("ten_a", "ent_a1", "ety_reading", named("ent_a1", "Ada")),
                    object("ten_a", "ent_a2", "ety_reading", named("ent_a2", "Grace")),
                ],
                vec![link("ent_a1", "ent_a2")],
            ),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("an entry carrying a link applies", format!("{error:?}")))?;

    let outbound = store
        .links_from("ten_a", "ent_a1")
        .map_err(|error| fail("outbound reads", format!("{error:?}")))?;
    if outbound != vec![link("ent_a1", "ent_a2")] {
        return Err(fail(
            "the edge is readable from its source",
            format!("{outbound:?}"),
        ));
    }

    let inbound = store
        .links_to("ten_a", "ent_a2")
        .map_err(|error| fail("inbound reads", format!("{error:?}")))?;
    if inbound != vec![link("ent_a1", "ent_a2")] {
        return Err(fail(
            "and from its target — traversal needs both directions",
            format!("{inbound:?}"),
        ));
    }

    let none = store
        .links_from("ten_a", "ent_a2")
        .map_err(|error| fail("outbound reads", format!("{error:?}")))?;
    if !none.is_empty() {
        return Err(fail("direction is not symmetric", format!("{none:?}")));
    }
    Ok(())
}

pub fn check_links_are_tenant_isolated<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    for tenant in ["ten_a", "ten_b"] {
        store
            .apply(
                applied_with_links(
                    tenant,
                    1,
                    vec![
                        object(tenant, "ent_x1", "ety_reading", named("ent_x1", "Ada")),
                        object(tenant, "ent_x2", "ety_reading", named("ent_x2", "Grace")),
                    ],
                    vec![link("ent_x1", "ent_x2")],
                ),
                &KeyDesignations::default(),
            )
            .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    }
    let a = store
        .links_from("ten_a", "ent_x1")
        .map_err(|error| fail("reads", format!("{error:?}")))?;
    if a.len() != 1 {
        return Err(fail(
            "each tenant sees exactly its own edge",
            format!("{a:?}"),
        ));
    }
    Ok(())
}

pub fn check_a_refused_apply_writes_no_links<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    // Ordinal 2 with nothing at 1: refused as non-dense, so its links
    // must not survive either.
    let refused = store.apply(
        applied_with_links(
            "ten_a",
            2,
            vec![
                object("ten_a", "ent_a1", "ety_reading", named("ent_a1", "Ada")),
                object("ten_a", "ent_a2", "ety_reading", named("ent_a2", "Grace")),
            ],
            vec![link("ent_a1", "ent_a2")],
        ),
        &KeyDesignations::default(),
    );
    if refused.is_ok() {
        return Err(fail(
            "the non-dense entry was refused",
            format!("{refused:?}"),
        ));
    }
    let leaked = store
        .links_from("ten_a", "ent_a1")
        .map_err(|error| fail("reads", format!("{error:?}")))?;
    if !leaked.is_empty() {
        return Err(fail(
            "a refused apply leaves no edge behind",
            format!("{leaked:?}"),
        ));
    }
    Ok(())
}

pub fn check_re_applying_an_entry_does_not_duplicate_links<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    let entry = applied_with_links(
        "ten_a",
        1,
        vec![
            object("ten_a", "ent_a1", "ety_reading", named("ent_a1", "Ada")),
            object("ten_a", "ent_a2", "ety_reading", named("ent_a2", "Grace")),
        ],
        vec![link("ent_a1", "ent_a2")],
    );
    store
        .apply(entry.clone(), &KeyDesignations::default())
        .map_err(|error| fail("first apply", format!("{error:?}")))?;
    store
        .apply(entry, &KeyDesignations::default())
        .map_err(|error| fail("byte-identical re-apply dedups", format!("{error:?}")))?;
    let edges = store
        .links_from("ten_a", "ent_a1")
        .map_err(|error| fail("reads", format!("{error:?}")))?;
    if edges.len() != 1 {
        return Err(fail(
            "a deduplicated re-apply does not double the edge",
            format!("{edges:?}"),
        ));
    }
    Ok(())
}

pub fn check_a_later_observation_updates_the_edge<F: ProjectionFixture>(
    fixture: &mut F,
) -> Result<(), String> {
    let store = fixture.store();
    let objects = || {
        vec![
            object("ten_a", "ent_a1", "ety_reading", named("ent_a1", "Ada")),
            object("ten_a", "ent_a2", "ety_reading", named("ent_a2", "Grace")),
        ]
    };
    for (ordinal, at) in [(1_u64, 1_700_000_000_000_u64), (2, 1_700_000_009_000)] {
        store
            .apply(
                applied_with_links(
                    "ten_a",
                    ordinal,
                    objects(),
                    vec![observed("ent_a1", "ent_a2", at)],
                ),
                &KeyDesignations::default(),
            )
            .map_err(|error| fail("both sightings apply", format!("{error:?}")))?;
    }
    let edges = store
        .links_from("ten_a", "ent_a1")
        .map_err(|error| fail("reads", format!("{error:?}")))?;
    if edges.len() != 1 {
        return Err(fail(
            "identity is (tenant, from, type, to) — a second sighting is the SAME edge",
            format!("{edges:?}"),
        ));
    }
    if edges[0].observed_at_epoch_ms != 1_700_000_009_000 {
        return Err(fail(
            "and the later observation wins, so a freshness floor sees the truth",
            format!("{:?}", edges[0]),
        ));
    }
    Ok(())
}
