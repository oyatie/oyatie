//! Workspace notes kernel.
//!
//! Typed kernel records for the W-Workspace-GA Notes / Keep surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns note-store
//! metadata, CRDT binding to the shared collab runtime, and tag/folder graph
//! validation without owning HTTP, storage, search indexing, or the Yrs adapter.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use intelligence_collab_runtime_domain::{CollabRuntime, CollabSurface};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const NOTE_STORE_SCHEMA_VERSION: u32 = 1;
const NOTE_SCHEMA_VERSION: u32 = 1;
const MAX_TAGS_PER_NOTE: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NoteError {
    InvalidStoreId,
    InvalidNoteId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidOwnerRef,
    InvalidTitle,
    InvalidFolderId,
    InvalidFolderName,
    DuplicateFolderId,
    MissingRootFolder,
    MultipleRootFolders,
    MissingFolderParent,
    SelfParentFolder,
    FolderCycle,
    UnknownFolder,
    InvalidTagId,
    InvalidTagLabel,
    DuplicateTagId,
    DuplicateTagLabel,
    UnknownTag,
    TooManyTags,
    DuplicateNoteTag,
    InvalidCollabRuntime,
    InvalidCollabSurface,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NoteColor {
    Default,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Gray,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteStoreCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub owner_ref: String,                    // data_class: PII_IDENTIFYING
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub folders: Vec<NoteFolder>,             // data_class: PII_QUASI_IDENTIFIER
    pub tags: Vec<NoteTag>,                   // data_class: PII_QUASI_IDENTIFIER
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteStore {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub owner_ref: Classified<String>,             // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub folders: Classified<Vec<NoteFolder>>,      // data_class: PII_QUASI_IDENTIFIER
    pub tags: Classified<Vec<NoteTag>>,            // data_class: PII_QUASI_IDENTIFIER
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteCreate {
    pub id: String,                            // data_class: INTERNAL_ONLY
    pub store_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: INTERNAL_ONLY
    pub cell_id: String,                       // data_class: INTERNAL_ONLY
    pub owner_ref: String,                     // data_class: PII_IDENTIFYING
    pub title: String,                         // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Option<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub collab_runtime: CollabRuntime,         // data_class: PII_IDENTIFYING
    pub folder_id: String,                     // data_class: INTERNAL_ONLY
    pub tag_ids: Vec<String>,                  // data_class: INTERNAL_ONLY
    pub color: NoteColor,                      // data_class: INTERNAL_ONLY
    pub is_pinned: bool,                       // data_class: INTERNAL_ONLY
    pub is_archived: bool,                     // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Note {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub store_id: Classified<String>,              // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub owner_ref: Classified<String>,             // data_class: PII_IDENTIFYING
    pub title: Classified<String>,                 // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub collab_runtime: Classified<CollabRuntime>, // data_class: PII_IDENTIFYING
    pub folder_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub tag_ids: Classified<Vec<String>>,          // data_class: INTERNAL_ONLY
    pub color: Classified<NoteColor>,              // data_class: INTERNAL_ONLY
    pub is_pinned: Classified<bool>,               // data_class: INTERNAL_ONLY
    pub is_archived: Classified<bool>,             // data_class: INTERNAL_ONLY
    pub indexed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NoteFolder {
    pub folder_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub parent_folder_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub name: Classified<String>,      // data_class: PII_QUASI_IDENTIFIER
    pub ordinal: Classified<u32>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NoteTag {
    pub tag_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub label: Classified<String>,    // data_class: PII_QUASI_IDENTIFIER
    pub color: Classified<NoteColor>, // data_class: INTERNAL_ONLY
}

pub trait NoteStoreReader {
    fn read_note(
        &self,
        tenant_id: &str,
        store_id: &str,
        note_id: &str,
    ) -> Result<Option<Note>, NoteError>;
}

impl NoteStore {
    pub fn new(input: NoteStoreCreate) -> Result<Self, NoteError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_note_data_class());
        validate_non_empty(&input.id, NoteError::InvalidStoreId)?;
        validate_non_empty(&input.tenant_id, NoteError::InvalidTenantId)?;
        validate_non_empty(&input.region, NoteError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, NoteError::InvalidCellId)?;
        validate_non_empty(&input.owner_ref, NoteError::InvalidOwnerRef)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        validate_folders(&input.folders)?;
        validate_tags(&input.tags)?;

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            owner_ref: Classified::new(input.owner_ref, note_owner_data_class()),
            data_class: internal(data_class),
            folders: Classified::new(input.folders, note_metadata_data_class()),
            tags: Classified::new(input.tags, note_metadata_data_class()),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(NOTE_STORE_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }

    pub fn folder_ids(&self) -> BTreeSet<String> {
        self.folders
            .value
            .iter()
            .map(|folder| folder.folder_id.value.clone())
            .collect()
    }

    pub fn tag_ids(&self) -> BTreeSet<String> {
        self.tags
            .value
            .iter()
            .map(|tag| tag.tag_id.value.clone())
            .collect()
    }
}

impl Note {
    pub fn new(input: NoteCreate, store: &NoteStore) -> Result<Self, NoteError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_note_data_class());
        validate_non_empty(&input.id, NoteError::InvalidNoteId)?;
        validate_non_empty(&input.store_id, NoteError::InvalidStoreId)?;
        validate_non_empty(&input.tenant_id, NoteError::InvalidTenantId)?;
        validate_non_empty(&input.region, NoteError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, NoteError::InvalidCellId)?;
        validate_non_empty(&input.owner_ref, NoteError::InvalidOwnerRef)?;
        validate_text(&input.title, NoteError::InvalidTitle)?;
        validate_non_empty(&input.folder_id, NoteError::InvalidFolderId)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        validate_store_binding(&input, store)?;
        validate_collab_runtime_binding(
            &input.collab_runtime,
            &input.id,
            &input.tenant_id,
            &input.region,
            &input.cell_id,
        )?;
        validate_note_placement(&input.folder_id, &input.tag_ids, store)?;

        Ok(Self {
            id: internal(input.id),
            store_id: internal(input.store_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            owner_ref: Classified::new(input.owner_ref, note_owner_data_class()),
            title: Classified::new(input.title, note_metadata_data_class()),
            data_class: internal(data_class),
            collab_runtime: Classified::new(input.collab_runtime, note_content_data_class()),
            folder_id: internal(input.folder_id),
            tag_ids: internal(input.tag_ids),
            color: internal(input.color),
            is_pinned: internal(input.is_pinned),
            is_archived: internal(input.is_archived),
            indexed_at_epoch_seconds: internal(input.indexed_at_epoch_seconds),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(NOTE_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl NoteFolder {
    pub fn new(
        folder_id: String,
        parent_folder_id: Option<String>,
        name: String,
        ordinal: u32,
    ) -> Result<Self, NoteError> {
        validate_non_empty(&folder_id, NoteError::InvalidFolderId)?;
        if let Some(parent_folder_id) = parent_folder_id.as_deref() {
            validate_non_empty(parent_folder_id, NoteError::InvalidFolderId)?;
            if parent_folder_id == folder_id {
                return Err(NoteError::SelfParentFolder);
            }
        }
        validate_text(&name, NoteError::InvalidFolderName)?;
        Ok(Self {
            folder_id: internal(folder_id),
            parent_folder_id: internal(parent_folder_id),
            name: Classified::new(name, note_metadata_data_class()),
            ordinal: internal(ordinal),
        })
    }
}

impl NoteTag {
    pub fn new(tag_id: String, label: String, color: NoteColor) -> Result<Self, NoteError> {
        validate_non_empty(&tag_id, NoteError::InvalidTagId)?;
        validate_text(&label, NoteError::InvalidTagLabel)?;
        Ok(Self {
            tag_id: internal(tag_id),
            label: Classified::new(label, note_metadata_data_class()),
            color: internal(color),
        })
    }
}

pub fn default_workspace_note_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn note_content_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn note_owner_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn note_metadata_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn workspace_note_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, NoteError> {
    PrivacyDataClass::new(data_class).map_err(|_| NoteError::InvalidDataClass)
}

fn validate_store_binding(input: &NoteCreate, store: &NoteStore) -> Result<(), NoteError> {
    if input.store_id != store.id.value {
        return Err(NoteError::InvalidStoreId);
    }
    if input.tenant_id != store.tenant_id.value {
        return Err(NoteError::InvalidTenantId);
    }
    if input.region != store.region.value {
        return Err(NoteError::InvalidRegion);
    }
    if input.cell_id != store.cell_id.value {
        return Err(NoteError::InvalidCellId);
    }
    if input.owner_ref != store.owner_ref.value {
        return Err(NoteError::InvalidOwnerRef);
    }
    Ok(())
}

fn validate_collab_runtime_binding(
    runtime: &CollabRuntime,
    note_id: &str,
    tenant_id: &str,
    region: &str,
    cell_id: &str,
) -> Result<(), NoteError> {
    if runtime.surface.value != CollabSurface::Notes {
        return Err(NoteError::InvalidCollabSurface);
    }
    if runtime.document_id.value != note_id
        || runtime.tenant_id.value != tenant_id
        || runtime.region.value != region
        || runtime.cell_id.value != cell_id
    {
        return Err(NoteError::InvalidCollabRuntime);
    }
    Ok(())
}

fn validate_note_placement(
    folder_id: &str,
    tag_ids: &[String],
    store: &NoteStore,
) -> Result<(), NoteError> {
    let folder_ids = store.folder_ids();
    if !folder_ids.contains(folder_id) {
        return Err(NoteError::UnknownFolder);
    }
    if tag_ids.len() > MAX_TAGS_PER_NOTE {
        return Err(NoteError::TooManyTags);
    }
    let known_tag_ids = store.tag_ids();
    let mut seen = BTreeSet::new();
    for tag_id in tag_ids {
        validate_non_empty(tag_id, NoteError::InvalidTagId)?;
        if !known_tag_ids.contains(tag_id) {
            return Err(NoteError::UnknownTag);
        }
        if !seen.insert(tag_id) {
            return Err(NoteError::DuplicateNoteTag);
        }
    }
    Ok(())
}

fn validate_folders(folders: &[NoteFolder]) -> Result<(), NoteError> {
    if folders.is_empty() {
        return Err(NoteError::MissingRootFolder);
    }
    let mut ids = BTreeSet::new();
    let mut parent_by_id = BTreeMap::new();
    let mut root_count = 0_u32;
    for folder in folders {
        validate_non_empty(&folder.folder_id.value, NoteError::InvalidFolderId)?;
        validate_text(&folder.name.value, NoteError::InvalidFolderName)?;
        if !ids.insert(folder.folder_id.value.clone()) {
            return Err(NoteError::DuplicateFolderId);
        }
        if folder.parent_folder_id.value.is_none() {
            root_count += 1;
        }
        if let Some(parent_id) = folder.parent_folder_id.value.as_deref() {
            validate_non_empty(parent_id, NoteError::InvalidFolderId)?;
            if parent_id == folder.folder_id.value {
                return Err(NoteError::SelfParentFolder);
            }
        }
        parent_by_id.insert(
            folder.folder_id.value.clone(),
            folder.parent_folder_id.value.clone(),
        );
    }
    match root_count {
        0 => return Err(NoteError::MissingRootFolder),
        1 => {}
        _ => return Err(NoteError::MultipleRootFolders),
    }
    for (folder_id, parent_id) in &parent_by_id {
        if let Some(parent_id) = parent_id {
            if !parent_by_id.contains_key(parent_id) {
                return Err(NoteError::MissingFolderParent);
            }
            validate_no_folder_cycle(folder_id, &parent_by_id)?;
        }
    }
    Ok(())
}

fn validate_no_folder_cycle(
    folder_id: &str,
    parent_by_id: &BTreeMap<String, Option<String>>,
) -> Result<(), NoteError> {
    let mut seen = BTreeSet::new();
    let mut current = Some(folder_id.to_owned());
    while let Some(current_id) = current {
        if !seen.insert(current_id.clone()) {
            return Err(NoteError::FolderCycle);
        }
        current = parent_by_id.get(&current_id).and_then(Clone::clone);
    }
    Ok(())
}

fn validate_tags(tags: &[NoteTag]) -> Result<(), NoteError> {
    let mut ids = BTreeSet::new();
    let mut labels = BTreeSet::new();
    for tag in tags {
        validate_non_empty(&tag.tag_id.value, NoteError::InvalidTagId)?;
        validate_text(&tag.label.value, NoteError::InvalidTagLabel)?;
        if !ids.insert(tag.tag_id.value.clone()) {
            return Err(NoteError::DuplicateTagId);
        }
        if !labels.insert(tag.label.value.clone()) {
            return Err(NoteError::DuplicateTagLabel);
        }
    }
    Ok(())
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), NoteError> {
    if updated_at < created_at {
        Err(NoteError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_text(value: &str, error: NoteError) -> Result<(), NoteError> {
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: NoteError) -> Result<(), NoteError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_collab_runtime_domain::{
        CollabRuntimeCreate, CollabSnapshotRef, CollabStateVectorRef,
    };
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn runtime(surface: CollabSurface) -> CollabRuntime {
        CollabRuntime::new(CollabRuntimeCreate {
            document_id: "note-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            surface,
            data_class: None,
            snapshot: CollabSnapshotRef::new(
                "snap-1".into(),
                "tenant-1/notes/note-1/snap-1".into(),
                "sha256:snapshot".into(),
                "sv:1".into(),
                1,
                1,
                128,
            )
            .unwrap(),
            state_vector: CollabStateVectorRef::new("sv:1".into(), 1, 1, 32).unwrap(),
            active_awareness: Vec::new(),
            created_at_epoch_millis: 1_700_000_000_000,
            updated_at_epoch_millis: 1_700_000_010_000,
        })
        .unwrap()
    }

    fn folder(folder_id: &str, parent_folder_id: Option<&str>, name: &str) -> NoteFolder {
        NoteFolder::new(
            folder_id.into(),
            parent_folder_id.map(str::to_owned),
            name.into(),
            1,
        )
        .unwrap()
    }

    fn tag(tag_id: &str, label: &str) -> NoteTag {
        NoteTag::new(tag_id.into(), label.into(), NoteColor::Yellow).unwrap()
    }

    fn store_input() -> NoteStoreCreate {
        NoteStoreCreate {
            id: "store-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            owner_ref: "user:owner@example.com".into(),
            data_class: None,
            folders: vec![
                folder("root", None, "Root"),
                folder("projects", Some("root"), "Projects"),
            ],
            tags: vec![tag("tag-1", "Incident"), tag("tag-2", "Idea")],
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn store() -> NoteStore {
        NoteStore::new(store_input()).unwrap()
    }

    fn note_input(surface: CollabSurface) -> NoteCreate {
        NoteCreate {
            id: "note-1".into(),
            store_id: "store-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            owner_ref: "user:owner@example.com".into(),
            title: "Postmortem draft".into(),
            data_class: None,
            collab_runtime: runtime(surface),
            folder_id: "projects".into(),
            tag_ids: vec!["tag-1".into()],
            color: NoteColor::Blue,
            is_pinned: true,
            is_archived: false,
            indexed_at_epoch_seconds: None,
            created_at_epoch_seconds: 1_700_000_020,
            updated_at_epoch_seconds: 1_700_000_030,
        }
    }

    #[test]
    fn note_store_defaults_to_identifying_and_classifies_metadata() {
        let store = store();

        assert_eq!(
            store.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            store.owner_ref.data_class,
            DataClassification::Privacy(note_owner_data_class())
        );
        assert_eq!(
            store.tags.data_class,
            DataClassification::Privacy(note_metadata_data_class())
        );
        assert_eq!(store.schema_version.value, 1);
    }

    #[test]
    fn note_requires_notes_collab_runtime_and_known_placement() {
        let store = store();
        let note = Note::new(note_input(CollabSurface::Notes), &store).unwrap();
        assert_eq!(
            note.collab_runtime.data_class,
            DataClassification::Privacy(note_content_data_class())
        );

        assert_eq!(
            Note::new(note_input(CollabSurface::Docs), &store),
            Err(NoteError::InvalidCollabSurface)
        );

        let mut unknown_folder = note_input(CollabSurface::Notes);
        unknown_folder.folder_id = "missing".into();
        assert_eq!(
            Note::new(unknown_folder, &store),
            Err(NoteError::UnknownFolder)
        );

        let mut duplicate_tag = note_input(CollabSurface::Notes);
        duplicate_tag.tag_ids = vec!["tag-1".into(), "tag-1".into()];
        assert_eq!(
            Note::new(duplicate_tag, &store),
            Err(NoteError::DuplicateNoteTag)
        );
    }

    #[test]
    fn folder_graph_rejects_missing_root_cycles_and_duplicate_tags() {
        let mut no_root = store_input();
        no_root.folders = vec![folder("a", Some("b"), "A"), folder("b", Some("a"), "B")];
        assert_eq!(NoteStore::new(no_root), Err(NoteError::MissingRootFolder));

        let mut cycle = store_input();
        cycle.folders = vec![
            folder("root", None, "Root"),
            folder("a", Some("b"), "A"),
            folder("b", Some("a"), "B"),
        ];
        assert_eq!(NoteStore::new(cycle), Err(NoteError::FolderCycle));

        let mut duplicate_tag = store_input();
        duplicate_tag.tags = vec![tag("tag-1", "Incident"), tag("tag-1", "Incident 2")];
        assert_eq!(
            NoteStore::new(duplicate_tag),
            Err(NoteError::DuplicateTagId)
        );
    }

    #[test]
    fn labels_and_titles_reject_control_or_padding() {
        assert_eq!(
            NoteTag::new("tag-bad".into(), " padded".into(), NoteColor::Red),
            Err(NoteError::InvalidTagLabel)
        );

        let mut bad_title = note_input(CollabSurface::Notes);
        bad_title.title = "bad\nname".into();
        assert_eq!(Note::new(bad_title, &store()), Err(NoteError::InvalidTitle));
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_note_data_class_from_legacy(DataClass::Audit),
            Err(NoteError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.notes STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotesSurfaceStaging {
    pub note_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub char_count: Classified<u64>,   // data_class: INTERNAL_ONLY
}

impl NotesSurfaceStaging {
    pub fn new(note_id: String, tenant_id: String, char_count: u64) -> Self {
        Self {
            note_id: Classified::new(note_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            char_count: Classified::new(char_count, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> NotesSurfaceStaging {
        NotesSurfaceStaging::new("notes-1".into(), "notes-1".into(), 0u64)
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.note_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
