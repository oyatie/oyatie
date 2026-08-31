//! The store contract: a per-entry write mirror only the projector may
//! drive, and the indexed, tenant-isolated read plane everything else
//! consumes.

use data_ontology_kernel::ObjectEntity;

use crate::predicate::PropertyPredicate;

/// One object as the projection holds it: the kernel entity plus the
/// fold's binding facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedObject {
    pub entity: ObjectEntity, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    pub schema_revision: u32, // data_class: INTERNAL_ONLY
    pub last_ordinal: u64,    // data_class: INTERNAL_ONLY
    pub last_actor: String,   // data_class: INTERNAL_ONLY
}

/// What one consumed log entry did to the projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntryOutcome {
    /// The entry applied; these are the touched objects, post-entry.
    Applied { objects: Vec<ProjectedObject> },
    /// The entry poisoned; the reason is a static label, never a
    /// classified value. The ordinal was spent and must advance.
    Poisoned { reason: String },
}

/// The per-entry mirror of the fold's outcome, applied atomically.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedEntry {
    pub tenant_id: String,     // data_class: INTERNAL_ONLY
    pub ordinal: u64,          // data_class: INTERNAL_ONLY
    pub outcome: EntryOutcome, // data_class: INTERNAL_ONLY
}

/// What the store did with an apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyReceipt {
    pub ordinal: u64,       // data_class: INTERNAL_ONLY
    pub deduplicated: bool, // data_class: INTERNAL_ONLY
}

/// Why the store refused. A store failure is infrastructure — never a
/// poison, because it is not derivable from (log bytes, registry).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionStoreError {
    /// Applies are dense per tenant; a gap or skip is refused so a
    /// crashed runner can never silently hole the projection.
    NonDenseOrdinal { expected: u64, found: u64 },
    /// A re-apply at an applied ordinal carried different content.
    DivergentReplay { ordinal: u64 },
    /// The entry is malformed (blank tenant, tenant-mismatched object,
    /// blank poison reason, zero page limit, ...).
    Entry { detail: &'static str },
    /// A range predicate met a stored value of a different storage
    /// class — schema drift surfaces loudly, never as a silent false.
    ClassMismatch { property: String },
    /// The backing store failed; the detail is diagnostic.
    Storage { detail: String },
}

/// A typed resume point: pages continue strictly after this object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionCursor {
    pub after_object_ref: String, // data_class: INTERNAL_ONLY
}

/// One read page request; `limit` must be non-zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageRequest {
    pub limit: usize,                     // data_class: INTERNAL_ONLY
    pub cursor: Option<ProjectionCursor>, // data_class: INTERNAL_ONLY
}

impl PageRequest {
    pub fn first(limit: usize) -> Self {
        Self {
            limit,
            cursor: None,
        }
    }

    pub fn after(limit: usize, cursor: ProjectionCursor) -> Self {
        Self {
            limit,
            cursor: Some(cursor),
        }
    }
}

/// One page of objects in `object_ref` order; `next` is present exactly
/// when more objects remain past this page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page {
    pub objects: Vec<ProjectedObject>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    pub next: Option<ProjectionCursor>, // data_class: INTERNAL_ONLY
}

/// The projection store: dense per-entry applies in, indexed
/// tenant-isolated reads out. Reads are deterministic — `object_ref`
/// ascending — so pages partition the full result.
pub trait ProjectionStore {
    fn apply(&mut self, entry: AppliedEntry) -> Result<ApplyReceipt, ProjectionStoreError>;

    /// The highest ordinal applied for the tenant (0 = nothing yet).
    fn applied_head(&self, tenant_id: &str) -> Result<u64, ProjectionStoreError>;

    fn get(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError>;

    fn objects_of_type(
        &self,
        tenant_id: &str,
        entity_type: &str,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError>;

    fn filter(
        &self,
        tenant_id: &str,
        entity_type: &str,
        predicate: &PropertyPredicate,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError>;

    /// Every poisoned ordinal and its static reason — nothing hidden.
    fn poisoned(&self, tenant_id: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError>;
}
