//! What an object set holds: the reader's own objects of the set's own type,
//! composed by the set operations, paged on the store's own keyset, and
//! filtered only on properties the reader's pin declares.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod object_set_support;

use data_ontology_kernel::PropertyValue;
use foundry_projection_draft::{PageRequest, ProjectionCursor, PropertyPredicate};
use foundry_spine::{MAX_SET_MEMBERS, PageError, SetDefinition, SetError};
use object_set_support::{members, named, neighbours_only, page_of, rows_of, three_tenants};

fn matching(property: &str, value: &str) -> SetDefinition {
    SetDefinition::Matching(
        PropertyPredicate::equals(property, PropertyValue::String(value.to_owned()))
            .expect("a well-formed equality"),
    )
}

/// A named set holds the reader's own objects of the set's type and nothing
/// else: a neighbour's ref, a ref of another type, and a ref no tenant holds
/// are each absent, and none of the three is a refusal. The unfiltered set is
/// the control on what the reader had to name.
#[test]
fn a_named_set_excludes_the_refs_it_cannot_own() {
    assert_eq!(
        members(named(&[
            "ent_mid_a",
            "ent_first_a",
            "ent_last_b",
            "ent_mid_other",
            "ent_nowhere",
        ])),
        ["ent_mid_a"],
    );
    assert_eq!(members(SetDefinition::Every), ["ent_mid_a", "ent_mid_b"]);
    assert_eq!(members(named(&[])), Vec::<String>::new());
}

/// Each operation over the same two memberships. `Subtract` is asked in both
/// directions, so the answer cannot come from an operand's size alone.
#[test]
fn the_operations_compose_over_one_membership() {
    let ada = || Box::new(matching("name", "Ada"));
    let three = || Box::new(named(&["ent_mid_a", "ent_mid_b", "ent_first_a"]));
    assert_eq!(
        members(SetDefinition::Union(Box::new(named(&["ent_mid_b"])), ada())),
        ["ent_mid_a", "ent_mid_b"],
    );
    assert_eq!(
        members(SetDefinition::Intersect(three(), ada())),
        ["ent_mid_a"],
    );
    assert_eq!(
        members(SetDefinition::Subtract(three(), ada())),
        ["ent_mid_b"],
    );
    assert_eq!(
        members(SetDefinition::Subtract(ada(), three())),
        Vec::<String>::new(),
    );
}

/// The pin guard holds per leaf: `grade` is stored on every object and
/// declared only at revision 2, so a filter leaf on it — inside a union whose
/// other operand is materializable at either pin — is refused at revision 1
/// and served at 2.
#[test]
fn a_filter_leaf_is_refused_on_a_property_the_pin_does_not_declare() {
    let union = || {
        SetDefinition::Union(
            Box::new(named(&["ent_mid_a"])),
            Box::new(matching("grade", "A")),
        )
    };
    let store = three_tenants();
    let page = PageRequest::first(MAX_SET_MEMBERS);
    assert_eq!(
        page_of(&store, union(), 1, &page),
        Err(SetError::Leaf(PageError::UndeclaredFilterProperty)),
    );
    assert_eq!(
        page_of(&store, union(), 2, &page).unwrap(),
        ["ent_mid_a", "ent_mid_b"],
        "revision 2 declares grade, so the same leaf is answerable",
    );
}

/// A set pages on the store's own keyset: `next` names the last row of a page
/// that has members behind it, and is absent from the page that has none.
#[test]
fn a_set_pages_like_a_listing_of_the_same_rows() {
    let store = three_tenants();
    assert_eq!(
        page_of(&store, SetDefinition::Every, 1, &PageRequest::first(1)).unwrap(),
        ["ent_mid_a", "next:ent_mid_a"],
    );
    let resumed = PageRequest::after(
        1,
        ProjectionCursor {
            after_object_ref: "ent_mid_a".to_owned(),
        },
    );
    assert_eq!(
        page_of(&store, SetDefinition::Every, 1, &resumed).unwrap(),
        ["ent_mid_b"],
    );
}

/// A named leaf's rows are the pinned reader's, not the stored object: behind
/// the pin a row carries only what the pin declares and reports the revision it
/// was written at, and at the pin it carries the property the pin adds. The
/// same rendering the listing's own page test uses, over the same helper both
/// build rows with.
#[test]
fn a_named_set_reads_its_rows_through_the_pin() {
    let store = three_tenants();
    let refs = || named(&["ent_mid_a", "ent_mid_b"]);
    assert_eq!(
        rows_of(&store, refs(), 1).unwrap(),
        ["ent_mid_a name r1 Current", "ent_mid_b name r1 Current"],
        "revision 1 declares name only, and both objects were written at it",
    );
    assert_eq!(
        rows_of(&store, refs(), 2).unwrap(),
        [
            "ent_mid_a grade,name r1 UpcastPending",
            "ent_mid_b grade,name r1 UpcastPending"
        ],
        "revision 2 declares grade, and both writes are behind it",
    );
}

/// The null condition: the reader holds nothing and both neighbours hold
/// objects under the refs the reader would have used. Every variant of a
/// definition serves an empty page, and none of them is a refusal.
#[test]
fn a_reader_holding_nothing_is_served_nothing_of_its_neighbours() {
    let store = neighbours_only();
    for definition in [
        SetDefinition::Every,
        named(&["ent_mid_a", "ent_mid_b", "ent_first_a", "ent_last_a"]),
        matching("name", "Ada"),
        SetDefinition::Union(
            Box::new(SetDefinition::Every),
            Box::new(named(&["ent_mid_a"])),
        ),
        SetDefinition::Intersect(
            Box::new(SetDefinition::Every),
            Box::new(matching("name", "Ada")),
        ),
        SetDefinition::Subtract(
            Box::new(SetDefinition::Every),
            Box::new(named(&["ent_mid_a"])),
        ),
    ] {
        assert_eq!(
            page_of(&store, definition, 1, &PageRequest::first(MAX_SET_MEMBERS)).unwrap(),
            Vec::<String>::new(),
        );
    }
}
