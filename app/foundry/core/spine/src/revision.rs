//! Reader pinning: one pure view over the projection and the kernel's
//! retained revision history. Behind the pin the view filters down to the
//! pinned vocabulary — lossless under additive-only evolution. Ahead of the
//! pin the view shows honest absence: a value the log never carried is
//! never synthesized at read. Refusals are typed; a read never touches the
//! poison ledger. With a matching plan, [`UpcastState`] is refined by the
//! SAME predicate the runner scans with; without one, written-below-pin is
//! pending.

use std::collections::{BTreeMap, BTreeSet};

use data_ontology_kernel::{EntityTypeId, ObjectProperty, OntologyEngine};
use foundry_projection_draft::{
    PageRequest, ProjectionCursor, ProjectionStore, ProjectionStoreError, PropertyPredicate,
};

use crate::migrate::MigrationPlan;
use crate::state::ProjectionState;

/// Where one object stands relative to a pinned revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpcastState {
    /// The object's last applied write is at or beyond the pin.
    Current,
    /// The object was last written below the pin; properties the pin
    /// declares beyond that write are honestly absent until a logged
    /// upcast lands.
    UpcastPending,
}

/// One object as a reader pinned at a revision sees it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedObject {
    /// The object's applied properties, filtered to the names the pinned
    /// definition declares. Every value is log-derived.
    pub properties: BTreeMap<String, ObjectProperty>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    pub written_revision: u32,     // data_class: INTERNAL_ONLY
    pub upcast_state: UpcastState, // data_class: INTERNAL_ONLY
}

/// Typed refusals of the pinned view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewError {
    /// The projection this view reads holds no object at this `object_ref`
    /// for this tenant. Over the durable store that means the store holds
    /// none: an applied entry the store did not mirror is unknown here and
    /// lag on the sync surface until a catch-up repairs it.
    UnknownObject,
    /// The pinned revision was never accepted for the object's entity type
    /// — retention holds accepted evolutions only, so an unretained pin is
    /// a caller error, not history.
    UnretainedRevision,
    /// The durable store could not be read; never answered as absence.
    StoreUnreadable(ProjectionStoreError),
}

/// The object at `object_ref` as a reader pinned at `pinned` sees it.
/// Pure over (projection facts, retained definitions).
///
/// With a plan whose entity type matches the object and whose
/// `to_revision` is the pin, `UpcastPending` is refined by the SAME
/// predicate the runner scans with: an object behind the pin that owes no
/// computed target reads [`UpcastState::Current`]. Any other plan says
/// nothing about this object and the structural rule stands.
pub fn object_at_revision(
    state: &ProjectionState,
    object_ref: &str,
    pinned: u32,
    plan: Option<&MigrationPlan>,
) -> Result<PinnedObject, ViewError> {
    let binding = state
        .bindings
        .get(object_ref)
        .ok_or(ViewError::UnknownObject)?;
    // Bindings hold only fold-validated `ety_` ids; a malformed one means
    // the projection cannot know this object.
    let type_id =
        EntityTypeId::new(binding.entity_type.clone()).map_err(|_| ViewError::UnknownObject)?;
    let definition = state
        .engine
        .entity_type_at_revision(&state.tenant_id, &type_id, pinned)
        .ok_or(ViewError::UnretainedRevision)?;
    let entity = state
        .objects
        .get(&state.tenant_id, object_ref)
        .ok_or(ViewError::UnknownObject)?;
    let properties = retained(definition, &entity.properties);
    let upcast_state = if binding.schema_revision >= pinned {
        UpcastState::Current
    } else {
        match plan {
            Some(plan) if plan.entity_type == binding.entity_type && plan.to_revision == pinned => {
                if crate::migrate::plan_owes(state, plan, object_ref) {
                    UpcastState::UpcastPending
                } else {
                    UpcastState::Current
                }
            }
            _ => UpcastState::UpcastPending,
        }
    };
    Ok(PinnedObject {
        properties,
        written_revision: binding.schema_revision,
        upcast_state,
    })
}

/// [`object_at_revision`] over the DURABLE store: what survives a restart is
/// what a reader is answered from. The registry supplies the retained
/// definitions; the store supplies the object and the revision it was
/// written under. This view takes no plan, so the structural rule is the
/// only one: written below the pin is pending.
pub fn object_at_revision_in_store(
    store: &dyn ProjectionStore,
    registry: &OntologyEngine,
    tenant_id: &str,
    object_ref: &str,
    pinned: u32,
) -> Result<PinnedObject, ViewError> {
    let projected = store
        .get(tenant_id, object_ref)
        .map_err(ViewError::StoreUnreadable)?
        .ok_or(ViewError::UnknownObject)?;
    let type_id = EntityTypeId::new(projected.entity.entity_type.value.clone())
        .map_err(|_| ViewError::UnknownObject)?;
    let definition = registry
        .entity_type_at_revision(tenant_id, &type_id, pinned)
        .ok_or(ViewError::UnretainedRevision)?;
    Ok(pinned_view(definition, &projected, pinned))
}

/// One projected object as a reader pinned at `pinned` sees it, without a
/// plan: the properties `definition` declares, and written below the pin
/// is pending. Every store-backed view builds its objects through this — the
/// single read, the page, and a set's named leaf — so no two of them can
/// derive one object differently.
pub(crate) fn pinned_view(
    definition: &data_ontology_kernel::EntityTypeDefinition,
    projected: &foundry_projection_draft::ProjectedObject,
    pinned: u32,
) -> PinnedObject {
    PinnedObject {
        properties: retained(definition, &projected.entity.properties),
        written_revision: projected.schema_revision,
        upcast_state: if projected.schema_revision >= pinned {
            UpcastState::Current
        } else {
            UpcastState::UpcastPending
        },
    }
}

/// The properties of an object that `definition` declares. Behind a pin the
/// filter is lossless under additive-only evolution; a property the pinned
/// definition does not declare is not the pinned reader's to see.
fn retained(
    definition: &data_ontology_kernel::EntityTypeDefinition,
    properties: &BTreeMap<String, ObjectProperty>,
) -> BTreeMap<String, ObjectProperty> {
    let declared: BTreeSet<&str> = definition
        .properties
        .iter()
        .map(|property| property.name.as_str())
        .collect();
    properties
        .iter()
        .filter(|(name, _)| declared.contains(name.as_str()))
        .map(|(name, property)| (name.clone(), property.clone()))
        .collect()
}

/// Typed refusals of the PAGED view, and of anything built on it. It shares
/// the pin and store arms with [`ViewError`] and adds the two only a filter can
/// reach, so the single-object read cannot be handed them. It has no
/// unknown-object arm: no producer of a page reads a single named object whose
/// absence it must report, so absence is an empty page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PageError {
    /// The pinned revision was never accepted for this entity type.
    UnretainedRevision,
    /// The durable store could not be read; never answered as absence.
    StoreUnreadable(ProjectionStoreError),
    /// A filter names a property the pinned definition does not declare.
    /// Refused rather than evaluated: the store matches on the object as
    /// stored, so such a filter would select rows by a value the pinned
    /// response omits, making the surface an oracle for a property outside
    /// the pin.
    UndeclaredFilterProperty,
    /// A range filter met a stored value of another kind. The caller's
    /// bounds are the argument, so this is a caller error; the store is
    /// not unreadable.
    FilterKindMismatch { property: String },
}

/// One page of rows, each as a reader pinned at `pinned` sees it, in
/// `object_ref` order. `next` is present exactly when the producer of this
/// page has rows past it: for a type scan, more objects of the type its
/// filter admits; for an object set, more members of the set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedPage {
    pub objects: Vec<(String, PinnedObject)>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    pub next: Option<ProjectionCursor>,       // data_class: INTERNAL_ONLY
}

/// [`object_at_revision_in_store`] over a whole page of one entity type,
/// optionally filtered. The pin resolves against the registry once, not per
/// row, because every object on the page has the queried type. This view
/// takes no plan, so written below the pin is pending for every row.
///
/// A `filter` may only name a property the pinned definition declares. The
/// store matches on the object AS STORED while the rows it returns are
/// filtered to the pinned vocabulary, so a filter on an undeclared property
/// would select rows the response cannot account for.
pub fn objects_of_type_at_revision(
    store: &dyn ProjectionStore,
    registry: &OntologyEngine,
    tenant_id: &str,
    entity_type: &EntityTypeId,
    pinned: u32,
    page: &PageRequest,
    filter: Option<&PropertyPredicate>,
) -> Result<PinnedPage, PageError> {
    let definition = registry
        .entity_type_at_revision(tenant_id, entity_type, pinned)
        .ok_or(PageError::UnretainedRevision)?;
    if let Some(filter) = filter
        && !definition
            .properties
            .iter()
            .any(|property| property.name == filter_property(filter))
    {
        return Err(PageError::UndeclaredFilterProperty);
    }
    let page = match filter {
        None => store.objects_of_type(tenant_id, &entity_type.value, page),
        Some(filter) => store.filter(tenant_id, &entity_type.value, filter, page),
    }
    .map_err(|error| match error {
        ProjectionStoreError::KindMismatch { property } => {
            PageError::FilterKindMismatch { property }
        }
        error => PageError::StoreUnreadable(error),
    })?;
    let objects = page
        .objects
        .into_iter()
        .map(|projected| {
            let view = pinned_view(definition, &projected, pinned);
            (projected.entity.id, view)
        })
        .collect();
    Ok(PinnedPage {
        objects,
        next: page.next,
    })
}

/// The property a predicate constrains.
fn filter_property(filter: &PropertyPredicate) -> &str {
    match filter {
        PropertyPredicate::Equals { property, .. } | PropertyPredicate::Range { property, .. } => {
            property.as_str()
        }
    }
}
