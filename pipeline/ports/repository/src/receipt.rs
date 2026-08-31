use crate::{
    ContentSelection, DigestBuilder, EvidenceDigest, PreparedSnapshot, ProducerId, ProfileId,
    RepositoryId, ResolvedRevision, RevisionId, SchemaId, SnapshotLimits, ToolId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotReceipt {
    repository: RepositoryId,
    base: RevisionId,
    head: RevisionId,
    resolved_base: ResolvedRevision,
    merge_base: ResolvedRevision,
    resolved_head: ResolvedRevision,
    profile: ProfileId,
    schema: SchemaId,
    limits: SnapshotLimits,
    profile_digest: EvidenceDigest,
    producer: ProducerId,
    tool: ToolId,
    merge_manifest_digest: EvidenceDigest,
    head_manifest_digest: EvidenceDigest,
    delta_digest: EvidenceDigest,
    selection_digest: EvidenceDigest,
    content_digest: EvidenceDigest,
    receipt_digest: EvidenceDigest,
}

impl SnapshotReceipt {
    pub(crate) fn complete(
        prepared: &PreparedSnapshot,
        selection: &ContentSelection,
        content_digest: EvidenceDigest,
    ) -> Self {
        let mut receipt = Self {
            repository: prepared.request().repository().clone(),
            base: prepared.request().base(),
            head: prepared.request().head(),
            resolved_base: prepared.resolved_base(),
            merge_base: prepared.merge_base().revision(),
            resolved_head: prepared.head().revision(),
            profile: prepared.request().profile().id().clone(),
            schema: prepared.request().profile().schema().clone(),
            limits: prepared.request().profile().limits(),
            profile_digest: prepared.request().profile().digest(),
            producer: prepared.producer().clone(),
            tool: prepared.tool().clone(),
            merge_manifest_digest: prepared.merge_base().digest(),
            head_manifest_digest: prepared.head().digest(),
            delta_digest: prepared.delta().digest(),
            selection_digest: selection.digest(),
            content_digest,
            receipt_digest: EvidenceDigest::of_bytes(b"pipeline-repository-empty-v1", b""),
        };
        receipt.receipt_digest = receipt.compute_digest();
        receipt
    }

    fn compute_digest(&self) -> EvidenceDigest {
        let mut digest = DigestBuilder::new(b"pipeline-repository-receipt-v1");
        digest.push_bytes(self.repository.as_str().as_bytes());
        digest.push_bytes(self.base.object_id().as_bytes());
        digest.push_bytes(self.head.object_id().as_bytes());
        digest_resolved(&mut digest, self.resolved_base);
        digest_resolved(&mut digest, self.merge_base);
        digest_resolved(&mut digest, self.resolved_head);
        digest.push_bytes(self.profile.as_str().as_bytes());
        digest.push_bytes(self.schema.as_str().as_bytes());
        self.limits.digest_into(&mut digest);
        digest.push_bytes(self.profile_digest.as_bytes());
        digest.push_bytes(self.producer.as_str().as_bytes());
        digest.push_bytes(self.tool.as_str().as_bytes());
        digest.push_bytes(self.merge_manifest_digest.as_bytes());
        digest.push_bytes(self.head_manifest_digest.as_bytes());
        digest.push_bytes(self.delta_digest.as_bytes());
        digest.push_bytes(self.selection_digest.as_bytes());
        digest.push_bytes(self.content_digest.as_bytes());
        digest.finish()
    }

    pub const fn digest(&self) -> EvidenceDigest {
        self.receipt_digest
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

    pub const fn resolved_base(&self) -> ResolvedRevision {
        self.resolved_base
    }

    pub const fn merge_base(&self) -> ResolvedRevision {
        self.merge_base
    }

    pub const fn resolved_head(&self) -> ResolvedRevision {
        self.resolved_head
    }

    pub const fn profile(&self) -> &ProfileId {
        &self.profile
    }

    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    pub const fn limits(&self) -> SnapshotLimits {
        self.limits
    }

    pub const fn profile_digest(&self) -> EvidenceDigest {
        self.profile_digest
    }

    pub const fn producer(&self) -> &ProducerId {
        &self.producer
    }

    pub const fn tool(&self) -> &ToolId {
        &self.tool
    }

    pub const fn merge_manifest_digest(&self) -> EvidenceDigest {
        self.merge_manifest_digest
    }

    pub const fn head_manifest_digest(&self) -> EvidenceDigest {
        self.head_manifest_digest
    }

    pub const fn content_digest(&self) -> EvidenceDigest {
        self.content_digest
    }

    pub const fn delta_digest(&self) -> EvidenceDigest {
        self.delta_digest
    }

    pub const fn selection_digest(&self) -> EvidenceDigest {
        self.selection_digest
    }
}

fn digest_resolved(digest: &mut DigestBuilder, revision: ResolvedRevision) {
    digest.push_bytes(revision.supplied().object_id().as_bytes());
    digest.push_bytes(revision.commit().object_id().as_bytes());
    digest.push_bytes(revision.tree().object_id().as_bytes());
}
