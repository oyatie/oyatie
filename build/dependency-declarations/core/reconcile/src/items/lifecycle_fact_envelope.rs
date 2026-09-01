/// Origin of one admitted repository fact.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FactEvidenceClassV1 {
    Declared = 0,
    Proven = 1,
    Observed = 2,
    Inferred = 3,
}

/// Canonical non-empty evidence-class set.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FactEvidenceClassesV1 {
    values: Box<[FactEvidenceClassV1]>,
    identity_sha256: DigestV1,
}

impl FactEvidenceClassesV1 {
    pub fn try_new(
        mut values: Vec<FactEvidenceClassV1>,
    ) -> Result<Self, LifecycleFailureV1> {
        if values.is_empty() || values.len() > 4 {
            return Err(lifecycle_bounds());
        }
        values.sort_unstable();
        if values.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::DuplicateIdentity,
            ));
        }
        let mut hash = CanonicalHasherV1::new(b"build.fact-evidence-classes.v1\0");
        hash.u64(lifecycle_len(values.len())?);
        for value in &values {
            hash.tag(*value as u8);
        }
        Ok(Self {
            values: values.into_boxed_slice(),
            identity_sha256: hash.finish(),
        })
    }

    #[must_use]
    pub fn values(&self) -> &[FactEvidenceClassV1] {
        &self.values
    }

    #[must_use]
    pub fn contains(&self, value: FactEvidenceClassV1) -> bool {
        self.values.binary_search(&value).is_ok()
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}

/// Strength of one fact independent of its origin and coverage.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum FactCertaintyV1 {
    Exact = 0,
    Conservative = 1,
    Speculative = 2,
}

/// Explicit completeness boundary for one fact set.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FactCoverageV1 {
    CompleteForScope {
        scope_sha256: DigestV1,
        exclusions_sha256: DigestV1,
    },
    Partial {
        scope_sha256: DigestV1,
        evidence_sha256: DigestV1,
    },
    Excluded {
        scope_sha256: DigestV1,
        exclusion_sha256: DigestV1,
    },
    Unknown {
        scope_sha256: DigestV1,
        reason_sha256: DigestV1,
    },
}

impl FactCoverageV1 {
    fn encode(self, hash: &mut CanonicalHasherV1) {
        match self {
            Self::CompleteForScope {
                scope_sha256,
                exclusions_sha256,
            } => {
                hash.tag(0);
                hash.digest(scope_sha256);
                hash.digest(exclusions_sha256);
            }
            Self::Partial {
                scope_sha256,
                evidence_sha256,
            } => {
                hash.tag(1);
                hash.digest(scope_sha256);
                hash.digest(evidence_sha256);
            }
            Self::Excluded {
                scope_sha256,
                exclusion_sha256,
            } => {
                hash.tag(2);
                hash.digest(scope_sha256);
                hash.digest(exclusion_sha256);
            }
            Self::Unknown {
                scope_sha256,
                reason_sha256,
            } => {
                hash.tag(3);
                hash.digest(scope_sha256);
                hash.digest(reason_sha256);
            }
        }
    }

    #[must_use]
    pub const fn is_complete_for_scope(self) -> bool {
        matches!(self, Self::CompleteForScope { .. })
    }
}

/// Proof envelope carried by one normalized fact graph.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FactEnvelopeV1 {
    evidence: FactEvidenceClassesV1,
    certainty: FactCertaintyV1,
    coverage: FactCoverageV1,
    temporal: FactTemporalIdentityV1,
    qualification_sha256: DigestV1,
    derivation_sha256: DigestV1,
    identity_sha256: DigestV1,
}

impl FactEnvelopeV1 {
    #[must_use]
    pub fn new(
        evidence: FactEvidenceClassesV1,
        certainty: FactCertaintyV1,
        coverage: FactCoverageV1,
        temporal: FactTemporalIdentityV1,
        qualification_sha256: DigestV1,
        derivation_sha256: DigestV1,
    ) -> Self {
        let mut hash = CanonicalHasherV1::new(b"build.fact-envelope.v1\0");
        hash.digest(evidence.identity_sha256());
        hash.tag(certainty as u8);
        coverage.encode(&mut hash);
        hash.digest(temporal.identity_sha256());
        hash.digest(qualification_sha256);
        hash.digest(derivation_sha256);
        Self {
            evidence,
            certainty,
            coverage,
            temporal,
            qualification_sha256,
            derivation_sha256,
            identity_sha256: hash.finish(),
        }
    }

    pub fn require_safe(
        &self,
        now: LifecycleTimestampV1,
    ) -> Result<(), LifecycleFailureV1> {
        if self.evidence.contains(FactEvidenceClassV1::Inferred)
            || self.certainty == FactCertaintyV1::Speculative
        {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::UnsupportedFactEvidence,
            ));
        }
        if !self.coverage.is_complete_for_scope() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::IncompleteFactCoverage,
            ));
        }
        if now < self.temporal.observed_at() || now > self.temporal.fresh_until() {
            return Err(LifecycleFailureV1::new(
                LifecycleFailureClassV1::StaleFact,
            ));
        }
        Ok(())
    }

    #[must_use]
    pub const fn evidence(&self) -> &FactEvidenceClassesV1 {
        &self.evidence
    }

    #[must_use]
    pub const fn certainty(&self) -> FactCertaintyV1 {
        self.certainty
    }

    #[must_use]
    pub const fn coverage(&self) -> FactCoverageV1 {
        self.coverage
    }

    #[must_use]
    pub const fn temporal(&self) -> &FactTemporalIdentityV1 {
        &self.temporal
    }

    #[must_use]
    pub const fn qualification_sha256(&self) -> DigestV1 {
        self.qualification_sha256
    }

    #[must_use]
    pub const fn derivation_sha256(&self) -> DigestV1 {
        self.derivation_sha256
    }

    #[must_use]
    pub const fn identity_sha256(&self) -> DigestV1 {
        self.identity_sha256
    }
}
