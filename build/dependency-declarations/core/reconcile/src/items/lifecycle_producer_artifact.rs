/// One immutable lifecycle observation assembled from qualified Build facts.
pub struct LifecycleProducerArtifactV1 {
    envelope: FactEnvelopeV1,
    released_releases: ReleaseLedgerV1,
    preview_releases: ReleaseLedgerV1,
    advisories: AdvisoryLedgerV1,
    dependencies: DependencyGraphV1,
    toolchains: ToolchainMatrixV1,
    channels: ToolchainChannelSnapshotV1,
    identity_sha256: DigestV1,
}

impl std::fmt::Debug for LifecycleProducerArtifactV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LifecycleProducerArtifactV1")
            .field("envelope_sha256", &self.envelope.identity_sha256())
            .field("released_item_count", &self.released_releases.items().len())
            .field("preview_item_count", &self.preview_releases.items().len())
            .field("advisory_count", &self.advisories.facts().len())
            .field("dependency_count", &self.dependencies.nodes().len())
            .field("identity_sha256", &self.identity_sha256)
            .finish()
    }
}

impl LifecycleProducerArtifactV1 {
    pub fn try_new(
        envelope: FactEnvelopeV1,
        released_releases: ReleaseLedgerV1,
        preview_releases: ReleaseLedgerV1,
        advisories: AdvisoryLedgerV1,
        dependencies: DependencyGraphV1,
        toolchains: ToolchainMatrixV1,
        channels: ToolchainChannelSnapshotV1,
    ) -> Result<Self, LifecycleFailureV1> {
        validate_artifact_envelopes(&envelope, &dependencies, &toolchains, &channels)?;
        released_releases.require_released_complete()?;
        validate_preview_releases(&preview_releases)?;
        validate_advisory_coverage(&advisories)?;

        let mut hash = CanonicalHasherV1::new(b"build.lifecycle-producer-artifact.v1\0");
        hash.digest(envelope.identity_sha256());
        hash.digest(released_releases.identity_sha256());
        hash.digest(preview_releases.identity_sha256());
        hash.digest(advisories.identity_sha256());
        hash.digest(dependencies.identity_sha256());
        hash.digest(toolchains.identity_sha256());
        hash.digest(channels.identity_sha256());

        Ok(Self {
            envelope,
            released_releases,
            preview_releases,
            advisories,
            dependencies,
            toolchains,
            channels,
            identity_sha256: hash.finish(),
        })
    }

    pub fn require_safe_at(
        &self,
        now: LifecycleTimestampV1,
    ) -> Result<(), LifecycleFailureV1> {
        if now < self.channels.observed_at() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::StaleFact,
            ));
        }
        self.envelope.require_safe(now)?;
        self.dependencies.envelope().require_safe(now)
    }

    #[must_use]
    pub const fn envelope(&self) -> &FactEnvelopeV1 {
        &self.envelope
    }

    #[must_use]
    pub const fn released_releases(&self) -> &ReleaseLedgerV1 {
        &self.released_releases
    }

    #[must_use]
    pub const fn preview_releases(&self) -> &ReleaseLedgerV1 {
        &self.preview_releases
    }

    #[must_use]
    pub const fn advisories(&self) -> &AdvisoryLedgerV1 {
        &self.advisories
    }

    #[must_use]
    pub const fn dependencies(&self) -> &DependencyGraphV1 {
        &self.dependencies
    }

    #[must_use]
    pub const fn toolchains(&self) -> &ToolchainMatrixV1 {
        &self.toolchains
    }

    #[must_use]
    pub const fn channels(&self) -> &ToolchainChannelSnapshotV1 {
        &self.channels
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

fn validate_artifact_envelopes(
    envelope: &FactEnvelopeV1,
    dependencies: &DependencyGraphV1,
    toolchains: &ToolchainMatrixV1,
    channels: &ToolchainChannelSnapshotV1,
) -> Result<(), LifecycleFailureV1> {
    envelope.require_safe(channels.observed_at())?;
    dependencies
        .envelope()
        .require_safe(channels.observed_at())?;

    let artifact_scope = envelope.temporal().scope();
    let dependency_scope = dependencies.envelope().temporal().scope();
    if !same_repository_view(artifact_scope, dependency_scope)
        || artifact_scope.toolchain_sha256() != toolchains.identity_sha256()
        || dependency_scope.toolchain_sha256() != toolchains.identity_sha256()
        || toolchains.stable().tools().rustc().host_triple() != channels.host_triple()
    {
        return Err(lifecycle_artifact_mismatch());
    }
    Ok(())
}

fn same_repository_view(left: &FactTemporalScopeV1, right: &FactTemporalScopeV1) -> bool {
    left.repository_identity() == right.repository_identity()
        && left.repository_revision_sha256() == right.repository_revision_sha256()
        && left.repository_snapshot_sha256() == right.repository_snapshot_sha256()
        && left.configuration_sha256() == right.configuration_sha256()
        && left.toolchain_sha256() == right.toolchain_sha256()
}

fn validate_preview_releases(preview: &ReleaseLedgerV1) -> Result<(), LifecycleFailureV1> {
    for batch in preview.batches() {
        if !batch.extraction().qualification().is_qualified() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnqualifiedExtraction,
            ));
        }
        if batch.source().maturity() != SourceMaturityV1::Provisional {
            return Err(lifecycle_artifact_mismatch());
        }
    }
    match preview.completeness() {
        ReleaseLedgerCompletenessV1::Provisional => Ok(()),
        ReleaseLedgerCompletenessV1::UnqualifiedExtraction => Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::UnqualifiedExtraction,
        )),
        ReleaseLedgerCompletenessV1::ReleasedComplete => Err(lifecycle_artifact_mismatch()),
    }
}

fn validate_advisory_coverage(advisories: &AdvisoryLedgerV1) -> Result<(), LifecycleFailureV1> {
    if advisories.facts().iter().any(|fact| {
        fact.affected_set_qualification()
            != NormalizedAdvisoryAffectedSetQualificationV1::Qualified
    }) {
        return Err(LifecycleFailureV1::new(
            LifecycleFailureClassV1::IncompleteFactCoverage,
        ));
    }
    Ok(())
}

const fn lifecycle_artifact_mismatch() -> LifecycleFailureV1 {
    LifecycleFailureV1::new(LifecycleFailureClassV1::LifecycleArtifactMismatch)
}
