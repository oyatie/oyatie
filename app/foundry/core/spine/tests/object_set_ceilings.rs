//! Both sides of each ceiling a set materialization holds: the membership it
//! may hold, and the leaves its definition may read. A set at either ceiling
//! is served, so neither refusal can be the route being closed.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod object_set_support;

use data_ontology_kernel::PropertyValue;
use foundry_projection_draft::{PageRequest, PropertyPredicate};
use foundry_spine::{MAX_SET_LEAVES, MAX_SET_MEMBERS, PageError, SetDefinition, SetError};
use object_set_support::{crowded, members, named, page_of, three_tenants};

/// A union of `leaves` empty named sets. The leaves hold nothing, so only
/// their count can decide the answer.
fn chain(leaves: usize) -> SetDefinition {
    (1..leaves).fold(named(&[]), |left, _| {
        SetDefinition::Union(Box::new(left), Box::new(named(&[])))
    })
}

/// A membership at the ceiling is served and one past it is refused. The
/// requested page is far smaller than either membership, so a ceiling applied
/// to the page rather than to the membership would serve both.
#[test]
fn a_membership_at_the_ceiling_is_served_and_one_past_it_is_refused() {
    assert_eq!(
        page_of(
            &crowded(MAX_SET_MEMBERS),
            SetDefinition::Every,
            1,
            &PageRequest::first(2),
        )
        .unwrap(),
        ["ent_00000", "ent_00001", "next:ent_00001"],
    );
    assert_eq!(
        page_of(
            &crowded(MAX_SET_MEMBERS + 1),
            SetDefinition::Every,
            1,
            &PageRequest::first(2),
        ),
        Err(SetError::TooManyMembers),
    );
}

/// A definition at the leaf ceiling is served and one past it is refused.
#[test]
fn a_definition_at_the_leaf_ceiling_is_served_and_one_past_it_is_refused() {
    assert_eq!(members(chain(MAX_SET_LEAVES)), Vec::<String>::new());
    assert_eq!(
        page_of(
            &three_tenants(),
            chain(MAX_SET_LEAVES + 1),
            1,
            &PageRequest::first(MAX_SET_MEMBERS),
        ),
        Err(SetError::TooManyLeaves),
    );
}

/// Both sides of the ceiling on a NAMED leaf, which reads no page and so is
/// bounded on its own. It bounds the DISTINCT NAMES asked for, not the
/// membership they yield: a thousand and one names of objects no tenant holds
/// is refused though the membership would be empty, while a thousand copies of
/// one name is one read and is served.
#[test]
fn a_named_leaf_at_the_ceiling_is_served_and_one_past_it_is_refused() {
    let store = three_tenants();
    let page = PageRequest::first(MAX_SET_MEMBERS);
    let distinct = |count: usize| {
        let refs: Vec<String> = (0..count).map(|index| format!("ent_{index:05}")).collect();
        SetDefinition::Named(refs)
    };
    assert_eq!(
        page_of(&store, distinct(MAX_SET_MEMBERS), 1, &page).unwrap(),
        Vec::<String>::new(),
        "a thousand names of objects no tenant holds is a served, empty page",
    );
    assert_eq!(
        page_of(&store, distinct(MAX_SET_MEMBERS + 1), 1, &page),
        Err(SetError::TooManyMembers),
    );
    let repeated = SetDefinition::Named(vec!["ent_mid_a".to_owned(); MAX_SET_MEMBERS + 1]);
    assert_eq!(
        page_of(&store, repeated, 1, &page).unwrap(),
        ["ent_mid_a"],
        "the ceiling counts the names asked for, and a repeated ref is one read",
    );
}

/// A page limit of zero is refused rather than answered: an empty page with no
/// cursor would say the set is exhausted when it is not. Both sides, over a set
/// whose membership is not empty.
#[test]
fn a_zero_page_limit_is_refused_and_one_is_served() {
    let store = three_tenants();
    assert_eq!(
        page_of(&store, SetDefinition::Every, 1, &PageRequest::first(0)),
        Err(SetError::UnusablePage),
    );
    assert_eq!(
        page_of(&store, SetDefinition::Every, 1, &PageRequest::first(1)).unwrap(),
        ["ent_mid_a", "next:ent_mid_a"],
    );
}

/// A named leaf resolves the pin too, so a revision the tenant never accepted
/// is refused before any object is read.
#[test]
fn a_named_leaf_refuses_a_revision_the_tenant_never_accepted() {
    assert_eq!(
        page_of(
            &three_tenants(),
            named(&["ent_mid_a"]),
            9,
            &PageRequest::first(MAX_SET_MEMBERS),
        ),
        Err(SetError::Leaf(PageError::UnretainedRevision)),
    );
}

/// The leaf ceiling over leaves that READ the store, which is the work it
/// exists to bound. The empty named leaves above cost nothing, so a ceiling
/// spent only on them would bound nothing that matters.
#[test]
fn the_leaf_ceiling_counts_leaves_that_read_the_store() {
    let store = three_tenants();
    let page = PageRequest::first(MAX_SET_MEMBERS);
    let reading_chain = |leaves: usize| {
        (1..leaves).fold(SetDefinition::Every, |left, _| {
            SetDefinition::Union(Box::new(left), Box::new(SetDefinition::Every))
        })
    };
    assert_eq!(
        page_of(&store, reading_chain(MAX_SET_LEAVES), 1, &page).unwrap(),
        ["ent_mid_a", "ent_mid_b"],
        "sixteen reading leaves are served, and a union of a type with itself is that type",
    );
    assert_eq!(
        page_of(&store, reading_chain(MAX_SET_LEAVES + 1), 1, &page),
        Err(SetError::TooManyLeaves),
    );
}

/// A union is bounded on what it composes, not only on what each operand
/// holds: two predicates that each take a legal part of the type compose a
/// membership past the ceiling. The type itself is past it too, so `Every` over
/// this store is refused at its own leaf — which is why the operands are
/// predicates and not `Every`.
#[test]
fn a_union_past_the_member_ceiling_is_refused_though_each_operand_is_within_it() {
    let store = crowded(MAX_SET_MEMBERS + 200);
    let page = PageRequest::first(MAX_SET_MEMBERS);
    let half = |value: &str| {
        Box::new(SetDefinition::Matching(
            PropertyPredicate::equals("name", PropertyValue::String(value.to_owned()))
                .expect("a well-formed equality"),
        ))
    };
    assert_eq!(
        page_of(&store, *half("Ada"), 1, &page).unwrap().len(),
        (MAX_SET_MEMBERS + 200) / 2,
        "each half is a legal membership on its own",
    );
    assert_eq!(
        page_of(
            &store,
            SetDefinition::Union(half("Ada"), half("Grace")),
            1,
            &page,
        ),
        Err(SetError::TooManyMembers),
    );

    // AT the ceiling, so the union site's comparator is bracketed the way the
    // leaf site's is: a composition of exactly the ceiling is served.
    let exact = crowded(MAX_SET_MEMBERS);
    assert_eq!(
        page_of(
            &exact,
            SetDefinition::Union(half("Ada"), half("Grace")),
            1,
            &PageRequest::first(2),
        )
        .unwrap(),
        ["ent_00000", "ent_00001", "next:ent_00001"],
    );
}

/// The published values of both ceilings. Every assertion above is written
/// relative to its constant, so without this the numbers a caller is told are
/// unasserted and a change to either would pass the suite.
#[test]
fn the_ceilings_are_the_values_the_contract_publishes() {
    assert_eq!(MAX_SET_MEMBERS, 1_000);
    assert_eq!(MAX_SET_LEAVES, 16);
}
