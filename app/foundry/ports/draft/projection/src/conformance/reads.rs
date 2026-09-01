//! Read laws of the conformance suite: get, tenant isolation, and the
//! typed-cursor pagination laws.

use data_ontology_kernel::PropertyValue;

use crate::conformance::{ProjectionFixture, applied, fail, object};
use crate::keys::KeyDesignations;
use crate::store::{PageRequest, ProjectionStore};

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
        .apply(
            applied("ten_a", 1, vec![stored.clone()]),
            &KeyDesignations::default(),
        )
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
        .apply(
            applied(
                "ten_a",
                1,
                vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
            ),
            &KeyDesignations::default(),
        )
        .map_err(|error| fail("tenant a seed", format!("{error:?}")))?;
    store
        .apply(
            applied(
                "ten_b",
                1,
                vec![object("ten_b", "ent_b1", "ety_reading", vec![])],
            ),
            &KeyDesignations::default(),
        )
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
            .apply(
                applied(
                    "ten_a",
                    ordinal as u64 + 1,
                    vec![object("ten_a", object_ref, entity_type, vec![])],
                ),
                &KeyDesignations::default(),
            )
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
