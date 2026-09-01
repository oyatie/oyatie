/// Canonical materialized identifiers for one dependency metadata axis.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyNamedFactSetV1 {
    values: Box<[Box<str>]>,
    identity_sha256: DigestV1,
}

impl DependencyNamedFactSetV1 {
    pub fn try_new(values: Vec<String>) -> Result<Self, LifecycleFailureV1> {
        if values.len() > LifecycleBoundsV1::MAX_DEPENDENCY_NAMED_FACTS {
            return Err(lifecycle_bounds());
        }
        let mut total_bytes = 0_usize;
        let mut canonical = Vec::with_capacity(values.len());
        for value in values {
            let value = lifecycle_identity(value)?;
            total_bytes = total_bytes
                .checked_add(value.len())
                .filter(|total| *total <= LifecycleBoundsV1::MAX_DEPENDENCY_NAMED_FACT_BYTES)
                .ok_or_else(lifecycle_bounds)?;
            canonical.push(value);
        }
        let mut values = canonical;
        values.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        if values.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.dependency-named-fact-set.v1\0");
        hash.u64(lifecycle_len(values.len())?);
        for value in &values {
            lifecycle_hash_string(&mut hash, value)?;
        }
        Ok(Self {
            values: values.into_boxed_slice(),
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn values(&self) -> impl ExactSizeIterator<Item = &str> {
        self.values.iter().map(Box::as_ref)
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Canonical normalized advisory identities affecting one exact release.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyAdvisorySetV1 {
    identities: Box<[DigestV1]>,
    identity_sha256: DigestV1,
}

impl DependencyAdvisorySetV1 {
    pub fn try_new(mut identities: Vec<DigestV1>) -> Result<Self, LifecycleFailureV1> {
        if identities.len() > LifecycleBoundsV1::MAX_DEPENDENCY_ADVISORIES {
            return Err(lifecycle_bounds());
        }
        identities.sort_unstable();
        if identities.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.dependency-advisory-set.v1\0");
        hash.u64(lifecycle_len(identities.len())?);
        for identity in &identities {
            hash.digest(*identity);
        }
        Ok(Self {
            identities: identities.into_boxed_slice(),
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn identities(&self) -> &[DigestV1] {
        &self.identities
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Exact dependency-fact producer qualification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DependencyFactQualificationV1 {
    Candidate {
        observation_receipt_sha256: DigestV1,
    },
    Qualified {
        qualification_receipt_sha256: DigestV1,
    },
}

impl DependencyFactQualificationV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::Candidate {
                observation_receipt_sha256,
            } => {
                hash.tag(0);
                hash.digest(observation_receipt_sha256);
            }
            Self::Qualified {
                qualification_receipt_sha256,
            } => {
                hash.tag(1);
                hash.digest(qualification_receipt_sha256);
            }
        }
    }

    #[must_use]
    pub const fn is_qualified(self) -> bool {
        matches!(self, Self::Qualified { .. })
    }
}

/// Exact license declaration plus maintained-parser evidence.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DependencyLicenseV1 {
    expression: Box<str>,
    evidence_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl DependencyLicenseV1 {
    pub fn try_new(
        expression: impl Into<String>,
        evidence_sha256: DigestV1,
    ) -> Result<Self, LifecycleFailureV1> {
        let expression = lifecycle_identity(expression.into())?;
        let mut hash = CanonicalHasherV1::new(b"build.dependency-license.v1\0");
        lifecycle_hash_string(&mut hash, &expression)?;
        hash.digest(evidence_sha256);
        Ok(Self {
            expression,
            evidence_sha256,
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }

    #[must_use]
    pub const fn evidence_sha256(&self) -> DigestV1 {
        self.evidence_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
