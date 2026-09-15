//! Object sets: one membership over one entity type, as a tenant pinned at
//! one revision sees it.
//!
//! A set is a tree whose leaves are reads of the projection and whose nodes
//! are set operations. Every leaf that filters is read through
//! [`objects_of_type_at_revision`], so a set answers a filter under the same
//! pin guard a listing does.
//!
//! A set has one entity type, so the pin resolves once and no operand can
//! bring a row of another type into the result. Membership is materialized
//! whole and then paged, so both ceilings below bound one request.

use std::collections::{BTreeMap, BTreeSet};

use data_ontology_kernel::{EntityTypeId, OntologyEngine};
use foundry_projection_draft::{PageRequest, ProjectionCursor, ProjectionStore, PropertyPredicate};

use crate::revision::{PageError, PinnedObject, PinnedPage, objects_of_type_at_revision};

/// The most rows any STEP of a definition may hold: what a leaf reads, what a
/// named leaf asks for, and what a union composes. Refused, never truncated —
/// and refused on the step, so a definition whose final membership would be
/// smaller is still refused when a step of it is not.
pub const MAX_SET_MEMBERS: usize = 1_000;

/// The most leaves one definition may read, over the whole tree rather than
/// any one branch.
pub const MAX_SET_LEAVES: usize = 16;

/// How a set's membership is decided. Every variant ranges over the set's
/// own entity type, in the reading tenant, at the reader's pin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetDefinition {
    /// Every object of the type.
    Every,
    /// Exactly the objects named. A name that the tenant's projection does
    /// not hold, or holds at another entity type, is not a member; it is
    /// not a refusal either.
    Named(Vec<String>), // data_class: INTERNAL_ONLY
    /// Every object of the type the predicate matches. Refused unless the
    /// pinned definition declares the property it names.
    Matching(PropertyPredicate), // data_class: PII_IDENTIFYING
    /// The members of either operand.
    Union(Box<SetDefinition>, Box<SetDefinition>),
    /// The members of both operands.
    Intersect(Box<SetDefinition>, Box<SetDefinition>),
    /// The left operand's members that the right operand does not hold.
    Subtract(Box<SetDefinition>, Box<SetDefinition>),
}

/// A membership over one entity type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectSet {
    pub entity_type: EntityTypeId, // data_class: INTERNAL_ONLY
    pub definition: SetDefinition,
}

/// Typed refusals of a set materialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetError {
    /// A leaf read refused. A filter leaf carries the refusal its page gave;
    /// a named leaf reaches the pin and store arms only.
    Leaf(PageError),
    /// A step of the definition exceeds [`MAX_SET_MEMBERS`]: the rows a leaf
    /// accumulates over its pages, the distinct names a named leaf asks for, or
    /// a union's composition. The final membership may be smaller than the step
    /// that was refused.
    TooManyMembers,
    /// The definition reads more than [`MAX_SET_LEAVES`] leaves.
    TooManyLeaves,
    /// The page request's limit is zero. [`PageRequest`] requires a non-zero
    /// limit and the store refuses one, so a set refuses it too rather than
    /// answering an empty page no cursor can resume.
    UnusablePage,
}

/// Members in `object_ref` order, each as the pinned reader sees it.
type Members = BTreeMap<String, PinnedObject>;

/// One page of `set`'s membership, in `object_ref` order, each row as a
/// reader pinned at `pinned` sees it.
///
/// `page.cursor` is the keyset the store's own pages use, so a set pages like
/// a listing of the same rows, and `next` is present exactly when members
/// remain past the page. `page.limit` must be non-zero, as the store requires
/// of a listing; a zero limit is [`SetError::UnusablePage`] and no page.
pub fn materialize_object_set(
    store: &dyn ProjectionStore,
    registry: &OntologyEngine,
    tenant_id: &str,
    set: &ObjectSet,
    pinned: u32,
    page: &PageRequest,
) -> Result<PinnedPage, SetError> {
    if page.limit == 0 {
        return Err(SetError::UnusablePage);
    }
    let mut leaves = MAX_SET_LEAVES;
    let members = members(
        store,
        registry,
        tenant_id,
        &set.entity_type,
        pinned,
        &set.definition,
        &mut leaves,
    )?;
    Ok(window(members, page))
}

/// `definition`'s membership, whole. `leaves` is the definition's remaining
/// leaf budget, spent by the whole tree rather than by any one branch.
fn members(
    store: &dyn ProjectionStore,
    registry: &OntologyEngine,
    tenant_id: &str,
    entity_type: &EntityTypeId,
    pinned: u32,
    definition: &SetDefinition,
    leaves: &mut usize,
) -> Result<Members, SetError> {
    let operand = |definition, leaves: &mut usize| {
        members(
            store,
            registry,
            tenant_id,
            entity_type,
            pinned,
            definition,
            leaves,
        )
    };
    match definition {
        SetDefinition::Every => read(
            store,
            registry,
            tenant_id,
            entity_type,
            pinned,
            None,
            leaves,
        ),
        SetDefinition::Matching(filter) => read(
            store,
            registry,
            tenant_id,
            entity_type,
            pinned,
            Some(filter),
            leaves,
        ),
        SetDefinition::Named(object_refs) => named(
            store,
            registry,
            tenant_id,
            entity_type,
            pinned,
            object_refs,
            leaves,
        ),
        SetDefinition::Union(left, right) => {
            let mut members = operand(left, leaves)?;
            members.extend(operand(right, leaves)?);
            bounded(members)
        }
        SetDefinition::Intersect(left, right) => {
            let held = operand(right, leaves)?;
            let mut members = operand(left, leaves)?;
            members.retain(|object_ref, _| held.contains_key(object_ref));
            Ok(members)
        }
        SetDefinition::Subtract(left, right) => {
            let held = operand(right, leaves)?;
            let mut members = operand(left, leaves)?;
            members.retain(|object_ref, _| !held.contains_key(object_ref));
            Ok(members)
        }
    }
}

/// Every object of the type, or every one `filter` matches, read through the
/// pinned page view. Pages are followed while the store reports more; a store
/// that reported a cursor on an empty page would not terminate, which the
/// port's law forbids — `next` is present exactly when more remain.
fn read(
    store: &dyn ProjectionStore,
    registry: &OntologyEngine,
    tenant_id: &str,
    entity_type: &EntityTypeId,
    pinned: u32,
    filter: Option<&PropertyPredicate>,
    leaves: &mut usize,
) -> Result<Members, SetError> {
    spend(leaves)?;
    let mut members = Members::new();
    let mut request = PageRequest::first(MAX_SET_MEMBERS);
    loop {
        let page = objects_of_type_at_revision(
            store,
            registry,
            tenant_id,
            entity_type,
            pinned,
            &request,
            filter,
        )
        .map_err(SetError::Leaf)?;
        let next = page.next;
        members.extend(page.objects);
        members = bounded(members)?;
        match next {
            Some(cursor) => request = PageRequest::after(MAX_SET_MEMBERS, cursor),
            None => return Ok(members),
        }
    }
}

/// The named objects this tenant's projection holds at `entity_type`.
fn named(
    store: &dyn ProjectionStore,
    registry: &OntologyEngine,
    tenant_id: &str,
    entity_type: &EntityTypeId,
    pinned: u32,
    object_refs: &[String],
    leaves: &mut usize,
) -> Result<Members, SetError> {
    spend(leaves)?;
    // The names asked for, deduplicated: a thousand point reads is the cost
    // whatever the projection holds, so this bounds the request and not the
    // membership it yields. Naming one ref a thousand times is one read.
    let named: BTreeSet<&str> = object_refs.iter().map(String::as_str).collect();
    if named.len() > MAX_SET_MEMBERS {
        return Err(SetError::TooManyMembers);
    }
    let definition = registry
        .entity_type_at_revision(tenant_id, entity_type, pinned)
        .ok_or(SetError::Leaf(PageError::UnretainedRevision))?;
    let mut members = Members::new();
    for object_ref in named {
        let held = store
            .get(tenant_id, object_ref)
            .map_err(|error| SetError::Leaf(PageError::StoreUnreadable(error)))?;
        let Some(projected) = held else { continue };
        if projected.entity.entity_type.value != entity_type.value {
            continue;
        }
        let view = crate::revision::pinned_view(definition, &projected, pinned);
        members.insert(projected.entity.id, view);
    }
    Ok(members)
}

/// One leaf of the definition's budget.
fn spend(leaves: &mut usize) -> Result<(), SetError> {
    *leaves = leaves.checked_sub(1).ok_or(SetError::TooManyLeaves)?;
    Ok(())
}

/// `members` if it is within [`MAX_SET_MEMBERS`].
fn bounded(members: Members) -> Result<Members, SetError> {
    if members.len() > MAX_SET_MEMBERS {
        return Err(SetError::TooManyMembers);
    }
    Ok(members)
}

/// The requested page of an ordered membership.
fn window(members: Members, page: &PageRequest) -> PinnedPage {
    let after = page
        .cursor
        .as_ref()
        .map(|cursor| cursor.after_object_ref.as_str());
    let mut past_cursor = members
        .into_iter()
        .filter(|(object_ref, _)| after.is_none_or(|after| object_ref.as_str() > after));
    let objects: Vec<(String, PinnedObject)> = past_cursor.by_ref().take(page.limit).collect();
    let next = match (past_cursor.next(), objects.last()) {
        (Some(_), Some((last, _))) => Some(ProjectionCursor {
            after_object_ref: last.clone(),
        }),
        _ => None,
    };
    PinnedPage { objects, next }
}
