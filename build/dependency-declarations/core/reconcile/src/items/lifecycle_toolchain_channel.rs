/// Exact release material observed at one Rust channel head.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainChannelHeadV1 {
    material: ToolchainProfileMaterialV1,
    released_at: LifecycleTimestampV1,
    identity_sha256: DigestV1,
}

impl ToolchainChannelHeadV1 {
    #[must_use]
    pub fn new(
        material: ToolchainProfileMaterialV1,
        released_at: LifecycleTimestampV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-channel-head.v1\0");
        hash.digest(material.identity_sha256());
        hash.u64(released_at.unix_seconds());
        Self {
            material,
            released_at,
            identity_sha256: hash.finish(),
        }
    }

    #[must_use]
    pub const fn role(&self) -> ToolchainRoleV1 {
        self.material.role()
    }

    #[must_use]
    pub const fn version(&self) -> RustVersionV1 {
        self.material.version()
    }

    #[must_use]
    pub fn host_triple(&self) -> &str {
        self.material.tools().rustc().host_triple()
    }

    #[must_use]
    pub const fn material_identity_sha256(&self) -> DigestV1 {
        self.material.identity_sha256()
    }

    #[must_use]
    pub const fn material(&self) -> &ToolchainProfileMaterialV1 {
        &self.material
    }

    #[must_use]
    pub const fn released_at(&self) -> LifecycleTimestampV1 {
        self.released_at
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Provenance for one bounded stable, beta, and nightly observation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainChannelSnapshotEvidenceV1 {
    provider_identity_sha256: DigestV1,
    schema_identity_sha256: DigestV1,
    source_snapshot_sha256: DigestV1,
    completeness_receipt_sha256: DigestV1,
}

impl ToolchainChannelSnapshotEvidenceV1 {
    #[must_use]
    pub const fn new(
        provider_identity_sha256: DigestV1,
        schema_identity_sha256: DigestV1,
        source_snapshot_sha256: DigestV1,
        completeness_receipt_sha256: DigestV1,
    ) -> Self {
        Self {
            provider_identity_sha256,
            schema_identity_sha256,
            source_snapshot_sha256,
            completeness_receipt_sha256,
        }
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        hash.digest(self.provider_identity_sha256);
        hash.digest(self.schema_identity_sha256);
        hash.digest(self.source_snapshot_sha256);
        hash.digest(self.completeness_receipt_sha256);
    }
}

/// One atomic observation of the three execution-channel heads for one host.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ToolchainChannelSnapshotV1 {
    stable: ToolchainChannelHeadV1,
    beta: ToolchainChannelHeadV1,
    nightly: ToolchainChannelHeadV1,
    observed_at: LifecycleTimestampV1,
    evidence: ToolchainChannelSnapshotEvidenceV1,
    identity_sha256: DigestV1,
}

impl ToolchainChannelSnapshotV1 {
    pub fn try_new(
        stable: ToolchainChannelHeadV1,
        beta: ToolchainChannelHeadV1,
        nightly: ToolchainChannelHeadV1,
        observed_at: LifecycleTimestampV1,
        evidence: ToolchainChannelSnapshotEvidenceV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if stable.role() != ToolchainRoleV1::QualifiedStableExecution
            || beta.role() != ToolchainRoleV1::BetaShadow
            || nightly.role() != ToolchainRoleV1::NightlyShadow
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainRoleMismatch,
            ));
        }
        if stable.host_triple() != beta.host_triple()
            || stable.host_triple() != nightly.host_triple()
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::ToolchainTargetMismatch,
            ));
        }
        if stable.version() >= beta.version() || beta.version() >= nightly.version() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnsupportedVersionRelation,
            ));
        }
        if [stable.released_at(), beta.released_at(), nightly.released_at()]
            .into_iter()
            .any(|released_at| released_at > observed_at)
        {
            return Err(lifecycle_invalid());
        }
        let mut hash = CanonicalHasherV1::new(b"build.toolchain-channel-snapshot.v1\0");
        for head in [&stable, &beta, &nightly] {
            hash.digest(head.identity_sha256());
        }
        hash.u64(observed_at.unix_seconds());
        evidence.encode(&mut hash);
        Ok(Self {
            stable,
            beta,
            nightly,
            observed_at,
            evidence,
            identity_sha256: hash.finish(),
        })
    }

    pub(crate) fn head(&self, role: ToolchainRoleV1) -> Option<&ToolchainChannelHeadV1> {
        match role {
            ToolchainRoleV1::DeclaredMsrvCompatibility => None,
            ToolchainRoleV1::QualifiedStableExecution => Some(&self.stable),
            ToolchainRoleV1::BetaShadow => Some(&self.beta),
            ToolchainRoleV1::NightlyShadow => Some(&self.nightly),
        }
    }

    #[must_use]
    pub const fn stable(&self) -> &ToolchainChannelHeadV1 {
        &self.stable
    }

    #[must_use]
    pub const fn beta(&self) -> &ToolchainChannelHeadV1 {
        &self.beta
    }

    #[must_use]
    pub const fn nightly(&self) -> &ToolchainChannelHeadV1 {
        &self.nightly
    }

    #[must_use]
    pub const fn observed_at(&self) -> LifecycleTimestampV1 {
        self.observed_at
    }

    #[must_use]
    pub fn host_triple(&self) -> &str {
        self.stable.host_triple()
    }

    #[must_use]
    pub const fn provider_identity_sha256(&self) -> DigestV1 {
        self.evidence.provider_identity_sha256
    }

    #[must_use]
    pub const fn schema_identity_sha256(&self) -> DigestV1 {
        self.evidence.schema_identity_sha256
    }

    #[must_use]
    pub const fn source_snapshot_sha256(&self) -> DigestV1 {
        self.evidence.source_snapshot_sha256
    }

    #[must_use]
    pub const fn completeness_receipt_sha256(&self) -> DigestV1 {
        self.evidence.completeness_receipt_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
