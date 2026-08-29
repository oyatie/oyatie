/// UTC instant represented without parser or locale dependence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdvisoryTimestampV1(u64);

impl AdvisoryTimestampV1 {
    #[must_use]
    pub const fn from_unix_seconds(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn unix_seconds(self) -> u64 {
        self.0
    }
}

/// Source-local lifecycle state of one advisory revision.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AdvisoryLifecycleV1 {
    Active {
        published_at: AdvisoryTimestampV1,
        modified_at: AdvisoryTimestampV1,
    },
    Withdrawn {
        published_at: AdvisoryTimestampV1,
        modified_at: AdvisoryTimestampV1,
        withdrawn_at: AdvisoryTimestampV1,
    },
}

impl AdvisoryLifecycleV1 {
    pub fn try_active(
        published_at: AdvisoryTimestampV1,
        modified_at: AdvisoryTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if published_at > modified_at {
            return Err(lifecycle_invalid());
        }
        Ok(Self::Active {
            published_at,
            modified_at,
        })
    }

    pub fn try_withdrawn(
        published_at: AdvisoryTimestampV1,
        modified_at: AdvisoryTimestampV1,
        withdrawn_at: AdvisoryTimestampV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if published_at > withdrawn_at || withdrawn_at > modified_at {
            return Err(lifecycle_invalid());
        }
        Ok(Self::Withdrawn {
            published_at,
            modified_at,
            withdrawn_at,
        })
    }

    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Active {
                published_at,
                modified_at,
            } => {
                hash.tag(0);
                hash.u64(published_at.unix_seconds());
                hash.u64(modified_at.unix_seconds());
            }
            Self::Withdrawn {
                published_at,
                modified_at,
                withdrawn_at,
            } => {
                hash.tag(1);
                hash.u64(published_at.unix_seconds());
                hash.u64(modified_at.unix_seconds());
                hash.u64(withdrawn_at.unix_seconds());
            }
        }
    }

    #[must_use]
    pub const fn modified_at(self) -> AdvisoryTimestampV1 {
        match self {
            Self::Active { modified_at, .. } | Self::Withdrawn { modified_at, .. } => modified_at,
        }
    }

    #[must_use]
    pub const fn is_withdrawn(self) -> bool {
        matches!(self, Self::Withdrawn { .. })
    }
}

/// Whether one source record supplies a complete affected-package set.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AdvisoryAffectedSetCompletenessV1 {
    ReferenceOnly,
    Complete,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum AdvisoryAffectedSetStateV1 {
    ReferenceOnly { evidence_sha256: DigestV1 },
    Complete { claims: Box<[CargoAdvisoryClaimV1]> },
}

/// Canonical affected-package claims or an explicit reference-only boundary.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AdvisoryAffectedSetV1 {
    state: AdvisoryAffectedSetStateV1,
    identity_sha256: DigestV1,
}

impl AdvisoryAffectedSetV1 {
    #[must_use]
    pub fn reference_only(evidence_sha256: DigestV1) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.advisory-affected-set.v1\0");
        hash.tag(0);
        hash.digest(evidence_sha256);
        Self {
            state: AdvisoryAffectedSetStateV1::ReferenceOnly { evidence_sha256 },
            identity_sha256: hash.finish(),
        }
    }

    pub fn try_complete(
        mut claims: Vec<CargoAdvisoryClaimV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        if claims.is_empty()
            || claims.len() > LifecycleBoundsV1::MAX_ADVISORY_PACKAGES_PER_RECORD
        {
            return Err(lifecycle_bounds());
        }
        claims.sort_by_key(|claim| claim.package().identity_sha256());
        if claims.windows(2).any(|pair| pair[0].package() == pair[1].package()) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.advisory-affected-set.v1\0");
        hash.tag(1);
        hash.u64(lifecycle_len(claims.len())?);
        for claim in &claims {
            hash.digest(claim.identity_sha256());
        }
        Ok(Self {
            state: AdvisoryAffectedSetStateV1::Complete {
                claims: claims.into_boxed_slice(),
            },
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub const fn completeness(&self) -> AdvisoryAffectedSetCompletenessV1 {
        match &self.state {
            AdvisoryAffectedSetStateV1::ReferenceOnly { .. } => {
                AdvisoryAffectedSetCompletenessV1::ReferenceOnly
            }
            AdvisoryAffectedSetStateV1::Complete { .. } => {
                AdvisoryAffectedSetCompletenessV1::Complete
            }
        }
    }

    #[must_use]
    pub fn claims(&self) -> Option<&[CargoAdvisoryClaimV1]> {
        match &self.state {
            AdvisoryAffectedSetStateV1::ReferenceOnly { .. } => None,
            AdvisoryAffectedSetStateV1::Complete { claims } => Some(claims),
        }
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// One immutable source revision of an advisory record.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AdvisoryRecordV1 {
    source: AdvisoryRecordSourceV1,
    primary: AdvisoryIdentifierV1,
    aliases: Box<[AdvisoryIdentifierV1]>,
    lifecycle: AdvisoryLifecycleV1,
    affected: AdvisoryAffectedSetV1,
    content_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl AdvisoryRecordV1 {
    pub fn try_new(
        source: AdvisoryRecordSourceV1,
        primary: AdvisoryIdentifierV1,
        mut aliases: Vec<AdvisoryIdentifierV1>,
        lifecycle: AdvisoryLifecycleV1,
        affected: AdvisoryAffectedSetV1,
        content_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        if !source
            .authority()
            .matches_primary_namespace(primary.namespace())
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::AdvisorySourceMismatch,
            ));
        }
        if aliases.len() > LifecycleBoundsV1::MAX_ADVISORY_ALIASES_PER_RECORD {
            return Err(lifecycle_bounds());
        }
        aliases.sort();
        if aliases.binary_search(&primary).is_ok()
            || aliases.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let mut value = Self {
            source,
            primary,
            aliases: aliases.into_boxed_slice(),
            lifecycle,
            affected,
            content_sha256,
            identity_sha256: DigestV1::from_bytes([0; 32]),
        };
        let mut hash = CanonicalHasherV1::new(b"build.advisory-record.v1\0");
        value.encode_fields(&mut hash)?;
        value.identity_sha256 = hash.finish();
        Ok(value)
    }

    fn encode_fields(&self, hash: &mut CanonicalHasherV1) -> Result<(), LifecycleFailureV1> {
        hash.digest(self.source.identity_sha256());
        hash.digest(self.primary.identity_sha256());
        hash.u64(lifecycle_len(self.aliases.len())?);
        for alias in &self.aliases {
            hash.digest(alias.identity_sha256());
        }
        self.lifecycle.encode(hash);
        hash.digest(self.affected.identity_sha256());
        hash.digest(self.content_sha256);
        Ok(())
    }

    pub(crate) fn identifiers(
        &self,
    ) -> impl Iterator<Item = &AdvisoryIdentifierV1> {
        std::iter::once(&self.primary).chain(self.aliases.iter())
    }

    pub(crate) fn same_payload(&self, other: &Self) -> bool {
        self.primary == other.primary
            && self.aliases == other.aliases
            && self.lifecycle == other.lifecycle
            && self.affected == other.affected
            && self.content_sha256 == other.content_sha256
    }

    #[must_use]
    pub const fn source(&self) -> &AdvisoryRecordSourceV1 {
        &self.source
    }

    #[must_use]
    pub const fn primary(&self) -> &AdvisoryIdentifierV1 {
        &self.primary
    }

    #[must_use]
    pub fn aliases(&self) -> &[AdvisoryIdentifierV1] {
        &self.aliases
    }

    #[must_use]
    pub const fn lifecycle(&self) -> AdvisoryLifecycleV1 {
        self.lifecycle
    }

    #[must_use]
    pub const fn affected(&self) -> &AdvisoryAffectedSetV1 {
        &self.affected
    }

    #[must_use]
    pub const fn content_sha256(&self) -> DigestV1 {
        self.content_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
