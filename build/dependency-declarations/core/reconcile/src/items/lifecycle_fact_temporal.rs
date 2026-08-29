/// Immutable repository and producer coordinates for one fact set.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FactTemporalScopeV1 {
    repository_identity: Box<str>,
    repository_revision_sha256: DigestV1,
    repository_snapshot_sha256: DigestV1,
    configuration_sha256: DigestV1,
    toolchain_sha256: DigestV1,
    producer_sha256: DigestV1,
    schema_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl FactTemporalScopeV1 {
    pub fn try_new(
        repository_identity: impl Into<String>,
        repository_revision_sha256: DigestV1,
        repository_snapshot_sha256: DigestV1,
        configuration_sha256: DigestV1,
        toolchain_sha256: DigestV1,
        producer_sha256: DigestV1,
        schema_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let repository_identity = lifecycle_identity(repository_identity.into())?;
        let mut hash = CanonicalHasherV1::new(b"build.fact-temporal-scope.v1\0");
        lifecycle_hash_string(&mut hash, &repository_identity)?;
        hash.digest(repository_revision_sha256);
        hash.digest(repository_snapshot_sha256);
        hash.digest(configuration_sha256);
        hash.digest(toolchain_sha256);
        hash.digest(producer_sha256);
        hash.digest(schema_sha256);
        Ok(Self {
            repository_identity,
            repository_revision_sha256,
            repository_snapshot_sha256,
            configuration_sha256,
            toolchain_sha256,
            producer_sha256,
            schema_sha256,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn repository_identity(&self) -> &str {
        &self.repository_identity
    }

    #[must_use]
    pub const fn repository_revision_sha256(&self) -> DigestV1 {
        self.repository_revision_sha256
    }

    #[must_use]
    pub const fn repository_snapshot_sha256(&self) -> DigestV1 {
        self.repository_snapshot_sha256
    }

    #[must_use]
    pub const fn configuration_sha256(&self) -> DigestV1 {
        self.configuration_sha256
    }

    #[must_use]
    pub const fn toolchain_sha256(&self) -> DigestV1 {
        self.toolchain_sha256
    }

    #[must_use]
    pub const fn producer_sha256(&self) -> DigestV1 {
        self.producer_sha256
    }

    #[must_use]
    pub const fn schema_sha256(&self) -> DigestV1 {
        self.schema_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Observation interval bound to immutable fact coordinates.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FactTemporalIdentityV1 {
    scope: FactTemporalScopeV1,
    observed_at: LifecycleTimestampV1,
    fresh_until: LifecycleTimestampV1,
    identity_sha256: DigestV1,
}

impl FactTemporalIdentityV1 {
    pub fn try_new(
        scope: FactTemporalScopeV1,
        observed_at: LifecycleTimestampV1,
        fresh_until: LifecycleTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if observed_at > fresh_until {
            return Err(lifecycle_invalid());
        }
        let mut hash = CanonicalHasherV1::new(b"build.fact-temporal-identity.v1\0");
        hash.digest(scope.identity_sha256());
        hash.u64(observed_at.unix_seconds());
        hash.u64(fresh_until.unix_seconds());
        Ok(Self {
            scope,
            observed_at,
            fresh_until,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn scope(&self) -> &FactTemporalScopeV1 {
        &self.scope
    }

    #[must_use]
    pub const fn observed_at(&self) -> LifecycleTimestampV1 {
        self.observed_at
    }

    #[must_use]
    pub const fn fresh_until(&self) -> LifecycleTimestampV1 {
        self.fresh_until
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
