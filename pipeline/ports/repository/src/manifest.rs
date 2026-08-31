use std::sync::Arc;

use crate::failure::enforce_limit;
use crate::manifest_digest::{entries_digest, manifest_digest};
use crate::{
    ContentId, DigestBuilder, EvidenceDigest, ObjectId, RepositoryPath, RevisionId,
    SnapshotFailure, SnapshotLimits, TreeId,
};

const MANIFEST_ENTRY_FRAMING_BYTES: u64 = 8;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryKind {
    Tree,
    Blob,
    ExecutableBlob,
    Symlink,
    Gitlink,
}

impl EntryKind {
    pub const fn is_tree(self) -> bool {
        matches!(self, Self::Tree)
    }

    pub const fn is_regular_blob(self) -> bool {
        matches!(self, Self::Blob | Self::ExecutableBlob)
    }

    pub const fn canonical_mode(self) -> &'static [u8] {
        match self {
            Self::Tree => b"040000",
            Self::Blob => b"100644",
            Self::ExecutableBlob => b"100755",
            Self::Symlink => b"120000",
            Self::Gitlink => b"160000",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntryState {
    kind: EntryKind,
    object: ObjectId,
}

impl EntryState {
    pub const fn new(kind: EntryKind, object: ObjectId) -> Self {
        Self { kind, object }
    }

    pub const fn kind(self) -> EntryKind {
        self.kind
    }

    pub const fn object(self) -> ObjectId {
        self.object
    }

    pub const fn content_id(self) -> Option<ContentId> {
        if self.kind.is_tree() || matches!(self.kind, EntryKind::Gitlink) {
            None
        } else {
            Some(ContentId::from_object_id(self.object))
        }
    }

    pub(crate) fn digest_into(self, digest: &mut DigestBuilder) {
        digest.push_bytes(self.kind.canonical_mode());
        digest.push_bytes(self.object.algorithm().as_str().as_bytes());
        digest.push_bytes(self.object.as_bytes());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    path: RepositoryPath,
    state: EntryState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ManifestMeter {
    entries: u64,
    bytes: u64,
}

impl ManifestMeter {
    pub fn admit(
        &mut self,
        path: &[u8],
        object: ObjectId,
        limits: SnapshotLimits,
    ) -> Result<(), SnapshotFailure> {
        let entries = self.entries.saturating_add(1);
        enforce_limit("entry count", limits.max_entries(), entries)?;

        let path_bytes = u64::try_from(path.len()).unwrap_or(u64::MAX);
        enforce_limit("path bytes", limits.max_path_bytes(), path_bytes)?;
        let entry_bytes = path_bytes
            .saturating_add(object.as_bytes().len() as u64 + MANIFEST_ENTRY_FRAMING_BYTES);
        let bytes = self.bytes.saturating_add(entry_bytes);
        enforce_limit("manifest bytes", limits.max_manifest_bytes(), bytes)?;

        self.entries = entries;
        self.bytes = bytes;
        Ok(())
    }
}

impl Entry {
    pub const fn new(path: RepositoryPath, state: EntryState) -> Self {
        Self { path, state }
    }

    pub const fn path(&self) -> &RepositoryPath {
        &self.path
    }

    pub const fn state(&self) -> EntryState {
        self.state
    }

    pub const fn kind(&self) -> EntryKind {
        self.state.kind()
    }

    pub const fn object(&self) -> ObjectId {
        self.state.object()
    }

    pub const fn content_id(&self) -> Option<ContentId> {
        self.state.content_id()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedRevision {
    supplied: RevisionId,
    commit: RevisionId,
    tree: TreeId,
}

impl ResolvedRevision {
    pub fn new(
        supplied: RevisionId,
        commit: RevisionId,
        tree: TreeId,
    ) -> Result<Self, SnapshotFailure> {
        if supplied.algorithm() != commit.algorithm() || commit.algorithm() != tree.algorithm() {
            return Err(SnapshotFailure::ObjectMismatch(
                "revision and tree algorithms differ".to_owned(),
            ));
        }
        Ok(Self {
            supplied,
            commit,
            tree,
        })
    }

    pub const fn supplied(self) -> RevisionId {
        self.supplied
    }

    pub const fn commit(self) -> RevisionId {
        self.commit
    }

    pub const fn tree(self) -> TreeId {
        self.tree
    }

    pub(crate) fn digest_into(self, digest: &mut crate::DigestBuilder) {
        digest.push_bytes(self.supplied.object_id().as_bytes());
        digest.push_bytes(self.commit.object_id().as_bytes());
        digest.push_bytes(self.tree.object_id().as_bytes());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryManifest {
    revision: ResolvedRevision,
    entries: Arc<[Entry]>,
    limits: SnapshotLimits,
    entries_digest: EvidenceDigest,
    digest: EvidenceDigest,
}

impl RepositoryManifest {
    pub fn new(
        revision: ResolvedRevision,
        mut entries: Vec<Entry>,
        limits: SnapshotLimits,
    ) -> Result<Self, SnapshotFailure> {
        enforce_limit("entry count", limits.max_entries(), entries.len() as u64)?;
        entries.sort_unstable_by(|left, right| left.path.cmp(&right.path));

        let mut meter = ManifestMeter::default();
        let mut previous: Option<&RepositoryPath> = None;
        let algorithm = revision.tree().algorithm();
        for entry in &entries {
            if previous == Some(entry.path()) {
                return Err(SnapshotFailure::DuplicatePath(
                    entry.path().as_bytes().to_vec(),
                ));
            }
            previous = Some(entry.path());
            meter.admit(entry.path().as_bytes(), entry.object(), limits)?;
            if entry.object().algorithm() != algorithm {
                return Err(SnapshotFailure::ObjectMismatch(format!(
                    "entry {} uses {} but tree uses {algorithm}",
                    entry.path(),
                    entry.object().algorithm()
                )));
            }
            if let Some(parent) = parent_path(entry.path())
                && !entries
                    .binary_search_by(|candidate| candidate.path().as_bytes().cmp(parent))
                    .ok()
                    .is_some_and(|index| entries[index].kind().is_tree())
            {
                return Err(SnapshotFailure::MalformedOutput(format!(
                    "entry {} has no tree entry for parent {parent:?}",
                    entry.path()
                )));
            }
        }

        let entries_digest = entries_digest(&entries);
        let digest = manifest_digest(revision, entries_digest);
        Ok(Self {
            revision,
            entries: entries.into(),
            limits,
            entries_digest,
            digest,
        })
    }

    pub fn at_revision(revision: ResolvedRevision, source: &Self) -> Result<Self, SnapshotFailure> {
        if revision.tree() != source.revision.tree() {
            return Err(SnapshotFailure::ObjectMismatch(
                "shared manifest revisions name different trees".to_owned(),
            ));
        }
        Ok(Self {
            revision,
            entries: Arc::clone(&source.entries),
            limits: source.limits,
            entries_digest: source.entries_digest,
            digest: manifest_digest(revision, source.entries_digest),
        })
    }

    pub const fn revision(&self) -> ResolvedRevision {
        self.revision
    }

    pub fn entries(&self) -> &[Entry] {
        self.entries.as_ref()
    }

    pub const fn digest(&self) -> EvidenceDigest {
        self.digest
    }

    pub(crate) const fn entries_digest(&self) -> EvidenceDigest {
        self.entries_digest
    }

    pub fn entry(&self, path: &RepositoryPath) -> Option<&Entry> {
        self.entries
            .binary_search_by(|entry| entry.path().cmp(path))
            .ok()
            .map(|index| &self.entries[index])
    }

    pub fn files_under<'a>(
        &'a self,
        directory: &RepositoryPath,
    ) -> impl Iterator<Item = &'a Entry> + 'a {
        let mut prefix = Vec::with_capacity(directory.as_bytes().len() + 1);
        prefix.extend_from_slice(directory.as_bytes());
        prefix.push(b'/');
        let start = self
            .entries
            .partition_point(|entry| entry.path().as_bytes() < prefix.as_slice());
        self.entries[start..]
            .iter()
            .take_while(move |entry| entry.path().as_bytes().starts_with(&prefix))
            .filter(|entry| !entry.kind().is_tree())
    }
}

fn parent_path(path: &RepositoryPath) -> Option<&[u8]> {
    let index = path.as_bytes().iter().rposition(|byte| *byte == b'/')?;
    Some(&path.as_bytes()[..index])
}
