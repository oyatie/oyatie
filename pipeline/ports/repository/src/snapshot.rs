use std::collections::{BTreeMap, BTreeSet};

use crate::failure::enforce_limit;
use crate::{
    ContentId, DigestBuilder, Entry, EvidenceDigest, ProducerId, ProfileId, RepositoryDelta,
    RepositoryId, RepositoryManifest, ResolvedRevision, RevisionId, SchemaId, SnapshotFailure,
    SnapshotLimits, SnapshotProfile, SnapshotReceipt, ToolId, WorkControl,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotRequest {
    repository: RepositoryId,
    base: RevisionId,
    head: RevisionId,
    profile: SnapshotProfile,
}

impl SnapshotRequest {
    pub fn new(
        repository: RepositoryId,
        base: RevisionId,
        head: RevisionId,
        profile: SnapshotProfile,
    ) -> Result<Self, SnapshotFailure> {
        if base.algorithm() != head.algorithm() {
            return Err(SnapshotFailure::ObjectMismatch(
                "base and head revision algorithms differ".to_owned(),
            ));
        }
        Ok(Self {
            repository,
            base,
            head,
            profile,
        })
    }

    pub const fn repository(&self) -> &RepositoryId {
        &self.repository
    }

    pub const fn base(&self) -> RevisionId {
        self.base
    }

    pub const fn head(&self) -> RevisionId {
        self.head
    }

    pub const fn profile(&self) -> &SnapshotProfile {
        &self.profile
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedSnapshot {
    request: SnapshotRequest,
    resolved_base: ResolvedRevision,
    merge_base: RepositoryManifest,
    head: RepositoryManifest,
    delta: RepositoryDelta,
    producer: ProducerId,
    tool: ToolId,
}

impl PreparedSnapshot {
    pub fn new(
        request: SnapshotRequest,
        resolved_base: ResolvedRevision,
        merge_base: RepositoryManifest,
        head: RepositoryManifest,
        producer: ProducerId,
        tool: ToolId,
    ) -> Result<Self, SnapshotFailure> {
        if resolved_base.supplied() != request.base() || resolved_base.commit() != request.base() {
            return Err(SnapshotFailure::ObjectMismatch(
                "resolved base does not match the supplied base revision".to_owned(),
            ));
        }
        if head.revision().supplied() != request.head()
            || head.revision().commit() != request.head()
        {
            return Err(SnapshotFailure::ObjectMismatch(
                "resolved head does not match the supplied head revision".to_owned(),
            ));
        }
        if merge_base.revision().supplied() != merge_base.revision().commit() {
            return Err(SnapshotFailure::ObjectMismatch(
                "merge-base revision did not resolve to itself".to_owned(),
            ));
        }
        let algorithm = request.base().algorithm();
        for revision in [resolved_base, merge_base.revision(), head.revision()] {
            if revision.commit().algorithm() != algorithm
                || revision.tree().algorithm() != algorithm
            {
                return Err(SnapshotFailure::ObjectMismatch(
                    "resolved repository objects use different algorithms".to_owned(),
                ));
            }
        }
        let delta = RepositoryDelta::between(&merge_base, &head);
        Ok(Self {
            request,
            resolved_base,
            merge_base,
            head,
            delta,
            producer,
            tool,
        })
    }

    pub const fn request(&self) -> &SnapshotRequest {
        &self.request
    }

    pub const fn resolved_base(&self) -> ResolvedRevision {
        self.resolved_base
    }

    pub const fn merge_base(&self) -> &RepositoryManifest {
        &self.merge_base
    }

    pub const fn head(&self) -> &RepositoryManifest {
        &self.head
    }

    pub const fn delta(&self) -> &RepositoryDelta {
        &self.delta
    }

    pub const fn producer(&self) -> &ProducerId {
        &self.producer
    }

    pub const fn tool(&self) -> &ToolId {
        &self.tool
    }

    pub fn select_content(
        &self,
        ids: BTreeSet<ContentId>,
    ) -> Result<ContentSelection, SnapshotFailure> {
        enforce_limit(
            "selected content count",
            self.request.profile().limits().max_selected_contents(),
            ids.len() as u64,
        )?;
        let mut missing = ids.clone();
        let distinct_head = (self.merge_base.entries_digest() != self.head.entries_digest())
            .then(|| self.head.entries())
            .into_iter()
            .flatten();
        if !missing.is_empty() {
            for entry in self.merge_base.entries().iter().chain(distinct_head) {
                if let Some(content) = entry.content_id() {
                    missing.remove(&content);
                    if missing.is_empty() {
                        break;
                    }
                }
            }
        }
        if let Some(missing) = missing.first() {
            return Err(SnapshotFailure::MissingContent(missing.to_string()));
        }
        let mut digest = DigestBuilder::new(b"pipeline-repository-selection-v1");
        digest.push_u64(ids.len() as u64);
        for id in &ids {
            digest.push_bytes(id.object_id().as_bytes());
        }
        Ok(ContentSelection {
            ids,
            digest: digest.finish(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSelection {
    ids: BTreeSet<ContentId>,
    digest: EvidenceDigest,
}

impl ContentSelection {
    pub fn ids(&self) -> &BTreeSet<ContentId> {
        &self.ids
    }

    pub const fn digest(&self) -> EvidenceDigest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HydratedSnapshot {
    prepared: PreparedSnapshot,
    contents: BTreeMap<ContentId, Vec<u8>>,
    receipt: SnapshotReceipt,
}

impl HydratedSnapshot {
    pub fn complete(
        prepared: PreparedSnapshot,
        selection: ContentSelection,
        contents: BTreeMap<ContentId, Vec<u8>>,
    ) -> Result<Self, SnapshotFailure> {
        if contents.keys().ne(selection.ids().iter()) {
            if let Some(missing) = selection.ids().iter().find(|id| !contents.contains_key(id)) {
                return Err(SnapshotFailure::MissingContent(missing.to_string()));
            }
            let unexpected = contents
                .keys()
                .find(|id| !selection.ids().contains(id))
                .expect("unequal complete key sets have a differing key");
            return Err(SnapshotFailure::UnexpectedContent(unexpected.to_string()));
        }

        let limits = prepared.request().profile().limits();
        let mut total = 0_u64;
        let mut digest = DigestBuilder::new(b"pipeline-repository-content-v1");
        digest.push_u64(contents.len() as u64);
        for (id, bytes) in &contents {
            enforce_limit(
                "content bytes",
                limits.max_content_bytes(),
                bytes.len() as u64,
            )?;
            total =
                total
                    .checked_add(bytes.len() as u64)
                    .ok_or(SnapshotFailure::LimitExceeded {
                        limit: "total content bytes",
                        maximum: limits.max_total_content_bytes(),
                        observed: u64::MAX,
                    })?;
            enforce_limit(
                "total content bytes",
                limits.max_total_content_bytes(),
                total,
            )?;
            digest.push_bytes(id.object_id().as_bytes());
            digest.push_bytes(bytes);
        }
        let content_digest = digest.finish();
        let receipt = SnapshotReceipt::complete(&prepared, &selection, content_digest);
        Ok(Self {
            prepared,
            contents,
            receipt,
        })
    }

    pub const fn prepared(&self) -> &PreparedSnapshot {
        &self.prepared
    }

    pub fn content(&self, id: ContentId) -> Option<&[u8]> {
        self.contents.get(&id).map(Vec::as_slice)
    }

    pub const fn receipt(&self) -> &SnapshotReceipt {
        &self.receipt
    }
}

pub trait SnapshotSession: Sized {
    fn prepared(&self) -> &PreparedSnapshot;

    fn hydrate(
        self,
        selection: ContentSelection,
        control: &dyn WorkControl,
    ) -> Result<HydratedSnapshot, SnapshotFailure>;
}

pub trait RepositorySnapshot {
    type Session: SnapshotSession;

    fn capture(
        &self,
        request: SnapshotRequest,
        control: &dyn WorkControl,
    ) -> Result<Self::Session, SnapshotFailure>;
}
